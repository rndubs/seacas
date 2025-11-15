"""
Type stubs for exodus.exomerge module.

Provides high-level Python interface for manipulating ExodusII files.
"""

from typing import List, Dict, Any, Union, Optional, Tuple
from dataclasses import dataclass

from exodus import (
    ExodusReader,
    ExodusWriter,
    CreateOptions,
    InitParams,
    CreateMode,
    Block,
    NodeSet,
    SideSet,
    QaRecord,
    EntityType,
)

__version__: str
VERSION: str
CONTACT: str
SHOW_BANNER: bool
EXIT_ON_WARNING: bool
DEPRECATED_FUNCTIONS: Dict[str, str]

class SafeExpressionEvaluator:
    """Safe mathematical expression evaluator for field calculations"""

    def __init__(self) -> None: ...
    def evaluate(self, expression: str, context: Dict[str, Any]) -> Any: ...

@dataclass
class ElementBlockData:
    """
    Storage for element block connectivity and metadata.

    Stores element connectivity as a flat array with direct exodus objects.
    """

    block_id: int
    element_topology: str
    connectivity: List[int]
    num_elements: int
    num_nodes_per_element: int

    @property
    def num_entries(self) -> int: ...

    @property
    def nodes_per_entry(self) -> int: ...

    @property
    def topology(self) -> str: ...

    def __getitem__(self, index: int) -> List[int]: ...
    def __setitem__(self, index: int, value: List[int]) -> None: ...
    def get_element_connectivity(self, elem_index: int) -> List[int]: ...

@dataclass
class NodeSetData:
    """Storage for node set data"""

    node_set_id: int
    _members: List[int]
    name: str

    @property
    def members(self) -> List[int]: ...

    def __getitem__(self, index: int) -> int: ...

@dataclass
class SideSetData:
    """Storage for side set data"""

    side_set_id: int
    _members: List[Tuple[int, int]]
    name: str

    @property
    def members(self) -> List[Tuple[int, int]]: ...

    def __getitem__(self, index: int) -> Tuple[int, int]: ...

class ElementBlocksDict(dict[int, ElementBlockData]):
    """Dictionary wrapper for element blocks with validation"""

    def __setitem__(self, key: int, value: ElementBlockData) -> None: ...

def import_model(filename: str, mode: str = "inmemory", **kwargs: Any) -> ExodusModel:
    """
    Import an ExodusII file and return an ExodusModel.

    Args:
        filename: Path to the Exodus file
        mode: Loading mode - "inmemory" (default) or "streaming"
        **kwargs: Additional options passed to ExodusModel

    Returns:
        Loaded ExodusModel instance

    Example:
        >>> model = import_model('mesh.exo')
        >>> model = import_model('mesh.exo', mode='streaming')
    """
    ...

