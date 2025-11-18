# Comprehensive API Analysis: exodus-py (Rust-based)

## Executive Summary

The exodus-py package is a complete Python binding to the Rust-based exodus-rs library, providing a modern, high-performance interface for reading, writing, and manipulating Exodus II finite element analysis files. The implementation uses PyO3 for Python interoperability and exposes three file access modes (Read, Write, Append) with a comprehensive feature set for mesh manipulation.

---

## 1. File Operations & Access Modes

### 1.1 ExodusReader - Read-Only Access
**File:** `/home/user/seacas/rust/exodus-py/src/file.rs`

**Purpose:** Open and read existing Exodus files

**Key Methods:**
- `ExodusReader.open(path: str) -> ExodusReader` - Static method to open files
- `init_params() -> InitParams` - Get database dimensions and metadata
- `path() -> str` - Get file path
- `version() -> (major, minor)` - Get Exodus format version
- `format() -> str` - Get NetCDF file format
- `close()` - Close file (auto-closes when dropped)
- Context manager support (`__enter__`, `__exit__`)

**Features:**
- Read-only mode with automatic file closure
- Full context manager support
- Version and format introspection

---

### 1.2 ExodusWriter - Write/Create Mode
**File:** `/home/user/seacas/rust/exodus-py/src/file.rs`

**Purpose:** Create new Exodus files and write data

**Key Methods:**
- `ExodusWriter.create(path: str, options: CreateOptions = None) -> ExodusWriter` - Create new file
- `put_init_params(params: InitParams)` - Initialize database structure
- `path() -> str` - Get file path
- `sync()` - Flush buffered data to disk
- `close()` - Explicitly close file
- Context manager support

**Features:**
- File creation with optional configuration
- Data flushing for consistency
- Automatic cleanup via context manager

---

### 1.3 ExodusAppender - Read-Write Mode
**File:** `/home/user/seacas/rust/exodus-py/src/file.rs`

**Purpose:** Open existing files for modification and appending

**Key Methods:**
- `ExodusAppender.append(path: str) -> ExodusAppender` - Static method to open for modification
- `init_params() -> InitParams` - Get current database parameters
- `path() -> str` - Get file path
- `close()` - Close file
- Context manager support

**Features:**
- Read-write access to existing files
- Preserves existing data while allowing modifications
- Full context manager support

---

## 2. Type System & Data Structures

### 2.1 Enumerations
**File:** `/home/user/seacas/rust/exodus-py/src/types.rs`

#### EntityType Enum
Represents different entity categories in the mesh:
- **Mesh Components:** `ElemBlock`, `NodeSet`, `SideSet`, `EdgeBlock`, `EdgeSet`, `FaceBlock`, `FaceSet`, `ElemSet`
- **Maps:** `NodeMap`, `ElemMap`, `EdgeMap`, `FaceMap`
- **Variables:** `Global`, `Nodal`
- **Advanced:** `Assembly`, `Blob`

#### CreateMode Enum
File creation behavior:
- `Clobber` - Overwrite existing files
- `NoClobber` - Fail if file exists

#### FloatSize Enum
Floating-point precision:
- `Float32` - 32-bit precision
- `Float64` - 64-bit (double) precision (default)

#### Int64Mode Enum
Integer storage:
- `Int32` - 32-bit integers
- `Int64` - 64-bit integers (default)

#### AttributeType Enum
Attribute data types:
- `Integer` - Integer attributes
- `Double` - Double precision floats
- `Char` - Character/string attributes

---

### 2.2 Core Data Classes

#### InitParams
Database initialization parameters:
```python
InitParams(
    title: str,                    # Database title (max 80 chars)
    num_dim: int,                  # Spatial dimensions (1, 2, or 3)
    num_nodes: int,                # Total nodes
    num_edges: int = 0,            # Total edges
    num_edge_blocks: int = 0,      # Edge block count
    num_faces: int = 0,            # Total faces
    num_face_blocks: int = 0,      # Face block count
    num_elems: int,                # Total elements
    num_elem_blocks: int,          # Element block count
    num_node_sets: int = 0,        # Node set count
    num_edge_sets: int = 0,        # Edge set count
    num_face_sets: int = 0,        # Face set count
    num_side_sets: int = 0,        # Side set count
    num_elem_sets: int = 0,        # Element set count
    num_node_maps: int = 0,        # Node map count
    num_edge_maps: int = 0,        # Edge map count
    num_face_maps: int = 0,        # Face map count
    num_elem_maps: int = 0,        # Element map count
    num_assemblies: int = 0,       # Assembly count
    num_blobs: int = 0             # Blob count
)
```

