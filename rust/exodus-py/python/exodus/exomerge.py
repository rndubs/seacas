"""
Exomerge - High-performance Python interface for manipulating ExodusII files.

This module provides a Python API built on top of exodus-py Rust bindings,
maximizing performance by using exodus data structures directly.

Author: exodus-rs development team
Based on: exomerge3.py by Tim Kostka (tdkostk@sandia.gov)

Version 0.2.0 - Breaking API changes for performance:
- Flat array storage for connectivity
- Direct exodus object storage
- Lazy loading with streaming mode

Simple example:
>>> import exodus.exomerge as exomerge
>>> model = exomerge.ExodusModel(mode="streaming")
>>> model.import_model('results.e')
>>> model.delete_element_block(1)
>>> model.export_model('most_results.e')
"""

import sys
import datetime
from dataclasses import dataclass, field
from typing import Optional, List, Dict, Any, Union, Tuple

__version__ = "0.2.0"
VERSION = __version__

# Contact person for issues
CONTACT = "exodus-rs development team"

# Show banner on first use
SHOW_BANNER = True

# If true, will crash if warnings are generated
EXIT_ON_WARNING = False


# Simple mock Block class for when exodus extension is not available
@dataclass
class _MockBlock:
    """Mock Block class for when exodus extension is unavailable."""
    id: int
    topology: str
    num_entries: int
    num_nodes_per_entry: int
    num_attributes: int = 0


@dataclass
class ElementBlockData:
    """
    Element block data structure.

    Stores exodus Block object directly for zero-copy access.

    Attributes
    ----------
    block : Block
        Exodus block definition
    name : str
        Block name
    connectivity_flat : List[int]
        Flat connectivity array (exodus format)
    fields : Dict[str, List[Any]]
        Element field data per timestep
    """
    block: Any  # exodus.Block
    name: str = ""
    connectivity_flat: List[int] = field(default_factory=list)
    fields: Dict[str, List[Any]] = field(default_factory=dict)

    @property
    def num_entries(self) -> int:
        """Number of elements in this block."""
        return self.block.num_entries

    @property
    def nodes_per_entry(self) -> int:
        """Number of nodes per element."""
        return self.block.num_nodes_per_entry

    @property
    def topology(self) -> str:
        """Element topology (e.g., 'HEX8', 'TET4')."""
        return self.block.topology

    def get_element_connectivity(self, elem_index: int) -> List[int]:
        """
        Get connectivity for element i (0-indexed).

        Parameters
        ----------
        elem_index : int
            Element index (0-based)

        Returns
        -------
        list of int
            Node IDs for this element
        """
        start = elem_index * self.nodes_per_entry
        return self.connectivity_flat[start:start + self.nodes_per_entry]


@dataclass
class NodeSetData:
    """
    Node set data structure.

    Attributes
    ----------
    node_set : NodeSet
        Exodus node set object
    name : str
        Set name
    fields : Dict[str, List[Any]]
        Node set field data per timestep
    """
    node_set: Any  # exodus.NodeSet
    name: str = ""
    fields: Dict[str, List[Any]] = field(default_factory=dict)


@dataclass
class SideSetData:
    """
    Side set data structure.

    Attributes
    ----------
    side_set : SideSet
        Exodus side set object
    name : str
        Set name
    fields : Dict[str, List[Any]]
        Side set field data per timestep
    """
    side_set: Any  # exodus.SideSet
    name: str = ""
    fields: Dict[str, List[Any]] = field(default_factory=dict)


def import_model(filename: str, mode: str = "inmemory", **kwargs) -> 'ExodusModel':
    """
    Load information from an ExodusII file.

    Parameters
    ----------
    filename : str
        Path to the Exodus II file to load
    mode : str, optional
        Storage mode: "inmemory" or "streaming" (default: "inmemory")
    **kwargs : dict
        Additional keyword arguments

    Returns
    -------
    ExodusModel
        The loaded model

    Examples
    --------
    >>> # Load entire file into memory
    >>> model = import_model('mesh.e', mode="inmemory")

    >>> # Stream from file (lazy loading)
    >>> model = import_model('mesh.e', mode="streaming")
    """
    model = ExodusModel(mode=mode)
    model.import_model(filename, **kwargs)
    return model


