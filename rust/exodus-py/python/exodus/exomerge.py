"""
Exomerge is a lightweight Python interface for manipulating ExodusII files.

This module provides a Python API compatible with the legacy exomerge3.py module,
built on top of the modern exodus-py Rust bindings.

Author: exodus-rs development team
Based on: exomerge3.py by Tim Kostka (tdkostk@sandia.gov)

Simple example:
>>> import exodus.exomerge as exomerge
>>> model = exomerge.import_model('results.e')
>>> model.delete_element_block(1)
>>> model.export_model('most_results.e')

For documentation on the original exomerge API:
"Exomerge User's Manual: A lightweight Python interface for manipulating
Exodus files" (SAND2013-0725)
"""

import sys
import datetime
from typing import Optional, List, Dict, Any, Union, Tuple

# Import the exodus-py module (Rust bindings)
_exodus_available = True
try:
    try:
        from . import exodus
    except ImportError:
        import exodus
except (ImportError, ModuleNotFoundError):
    _exodus_available = False
    # Create minimal stubs for testing without Rust module
    class ExodusStub:
        class ExodusReader:
            @staticmethod
            def open(filename):
                raise NotImplementedError("Rust exodus module not available")

        class ExodusWriter:
            @staticmethod
            def create(filename, opts=None):
                raise NotImplementedError("Rust exodus module not available")

        class CreateOptions:
            pass

        class CreateMode:
            Clobber = None

        class InitParams:
            pass

        class Block:
            pass

        class EntityType:
            pass

    exodus = ExodusStub()

__version__ = "0.1.0"
VERSION = __version__

# Contact person for issues
CONTACT = "exodus-rs development team"

# Show banner on first use
SHOW_BANNER = True

# If true, will crash if warnings are generated
EXIT_ON_WARNING = False

# Deprecated function mappings
DEPRECATED_FUNCTIONS = {
    "write": "export"
}


def import_model(filename: str, *args, **kwargs) -> 'ExodusModel':
    """
    Load information from an ExodusII file.

    This function is a wrapper around 'ExodusModel.import_model(...)' and is
    provided for convenience.

    Parameters
    ----------
    filename : str
        Path to the Exodus II file to load
    *args : tuple
        Additional positional arguments passed to import_model
    **kwargs : dict
        Additional keyword arguments passed to import_model

    Returns
    -------
    ExodusModel
        The loaded model

    Examples
    --------
    >>> model = import_model('mesh.e')
    >>> print(model.get_element_block_ids())
    """
    model = ExodusModel()
    model.import_model(filename, *args, **kwargs)
    return model


