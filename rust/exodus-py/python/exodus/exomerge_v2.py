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
from . import exodus

__version__ = "0.2.0"
VERSION = __version__

# Contact person for issues
CONTACT = "exodus-rs development team"

# Show banner on first use
SHOW_BANNER = True

# If true, will crash if warnings are generated
EXIT_ON_WARNING = False


@dataclass
class ElementBlockData:
    """
    Element block data structure.

    Stores exodus Block object directly for zero-copy access.

    Attributes
    ----------
    block : exodus.Block
        Exodus block definition
    name : str
        Block name
    connectivity_flat : List[int]
        Flat connectivity array (exodus format)
    fields : Dict[str, List[Any]]
        Element field data per timestep
    """
    block: 'exodus.Block'
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
    node_set : exodus.NodeSet
        Exodus node set object
    name : str
        Set name
    fields : Dict[str, List[Any]]
        Node set field data per timestep
    """
    node_set: 'exodus.NodeSet'
    name: str = ""
    fields: Dict[str, List[Any]] = field(default_factory=dict)


@dataclass
class SideSetData:
    """
    Side set data structure.

    Attributes
    ----------
    side_set : exodus.SideSet
        Exodus side set object
    name : str
        Set name
    fields : Dict[str, List[Any]]
        Side set field data per timestep
    """
    side_set: 'exodus.SideSet'
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
        self._reader: Optional['exodus.ExodusReader'] = None
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
        params = self._reader.get_init_params()
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
        return [[self.coords_x[i], self.coords_y[i], self.coords_z[i]]
                for i in range(self.num_nodes)]

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
        from . import Block, EntityType

        topology, num_elems, nodes_per_elem, num_attrs = info

        block = Block(
            id=block_id,
            entity_type=EntityType.ElemBlock,
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

    def _get_dimension(self, topology: str) -> int:
        """Get dimension for topology."""
        return self.DIMENSION.get(topology.lower(), 3)