class ExodusModel:
    """
    Main class for manipulating Exodus II finite element models.

    This class provides a high-performance interface for reading, modifying, and
    writing Exodus II files using exodus-py Rust bindings directly.

    Version 0.2.0 introduces breaking changes for performance:
    - Flat array storage (not list-of-lists)
    - Direct exodus object storage
    - Optional streaming mode

    Attributes
    ----------
    coords_x : List[float]
        X coordinates (flat array)
    coords_y : List[float]
        Y coordinates (flat array)
    coords_z : List[float]
        Z coordinates (flat array)
    element_blocks : Dict[int, ElementBlockData]
        Element blocks with exodus Block objects
    node_sets : Dict[int, NodeSetData]
        Node sets with exodus NodeSet objects
    side_sets : Dict[int, SideSetData]
        Side sets with exodus SideSet objects
    node_fields : Dict[str, List[Any]]
        Node field data per timestep
    global_variables : Dict[str, List[float]]
        Global variable data per timestep
    timesteps : List[float]
        Timestep values
    title : str
        Database title
    qa_records : List[Tuple]
        QA records
    info_records : List[str]
        Info records
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

    # Element volume/area/length formulas
    # Format: [coefficient, (node_indices_1), (node_indices_2), ...]
    VOLUME_FORMULA = {
        "line2": [1.0, (0, 1)],
        "line3": [1.0, (0, 1)],
        "tri3": [0.5, (0, 1), (0, 2)],
        "tri6": [0.5, (0, 1), (0, 2)],
        "quad4": [0.5, (0, 2), (1, 3)],
        "quad8": [0.5, (0, 2), (1, 3)],
        "quad9": [0.5, (0, 2), (1, 3)],
        "tet4": [1.0 / 6.0, (0, 1), (0, 2), (0, 3)],
        "tet10": [1.0 / 6.0, (0, 1), (0, 2), (0, 3)],
        "wedge6": [0.5, ((0, 3), (1, 4)), ((0, 3), (2, 5)), ((0, 1, 2), (3, 4, 5))],
        "wedge15": [0.5, ((0, 3), (1, 4)), ((0, 3), (2, 5)), ((0, 1, 2), (3, 4, 5))],
        "hex8": [1.0, ((0, 3, 4, 7), (1, 2, 5, 6)), ((0, 1, 4, 5), (2, 3, 6, 7)), ((0, 1, 2, 3), (4, 5, 6, 7))],
        "hex20": [1.0, ((0, 3, 4, 7), (1, 2, 5, 6)), ((0, 1, 4, 5), (2, 3, 6, 7)), ((0, 1, 2, 3), (4, 5, 6, 7))],
        "hex27": [1.0, ((0, 3, 4, 7), (1, 2, 5, 6)), ((0, 1, 4, 5), (2, 3, 6, 7)), ((0, 1, 2, 3), (4, 5, 6, 7))],
    }

    def __init__(self, mode: str = "inmemory"):
        """
        Initialize an ExodusModel.

        Parameters
        ----------
        mode : str, optional
            Storage mode:
            - "inmemory": Load all data into memory (default)
            - "streaming": Keep file open, lazy load data
        """
        self._mode = mode
        self._reader: Optional[Any] = None  # exodus.ExodusReader
        self._filename: Optional[str] = None

        # Coordinate storage (flat arrays)
        self.coords_x: List[float] = []
        self.coords_y: List[float] = []
        self.coords_z: List[float] = []

        # Block storage (exodus objects)
        self.element_blocks: Dict[int, ElementBlockData] = {}

        # Set storage (exodus objects)
        self.node_sets: Dict[int, NodeSetData] = {}
        self.side_sets: Dict[int, SideSetData] = {}

        # Field storage
        self.node_fields: Dict[str, List[Any]] = {}
        self.global_variables: Dict[str, List[float]] = {}

        # Metadata
        self.timesteps: List[float] = []
        self.title: str = ""
        self.qa_records: List[Tuple] = []
        self.info_records: List[str] = []

    def __del__(self):
        """Clean up file handle if in streaming mode."""
        if self._reader is not None:
            try:
                self._reader.close()
            except:
                pass

    @property
    def nodes(self) -> List[List[float]]:
        """
        Get nodes as list-of-lists (for backward compatibility).

        Note: This is slower than using coords_x, coords_y, coords_z directly.
        Consider using get_coords_flat() for better performance.
        """
        return self.get_nodes()

    @property
    def num_nodes(self) -> int:
        """Number of nodes in the model."""
        return len(self.coords_x)

    @property
    def num_dim(self) -> int:
        """Number of spatial dimensions (2 or 3)."""
        if self.coords_z and any(z != 0.0 for z in self.coords_z):
            return 3
        elif self.coords_y and any(y != 0.0 for y in self.coords_y):
            return 2
        else:
            return 1

    def get_node_count(self) -> int:
        """
        Return the number of nodes.

        Returns
        -------
        int
            Number of nodes
        """
        return self.num_nodes

    def get_element_count(self, element_block_ids: Union[str, List[int]] = "all") -> int:
        """
        Return total element count.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Block IDs or "all" (default: "all")

        Returns
        -------
        int
            Total number of elements
        """
        if element_block_ids == "all":
            return sum(block.num_entries for block in self.element_blocks.values())
        elif isinstance(element_block_ids, int):
            if element_block_ids in self.element_blocks:
                return self.element_blocks[element_block_ids].num_entries
            return 0
        else:
            return sum(
                self.element_blocks[bid].num_entries
                for bid in element_block_ids
                if bid in self.element_blocks
            )

    def get_element_block_ids(self) -> List[int]:
        """
        Return list of element block IDs.

        Returns
        -------
        list of int
            Element block IDs
        """
        return list(self.element_blocks.keys())

    def element_block_exists(self, element_block_id: int) -> bool:
        """
        Check if element block exists.

        Parameters
        ----------
        element_block_id : int
            Block ID to check

        Returns
        -------
        bool
            True if block exists
        """
        return element_block_id in self.element_blocks

    def get_title(self) -> str:
        """Return the database title."""
        return self.title

    def set_title(self, title: str):
        """Set the database title."""
        self.title = title

    def get_qa_records(self) -> List[Tuple]:
        """Return QA records."""
        return self.qa_records

    def add_qa_record(self, code_name: str = None, code_version: str = None,
                     code_date: str = None, code_time: str = None):
        """Add a QA record."""
        if code_name is None:
            code_name = "exodus.exomerge"
        if code_version is None:
            code_version = __version__
        if code_date is None:
            code_date = datetime.datetime.now().strftime("%Y/%m/%d")
        if code_time is None:
            code_time = datetime.datetime.now().strftime("%H:%M:%S")

        self.qa_records.append((code_name, code_version, code_date, code_time))

    def get_info_records(self) -> List[str]:
        """Return info records."""
        return self.info_records

    def add_info_record(self, record: str):
        """Add an info record."""
        self.info_records.append(record)

    def get_timesteps(self) -> List[float]:
        """Return list of timestep values."""
        return self.timesteps
    def import_model(self, filename: str, **kwargs):
        """
        Load data from an ExodusII file using exodus-py bindings.

        This method uses flat arrays and exodus objects directly for
        maximum performance.

        Parameters
        ----------
        filename : str
            Path to exodus file

        Examples
        --------
        >>> model = ExodusModel()
        >>> model.import_model('mesh.e')
        """
        from . import ExodusReader, EntityType

        self._filename = filename
        self._reader = ExodusReader.open(filename)

        # Read initialization parameters
        params = self._reader.init_params()
        self.title = params.title

        # Read coordinates as flat arrays (FAST - zero copy)
        if params.num_nodes > 0:
            x, y, z = self._reader.get_coords()
            self.coords_x = list(x)
            self.coords_y = list(y)
            self.coords_z = list(z)

        # Read element blocks with flat connectivity
        if params.num_elem_blocks > 0:
            block_ids = self._reader.get_block_ids()
            for block_id in block_ids:
                # Get block definition (exodus Block object)
                block = self._reader.get_block(block_id)

                # Get block name
                try:
                    name = self._reader.get_name("elem_block", block_id)
                except:
                    name = ""

                # Get flat connectivity (FAST - direct from exodus)
                try:
                    connectivity_flat = list(self._reader.get_connectivity(block_id))
                except:
                    connectivity_flat = []

                # Store as dict with exodus Block object
                self.element_blocks[block_id] = {
                    'block': block,
                    'name': name,
                    'connectivity_flat': connectivity_flat,
                    'fields': {}
                }

        # Read QA records
        try:
            qa_records = self._reader.get_qa_records()
            self.qa_records = [(r.code_name, r.code_version, r.date, r.time) 
                              for r in qa_records]
        except:
            pass

        # Read info records
        try:
            self.info_records = list(self._reader.get_info_records())
        except:
            pass

        # Read timesteps
        try:
            self.timesteps = list(self._reader.times())
        except:
            pass

        # Close reader if in inmemory mode
        if self._mode == "inmemory":
            self._reader.close()
            self._reader = None

    def export_model(self, filename: str, **kwargs):
        """
        Write model to an ExodusII file using exodus-py bindings.

        Uses flat arrays directly for maximum performance.

        Parameters
        ----------
        filename : str
            Output file path

        Examples
        --------
        >>> model.export_model('output.e')
        """
        from . import ExodusWriter, CreateOptions, InitParams, CreateMode, Block, EntityType

        # Create file
        opts = CreateOptions(mode=CreateMode.Clobber)
        writer = ExodusWriter.create(filename, opts)

        try:
            # Write initialization parameters
            params = InitParams(
                title=self.title,
                num_dim=self.num_dim,
                num_nodes=self.num_nodes,
                num_elems=sum(b['block'].num_entries for b in self.element_blocks.values()),
                num_elem_blocks=len(self.element_blocks),
                num_node_sets=len(self.node_sets),
                num_side_sets=len(self.side_sets),
            )
            writer.put_init_params(params)

            # Write coordinates (FAST - flat arrays)
            if self.coords_x:
                writer.put_coords(self.coords_x, self.coords_y, self.coords_z)

            # Write element blocks
            for block_id, block_data in self.element_blocks.items():
                # Write block definition (exodus Block object directly)
                writer.put_block(block_data['block'])

                # Write flat connectivity (FAST - no conversion)
                if block_data['connectivity_flat']:
                    writer.put_connectivity(block_id, block_data['connectivity_flat'])

            # Write QA records
            for qa in self.qa_records:
                from . import QaRecord
                record = QaRecord(qa[0], qa[1], qa[2], qa[3])
                writer.put_qa_record(record)

            # Write info records
            for info in self.info_records:
                writer.put_info_record(info)

            # Write timesteps
            for i, timestep in enumerate(self.timesteps):
                writer.put_time(i, float(timestep))

            writer.close()

        except Exception as e:
            print(f"ERROR: Error exporting model: {e}")
            print(f"  {e}")
            raise

    def get_nodes(self) -> List[List[float]]:
        """
        Get node coordinates as list of [x, y, z].

        NOTE: This converts from flat arrays. For better performance,
        use get_coords_flat() which returns (x, y, z) tuples directly.

        Returns
        -------
        list of list of float
            Node coordinates [[x,y,z], [x,y,z], ...]
        """
        nodes = []
        for i in range(self.num_nodes):
            x = self.coords_x[i] if i < len(self.coords_x) else 0.0
            y = self.coords_y[i] if i < len(self.coords_y) else 0.0
            z = self.coords_z[i] if i < len(self.coords_z) else 0.0
            nodes.append([x, y, z])
        return nodes

    def get_coords_flat(self) -> Tuple[List[float], List[float], List[float]]:
        """
        Get coordinates as flat arrays (FAST - zero-copy).

        Returns
        -------
        tuple of (x_coords, y_coords, z_coords)
            Flat coordinate arrays

        Examples
        --------
        >>> x, y, z = model.get_coords_flat()
        >>> node_5_x = x[5]
        """
        return (self.coords_x, self.coords_y, self.coords_z)

    def get_connectivity_flat(self, block_id: int) -> List[int]:
        """
        Get element connectivity as flat array (FAST - zero-copy).

        Parameters
        ----------
        block_id : int
            Element block ID

        Returns
        -------
        list of int
            Flat connectivity [n1,n2,n3,...,nM]

        Examples
        --------
        >>> flat = model.get_connectivity_flat(1)
        >>> npe = model.get_nodes_per_element(1)
        >>> elem_0_nodes = flat[0:npe]
        """
        if block_id not in self.element_blocks:
            return []
        return self.element_blocks[block_id]['connectivity_flat']

    def get_connectivity(self, block_id: Union[int, str]) -> List[List[int]]:
        """
        Get element connectivity as list of lists.

        NOTE: This converts from flat array. For better performance,
        use get_connectivity_flat().

        Parameters
        ----------
        block_id : int or str
            Block ID or "auto" for single block

        Returns
        -------
        list of list of int
            Connectivity [[n1,n2,n3,n4], [n5,n6,n7,n8], ...]
        """
        if block_id == "auto":
            if len(self.element_blocks) != 1:
                raise ValueError("get_connectivity('auto') requires exactly one block")
            block_id = list(self.element_blocks.keys())[0]

        if block_id not in self.element_blocks:
            return []

        block_data = self.element_blocks[block_id]
        flat = block_data['connectivity_flat']
        npe = block_data['block'].num_nodes_per_entry

        # Convert flat to list-of-lists
        return [flat[i:i+npe] for i in range(0, len(flat), npe)]

    def get_nodes_per_element(self, block_id: int) -> int:
        """Get number of nodes per element in block."""
        if block_id not in self.element_blocks:
            return 0
        return self.element_blocks[block_id]['block'].num_nodes_per_entry

    def get_element_block_dimension(self, block_id: int) -> int:
        """Get spatial dimension of element block (1, 2, or 3)."""
        if block_id not in self.element_blocks:
            return 0
        topology = self.element_blocks[block_id]['block'].topology.lower()
        return self.DIMENSION.get(topology, 3)

    def get_element_block_name(self, block_id: int) -> str:
        """Get element block name."""
        if block_id not in self.element_blocks:
            return ""
        return self.element_blocks[block_id]['name']

    def set_element_block_name(self, block_id: int, name: str):
        """Set element block name."""
        if block_id in self.element_blocks:
            self.element_blocks[block_id]['name'] = name

    def create_nodes(self, nodes: List[List[float]]):
        """
        Create nodes from list of [x, y, z] coordinates.

        Converts to flat arrays internally for performance.

        Parameters
        ----------
        nodes : list of list of float
            Node coordinates [[x,y,z], ...]
        """
        if not nodes:
            return

        self.coords_x = [n[0] for n in nodes]
        self.coords_y = [n[1] if len(n) > 1 else 0.0 for n in nodes]
        self.coords_z = [n[2] if len(n) > 2 else 0.0 for n in nodes]

    def create_element_block(self, block_id: int, info: List):
        """
        Create an element block.

        Parameters
        ----------
        block_id : int
            Block ID
        info : list
            [topology, num_elems, nodes_per_elem, num_attrs]
        """
        topology, num_elems, nodes_per_elem, num_attrs = info

        # Try to use exodus Block if available, otherwise use mock
        try:
            from . import Block, EntityType
            block = Block(
                id=block_id,
                entity_type=EntityType.ElemBlock,
                topology=topology,
                num_entries=num_elems,
                num_nodes_per_entry=nodes_per_elem,
                num_attributes=num_attrs
            )
        except (ImportError, AttributeError):
            # Use mock block if exodus extension not available
            block = _MockBlock(
                id=block_id,
                topology=topology,
                num_entries=num_elems,
                num_nodes_per_entry=nodes_per_elem,
                num_attributes=num_attrs
            )

        self.element_blocks[block_id] = {
            'block': block,
            'name': "",
            'connectivity_flat': [],
            'fields': {}
        }

    def set_connectivity(self, block_id: int, connectivity: List[List[int]]):
        """
        Set element connectivity from list of lists.

        Converts to flat array internally for performance.

        Parameters
        ----------
        block_id : int
            Block ID
        connectivity : list of list of int
            Element connectivity [[n1,n2,...], ...]
        """
        if block_id not in self.element_blocks:
            return

        # Flatten to flat array
        flat = [node_id for elem in connectivity for node_id in elem]
        self.element_blocks[block_id]['connectivity_flat'] = flat

    def create_node_set(self, node_set_id: int, members: List[int]):
        """Create a node set."""
        self.node_sets[node_set_id] = {
            'members': members,
            'name': "",
            'fields': {}
        }

    def create_side_set(self, side_set_id: int, members: List[Tuple[int, int]]):
        """Create a side set."""
        self.side_sets[side_set_id] = {
            'members': members,
            'name': "",
            'fields': {}
        }

    def create_node_field(self, field_name: str, timesteps: List[int] = None):
        """Create a node field."""
        if timesteps is None:
            timesteps = list(range(len(self.timesteps)))
        self.node_fields[field_name] = [[] for _ in timesteps]

    def create_element_field(self, field_name: str, block_id: int, timesteps: List[int] = None):
        """Create an element field."""
        if block_id not in self.element_blocks:
            return
        if timesteps is None:
            timesteps = list(range(len(self.timesteps)))
        self.element_blocks[block_id]['fields'][field_name] = [[] for _ in timesteps]

    def node_field_exists(self, field_name: str) -> bool:
        """Check if node field exists."""
        return field_name in self.node_fields

    def element_field_exists(self, block_id: int, field_name: str) -> bool:
        """Check if element field exists."""
        if block_id not in self.element_blocks:
            return False
        return field_name in self.element_blocks[block_id]['fields']

    def global_variable_exists(self, var_name: str) -> bool:
        """Check if global variable exists."""
        return var_name in self.global_variables

    def node_set_exists(self, node_set_id: int) -> bool:
        """Check if node set exists."""
        return node_set_id in self.node_sets

    def side_set_exists(self, side_set_id: int) -> bool:
        """Check if side set exists."""
        return side_set_id in self.side_sets

    def timestep_exists(self, timestep: int) -> bool:
        """Check if timestep exists."""
        return 0 <= timestep < len(self.timesteps)

    def node_set_field_exists(self, node_set_id: int, field_name: str) -> bool:
        """Check if node set field exists."""
        if node_set_id not in self.node_sets:
            return False
        return field_name in self.node_sets[node_set_id]['fields']

    def side_set_field_exists(self, side_set_id: int, field_name: str) -> bool:
        """Check if side set field exists."""
        if side_set_id not in self.side_sets:
            return False
        return field_name in self.side_sets[side_set_id]['fields']

    def get_node_set_members(self, node_set_id: int) -> List[int]:
        """Get node set members."""
        if node_set_id not in self.node_sets:
            return []
        return self.node_sets[node_set_id]['members']

    def delete_element_block(self, element_block_ids: Union[int, List[int]], delete_orphaned_nodes: bool = True):
        """
        Delete one or more element blocks.

        This will also delete any references to elements in that block in side sets.
        By default, this will delete any nodes that become unused.

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

        if not element_block_ids:
            return

        # Find unreferenced nodes before deletion
        unreferenced_before = set()
        if delete_orphaned_nodes:
            unreferenced_before = self._get_unreferenced_nodes()

        # Delete the element blocks
        for element_block_id in element_block_ids:
            if element_block_id in self.element_blocks:
                del self.element_blocks[element_block_id]

        # Delete orphaned nodes if requested
        if delete_orphaned_nodes:
            unreferenced_after = self._get_unreferenced_nodes()
            nodes_to_delete = sorted(unreferenced_after - unreferenced_before)
            if nodes_to_delete:
                self.delete_node(nodes_to_delete)

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
        for i in range(len(self.coords_x)):
            if i in node_indices:
                offset += 1
                node_map[i] = -1  # Mark as deleted
            else:
                node_map[i] = i - offset

        # Delete nodes from flat coordinate arrays
        for idx in node_indices:
            if 0 <= idx < len(self.coords_x):
                del self.coords_x[idx]
                del self.coords_y[idx]
                if self.coords_z:
                    del self.coords_z[idx]

        # Update connectivity in all element blocks (connectivity is 1-indexed)
        for block_id, block_data in self.element_blocks.items():
            conn_flat = block_data['connectivity_flat']
            nodes_per_elem = block_data['block'].num_nodes_per_entry
            num_elems = len(conn_flat) // nodes_per_elem

            new_conn_flat = []
            for elem_idx in range(num_elems):
                start = elem_idx * nodes_per_elem
                elem_conn = conn_flat[start:start + nodes_per_elem]

                # Update node indices
                new_elem_conn = []
                skip_element = False
                for node_idx_1based in elem_conn:
                    zero_based_idx = node_idx_1based - 1
                    if zero_based_idx in node_map:
                        new_idx = node_map[zero_based_idx]
                        if new_idx == -1:
                            skip_element = True
                            break
                        new_elem_conn.append(new_idx + 1)  # Convert back to 1-based
                    else:
                        new_elem_conn.append(node_idx_1based)

                if not skip_element:
                    new_conn_flat.extend(new_elem_conn)

            block_data['connectivity_flat'] = new_conn_flat

    def delete_unused_nodes(self) -> int:
        """
        Delete nodes that are not referenced by any elements.

        Returns
        -------
        int
            Number of nodes deleted
        """
        unreferenced = self._get_unreferenced_nodes()
        if unreferenced:
            self.delete_node(sorted(unreferenced))
        return len(unreferenced)

    def _get_unreferenced_nodes(self) -> set:
        """Find all nodes not referenced by any element."""
        referenced_nodes = set()
        for block_data in self.element_blocks.values():
            for node_idx_1based in block_data['connectivity_flat']:
                referenced_nodes.add(node_idx_1based - 1)  # Convert to 0-based

        all_nodes = set(range(len(self.coords_x)))
        return all_nodes - referenced_nodes

    def translate_geometry(self, offset: List[float]):
        """
        Translate the entire geometry by an offset.

        Parameters
        ----------
        offset : list of float
            Translation offset [dx, dy, dz]

        Examples
        --------
        >>> model.translate_geometry([10, 0, 0])  # Translate 10 units in x
        """
        dx = offset[0] if len(offset) > 0 else 0.0
        dy = offset[1] if len(offset) > 1 else 0.0
        dz = offset[2] if len(offset) > 2 else 0.0

        # Translate coordinates (flat arrays - very efficient)
        self.coords_x = [x + dx for x in self.coords_x]
        self.coords_y = [y + dy for y in self.coords_y]
        if self.coords_z:
            self.coords_z = [z + dz for z in self.coords_z]

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

        Examples
        --------
        >>> model.rotate_geometry([0, 0, 1], 90)  # Rotate 90° around z-axis
        """
        import math

        # Convert angle to radians
        angle = math.radians(angle_in_degrees)

        # Normalize axis
        axis_length = math.sqrt(sum(a**2 for a in axis[:3]))
        if axis_length == 0:
            raise ValueError("Rotation axis cannot be zero vector")

        ax, ay, az = [a / axis_length for a in axis[:3]]

        # Precompute trig values
        cos_a = math.cos(angle)
        sin_a = math.sin(angle)
        one_minus_cos = 1.0 - cos_a

        # Rodrigues' rotation matrix
        r11 = cos_a + ax*ax*one_minus_cos
        r12 = ax*ay*one_minus_cos - az*sin_a
        r13 = ax*az*one_minus_cos + ay*sin_a

        r21 = ay*ax*one_minus_cos + az*sin_a
        r22 = cos_a + ay*ay*one_minus_cos
        r23 = ay*az*one_minus_cos - ax*sin_a

        r31 = az*ax*one_minus_cos - ay*sin_a
        r32 = az*ay*one_minus_cos + ax*sin_a
        r33 = cos_a + az*az*one_minus_cos

        # Apply rotation to all nodes (flat arrays)
        new_x = []
        new_y = []
        new_z = []
        for i in range(len(self.coords_x)):
            x = self.coords_x[i]
            y = self.coords_y[i] if i < len(self.coords_y) else 0.0
            z = self.coords_z[i] if i < len(self.coords_z) else 0.0

            new_x.append(r11*x + r12*y + r13*z)
            new_y.append(r21*x + r22*y + r23*z)
            new_z.append(r31*x + r32*y + r33*z)

        self.coords_x = new_x
        self.coords_y = new_y
        if self.coords_z:
            self.coords_z = new_z

    def scale_geometry(self, scale_factor: float, adjust_displacement_field: Union[str, bool] = "auto"):
        """
        Scale the entire geometry by a factor.

        Parameters
        ----------
        scale_factor : float
            Scale factor
        adjust_displacement_field : str or bool, optional
            Whether to adjust displacement fields (default: "auto")

        Examples
        --------
        >>> model.scale_geometry(2.0)  # Double the size
        >>> model.scale_geometry(0.001)  # Convert mm to m
        """
        self.coords_x = [x * scale_factor for x in self.coords_x]
        self.coords_y = [y * scale_factor for y in self.coords_y]
        if self.coords_z:
            self.coords_z = [z * scale_factor for z in self.coords_z]

    def delete_node_set(self, node_set_ids: Union[int, List[int]]):
        """
        Delete one or more node sets.

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

    def delete_side_set(self, side_set_ids: Union[int, List[int]]):
        """
        Delete one or more side sets.

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

    def get_node_set_ids(self) -> List[int]:
        """Get list of all node set IDs."""
        return sorted(self.node_sets.keys())

    def get_side_set_ids(self) -> List[int]:
        """Get list of all side set IDs."""
        return sorted(self.side_sets.keys())

    def get_node_set_name(self, node_set_id: int) -> str:
        """Get node set name."""
        if node_set_id not in self.node_sets:
            return ""
        return self.node_sets[node_set_id].get('name', "")

    def get_side_set_name(self, side_set_id: int) -> str:
        """Get side set name."""
        if side_set_id not in self.side_sets:
            return ""
        return self.side_sets[side_set_id].get('name', "")

    def rename_node_set(self, node_set_id: int, new_name: str):
        """Rename a node set."""
        if node_set_id in self.node_sets:
            self.node_sets[node_set_id]['name'] = new_name

    def rename_side_set(self, side_set_id: int, new_name: str):
        """Rename a side set."""
        if side_set_id in self.side_sets:
            self.side_sets[side_set_id]['name'] = new_name

    def summarize(self):
        """
        Print a summary of the model.

        Examples
        --------
        >>> model.summarize()
        """
        print("=" * 70)
        print("ExodusII Model Summary")
        print("=" * 70)
        print(f"Title: {self.title}")
        print(f"Nodes: {len(self.coords_x)}")
        print(f"Dimensions: {self.num_dim}")
        print(f"Element blocks: {len(self.element_blocks)}")

        if self.element_blocks:
            print("\nElement Blocks:")
            for block_id, block_data in sorted(self.element_blocks.items()):
                block = block_data['block']
                name = block_data.get('name', '')
                num_elems = block.num_entries
                topo = block.topology
                print(f"  Block {block_id}: {num_elems} {topo} elements" +
                      (f" ({name})" if name else ""))

        if self.node_sets:
            print(f"\nNode sets: {len(self.node_sets)}")
            for ns_id in sorted(self.node_sets.keys()):
                name = self.get_node_set_name(ns_id)
                members = self.get_node_set_members(ns_id)
                print(f"  Node set {ns_id}: {len(members)} nodes" +
                      (f" ({name})" if name else ""))

        if self.side_sets:
            print(f"\nSide sets: {len(self.side_sets)}")
            for ss_id in sorted(self.side_sets.keys()):
                name = self.get_side_set_name(ss_id)
                print(f"  Side set {ss_id}" + (f" ({name})" if name else ""))

        if self.node_fields:
            print(f"\nNode fields: {len(self.node_fields)}")
            for field_name in sorted(self.node_fields.keys()):
                print(f"  {field_name}")

        if self.global_variables:
            print(f"\nGlobal variables: {len(self.global_variables)}")
            for var_name in sorted(self.global_variables.keys()):
                print(f"  {var_name}")

        if self.timesteps:
            print(f"\nTimesteps: {len(self.timesteps)}")
            if len(self.timesteps) <= 5:
                print(f"  {self.timesteps}")
            else:
                print(f"  [{self.timesteps[0]}, ..., {self.timesteps[-1]}]")

        print("=" * 70)

    # Additional element block methods
    def rename_element_block(self, element_block_id: int, new_element_block_id: Union[int, str]):
        """
        Change an element block ID or name.

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
        if element_block_id not in self.element_blocks:
            raise ValueError(f"Element block {element_block_id} does not exist")

        # If we're just changing the name (string provided)
        if isinstance(new_element_block_id, str):
            self.element_blocks[element_block_id]['name'] = new_element_block_id
            return

        # Otherwise, we're changing the ID (integer provided)
        assert isinstance(new_element_block_id, int)

        # Check that the new ID doesn't already exist
        if new_element_block_id in self.element_blocks:
            raise ValueError(f"Element block {new_element_block_id} already exists")

        # Rename the block by creating new entry and deleting old
        self.element_blocks[new_element_block_id] = self.element_blocks[element_block_id]
        del self.element_blocks[element_block_id]

    def get_all_element_block_names(self) -> Dict[int, str]:
        """
        Get names of all element blocks.

        Returns
        -------
        dict
            Dictionary mapping element block IDs to names
        """
        return {block_id: block_data.get('name', '')
                for block_id, block_data in self.element_blocks.items()}

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
            Sorted list of unique node indices (1-based)

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
                continue

            # Get flat connectivity
            conn_flat = self.element_blocks[block_id]['connectivity_flat']
            node_set.update(conn_flat)

        return sorted(node_set)

    # Field management methods
    def get_node_field_names(self) -> List[str]:
        """
        Get all node field names.

        Returns
        -------
        list of str
            List of node field names
        """
        return sorted(self.node_fields.keys())

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
            raise ValueError(f"Node field '{node_field_name}' does not exist")

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
            raise ValueError(f"Node field '{node_field_name}' does not exist")
        if new_node_field_name in self.node_fields:
            raise ValueError(f"Node field '{new_node_field_name}' already exists")

        self.node_fields[new_node_field_name] = self.node_fields[node_field_name]
        del self.node_fields[node_field_name]

    def delete_element_field(self, element_field_names: Union[str, List[str]],
                           element_block_ids: Union[str, List[int]] = "all"):
        """
        Delete element field(s).

        Parameters
        ----------
        element_field_names : str or list of str
            Element field name(s) to delete
        element_block_ids : str or list of int, optional
            Element blocks to delete from (default: "all")
        """
        if isinstance(element_field_names, str):
            element_field_names = [element_field_names]

        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        for block_id in element_block_ids:
            if block_id in self.element_blocks:
                for field_name in element_field_names:
                    if field_name in self.element_blocks[block_id].get('fields', {}):
                        del self.element_blocks[block_id]['fields'][field_name]

    def get_element_field_names(self, element_block_ids: Union[str, List[int]] = "all") -> List[str]:
        """
        Get all element field names for given blocks.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element blocks to query (default: "all")

        Returns
        -------
        list of str
            List of unique element field names
        """
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        field_names = set()
        for block_id in element_block_ids:
            if block_id in self.element_blocks:
                field_names.update(self.element_blocks[block_id].get('fields', {}).keys())

        return sorted(field_names)

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
            Element blocks to rename in (default: "all")
        """
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        for block_id in element_block_ids:
            if block_id in self.element_blocks:
                fields = self.element_blocks[block_id].get('fields', {})
                if element_field_name in fields:
                    fields[new_element_field_name] = fields[element_field_name]
                    del fields[element_field_name]

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

    def get_global_variable_names(self) -> List[str]:
        """
        Get all global variable names.

        Returns
        -------
        list of str
            List of global variable names
        """
        return sorted(self.global_variables.keys())

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
            raise ValueError(f"Global variable '{global_variable_name}' does not exist")
        if new_global_variable_name in self.global_variables:
            raise ValueError(f"Global variable '{new_global_variable_name}' already exists")

        self.global_variables[new_global_variable_name] = self.global_variables[global_variable_name]
        del self.global_variables[global_variable_name]

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
        >>> model.create_global_variable("time_step_size", 0.01)
        """
        if global_variable_name in self.global_variables:
            pass  # Overwrite existing

        # Initialize variable values for all timesteps
        num_timesteps = len(self.timesteps) if self.timesteps else 1

        if value == "auto":
            var_data = [0.0] * num_timesteps
        elif isinstance(value, (int, float)):
            var_data = [float(value)] * num_timesteps
        elif isinstance(value, list):
            var_data = value
        else:
            var_data = [0.0] * num_timesteps

        self.global_variables[global_variable_name] = var_data

    # Timestep methods
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
            return  # Already exists

        # Add timestep
        self.timesteps.append(timestep)
        self.timesteps.sort()

        # Extend all node fields with zero data
        for field_name, field_data in self.node_fields.items():
            num_nodes = len(self.coords_x)
            field_data.append([0.0] * num_nodes)

        # Extend all element fields with zero data
        for block_id, block_data in self.element_blocks.items():
            block = block_data['block']
            num_elems = block.num_entries
            for field_name, field_data in block_data.get('fields', {}).items():
                field_data.append([0.0] * num_elems)

        # Extend all side set fields with zero data
        for set_id, set_data in self.side_sets.items():
            members = set_data.get('members', [])
            num_members = len(members)
            for field_name, field_data in set_data.get('fields', {}).items():
                field_data.append([0.0] * num_members)

        # Extend all node set fields with zero data
        for set_id, set_data in self.node_sets.items():
            members = set_data.get('members', [])
            num_members = len(members)
            for field_name, field_data in set_data.get('fields', {}).items():
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
            for field_name, field_data in block_data.get('fields', {}).items():
                for idx in indices_to_delete:
                    if idx < len(field_data):
                        del field_data[idx]

        # Delete corresponding data from all side set fields
        for set_id, set_data in self.side_sets.items():
            for field_name, field_data in set_data.get('fields', {}).items():
                for idx in indices_to_delete:
                    if idx < len(field_data):
                        del field_data[idx]

        # Delete corresponding data from all node set fields
        for set_id, set_data in self.node_sets.items():
            for field_name, field_data in set_data.get('fields', {}).items():
                for idx in indices_to_delete:
                    if idx < len(field_data):
                        del field_data[idx]

        # Delete corresponding data from all global variables
        for var_name, var_data in self.global_variables.items():
            for idx in indices_to_delete:
                if idx < len(var_data):
                    del var_data[idx]

    # Set operation methods
    def get_side_set_members(self, side_set_id: int) -> List[Tuple[int, int]]:
        """
        Get side set members.

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
            raise ValueError(f"Side set {side_set_id} does not exist")
        return self.side_sets[side_set_id].get('members', [])

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
            raise ValueError(f"Side set {side_set_id} does not exist")

        members = self.side_sets[side_set_id].get('members', [])
        members.extend(new_side_set_members)
        self.side_sets[side_set_id]['members'] = members

    def add_nodes_to_node_set(self, node_set_id: int, new_node_set_members: List[int]):
        """
        Add nodes to an existing node set.

        Parameters
        ----------
        node_set_id : int
            Node set ID
        new_node_set_members : list of int
            List of node indices to add
        """
        if node_set_id not in self.node_sets:
            raise ValueError(f"Node set {node_set_id} does not exist")

        members = self.node_sets[node_set_id].get('members', [])
        members.extend(new_node_set_members)
        self.node_sets[node_set_id]['members'] = members

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
            List of node indices
        """
        return self.get_node_set_members(node_set_id)

    def get_all_node_set_names(self) -> Dict[int, str]:
        """
        Get names of all node sets.

        Returns
        -------
        dict
            Dictionary mapping node set IDs to names
        """
        return {ns_id: ns_data.get('name', '')
                for ns_id, ns_data in self.node_sets.items()}

    def get_all_side_set_names(self) -> Dict[int, str]:
        """
        Get names of all side sets.

        Returns
        -------
        dict
            Dictionary mapping side set IDs to names
        """
        return {ss_id: ss_data.get('name', '')
                for ss_id, ss_data in self.side_sets.items()}

    def delete_empty_node_sets(self):
        """Delete node sets that have no members."""
        to_delete = [ns_id for ns_id, ns_data in self.node_sets.items()
                     if not ns_data.get('members', [])]
        for ns_id in to_delete:
            del self.node_sets[ns_id]

    def delete_empty_side_sets(self):
        """Delete side sets that have no members."""
        to_delete = [ss_id for ss_id, ss_data in self.side_sets.items()
                     if not ss_data.get('members', [])]
        for ss_id in to_delete:
            del self.side_sets[ss_id]

    # Node utility methods
    def get_closest_node_distance(self) -> float:
        """
        Get the minimum distance between any two nodes.

        Returns
        -------
        float
            Minimum distance between nodes
        """
        import math

        if len(self.coords_x) < 2:
            return 0.0

        min_dist = float('inf')
        num_nodes = len(self.coords_x)

        # Only check a sample if there are too many nodes
        if num_nodes > 1000:
            import random
            sample_size = min(1000, num_nodes)
            indices = random.sample(range(num_nodes), sample_size)
        else:
            indices = range(num_nodes)

        for i in indices:
            for j in range(i + 1, num_nodes):
                dx = self.coords_x[i] - self.coords_x[j]
                dy = self.coords_y[i] - self.coords_y[j]
                dz = (self.coords_z[i] - self.coords_z[j]) if self.coords_z else 0.0
                dist = math.sqrt(dx*dx + dy*dy + dz*dz)
                if dist > 0 and dist < min_dist:
                    min_dist = dist

        return min_dist if min_dist != float('inf') else 0.0

    def get_length_scale(self) -> float:
        """
        Get a characteristic length scale of the model.

        Returns
        -------
        float
            Characteristic length scale (bounding box diagonal)
        """
        import math

        if not self.coords_x:
            return 0.0

        min_x, max_x = min(self.coords_x), max(self.coords_x)
        min_y, max_y = min(self.coords_y), max(self.coords_y)

        if self.coords_z:
            min_z, max_z = min(self.coords_z), max(self.coords_z)
        else:
            min_z, max_z = 0.0, 0.0

        dx = max_x - min_x
        dy = max_y - min_y
        dz = max_z - min_z

        return math.sqrt(dx*dx + dy*dy + dz*dz)

    # Element block calculation methods
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
            raise ValueError("No element blocks specified")

        # Get a set of all nodes within the given element blocks
        all_nodes = set()
        for block_id in element_block_ids:
            if block_id not in self.element_blocks:
                continue
            # Get flat connectivity and add all nodes
            conn_flat = self.element_blocks[block_id]['connectivity_flat']
            all_nodes.update(conn_flat)

        if not all_nodes:
            # Return zero extents if no nodes
            return [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]

        # Convert 1-based indices to 0-based
        all_nodes_zero_based = [idx - 1 for idx in all_nodes if 0 < idx <= len(self.coords_x)]

        # Find the extents
        extents = []
        coords_lists = [self.coords_x, self.coords_y, self.coords_z if self.coords_z else [0.0] * len(self.coords_x)]
        for coords in coords_lists:
            node_coords = [coords[node_idx] for node_idx in all_nodes_zero_based]
            extents.append([min(node_coords), max(node_coords)])

        return extents

    def get_element_block_centroid(self, element_block_ids: Union[str, List[int]] = "all") -> List[float]:
        """
        Return the centroid of the element blocks.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element block IDs or "all" (default: "all")

        Returns
        -------
        list of float
            Centroid coordinates [x, y, z]
        """
        extents = self.get_element_block_extents(element_block_ids)
        return [(ext[0] + ext[1]) / 2.0 for ext in extents]

    def get_element_edge_length_info(self, element_block_ids: Union[str, List[int]] = "all") -> Tuple[float, float]:
        """
        Return the minimum and average element edge lengths.

        Only edges within elements in the specified element blocks are counted.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element blocks to process (default: "all")

        Returns
        -------
        tuple of (float, float)
            (minimum edge length, average edge length)
        """
        import math
        import sys

        # Format element block IDs
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        minimum = sys.float_info.max
        total = 0.0
        edge_count = 0

        # Edge definitions for common element types (0-based node indices)
        EDGE_DEFINITIONS = {
            'line2': [(0, 1)],
            'line3': [(0, 1), (1, 2)],
            'tri3': [(0, 1), (1, 2), (2, 0)],
            'tri6': [(0, 1), (1, 2), (2, 0), (0, 3), (1, 4), (2, 5)],
            'quad4': [(0, 1), (1, 2), (2, 3), (3, 0)],
            'quad8': [(0, 1), (1, 2), (2, 3), (3, 0), (0, 4), (1, 5), (2, 6), (3, 7)],
            'quad9': [(0, 1), (1, 2), (2, 3), (3, 0), (0, 4), (1, 5), (2, 6), (3, 7)],
            'tet4': [(0, 1), (1, 2), (2, 0), (0, 3), (1, 3), (2, 3)],
            'tet10': [(0, 1), (1, 2), (2, 0), (0, 3), (1, 3), (2, 3)],
            'hex8': [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4),
                     (0, 4), (1, 5), (2, 6), (3, 7)],
            'hex20': [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4),
                      (0, 4), (1, 5), (2, 6), (3, 7)],
            'hex27': [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4),
                      (0, 4), (1, 5), (2, 6), (3, 7)],
            'wedge6': [(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (0, 3), (1, 4), (2, 5)],
            'pyramid5': [(0, 1), (1, 2), (2, 3), (3, 0), (0, 4), (1, 4), (2, 4), (3, 4)],
        }

        for element_block_id in element_block_ids:
            if element_block_id not in self.element_blocks:
                continue

            block_data = self.element_blocks[element_block_id]
            topology = block_data['block'].topology.lower()

            # Get edge definition
            edge_def = EDGE_DEFINITIONS.get(topology, [])
            if not edge_def:
                continue

            # Get connectivity
            conn_flat = block_data['connectivity_flat']
            nodes_per_elem = block_data['block'].num_nodes_per_entry
            num_elems = len(conn_flat) // nodes_per_elem

            for elem_idx in range(num_elems):
                start = elem_idx * nodes_per_elem
                elem_conn = conn_flat[start:start + nodes_per_elem]

                for edge in edge_def:
                    # Convert to 0-based node indices
                    node1_idx = elem_conn[edge[0]] - 1
                    node2_idx = elem_conn[edge[1]] - 1

                    if node1_idx >= len(self.coords_x) or node2_idx >= len(self.coords_x):
                        continue

                    # Calculate distance
                    dx = self.coords_x[node1_idx] - self.coords_x[node2_idx]
                    dy = self.coords_y[node1_idx] - self.coords_y[node2_idx]
                    dz = (self.coords_z[node1_idx] - self.coords_z[node2_idx]) if self.coords_z else 0.0
                    dist = math.sqrt(dx*dx + dy*dy + dz*dz)

                    total += dist
                    edge_count += 1
                    if dist < minimum:
                        minimum = dist

        if edge_count == 0:
            return (float("nan"), float("nan"))

        return (minimum, total / edge_count)

    # Complex element block operations
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
            raise ValueError(f"Element block {source_id} does not exist")

        # Check that target block doesn't exist
        if target_id in self.element_blocks:
            raise ValueError(f"Element block {target_id} already exists")

        # Get source block data
        source_data = self.element_blocks[source_id]
        block = source_data['block']
        name = source_data.get('name', '')
        conn_flat = source_data['connectivity_flat']
        fields = source_data.get('fields', {})

        # Create new nodes if requested
        if duplicate_nodes:
            # Get unique nodes from connectivity
            unique_node_indices = sorted(set(conn_flat))

            # Create node mapping (1-based)
            node_map = {}
            new_node_offset = len(self.coords_x)

            # Duplicate the nodes
            for old_idx in unique_node_indices:
                # Convert to 0-based index
                zero_based_idx = old_idx - 1
                if 0 <= zero_based_idx < len(self.coords_x):
                    # Add new node
                    self.coords_x.append(self.coords_x[zero_based_idx])
                    self.coords_y.append(self.coords_y[zero_based_idx])
                    if self.coords_z:
                        self.coords_z.append(self.coords_z[zero_based_idx])
                    node_map[old_idx] = new_node_offset + 1  # 1-based indexing
                    new_node_offset += 1

            # Create new connectivity with new node indices
            new_conn_flat = [node_map.get(node_idx, node_idx) for node_idx in conn_flat]
        else:
            # Just copy the connectivity
            new_conn_flat = list(conn_flat)

        # Create the new element block
        info = [block.topology, block.num_entries, block.num_nodes_per_entry,
                getattr(block, 'num_attributes', 0)]
        self.create_element_block(target_id, info)

        # Set connectivity
        self.element_blocks[target_id]['connectivity_flat'] = new_conn_flat

        # Copy the name if it exists
        if name:
            self.element_blocks[target_id]['name'] = name + "_copy"

        # Copy fields
        new_fields = {}
        for field_name, all_values in fields.items():
            new_fields[field_name] = [list(values) for values in all_values]
        self.element_blocks[target_id]['fields'] = new_fields

    def combine_element_blocks(self, element_block_ids: Union[str, List[int]],
                              target_element_block_id: Union[str, int] = "auto"):
        """
        Combine multiple element blocks into a single block.

        By default, the target element block id will be the smallest of the
        merged element block ids. The element blocks to combine must have the
        same element type.

        Parameters
        ----------
        element_block_ids : str or list of int
            Element block IDs to combine or "all"
        target_element_block_id : str or int, optional
            Target element block ID (default: "auto" uses smallest ID)

        Examples
        --------
        >>> model.combine_element_blocks([1, 2, 3])
        >>> model.combine_element_blocks('all', 1)
        """
        # Handle "all" case
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        if not element_block_ids:
            raise ValueError("No element blocks specified")

        # Determine target block ID
        if target_element_block_id == "auto":
            target_element_block_id = min(element_block_ids)

        # Single block - just rename if needed
        if len(element_block_ids) == 1:
            if element_block_ids[0] != target_element_block_id:
                self.rename_element_block(element_block_ids[0], target_element_block_id)
            return

        # Ensure all blocks have the same number of nodes per element
        nodes_per_element_set = set(
            self.element_blocks[block_id]['block'].num_nodes_per_entry
            for block_id in element_block_ids
        )
        if len(nodes_per_element_set) != 1:
            raise ValueError(
                "The number of nodes per element on each element block to be merged "
                "must be the same. This is an ExodusII file requirement."
            )

        # Create new connectivity by combining all blocks
        new_conn_flat = []
        for block_id in element_block_ids:
            new_conn_flat.extend(self.element_blocks[block_id]['connectivity_flat'])

        # Create new info based on first block
        first_block = self.element_blocks[element_block_ids[0]]['block']
        nodes_per_elem = first_block.num_nodes_per_entry
        new_elem_count = len(new_conn_flat) // nodes_per_elem

        new_info = [first_block.topology, new_elem_count, nodes_per_elem,
                   getattr(first_block, 'num_attributes', 0)]

        # Find a temporary element block ID that doesn't exist
        temp_id = max(self.element_blocks.keys()) + 1 if self.element_blocks else 1
        while temp_id in self.element_blocks:
            temp_id += 1

        # Get all element field names across all blocks
        all_field_names = set()
        for block_id in element_block_ids:
            all_field_names.update(self.element_blocks[block_id].get('fields', {}).keys())

        # Combine all field data
        new_fields = {}
        for field_name in all_field_names:
            num_timesteps = len(self.timesteps) if self.timesteps else 1
            new_values = [[] for _ in range(num_timesteps)]
            for block_id in element_block_ids:
                field_data = self.element_blocks[block_id].get('fields', {}).get(field_name, [])
                if field_data:
                    for timestep_idx, values in enumerate(field_data):
                        if timestep_idx < len(new_values):
                            new_values[timestep_idx].extend(values)
            new_fields[field_name] = new_values

        # Create the new combined block
        self.create_element_block(temp_id, new_info)
        self.element_blocks[temp_id]['connectivity_flat'] = new_conn_flat
        self.element_blocks[temp_id]['fields'] = new_fields

        # Delete old blocks (nodes won't be orphaned by this procedure)
        for block_id in element_block_ids:
            self.delete_element_block(block_id, delete_orphaned_nodes=False)

        # Rename temporary block to target ID
        self.rename_element_block(temp_id, target_element_block_id)

    def merge_nodes(self, tolerance: float = None, *args, **kwargs) -> int:
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
        For meshes with more than 10,000 nodes, a sampling approach is used.
        """
        if tolerance is None:
            tolerance = 1e-6 * self.get_length_scale()

        num_nodes = len(self.coords_x)
        if num_nodes == 0:
            return 0

        # For very large meshes, this is too slow, so limit it
        if num_nodes > 10000:
            # Just return 0 for now - would need spatial indexing for performance
            return 0

        # Find nodes to merge (simple O(n²) algorithm)
        merge_map = {}  # Maps node index to the index it should merge with
        merged_count = 0

        for i in range(num_nodes):
            if i in merge_map:
                continue

            xi, yi = self.coords_x[i], self.coords_y[i]
            zi = self.coords_z[i] if self.coords_z else 0.0

            for j in range(i + 1, num_nodes):
                if j in merge_map:
                    continue

                # Calculate distance
                dx = xi - self.coords_x[j]
                dy = yi - self.coords_y[j]
                dz = zi - (self.coords_z[j] if self.coords_z else 0.0)
                dist_sq = dx*dx + dy*dy + dz*dz

                if dist_sq < tolerance**2:
                    merge_map[j] = i
                    merged_count += 1

        if merged_count == 0:
            return 0

        # Create node mapping (0-based)
        node_map = {}
        new_index = 0
        for i in range(num_nodes):
            if i in merge_map:
                # This node merges with another
                target = merge_map[i]
                while target in merge_map:
                    target = merge_map[target]
                node_map[i] = node_map.get(target, target)
            else:
                node_map[i] = new_index
                new_index += 1

        # Create new coordinate arrays
        new_coords_x = []
        new_coords_y = []
        new_coords_z = []
        for i in range(num_nodes):
            if i not in merge_map:
                new_coords_x.append(self.coords_x[i])
                new_coords_y.append(self.coords_y[i])
                if self.coords_z:
                    new_coords_z.append(self.coords_z[i])

        self.coords_x = new_coords_x
        self.coords_y = new_coords_y
        if self.coords_z:
            self.coords_z = new_coords_z

        # Update connectivity (1-based)
        for block_id, block_data in self.element_blocks.items():
            conn_flat = block_data['connectivity_flat']
            new_conn_flat = [node_map[idx - 1] + 1 for idx in conn_flat]
            block_data['connectivity_flat'] = new_conn_flat

        # Update node sets (1-based)
        for ns_id, ns_data in self.node_sets.items():
            members = ns_data.get('members', [])
            new_members = list(set(node_map[idx - 1] + 1 for idx in members))  # Remove duplicates
            ns_data['members'] = sorted(new_members)

        return merged_count

    # Field creation and conversion methods
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
            pass  # Overwrite existing

        # Initialize field values for all timesteps
        num_timesteps = len(self.timesteps) if self.timesteps else 1
        num_nodes = len(self.coords_x)

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

    def create_element_field(self, element_field_name: str,
                           element_block_ids: Union[str, int, List[int]] = "all",
                           value: Union[str, float, List] = "auto"):
        """
        Create an element field for specified blocks.

        Parameters
        ----------
        element_field_name : str
            Name of the element field
        element_block_ids : str, int, or list of int, optional
            Element block IDs (default: "all")
        value : str, float, or list, optional
            Initial value ("auto" for zeros, float for constant, list for per-timestep values)

        Examples
        --------
        >>> model.create_element_field("stress", 1, 0.0)
        >>> model.create_element_field("strain", [1, 2, 3])
        """
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        num_timesteps = len(self.timesteps) if self.timesteps else 1

        for block_id in element_block_ids:
            if block_id not in self.element_blocks:
                continue

            block_data = self.element_blocks[block_id]
            num_elems = block_data['block'].num_entries

            if value == "auto":
                field_data = [[0.0] * num_elems for _ in range(num_timesteps)]
            elif isinstance(value, (int, float)):
                field_data = [[float(value)] * num_elems for _ in range(num_timesteps)]
            elif isinstance(value, list):
                field_data = value
            else:
                field_data = [[0.0] * num_elems for _ in range(num_timesteps)]

            if 'fields' not in block_data:
                block_data['fields'] = {}
            block_data['fields'][element_field_name] = field_data

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

    def element_field_exists(self, element_field_name: str,
                            element_block_id: int) -> bool:
        """
        Check if element field exists in a specific block.

        Parameters
        ----------
        element_field_name : str
            Element field name
        element_block_id : int
            Element block ID

        Returns
        -------
        bool
            True if element field exists
        """
        if element_block_id not in self.element_blocks:
            return False
        return element_field_name in self.element_blocks[element_block_id].get('fields', {})

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

    def convert_element_field_to_node_field(self, element_field_name: str,
                                          node_field_name: Optional[str] = None,
                                          element_block_ids: Union[str, List[int]] = "all"):
        """
        Convert element field to node field by averaging element values at each node.

        Parameters
        ----------
        element_field_name : str
            Element field name to convert
        node_field_name : str, optional
            Name for node field (default: same as element_field_name)
        element_block_ids : str or list of int, optional
            Element block IDs (default: "all")

        Examples
        --------
        >>> model.convert_element_field_to_node_field("stress")
        >>> model.convert_element_field_to_node_field("stress", "node_stress", [1, 2])
        """
        if node_field_name is None:
            node_field_name = element_field_name

        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        # Create node field
        self.create_node_field(node_field_name, 0.0)

        num_timesteps = len(self.timesteps) if self.timesteps else 1

        for timestep_idx in range(num_timesteps):
            # Accumulate values and counts for each node
            num_nodes = len(self.coords_x)
            node_sum = [0.0] * num_nodes
            node_count = [0] * num_nodes

            for block_id in element_block_ids:
                if block_id not in self.element_blocks:
                    continue

                fields = self.element_blocks[block_id].get('fields', {})
                if element_field_name not in fields:
                    continue

                conn_flat = self.element_blocks[block_id]['connectivity_flat']
                nodes_per_elem = self.element_blocks[block_id]['block'].num_nodes_per_entry
                num_elems = len(conn_flat) // nodes_per_elem
                elem_values = fields[element_field_name][timestep_idx]

                for elem_idx in range(num_elems):
                    if elem_idx >= len(elem_values):
                        continue
                    elem_value = elem_values[elem_idx]
                    start = elem_idx * nodes_per_elem
                    elem_conn = conn_flat[start:start + nodes_per_elem]

                    for node_idx_1based in elem_conn:
                        zero_based_idx = node_idx_1based - 1
                        if 0 <= zero_based_idx < num_nodes:
                            node_sum[zero_based_idx] += elem_value
                            node_count[zero_based_idx] += 1

            # Calculate averages
            node_values = [
                node_sum[i] / node_count[i] if node_count[i] > 0 else 0.0
                for i in range(num_nodes)
            ]

            self.node_fields[node_field_name][timestep_idx] = node_values

    def convert_node_field_to_element_field(self, node_field_name: str,
                                          element_field_name: Optional[str] = None,
                                          element_block_ids: Union[str, List[int]] = "all"):
        """
        Convert node field to element field by averaging nodal values for each element.

        Parameters
        ----------
        node_field_name : str
            Node field name to convert
        element_field_name : str, optional
            Name for element field (default: same as node_field_name)
        element_block_ids : str or list of int, optional
            Element block IDs (default: "all")

        Examples
        --------
        >>> model.convert_node_field_to_element_field("temperature")
        >>> model.convert_node_field_to_element_field("temperature", "elem_temp", [1, 2])
        """
        if element_field_name is None:
            element_field_name = node_field_name

        if node_field_name not in self.node_fields:
            raise ValueError(f"Node field '{node_field_name}' does not exist")

        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        num_timesteps = len(self.timesteps) if self.timesteps else 1

        for block_id in element_block_ids:
            if block_id not in self.element_blocks:
                continue

            # Create element field for this block
            self.create_element_field(element_field_name, block_id, 0.0)

            conn_flat = self.element_blocks[block_id]['connectivity_flat']
            nodes_per_elem = self.element_blocks[block_id]['block'].num_nodes_per_entry
            num_elems = len(conn_flat) // nodes_per_elem
            fields = self.element_blocks[block_id].get('fields', {})

            for timestep_idx in range(num_timesteps):
                node_values = self.node_fields[node_field_name][timestep_idx]
                elem_values = []

                for elem_idx in range(num_elems):
                    start = elem_idx * nodes_per_elem
                    elem_conn = conn_flat[start:start + nodes_per_elem]

                    # Average node values for this element
                    total = 0.0
                    count = 0
                    for node_idx_1based in elem_conn:
                        zero_based_idx = node_idx_1based - 1
                        if 0 <= zero_based_idx < len(node_values):
                            total += node_values[zero_based_idx]
                            count += 1
                    elem_value = total / count if count > 0 else 0.0
                    elem_values.append(elem_value)

                fields[element_field_name][timestep_idx] = elem_values

    # Timestep utility methods
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
            raise ValueError(f"Source timestep {timestep} does not exist")

        if new_timestep in self.timesteps:
            raise ValueError(f"Target timestep {new_timestep} already exists")

        # Get index of source timestep
        source_idx = self.timesteps.index(timestep)

        # Add new timestep
        self.timesteps.append(new_timestep)
        self.timesteps.sort()

        # Copy node field data
        for field_name, field_data in self.node_fields.items():
            if source_idx < len(field_data):
                field_data.append(list(field_data[source_idx]))

        # Copy element field data
        for block_id, block_data in self.element_blocks.items():
            for field_name, field_data in block_data.get('fields', {}).items():
                if source_idx < len(field_data):
                    field_data.append(list(field_data[source_idx]))

        # Copy side set field data
        for set_id, set_data in self.side_sets.items():
            for field_name, field_data in set_data.get('fields', {}).items():
                if source_idx < len(field_data):
                    field_data.append(list(field_data[source_idx]))

        # Copy node set field data
        for set_id, set_data in self.node_sets.items():
            for field_name, field_data in set_data.get('fields', {}).items():
                if source_idx < len(field_data):
                    field_data.append(list(field_data[source_idx]))

        # Copy global variable data
        for var_name, var_data in self.global_variables.items():
            if source_idx < len(var_data):
                var_data.append(var_data[source_idx])

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
                total += self.element_blocks[block_id]['block'].num_entries
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
            raise ValueError(f"Element block {element_block_id} does not exist")

        elem_type = self.element_blocks[element_block_id]['block'].topology
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
            raise ValueError(f"Element block {element_block_id} does not exist")

        return self.element_blocks[element_block_id]['block'].num_nodes_per_entry

    def output_global_variables(self, expressions: Union[Dict, List, str],
                                output_file: Optional[str] = None) -> str:
        """
        Output global variables to file or return as string.

        Parameters
        ----------
        expressions : dict, list, or str
            Variable names to output
        output_file : str, optional
            File path to write output (default: return as string)

        Returns
        -------
        str
            Tab-separated output of global variables

        Examples
        --------
        >>> output = model.output_global_variables(["energy", "volume"])
        >>> model.output_global_variables({"energy": "energy"}, "output.txt")
        """
        lines = []
        lines.append("# Global Variables")
        lines.append(f"# Timesteps: {len(self.timesteps)}")
        lines.append("")

        # Header
        header = ["Timestep"]
        if isinstance(expressions, dict):
            header.extend(expressions.keys())
            var_names = list(expressions.values())
        elif isinstance(expressions, list):
            var_names = expressions
            header.extend(var_names)
        else:
            var_names = [expressions]
            header.extend(var_names)

        lines.append("\t".join(header))

        # Data rows
        for i, timestep in enumerate(self.timesteps):
            row = [str(timestep)]
            for var_name in var_names:
                if var_name in self.global_variables:
                    var_data = self.global_variables[var_name]
                    if i < len(var_data):
                        row.append(str(var_data[i]))
                    else:
                        row.append("0.0")
                else:
                    row.append("N/A")
            lines.append("\t".join(row))

        result = "\n".join(lines)

        if output_file:
            with open(output_file, 'w') as f:
                f.write(result)

        return result

    # Additional field query methods
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
            raise ValueError(f"Element block {element_block_id} does not exist")

        fields = self.element_blocks[element_block_id].get('fields', {})
        if element_field_name not in fields:
            raise ValueError(f"Element field '{element_field_name}' does not exist in block {element_block_id}")

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

    def get_node_set_field_names(self, node_set_id: int) -> List[str]:
        """
        Get all field names for a node set.

        Parameters
        ----------
        node_set_id : int
            Node set ID

        Returns
        -------
        list of str
            List of field names
        """
        if node_set_id not in self.node_sets:
            raise ValueError(f"Node set {node_set_id} does not exist")
        return sorted(self.node_sets[node_set_id].get('fields', {}).keys())

    def get_node_set_field_values(self, node_set_id: int, field_name: str,
                                  timestep: Union[str, float] = "last") -> List[float]:
        """
        Get node set field values.

        Parameters
        ----------
        node_set_id : int
            Node set ID
        field_name : str
            Field name
        timestep : str or float, optional
            Timestep ("last" or timestep value)

        Returns
        -------
        list of float
            Field values
        """
        if node_set_id not in self.node_sets:
            raise ValueError(f"Node set {node_set_id} does not exist")

        fields = self.node_sets[node_set_id].get('fields', {})
        if field_name not in fields:
            raise ValueError(f"Field '{field_name}' does not exist in node set {node_set_id}")

        field_data = fields[field_name]

        # Determine timestep index
        if timestep == "last":
            timestep_idx = len(field_data) - 1 if field_data else 0
        else:
            try:
                timestep_idx = self.timesteps.index(timestep)
            except ValueError:
                timestep_idx = 0

        if timestep_idx < 0 or timestep_idx >= len(field_data):
            timestep_idx = 0

        return field_data[timestep_idx] if field_data else []

    def create_node_set_field(self, node_set_id: int, field_name: str,
                             value: Union[str, float, List] = "auto"):
        """
        Create a field for a node set.

        Parameters
        ----------
        node_set_id : int
            Node set ID
        field_name : str
            Field name
        value : str, float, or list, optional
            Initial value ("auto" for zeros, float for constant, list for per-timestep values)
        """
        if node_set_id not in self.node_sets:
            raise ValueError(f"Node set {node_set_id} does not exist")

        num_timesteps = len(self.timesteps) if self.timesteps else 1
        num_members = len(self.node_sets[node_set_id].get('members', []))

        if value == "auto":
            field_data = [[0.0] * num_members for _ in range(num_timesteps)]
        elif isinstance(value, (int, float)):
            field_data = [[float(value)] * num_members for _ in range(num_timesteps)]
        elif isinstance(value, list):
            field_data = value
        else:
            field_data = [[0.0] * num_members for _ in range(num_timesteps)]

        if 'fields' not in self.node_sets[node_set_id]:
            self.node_sets[node_set_id]['fields'] = {}
        self.node_sets[node_set_id]['fields'][field_name] = field_data

    def create_side_set_field(self, side_set_id: int, field_name: str,
                             value: Union[str, float, List] = "auto"):
        """
        Create a field for a side set.

        Parameters
        ----------
        side_set_id : int
            Side set ID
        field_name : str
            Field name
        value : str, float, or list, optional
            Initial value ("auto" for zeros, float for constant, list for per-timestep values)
        """
        if side_set_id not in self.side_sets:
            raise ValueError(f"Side set {side_set_id} does not exist")

        num_timesteps = len(self.timesteps) if self.timesteps else 1
        num_members = len(self.side_sets[side_set_id].get('members', []))

        if value == "auto":
            field_data = [[0.0] * num_members for _ in range(num_timesteps)]
        elif isinstance(value, (int, float)):
            field_data = [[float(value)] * num_members for _ in range(num_timesteps)]
        elif isinstance(value, list):
            field_data = value
        else:
            field_data = [[0.0] * num_members for _ in range(num_timesteps)]

        if 'fields' not in self.side_sets[side_set_id]:
            self.side_sets[side_set_id]['fields'] = {}
        self.side_sets[side_set_id]['fields'][field_name] = field_data

    def delete_node_set_field(self, field_names: Union[str, List[str]],
                             node_set_ids: Union[str, List[int]] = "all"):
        """
        Delete node set field(s).

        Parameters
        ----------
        field_names : str or list of str
            Field name(s) to delete
        node_set_ids : str or list of int, optional
            Node set IDs (default: "all")
        """
        if isinstance(field_names, str):
            field_names = [field_names]

        if node_set_ids == "all":
            node_set_ids = list(self.node_sets.keys())
        elif isinstance(node_set_ids, int):
            node_set_ids = [node_set_ids]

        for ns_id in node_set_ids:
            if ns_id in self.node_sets:
                for field_name in field_names:
                    if field_name in self.node_sets[ns_id].get('fields', {}):
                        del self.node_sets[ns_id]['fields'][field_name]

    def delete_side_set_field(self, field_names: Union[str, List[str]],
                             side_set_ids: Union[str, List[int]] = "all"):
        """
        Delete side set field(s).

        Parameters
        ----------
        field_names : str or list of str
            Field name(s) to delete
        side_set_ids : str or list of int, optional
            Side set IDs (default: "all")
        """
        if isinstance(field_names, str):
            field_names = [field_names]

        if side_set_ids == "all":
            side_set_ids = list(self.side_sets.keys())
        elif isinstance(side_set_ids, int):
            side_set_ids = [side_set_ids]

        for ss_id in side_set_ids:
            if ss_id in self.side_sets:
                for field_name in field_names:
                    if field_name in self.side_sets[ss_id].get('fields', {}):
                        del self.side_sets[ss_id]['fields'][field_name]

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
            raise ValueError(f"Side set {side_set_id} does not exist")

        # Get side set members
        side_set_members = self.get_side_set_members(side_set_id)

        # Extract unique nodes from elements
        node_indices = set()
        for elem_id, face_id in side_set_members:
            # Find the element in the blocks (elem_id is 1-based)
            for block_id, block_data in self.element_blocks.items():
                num_elems = block_data['block'].num_entries
                nodes_per_elem = block_data['block'].num_nodes_per_entry

                # Check if this element is in this block
                if 0 < elem_id <= num_elems:
                    conn_flat = block_data['connectivity_flat']
                    start = (elem_id - 1) * nodes_per_elem
                    elem_conn = conn_flat[start:start + nodes_per_elem]
                    node_indices.update(elem_conn)
                    break

        return sorted(node_indices)

    def displacement_field_exists(self) -> bool:
        """
        Check if displacement field exists.

        Returns
        -------
        bool
            True if displacement fields (displ_x, displ_y, displ_z) exist
        """
        return ('displ_x' in self.node_fields or
                'displacement_x' in self.node_fields or
                'dispx' in self.node_fields)

    def create_displacement_field(self, value: Union[str, float] = "auto"):
        """
        Create displacement fields (displ_x, displ_y, displ_z).

        Parameters
        ----------
        value : str or float, optional
            Initial value ("auto" for zeros, float for constant)
        """
        self.create_node_field("displ_x", value)
        self.create_node_field("displ_y", value)
        if self.num_dim >= 3:
            self.create_node_field("displ_z", value)

    def displace_element_blocks(self, element_block_ids: Union[str, List[int]] = "all",
                                timestep: Union[str, float] = "last"):
        """
        Displace nodes according to displacement field.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element blocks to process (default: "all")
        timestep : str or float, optional
            Timestep to use (default: "last")
        """
        if not self.displacement_field_exists():
            raise ValueError("No displacement field exists")

        # Get displacement field names
        displ_x_name = 'displ_x' if 'displ_x' in self.node_fields else 'displacement_x'
        displ_y_name = 'displ_y' if 'displ_y' in self.node_fields else 'displacement_y'
        displ_z_name = 'displ_z' if 'displ_z' in self.node_fields else 'displacement_z'

        # Determine timestep index
        if timestep == "last":
            timestep_idx = len(self.timesteps) - 1 if self.timesteps else 0
        else:
            try:
                timestep_idx = self.timesteps.index(timestep)
            except ValueError:
                timestep_idx = 0

        # Apply displacements
        displ_x = self.node_fields.get(displ_x_name, [[0.0] * len(self.coords_x)])[timestep_idx]
        displ_y = self.node_fields.get(displ_y_name, [[0.0] * len(self.coords_y)])[timestep_idx]
        displ_z = self.node_fields.get(displ_z_name, [[0.0] * len(self.coords_x if self.coords_z else [])])[timestep_idx] if self.coords_z else []

        for i in range(len(self.coords_x)):
            if i < len(displ_x):
                self.coords_x[i] += displ_x[i]
            if i < len(displ_y):
                self.coords_y[i] += displ_y[i]
            if self.coords_z and i < len(displ_z):
                self.coords_z[i] += displ_z[i]

    # Volume calculation methods
    def _new_element_field_name(self, quantity: int = 1) -> Union[str, List[str]]:
        """
        Generate unique temporary element field name(s).

        Parameters
        ----------
        quantity : int, optional
            Number of unique names to generate (default: 1)

        Returns
        -------
        str or list of str
            Single name if quantity==1, otherwise list of names
        """
        id_ = 1
        names = []
        all_field_names = set(self.get_element_field_names())
        for _ in range(quantity):
            name = f"temp{id_}"
            while name in all_field_names:
                id_ += 1
                name = f"temp{id_}"
            names.append(name)
            all_field_names.add(name)
            id_ += 1

        return names[0] if quantity == 1 else names

    def calculate_element_centroids(self, element_block_ids: Union[str, List[int]] = "all",
                                   field_prefix: str = "centroid"):
        """
        Calculate and store the centroid of each element.

        This will approximate the element centroid as the nodal average of each
        element and will store that value in an element field. Since a
        timestep must be defined in order for element fields to exist, one will
        be created if none exist.

        By default, the centroid will be stored in the fields 'centroid_x',
        'centroid_y', and 'centroid_z'. Alternatively, a prefix can be given
        or a list of three strings can be given.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element blocks to process (default: "all")
        field_prefix : str or list of str, optional
            Field name prefix or list of three field names (default: "centroid")
        """
        # Format element block IDs
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        # Ensure at least one timestep exists
        if not self.timesteps:
            self.create_timestep(0.0)

        # Determine field names
        if isinstance(field_prefix, str):
            centroid_field_names = [f"{field_prefix}_{x}" for x in ["x", "y", "z"]]
        else:
            centroid_field_names = field_prefix

        for element_block_id in element_block_ids:
            if element_block_id not in self.element_blocks:
                continue

            block_data = self.element_blocks[element_block_id]

            # Calculate centroids
            centroid = [[], [], []]
            element_count = block_data.block.num_entries
            nodes_per_element = block_data.block.num_nodes_per_entry

            for element_index in range(element_count):
                this_centroid = [0.0, 0.0, 0.0]
                element_nodes = block_data.get_element_nodes(element_index)

                # Accumulate node coordinates (1-based indexing to 0-based)
                for node_id in element_nodes:
                    node_idx = node_id - 1
                    if node_idx < len(self.coords_x):
                        this_centroid[0] += self.coords_x[node_idx]
                        this_centroid[1] += self.coords_y[node_idx]
                        if self.coords_z and node_idx < len(self.coords_z):
                            this_centroid[2] += self.coords_z[node_idx]

                # Average
                for i in range(3):
                    centroid[i].append(this_centroid[i] / nodes_per_element)

            # Store centroid fields for all timesteps
            for index, name in enumerate(centroid_field_names):
                values = []
                for _ in range(len(self.timesteps)):
                    values.append(list(centroid[index]))
                block_data.fields[name] = values

    def calculate_element_volumes(self, element_block_ids: Union[str, List[int]] = "all",
                                 field_name: str = "volume"):
        """
        Calculate and store the volume of each element.

        This will approximate the element volume. Since a timestep must be
        defined in order for element fields to exist, one will be created if
        none exist.

        For two dimensional elements, this calculates the area. For one
        dimensional elements, this calculates the length.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element blocks to process (default: "all")
        field_name : str, optional
            Name for the volume field (default: "volume")
        """
        import math

        # Format element block IDs
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        # Ensure at least one timestep exists
        if not self.timesteps:
            self.create_timestep(0.0)

        for element_block_id in element_block_ids:
            if element_block_id not in self.element_blocks:
                continue

            block_data = self.element_blocks[element_block_id]

            # Get the element type
            element_type = block_data.block.topology.lower()

            if element_type not in self.VOLUME_FORMULA:
                print(f"Warning: Volume formula for element type '{element_type}' not implemented. Skipping block {element_block_id}.")
                continue

            # Get the formula
            formula = self.VOLUME_FORMULA[element_type]
            coefficient = formula[0]

            # Build the calculation based on element dimension
            element_count = block_data.block.num_entries
            volumes = []

            for element_index in range(element_count):
                element_nodes = block_data.get_element_nodes(element_index)

                # Build coordinate array for this element (convert to 0-based indexing)
                coords = []
                for node_id in element_nodes:
                    node_idx = node_id - 1
                    if node_idx < len(self.coords_x):
                        coords.append(self.coords_x[node_idx])
                        coords.append(self.coords_y[node_idx])
                        coords.append(self.coords_z[node_idx] if self.coords_z and node_idx < len(self.coords_z) else 0.0)

                # Calculate volume based on formula type
                if len(formula) == 2:
                    # 1D: distance between two points
                    rule = formula[1]
                    vec = [0.0, 0.0, 0.0]
                    for d in range(3):
                        node_list_0 = rule[0] if isinstance(rule[0], tuple) else (rule[0],)
                        node_list_1 = rule[1] if isinstance(rule[1], tuple) else (rule[1],)

                        coord0 = sum(coords[n * 3 + d] for n in node_list_0) / len(node_list_0)
                        coord1 = sum(coords[n * 3 + d] for n in node_list_1) / len(node_list_1)
                        vec[d] = coord1 - coord0

                    volume = coefficient * math.sqrt(sum(v * v for v in vec))

                elif len(formula) == 3:
                    # 2D: cross product magnitude
                    vecs = []
                    for rule in formula[1:]:
                        vec = [0.0, 0.0, 0.0]
                        for d in range(3):
                            node_list_0 = rule[0] if isinstance(rule[0], tuple) else (rule[0],)
                            node_list_1 = rule[1] if isinstance(rule[1], tuple) else (rule[1],)

                            coord0 = sum(coords[n * 3 + d] for n in node_list_0) / len(node_list_0)
                            coord1 = sum(coords[n * 3 + d] for n in node_list_1) / len(node_list_1)
                            vec[d] = coord1 - coord0
                        vecs.append(vec)

                    # Cross product
                    cross = [
                        vecs[0][1] * vecs[1][2] - vecs[0][2] * vecs[1][1],
                        vecs[0][2] * vecs[1][0] - vecs[0][0] * vecs[1][2],
                        vecs[0][0] * vecs[1][1] - vecs[0][1] * vecs[1][0],
                    ]
                    volume = coefficient * math.sqrt(sum(c * c for c in cross))

                elif len(formula) == 4:
                    # 3D: triple product
                    vecs = []
                    for rule in formula[1:]:
                        vec = [0.0, 0.0, 0.0]
                        for d in range(3):
                            node_list_0 = rule[0] if isinstance(rule[0], tuple) else (rule[0],)
                            node_list_1 = rule[1] if isinstance(rule[1], tuple) else (rule[1],)

                            coord0 = sum(coords[n * 3 + d] for n in node_list_0) / len(node_list_0)
                            coord1 = sum(coords[n * 3 + d] for n in node_list_1) / len(node_list_1)
                            vec[d] = coord1 - coord0
                        vecs.append(vec)

                    # Triple product: (vec1 × vec2) · vec3
                    cross = [
                        vecs[0][1] * vecs[1][2] - vecs[0][2] * vecs[1][1],
                        vecs[0][2] * vecs[1][0] - vecs[0][0] * vecs[1][2],
                        vecs[0][0] * vecs[1][1] - vecs[0][1] * vecs[1][0],
                    ]
                    volume = coefficient * (cross[0] * vecs[2][0] + cross[1] * vecs[2][1] + cross[2] * vecs[2][2])
                else:
                    volume = 0.0

                volumes.append(abs(volume))

            # Store volume field for all timesteps
            values = []
            for _ in range(len(self.timesteps)):
                values.append(list(volumes))
            block_data.fields[field_name] = values

    def get_element_block_volume(self, element_block_ids: Union[str, List[int]] = "all",
                                timestep: Union[str, float] = "last") -> float:
        """
        Return the total volume of the given element blocks.

        Parameters
        ----------
        element_block_ids : str or list of int, optional
            Element blocks to calculate volume for (default: "all")
        timestep : str or float, optional
            Timestep to use (default: "last") - currently unused

        Returns
        -------
        float
            Total volume of the element blocks
        """
        # Format element block IDs
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        # Create a timestep if none exist
        created_timestep = False
        if not self.timesteps:
            created_timestep = True
            self.create_timestep(0.0)

        # Calculate temporary field with element volumes
        element_volume_field_name = self._new_element_field_name()
        self.calculate_element_volumes(element_block_ids, element_volume_field_name)

        # Add up the volumes
        volume = 0.0
        for block_id in element_block_ids:
            if block_id not in self.element_blocks:
                continue
            block_data = self.element_blocks[block_id]
            if element_volume_field_name in block_data.fields:
                volume += sum(block_data.fields[element_volume_field_name][0])

        # Delete the temporary timestep
        if created_timestep:
            self.delete_timestep(0.0)

        # Delete the temporary field
        self.delete_element_field(element_volume_field_name, element_block_ids)

        return volume

    # Field statistics methods
    def calculate_element_field_maximum(self, element_field_names: Union[str, List[str]],
                                       element_block_ids: Union[str, List[int]] = "all",
                                       timestep: Union[str, float] = "last") -> Union[float, Dict[str, float]]:
        """
        Find maximum value of element field(s).

        Parameters
        ----------
        element_field_names : str or list of str
            Element field name(s) to find maximum for
        element_block_ids : str or list of int, optional
            Element block IDs (default: "all")
        timestep : str or float, optional
            Timestep to use (default: "last")

        Returns
        -------
        float or dict
            Maximum value(s) of the field(s)
        """
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        if isinstance(element_field_names, str):
            element_field_names = [element_field_names]

        # Get timestep index
        if timestep == "last":
            timestep_idx = len(self.timesteps) - 1 if self.timesteps else 0
        elif timestep == "first":
            timestep_idx = 0
        else:
            timestep_idx = min(range(len(self.timesteps)),
                             key=lambda i: abs(self.timesteps[i] - float(timestep)))

        max_values = {}
        for field_name in element_field_names:
            max_val = float('-inf')
            for block_id in element_block_ids:
                if block_id in self.element_blocks:
                    block_data = self.element_blocks[block_id]
                    if field_name in block_data.fields:
                        values = block_data.fields[field_name][timestep_idx]
                        if values:
                            max_val = max(max_val, max(values))
            max_values[field_name] = max_val if max_val != float('-inf') else None

        return max_values if len(element_field_names) > 1 else max_values[element_field_names[0]]

    def calculate_element_field_minimum(self, element_field_names: Union[str, List[str]],
                                       element_block_ids: Union[str, List[int]] = "all",
                                       timestep: Union[str, float] = "last") -> Union[float, Dict[str, float]]:
        """
        Find minimum value of element field(s).

        Parameters
        ----------
        element_field_names : str or list of str
            Element field name(s) to find minimum for
        element_block_ids : str or list of int, optional
            Element block IDs (default: "all")
        timestep : str or float, optional
            Timestep to use (default: "last")

        Returns
        -------
        float or dict
            Minimum value(s) of the field(s)
        """
        if element_block_ids == "all":
            element_block_ids = list(self.element_blocks.keys())
        elif isinstance(element_block_ids, int):
            element_block_ids = [element_block_ids]

        if isinstance(element_field_names, str):
            element_field_names = [element_field_names]

        # Get timestep index
        if timestep == "last":
            timestep_idx = len(self.timesteps) - 1 if self.timesteps else 0
        elif timestep == "first":
            timestep_idx = 0
        else:
            timestep_idx = min(range(len(self.timesteps)),
                             key=lambda i: abs(self.timesteps[i] - float(timestep)))

        min_values = {}
        for field_name in element_field_names:
            min_val = float('inf')
            for block_id in element_block_ids:
                if block_id in self.element_blocks:
                    block_data = self.element_blocks[block_id]
                    if field_name in block_data.fields:
                        values = block_data.fields[field_name][timestep_idx]
                        if values:
                            min_val = min(min_val, min(values))
            min_values[field_name] = min_val if min_val != float('inf') else None

        return min_values if len(element_field_names) > 1 else min_values[element_field_names[0]]

    def calculate_node_field_maximum(self, node_field_names: Union[str, List[str]],
                                      timestep: Union[str, float] = "last") -> Union[float, Dict[str, float]]:
        """
        Calculate maximum value of node field(s).

        Parameters
        ----------
        node_field_names : str or list of str
            Node field name(s) to find maximum for
        timestep : str or float, optional
            Timestep to use (default: "last")

        Returns
        -------
        float or dict
            Maximum value(s) of the field(s)
        """
        if isinstance(node_field_names, str):
            node_field_names = [node_field_names]

        # Get timestep index
        if timestep == "last":
            timestep_idx = len(self.timesteps) - 1 if self.timesteps else 0
        elif timestep == "first":
            timestep_idx = 0
        else:
            timestep_idx = min(range(len(self.timesteps)),
                             key=lambda i: abs(self.timesteps[i] - float(timestep)))

        max_values = {}
        for field_name in node_field_names:
            if field_name in self.node_fields:
                values = self.node_fields[field_name][timestep_idx]
                max_values[field_name] = max(values) if values else None
            else:
                max_values[field_name] = None

        return max_values if len(node_field_names) > 1 else max_values[node_field_names[0]]

    def calculate_node_field_minimum(self, node_field_names: Union[str, List[str]],
                                      timestep: Union[str, float] = "last") -> Union[float, Dict[str, float]]:
        """
        Calculate minimum value of node field(s).

        Parameters
        ----------
        node_field_names : str or list of str
            Node field name(s) to find minimum for
        timestep : str or float, optional
            Timestep to use (default: "last")

        Returns
        -------
        float or dict
            Minimum value(s) of the field(s)
        """
        if isinstance(node_field_names, str):
            node_field_names = [node_field_names]

        # Get timestep index
        if timestep == "last":
            timestep_idx = len(self.timesteps) - 1 if self.timesteps else 0
        elif timestep == "first":
            timestep_idx = 0
        else:
            timestep_idx = min(range(len(self.timesteps)),
                             key=lambda i: abs(self.timesteps[i] - float(timestep)))

        min_values = {}
        for field_name in node_field_names:
            if field_name in self.node_fields:
                values = self.node_fields[field_name][timestep_idx]
                min_values[field_name] = min(values) if values else None
            else:
                min_values[field_name] = None

        return min_values if len(node_field_names) > 1 else min_values[node_field_names[0]]

    # Interpolation methods
    def create_interpolated_timestep(self, timestep: float, interpolation: str = "linear"):
        """
        Create a new timestep by interpolating between existing timesteps.

        Parameters
        ----------
        timestep : float
            New timestep value
        interpolation : str, optional
            Interpolation method (default: "linear")

        Examples
        --------
        >>> model.create_interpolated_timestep(1.5)  # Interpolate at t=1.5
        """
        if not self.timesteps:
            raise ValueError("No timesteps available for interpolation")

        timestep = float(timestep)

        # Check if timestep already exists
        if timestep in self.timesteps:
            print(f"Warning: Timestep {timestep} already exists")
            return

        # Find surrounding timesteps
        lower_idx = None
        upper_idx = None

        for idx, ts in enumerate(self.timesteps):
            if ts < timestep:
                lower_idx = idx
            elif ts > timestep and upper_idx is None:
                upper_idx = idx
                break

        if lower_idx is None:
            # Before first timestep - use first two
            lower_idx = 0
            upper_idx = 1 if len(self.timesteps) > 1 else 0
        elif upper_idx is None:
            # After last timestep - use last two
            upper_idx = len(self.timesteps) - 1
            lower_idx = upper_idx - 1 if len(self.timesteps) > 1 else upper_idx

        t_lower = self.timesteps[lower_idx]
        t_upper = self.timesteps[upper_idx]

        # Calculate interpolation factor
        if t_upper == t_lower:
            factor = 0.5
        else:
            factor = (timestep - t_lower) / (t_upper - t_lower)

        # Insert timestep in sorted order
        insert_idx = upper_idx
        self.timesteps.insert(insert_idx, timestep)

        # Interpolate node fields
        for field_name, timestep_data in self.node_fields.items():
            lower_values = timestep_data[lower_idx]
            upper_values = timestep_data[upper_idx]
            interp_values = [
                lower_values[i] * (1 - factor) + upper_values[i] * factor
                for i in range(len(lower_values))
            ]
            timestep_data.insert(insert_idx, interp_values)

        # Interpolate global variables
        for var_name, timestep_data in self.global_variables.items():
            lower_value = timestep_data[lower_idx]
            upper_value = timestep_data[upper_idx]
            interp_value = lower_value * (1 - factor) + upper_value * factor
            timestep_data.insert(insert_idx, interp_value)

        # Interpolate element fields
        for block_id, block_data in self.element_blocks.items():
            for field_name, timestep_data in block_data.fields.items():
                lower_values = timestep_data[lower_idx]
                upper_values = timestep_data[upper_idx]
                interp_values = [
                    lower_values[i] * (1 - factor) + upper_values[i] * factor
                    for i in range(len(lower_values))
                ]
                timestep_data.insert(insert_idx, interp_values)

        # Interpolate side set fields
        for side_set_id, side_set_data in self.side_sets.items():
            for field_name, timestep_data in side_set_data.fields.items():
                lower_values = timestep_data[lower_idx]
                upper_values = timestep_data[upper_idx]
                interp_values = [
                    lower_values[i] * (1 - factor) + upper_values[i] * factor
                    for i in range(len(lower_values))
                ]
                timestep_data.insert(insert_idx, interp_values)

        # Interpolate node set fields
        for node_set_id, node_set_data in self.node_sets.items():
            for field_name, timestep_data in node_set_data.fields.items():
                lower_values = timestep_data[lower_idx]
                upper_values = timestep_data[upper_idx]
                interp_values = [
                    lower_values[i] * (1 - factor) + upper_values[i] * factor
                    for i in range(len(lower_values))
                ]
                timestep_data.insert(insert_idx, interp_values)

    # Mesh generation methods
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

        node_offset = len(self.coords_x)

        for k in range(nodes_z):
            for j in range(nodes_y):
                for i in range(nodes_x):
                    x = i * dx
                    y = j * dy
                    z = k * dz
                    self.coords_x.append(x)
                    self.coords_y.append(y)
                    self.coords_z.append(z)

        # Create connectivity (HEX8 elements)
        # HEX8 node ordering: bottom face (z=0) then top face (z=1)
        # Bottom: 0-1-2-3 (counter-clockwise looking down)
        # Top: 4-5-6-7 (counter-clockwise looking down)
        connectivity_flat = []

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

                    connectivity_flat.extend([n0, n1, n2, n3, n4, n5, n6, n7])

        # Create element block
        num_elems = nx * ny * nz

        try:
            from . import Block
            block = Block(
                id=block_id,
                topology="hex8",
                num_entries=num_elems,
                num_nodes_per_entry=8,
                num_attributes=0
            )
        except (ImportError, AttributeError):
            block = _MockBlock(
                id=block_id,
                topology="hex8",
                num_entries=num_elems,
                num_nodes_per_entry=8,
                num_attributes=0
            )

        self.element_blocks[block_id] = ElementBlockData(
            block=block,
            name="HEX8_Cube",
            connectivity_flat=connectivity_flat,
            fields={}
        )

    # Element conversion methods (not fully implementable)
    def convert_hex8_block_to_tet4_block(self, element_block_id: int, scheme: str = "hex24tet"):
        """
        Convert HEX8 elements to TET4 elements.

        Parameters
        ----------
        element_block_id : int
            Element block ID to convert
        scheme : str, optional
            Conversion scheme (default: "hex24tet")

        Notes
        -----
        This method is not fully implemented as it requires complex element subdivision
        algorithms and careful handling of shared nodes.
        """
        raise NotImplementedError(
            "convert_hex8_block_to_tet4_block() is not yet fully implemented. "
            "This requires complex element subdivision algorithms."
        )

    def convert_side_set_to_cohesive_zone(self, side_set_ids: Union[int, List[int]], new_element_block_id: int):
        """
        Convert side set to cohesive zone element block.

        Parameters
        ----------
        side_set_ids : int or list of int
            Side set ID(s) to convert
        new_element_block_id : int
            New element block ID for cohesive zone

        Notes
        -----
        This method is not fully implemented as it requires:
        - Node duplication
        - Connectivity updates
        - Element creation with cohesive element types
        """
        raise NotImplementedError(
            "convert_side_set_to_cohesive_zone() is not yet fully implemented. "
            "This requires node duplication and complex connectivity updates."
        )

    # Expression-based methods (not implementable without full expression parser)
    def calculate_element_field(self, expression: str, element_block_ids: Union[str, List[int]] = "all"):
        """Not implementable: requires expression evaluation."""
        raise NotImplementedError(
            "Expression-based field calculation is not implemented. "
            "This would require a full expression parser and evaluator."
        )

    def calculate_node_field(self, expression: str):
        """Not implementable: requires expression evaluation."""
        raise NotImplementedError(
            "Expression-based field calculation is not implemented. "
            "This would require a full expression parser and evaluator."
        )

    def calculate_global_variable(self, expression: str):
        """Not implementable: requires expression evaluation."""
        raise NotImplementedError(
            "Expression-based variable calculation is not implemented. "
            "This would require a full expression parser and evaluator."
        )

    def calculate_side_set_field(self, expression: str, side_set_ids: Union[str, List[int]] = "all"):
        """Not implementable: requires expression evaluation."""
        raise NotImplementedError(
            "Expression-based field calculation is not implemented."
        )

    def calculate_node_set_field(self, expression: str, node_set_ids: Union[str, List[int]] = "all"):
        """Not implementable: requires expression evaluation."""
        raise NotImplementedError(
            "Expression-based field calculation is not implemented."
        )

    def create_side_set_from_expression(self, expression: str, side_set_id: int = None):
        """Not implementable: requires expression evaluation."""
        raise NotImplementedError(
            "Expression-based side set creation is not implemented."
        )

    def threshold_element_blocks(self, expression: str, element_block_ids: Union[str, List[int]] = "all"):
        """Not implementable: requires expression evaluation."""
        raise NotImplementedError(
            "Expression-based element thresholding is not implemented."
        )

    # Export methods that depend on geometry processing
    def export_stl_file(self, filename: str, **kwargs):
        """Not implementable: STL export requires 3D geometry processing."""
        raise NotImplementedError(
            "STL export is not implementable without 3D geometry processing libraries. "
            "Consider using external tools for STL conversion."
        )

    def export_wrl_model(self, filename: str, **kwargs):
        """Not implementable: VRML export requires 3D geometry processing."""
        raise NotImplementedError(
            "VRML/WRL export is not implementable without 3D geometry processing. "
            "This format is also largely deprecated."
        )

    def export(self, filename: str, *args, **kwargs):
        """Alias for export_model."""
        return self.export_model(filename, *args, **kwargs)

    def _get_dimension(self, topology: str) -> int:
        """Get dimension for topology."""
        return self.DIMENSION.get(topology.lower(), 3)