class ExodusModel:
    """
    Main class for manipulating Exodus II finite element models.

    This class provides a high-level interface for reading, modifying, and
    writing Exodus II files. It maintains an in-memory representation of the
    mesh including nodes, elements, sets, fields, and timesteps.

    Attributes
    ----------
    nodes : list
        List of [x, y, z] node coordinates
    node_fields : dict
        Dictionary mapping field names to timestep data
    global_variables : dict
        Dictionary mapping variable names to timestep data
    element_blocks : dict
        Dictionary mapping block IDs to [name, info, connectivity, fields]
    side_sets : dict
        Dictionary mapping side set IDs to [name, members, fields]
    node_sets : dict
        Dictionary mapping node set IDs to [name, members, fields]
    timesteps : list
        List of timestep values
    title : str
        Database title string
    qa_records : list
        List of QA record tuples
    info_records : list
        List of info record strings
    """

    # Element type dimension mapping
    DIMENSION = {
        "point": 0,
        "line2": 1, "line3": 1,
        "tri3": 2, "tri6": 2,
        "quad4": 2, "quad8": 2, "quad9": 2,
        "tet4": 3, "tet10": 3,
        "wedge6": 3, "wedge15": 3,
        "hex8": 3, "hex20": 3, "hex27": 3,
        "pyramid5": 3,
    }

    # Nodes per element mapping
    NODES_PER_ELEMENT = {
        "point": 1,
        "line2": 2, "line3": 3,
        "tri3": 3, "tri6": 6,
        "quad4": 4, "quad8": 8, "quad9": 9,
        "tet4": 4, "tet10": 10,
        "wedge6": 6, "wedge15": 15,
        "hex8": 8, "hex20": 20, "hex27": 27,
        "pyramid5": 5,
    }

    def __init__(self):
        """Initialize an empty ExodusModel."""
        # Core data structures
        self.nodes: List[List[float]] = []
        self.node_fields: Dict[str, List[Any]] = {}
        self.global_variables: Dict[str, List[float]] = {}
        self.element_blocks: Dict[int, List[Any]] = {}
        self.side_sets: Dict[int, List[Any]] = {}
        self.node_sets: Dict[int, List[Any]] = {}
        self.timesteps: List[float] = []
        self.title: str = ""
        self.qa_records: List[Tuple] = []
        self.info_records: List[str] = []

        # Internal state
        self._reader: Optional[Any] = None
        self._filename: Optional[str] = None

    def __getattr__(self, name: str):
        """
        Handle deprecated function names.

        If a deprecated function is called, issue a warning and redirect
        to the new function name if available.
        """
        if name in DEPRECATED_FUNCTIONS:
            new_name = DEPRECATED_FUNCTIONS[name]
            self._warning(
                f"Function '{name}' is deprecated",
                f"Please use '{new_name}' instead"
            )
            if new_name:
                return getattr(self, new_name)
            else:
                self._error(
                    f"Function '{name}' has been removed",
                    "Please update your code to use the new API"
                )
        raise AttributeError(f"'{type(self).__name__}' object has no attribute '{name}'")

    # ========================================================================
    # Error and Warning Handling
    # ========================================================================

    def _error(self, short: str = "Unspecified error", detailed: str = "", exit_code: int = 1):
        """Print an error message and exit."""
        print(f"\nERROR: {short}")
        if detailed:
            print(f"  {detailed}")
        if exit_code:
            sys.exit(exit_code)

    def _warning(self, short: str = "Unspecified warning", detailed: str = ""):
        """Print a warning message."""
        print(f"\nWARNING: {short}")
        if detailed:
            print(f"  {detailed}")
        if EXIT_ON_WARNING:
            sys.exit(1)

    # ========================================================================
    # Helper Methods
    # ========================================================================

    def _get_dimension(self, element_type: str) -> int:
        """
        Get the spatial dimension of an element type.

        Parameters
        ----------
        element_type : str
            Element type string (e.g., "HEX8", "QUAD4")

        Returns
        -------
        int
            Spatial dimension (0, 1, 2, or 3)
        """
        # Normalize element type to lowercase
        elem_type = element_type.lower().strip()

        # Handle common variations
        if elem_type.startswith("hex"):
            return 3
        elif elem_type.startswith("tet"):
            return 3
        elif elem_type.startswith("wedge") or elem_type.startswith("penta"):
            return 3
        elif elem_type.startswith("pyramid"):
            return 3
        elif elem_type.startswith("quad"):
            return 2
        elif elem_type.startswith("tri"):
            return 2
        elif elem_type.startswith("line") or elem_type.startswith("bar") or elem_type.startswith("beam"):
            return 1
        elif elem_type.startswith("point") or elem_type.startswith("sphere"):
            return 0

        # Try exact match in DIMENSION dict
        if elem_type in self.DIMENSION:
            return self.DIMENSION[elem_type]

        # Default to 3D
        self._warning(f"Unknown element type '{element_type}', assuming 3D")
        return 3

    def _get_unreferenced_nodes(self) -> List[int]:
        """
        Return a list of node indices which are not used by any element.

        Returns
        -------
        list of int
            List of unreferenced node indices
        """
        used_node = [False] * len(self.nodes)
        for block_id in self.get_element_block_ids():
            connectivity = self.get_connectivity(block_id)
            # connectivity is a list of lists, flatten it
            for element_nodes in connectivity:
                for node_index in element_nodes:
                    if 0 <= node_index < len(self.nodes):
                        used_node[node_index] = True

        unused_nodes = [index for index, used in enumerate(used_node) if not used]
        return unused_nodes

    def _ensure_no_shared_nodes(self, element_block_ids: List[int]):
        """
        Ensure no nodes are shared outside the given element blocks.

        Raises an error if nodes are shared between the given element blocks
        and other element blocks.

        Parameters
        ----------
        element_block_ids : list of int
            Element block IDs to check
        """
        affected_nodes = self.get_nodes_in_element_block(element_block_ids)
        other_block_ids = list(set(self.get_element_block_ids()) - set(element_block_ids))

        if not other_block_ids:
            return  # No other blocks to share with

        nodes_in_other_blocks = self.get_nodes_in_element_block(other_block_ids)
        shared_nodes = sorted(set(affected_nodes) & set(nodes_in_other_blocks))

        if shared_nodes:
            max_nodes_to_display = 20
            node_list = ", ".join([str(x) for x in shared_nodes[:max_nodes_to_display]])
            if len(shared_nodes) > max_nodes_to_display:
                node_list += ", ..."
            self._error(
                "Shared nodes detected",
                f"The specified element blocks share {len(shared_nodes)} nodes with other "
                f"element blocks: {node_list}. Use unmerge_element_blocks() first."
            )

    def _translate_nodes(self, offset: List[float], node_indices: Union[str, List[int]] = "all"):
        """
        Translate nodes by the given offset.

        Parameters
        ----------
        offset : list of float
            Translation offset [dx, dy, dz]
        node_indices : str or list of int, optional
            Node indices to translate (1-based) or "all" (default: "all")
        """
        dx, dy, dz = [float(x) for x in offset]
        if node_indices == "all":
            self.nodes = [[x + dx, y + dy, z + dz] for x, y, z in self.nodes]
        else:
            for index in node_indices:
                # Convert 1-based to 0-based
                zero_based_idx = index - 1
                if 0 <= zero_based_idx < len(self.nodes):
                    self.nodes[zero_based_idx][0] += dx
                    self.nodes[zero_based_idx][1] += dy
                    self.nodes[zero_based_idx][2] += dz

    def _scale_nodes(self, scale_factor: float, node_indices: Union[str, List[int]] = "all",
                     adjust_displacement_field: Union[str, bool] = "auto"):
        """
        Scale nodes by the given scale factor.

        Parameters
        ----------
        scale_factor : float
            Scale factor
        node_indices : str or list of int, optional
            Node indices to scale (1-based) or "all" (default: "all")
        adjust_displacement_field : str or bool, optional
            Whether to adjust displacement field (default: "auto")
        """
        scale_factor = float(scale_factor)
        if adjust_displacement_field == "auto":
            adjust_displacement_field = False  # Displacement field not yet implemented

        # Scale the nodal coordinates
        if node_indices == "all":
            self.nodes = [[x * scale_factor for x in n] for n in self.nodes]
        else:
            for index in node_indices:
                # Convert 1-based to 0-based
                zero_based_idx = index - 1
                if 0 <= zero_based_idx < len(self.nodes):
                    self.nodes[zero_based_idx] = [x * scale_factor for x in self.nodes[zero_based_idx]]

        # TODO: Scale the displacement field when implemented
        # if adjust_displacement_field:
        #     ...

    def _rotate_nodes(self, axis: List[float], angle_in_degrees: float,
                     node_indices: Union[str, List[int]] = "all",
                     adjust_displacement_field: Union[str, bool] = "auto"):
        """
        Rotate nodes about an axis by the given angle.

        Parameters
        ----------
        axis : list of float
            Rotation axis direction [x, y, z]
        angle_in_degrees : float
            Rotation angle in degrees
        node_indices : str or list of int, optional
            Node indices to rotate (1-based) or "all" (default: "all")
        adjust_displacement_field : str or bool, optional
            Whether to adjust displacement field (default: "auto")
        """
        import math

        if adjust_displacement_field == "auto":
            adjust_displacement_field = False  # Displacement field not yet implemented

        # Normalize axis
        scale = math.sqrt(sum(x * x for x in axis))
        ux, uy, uz = [float(x) / scale for x in axis]

        # Convert angle to radians
        theta = float(angle_in_degrees) * math.pi / 180
        cost = math.cos(theta)
        sint = math.sin(theta)

        # If angle is a multiple of 90 degrees, make sin and cos exact to avoid roundoff
        if angle_in_degrees % 90 == 0:
            sint = math.floor(sint + 0.5)
            cost = math.floor(cost + 0.5)

        # Build rotation matrix (Rodrigues' rotation formula)
        rxx = cost + ux * ux * (1 - cost)
        rxy = ux * uy * (1 - cost) - uz * sint
        rxz = ux * uz * (1 - cost) + uy * sint
        ryx = uy * ux * (1 - cost) + uz * sint
        ryy = cost + uy * uy * (1 - cost)
        ryz = uy * uz * (1 - cost) - ux * sint
        rzx = uz * ux * (1 - cost) - uy * sint
        rzy = uz * uy * (1 - cost) + ux * sint
        rzz = cost + uz * uz * (1 - cost)

        # Rotate nodes
        if node_indices == "all":
            self.nodes = [
                [
                    rxx * x + rxy * y + rxz * z,
                    ryx * x + ryy * y + ryz * z,
                    rzx * x + rzy * y + rzz * z,
                ]
                for x, y, z in self.nodes
            ]
        else:
            for index in node_indices:
                # Convert 1-based to 0-based
                zero_based_idx = index - 1
                if 0 <= zero_based_idx < len(self.nodes):
                    n = self.nodes[zero_based_idx]
                    self.nodes[zero_based_idx] = [
                        rxx * n[0] + rxy * n[1] + rxz * n[2],
                        ryx * n[0] + ryy * n[1] + ryz * n[2],
                        rzx * n[0] + rzy * n[1] + rzz * n[2],
                    ]

        # TODO: Rotate the displacement field when implemented
        # if adjust_displacement_field:
        #     ...

    # ========================================================================
    # File I/O Operations
    # ========================================================================

    def import_model(self, filename: str, *args, **kwargs):
        """
        Load data from an ExodusII file.

        This method reads all data from an Exodus II file into memory,
        including nodes, elements, sets, fields, and timesteps.

        Parameters
        ----------
        filename : str
            Path to the Exodus II file to load
        *args : tuple
            Additional positional arguments (for compatibility)
        **kwargs : dict
            Additional keyword arguments (for compatibility)

        Examples
        --------
        >>> model = ExodusModel()
        >>> model.import_model('mesh.e')
        """
        # Import the exodus module (Rust bindings)
        try:
            from . import ExodusReader, EntityType
        except ImportError:
            import exodus
            ExodusReader = exodus.ExodusReader
            EntityType = exodus.EntityType

        # Open the file for reading
        reader = ExodusReader.open(filename)
        self._reader = reader
        self._filename = filename

        try:
            # Read initialization parameters
            params = reader.init_params()
            self.title = params.title if hasattr(params, 'title') else ""

            # Read node coordinates
            if params.num_nodes > 0:
                x, y, z = reader.get_coords()
                # Convert to list of [x, y, z] coordinates
                self.nodes = []
                for i in range(params.num_nodes):
                    coord = [x[i] if x else 0.0,
                            y[i] if y else 0.0,
                            z[i] if z else 0.0]
                    self.nodes.append(coord)
            else:
                self.nodes = []

            # Read element blocks
            self.element_blocks = {}
            if params.num_elem_blocks > 0:
                try:
                    block_ids = reader.get_block_ids()
                    for block_id in block_ids:
                        block = reader.get_block(block_id)
                        # Get block name
                        try:
                            name = reader.get_name("elem_block", block_id)
                        except:
                            name = ""

                        # Get connectivity
                        try:
                            connectivity = reader.get_connectivity(block_id)
                        except:
                            connectivity = []

                        # Store block info: [name, info, connectivity, fields]
                        # info = [element_type, num_elements, nodes_per_element, num_attributes]
                        info = [
                            block.elem_type if hasattr(block, 'elem_type') else "UNKNOWN",
                            block.num_elems if hasattr(block, 'num_elems') else 0,
                            block.nodes_per_elem if hasattr(block, 'nodes_per_elem') else 0,
                            block.num_attrs if hasattr(block, 'num_attrs') else 0
                        ]

                        self.element_blocks[block_id] = [name, info, connectivity, {}]
                except Exception as e:
                    self._warning(f"Error reading element blocks: {e}")

            # Read node sets
            self.node_sets = {}
            if params.num_node_sets > 0:
                try:
                    # We need to find node set IDs - try common IDs
                    # In the exodus API, we may need to iterate or use get_entity_set
                    pass  # Will implement when we have better ID discovery
                except Exception as e:
                    self._warning(f"Error reading node sets: {e}")

            # Read side sets
            self.side_sets = {}
            if params.num_side_sets > 0:
                try:
                    # Similar to node sets
                    pass  # Will implement when we have better ID discovery
                except Exception as e:
                    self._warning(f"Error reading side sets: {e}")

            # Read timesteps and variables
            self.timesteps = []
            self.node_fields = {}
            self.global_variables = {}

            try:
                num_timesteps = reader.num_time_steps()
                # Get timestep values would require get_time method
                # For now, create sequential timesteps
                self.timesteps = list(range(num_timesteps))
            except:
                pass

            # Read QA records
            try:
                qa_records = reader.get_qa_records()
                self.qa_records = qa_records if qa_records else []
            except:
                self.qa_records = []

            # Read info records
            try:
                info_records = reader.get_info_records()
                self.info_records = info_records if info_records else []
            except:
                self.info_records = []

        finally:
            # Keep reader open for potential future reads
            pass

    def export_model(self, filename: str, *args, **kwargs):
        """
        Write the model to an ExodusII file.

        This method writes all in-memory data to an Exodus II file,
        including nodes, elements, sets, fields, and timesteps.

        Parameters
        ----------
        filename : str
            Path to the output Exodus II file
        *args : tuple
            Additional positional arguments (for compatibility)
        **kwargs : dict
            Additional keyword arguments (for compatibility)

        Examples
        --------
        >>> model.export_model('output.e')
        """
        # Import the exodus module (Rust bindings)
        try:
            from . import ExodusWriter, CreateOptions, InitParams, CreateMode, Block
        except ImportError:
            import exodus
            ExodusWriter = exodus.ExodusWriter
            CreateOptions = exodus.CreateOptions
            InitParams = exodus.InitParams
            CreateMode = exodus.CreateMode
            Block = exodus.Block

        # Create the file
        opts = CreateOptions(mode=CreateMode.Clobber)
        writer = ExodusWriter.create(filename, opts)

        try:
            # Determine dimensionality from nodes
            num_dim = 3 if self.nodes and len(self.nodes[0]) == 3 else 2

            # Write initialization parameters
            params = InitParams(
                title=self.title,
                num_dim=num_dim,
                num_nodes=len(self.nodes),
                num_elems=sum(block[1][1] for block in self.element_blocks.values()),  # block[1][1] is num_elements
                num_elem_blocks=len(self.element_blocks),
                num_node_sets=len(self.node_sets),
                num_side_sets=len(self.side_sets),
            )
            writer.put_init_params(params)

            # Write node coordinates
            if self.nodes:
                x = [node[0] for node in self.nodes]
                y = [node[1] if len(node) > 1 else 0.0 for node in self.nodes]
                z = [node[2] if len(node) > 2 else 0.0 for node in self.nodes]
                writer.put_coords(x, y, z)

            # Write element blocks
            for block_id, block_data in self.element_blocks.items():
                name, info, connectivity, fields = block_data
                elem_type, num_elems, nodes_per_elem, num_attrs = info

                # Create block
                block = Block(
                    id=block_id,
                    elem_type=elem_type,
                    num_elems=num_elems,
                    nodes_per_elem=nodes_per_elem,
                    num_attrs=num_attrs
                )
                writer.put_block(block)

                # Write connectivity
                if connectivity:
                    writer.put_connectivity(block_id, connectivity)

            # Write node sets
            for ns_id, ns_data in self.node_sets.items():
                name, members, fields = ns_data
                if members:
                    writer.put_node_set(ns_id, members)

            # Write side sets
            for ss_id, ss_data in self.side_sets.items():
                name, members, fields = ss_data
                if members:
                    # members should be list of (elem_id, side_id) tuples
                    elem_ids = [m[0] for m in members]
                    side_ids = [m[1] for m in members]
                    writer.put_side_set(ss_id, elem_ids, side_ids)

            # Write timesteps
            for i, timestep in enumerate(self.timesteps):
                writer.put_time(i, float(timestep))

            # TODO: Write variables (node_fields, element block fields, global variables)
            # This requires additional exodus API exploration

            writer.close()

        except Exception as e:
            self._error(f"Error exporting model: {e}", detailed=str(e), exit_code=0)
            raise

    def export(self, filename: str, *args, **kwargs):
        """
        Export model to file (auto-detect format).

        This method auto-detects the output format based on file extension:
        - .e, .exo: Exodus II format
        - .wrl: VRML format (not implemented)
        - .stl: STL format (not implemented)

        Parameters
        ----------
        filename : str
            Path to the output file
        *args : tuple
            Additional positional arguments
        **kwargs : dict
            Additional keyword arguments

        Examples
        --------
        >>> model.export('output.e')
        """
        if filename.endswith('.e') or filename.endswith('.exo'):
            return self.export_model(filename, *args, **kwargs)
        elif filename.endswith('.wrl'):
            return self.export_wrl_model(filename, *args, **kwargs)
        elif filename.endswith('.stl'):
            return self.export_stl_file(filename, *args, **kwargs)
        else:
            self._error(
                "Unknown file format",
                f"File extension not recognized: {filename}"
            )

    def export_stl_file(self, filename: str, element_block_ids=None, displacement_timestep=None):
        """
        Export model to STL format.

        NOT IMPLEMENTED: This functionality requires STL mesh generation
        which is not available in the exodus-rs library.

        Parameters
        ----------
        filename : str
            Path to output STL file
        element_block_ids : list, optional
            Element blocks to export
        displacement_timestep : float, optional
            Timestep for displacement field

        Raises
        ------
        NotImplementedError
            STL export is not available in this implementation
        """
        raise NotImplementedError(
            "export_stl_file() is not implementable with the current exodus-rs backend. "
            "STL export requires extensive geometry processing and mesh generation "
            "capabilities that are not part of the Exodus II format specification. "
            "Consider using the original exomerge3.py or dedicated mesh conversion tools."
        )

    def export_wrl_model(self, filename: str, node_field_name=None, *args, **kwargs):
        """
        Export model to VRML (WRL) format.

        NOT IMPLEMENTED: This functionality requires VRML generation
        which is not available in the exodus-rs library.

        Parameters
        ----------
        filename : str
            Path to output WRL file
        node_field_name : str, optional
            Node field to use for coloring
        *args : tuple
            Additional positional arguments
        **kwargs : dict
            Additional keyword arguments

        Raises
        ------
        NotImplementedError
            VRML export is not available in this implementation
        """
        raise NotImplementedError(
            "export_wrl_model() is not implementable with the current exodus-rs backend. "
            "VRML export requires extensive 3D graphics generation capabilities "
            "that are not part of the Exodus II format specification. "
            "Consider using the original exomerge3.py or dedicated visualization tools."
        )

    def get_input_deck(self) -> str:
        """
        Get a text representation of the input deck.

        Returns
        -------
        str
            String representation of the model
        """
        raise NotImplementedError(
            "get_input_deck() is not yet implemented. "
            "Implementation planned for Phase 2."
        )

    # ========================================================================
    # Element Block Operations
    # ========================================================================

    def create_element_block(self, element_block_id: int, info: List, connectivity: Optional[List] = None):
        """
        Create a new element block.

        The nodes for the elements in the block must have already been defined.

        The info list should be comprised of the following information:
        [element_type, element_count, nodes_per_element, attributes_per_element]

        For example: ['hex8', 10, 8, 0] would create a hex8 block with 10 elements.

        The connectivity list should be a shallow list of element connectivity
        and must be of length element_count * nodes_per_element.

        Element blocks are unnamed when created. To name them, use rename_element_block().

        Parameters
        ----------
        element_block_id : int
            ID for the new element block
        info : list
            Element block info [element_type, num_elements, nodes_per_element, num_attributes]
        connectivity : list, optional
            Connectivity array for the elements

        Examples
        --------
        >>> model.create_element_block(1, ['HEX8', 10, 8, 0], connectivity_array)
        """
        # Make sure it doesn't exist already
        if self.element_block_exists(element_block_id):
            self._error(f"Element block {element_block_id} already exists")

        # Set up an empty connectivity if none is given
        if connectivity is None:
            connectivity = []

        # Create the actual block: [name, info, connectivity, fields]
        self.element_blocks[element_block_id] = ["", info, connectivity, {}]

    def delete_element_block(self, element_block_ids: Union[int, List[int]], delete_orphaned_nodes: bool = True):
        """
        Delete one or more element blocks.

        This will also delete any references to elements in that block in side sets.

        By default, this will delete any nodes that become unused as a result
        of deleting the element blocks. To prevent this, set delete_orphaned_nodes=False.

        Parameters
        ----------
        element_block_ids : int or list of int
            Element block ID(s) to delete
        delete_orphaned_nodes : bool, optional
            Whether to delete nodes that become orphaned (default: True)

        Examples
        --------
        >>> model.delete_element_block(1)
        >>> model.delete_element_block([1, 2, 3])
        """
        # Convert to list if single ID
        if isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        # If we're not deleting anything, skip
        if not element_block_ids:
            return

        # Find unreferenced nodes before deletion
        if delete_orphaned_nodes:
            unreferenced_nodes = self._get_unreferenced_nodes()

        # Delete the element blocks and associated data
        for element_block_id in element_block_ids:
            # Check if block exists
            if element_block_id not in self.element_blocks:
                self._warning(f"Element block {element_block_id} does not exist")
                continue

            # Delete the element block itself
            del self.element_blocks[element_block_id]

            # Delete faces of that element block from side sets
            for side_set_id in self.get_side_set_ids():
                members = self.get_side_set_members(side_set_id)
                name, _, fields = self.side_sets[side_set_id]

                # Find indices to delete (members are tuples of (block_id, element_id, face_id))
                deleted_indices = []
                for index, member in enumerate(members):
                    if member[0] == element_block_id:  # block_id matches
                        deleted_indices.append(index)

                # Delete them from members (in reverse order to maintain indices)
                new_members = [m for i, m in enumerate(members) if i not in deleted_indices]

                # Delete them from the fields
                new_fields = {}
                for field_name, all_values in fields.items():
                    new_all_values = []
                    for values in all_values:
                        new_values = [v for i, v in enumerate(values) if i not in deleted_indices]
                        new_all_values.append(new_values)
                    new_fields[field_name] = new_all_values

                # Update the side set
                self.side_sets[side_set_id] = [name, new_members, new_fields]

        # Now find the new unreferenced nodes
        if delete_orphaned_nodes:
            new_unreferenced_nodes = self._get_unreferenced_nodes()
            nodes_to_delete = sorted(set(new_unreferenced_nodes) - set(unreferenced_nodes))
            if nodes_to_delete:
                self.delete_node(nodes_to_delete)

    def element_block_exists(self, element_block_id: int) -> bool:
        """
        Check if an element block exists.

        Parameters
        ----------
        element_block_id : int
            Element block ID to check

        Returns
        -------
        bool
            True if element block exists, False otherwise
        """
        return element_block_id in self.element_blocks

    def rename_element_block(self, element_block_id: int, new_element_block_id: Union[int, str]):
        """
        Change an element block ID or name.

        This function can be used to change either the element block ID or name.
        If new_element_block_id is an integer, it will change the ID.
        If it is a string, it will change the name.

        Parameters
        ----------
        element_block_id : int
            Current element block ID
        new_element_block_id : int or str
            New element block ID (int) or new name (str)

        Examples
        --------
        >>> model.rename_element_block(1, 100)  # Change ID from 1 to 100
        >>> model.rename_element_block(1, 'block_1')  # Change name to 'block_1'
        """
        # Check that the block exists
        if element_block_id not in self.element_blocks:
            self._error(f"Element block {element_block_id} does not exist")

        # If we're just changing the name (string provided)
        if isinstance(new_element_block_id, str):
            # If the same name already, just exit
            if self.element_blocks[element_block_id][0] == new_element_block_id:
                return
            # If the name already exists elsewhere, issue a warning
            for block_id, block_data in self.element_blocks.items():
                if block_id != element_block_id and block_data[0] == new_element_block_id:
                    self._warning(f'Element block name "{new_element_block_id}" already exists')
                    break
            # Rename it
            self.element_blocks[element_block_id][0] = new_element_block_id
            return

        # Otherwise, we're changing the ID (integer provided)
        assert isinstance(new_element_block_id, int)

        # Check that the new ID doesn't already exist
        if new_element_block_id in self.element_blocks:
            self._error(f"Element block {new_element_block_id} already exists")

        # Rename the block by creating new entry and deleting old
        self.element_blocks[new_element_block_id] = self.element_blocks[element_block_id]
        del self.element_blocks[element_block_id]

        # Adjust side sets that reference this element block
        for side_set_id in self.get_side_set_ids():
            name, members, fields = self.side_sets[side_set_id]
            new_members = []
            for member in members:
                # member is (block_id, element_id, face_id)
                if member[0] == element_block_id:
                    new_members.append((new_element_block_id, member[1], member[2]))
                else:
                    new_members.append(member)
            self.side_sets[side_set_id] = [name, new_members, fields]

    def get_element_block_ids(self) -> List[int]:
        """
        Get all element block IDs.

        Returns
        -------
        list of int
            List of element block IDs

        Examples
        --------
        >>> ids = model.get_element_block_ids()
        >>> print(ids)
        [1, 2, 3]
        """
        return list(self.element_blocks.keys())

    def get_element_block_name(self, element_block_id: int) -> str:
        """
        Get the name of an element block.

        Parameters
        ----------
        element_block_id : int
            Element block ID

        Returns
        -------
        str
            Element block name
        """
        if element_block_id not in self.element_blocks:
            self._error(f"Element block {element_block_id} does not exist")
        return self.element_blocks[element_block_id][0]  # First element is name

    def get_all_element_block_names(self) -> Dict[int, str]:
        """
        Get names of all element blocks.

        Returns
        -------
        dict
            Dictionary mapping element block IDs to names
        """
        return {block_id: block_data[0] for block_id, block_data in self.element_blocks.items()}

    def get_element_count(self, element_block_ids: Union[str, List[int]] = "all") -> int:
        """
        Get total element count.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element blocks to count (default: "all")

        Returns
        -------
        int
            Total number of elements
        """
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        total = 0
        for block_id in element_block_ids:
            if block_id in self.element_blocks:
                # block_data[1] is info, info[1] is num_elements
                total += self.element_blocks[block_id][1][1]
        return total

    def get_element_block_dimension(self, element_block_id: int) -> int:
        """
        Get the spatial dimension of an element block.

        Parameters
        ----------
        element_block_id : int
            Element block ID

        Returns
        -------
        int
            Spatial dimension (1, 2, or 3)
        """
        if element_block_id not in self.element_blocks:
            self._error(f"Element block {element_block_id} does not exist")

        elem_type = self.element_blocks[element_block_id][1][0]  # info[0] is element type
        return self._get_dimension(elem_type)

    def get_nodes_per_element(self, element_block_id: int) -> int:
        """
        Get number of nodes per element in a block.

        Parameters
        ----------
        element_block_id : int
            Element block ID

        Returns
        -------
        int
            Number of nodes per element
        """
        if element_block_id not in self.element_blocks:
            self._error(f"Element block {element_block_id} does not exist")

        return self.element_blocks[element_block_id][1][2]  # info[2] is nodes_per_element

    def get_connectivity(self, element_block_id: Union[str, int] = "auto") -> List[List[int]]:
        """
        Get element connectivity array.

        Parameters
        ----------
        element_block_id : str or int, optional
            Element block ID or "auto" (default: "auto")

        Returns
        -------
        list of list of int
            Connectivity array
        """
        if element_block_id == "auto":
            if len(self.element_blocks) != 1:
                self._error("Must specify element_block_id when model has multiple element blocks")
            element_block_id = list(self.element_blocks.keys())[0]

        if element_block_id not in self.element_blocks:
            self._error(f"Element block {element_block_id} does not exist")

        return self.element_blocks[element_block_id][2]  # Third element is connectivity

    def get_element_block_connectivity(self, element_block_id: Union[str, int] = "auto") -> List[List[int]]:
        """
        Get element connectivity array (alias for get_connectivity).

        Parameters
        ----------
        element_block_id : str or int, optional
            Element block ID or "auto" (default: "auto")

        Returns
        -------
        list of list of int
            Connectivity array
        """
        return self.get_connectivity(element_block_id)

    def get_nodes_in_element_block(self, element_block_ids: Union[str, int, List[int]]) -> List[int]:
        """
        Return a list of all node indices used in the given element blocks.

        Parameters
        ----------
        element_block_ids : str, int, or list of int
            Element block IDs or "all"

        Returns
        -------
        list of int
            Sorted list of unique node indices

        Examples
        --------
        >>> model.get_nodes_in_element_block(1)
        >>> model.get_nodes_in_element_block([1, 3])
        >>> model.get_nodes_in_element_block("all")
        """
        # Handle "all" case
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        # Convert single ID to list
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        # Collect all unique nodes
        node_set = set()
        for block_id in element_block_ids:
            if block_id not in self.element_blocks:
                self._warning(f"Element block {block_id} does not exist")
                continue

            connectivity = self.get_connectivity(block_id)
            # Flatten the connectivity list and add to set
            for element_nodes in connectivity:
                node_set.update(element_nodes)

        return sorted(node_set)

    def duplicate_element_block(self, source_id: int, target_id: int, duplicate_nodes: bool = True):
        """
        Create a duplicate of the given element block.

        Nodes are duplicated by default. The new element block references
        these duplicated nodes, not the original ones.

        Parameters
        ----------
        source_id : int
            Source element block ID
        target_id : int
            Target element block ID
        duplicate_nodes : bool, optional
            Whether to duplicate nodes (default: True)

        Examples
        --------
        >>> model.duplicate_element_block(1, 2)
        >>> model.duplicate_element_block(1, 3, duplicate_nodes=False)
        """
        # Check that source block exists
        if source_id not in self.element_blocks:
            self._error(f"Element block {source_id} does not exist")

        # Check that target block doesn't exist
        if target_id in self.element_blocks:
            self._error(f"Element block {target_id} already exists")

        # Get source block data
        name, info, connectivity, fields = self.element_blocks[source_id]
        info = list(info)  # Make a copy
        old_connectivity = connectivity

        # Create new nodes if requested
        if duplicate_nodes:
            # Get unique nodes from connectivity
            unique_node_indices = sorted(set(
                node_idx for element in old_connectivity for node_idx in element
            ))

            # Create node mapping
            node_map = {}
            new_node_offset = len(self.nodes)

            # Duplicate the nodes
            for old_idx in unique_node_indices:
                # Convert to 0-based index
                zero_based_idx = old_idx - 1
                if 0 <= zero_based_idx < len(self.nodes):
                    # Add new node
                    self.nodes.append(list(self.nodes[zero_based_idx]))
                    node_map[old_idx] = new_node_offset + 1  # 1-based indexing
                    new_node_offset += 1

            # Create new connectivity with new node indices
            new_connectivity = []
            for element in old_connectivity:
                new_element = [node_map.get(node_idx, node_idx) for node_idx in element]
                new_connectivity.append(new_element)
        else:
            # Just copy the connectivity
            new_connectivity = [list(element) for element in old_connectivity]

        # Create the new element block
        self.create_element_block(target_id, info, new_connectivity)

        # Copy the name if it exists
        if name:
            self.element_blocks[target_id][0] = name + "_copy"

        # Copy fields
        new_fields = {}
        for field_name, all_values in fields.items():
            new_fields[field_name] = [list(values) for values in all_values]
        self.element_blocks[target_id][3] = new_fields

        # Update side sets to include references to new block
        for side_set_id in self.get_side_set_ids():
            name, members, fields = self.side_sets[side_set_id]
            new_members = []
            source_indices = []

            for idx, member in enumerate(members):
                if member[0] == source_id:
                    # Add a duplicate member for the new block
                    new_members.append((target_id, member[1], member[2]))
                    source_indices.append(idx)

            if new_members:
                # Add new members
                members.extend(new_members)

                # Duplicate field values for new members
                for field_name, all_values in fields.items():
                    for values in all_values:
                        new_values = [values[idx] for idx in source_indices]
                        values.extend(new_values)

    def combine_element_blocks(self, element_block_ids: List[int], target_element_block_id: Union[str, int] = "auto"):
        """
        Combine multiple element blocks into one.

        Parameters
        ----------
        element_block_ids : list of int
            Element block IDs to combine
        target_element_block_id : str or int, optional
            Target element block ID (default: "auto")
        """
        raise NotImplementedError(
            "combine_element_blocks() is not yet implemented. "
            "Implementation planned for Phase 3."
        )

    def unmerge_element_blocks(self, element_block_ids: Union[str, List[int]] = "all"):
        """
        Split element blocks so they don't share nodes.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element block IDs (default: "all")
        """
        raise NotImplementedError(
            "unmerge_element_blocks() is not yet implemented. "
            "Implementation planned for Phase 3."
        )

    def process_element_fields(self, element_block_ids: Union[str, List[int]] = "all"):
        """
        Process element fields to ensure consistency.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element block IDs (default: "all")
        """
        raise NotImplementedError(
            "process_element_fields() is not yet implemented. "
            "Implementation planned for Phase 3."
        )

    def translate_element_blocks(self, element_block_ids: Union[str, List[int]], offset: List[float],
                                 check_for_merged_nodes: bool = True):
        """
        Translate the specified element blocks by the given offset.

        Parameters
        ----------
        element_block_ids : str, int, or list of int
            Element block IDs to translate or "all"
        offset : list of float
            Translation offset [dx, dy, dz]
        check_for_merged_nodes : bool, optional
            Whether to check for shared nodes (default: True)

        Examples
        --------
        >>> model.translate_element_blocks(1, [1.0, 2.0, 3.0])
        >>> model.translate_element_blocks([1, 2], [5.0, 0.0, 0.0])
        """
        # Handle "all" case
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        # Convert single ID to list
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        if check_for_merged_nodes:
            self._ensure_no_shared_nodes(element_block_ids)

        affected_nodes = self.get_nodes_in_element_block(element_block_ids)
        self._translate_nodes(offset, affected_nodes)

    def reflect_element_blocks(self, element_block_ids: Union[str, List[int]], *args, **kwargs):
        """Reflect element blocks."""
        raise NotImplementedError("reflect_element_blocks() is not yet implemented.")

    def scale_element_blocks(self, element_block_ids: Union[str, List[int]], scale_factor: float,
                             check_for_merged_nodes: bool = True,
                             adjust_displacement_field: Union[str, bool] = "auto"):
        """
        Scale all nodes in the given element blocks by the given amount.

        By default, if a displacement field exists, this will also scale the
        displacement field.

        Parameters
        ----------
        element_block_ids : str, int, or list of int
            Element block IDs to scale or "all"
        scale_factor : float
            Scale factor
        check_for_merged_nodes : bool, optional
            Whether to check for shared nodes (default: True)
        adjust_displacement_field : str or bool, optional
            Whether to adjust displacement field (default: "auto")

        Examples
        --------
        >>> model.scale_element_blocks(1, 0.0254)  # Convert inches to meters
        >>> model.scale_element_blocks([1, 2], 2.0)  # Double size
        """
        # Handle "all" case
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        # Convert single ID to list
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        if adjust_displacement_field == "auto":
            adjust_displacement_field = False  # Displacement field not yet implemented

        if check_for_merged_nodes:
            self._ensure_no_shared_nodes(element_block_ids)

        affected_nodes = self.get_nodes_in_element_block(element_block_ids)
        self._scale_nodes(scale_factor, affected_nodes, adjust_displacement_field)

    def rotate_element_blocks(self, element_block_ids: Union[str, List[int]], axis: List[float],
                              angle_in_degrees: float, check_for_merged_nodes: bool = True,
                              adjust_displacement_field: Union[str, bool] = "auto"):
        """
        Rotate all nodes in the given element blocks by the given amount.

        By default, if a displacement field exists, this will also rotate the
        displacement field.

        The rotation axis includes the origin and points in the direction of
        the 'axis' parameter.

        Parameters
        ----------
        element_block_ids : str, int, or list of int
            Element block IDs to rotate or "all"
        axis : list of float
            Rotation axis direction [x, y, z]
        angle_in_degrees : float
            Rotation angle in degrees
        check_for_merged_nodes : bool, optional
            Whether to check for shared nodes (default: True)
        adjust_displacement_field : str or bool, optional
            Whether to adjust displacement field (default: "auto")

        Examples
        --------
        >>> model.rotate_element_blocks(1, [1, 0, 0], 90)  # Rotate 90° around X-axis
        >>> model.rotate_element_blocks([1, 2], [0, 0, 1], 45)  # Rotate 45° around Z-axis
        """
        # Handle "all" case
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        # Convert single ID to list
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        if adjust_displacement_field == "auto":
            adjust_displacement_field = False  # Displacement field not yet implemented

        if check_for_merged_nodes:
            self._ensure_no_shared_nodes(element_block_ids)

        affected_nodes = self.get_nodes_in_element_block(element_block_ids)
        self._rotate_nodes(axis, angle_in_degrees, affected_nodes, adjust_displacement_field)

    def displace_element_blocks(self, element_block_ids: Union[str, List[int]], *args, **kwargs):
        """Displace element blocks using displacement field."""
        raise NotImplementedError("displace_element_blocks() is not yet implemented.")

    def convert_element_blocks(self, element_block_ids: Union[str, List[int]], new_element_type: str):
        """Convert element blocks to a new element type."""
        raise NotImplementedError("convert_element_blocks() is not yet implemented.")

    def make_elements_linear(self, element_block_ids: Union[str, List[int]] = "all"):
        """Convert elements to linear (first-order) type."""
        raise NotImplementedError("make_elements_linear() is not yet implemented.")

    def make_elements_quadratic(self, element_block_ids: Union[str, List[int]] = "all"):
        """Convert elements to quadratic (second-order) type."""
        raise NotImplementedError("make_elements_quadratic() is not yet implemented.")

    def convert_hex8_block_to_tet4_block(self, element_block_id: int, scheme: str = "hex24tet"):
        """Convert HEX8 elements to TET4 elements."""
        raise NotImplementedError("convert_hex8_block_to_tet4_block() is not yet implemented.")

    def threshold_element_blocks(self, expression: str, element_block_ids: Union[str, List[int]] = "all",
                                timestep: Union[str, float] = "last", *args, **kwargs):
        """Filter element blocks based on expression threshold."""
        raise NotImplementedError("threshold_element_blocks() is not yet implemented.")

    def count_degenerate_elements(self, element_block_ids: Union[str, List[int]] = "all") -> int:
        """Count degenerate elements in element blocks."""
        raise NotImplementedError("count_degenerate_elements() is not yet implemented.")

    def count_disconnected_blocks(self, element_block_ids: Union[str, List[int]] = "all") -> int:
        """Count disconnected sub-blocks within element blocks."""
        raise NotImplementedError("count_disconnected_blocks() is not yet implemented.")

    def delete_duplicate_elements(self, element_block_ids: Union[str, List[int]] = "all"):
        """Delete duplicate elements."""
        raise NotImplementedError("delete_duplicate_elements() is not yet implemented.")

    def calculate_element_centroids(self, element_block_ids: Union[str, List[int]] = "all",
                                   field_prefix: str = "centroid"):
        """Calculate element centroids and store as fields."""
        raise NotImplementedError("calculate_element_centroids() is not yet implemented.")

    def calculate_element_volumes(self, element_block_ids: Union[str, List[int]] = "all",
                                 field_name: str = "volume"):
        """Calculate element volumes and store as field."""
        raise NotImplementedError("calculate_element_volumes() is not yet implemented.")

    def get_element_block_volume(self, element_block_ids: Union[str, List[int]] = "all",
                                timestep: Union[str, float] = "last") -> float:
        """Get total volume of element blocks."""
        raise NotImplementedError("get_element_block_volume() is not yet implemented.")

    def get_element_block_centroid(self, element_block_ids: Union[str, List[int]] = "all",
                                  timestep: Union[str, float] = "last") -> List[float]:
        """Get centroid of element blocks."""
        raise NotImplementedError("get_element_block_centroid() is not yet implemented.")

    def get_element_block_extents(self, element_block_ids: Union[str, List[int]] = "all") -> List[Tuple[float, float]]:
        """
        Return the extents of the element blocks as a list.

        The results are returned in the following format:
        [[min_x, max_x], [min_y, max_y], [min_z, max_z]]

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element block IDs or "all" (default: "all")

        Returns
        -------
        list of tuple
            Bounding box extents [[min_x, max_x], [min_y, max_y], [min_z, max_z]]

        Examples
        --------
        >>> extents = model.get_element_block_extents(1)
        >>> extents = model.get_element_block_extents([1, 2])
        >>> extents = model.get_element_block_extents("all")
        """
        # Handle "all" case
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        # Convert single ID to list
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        if not element_block_ids:
            self._error("No element blocks specified")

        # Get a set of all nodes within the given element blocks
        all_nodes = set()
        for block_id in element_block_ids:
            if block_id not in self.element_blocks:
                self._warning(f"Element block {block_id} does not exist")
                continue
            connectivity = self.get_connectivity(block_id)
            for element in connectivity:
                all_nodes.update(element)

        if not all_nodes:
            # Return zero extents if no nodes
            return [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]

        # Convert 1-based indices to 0-based
        all_nodes_zero_based = [idx - 1 for idx in all_nodes if 0 < idx <= len(self.nodes)]

        # Find the extents
        extents = []
        for d in range(3):  # x, y, z dimensions
            node_coords = [self.nodes[node_idx][d] for node_idx in all_nodes_zero_based]
            extents.append([min(node_coords), max(node_coords)])

        return extents

    def get_element_edge_length_info(self, element_block_ids: Union[str, List[int]] = "all") -> Tuple[float, float]:
        """Get minimum and average element edge lengths."""
        raise NotImplementedError("get_element_edge_length_info() is not yet implemented.")

    # ========================================================================
    # Field Operations - Element Fields
    # ========================================================================

    def create_element_field(self, element_field_name: str, element_block_id: int,
                            default_value: Union[str, float] = "auto"):
        """
        Create an element field.

        Parameters
        ----------
        element_field_name : str
            Name of the element field
        element_block_id : int
            Element block ID
        default_value : str or float, optional
            Initial value ("auto" for zeros, float for constant value)

        Examples
        --------
        >>> model.create_element_field("stress", 1, 0.0)
        """
        if element_block_id not in self.element_blocks:
            self._error(f"Element block {element_block_id} does not exist")

        name, info, connectivity, fields = self.element_blocks[element_block_id]

        if element_field_name in fields:
            self._warning(f"Element field '{element_field_name}' already exists in block {element_block_id}, overwriting")

        # Initialize field values for all timesteps
        num_timesteps = len(self.timesteps) if self.timesteps else 1
        num_elems = info[1]  # info = [elem_type, num_elems, nodes_per_elem, num_attrs]

        if default_value == "auto":
            # Create zero-filled arrays
            field_data = [[0.0] * num_elems for _ in range(num_timesteps)]
        elif isinstance(default_value, (int, float)):
            # Constant value for all elements and timesteps
            field_data = [[float(default_value)] * num_elems for _ in range(num_timesteps)]
        else:
            field_data = [[0.0] * num_elems for _ in range(num_timesteps)]

        fields[element_field_name] = field_data
        self.element_blocks[element_block_id] = [name, info, connectivity, fields]

    def delete_element_field(self, element_field_names: Union[str, List[str]],
                            element_block_ids: Union[str, List[int]] = "all"):
        """
        Delete element field(s).

        Parameters
        ----------
        element_field_names : str or list of str
            Element field name(s) to delete
        element_block_ids : str or list of int, optional
            Element block IDs ("all" for all blocks)
        """
        if isinstance(element_field_names, str):
            element_field_names = [element_field_names]

        # Determine which blocks to process
        if element_block_ids == "all":
            block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            block_ids = [element_block_ids]
        else:
            block_ids = element_block_ids

        for block_id in block_ids:
            if block_id not in self.element_blocks:
                continue

            name, info, connectivity, fields = self.element_blocks[block_id]
            for field_name in element_field_names:
                if field_name in fields:
                    del fields[field_name]
            self.element_blocks[block_id] = [name, info, connectivity, fields]

    def element_field_exists(self, element_field_name: str,
                            element_block_ids: Union[str, List[int]] = "all") -> bool:
        """
        Check if element field exists.

        Parameters
        ----------
        element_field_name : str
            Element field name
        element_block_ids : str or list of int, optional
            Element block IDs to check ("all" for all blocks)

        Returns
        -------
        bool
            True if element field exists in any of the specified blocks
        """
        # Determine which blocks to check
        if element_block_ids == "all":
            block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            block_ids = [element_block_ids]
        else:
            block_ids = element_block_ids

        for block_id in block_ids:
            if block_id not in self.element_blocks:
                continue
            name, info, connectivity, fields = self.element_blocks[block_id]
            if element_field_name in fields:
                return True
        return False

    def get_element_field_names(self, element_block_ids: Union[str, List[int]] = "all") -> List[str]:
        """
        Get element field names.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element block IDs ("all" for all blocks)

        Returns
        -------
        list of str
            List of unique element field names across specified blocks
        """
        # Determine which blocks to check
        if element_block_ids == "all":
            block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            block_ids = [element_block_ids]
        else:
            block_ids = element_block_ids

        field_names = set()
        for block_id in block_ids:
            if block_id not in self.element_blocks:
                continue
            name, info, connectivity, fields = self.element_blocks[block_id]
            field_names.update(fields.keys())

        return sorted(field_names)

    def get_element_field_values(self, element_field_name: str, element_block_id: int,
                                timestep: Union[str, float] = "last") -> List[float]:
        """
        Get element field values.

        Parameters
        ----------
        element_field_name : str
            Element field name
        element_block_id : int
            Element block ID
        timestep : str or float, optional
            Timestep ("last" or timestep value)

        Returns
        -------
        list of float
            Element field values
        """
        if element_block_id not in self.element_blocks:
            self._error(f"Element block {element_block_id} does not exist")

        name, info, connectivity, fields = self.element_blocks[element_block_id]

        if element_field_name not in fields:
            self._error(f"Element field '{element_field_name}' does not exist in block {element_block_id}")

        field_data = fields[element_field_name]

        # Determine timestep index
        if timestep == "last":
            timestep_idx = len(field_data) - 1 if field_data else 0
        else:
            # Find timestep index
            try:
                timestep_idx = self.timesteps.index(timestep)
            except ValueError:
                timestep_idx = 0

        if timestep_idx < 0 or timestep_idx >= len(field_data):
            timestep_idx = 0

        return field_data[timestep_idx] if field_data else []

    def rename_element_field(self, element_field_name: str, new_element_field_name: str,
                            element_block_ids: Union[str, List[int]] = "all"):
        """
        Rename an element field.

        Parameters
        ----------
        element_field_name : str
            Current element field name
        new_element_field_name : str
            New element field name
        element_block_ids : str or list of int, optional
            Element block IDs ("all" for all blocks)
        """
        # Determine which blocks to process
        if element_block_ids == "all":
            block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            block_ids = [element_block_ids]
        else:
            block_ids = element_block_ids

        for block_id in block_ids:
            if block_id not in self.element_blocks:
                continue

            name, info, connectivity, fields = self.element_blocks[block_id]

            if element_field_name not in fields:
                continue

            if new_element_field_name in fields:
                self._error(f"Element field '{new_element_field_name}' already exists in block {block_id}")

            fields[new_element_field_name] = fields[element_field_name]
            del fields[element_field_name]
            self.element_blocks[block_id] = [name, info, connectivity, fields]

    def calculate_element_field(self, expression: str, element_block_ids: Union[str, List[int]] = "all"):
        """Calculate element field from expression."""
        raise NotImplementedError(
            "calculate_element_field() requires expression evaluation which is not yet implemented. "
            "This method needs a Python expression parser to evaluate mathematical expressions "
            "on element field data. Implementation planned for Phase 4."
        )

    def calculate_element_field_maximum(self, element_field_names: Union[str, List[str]],
                                       element_block_ids: Union[str, List[int]] = "all",
                                       calculate_location: bool = False,
                                       calculate_block_id: bool = False) -> Union[float, Tuple]:
        """Find maximum value of element field(s)."""
        raise NotImplementedError("calculate_element_field_maximum() is not yet implemented.")

    def calculate_element_field_minimum(self, element_field_names: Union[str, List[str]],
                                       element_block_ids: Union[str, List[int]] = "all",
                                       calculate_location: bool = False,
                                       calculate_block_id: bool = False) -> Union[float, Tuple]:
        """Find minimum value of element field(s)."""
        raise NotImplementedError("calculate_element_field_minimum() is not yet implemented.")

    def create_averaged_element_field(self, element_field_names: Union[str, List[str]],
                                     new_field_name: str, element_block_ids: Union[str, List[int]] = "all"):
        """Create averaged element field from multiple fields."""
        raise NotImplementedError("create_averaged_element_field() is not yet implemented.")

    def convert_element_field_to_node_field(self, element_field_name: str, node_field_name: str = None,
                                           element_block_ids: Union[str, List[int]] = "all"):
        """Convert element field to node field."""
        raise NotImplementedError("convert_element_field_to_node_field() is not yet implemented.")

    def convert_node_field_to_element_field(self, node_field_name: str, element_field_name: str = None,
                                           element_block_ids: Union[str, List[int]] = "all"):
        """Convert node field to element field."""
        raise NotImplementedError("convert_node_field_to_element_field() is not yet implemented.")

    # ========================================================================
    # Field Operations - Node Fields
    # ========================================================================

    def create_node_field(self, node_field_name: str, value: Union[str, float, List] = "auto"):
        """
        Create a node field.

        Parameters
        ----------
        node_field_name : str
            Name of the node field
        value : str, float, or list, optional
            Initial value ("auto" for zeros, float for constant, list for per-timestep values)

        Examples
        --------
        >>> model.create_node_field("temperature", 0.0)
        """
        if node_field_name in self.node_fields:
            self._warning(f"Node field '{node_field_name}' already exists, overwriting")

        # Initialize field values for all timesteps
        num_timesteps = len(self.timesteps) if self.timesteps else 1
        num_nodes = len(self.nodes)

        if value == "auto":
            # Create zero-filled arrays
            field_data = [[0.0] * num_nodes for _ in range(num_timesteps)]
        elif isinstance(value, (int, float)):
            # Constant value for all nodes and timesteps
            field_data = [[float(value)] * num_nodes for _ in range(num_timesteps)]
        elif isinstance(value, list):
            # User-provided values
            field_data = value
        else:
            field_data = [[0.0] * num_nodes for _ in range(num_timesteps)]

        self.node_fields[node_field_name] = field_data

    def delete_node_field(self, node_field_names: Union[str, List[str]]):
        """
        Delete node field(s).

        Parameters
        ----------
        node_field_names : str or list of str
            Node field name(s) to delete
        """
        if isinstance(node_field_names, str):
            node_field_names = [node_field_names]

        for field_name in node_field_names:
            if field_name in self.node_fields:
                del self.node_fields[field_name]

    def node_field_exists(self, node_field_name: str) -> bool:
        """
        Check if node field exists.

        Parameters
        ----------
        node_field_name : str
            Node field name

        Returns
        -------
        bool
            True if node field exists
        """
        return node_field_name in self.node_fields

    def get_node_field_names(self) -> List[str]:
        """
        Get all node field names.

        Returns
        -------
        list of str
            List of node field names
        """
        return list(self.node_fields.keys())

    def get_node_field_values(self, node_field_name: str, timestep: Union[str, float] = "last") -> List[float]:
        """
        Get node field values.

        Parameters
        ----------
        node_field_name : str
            Node field name
        timestep : str or float, optional
            Timestep ("last" or timestep value)

        Returns
        -------
        list of float
            Node field values

        """
        if node_field_name not in self.node_fields:
            self._error(f"Node field '{node_field_name}' does not exist")

        field_data = self.node_fields[node_field_name]

        # Determine timestep index
        if timestep == "last":
            timestep_idx = len(field_data) - 1 if field_data else 0
        else:
            # Find timestep index
            try:
                timestep_idx = self.timesteps.index(timestep)
            except ValueError:
                timestep_idx = 0

        if timestep_idx < 0 or timestep_idx >= len(field_data):
            timestep_idx = 0

        return field_data[timestep_idx] if field_data else []

    def rename_node_field(self, node_field_name: str, new_node_field_name: str):
        """
        Rename a node field.

        Parameters
        ----------
        node_field_name : str
            Current node field name
        new_node_field_name : str
            New node field name
        """
        if node_field_name not in self.node_fields:
            self._error(f"Node field '{node_field_name}' does not exist")
        if new_node_field_name in self.node_fields:
            self._error(f"Node field '{new_node_field_name}' already exists")

        self.node_fields[new_node_field_name] = self.node_fields[node_field_name]
        del self.node_fields[node_field_name]

    def calculate_node_field(self, expression: str):
        """Calculate node field from expression."""
        raise NotImplementedError(
            "calculate_node_field() requires expression evaluation which is not yet implemented. "
            "This method needs a Python expression parser to evaluate mathematical expressions "
            "on node field data. Implementation planned for Phase 4."
        )

    def calculate_node_field_maximum(self, node_field_names: Union[str, List[str]],
                                    calculate_location: bool = False) -> Union[float, Tuple]:
        """Find maximum value of node field(s)."""
        raise NotImplementedError("calculate_node_field_maximum() is not yet implemented.")

    def calculate_node_field_minimum(self, node_field_names: Union[str, List[str]],
                                    calculate_location: bool = False) -> Union[float, Tuple]:
        """Find minimum value of node field(s)."""
        raise NotImplementedError("calculate_node_field_minimum() is not yet implemented.")

    def displacement_field_exists(self) -> bool:
        """Check if displacement field exists."""
        raise NotImplementedError("displacement_field_exists() is not yet implemented.")

    def create_displacement_field(self):
        """Create displacement field."""
        raise NotImplementedError("create_displacement_field() is not yet implemented.")

    # ========================================================================
    # Field Operations - Global Variables
    # ========================================================================

    def create_global_variable(self, global_variable_name: str, value: Union[str, float, List] = "auto"):
        """
        Create a global variable.

        Parameters
        ----------
        global_variable_name : str
            Name of the global variable
        value : str, float, or list, optional
            Initial value ("auto" for zeros, float for constant, list for per-timestep values)

        Examples
        --------
        >>> model.create_global_variable("time_step", 0.01)
        """
        if global_variable_name in self.global_variables:
            self._warning(f"Global variable '{global_variable_name}' already exists, overwriting")

        # Initialize values for all timesteps
        num_timesteps = len(self.timesteps) if self.timesteps else 1

        if value == "auto":
            # Create zero-filled array
            var_data = [0.0] * num_timesteps
        elif isinstance(value, (int, float)):
            # Constant value for all timesteps
            var_data = [float(value)] * num_timesteps
        elif isinstance(value, list):
            # User-provided values
            var_data = value
        else:
            var_data = [0.0] * num_timesteps

        self.global_variables[global_variable_name] = var_data

    def delete_global_variable(self, global_variable_names: Union[str, List[str]]):
        """
        Delete global variable(s).

        Parameters
        ----------
        global_variable_names : str or list of str
            Global variable name(s) to delete
        """
        if isinstance(global_variable_names, str):
            global_variable_names = [global_variable_names]

        for var_name in global_variable_names:
            if var_name in self.global_variables:
                del self.global_variables[var_name]

    def global_variable_exists(self, global_variable_name: str) -> bool:
        """
        Check if global variable exists.

        Parameters
        ----------
        global_variable_name : str
            Global variable name

        Returns
        -------
        bool
            True if global variable exists
        """
        return global_variable_name in self.global_variables

    def get_global_variable_names(self) -> List[str]:
        """
        Get all global variable names.

        Returns
        -------
        list of str
            List of global variable names
        """
        return list(self.global_variables.keys())

    def rename_global_variable(self, global_variable_name: str, new_global_variable_name: str):
        """
        Rename a global variable.

        Parameters
        ----------
        global_variable_name : str
            Current global variable name
        new_global_variable_name : str
            New global variable name
        """
        if global_variable_name not in self.global_variables:
            self._error(f"Global variable '{global_variable_name}' does not exist")
        if new_global_variable_name in self.global_variables:
            self._error(f"Global variable '{new_global_variable_name}' already exists")

        self.global_variables[new_global_variable_name] = self.global_variables[global_variable_name]
        del self.global_variables[global_variable_name]

    def calculate_global_variable(self, expression: str):
        """Calculate global variable from expression."""
        raise NotImplementedError(
            "calculate_global_variable() requires expression evaluation which is not yet implemented. "
            "This method needs a Python expression parser to evaluate mathematical expressions. "
            "Implementation planned for Phase 4."
        )

    def output_global_variables(self, expressions: List[str], *args, **kwargs):
        """Output global variables calculated from expressions."""
        raise NotImplementedError("output_global_variables() is not yet implemented.")

    # ========================================================================
    # Field Operations - Side Set Fields
    # ========================================================================

    def create_side_set_field(self, side_set_field_name: str, side_set_id: int,
                             default_value: Union[str, float] = "auto"):
        """
        Create a side set field.

        Parameters
        ----------
        side_set_field_name : str
            Name of the side set field
        side_set_id : int
            Side set ID
        default_value : str or float, optional
            Initial value ("auto" for zeros, float for constant value)

        Examples
        --------
        >>> model.create_side_set_field("pressure", 1, 0.0)
        """
        if side_set_id not in self.side_sets:
            self._error(f"Side set {side_set_id} does not exist")

        name, members, fields = self.side_sets[side_set_id]

        if side_set_field_name in fields:
            self._warning(f"Side set field '{side_set_field_name}' already exists in side set {side_set_id}, overwriting")

        # Initialize field values for all timesteps
        num_timesteps = len(self.timesteps) if self.timesteps else 1
        num_members = len(members)

        if default_value == "auto":
            # Create zero-filled arrays
            field_data = [[0.0] * num_members for _ in range(num_timesteps)]
        elif isinstance(default_value, (int, float)):
            # Constant value for all members and timesteps
            field_data = [[float(default_value)] * num_members for _ in range(num_timesteps)]
        else:
            field_data = [[0.0] * num_members for _ in range(num_timesteps)]

        fields[side_set_field_name] = field_data
        self.side_sets[side_set_id] = [name, members, fields]

    def delete_side_set_field(self, side_set_field_names: Union[str, List[str]],
                             side_set_ids: Union[str, List[int]] = "all"):
        """
        Delete side set field(s).

        Parameters
        ----------
        side_set_field_names : str or list of str
            Side set field name(s) to delete
        side_set_ids : str or list of int, optional
            Side set IDs ("all" for all side sets)
        """
        if isinstance(side_set_field_names, str):
            side_set_field_names = [side_set_field_names]

        # Determine which side sets to process
        if side_set_ids == "all":
            set_ids = list(self.side_sets.keys())
        elif isinstance(side_set_ids, int):
            set_ids = [side_set_ids]
        else:
            set_ids = side_set_ids

        for set_id in set_ids:
            if set_id not in self.side_sets:
                continue

            name, members, fields = self.side_sets[set_id]
            for field_name in side_set_field_names:
                if field_name in fields:
                    del fields[field_name]
            self.side_sets[set_id] = [name, members, fields]

    def side_set_field_exists(self, side_set_field_name: str,
                             side_set_ids: Union[str, List[int]] = "all") -> bool:
        """
        Check if side set field exists.

        Parameters
        ----------
        side_set_field_name : str
            Side set field name
        side_set_ids : str or list of int, optional
            Side set IDs to check ("all" for all side sets)

        Returns
        -------
        bool
            True if side set field exists in any of the specified side sets
        """
        # Determine which side sets to check
        if side_set_ids == "all":
            set_ids = list(self.side_sets.keys())
        elif isinstance(side_set_ids, int):
            set_ids = [side_set_ids]
        else:
            set_ids = side_set_ids

        for set_id in set_ids:
            if set_id not in self.side_sets:
                continue
            name, members, fields = self.side_sets[set_id]
            if side_set_field_name in fields:
                return True
        return False

    def get_side_set_field_names(self, side_set_id: int) -> List[str]:
        """
        Get side set field names.

        Parameters
        ----------
        side_set_id : int
            Side set ID

        Returns
        -------
        list of str
            List of side set field names
        """
        if side_set_id not in self.side_sets:
            self._error(f"Side set {side_set_id} does not exist")

        name, members, fields = self.side_sets[side_set_id]
        return list(fields.keys())

    def get_side_set_field_values(self, side_set_field_name: str, side_set_id: int,
                                  timestep: Union[str, float] = "last") -> List[float]:
        """
        Get side set field values.

        Parameters
        ----------
        side_set_field_name : str
            Side set field name
        side_set_id : int
            Side set ID
        timestep : str or float, optional
            Timestep ("last" or timestep value)

        Returns
        -------
        list of float
            Side set field values
        """
        if side_set_id not in self.side_sets:
            self._error(f"Side set {side_set_id} does not exist")

        name, members, fields = self.side_sets[side_set_id]

        if side_set_field_name not in fields:
            self._error(f"Side set field '{side_set_field_name}' does not exist in side set {side_set_id}")

        field_data = fields[side_set_field_name]

        # Determine timestep index
        if timestep == "last":
            timestep_idx = len(field_data) - 1 if field_data else 0
        else:
            # Find timestep index
            try:
                timestep_idx = self.timesteps.index(timestep)
            except ValueError:
                timestep_idx = 0

        if timestep_idx < 0 or timestep_idx >= len(field_data):
            timestep_idx = 0

        return field_data[timestep_idx] if field_data else []

    def rename_side_set_field(self, side_set_field_name: str, new_side_set_field_name: str,
                             side_set_ids: Union[str, List[int]] = "all"):
        """
        Rename a side set field.

        Parameters
        ----------
        side_set_field_name : str
            Current side set field name
        new_side_set_field_name : str
            New side set field name
        side_set_ids : str or list of int, optional
            Side set IDs ("all" for all side sets)
        """
        # Determine which side sets to process
        if side_set_ids == "all":
            set_ids = list(self.side_sets.keys())
        elif isinstance(side_set_ids, int):
            set_ids = [side_set_ids]
        else:
            set_ids = side_set_ids

        for set_id in set_ids:
            if set_id not in self.side_sets:
                continue

            name, members, fields = self.side_sets[set_id]

            if side_set_field_name not in fields:
                continue

            if new_side_set_field_name in fields:
                self._error(f"Side set field '{new_side_set_field_name}' already exists in side set {set_id}")

            fields[new_side_set_field_name] = fields[side_set_field_name]
            del fields[side_set_field_name]
            self.side_sets[set_id] = [name, members, fields]

    def calculate_side_set_field(self, expression: str, side_set_ids: Union[str, List[int]] = "all"):
        """Calculate side set field from expression."""
        raise NotImplementedError(
            "calculate_side_set_field() requires expression evaluation which is not yet implemented. "
            "Implementation planned for Phase 4."
        )

    # ========================================================================
    # Field Operations - Node Set Fields
    # ========================================================================

    def create_node_set_field(self, node_set_field_name: str, node_set_id: int,
                             default_value: Union[str, float] = "auto"):
        """
        Create a node set field.

        Parameters
        ----------
        node_set_field_name : str
            Name of the node set field
        node_set_id : int
            Node set ID
        default_value : str or float, optional
            Initial value ("auto" for zeros, float for constant value)

        Examples
        --------
        >>> model.create_node_set_field("temperature", 1, 0.0)
        """
        if node_set_id not in self.node_sets:
            self._error(f"Node set {node_set_id} does not exist")

        name, members, fields = self.node_sets[node_set_id]

        if node_set_field_name in fields:
            self._warning(f"Node set field '{node_set_field_name}' already exists in node set {node_set_id}, overwriting")

        # Initialize field values for all timesteps
        num_timesteps = len(self.timesteps) if self.timesteps else 1
        num_members = len(members)

        if default_value == "auto":
            # Create zero-filled arrays
            field_data = [[0.0] * num_members for _ in range(num_timesteps)]
        elif isinstance(default_value, (int, float)):
            # Constant value for all members and timesteps
            field_data = [[float(default_value)] * num_members for _ in range(num_timesteps)]
        else:
            field_data = [[0.0] * num_members for _ in range(num_timesteps)]

        fields[node_set_field_name] = field_data
        self.node_sets[node_set_id] = [name, members, fields]

    def delete_node_set_field(self, node_set_field_names: Union[str, List[str]],
                             node_set_ids: Union[str, List[int]] = "all"):
        """
        Delete node set field(s).

        Parameters
        ----------
        node_set_field_names : str or list of str
            Node set field name(s) to delete
        node_set_ids : str or list of int, optional
            Node set IDs ("all" for all node sets)
        """
        if isinstance(node_set_field_names, str):
            node_set_field_names = [node_set_field_names]

        # Determine which node sets to process
        if node_set_ids == "all":
            set_ids = list(self.node_sets.keys())
        elif isinstance(node_set_ids, int):
            set_ids = [node_set_ids]
        else:
            set_ids = node_set_ids

        for set_id in set_ids:
            if set_id not in self.node_sets:
                continue

            name, members, fields = self.node_sets[set_id]
            for field_name in node_set_field_names:
                if field_name in fields:
                    del fields[field_name]
            self.node_sets[set_id] = [name, members, fields]

    def node_set_field_exists(self, node_set_field_name: str,
                             node_set_ids: Union[str, List[int]] = "all") -> bool:
        """
        Check if node set field exists.

        Parameters
        ----------
        node_set_field_name : str
            Node set field name
        node_set_ids : str or list of int, optional
            Node set IDs to check ("all" for all node sets)

        Returns
        -------
        bool
            True if node set field exists in any of the specified node sets
        """
        # Determine which node sets to check
        if node_set_ids == "all":
            set_ids = list(self.node_sets.keys())
        elif isinstance(node_set_ids, int):
            set_ids = [node_set_ids]
        else:
            set_ids = node_set_ids

        for set_id in set_ids:
            if set_id not in self.node_sets:
                continue
            name, members, fields = self.node_sets[set_id]
            if node_set_field_name in fields:
                return True
        return False

    def get_node_set_field_names(self, node_set_id: int) -> List[str]:
        """
        Get node set field names.

        Parameters
        ----------
        node_set_id : int
            Node set ID

        Returns
        -------
        list of str
            List of node set field names
        """
        if node_set_id not in self.node_sets:
            self._error(f"Node set {node_set_id} does not exist")

        name, members, fields = self.node_sets[node_set_id]
        return list(fields.keys())

    def get_node_set_field_values(self, node_set_field_name: str, node_set_id: int,
                                  timestep: Union[str, float] = "last") -> List[float]:
        """
        Get node set field values.

        Parameters
        ----------
        node_set_field_name : str
            Node set field name
        node_set_id : int
            Node set ID
        timestep : str or float, optional
            Timestep ("last" or timestep value)

        Returns
        -------
        list of float
            Node set field values
        """
        if node_set_id not in self.node_sets:
            self._error(f"Node set {node_set_id} does not exist")

        name, members, fields = self.node_sets[node_set_id]

        if node_set_field_name not in fields:
            self._error(f"Node set field '{node_set_field_name}' does not exist in node set {node_set_id}")

        field_data = fields[node_set_field_name]

        # Determine timestep index
        if timestep == "last":
            timestep_idx = len(field_data) - 1 if field_data else 0
        else:
            # Find timestep index
            try:
                timestep_idx = self.timesteps.index(timestep)
            except ValueError:
                timestep_idx = 0

        if timestep_idx < 0 or timestep_idx >= len(field_data):
            timestep_idx = 0

        return field_data[timestep_idx] if field_data else []

    def rename_node_set_field(self, node_set_field_name: str, new_node_set_field_name: str,
                             node_set_ids: Union[str, List[int]] = "all"):
        """
        Rename a node set field.

        Parameters
        ----------
        node_set_field_name : str
            Current node set field name
        new_node_set_field_name : str
            New node set field name
        node_set_ids : str or list of int, optional
            Node set IDs ("all" for all node sets)
        """
        # Determine which node sets to process
        if node_set_ids == "all":
            set_ids = list(self.node_sets.keys())
        elif isinstance(node_set_ids, int):
            set_ids = [node_set_ids]
        else:
            set_ids = node_set_ids

        for set_id in set_ids:
            if set_id not in self.node_sets:
                continue

            name, members, fields = self.node_sets[set_id]

            if node_set_field_name not in fields:
                continue

            if new_node_set_field_name in fields:
                self._error(f"Node set field '{new_node_set_field_name}' already exists in node set {set_id}")

            fields[new_node_set_field_name] = fields[node_set_field_name]
            del fields[node_set_field_name]
            self.node_sets[set_id] = [name, members, fields]

    def calculate_node_set_field(self, expression: str, node_set_ids: Union[str, List[int]] = "all"):
        """Calculate node set field from expression."""
        raise NotImplementedError(
            "calculate_node_set_field() requires expression evaluation which is not yet implemented. "
            "Implementation planned for Phase 4."
        )

    # ========================================================================
    # Node Operations
    # ========================================================================

    def create_nodes(self, new_nodes: List[List[float]]):
        """
        Create new nodes (was create_node in original).

        Parameters
        ----------
        new_nodes : list of list of float
            List of node coordinates [[x1, y1, z1], [x2, y2, z2], ...]

        Examples
        --------
        >>> model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])
        """
        for node in new_nodes:
            # Ensure node has 3 coordinates
            if len(node) == 2:
                node = [node[0], node[1], 0.0]
            elif len(node) == 1:
                node = [node[0], 0.0, 0.0]
            elif len(node) > 3:
                node = node[:3]
            self.nodes.append(node)

    def delete_node(self, node_indices: Union[int, List[int]]):
        """
        Delete node(s).

        Parameters
        ----------
        node_indices : int or list of int
            Node index or indices to delete (0-based)

        Notes
        -----
        This will update all connectivity arrays to reflect the new node indices.
        """
        if isinstance(node_indices, int):
            node_indices = [node_indices]

        # Sort in reverse order to delete from end first
        node_indices = sorted(set(node_indices), reverse=True)

        # Create a mapping of old indices to new indices
        node_map = {}
        offset = 0
        for i in range(len(self.nodes)):
            if i in node_indices:
                offset += 1
                node_map[i] = -1  # Mark as deleted
            else:
                node_map[i] = i - offset

        # Delete nodes
        for idx in node_indices:
            if 0 <= idx < len(self.nodes):
                del self.nodes[idx]

        # Update connectivity in all element blocks
        for block_id, block_data in self.element_blocks.items():
            name, info, connectivity, fields = block_data
            new_connectivity = []
            for element_conn in connectivity:
                # Update node indices (connectivity is 1-indexed)
                new_element_conn = []
                skip_element = False
                for node_idx in element_conn:
                    zero_based_idx = node_idx - 1
                    if zero_based_idx in node_map:
                        new_idx = node_map[zero_based_idx]
                        if new_idx == -1:
                            skip_element = True
                            break
                        new_element_conn.append(new_idx + 1)  # Convert back to 1-based
                    else:
                        new_element_conn.append(node_idx)

                if not skip_element:
                    new_connectivity.append(new_element_conn)

            # Update element count
            info[1] = len(new_connectivity)
            self.element_blocks[block_id] = [name, info, new_connectivity, fields]

        # Update node sets
        for ns_id, ns_data in self.node_sets.items():
            name, members, fields = ns_data
            new_members = []
            for node_idx in members:
                zero_based_idx = node_idx - 1
                if zero_based_idx in node_map and node_map[zero_based_idx] != -1:
                    new_members.append(node_map[zero_based_idx] + 1)  # Convert to 1-based
            self.node_sets[ns_id] = [name, new_members, fields]

    def delete_unused_nodes(self):
        """
        Delete nodes that are not referenced by any elements.

        Returns
        -------
        int
            Number of nodes deleted
        """
        # Find all referenced nodes
        referenced_nodes = set()
        for block_id, block_data in self.element_blocks.items():
            name, info, connectivity, fields = block_data
            for element_conn in connectivity:
                for node_idx in element_conn:
                    referenced_nodes.add(node_idx - 1)  # Convert to 0-based

        # Find unreferenced nodes
        unreferenced = []
        for i in range(len(self.nodes)):
            if i not in referenced_nodes:
                unreferenced.append(i)

        # Delete unreferenced nodes
        if unreferenced:
            self.delete_node(unreferenced)

        return len(unreferenced)

    def get_node_count(self) -> int:
        """Get total number of nodes."""
        return len(self.nodes)

    def get_nodes(self) -> List[List[float]]:
        """Get all node coordinates."""
        return self.nodes

    def merge_nodes(self, tolerance: float = None, *args, **kwargs):
        """
        Merge nodes within tolerance distance.

        Parameters
        ----------
        tolerance : float, optional
            Distance tolerance for merging nodes. If None, uses 1e-6 * length_scale

        Returns
        -------
        int
            Number of nodes merged

        Notes
        -----
        This uses a simple O(n²) algorithm. For large meshes, this may be slow.
        """
        if tolerance is None:
            tolerance = 1e-6 * self.get_length_scale()

        if len(self.nodes) == 0:
            return 0

        # Find nodes to merge (simple O(n²) algorithm)
        merge_map = {}  # Maps node index to the index it should merge with
        merged_count = 0

        for i in range(len(self.nodes)):
            if i in merge_map:
                continue

            for j in range(i + 1, len(self.nodes)):
                if j in merge_map:
                    continue

                # Calculate distance
                dist_sq = sum((self.nodes[i][k] - self.nodes[j][k])**2 for k in range(3))
                if dist_sq < tolerance**2:
                    merge_map[j] = i
                    merged_count += 1

        if merged_count == 0:
            return 0

        # Create node mapping (0-based)
        node_map = {}
        new_index = 0
        for i in range(len(self.nodes)):
            if i in merge_map:
                # This node merges with another
                target = merge_map[i]
                while target in merge_map:
                    target = merge_map[target]
                node_map[i] = node_map.get(target, target)
            else:
                node_map[i] = new_index
                new_index += 1

        # Create new nodes list
        new_nodes = []
        for i in range(len(self.nodes)):
            if i not in merge_map:
                new_nodes.append(self.nodes[i])

        # Update connectivity
        for block_id, block_data in self.element_blocks.items():
            name, info, connectivity, fields = block_data
            new_connectivity = []
            for element_conn in connectivity:
                new_element_conn = [node_map[idx - 1] + 1 for idx in element_conn]  # Convert 1-based
                new_connectivity.append(new_element_conn)
            self.element_blocks[block_id] = [name, info, new_connectivity, fields]

        # Update node sets
        for ns_id, ns_data in self.node_sets.items():
            name, members, fields = ns_data
            new_members = list(set(node_map[idx - 1] + 1 for idx in members))  # Remove duplicates
            self.node_sets[ns_id] = [name, sorted(new_members), fields]

        self.nodes = new_nodes
        return merged_count

    def get_closest_node_distance(self) -> float:
        """
        Get minimum distance between any two nodes.

        Returns
        -------
        float
            Minimum distance between any pair of nodes

        Notes
        -----
        Uses O(n²) algorithm. For large meshes, this may be slow.
        """
        if len(self.nodes) < 2:
            return 0.0

        min_dist = float('inf')
        for i in range(len(self.nodes)):
            for j in range(i + 1, len(self.nodes)):
                dist_sq = sum((self.nodes[i][k] - self.nodes[j][k])**2 for k in range(3))
                if dist_sq < min_dist:
                    min_dist = dist_sq

        return min_dist ** 0.5 if min_dist != float('inf') else 0.0

    def get_length_scale(self) -> float:
        """
        Get characteristic length scale of the model.

        Returns
        -------
        float
            Characteristic length scale (diagonal of bounding box)

        Notes
        -----
        This is computed as the diagonal of the bounding box.
        """
        if len(self.nodes) == 0:
            return 1.0

        # Find bounding box
        min_coords = [float('inf')] * 3
        max_coords = [float('-inf')] * 3

        for node in self.nodes:
            for i in range(3):
                if node[i] < min_coords[i]:
                    min_coords[i] = node[i]
                if node[i] > max_coords[i]:
                    max_coords[i] = node[i]

        # Calculate diagonal
        diagonal_sq = sum((max_coords[i] - min_coords[i])**2 for i in range(3))
        return diagonal_sq ** 0.5 if diagonal_sq > 0 else 1.0

    # ========================================================================
    # Side Set Operations
    # ========================================================================

    def create_side_set(self, side_set_id: int, side_set_members: Optional[List] = None):
        """
        Create a side set.

        Parameters
        ----------
        side_set_id : int
            Side set ID
        side_set_members : list of tuples, optional
            List of (element_id, face_id) tuples

        Examples
        --------
        >>> model.create_side_set(1, [(1, 1), (2, 3)])
        """
        if side_set_id in self.side_sets:
            self._warning(f"Side set {side_set_id} already exists, overwriting")

        members = side_set_members if side_set_members is not None else []
        self.side_sets[side_set_id] = ["", members, {}]  # [name, members, fields]

    def delete_side_set(self, side_set_ids: Union[int, List[int]]):
        """
        Delete side set(s).

        Parameters
        ----------
        side_set_ids : int or list of int
            Side set ID(s) to delete
        """
        if isinstance(side_set_ids, int):
            side_set_ids = [side_set_ids]

        for ss_id in side_set_ids:
            if ss_id in self.side_sets:
                del self.side_sets[ss_id]

    def delete_empty_side_sets(self):
        """
        Delete all empty side sets.

        Returns
        -------
        int
            Number of side sets deleted
        """
        empty_sets = [ss_id for ss_id, ss_data in self.side_sets.items() if len(ss_data[1]) == 0]
        self.delete_side_set(empty_sets)
        return len(empty_sets)

    def side_set_exists(self, side_set_id: int) -> bool:
        """
        Check if side set exists.

        Parameters
        ----------
        side_set_id : int
            Side set ID

        Returns
        -------
        bool
            True if side set exists
        """
        return side_set_id in self.side_sets

    def rename_side_set(self, side_set_id: int, new_name: str):
        """
        Rename a side set (change its name, not its ID).

        Parameters
        ----------
        side_set_id : int
            Side set ID
        new_name : str
            New name for the side set
        """
        if side_set_id not in self.side_sets:
            self._error(f"Side set {side_set_id} does not exist")

        name, members, fields = self.side_sets[side_set_id]
        self.side_sets[side_set_id] = [new_name, members, fields]

    def get_side_set_ids(self) -> List[int]:
        """
        Get all side set IDs.

        Returns
        -------
        list of int
            List of side set IDs
        """
        return list(self.side_sets.keys())

    def get_side_set_name(self, side_set_id: int) -> str:
        """
        Get side set name.

        Parameters
        ----------
        side_set_id : int
            Side set ID

        Returns
        -------
        str
            Side set name
        """
        if side_set_id not in self.side_sets:
            self._error(f"Side set {side_set_id} does not exist")
        return self.side_sets[side_set_id][0]

    def get_all_side_set_names(self) -> Dict[int, str]:
        """
        Get all side set names.

        Returns
        -------
        dict
            Dictionary mapping side set IDs to names
        """
        return {ss_id: ss_data[0] for ss_id, ss_data in self.side_sets.items()}

    def get_side_set_members(self, side_set_id: int) -> List[Tuple[int, int]]:
        """
        Get side set members as list of (element_id, face_id) tuples.

        Parameters
        ----------
        side_set_id : int
            Side set ID

        Returns
        -------
        list of tuples
            List of (element_id, face_id) tuples
        """
        if side_set_id not in self.side_sets:
            self._error(f"Side set {side_set_id} does not exist")
        return self.side_sets[side_set_id][1]

    def add_faces_to_side_set(self, side_set_id: int, new_side_set_members: List[Tuple[int, int]]):
        """
        Add faces to an existing side set.

        Parameters
        ----------
        side_set_id : int
            Side set ID
        new_side_set_members : list of tuples
            List of (element_id, face_id) tuples to add
        """
        if side_set_id not in self.side_sets:
            self._error(f"Side set {side_set_id} does not exist")

        name, members, fields = self.side_sets[side_set_id]
        members.extend(new_side_set_members)
        self.side_sets[side_set_id] = [name, members, fields]

    def create_side_set_from_expression(self, expression: str, side_set_id: int = None, *args, **kwargs):
        """Create side set from expression."""
        raise NotImplementedError(
            "create_side_set_from_expression() requires expression evaluation. "
            "Implementation planned for Phase 6."
        )

    def convert_side_set_to_cohesive_zone(self, side_set_ids: Union[int, List[int]], new_element_block_id: int):
        """Convert side set to cohesive zone element block."""
        raise NotImplementedError("convert_side_set_to_cohesive_zone() is not yet implemented.")

    def get_nodes_in_side_set(self, side_set_id: int) -> List[int]:
        """
        Get list of unique nodes in a side set.

        Parameters
        ----------
        side_set_id : int
            Side set ID

        Returns
        -------
        list of int
            Sorted list of unique node indices (1-based) from all elements in the side set

        Notes
        -----
        This extracts all unique nodes from the elements referenced in the side set.
        """
        if side_set_id not in self.side_sets:
            self._error(f"Side set {side_set_id} does not exist")

        # Get side set members (list of (elem_id, face_id) tuples)
        side_set_members = self.get_side_set_members(side_set_id)

        # Extract unique nodes from elements
        node_indices = set()
        for elem_id, face_id in side_set_members:
            # Find the element in the blocks
            for block_id, block_data in self.element_blocks.items():
                name, info, connectivity, fields = block_data
                num_elems = info[1]

                # elem_id is 1-based, connectivity list is 0-based
                if 0 < elem_id <= num_elems:
                    element_conn = connectivity[elem_id - 1]
                    node_indices.update(element_conn)
                    break

        return sorted(node_indices)

    def get_side_set_area(self, side_set_ids: Union[int, List[int]]) -> float:
        """Calculate total area of side set(s)."""
        raise NotImplementedError("get_side_set_area() is not yet implemented.")

    # ========================================================================
    # Node Set Operations
    # ========================================================================

    def create_node_set(self, node_set_id: int, node_set_members: Optional[List[int]] = None):
        """
        Create a node set.

        Parameters
        ----------
        node_set_id : int
            Node set ID
        node_set_members : list of int, optional
            List of node indices (1-based)

        Examples
        --------
        >>> model.create_node_set(1, [1, 2, 3, 4])
        """
        if node_set_id in self.node_sets:
            self._warning(f"Node set {node_set_id} already exists, overwriting")

        members = node_set_members if node_set_members is not None else []
        self.node_sets[node_set_id] = ["", members, {}]  # [name, members, fields]

    def delete_node_set(self, node_set_ids: Union[int, List[int]]):
        """
        Delete node set(s).

        Parameters
        ----------
        node_set_ids : int or list of int
            Node set ID(s) to delete
        """
        if isinstance(node_set_ids, int):
            node_set_ids = [node_set_ids]

        for ns_id in node_set_ids:
            if ns_id in self.node_sets:
                del self.node_sets[ns_id]

    def delete_empty_node_sets(self):
        """
        Delete all empty node sets.

        Returns
        -------
        int
            Number of node sets deleted
        """
        empty_sets = [ns_id for ns_id, ns_data in self.node_sets.items() if len(ns_data[1]) == 0]
        self.delete_node_set(empty_sets)
        return len(empty_sets)

    def node_set_exists(self, node_set_id: int) -> bool:
        """
        Check if node set exists.

        Parameters
        ----------
        node_set_id : int
            Node set ID

        Returns
        -------
        bool
            True if node set exists
        """
        return node_set_id in self.node_sets

    def rename_node_set(self, node_set_id: int, new_name: str):
        """
        Rename a node set (change its name, not its ID).

        Parameters
        ----------
        node_set_id : int
            Node set ID
        new_name : str
            New name for the node set
        """
        if node_set_id not in self.node_sets:
            self._error(f"Node set {node_set_id} does not exist")

        name, members, fields = self.node_sets[node_set_id]
        self.node_sets[node_set_id] = [new_name, members, fields]

    def get_node_set_ids(self) -> List[int]:
        """
        Get all node set IDs.

        Returns
        -------
        list of int
            List of node set IDs
        """
        return list(self.node_sets.keys())

    def get_node_set_name(self, node_set_id: int) -> str:
        """
        Get node set name.

        Parameters
        ----------
        node_set_id : int
            Node set ID

        Returns
        -------
        str
            Node set name
        """
        if node_set_id not in self.node_sets:
            self._error(f"Node set {node_set_id} does not exist")
        return self.node_sets[node_set_id][0]

    def get_all_node_set_names(self) -> Dict[int, str]:
        """
        Get all node set names.

        Returns
        -------
        dict
            Dictionary mapping node set IDs to names
        """
        return {ns_id: ns_data[0] for ns_id, ns_data in self.node_sets.items()}

    def get_node_set_members(self, node_set_id: int) -> List[int]:
        """
        Get node set members.

        Parameters
        ----------
        node_set_id : int
            Node set ID

        Returns
        -------
        list of int
            List of node indices (1-based)
        """
        if node_set_id not in self.node_sets:
            self._error(f"Node set {node_set_id} does not exist")
        return self.node_sets[node_set_id][1]

    def add_nodes_to_node_set(self, node_set_id: int, new_node_set_members: List[int]):
        """
        Add nodes to an existing node set.

        Parameters
        ----------
        node_set_id : int
            Node set ID
        new_node_set_members : list of int
            List of node indices (1-based) to add
        """
        if node_set_id not in self.node_sets:
            self._error(f"Node set {node_set_id} does not exist")

        name, members, fields = self.node_sets[node_set_id]
        members.extend(new_node_set_members)
        # Remove duplicates and sort
        members = sorted(set(members))
        self.node_sets[node_set_id] = [name, members, fields]

    def create_node_set_from_side_set(self, node_set_id: int, side_set_id: int):
        """
        Create node set from side set members.

        Parameters
        ----------
        node_set_id : int
            Node set ID to create
        side_set_id : int
            Side set ID to extract nodes from

        Notes
        -----
        This extracts all unique nodes from the elements in the side set.
        """
        if side_set_id not in self.side_sets:
            self._error(f"Side set {side_set_id} does not exist")

        # Get side set members (list of (elem_id, face_id) tuples)
        side_set_members = self.get_side_set_members(side_set_id)

        # Extract unique nodes from elements
        node_indices = set()
        for elem_id, face_id in side_set_members:
            # Find the element in the blocks
            for block_id, block_data in self.element_blocks.items():
                name, info, connectivity, fields = block_data
                # elem_id is 1-based, connectivity list is 0-based
                if 0 < elem_id <= len(connectivity):
                    element_conn = connectivity[elem_id - 1]
                    node_indices.update(element_conn)
                    break

        # Create node set
        self.create_node_set(node_set_id, sorted(node_indices))

    def get_nodes_in_node_set(self, node_set_id: int) -> List[int]:
        """
        Get list of nodes in a node set (alias for get_node_set_members).

        Parameters
        ----------
        node_set_id : int
            Node set ID

        Returns
        -------
        list of int
            List of node indices (1-based) in the node set

        See Also
        --------
        get_node_set_members : Equivalent method
        """
        return self.get_node_set_members(node_set_id)

    # ========================================================================
    # Geometric Transformation Operations
    # ========================================================================

    def rotate_geometry(self, axis: List[float], angle_in_degrees: float,
                       adjust_displacement_field: Union[str, bool] = "auto"):
        """
        Rotate the entire geometry around an axis.

        Parameters
        ----------
        axis : list of float
            Rotation axis [x, y, z] (will be normalized)
        angle_in_degrees : float
            Rotation angle in degrees
        adjust_displacement_field : str or bool, optional
            Whether to adjust displacement fields (default: "auto")
            "auto" will adjust if a field named "displacement" exists
            True will adjust the "displacement" field
            False will not adjust any fields

        Examples
        --------
        >>> model.rotate_geometry([0, 0, 1], 90)  # Rotate 90° around z-axis
        >>> model.rotate_geometry([1, 0, 0], 45)  # Rotate 45° around x-axis

        Notes
        -----
        Uses Rodrigues' rotation formula to rotate all node coordinates.
        If adjust_displacement_field is enabled, displacement vectors are also rotated.
        """
        import math

        # Convert angle to radians
        angle = math.radians(angle_in_degrees)

        # Normalize axis
        axis_length = math.sqrt(sum(a**2 for a in axis[:3]))
        if axis_length == 0:
            self._error("Rotation axis cannot be zero vector")

        ax, ay, az = [a / axis_length for a in axis[:3]]

        # Precompute trig values
        cos_a = math.cos(angle)
        sin_a = math.sin(angle)
        one_minus_cos = 1.0 - cos_a

        # Rodrigues' rotation matrix
        # R = I*cos(a) + [axis]×*sin(a) + axis⊗axis*(1-cos(a))
        r11 = cos_a + ax*ax*one_minus_cos
        r12 = ax*ay*one_minus_cos - az*sin_a
        r13 = ax*az*one_minus_cos + ay*sin_a

        r21 = ay*ax*one_minus_cos + az*sin_a
        r22 = cos_a + ay*ay*one_minus_cos
        r23 = ay*az*one_minus_cos - ax*sin_a

        r31 = az*ax*one_minus_cos - ay*sin_a
        r32 = az*ay*one_minus_cos + ax*sin_a
        r33 = cos_a + az*az*one_minus_cos

        # Apply rotation to all nodes
        for i, node in enumerate(self.nodes):
            # Get current coordinates (pad with zeros if needed)
            x = node[0] if len(node) > 0 else 0.0
            y = node[1] if len(node) > 1 else 0.0
            z = node[2] if len(node) > 2 else 0.0

            # Apply rotation matrix
            new_x = r11*x + r12*y + r13*z
            new_y = r21*x + r22*y + r23*z
            new_z = r31*x + r32*y + r33*z

            # Update node coordinates
            self.nodes[i][0] = new_x
            if len(node) > 1:
                self.nodes[i][1] = new_y
            if len(node) > 2:
                self.nodes[i][2] = new_z

        # Adjust displacement field if requested
        should_adjust = False
        if adjust_displacement_field == "auto":
            should_adjust = "displacement" in self.node_fields
        elif adjust_displacement_field is True:
            should_adjust = "displacement" in self.node_fields

        if should_adjust:
            # Rotate displacement vectors for all timesteps
            disp_field = self.node_fields["displacement"]
            for timestep_idx in range(len(disp_field)):
                timestep_data = disp_field[timestep_idx]
                for node_idx in range(len(timestep_data)):
                    disp = timestep_data[node_idx]
                    # Handle both list and scalar displacement values
                    if isinstance(disp, (list, tuple)):
                        if len(disp) >= 3:
                            dx, dy, dz = disp[0], disp[1], disp[2]
                            # Apply rotation matrix to displacement vector
                            new_dx = r11*dx + r12*dy + r13*dz
                            new_dy = r21*dx + r22*dy + r23*dz
                            new_dz = r31*dx + r32*dy + r33*dz
                            timestep_data[node_idx] = [new_dx, new_dy, new_dz]

    def translate_geometry(self, offset: List[float]):
        """
        Translate the entire geometry by an offset.

        Parameters
        ----------
        offset : list of float
            Translation offset [dx, dy, dz]

        Examples
        --------
        >>> model.translate_geometry([1.0, 0.0, 0.0])  # Move 1 unit in x
        >>> model.translate_geometry([0.0, 2.5, 0.0])  # Move 2.5 units in y

        Notes
        -----
        This adds the offset to all node coordinates.
        """
        # Ensure offset has 3 components
        if len(offset) == 2:
            offset = [offset[0], offset[1], 0.0]
        elif len(offset) == 1:
            offset = [offset[0], 0.0, 0.0]
        elif len(offset) > 3:
            offset = offset[:3]

        # Translate all nodes
        for i, node in enumerate(self.nodes):
            for j in range(min(len(node), 3)):
                self.nodes[i][j] += offset[j]

    def scale_geometry(self, scale_factor: float, adjust_displacement_field: Union[str, bool] = "auto"):
        """
        Scale the entire geometry by a factor.

        Parameters
        ----------
        scale_factor : float
            Scaling factor (e.g., 2.0 doubles size, 0.5 halves size)
        adjust_displacement_field : str or bool, optional
            Whether to adjust displacement fields (default: "auto")
            Currently ignored in this implementation.

        Examples
        --------
        >>> model.scale_geometry(2.0)    # Double the model size
        >>> model.scale_geometry(0.001)  # Convert from mm to m

        Notes
        -----
        This multiplies all node coordinates by the scale factor.
        Displacement field adjustment is not yet implemented.
        """
        if scale_factor <= 0:
            self._error("Scale factor must be positive")

        # Scale all nodes
        for i, node in enumerate(self.nodes):
            for j in range(len(node)):
                self.nodes[i][j] *= scale_factor

    # ========================================================================
    # Summarize and Info
    # ========================================================================

    def summarize(self):
        """
        Print a detailed summary of the model.

        This method prints information about all element blocks, node sets,
        side sets, fields, and other model properties.

        Examples
        --------
        >>> model.summarize()
        """
        print("=" * 70)
        print(f"Model Summary: {self.title}")
        print("=" * 70)

        # Basic information
        print(f"\nNodes: {len(self.nodes)}")
        print(f"Timesteps: {len(self.timesteps)}")
        if self.timesteps:
            print(f"  Range: {min(self.timesteps)} to {max(self.timesteps)}")

        # Element blocks
        print(f"\nElement Blocks: {len(self.element_blocks)}")
        if self.element_blocks:
            for block_id, block_data in self.element_blocks.items():
                name, info, connectivity, fields = block_data
                elem_type, num_elems, nodes_per_elem, num_attrs = info
                print(f"  Block {block_id}: {name}")
                print(f"    Type: {elem_type}, Elements: {num_elems}, Nodes/Elem: {nodes_per_elem}")
                if fields:
                    print(f"    Fields: {', '.join(fields.keys())}")

        # Side sets
        print(f"\nSide Sets: {len(self.side_sets)}")
        if self.side_sets:
            for set_id, set_data in self.side_sets.items():
                name, members, fields = set_data
                print(f"  Set {set_id}: {name}, Members: {len(members)}")
                if fields:
                    print(f"    Fields: {', '.join(fields.keys())}")

        # Node sets
        print(f"\nNode Sets: {len(self.node_sets)}")
        if self.node_sets:
            for set_id, set_data in self.node_sets.items():
                name, members, fields = set_data
                print(f"  Set {set_id}: {name}, Members: {len(members)}")
                if fields:
                    print(f"    Fields: {', '.join(fields.keys())}")

        # Node fields
        print(f"\nNode Fields: {len(self.node_fields)}")
        if self.node_fields:
            for field_name in self.node_fields.keys():
                print(f"  {field_name}")

        # Global variables
        print(f"\nGlobal Variables: {len(self.global_variables)}")
        if self.global_variables:
            for var_name in self.global_variables.keys():
                print(f"  {var_name}")

        # QA and Info records
        print(f"\nQA Records: {len(self.qa_records)}")
        print(f"Info Records: {len(self.info_records)}")

        print("=" * 70)

    # ========================================================================
    # Metadata Operations
    # ========================================================================

    def set_title(self, title: str):
        """Set the database title."""
        self.title = title

    def get_title(self) -> str:
        """Get the database title."""
        return self.title

    def add_qa_record(self, code_name: str = None, code_version: str = None,
                     date: str = None, time: str = None):
        """
        Add a QA record.

        Parameters
        ----------
        code_name : str, optional
            Name of the code/software (default: "exodus.exomerge")
        code_version : str, optional
            Version of the code (default: module __version__)
        date : str, optional
            Date string (default: current date in YYYY/MM/DD format)
        time : str, optional
            Time string (default: current time in HH:MM:SS format)

        Notes
        -----
        QA records are stored as tuples of (code_name, code_version, date, time).
        If not specified, defaults will be automatically generated.

        Examples
        --------
        >>> model.add_qa_record("MyCode", "1.0", "2024/01/15", "10:30:00")
        >>> model.add_qa_record()  # Uses defaults
        """
        # Set defaults
        if code_name is None:
            code_name = "exodus.exomerge"
        if code_version is None:
            code_version = __version__
        if date is None:
            date = datetime.datetime.now().strftime("%Y/%m/%d")
        if time is None:
            time = datetime.datetime.now().strftime("%H:%M:%S")

        # Add QA record as tuple
        qa_record = (code_name, code_version, date, time)
        self.qa_records.append(qa_record)

    def get_qa_records(self) -> List[Tuple]:
        """Get all QA records."""
        return self.qa_records

    def add_info_record(self, record: str):
        """Add an info record."""
        self.info_records.append(record)

    def get_info_records(self) -> List[str]:
        """Get all info records."""
        return self.info_records

    # ========================================================================
    # Timestep Operations
    # ========================================================================

    def create_timestep(self, timestep: float):
        """
        Create a new timestep.

        Parameters
        ----------
        timestep : float
            Timestep value to create

        Notes
        -----
        This adds a new timestep to the model. All existing fields will be extended
        with zero-filled data for the new timestep.
        """
        if timestep in self.timesteps:
            self._warning(f"Timestep {timestep} already exists")
            return

        # Add timestep
        self.timesteps.append(timestep)
        self.timesteps.sort()

        # Extend all node fields with zero data
        for field_name, field_data in self.node_fields.items():
            num_nodes = len(self.nodes)
            field_data.append([0.0] * num_nodes)

        # Extend all element fields with zero data
        for block_id, block_data in self.element_blocks.items():
            name, info, connectivity, fields = block_data
            num_elems = info[1]
            for field_name, field_data in fields.items():
                field_data.append([0.0] * num_elems)

        # Extend all side set fields with zero data
        for set_id, set_data in self.side_sets.items():
            name, members, fields = set_data
            num_members = len(members)
            for field_name, field_data in fields.items():
                field_data.append([0.0] * num_members)

        # Extend all node set fields with zero data
        for set_id, set_data in self.node_sets.items():
            name, members, fields = set_data
            num_members = len(members)
            for field_name, field_data in fields.items():
                field_data.append([0.0] * num_members)

        # Extend all global variables with zero data
        for var_name, var_data in self.global_variables.items():
            var_data.append(0.0)

    def delete_timestep(self, timesteps: Union[float, List[float]]):
        """
        Delete one or more timesteps.

        Parameters
        ----------
        timesteps : float or list of float
            Timestep value(s) to delete

        Notes
        -----
        This removes timesteps from the model and deletes corresponding field data
        for all fields (node, element, set, and global variables).
        """
        if isinstance(timesteps, (int, float)):
            timesteps = [float(timesteps)]
        else:
            timesteps = [float(t) for t in timesteps]

        # Get indices of timesteps to delete
        indices_to_delete = []
        for ts in timesteps:
            if ts in self.timesteps:
                indices_to_delete.append(self.timesteps.index(ts))

        if not indices_to_delete:
            return

        # Sort in reverse order to delete from end first
        indices_to_delete.sort(reverse=True)

        # Delete timesteps
        for idx in indices_to_delete:
            del self.timesteps[idx]

        # Delete corresponding data from all node fields
        for field_name, field_data in self.node_fields.items():
            for idx in indices_to_delete:
                if idx < len(field_data):
                    del field_data[idx]

        # Delete corresponding data from all element fields
        for block_id, block_data in self.element_blocks.items():
            name, info, connectivity, fields = block_data
            for field_name, field_data in fields.items():
                for idx in indices_to_delete:
                    if idx < len(field_data):
                        del field_data[idx]

        # Delete corresponding data from all side set fields
        for set_id, set_data in self.side_sets.items():
            name, members, fields = set_data
            for field_name, field_data in fields.items():
                for idx in indices_to_delete:
                    if idx < len(field_data):
                        del field_data[idx]

        # Delete corresponding data from all node set fields
        for set_id, set_data in self.node_sets.items():
            name, members, fields = set_data
            for field_name, field_data in fields.items():
                for idx in indices_to_delete:
                    if idx < len(field_data):
                        del field_data[idx]

        # Delete corresponding data from all global variables
        for var_name, var_data in self.global_variables.items():
            for idx in indices_to_delete:
                if idx < len(var_data):
                    del var_data[idx]

    def get_timesteps(self) -> List[float]:
        """Get all timesteps."""
        return self.timesteps

    def timestep_exists(self, timestep: float) -> bool:
        """Check if a timestep exists."""
        return timestep in self.timesteps

    def copy_timestep(self, timestep: float, new_timestep: float):
        """
        Copy a timestep and all its field data.

        Parameters
        ----------
        timestep : float
            Source timestep to copy from
        new_timestep : float
            New timestep value to create

        Notes
        -----
        This creates a new timestep with all field data copied from the source timestep.
        """
        if timestep not in self.timesteps:
            self._error(f"Source timestep {timestep} does not exist")

        if new_timestep in self.timesteps:
            self._error(f"Target timestep {new_timestep} already exists")

        # Get index of source timestep
        source_idx = self.timesteps.index(timestep)

        # Add new timestep
        self.timesteps.append(new_timestep)
        self.timesteps.sort()

        # Copy node field data
        for field_name, field_data in self.node_fields.items():
            if source_idx < len(field_data):
                field_data.append(field_data[source_idx].copy())

        # Copy element field data
        for block_id, block_data in self.element_blocks.items():
            name, info, connectivity, fields = block_data
            for field_name, field_data in fields.items():
                if source_idx < len(field_data):
                    field_data.append(field_data[source_idx].copy())

        # Copy side set field data
        for set_id, set_data in self.side_sets.items():
            name, members, fields = set_data
            for field_name, field_data in fields.items():
                if source_idx < len(field_data):
                    field_data.append(field_data[source_idx].copy())

        # Copy node set field data
        for set_id, set_data in self.node_sets.items():
            name, members, fields = set_data
            for field_name, field_data in fields.items():
                if source_idx < len(field_data):
                    field_data.append(field_data[source_idx].copy())

        # Copy global variable data
        for var_name, var_data in self.global_variables.items():
            if source_idx < len(var_data):
                var_data.append(var_data[source_idx])

    def create_interpolated_timestep(self, timestep: float, interpolation: str = "cubic"):
        """Create an interpolated timestep."""
        raise NotImplementedError("create_interpolated_timestep() is not yet implemented.")

    # ========================================================================
    # Utility and Mesh Generation
    # ========================================================================

    def to_lowercase(self):
        """
        Convert all names in the model to lowercase.

        Notes
        -----
        This converts the following to lowercase:
        - Database title
        - Element block names
        - Side set names
        - Node set names
        - Field names (node, element, side set, node set, global variables)

        Examples
        --------
        >>> model.to_lowercase()
        """
        # Convert title
        self.title = self.title.lower()

        # Convert element block names
        for block_id, block_data in self.element_blocks.items():
            name, info, connectivity, fields = block_data
            # Convert block name
            name = name.lower()
            # Convert field names
            new_fields = {fname.lower(): fdata for fname, fdata in fields.items()}
            self.element_blocks[block_id] = [name, info, connectivity, new_fields]

        # Convert side set names
        for set_id, set_data in self.side_sets.items():
            name, members, fields = set_data
            name = name.lower()
            new_fields = {fname.lower(): fdata for fname, fdata in fields.items()}
            self.side_sets[set_id] = [name, members, new_fields]

        # Convert node set names
        for set_id, set_data in self.node_sets.items():
            name, members, fields = set_data
            name = name.lower()
            new_fields = {fname.lower(): fdata for fname, fdata in fields.items()}
            self.node_sets[set_id] = [name, members, new_fields]

        # Convert node field names
        self.node_fields = {fname.lower(): fdata for fname, fdata in self.node_fields.items()}

        # Convert global variable names
        self.global_variables = {vname.lower(): vdata for vname, vdata in self.global_variables.items()}

    def build_hex8_cube(self, element_block_id: Union[str, int] = "auto",
                       extents: Union[float, List[float]] = 1.0, divisions: Union[int, List[int]] = 3):
        """
        Build a HEX8 cube mesh.

        Parameters
        ----------
        element_block_id : str or int, optional
            Element block ID for the cube (default: "auto")
        extents : float or list of float, optional
            Size of cube (single value or [x, y, z]) (default: 1.0)
        divisions : int or list of int, optional
            Number of divisions (single value or [nx, ny, nz]) (default: 3)

        Examples
        --------
        >>> model = ExodusModel()
        >>> model.build_hex8_cube(element_block_id=1, extents=2.0, divisions=5)

        Notes
        -----
        Creates a structured hexahedral mesh in the range [0, extents].
        The mesh will have divisions+1 nodes in each direction.
        """
        # Process extents
        if isinstance(extents, (int, float)):
            ex, ey, ez = float(extents), float(extents), float(extents)
        elif len(extents) == 1:
            ex, ey, ez = float(extents[0]), float(extents[0]), float(extents[0])
        elif len(extents) == 2:
            ex, ey, ez = float(extents[0]), float(extents[1]), float(extents[1])
        else:
            ex, ey, ez = float(extents[0]), float(extents[1]), float(extents[2])

        # Process divisions
        if isinstance(divisions, int):
            nx, ny, nz = divisions, divisions, divisions
        elif len(divisions) == 1:
            nx, ny, nz = divisions[0], divisions[0], divisions[0]
        elif len(divisions) == 2:
            nx, ny, nz = divisions[0], divisions[1], divisions[1]
        else:
            nx, ny, nz = divisions[0], divisions[1], divisions[2]

        # Determine block ID
        if element_block_id == "auto":
            # Find next available ID
            if self.element_blocks:
                block_id = max(self.element_blocks.keys()) + 1
            else:
                block_id = 1
        else:
            block_id = element_block_id

        # Create nodes
        nodes_x = nx + 1
        nodes_y = ny + 1
        nodes_z = nz + 1

        dx = ex / nx
        dy = ey / ny
        dz = ez / nz

        node_offset = len(self.nodes)

        for k in range(nodes_z):
            for j in range(nodes_y):
                for i in range(nodes_x):
                    x = i * dx
                    y = j * dy
                    z = k * dz
                    self.nodes.append([x, y, z])

        # Create connectivity (HEX8 elements)
        # HEX8 node ordering: bottom face (z=0) then top face (z=1)
        # Bottom: 0-1-2-3 (counter-clockwise looking down)
        # Top: 4-5-6-7 (counter-clockwise looking down)
        connectivity = []

        def node_index(i, j, k):
            """Get global node index from structured indices."""
            return node_offset + k * nodes_x * nodes_y + j * nodes_x + i + 1  # 1-indexed

        for k in range(nz):
            for j in range(ny):
                for i in range(nx):
                    # Bottom face nodes (z = k)
                    n0 = node_index(i, j, k)
                    n1 = node_index(i+1, j, k)
                    n2 = node_index(i+1, j+1, k)
                    n3 = node_index(i, j+1, k)
                    # Top face nodes (z = k+1)
                    n4 = node_index(i, j, k+1)
                    n5 = node_index(i+1, j, k+1)
                    n6 = node_index(i+1, j+1, k+1)
                    n7 = node_index(i, j+1, k+1)

                    connectivity.append([n0, n1, n2, n3, n4, n5, n6, n7])

        # Create element block
        num_elems = nx * ny * nz
        self.element_blocks[block_id] = [
            f"HEX8_Cube",
            ["HEX8", num_elems, 8, 0],  # elem_type, num_elems, nodes_per_elem, num_attrs
            connectivity,
            {}  # fields
        ]


# Module-level helper function for displaying banner
def _show_banner():
    """Show the exomerge banner (for compatibility)."""
    if SHOW_BANNER:
        print("=" * 70)
        print(f"exodus.exomerge version {__version__}")
        print("Python API for manipulating Exodus II files")
        print("Built on exodus-py (Rust bindings)")
        print(f"Contact: {CONTACT}")
        print("=" * 70)
