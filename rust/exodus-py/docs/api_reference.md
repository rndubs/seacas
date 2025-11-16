# API Reference

Complete reference for all exodus-py classes, methods, and types.

## File Classes

### ExodusReader

Read-only access to Exodus files.

#### Methods

##### `ExodusReader.open(path: str) -> ExodusReader`

Open an existing Exodus file for reading.

**Parameters:**
- `path` (str): Path to the Exodus file

**Returns:**
- ExodusReader instance

**Example:**
```python
reader = ExodusReader.open("mesh.exo")
# Or use with context manager
with ExodusReader.open("mesh.exo") as reader:
    params = reader.init_params()
```

##### `init_params() -> InitParams`

Get initialization parameters from the file.

**Returns:**
- InitParams containing database dimensions and counts

##### `path() -> str`

Get the file path.

**Returns:**
- Path to the file as a string

##### `version() -> tuple[int, int]`

Get Exodus file format version.

**Returns:**
- Tuple of (major_version, minor_version) as integers

##### `format() -> str`

Get NetCDF file format information.

**Returns:**
- File format as string (e.g., "NetCDF4", "NetCDF3")

##### `close()`

Close the file. Files are automatically closed when the reader is destroyed or when exiting a context manager.

#### Coordinate Methods

##### `get_coords() -> tuple[list[float], list[float], list[float]]`

Get all nodal coordinates.

**Returns:**
- Tuple of (x, y, z) coordinate lists

##### `get_coord_names() -> list[str]`

Get coordinate axis names.

**Returns:**
- List of coordinate names (e.g., ["X", "Y", "Z"])

#### Block Methods

##### `get_block_ids() -> list[int]`

Get all element block IDs.

**Returns:**
- List of block IDs

##### `get_block(block_id: int) -> Block`

Get element block information.

**Parameters:**
- `block_id` (int): Block ID

**Returns:**
- Block object containing block metadata

##### `get_connectivity(block_id: int) -> list[int]`

Get element connectivity for a block.

**Parameters:**
- `block_id` (int): Block ID

**Returns:**
- Flat list of node IDs (1-based indexing)
- Length = num_elements × nodes_per_element

##### `get_block_name(block_id: int) -> str`

Get block name.

**Parameters:**
- `block_id` (int): Block ID

**Returns:**
- Block name string

##### `get_block_attributes(block_id: int) -> list[float]`

Get element attributes for a block.

**Parameters:**
- `block_id` (int): Block ID

**Returns:**
- Flat list of attribute values

##### `get_block_attribute_names(block_id: int) -> list[str]`

Get attribute names for a block.

**Parameters:**
- `block_id` (int): Block ID

**Returns:**
- List of attribute names

#### Set Methods

##### `get_node_set_ids() -> list[int]`

Get all node set IDs.

**Returns:**
- List of node set IDs

##### `get_node_set(set_id: int) -> NodeSet`

Get node set data.

**Parameters:**
- `set_id` (int): Node set ID

**Returns:**
- NodeSet object

##### `get_node_set_name(set_id: int) -> str`

Get node set name.

**Parameters:**
- `set_id` (int): Node set ID

**Returns:**
- Node set name

##### `get_side_set_ids() -> list[int]`

Get all side set IDs.

**Returns:**
- List of side set IDs

##### `get_side_set(set_id: int) -> SideSet`

Get side set data.

**Parameters:**
- `set_id` (int): Side set ID

**Returns:**
- SideSet object

##### `get_side_set_name(set_id: int) -> str`

Get side set name.

**Parameters:**
- `set_id` (int): Side set ID

**Returns:**
- Side set name

##### `get_entity_set(var_type: EntityType, set_id: int) -> EntitySet`

Get generic entity set.

**Parameters:**
- `var_type` (EntityType): Type of set
- `set_id` (int): Set ID

**Returns:**
- EntitySet object

#### Variable Methods

##### `variable_names(var_type: EntityType) -> list[str]`

Get variable names for an entity type.

**Parameters:**
- `var_type` (EntityType): Entity type (e.g., EntityType.Nodal, EntityType.Global)

**Returns:**
- List of variable names

##### `num_time_steps() -> int`

Get number of time steps in the file.

**Returns:**
- Number of time steps

##### `times() -> list[float]`

Get all time values.

**Returns:**
- List of time values for all steps

##### `time(step: int) -> float`