#### CreateOptions
File creation configuration:
```python
CreateOptions(
    mode: CreateMode = CreateMode.Clobber,
    float_size: FloatSize = FloatSize.Float64,
    int64_mode: Int64Mode = Int64Mode.Int64,
    performance: PerformanceConfig = None
)
```

#### Block
Element/Edge/Face block definition:
```python
Block(
    id: i64,                       # Unique block ID
    entity_type: EntityType,       # ElemBlock, EdgeBlock, or FaceBlock
    topology: str,                 # Topology name (e.g., "HEX8", "QUAD4")
    num_entries: int,              # Elements/edges/faces count
    num_nodes_per_entry: int,      # Nodes per element
    num_edges_per_entry: int = 0,  # Edges per element
    num_faces_per_entry: int = 0,  # Faces per element
    num_attributes: int = 0        # Attributes per element
)
```

#### NodeSet
Node grouping:
```python
NodeSet(
    id: i64,
    nodes: Vec<i64>,               # 1-based node IDs
    dist_factors: Vec<f64> = []    # Distribution factors (optional)
)
```

#### SideSet
Element-side grouping (for boundary conditions):
```python
SideSet(
    id: i64,
    elements: Vec<i64>,            # Element IDs
    sides: Vec<i64>,               # Side numbers within elements
    dist_factors: Vec<f64> = []    # Distribution factors (optional)
)
```

#### EntitySet
Generic entity grouping (edges, faces, or elements):
```python
EntitySet(
    id: i64,
    entity_type: EntityType,       # EdgeSet, FaceSet, or ElemSet
    entities: Vec<i64>             # Entity IDs
)
```

#### Assembly
Hierarchical grouping:
```python
Assembly(
    id: i64,
    name: str,
    entity_type: EntityType,       # Type of contained entities
    entity_list: Vec<i64>          # Entity IDs in assembly
)
```

#### Blob
Binary data storage:
```python
Blob(
    id: i64,
    name: str
)
```

#### QaRecord
Provenance tracking:
```python
QaRecord(
    code_name: str,                # Application name
    code_version: str,             # Version string
    date: str,                     # Date (e.g., "2025-01-15")
    time: str                      # Time (e.g., "14:30:00")
)
```

#### TruthTable
Sparse variable storage indicator:
```python
TruthTable.new(var_type, num_blocks, num_vars)
table.set(block_idx, var_idx, exists: bool)
value = table.get(block_idx, var_idx)
```

#### AttributeData
Typed attribute values:
```python
# Creation methods
AttributeData.integer(values: Vec<i64>)
AttributeData.double(values: Vec<f64>)
AttributeData.char(value: str)

# Extraction methods
attr.as_integer() -> Vec<i64>
attr.as_double() -> Vec<f64>
attr.as_char() -> str
```

---

## 3. File Operations by Component

### 3.1 Coordinate Operations
**File:** `/home/user/seacas/rust/exodus-py/src/coord.rs`

#### Writing (ExodusWriter)
- `put_coords(x, y=None, z=None)` - Write nodal coordinates
- `put_coord_names(names)` - Write coordinate axis names

#### Reading (ExodusReader)
- `get_coords() -> (x, y, z)` - Read all coordinates as tuple
- `get_coord_x() -> Vec<f64>` - Read X coordinates only
- `get_coord_y() -> Vec<f64>` - Read Y coordinates only
- `get_coord_z() -> Vec<f64>` - Read Z coordinates only
- `get_coord_names() -> Vec<str>` - Read coordinate axis names

#### Appending (ExodusAppender)
- `put_coords(x, y=None, z=None)` - Update coordinates
- `get_coords() -> (x, y, z)` - Read current coordinates

---

### 3.2 Block/Element Operations
**File:** `/home/user/seacas/rust/exodus-py/src/block.rs`

#### Writing (ExodusWriter)
- `put_block(block: Block)` - Define block structure
- `put_connectivity(block_id, nodes)` - Write element connectivity (1-based)
- `put_block_attributes(block_id, values)` - Write element attributes
- `put_block_attribute_names(block_id, names)` - Define attribute names