class ExodusModel:
    """
    High-performance interface for manipulating ExodusII finite element models.

    This class provides a Python-friendly API for creating, modifying, and
    exporting Exodus mesh files using exodus-rs Rust bindings for performance.

    Attributes:
        mode: Loading mode - "inmemory" or "streaming"
        _title: Mesh title
        _num_dim: Number of spatial dimensions (1, 2, or 3)
        _nodes: Node coordinates [node_id][dim]
        _element_blocks: Dictionary of element blocks by ID
        _node_sets: Dictionary of node sets by ID
        _side_sets: Dictionary of side sets by ID
        _qa_records: List of QA records
        _info_records: List of info strings
        _timesteps: List of time step values
        _node_fields: Nodal variable data
        _element_fields: Element variable data
        _global_variables: Global variable data
    """

    mode: str
    _title: str
    _num_dim: int
    _nodes: List[List[float]]
    _element_blocks: ElementBlocksDict
    _node_sets: Dict[int, NodeSetData]
    _side_sets: Dict[int, SideSetData]
    _qa_records: List[Tuple[str, str, str, str]]
    _info_records: List[str]
    _timesteps: List[float]
    _node_fields: Dict[str, Dict[int, List[float]]]
    _element_fields: Dict[int, Dict[str, Dict[int, List[float]]]]
    _global_variables: Dict[str, List[float]]

    def __init__(self, mode: str = "inmemory") -> None:
        """
        Initialize an empty ExodusModel.

        Args:
            mode: Loading mode - "inmemory" (default) or "streaming"
                 "inmemory": Load all data into memory (faster for small models)
                 "streaming": Stream data from disk (lower memory for large models)
        """
        ...

    def __del__(self) -> None: ...

    # Properties
    @property
    def num_nodes(self) -> int:
        """Get the number of nodes in the model"""
        ...

    @property
    def num_dim(self) -> int:
        """Get the number of spatial dimensions"""
        ...

    @property
    def nodes(self) -> List[List[float]]:
        """Get node coordinates as [node_id][dimension]"""
        ...

    @nodes.setter
    def nodes(self, node_list: List[List[float]]) -> None:
        """Set node coordinates"""
        ...

    # Counting methods
    def get_node_count(self) -> int:
        """Get the total number of nodes"""
        ...

    def get_element_count(
        self, element_block_ids: Union[str, List[int]] = "all"
    ) -> int:
        """
        Get the total number of elements.

        Args:
            element_block_ids: "all" or list of block IDs to count

        Returns:
            Total number of elements
        """
        ...

    def get_element_block_ids(self) -> List[int]:
        """Get list of all element block IDs"""
        ...

    def element_block_exists(self, element_block_id: int) -> bool:
        """Check if an element block exists"""
        ...

    # Title and metadata
    def get_title(self) -> str:
        """Get the model title"""
        ...

    def set_title(self, title: str) -> None:
        """Set the model title"""
        ...

    def to_lowercase(self) -> None:
        """Convert all names to lowercase"""
        ...

    def get_qa_records(self) -> List[Tuple[str, str, str, str]]:
        """Get QA records as list of (code, version, date, time) tuples"""
        ...

    def add_qa_record(
        self,
        code_name: Optional[str] = None,
        code_version: Optional[str] = None,
        date: Optional[str] = None,
        time: Optional[str] = None,
    ) -> None:
        """Add a QA record for provenance tracking"""
        ...

    def get_info_records(self) -> List[str]:
        """Get information records"""
        ...

    def add_info_record(self, record: str) -> None:
        """Add an information record"""
        ...

    def get_timesteps(self) -> List[float]:
        """Get list of all time step values"""
        ...

    # Import/Export
    def import_model(self, filename: str, **kwargs: Any) -> None:
        """
        Import data from an ExodusII file.

        Args:
            filename: Path to the Exodus file
            **kwargs: Additional import options
        """
        ...

    def export_model(self, filename: str, **kwargs: Any) -> None:
        """
        Export model to an ExodusII file.

        Args:
            filename: Output file path
            **kwargs: Additional export options (e.g., clobber=True)
        """
        ...

    # Node operations
    def get_nodes(self) -> List[List[float]]:
        """Get node coordinates as [node_id][dimension]"""
        ...

    def get_coords_flat(self) -> Tuple[List[float], List[float], List[float]]:
        """Get node coordinates as separate (x, y, z) arrays"""
        ...

    def create_nodes(self, nodes: List[List[float]]) -> None:
        """
        Create nodes in the model.

        Args:
            nodes: Node coordinates as [node_id][dimension]
        """
        ...

    def delete_node(self, node_indices: Union[int, List[int]]) -> None:
        """Delete specified nodes"""
        ...

    def delete_unused_nodes(self) -> int:
        """
        Delete nodes not referenced by any elements.

        Returns:
            Number of nodes deleted
        """
        ...

    # Element block operations
    def get_connectivity_flat(self, block_id: int) -> List[int]:
        """Get element connectivity as flat array"""
        ...

    def get_connectivity(
        self, block_id: Union[int, str]
    ) -> List[List[int]]:
        """
        Get element connectivity as nested list.

        Args:
            block_id: Element block ID

        Returns:
            Connectivity as [element_id][node_indices]
        """
        ...

    def get_nodes_per_element(self, block_id: int) -> int:
        """Get number of nodes per element in block"""
        ...

    def get_element_block_dimension(self, block_id: int) -> int:
        """Get spatial dimension of element block (1D, 2D, or 3D)"""
        ...

    def get_element_block_name(self, block_id: int) -> str:
        """Get element block name"""
        ...

    def set_element_block_name(self, block_id: int, name: str) -> None:
        """Set element block name"""
        ...

    def create_element_block(
        self,
        block_id: int,
        info: List[Any],
        connectivity: Optional[List[List[int]]] = None,
    ) -> None:
        """
        Create a new element block.

        Args:
            block_id: Unique block ID
            info: [topology, num_elements, num_nodes_per_element]
            connectivity: Optional connectivity array
        """
        ...

    def set_connectivity(
        self, block_id: int, connectivity: List[List[int]]
    ) -> None:
        """Set element connectivity for a block"""
        ...

    def delete_element_block(
        self,
        element_block_ids: Union[int, List[int]],
        delete_orphaned_nodes: bool = True,
    ) -> None:
        """
        Delete element blocks.

        Args:
            element_block_ids: Block ID or list of block IDs
            delete_orphaned_nodes: Delete nodes not referenced by remaining elements
        """
        ...

    def rename_element_block(
        self, element_block_id: int, new_element_block_id: Union[int, str]
    ) -> None:
        """Rename (renumber) an element block"""
        ...

    def get_all_element_block_names(self) -> Dict[int, str]:
        """Get mapping of block IDs to names"""
        ...

    def get_element_block_connectivity(
        self, element_block_id: Union[str, int] = "auto"
    ) -> List[List[int]]:
        """Get element connectivity for a block"""
        ...

    def get_nodes_in_element_block(
        self, element_block_ids: Union[str, int, List[int]]
    ) -> List[int]:
        """Get list of nodes used by specified element blocks"""
        ...

    # Geometric transformations
    def translate_geometry(self, offset: List[float]) -> None:
        """
        Translate all nodes by offset.

        Args:
            offset: Translation vector [dx, dy, dz]
        """
        ...

    def rotate_geometry(
        self,
        axis: List[float],
        angle_in_degrees: float,
        center: Optional[List[float]] = None,
    ) -> None:
        """
        Rotate geometry about an axis.

        Args:
            axis: Rotation axis vector [x, y, z]
            angle_in_degrees: Rotation angle in degrees
            center: Optional rotation center (default: origin)
        """
        ...

    def scale_geometry(
        self,
        scale_factor: float,
        adjust_displacement_field: Union[str, bool] = "auto",
    ) -> None:
        """
        Scale geometry by factor.

        Args:
            scale_factor: Scaling factor
            adjust_displacement_field: Whether to scale displacement fields
        """
        ...

    def translate_element_blocks(
        self,
        element_block_ids: Union[str, List[int]],
        offset: List[float],
        allow_shared_nodes: bool = False,
    ) -> None:
        """Translate specific element blocks"""
        ...

    def scale_element_blocks(
        self,
        element_block_ids: Union[str, List[int]],
        scale_factor: float,
        allow_shared_nodes: bool = False,
        adjust_displacement_field: Union[str, bool] = "auto",
    ) -> None:
        """Scale specific element blocks"""
        ...

    def rotate_element_blocks(
        self,
        element_block_ids: Union[str, List[int]],
        axis: List[float],
        angle_in_degrees: float,
        center: Optional[List[float]] = None,
        allow_shared_nodes: bool = False,
    ) -> None:
        """Rotate specific element blocks"""
        ...

    # Node sets
    def create_node_set(
        self,
        node_set_id: int,
        node_set_members: Optional[List[int]] = None,
    ) -> None:
        """Create a new node set"""
        ...

    def node_set_exists(self, node_set_id: int) -> bool:
        """Check if node set exists"""
        ...

    def get_node_set_members(self, node_set_id: int) -> List[int]:
        """Get list of nodes in a node set"""
        ...

    def delete_node_set(self, node_set_ids: Union[int, List[int]]) -> None:
        """Delete node sets"""
        ...

    def get_node_set_ids(self) -> List[int]:
        """Get list of all node set IDs"""
        ...

    def get_node_set_name(self, node_set_id: int) -> str:
        """Get node set name"""
        ...

    def rename_node_set(self, node_set_id: int, new_name: str) -> None:
        """Rename a node set"""
        ...

    # Side sets
    def create_side_set(
        self,
        side_set_id: int,
        members: Optional[List[Tuple[int, int]]] = None,
    ) -> None:
        """
        Create a new side set.

        Args:
            side_set_id: Unique side set ID
            members: List of (element_id, side_number) tuples
        """
        ...

    def side_set_exists(self, side_set_id: int) -> bool:
        """Check if side set exists"""
        ...

    def get_side_set_members(
        self, side_set_id: int
    ) -> List[Tuple[int, int]]:
        """
        Get side set members.

        Returns:
            List of (element_id, side_number) tuples
        """
        ...

    def delete_side_set(self, side_set_ids: Union[int, List[int]]) -> None:
        """Delete side sets"""
        ...

    def get_side_set_ids(self) -> List[int]:
        """Get list of all side set IDs"""
        ...

    def get_side_set_name(self, side_set_id: int) -> str:
        """Get side set name"""
        ...

    def rename_side_set(self, side_set_id: int, new_name: str) -> None:
        """Rename a side set"""
        ...

    def add_faces_to_side_set(
        self, side_set_id: int, new_side_set_members: List[Tuple[int, int]]
    ) -> None:
        """Add faces to an existing side set"""
        ...

    # Node fields (nodal variables)
    def create_node_field(
        self, field_name: str, timesteps: Optional[List[int]] = None
    ) -> None:
        """Create a new nodal variable field"""
        ...

    def node_field_exists(self, field_name: str) -> bool:
        """Check if nodal field exists"""
        ...

    def get_node_field_names(self) -> List[str]:
        """Get list of all nodal variable names"""
        ...

    def get_node_field_values(
        self, node_field_name: str, timestep: Union[str, float] = "last"
    ) -> List[float]:
        """
        Get nodal variable values at a time step.

        Args:
            node_field_name: Variable name
            timestep: Time step value or "last"

        Returns:
            Values for all nodes
        """
        ...

    def delete_node_field(
        self, node_field_names: Union[str, List[str]]
    ) -> None:
        """Delete nodal variables"""
        ...

    def rename_node_field(
        self, node_field_name: str, new_node_field_name: str
    ) -> None:
        """Rename a nodal variable"""
        ...

    def node_set_field_exists(
        self, field_name: str, node_set_id: int
    ) -> bool:
        """Check if node set variable exists"""
        ...

    # Element fields (element variables)
    def create_element_field(
        self,
        field_name: str,
        block_id: int,
        timesteps: Optional[List[int]] = None,
    ) -> None:
        """Create a new element variable field"""
        ...

    def element_field_exists(self, block_id: int, field_name: str) -> bool:
        """Check if element field exists"""
        ...

    def get_element_field_names(
        self, element_block_ids: Union[str, List[int]] = "all"
    ) -> List[str]:
        """Get list of element variable names"""
        ...

    def delete_element_field(
        self,
        element_field_names: Union[str, List[str]],
        element_block_ids: Union[str, List[int]] = "all",
    ) -> None:
        """Delete element variables"""
        ...

    def rename_element_field(
        self,
        element_field_name: str,
        new_element_field_name: str,
        element_block_ids: Union[str, List[int]] = "all",
    ) -> None:
        """Rename element variables"""
        ...

    def side_set_field_exists(
        self, field_name: str, side_set_id: int
    ) -> bool:
        """Check if side set variable exists"""
        ...

    # Global variables
    def global_variable_exists(self, var_name: str) -> bool:
        """Check if global variable exists"""
        ...

    def get_global_variable_names(self) -> List[str]:
        """Get list of global variable names"""
        ...

    def create_global_variable(
        self,
        global_variable_name: str,
        value: Union[str, float, List[float]] = "auto",
    ) -> None:
        """Create a new global variable"""
        ...

    def delete_global_variable(
        self, global_variable_names: Union[str, List[str]]
    ) -> None:
        """Delete global variables"""
        ...

    def rename_global_variable(
        self, global_variable_name: str, new_global_variable_name: str
    ) -> None:
        """Rename a global variable"""
        ...

    # Time steps
    def timestep_exists(self, timestep: float) -> bool:
        """Check if time step exists"""
        ...

    def create_timestep(self, timestep: float) -> None:
        """Create a new time step"""
        ...

    def delete_timestep(self, timesteps: Union[float, List[float]]) -> None:
        """Delete time steps"""
        ...

    # Utility
    def summarize(self) -> None:
        """Print a summary of the model contents"""
        ...