Get time value for a specific step.

**Parameters:**
- `step` (int): Time step index (0-based)

**Returns:**
- Time value

##### `var(step: int, var_type: EntityType, entity_id: int, var_index: int) -> list[float]`

Read variable values at a time step.

**Parameters:**
- `step` (int): Time step index (0-based)
- `var_type` (EntityType): Entity type
- `entity_id` (int): Entity ID (block ID for block variables, 0 for global/nodal)
- `var_index` (int): Variable index (0-based)

**Returns:**
- List of variable values

##### `var_multi(step: int, var_type: EntityType, entity_id: int) -> list[float]`

Read all variables for an entity at a time step.

**Parameters:**
- `step` (int): Time step index (0-based)
- `var_type` (EntityType): Entity type
- `entity_id` (int): Entity ID

**Returns:**
- Flat list of all variable values

##### `var_time_series(start_step: int, end_step: int, var_type: EntityType, entity_id: int, var_index: int) -> list[float]`

Read variable time series.

**Parameters:**
- `start_step` (int): Starting time step index (0-based, inclusive)
- `end_step` (int): Ending time step index (exclusive)
- `var_type` (EntityType): Entity type
- `entity_id` (int): Entity ID
- `var_index` (int): Variable index (0-based)

**Returns:**
- Variable values for all requested time steps

##### `truth_table(var_type: EntityType) -> TruthTable`

Get truth table for sparse variable storage.

**Parameters:**
- `var_type` (EntityType): Entity type (must be a block type)

**Returns:**
- TruthTable object

##### `reduction_variable_names(var_type: EntityType) -> list[str]`

Get reduction variable names for an entity type.

**Parameters:**
- `var_type` (EntityType): Entity type (e.g., EntityType.Assembly)

**Returns:**
- List of reduction variable names

##### `get_reduction_vars(step: int, var_type: EntityType, entity_id: int) -> list[float]`

Read reduction variable values for a time step.

**Parameters:**
- `step` (int): Time step index (0-based)
- `var_type` (EntityType): Entity type
- `entity_id` (int): Entity ID

**Returns:**
- List of reduction variable values

#### Assembly Methods

##### `get_assembly_ids() -> list[int]`

Get all assembly IDs.

**Returns:**
- List of assembly IDs

##### `get_assembly(assembly_id: int) -> Assembly`

Get assembly data.

**Parameters:**
- `assembly_id` (int): Assembly ID

**Returns:**
- Assembly object

#### Map Methods

##### `get_elem_num_map() -> list[int]`

Get element number map.

**Returns:**
- List mapping local to global element IDs

##### `get_node_num_map() -> list[int]`

Get node number map.

**Returns:**
- List mapping local to global node IDs

##### `get_elem_order_map() -> list[int]`

Get element order map.

**Returns:**
- List specifying element processing order

#### Metadata Methods

##### `get_qa_records() -> list[QaRecord]`

Get quality assurance records.

**Returns:**
- List of QaRecord objects

##### `get_info_records() -> list[str]`

Get information records.

**Returns:**
- List of information strings

#### Blob Methods

##### `get_blob_ids() -> list[int]`

Get all blob IDs.

**Returns:**
- List of blob IDs

##### `get_blob(blob_id: int) -> Blob`

Get blob data.

**Parameters:**
- `blob_id` (int): Blob ID

**Returns:**
- Blob object

---

### ExodusWriter

Write-only access for creating new Exodus files.

#### Methods

##### `ExodusWriter.create(path: str, options: CreateOptions | None = None) -> ExodusWriter`

Create a new Exodus file for writing.

**Parameters:**
- `path` (str): Path where the file should be created
- `options` (CreateOptions, optional): File creation options

**Returns:**
- ExodusWriter instance

**Example:**
```python
import tempfile
import os

# Create unique temporary file paths
fd1, temp_path1 = tempfile.mkstemp(suffix=".exo")
os.close(fd1)
os.unlink(temp_path1)  # Remove so ExodusWriter can create it

fd2, temp_path2 = tempfile.mkstemp(suffix=".exo")
os.close(fd2)
os.unlink(temp_path2)  # Remove so ExodusWriter can create it

writer = ExodusWriter.create(temp_path1, CreateOptions(mode=CreateMode.Clobber))
writer.close()

# Or with options:
opts = CreateOptions(mode=CreateMode.Clobber)
writer = ExodusWriter.create(temp_path2, opts)
writer.close()

# Cleanup
os.unlink(temp_path1)
os.unlink(temp_path2)
```