#### Reading (ExodusReader)
- `get_block(block_id) -> Block` - Read block definition
- `get_block_ids() -> Vec<i64>` - Get all block IDs
- `get_connectivity(block_id) -> Vec<i64>` - Read element connectivity
- `get_block_attributes(block_id) -> Vec<f64>` - Read element attributes
- `get_block_attribute_names(block_id) -> Vec<str>` - Read attribute names

#### Appending (ExodusAppender)
- `get_block_ids() -> Vec<i64>` - Get block IDs (limited support)

**Topology Support:**
Common topologies: HEX8, QUAD4, TET4, PYRAMID5, WEDGE6, TRI3, LINE2, and higher-order variants

---

### 3.3 Set Operations (Node/Side/Entity Sets)
**File:** `/home/user/seacas/rust/exodus-py/src/set.rs`

#### Writing (ExodusWriter)
- `put_set(entity_type, set_id, num_entries, num_dist_factors)` - Define set
- `put_node_set(set_id, nodes, dist_factors=None)` - Write node set
- `put_side_set(set_id, elements, sides, dist_factors=None)` - Write side set
- `put_entity_set(entity_type, set_id, entities)` - Write edge/face/element set

#### Reading (ExodusReader)
- `get_node_set(set_id) -> NodeSet` - Read node set
- `get_side_set(set_id) -> SideSet` - Read side set
- `get_entity_set(entity_type, set_id) -> EntitySet` - Read entity set
- `get_node_set_ids() -> Vec<i64>` - Get all node set IDs
- `get_side_set_ids() -> Vec<i64>` - Get all side set IDs
- `get_elem_set_ids() -> Vec<i64>` - Get all element set IDs
- `get_set_ids(entity_type) -> Vec<i64>` - Get IDs for specific type
- `convert_nodeset_to_sideset(nodeset_id, new_sideset_id) -> SideSet` - Convert nodeset to sideset (boundary faces only)

#### Appending (ExodusAppender)
- `create_sideset_from_nodeset(nodeset_id, new_sideset_id)` - Convert and write sideset

---

### 3.4 Variable Management
**File:** `/home/user/seacas/rust/exodus-py/src/variable.rs`

#### Writing (ExodusWriter)
- `define_variables(var_type, names)` - Define variables for entity type
- `put_time(step, time)` - Write time value
- `put_var(step, var_type, entity_id, var_index, values)` - Write single variable
- `put_var_multi(step, var_type, entity_id, values)` - Write all variables
- `put_var_time_series(start, end, var_type, entity_id, var_index, values)` - Write across time steps
- `put_truth_table(var_type, table)` - Set sparse variable definitions
- `define_reduction_variables(var_type, names)` - Define reduction variables
- `put_reduction_vars(step, var_type, entity_id, values)` - Write reduction variables

#### Reading (ExodusReader)
- `variable_names(var_type) -> Vec<str>` - Get variable names
- `num_time_steps() -> int` - Get time step count
- `times() -> Vec<f64>` - Get all time values
- `time(step) -> f64` - Get single time value
- `var(step, var_type, entity_id, var_index) -> Vec<f64>` - Read single variable
- `var_multi(step, var_type, entity_id) -> Vec<f64>` - Read all variables
- `var_time_series(start, end, var_type, entity_id, var_index) -> Vec<f64>` - Read time series
- `truth_table(var_type) -> TruthTable` - Get sparse variable definitions
- `reduction_variable_names(var_type) -> Vec<str>` - Get reduction variable names
- `get_reduction_vars(step, var_type, entity_id) -> Vec<f64>` - Read reduction variables

**Variable Types:**
- Global variables (scalar, across entire database)
- Nodal variables (one value per node)
- Element block variables (one value per element in block)
- Reduction variables (aggregated values for objects)

---

### 3.5 ID Maps and Properties
**File:** `/home/user/seacas/rust/exodus-py/src/map.rs`

#### Writing (ExodusWriter)
- `put_id_map(entity_type, id_map)` - Write ID maps
  - entity_type: "node", "elem", "edge", "face"
- `put_elem_order_map(order)` - Write element processing order
- `put_name(entity_type, entity_index, name)` - Name single entity
- `put_names(entity_type, names)` - Name all entities of type
- `put_property(entity_type, entity_id, prop_name, value)` - Set property on entity
- `put_property_array(entity_type, prop_name, values)` - Set properties on all entities

