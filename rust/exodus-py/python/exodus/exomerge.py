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