##### `put_init_params(params: InitParams)`

Initialize the database with parameters. Must be called before writing data.

**Parameters:**
- `params` (InitParams): Database initialization parameters

##### `close()`

Close the file and flush all data to disk.

#### Coordinate Methods

##### `put_coords(x: list[float], y: list[float], z: list[float])`

Write nodal coordinates.

**Parameters:**
- `x` (list[float]): X coordinates
- `y` (list[float]): Y coordinates
- `z` (list[float]): Z coordinates (can be empty for 2D)

##### `put_coord_names(names: list[str])`

Set coordinate axis names.

**Parameters:**
- `names` (list[str]): Coordinate names (e.g., ["X", "Y", "Z"])

#### Block Methods

##### `put_block(block: Block)`

Define an element/edge/face block.

**Parameters:**
- `block` (Block): Block definition

##### `put_connectivity(block_id: int, connectivity: list[int])`

Write element connectivity for a block.

**Parameters:**
- `block_id` (int): Block ID
- `connectivity` (list[int]): Flat list of node IDs (1-based)

##### `put_block_name(block_id: int, name: str)`

Set block name.

**Parameters:**
- `block_id` (int): Block ID
- `name` (str): Block name

##### `put_block_attributes(block_id: int, attributes: list[float])`

Write element attributes for a block.

**Parameters:**
- `block_id` (int): Block ID
- `attributes` (list[float]): Flat list of attribute values

##### `put_block_attribute_names(block_id: int, names: list[str])`

Set attribute names for a block.

**Parameters:**
- `block_id` (int): Block ID
- `names` (list[str]): Attribute names

#### Set Methods

##### `put_node_set(set_id: int, nodes: list[int], dist_factors: list[float] | None)`

Write a node set.

**Parameters:**
- `set_id` (int): Node set ID
- `nodes` (list[int]): Node IDs (1-based)
- `dist_factors` (list[float] | None): Distribution factors (optional)

##### `put_node_set_name(set_id: int, name: str)`

Set node set name.

**Parameters:**
- `set_id` (int): Node set ID
- `name` (str): Node set name

##### `put_side_set(set_id: int, elements: list[int], sides: list[int], dist_factors: list[float] | None)`

Write a side set.

**Parameters:**
- `set_id` (int): Side set ID
- `elements` (list[int]): Element IDs
- `sides` (list[int]): Side numbers (1-based, topology-dependent)
- `dist_factors` (list[float] | None): Distribution factors (optional)

##### `put_side_set_name(set_id: int, name: str)`

Set side set name.

**Parameters:**
- `set_id` (int): Side set ID
- `name` (str): Side set name

##### `put_entity_set(entity_set: EntitySet)`

Write a generic entity set.

**Parameters:**
- `entity_set` (EntitySet): Entity set object

#### Variable Methods

##### `define_variables(var_type: EntityType, names: list[str])`

Define variables for an entity type.

**Parameters:**
- `var_type` (EntityType): Entity type
- `names` (list[str]): Variable names

##### `put_time(step: int, time_value: float)`

Write time value for a time step.

**Parameters:**
- `step` (int): Time step index (0-based)
- `time_value` (float): Time value

##### `put_var(step: int, var_type: EntityType, entity_id: int, var_index: int, values: list[float])`

Write variable values for a time step.

**Parameters:**
- `step` (int): Time step index (0-based)
- `var_type` (EntityType): Entity type
- `entity_id` (int): Entity ID (0 for global/nodal, block ID for blocks)
- `var_index` (int): Variable index (0-based)
- `values` (list[float]): Variable values

##### `put_truth_table(var_type: EntityType, table: list[list[bool]])`

Set truth table for sparse variable storage.

**Parameters:**
- `var_type` (EntityType): Entity type
- `table` (list[list[bool]]): 2D table (rows=entities, cols=variables)

##### `define_reduction_variables(var_type: EntityType, names: list[str])`

Define reduction variables for an entity type.

**Parameters:**
- `var_type` (EntityType): Entity type
- `names` (list[str]): Reduction variable names

##### `put_reduction_vars(step: int, var_type: EntityType, entity_id: int, values: list[float])`

Write reduction variable values for a time step.