#### Reading (ExodusReader)
- `get_id_map(entity_type) -> Vec<i64>` - Read ID maps
- `get_elem_order_map() -> Vec<i64>` - Read element order
- `get_name(entity_type, entity_index) -> str` - Get single entity name
- `get_names(entity_type) -> Vec<str>` - Get all entity names
- `get_property(entity_type, entity_id, prop_name) -> i64` - Get single property
- `get_property_array(entity_type, prop_name) -> Vec<i64>` - Get property array
- `get_property_names(entity_type) -> Vec<str>` - Get available properties

**Supported Entities:** elem_block, edge_block, face_block, node_set, edge_set, face_set, elem_set, side_set, node_map, edge_map, face_map, elem_map

---

### 3.6 Assemblies and Blobs
**File:** `/home/user/seacas/rust/exodus-py/src/assembly.rs`

#### Writing (ExodusWriter)
- `put_assembly(assembly: Assembly)` - Write assembly definition
- `put_blob(blob: Blob, data: bytes)` - Write binary blob data

#### Reading (ExodusReader)
- `get_assembly(assembly_id) -> Assembly` - Read assembly
- `get_blob(blob_id) -> (Blob, bytes)` - Read blob with data
- `get_assembly_ids() -> Vec<i64>` - Get all assembly IDs
- `get_blob_ids() -> Vec<i64>` - Get all blob IDs

---

### 3.7 Metadata (QA & Info Records)
**File:** `/home/user/seacas/rust/exodus-py/src/metadata.rs`

#### Writing (ExodusWriter)
- `put_info_records(info: Vec<str>)` - Write info records (arbitrary text, max 80 chars each)
- `put_qa_records(records: Vec<QaRecord>)` - Write provenance tracking

#### Reading (ExodusReader)
- `get_qa_records() -> Vec<QaRecord>` - Read provenance records
- `get_info_records() -> Vec<str>` - Read info records

---

### 3.8 Attributes
**File:** `/home/user/seacas/rust/exodus-py/src/attribute.rs`

#### Writing (ExodusWriter)
- `put_attribute(entity_type, entity_id, name, attr_type, data)` - Write attribute

#### Reading (ExodusReader)
- `get_attribute(entity_type, entity_id, name) -> AttributeData` - Read attribute
- `get_attribute_names(entity_type, entity_id) -> Vec<str>` - List attribute names

**Attribute Types:**
- Integer attributes (array of i64)
- Double attributes (array of f64)
- Character attributes (single string)

---

## 4. Builder API (High-Level Mesh Creation)

### 4.1 MeshBuilder
**File:** `/home/user/seacas/rust/exodus-py/src/builder.rs`

**Purpose:** Fluent API for creating Exodus files from scratch

**Methods:**

**Initialization:**
```python
builder = MeshBuilder("My Mesh Title")
```

**Configuration (both fluent and setter variants):**
```python
# Fluent API
builder.dimensions(3).coordinates(x, y, z).add_block(block).write("output.exo")

# Or setter API
builder.set_dimensions(3)
builder.set_coordinates(x, y, z)
builder.add_block(block)
builder.write("output.exo")
```

**Methods:**
- `dimensions(num_dim) -> Self` / `set_dimensions(num_dim)` - Set spatial dimensions
- `coordinates(x, y=[], z=[]) -> Self` / `set_coordinates(x, y=[], z=[])` - Set nodal coordinates
- `add_block(BlockBuilder) -> Self` - Add element block
- `qa_record(code_name, version, date, time) -> Self` / `add_qa_record(...)` - Add provenance
- `info(text) -> Self` / `add_info(text)` - Add info records
- `write(path)` - Write to file with default options
- `write_with_options(path, CreateOptions)` - Write with custom options

---

### 4.2 BlockBuilder
**File:** `/home/user/seacas/rust/exodus-py/src/builder.rs`

**Purpose:** Fluent API for building element blocks

**Methods:**
```python
block = (BlockBuilder(block_id, "HEX8")
    .connectivity([1, 2, 3, 4, 5, 6, 7, 8])
    .attributes([100.0])
    .attribute_names(["MaterialID"])
    .build())
```

- `connectivity(nodes) -> Self` - Set element connectivity (1-based)
- `attributes(values) -> Self` - Set element attributes
- `attribute_names(names) -> Self` - Set attribute names
- `build() -> Self` - Finalize (no-op, for API consistency)

---

## 5. Performance Configuration

### 5.1 Performance Types
**File:** `/home/user/seacas/rust/exodus-py/src/performance.rs`

