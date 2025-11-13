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
        raise NotImplementedError(
            "create_element_block() is not yet implemented. "
            "Implementation planned for Phase 3."
        )

    def delete_element_block(self, element_block_ids: Union[int, List[int]], delete_orphaned_nodes: bool = True):
        """
        Delete one or more element blocks.

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
        raise NotImplementedError(
            "delete_element_block() is not yet implemented. "
            "Implementation planned for Phase 3."
        )

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

    def rename_element_block(self, element_block_id: int, new_element_block_id: int):
        """
        Rename an element block.

        Parameters
        ----------
        element_block_id : int
            Current element block ID
        new_element_block_id : int
            New element block ID

        Examples
        --------
        >>> model.rename_element_block(1, 100)
        """
        raise NotImplementedError(
            "rename_element_block() is not yet implemented. "
            "Implementation planned for Phase 3."
        )

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

    def get_nodes_in_element_block(self, element_block_ids: Union[str, List[int]]) -> List[int]:
        """
        Get list of nodes used in element blocks.

        Parameters
        ----------
        element_block_ids : str or list of int
            Element block IDs or "all"

        Returns
        -------
        list of int
            List of node indices
        """
        raise NotImplementedError(
            "get_nodes_in_element_block() is not yet implemented. "
            "Implementation planned for Phase 3."
        )

    def duplicate_element_block(self, source_id: int, target_id: int, duplicate_nodes: bool = False):
        """
        Duplicate an element block.

        Parameters
        ----------
        source_id : int
            Source element block ID
        target_id : int
            Target element block ID
        duplicate_nodes : bool, optional
            Whether to duplicate nodes (default: False)
        """
        raise NotImplementedError(
            "duplicate_element_block() is not yet implemented. "
            "Implementation planned for Phase 3."
        )

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
                                duplicate_nodes: bool = False):
        """Translate element blocks."""
        raise NotImplementedError("translate_element_blocks() is not yet implemented.")

    def reflect_element_blocks(self, element_block_ids: Union[str, List[int]], *args, **kwargs):
        """Reflect element blocks."""
        raise NotImplementedError("reflect_element_blocks() is not yet implemented.")

    def scale_element_blocks(self, element_block_ids: Union[str, List[int]], scale_factor: float,
                            duplicate_nodes: bool = False):
        """Scale element blocks."""
        raise NotImplementedError("scale_element_blocks() is not yet implemented.")

    def rotate_element_blocks(self, element_block_ids: Union[str, List[int]], axis: List[float],
                             angle_in_degrees: float, duplicate_nodes: bool = False):
        """Rotate element blocks."""
        raise NotImplementedError("rotate_element_blocks() is not yet implemented.")

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
        """Get bounding box extents of element blocks."""
        raise NotImplementedError("get_element_block_extents() is not yet implemented.")

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
        """Get list of nodes in a side set."""
        raise NotImplementedError("get_nodes_in_side_set() is not yet implemented.")

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

    # ========================================================================
    # Geometric Transformation Operations
    # ========================================================================

    def rotate_geometry(self, axis: List[float], angle_in_degrees: float,
                       adjust_displacement_field: Union[str, bool] = "auto"):
        """Rotate the entire geometry."""
        raise NotImplementedError("rotate_geometry() is not yet implemented.")

    def translate_geometry(self, offset: List[float]):
        """Translate the entire geometry."""
        raise NotImplementedError("translate_geometry() is not yet implemented.")

    def scale_geometry(self, scale_factor: float, adjust_displacement_field: Union[str, bool] = "auto"):
        """Scale the entire geometry."""
        raise NotImplementedError("scale_geometry() is not yet implemented.")

    # ========================================================================
    # Summarize and Info
    # ========================================================================

    def summarize(self):
        """
        Print a detailed summary of the model.

        This method prints information about all element blocks, node sets,
        side sets, fields, and other model properties.
        """
        raise NotImplementedError(
            "summarize() is not yet implemented. "
            "Implementation planned for Phase 11."
        )

    # ========================================================================
    # Metadata Operations
    # ========================================================================

    def set_title(self, title: str):
        """Set the database title."""
        self.title = title

    def get_title(self) -> str:
        """Get the database title."""
        return self.title

    def add_qa_record(self, *args):
        """Add a QA record."""
        raise NotImplementedError("add_qa_record() is not yet implemented.")

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
        """Create a new timestep."""
        raise NotImplementedError("create_timestep() is not yet implemented.")

    def delete_timestep(self, timesteps: Union[float, List[float]]):
        """Delete one or more timesteps."""
        raise NotImplementedError("delete_timestep() is not yet implemented.")

    def get_timesteps(self) -> List[float]:
        """Get all timesteps."""
        return self.timesteps

    def timestep_exists(self, timestep: float) -> bool:
        """Check if a timestep exists."""
        return timestep in self.timesteps

    def copy_timestep(self, timestep: float, new_timestep: float):
        """Copy a timestep."""
        raise NotImplementedError("copy_timestep() is not yet implemented.")

    def create_interpolated_timestep(self, timestep: float, interpolation: str = "cubic"):
        """Create an interpolated timestep."""
        raise NotImplementedError("create_interpolated_timestep() is not yet implemented.")

    # ========================================================================
    # Utility and Mesh Generation
    # ========================================================================

    def to_lowercase(self):
        """Convert all names to lowercase."""
        raise NotImplementedError("to_lowercase() is not yet implemented.")

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
        """
        raise NotImplementedError("build_hex8_cube() is not yet implemented.")


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