**Parameters:**
- `step` (int): Time step index (0-based)
- `var_type` (EntityType): Entity type
- `entity_id` (int): Entity ID
- `values` (list[float]): Reduction variable values

#### Assembly Methods

##### `put_assembly(assembly: Assembly)`

Write an assembly.

**Parameters:**
- `assembly` (Assembly): Assembly object

#### Map Methods

##### `put_elem_num_map(map: list[int])`

Write element number map.

**Parameters:**
- `map` (list[int]): Element number map

##### `put_node_num_map(map: list[int])`

Write node number map.

**Parameters:**
- `map` (list[int]): Node number map

##### `put_elem_order_map(map: list[int])`

Write element order map.

**Parameters:**
- `map` (list[int]): Element order map

#### Metadata Methods

##### `put_qa_records(records: list[QaRecord])`

Write quality assurance records.

**Parameters:**
- `records` (list[QaRecord]): QA records

##### `put_info_records(records: list[str])`

Write information records.

**Parameters:**
- `records` (list[str]): Information strings

#### Blob Methods

##### `put_blob(blob: Blob)`

Write a blob.

**Parameters:**
- `blob` (Blob): Blob object

---

### ExodusAppender

Read-write access to existing Exodus files.

Combines all methods from ExodusReader and ExodusWriter. Use this when you need to both read from and write to an existing file.

#### Methods

##### `ExodusAppender.append(path: str) -> ExodusAppender`

Open an existing file for read-write access.

**Parameters:**
- `path` (str): Path to the Exodus file

**Returns:**
- ExodusAppender instance

**Example:**
```python
# Note: Uses mesh.exo created by test fixtures
with ExodusAppender.append("mesh.exo") as appender:
    # Read existing data
    params = appender.init_params()
    num_nodes = params.num_nodes
    num_dim = params.num_dim

    # Get coordinates
    x, y, z = appender.get_coords()
    z_val = z[0] if z else 0.0
    first_node = (x[0], y[0], z_val)
```

---

## Builder Classes

### MeshBuilder

High-level fluent API for creating meshes.

#### Methods

##### `MeshBuilder(title: str) -> MeshBuilder`

Create a new mesh builder.

**Parameters:**
- `title` (str): Mesh title (max 80 characters)

**Example:**
```python
builder = MeshBuilder("My Mesh")
```

##### `dimensions(num_dim: int) -> MeshBuilder`

Set the number of spatial dimensions.

**Parameters:**
- `num_dim` (int): Number of dimensions (1, 2, or 3)

**Returns:**
- Self for method chaining

##### `coordinates(x: list[float], y: list[float], z: list[float]) -> MeshBuilder`

Set nodal coordinates.

**Parameters:**
- `x` (list[float]): X coordinates
- `y` (list[float]): Y coordinates
- `z` (list[float]): Z coordinates (can be empty for 2D)

**Returns:**
- Self for method chaining

##### `add_block(block: BlockBuilder) -> MeshBuilder`

Add an element block.

**Parameters:**
- `block` (BlockBuilder): Block builder

**Returns:**
- Self for method chaining

##### `qa_record(code_name: str, version: str, date: str, time: str) -> MeshBuilder`

Add a QA record.

**Parameters:**
- `code_name` (str): Software name
- `version` (str): Version string
- `date` (str): Date string
- `time` (str): Time string

**Returns:**
- Self for method chaining

##### `info(info_text: str) -> MeshBuilder`

Add an info record.

**Parameters:**
- `info_text` (str): Information text

**Returns:**
- Self for method chaining

##### `write(path: str)`

Write the mesh to a file.

**Parameters:**
- `path` (str): Output file path

**Example:**
```python
import tempfile
import os

# Create a unique temporary path
fd, temp_path = tempfile.mkstemp(suffix=".exo")
os.close(fd)
os.unlink(temp_path)

builder = MeshBuilder("Mesh")
builder.dimensions(2)
builder.coordinates(x=[0.0, 1.0, 1.0, 0.0], y=[0.0, 0.0, 1.0, 1.0])

block = BlockBuilder(1, "QUAD4")
block.connectivity([1, 2, 3, 4])
builder.add_block(block.build())
builder.write(temp_path)

# Cleanup
os.unlink(temp_path)
```

---

### BlockBuilder

Fluent API for building element blocks.

#### Methods