#### NodeType
HPC environment detection:
```python
NodeType.detect()      # Auto-detect
NodeType.compute()     # Force Compute node settings
NodeType.login()       # Force Login node settings
NodeType.unknown()     # Force Unknown/local settings

# Methods
node_type.default_cache_size() -> int
node_type.default_chunk_nodes() -> int
node_type.default_chunk_elements() -> int
```

#### CacheConfig
HDF5 chunk cache tuning:
```python
cache = CacheConfig(
    cache_size=256*1024*1024,  # 256 MB
    num_slots=0,                # Auto-calculate
    preemption=0.75             # Preemption policy (0.0-1.0)
)

# Fluent methods
cache.with_cache_size(size)
cache.with_cache_mb(mb)        # Convenience method
cache.with_slots(num_slots)
cache.with_preemption(value)

# Properties
cache.cache_size, cache.num_slots, cache.preemption
```

#### ChunkConfig
HDF5 chunk size tuning:
```python
chunks = ChunkConfig(
    node_chunk_size=20000,       # Nodes per chunk
    element_chunk_size=10000,    # Elements per chunk
    time_chunk_size=0            # Time steps per chunk
)

# Fluent methods
chunks.with_node_chunk_size(size)
chunks.with_element_chunk_size(size)
chunks.with_time_chunk_size(size)

# Properties
chunks.node_chunk_size, chunks.element_chunk_size, chunks.time_chunk_size
```

#### PerformanceConfig
Complete I/O optimization:
```python
# Predefined configurations
perf = PerformanceConfig.auto()          # Auto-detect and optimize
perf = PerformanceConfig.conservative()  # Login node settings
perf = PerformanceConfig.aggressive()    # Compute node settings

# Custom fluent API
perf = (PerformanceConfig.auto()
    .with_cache_mb(256)
    .with_node_chunk_size(20000))
```

---

## 6. ExoMerge - High-Level Mesh Manipulation

### 6.1 Overview
**File:** `/home/user/seacas/rust/exodus-py/python/exodus/exomerge.py`

ExoMerge provides a comprehensive Python API for mesh manipulation, built on top of exodus-py. It offers both in-memory and streaming modes for handling large files.

### 6.2 ExodusModel Class

**Initialization:**
```python
model = ExodusModel(mode="inmemory")  # or "streaming"
```

**Modes:**
- `"inmemory"` - Load all data into memory (faster, requires more RAM)
- `"streaming"` - Keep file open, lazy load data (slower, minimal RAM)

### 6.3 File I/O Methods

**Import/Export:**
- `import_model(filename)` - Read Exodus file
- `export_model(filename)` - Write modified model

### 6.4 Mesh Query Methods

**Dimensions & Counts:**
- `num_nodes` - Node count (property)
- `num_dim` - Spatial dimensions (property)
- `get_node_count() -> int`
- `get_element_count(element_block_ids) -> int`
- `get_element_block_ids() -> List[int]`
- `element_block_exists(block_id) -> bool`

**Geometry Access:**
- `coords_x, coords_y, coords_z` - Coordinate arrays (direct access)
- `get_coords_flat() -> (x, y, z)` - Flat coordinate arrays
- `get_nodes() -> List[List[float]]` - Nodes as [x, y, z] lists
- `get_connectivity_flat(block_id) -> Vec<i64>` - Flat connectivity
- `get_connectivity(block_id) -> Vec<i64>` - Connectivity (same as above)
- `get_nodes_per_element(block_id) -> int`
- `get_element_block_dimension(block_id) -> int`

**Block Operations:**
- `element_blocks` - Dict of ElementBlockData indexed by ID
- `get_element_block_name(block_id) -> str`
- `set_element_block_name(block_id, name)`
- `rename_element_block(block_id, name)`
- `get_all_element_block_names() -> Dict[int, str]`
- `get_element_block_connectivity(block_id) -> Vec<i64>`
- `get_nodes_in_element_block(block_ids) -> Set<int>`

**Set Operations:**
- `node_sets` - Dict of NodeSetData indexed by ID
- `side_sets` - Dict of SideSetData indexed by ID
- `get_node_set_ids() -> List[int]`
- `get_side_set_ids() -> List[int]`
- `node_set_exists(node_set_id) -> bool`
- `side_set_exists(side_set_id) -> bool`
- `get_node_set_name(node_set_id) -> str`
- `get_side_set_name(side_set_id) -> str`
- `get_node_set_members(node_set_id) -> List[int]`
- `get_side_set_members(side_set_id) -> List[Tuple[int, int]]`
- `get_nodes_in_node_set(node_set_id) -> Set<int>`
- `get_nodes_in_side_set(side_set_id) -> Set<int>`
- `get_all_node_set_names() -> Dict[int, str]`
- `get_all_side_set_names() -> Dict[int, str]`