##### `BlockBuilder(id: int, topology: str) -> BlockBuilder`

Create a new block builder.

**Parameters:**
- `id` (int): Block ID (must be unique)
- `topology` (str): Element topology (e.g., "HEX8", "QUAD4", "TET4")

**Example:**
```python
builder = BlockBuilder(100, "HEX8")
```

##### `connectivity(conn: list[int]) -> BlockBuilder`

Set element connectivity.

**Parameters:**
- `conn` (list[int]): Flat array of node IDs (1-based)

**Returns:**
- Self for method chaining

##### `attributes(attrs: list[float]) -> BlockBuilder`

Set element attributes.

**Parameters:**
- `attrs` (list[float]): Flat array of attribute values

**Returns:**
- Self for method chaining

##### `attribute_names(names: list[str]) -> BlockBuilder`

Set attribute names.

**Parameters:**
- `names` (list[str]): Attribute names

**Returns:**
- Self for method chaining

##### `build() -> BlockBuilder`

Build and return the block builder (no-op for API compatibility).

**Returns:**
- Self

**Example:**
```python
block = BlockBuilder(1, "HEX8")
block.connectivity([1, 2, 3, 4, 5, 6, 7, 8])
block.attributes([100.0])
block.attribute_names(["MaterialID"])
block.build()
```

---

## Data Types

### InitParams

Database initialization parameters.

#### Attributes

- `title` (str): Database title
- `num_dim` (int): Number of spatial dimensions (1, 2, or 3)
- `num_nodes` (int): Number of nodes
- `num_elems` (int): Number of elements
- `num_elem_blocks` (int): Number of element blocks
- `num_node_sets` (int): Number of node sets (default: 0)
- `num_side_sets` (int): Number of side sets (default: 0)
- `num_edge_blocks` (int): Number of edge blocks (default: 0)
- `num_face_blocks` (int): Number of face blocks (default: 0)
- `num_elem_sets` (int): Number of element sets (default: 0)
- `num_edge_sets` (int): Number of edge sets (default: 0)
- `num_face_sets` (int): Number of face sets (default: 0)
- `num_node_maps` (int): Number of node maps (default: 0)
- `num_elem_maps` (int): Number of element maps (default: 0)
- `num_edge_maps` (int): Number of edge maps (default: 0)
- `num_face_maps` (int): Number of face maps (default: 0)
- `num_assemblies` (int): Number of assemblies (default: 0)
- `num_blobs` (int): Number of blobs (default: 0)

#### Example

```python
params = InitParams(
    title="My Mesh",
    num_dim=3,
    num_nodes=100,
    num_elems=50,
    num_elem_blocks=2,
    num_node_sets=3,
)
```

---

### CreateOptions

Options for file creation.

#### Attributes

- `mode` (CreateMode): Creation mode (default: CreateMode.Clobber)
- `float_size` (FloatSize): Float precision (default: FloatSize.Float64)
- `int64_mode` (Int64Mode): Integer size (default: Int64Mode.Int64)
- `netcdf4` (bool): Use NetCDF-4 format (default: True)
- `compression_level` (int): Compression level 0-9 (default: 0, no compression)

#### Example

```python
from exodus import CreateOptions, CreateMode, FloatSize

options = CreateOptions(
    mode=CreateMode.NoClobber,
    float_size=FloatSize.Float64,
)
```

---

### Block

Element/edge/face block definition.

#### Attributes

- `id` (int): Block ID
- `entity_type` (EntityType): Block type (ElemBlock, EdgeBlock, FaceBlock)
- `topology` (str): Element topology string
- `num_entries` (int): Number of elements/edges/faces
- `num_nodes_per_entry` (int): Nodes per element
- `num_attributes` (int): Attributes per element (default: 0)

#### Example

```python
block = Block(
    id=100,
    entity_type=EntityType.ElemBlock,
    topology="HEX8",
    num_entries=50,
    num_nodes_per_entry=8,
    num_attributes=1,
)
```

---

### NodeSet

Node set with optional distribution factors.

#### Attributes

- `nodes` (list[int]): Node IDs (1-based)
- `dist_factors` (list[float] | None): Distribution factors (optional)

---

### SideSet

Side set definition.

#### Attributes

- `elements` (list[int]): Element IDs
- `sides` (list[int]): Side numbers (1-based, topology-dependent)
- `dist_factors` (list[float] | None): Distribution factors (optional)