### 6.5 Mesh Creation Methods

**Coordinate Creation:**
- `create_nodes(nodes: List[List[float]])`

**Block Creation:**
- `create_element_block(block_id, topology, elements)`
- `set_connectivity(block_id, connectivity)`

**Set Creation:**
- `create_node_set(node_set_id, nodes, name="")`
- `create_side_set(side_set_id, element_sides, name="")`

**Field Creation:**
- `create_node_field(field_name, num_components=1)` / `create_node_field(name, ...)`
- `create_element_field(field_name, block_id, num_components=1)`
- `create_global_variable(variable_name)`

**Set Management:**
- `add_nodes_to_node_set(node_set_id, nodes)`
- `add_faces_to_side_set(side_set_id, element_sides)`
- `create_node_set_field(field_name, node_set_id)`
- `create_side_set_field(field_name, side_set_id)`

### 6.6 Geometry Transformation Methods

**Translation:**
- `translate_geometry(offset: Tuple[float, float, float])`
- `translate_element_blocks(block_ids, offset)`

**Scaling:**
- `scale_geometry(scale_factor: float)`
- `scale_element_blocks(block_ids, scale_factor)`

**Rotation:**
- `rotate_geometry(axis: str, angle: float)` - axis: "x", "y", "z"
- `rotate_element_blocks(block_ids, axis, angle)`

**Reflection:**
- `reflect_element_blocks(block_ids, axis: str)`

**Displacement:**
- `displace_element_blocks(block_ids)` - Apply displacement field

### 6.7 Block Manipulation Methods

**Block Deletion & Creation:**
- `delete_element_block(element_block_ids)`
- `delete_unused_nodes() -> int` - Returns count of deleted nodes
- `duplicate_element_block(source_id, new_id)`
- `combine_element_blocks(element_block_ids)`
- `unmerge_element_blocks(element_block_ids)`

**Block Merging & Node Operations:**
- `merge_nodes(tolerance: float)` - Merge nearby nodes
- `delete_node(node_indices)`

**Element Conversion:**
- `convert_element_blocks(block_id)` - Convert between types
- `convert_hex8_block_to_tet4_block(block_id)`
- `make_elements_linear(block_ids)` - Higher-order to linear
- `make_elements_quadratic(block_ids)` - Linear to higher-order

**Set Conversion:**
- `convert_side_set_to_cohesive_zone(side_set_ids)` - Create interface elements

### 6.8 Set Manipulation Methods

**Set Deletion & Management:**
- `delete_node_set(node_set_ids)`
- `delete_side_set(side_set_ids)`
- `delete_empty_node_sets()`
- `delete_empty_side_sets()`
- `rename_node_set(node_set_id, new_name)`
- `rename_side_set(side_set_id, new_name)`

**Set Analysis:**
- `get_side_set_area(side_set_id) -> float` - Calculate boundary area

### 6.9 Field/Variable Methods

**Field/Variable Creation & Deletion:**
- `node_field_exists(field_name) -> bool`
- `element_field_exists(block_id, field_name) -> bool`
- `global_variable_exists(var_name) -> bool`
- `node_set_field_exists(field_name) -> bool`
- `side_set_field_exists(field_name) -> bool`
- `delete_node_field(node_field_names)`
- `delete_element_field(element_field_names)`
- `delete_global_variable(global_variable_names)`
- `delete_node_set_field(field_names)`
- `delete_side_set_field(field_names)`

**Field/Variable Renaming:**
- `rename_node_field(field_name, new_name)`
- `rename_element_field(field_name, new_name)`
- `rename_global_variable(variable_name, new_name)`
- `rename_node_set_field(field_name, new_name)`
- `rename_side_set_field(field_name, new_name)`

**Field/Variable Reading:**
- `get_node_field_names() -> List[str]`
- `get_node_field_values(field_name) -> List[...] per timestep`
- `get_element_field_names(block_ids) -> List[str]`
- `get_element_field_values(field_name) -> List[...]`
- `get_global_variable_names() -> List[str]`
- `get_node_set_field_names(node_set_id) -> List[str]`
- `get_node_set_field_values(field_name) -> List[...]`
- `get_side_set_field_names(side_set_id) -> List[str]`
- `get_side_set_field_values(field_name) -> List[...]`

**Field/Variable Conversion:**
- `convert_element_field_to_node_field(element_field_name)`
- `convert_node_field_to_element_field(node_field_name)`
- `create_averaged_element_field(field_names)` - Average element fields

**Field/Variable Calculation:**
- `calculate_element_field(new_field_name, expression)`
- `calculate_node_field(new_field_name, expression)`
- `calculate_global_variable(new_var_name, expression)`
- `calculate_side_set_field(new_field_name, expression)`
- `calculate_node_set_field(new_field_name, expression)`
- `process_element_fields(block_id)` - Post-process fields
- `output_global_variables(expressions)` - Export global variable calculations

### 6.10 Time-Related Methods

**Timestep Management:**
- `timesteps` - List of time values (property)
- `get_timesteps() -> List[float]`
- `timestep_exists(timestep) -> bool`
- `create_timestep(timestep)`
- `delete_timestep(timesteps)`
- `copy_timestep(timestep)` - Duplicate timestep
- `create_interpolated_timestep(timestep)` - Linear interpolation

### 6.11 Analysis & Quality Methods

**Geometry Analysis:**
- `get_element_block_extents(block_ids) -> Dict` - Min/max coordinates
- `get_element_block_centroid(block_ids) -> Tuple[x, y, z]`
- `get_element_edge_length_info(block_ids) -> Dict` - Min/max/avg edge lengths
- `get_closest_node_distance() -> float`
- `get_length_scale() -> float`

**Element Quality:**
- `count_degenerate_elements(block_ids) -> int`
- `count_disconnected_blocks(block_ids) -> int`
- `delete_duplicate_elements(block_ids)`

**Volume Calculations:**
- `calculate_element_volumes(block_ids)` - Create volume field
- `get_element_block_volume(block_ids) -> float`

**Field Statistics:**
- `calculate_element_field_maximum(field_names) -> Dict`
- `calculate_element_field_minimum(field_names) -> Dict`
- `calculate_node_field_maximum(field_names) -> Dict`
- `calculate_node_field_minimum(field_names) -> Dict`

**Geometry Calculations:**
- `calculate_element_centroids(block_ids)` - Create centroid field

### 6.12 Metadata Methods

**Title & QA Records:**
- `title` - Database title (property)
- `get_title() -> str`
- `set_title(title)`
- `to_lowercase()` - Convert all names to lowercase
- `get_qa_records() -> List[Tuple]`
- `add_qa_record(code_name, version, date, time)`
- `get_info_records() -> List[str]`
- `add_info_record(record)`

### 6.13 Special Features

**Expression Evaluation:**
- `calculate_element_field(name, expression)` - Evaluate expressions for elements
- `calculate_node_field(name, expression)` - Evaluate expressions for nodes
- `calculate_global_variable(name, expression)` - Evaluate expressions globally
- Supports: arithmetic (+, -, *, /, **), math functions (sqrt, sin, cos, exp, log), field variables

**Set Selection:**
- `create_side_set_from_expression(side_set_id, expression)` - Select sides by expression
- `create_node_set_from_expression(node_set_id, expression)` - Select nodes by expression
- `threshold_element_blocks(expression)` - Delete blocks not matching expression

**STL Export:**
- `export_stl_file(filename)` - Export surface mesh to STL format

**VRML Export:**
- `export_wrl_model(filename)` - Export model to VRML

**Element Type Support:**
Built-in definitions for: point, line2, line3, tri3, tri6, quad4, quad8, quad9, tet4, tet10, wedge6, wedge15, hex8, hex20, hex27, pyramid5

### 6.14 Utility & Analysis Methods

**Displacement Field:**
- `displacement_field_exists() -> bool`
- `create_displacement_field(value)`

**Mesh Building:**
- `build_hex8_cube(block_id)` - Create unit cube mesh

**Utility Methods:**
- `summarize()` - Print model summary
- `get_input_deck() -> str` - Generate input deck
- `_calculate_side_area(elem_coords) -> float` - Area calculation helper

---

## 7. Entity Type String Mapping

For map.rs and related functions, entity types are referenced by string:

```
"elem_block"     - Element blocks
"edge_block"     - Edge blocks
"face_block"     - Face blocks
"node_set"       - Node sets
"edge_set"       - Edge sets
"face_set"       - Face sets
"elem_set"       - Element sets
"side_set"       - Side sets
"node_map"       - Node maps
"edge_map"       - Edge maps
"face_map"       - Face maps
"elem_map"       - Element maps
```