---

### EntitySet

Generic entity set.

#### Attributes

- `id` (int): Set ID
- `entity_type` (EntityType): Set type
- `entities` (list[int]): Entity IDs
- `dist_factors` (list[float] | None): Distribution factors (optional)

---

### Assembly

Hierarchical grouping of entities.

#### Attributes

- `id` (int): Assembly ID
- `name` (str): Assembly name
- `assembly_type` (EntityType): Type (usually EntityType.Assembly)
- `entity_list` (list[int]): IDs of contained entities

#### Example

```python
from exodus import Assembly, EntityType

assembly = Assembly(
    id=1,
    name="Structure",
    entity_type=EntityType.Assembly,
    entity_list=[100, 101, 102],
)
```

---

### QaRecord

Quality assurance record.

#### Attributes

- `code_name` (str): Software name
- `code_version` (str): Version string
- `date` (str): Date string
- `time` (str): Time string

#### Example

```python
qa = QaRecord(
    code_name="MyCode",
    code_version="1.0.0",
    date="2025-01-15",
    time="14:30:00",
)
```

---

### Blob

Arbitrary binary data storage.

#### Attributes

- `id` (int): Blob ID
- `name` (str): Blob name
- `data` (bytes): Binary data

---

### TruthTable

Sparse variable storage control (which variables exist on which entities).

#### Methods

##### `is_valid(block_id: int, var_index: int) -> bool`

Check if a variable is defined on an entity.

**Parameters:**
- `block_id` (int): Entity/block ID
- `var_index` (int): Variable index (0-based)

**Returns:**
- True if variable is defined, False otherwise

---

### AttributeData

Attribute data for blocks.

#### Attributes

- `values` (list[float]): Attribute values
- `names` (list[str]): Attribute names

---

## Enumerations

### EntityType

Entity types in Exodus II.

**Values:**
- `EntityType.ElemBlock`: Element block
- `EntityType.NodeSet`: Node set
- `EntityType.SideSet`: Side set
- `EntityType.EdgeBlock`: Edge block
- `EntityType.EdgeSet`: Edge set
- `EntityType.FaceBlock`: Face block
- `EntityType.FaceSet`: Face set
- `EntityType.ElemSet`: Element set
- `EntityType.NodeMap`: Node map
- `EntityType.ElemMap`: Element map
- `EntityType.EdgeMap`: Edge map
- `EntityType.FaceMap`: Face map
- `EntityType.Global`: Global variables
- `EntityType.Nodal`: Nodal variables
- `EntityType.Assembly`: Assembly
- `EntityType.Blob`: Blob

---

### CreateMode

File creation mode.

**Values:**
- `CreateMode.Clobber`: Overwrite existing file
- `CreateMode.NoClobber`: Fail if file exists

---

### FloatSize

Floating point precision.

**Values:**
- `FloatSize.Float32`: 32-bit floats
- `FloatSize.Float64`: 64-bit floats (default)

---

### Int64Mode

Integer storage mode.

**Values:**
- `Int64Mode.Int32`: 32-bit integers
- `Int64Mode.Int64`: 64-bit integers (default)

---

### AttributeType

Attribute value type.

**Values:**
- `AttributeType.Integer`: Integer attribute
- `AttributeType.Double`: Double precision float
- `AttributeType.Char`: Character/string

---

## Performance Types

### PyPerformanceConfig

Performance configuration for large files.

#### Attributes

- `chunk_config` (PyChunkConfig): Chunking configuration
- `cache_config` (PyCacheConfig): Caching configuration
- `parallel_io` (bool): Enable parallel I/O

---

### PyChunkConfig

Chunking configuration.

#### Attributes

- `enabled` (bool): Enable chunking
- `chunk_size` (int): Chunk size in bytes

---

### PyCacheConfig

Cache configuration.

#### Attributes

- `enabled` (bool): Enable caching
- `size` (int): Cache size in bytes

---

### PyNodeType

Node type for distributed computing (advanced feature).

**Values:**
- `PyNodeType.Processor`: Processor node
- `PyNodeType.IO`: I/O node

---

## Module Information

### Version

```python
import exodus
print(exodus.__version__)  # e.g., "0.1.0"
```

### Module Docstring

```python
import exodus
print(exodus.__doc__)
# "Python bindings for exodus-rs - Pure Rust Exodus II implementation"
```