For ID maps, types are:
```
"node"  - Node ID map
"elem"  - Element ID map
"edge"  - Edge ID map
"face"  - Face ID map
```

---

## 8. Summary of Key Features

### Supported File Operations
- Full read, write, and append modes
- Context manager support for all file types
- Version and format introspection

### Mesh Definition Capabilities
- Arbitrary element types (topologies)
- Element blocks with attributes
- Node/side/entity sets with distribution factors
- Assemblies and hierarchical grouping
- Blob storage for arbitrary binary data

### Variable Support
- Global variables (scalar)
- Nodal variables (per-node)
- Block variables (per-element)
- Reduction variables (aggregate)
- Sparse variable definitions via truth tables
- Time-dependent variables

### Coordinate Operations
- Full 3D coordinate storage (x, y, z)
- Configurable dimensions (1D, 2D, 3D)
- Coordinate axis naming

### Set Operations
- Node sets with distribution factors
- Side sets (boundary element-side pairs)
- Entity sets (generic edges/faces/elements)
- Set ID mapping
- Automatic nodeset-to-sideset conversion

### Maps & Properties
- ID maps for all entity types
- Element order maps
- Entity naming
- Property arrays for arbitrary metadata

### Metadata & Provenance
- QA records for application tracking
- Info records for arbitrary documentation
- Attribute storage (int/double/string)

### High-Level Features (ExoMerge)
- Comprehensive mesh manipulation
- Geometry transformations (translate, scale, rotate, reflect)
- Block operations (merge, split, convert)
- Field operations (create, convert, calculate)
- Element quality analysis
- Set analysis and manipulation
- Time step management
- STL/VRML export
- Expression-based field calculation

### Performance Features
- HPC environment detection
- Cache configuration
- Chunk size tuning
- Automatic optimization profiles (conservative/aggressive/auto)

---

## 9. File Access Mode Capabilities

| Feature | Read | Write | Append |
|---------|------|-------|--------|
| Open existing file | Yes | No | Yes |
| Create new file | No | Yes | No |
| Read geometry | Yes | No* | Yes* |
| Write geometry | No | Yes | Yes |
| Read variables | Yes | No | Yes* |
| Write variables | No | Yes | Yes |
| Read metadata | Yes | No | Yes* |
| Write metadata | No | Yes | Yes |
| Read sets | Yes | No | Yes* |
| Write sets | No | Yes | Yes |
| Read blocks | Yes | No | Limited |

*Limited functionality in Append mode per implementation notes

---

## 10. Python Module Structure

The module is organized as:
```
exodus (PyO3 extension module)
├── ExodusReader (file operations - read)
├── ExodusWriter (file operations - write)
├── ExodusAppender (file operations - append)
├── MeshBuilder (high-level mesh creation)
├── BlockBuilder (high-level block creation)
├── [Types]
│   ├── EntityType (enum)
│   ├── CreateMode (enum)
│   ├── FloatSize (enum)
│   ├── Int64Mode (enum)
│   ├── AttributeType (enum)
│   ├── InitParams (class)
│   ├── CreateOptions (class)
│   ├── Block (class)
│   ├── NodeSet (class)
│   ├── SideSet (class)
│   ├── EntitySet (class)
│   ├── Assembly (class)
│   ├── Blob (class)
│   ├── QaRecord (class)
│   ├── TruthTable (class)
│   └── AttributeData (class)
├── [Performance]
│   ├── NodeType (class)
│   ├── CacheConfig (class)
│   ├── ChunkConfig (class)
│   └── PerformanceConfig (class)
└── exomerge.py (pure Python high-level API)
    └── ExodusModel (comprehensive mesh manipulation class)
```

---

## Conclusion

The exodus-py implementation provides a comprehensive, modern Python interface to Exodus II files through Rust bindings. It combines:

1. **Low-level control** via ExodusReader/Writer/Appender with fine-grained access
2. **High-level convenience** via MeshBuilder for quick file creation
3. **Comprehensive mesh manipulation** via ExoMerge for complex operations
4. **Performance tuning** via configuration options for HPC environments
5. **Rich type system** supporting all Exodus II entities and data types

The three-tier API (file I/O, builder, exomerge) allows users to work at their preferred level of abstraction while maintaining full control and high performance through Rust's underlying implementation.

