# User Guide

This comprehensive guide covers all aspects of using exodus-py to work with Exodus II files.

## File Operations

### Opening Files

exodus-py provides three classes for file access, each with different capabilities:

#### ExodusReader (Read-Only)

Use `ExodusReader` for read-only access to existing files:

```python
from exodus import ExodusReader

# Using context manager (recommended)
with ExodusReader.open("mesh.exo") as reader:
    params = reader.init_params()
    coords = reader.get_coords()

# Or manual management
reader = ExodusReader.open("mesh.exo")
params = reader.init_params()
reader.close()
```

#### ExodusWriter (Write-Only)

Use `ExodusWriter` for creating new files:

```python
from exodus import ExodusWriter, CreateMode, CreateOptions, InitParams

# Create with default options (clobber mode - overwrites existing files)
writer = ExodusWriter.create("output.exo")

# Or with custom options
options = CreateOptions(mode=CreateMode.NoClobber)  # Fail if file exists
writer = ExodusWriter.create("output.exo", options)

# Initialize the database
params = InitParams(
    title="My Mesh",
    num_dim=3,
    num_nodes=8,
    num_elems=1,
    num_elem_blocks=1,
)
writer.put_init_params(params)

# ... write data ...

writer.close()
```

#### ExodusAppender (Read-Write)

Use `ExodusAppender` for modifying existing files:

```python
from exodus import ExodusAppender

# Open existing file for read-write access
appender = ExodusAppender.append("mesh.exo")

# Can read existing data
params = appender.init_params()

# And write new data (e.g., new time steps)
appender.define_variables(EntityType.Nodal, ["NewVariable"])
appender.put_time(5, 5.0)
appender.put_var(5, EntityType.Nodal, 0, 0, data)

appender.close()
```

### Create Options

Control file creation behavior with `CreateOptions`:

```python
from exodus import CreateOptions, CreateMode, FloatSize, Int64Mode

options = CreateOptions(
    mode=CreateMode.Clobber,        # Overwrite existing file
    float_size=FloatSize.Float64,   # Use 64-bit floats
    int64_mode=Int64Mode.All,       # Use 64-bit integers
    netcdf4=True,                   # Use NetCDF-4 format
    compression_level=1,            # Enable compression (1-9)
)

writer = ExodusWriter.create("mesh.exo", options)
```

**Create Modes:**
- `CreateMode.Clobber`: Overwrite existing file (default)
- `CreateMode.NoClobber`: Fail if file exists

**Float Sizes:**
- `FloatSize.Float32`: 32-bit floating point
- `FloatSize.Float64`: 64-bit floating point (default)

**Int64 Modes:**
- `Int64Mode.None_`: Use 32-bit integers
- `Int64Mode.All`: Use 64-bit integers throughout (default)
- `Int64Mode.Bulk`: 64-bit for bulk data only

## Creating Meshes

### Using MeshBuilder (Recommended)

The builder API provides a fluent interface for creating meshes:

```python
from exodus import MeshBuilder, BlockBuilder

# Simple 2D quadrilateral mesh
(MeshBuilder("2D Mesh")
    .dimensions(2)
    .coordinates(
        x=[0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
        y=[0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        z=[]
    )
    .add_block(
        BlockBuilder(1, "QUAD4")
            .connectivity([1, 2, 5, 4, 2, 3, 6, 5])
            .build()
    )
    .write("mesh.exo"))
```

### Using Low-Level Writer API

For more control, use the writer directly:

```python
from exodus import ExodusWriter, InitParams, Block, EntityType

# Create writer
writer = ExodusWriter.create("mesh.exo")

# Initialize
params = InitParams(
    title="3D Hex Mesh",
    num_dim=3,
    num_nodes=8,
    num_elems=1,
    num_elem_blocks=1,
)
writer.put_init_params(params)

# Write coordinates
x = [0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
y = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0]
z = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
writer.put_coords(x, y, z)

# Define element block
block = Block(
    id=100,
    entity_type=EntityType.ElemBlock,
    topology="HEX8",
    num_entries=1,
    num_nodes_per_entry=8,
    num_attributes=0,
)
writer.put_block(block)

# Write connectivity
writer.put_connectivity(100, [1, 2, 3, 4, 5, 6, 7, 8])

writer.close()
```

## Reading Mesh Data

### Initialization Parameters

Get basic mesh information:

```python
with ExodusReader.open("mesh.exo") as reader:
    params = reader.init_params()

    print(f"Title: {params.title}")
    print(f"Dimensions: {params.num_dim}")
    print(f"Nodes: {params.num_nodes}")
    print(f"Elements: {params.num_elems}")
    print(f"Element Blocks: {params.num_elem_blocks}")
    print(f"Node Sets: {params.num_node_sets}")
    print(f"Side Sets: {params.num_side_sets}")
```

### Coordinates

Read nodal coordinates:

```python
# Get all coordinates
x, y, z = reader.get_coords()

# Get coordinate names if available
coord_names = reader.get_coord_names()  # e.g., ["X", "Y", "Z"]

# Set coordinate names (writer only)
writer.put_coord_names(["X", "Y", "Z"])
```

### Element Blocks

Work with element blocks:

```python
# Get all block IDs
block_ids = reader.get_block_ids()

# Get a specific block
block = reader.get_block(100)
print(f"Topology: {block.topology}")
print(f"Elements: {block.num_entries}")
print(f"Nodes/Element: {block.num_nodes_per_entry}")

# Get connectivity
conn = reader.get_connectivity(100)
# Returns flat list: [n1, n2, ..., nk, n1, n2, ..., nk, ...]
# where k = nodes_per_element

# Get block name
name = reader.get_block_name(100)

# Set block name (writer only)
writer.put_block_name(100, "Steel Elements")
```

**Common Element Topologies:**
- `EDGE2`, `EDGE3`: 2D/3D line elements
- `TRI3`, `TRI6`: Triangles
- `QUAD4`, `QUAD8`, `QUAD9`: Quadrilaterals
- `TET4`, `TET10`: Tetrahedra
- `HEX8`, `HEX20`, `HEX27`: Hexahedra
- `WEDGE6`, `WEDGE15`: Wedges (prisms)
- `PYRAMID5`, `PYRAMID13`: Pyramids

## Variables and Time Steps

### Defining Variables

Variables must be defined before writing time step data:

```python
from exodus import EntityType

# Define nodal variables (e.g., temperature, displacement)
writer.define_variables(EntityType.Nodal, ["Temperature", "Pressure"])

# Define global variables (e.g., total energy)
writer.define_variables(EntityType.Global, ["TotalEnergy", "KineticEnergy"])

# Define element block variables (e.g., stress, strain)
writer.define_variables(EntityType.ElemBlock, ["Stress", "Strain"])

# Define node set variables
writer.define_variables(EntityType.NodeSet, ["BoundaryFlux"])

# Define side set variables
writer.define_variables(EntityType.SideSet, ["WallShear"])
```

### Writing Time Steps

Write results for each time step:

```python
num_nodes = 100
num_steps = 10

for step in range(num_steps):
    time_value = step * 0.1

    # Write time value
    writer.put_time(step, time_value)

    # Write global variables (single value per variable)
    writer.put_var(step, EntityType.Global, 0, 0, [1000.0])  # TotalEnergy
    writer.put_var(step, EntityType.Global, 0, 1, [500.0])   # KineticEnergy

    # Write nodal variables (one value per node)
    temperatures = [300.0 + i * 0.1 for i in range(num_nodes)]
    writer.put_var(step, EntityType.Nodal, 0, 0, temperatures)

    pressures = [100.0 + i * 0.01 for i in range(num_nodes)]
    writer.put_var(step, EntityType.Nodal, 0, 1, pressures)

    # Write element block variables (one value per element)
    num_elems_in_block = 50
    stresses = [100.0 + i for i in range(num_elems_in_block)]
    writer.put_var(step, EntityType.ElemBlock, 100, 0, stresses)
```

**Variable Writing Parameters:**
- `step`: Time step index (0-based)
- `var_type`: Entity type (Global, Nodal, ElemBlock, etc.)
- `entity_id`: Entity ID (0 for global/nodal, block ID for blocks)
- `var_index`: Variable index (0-based, matching definition order)
- `values`: List of values

### Reading Time Steps

Read time-dependent data:

```python
# Get number of time steps
num_steps = reader.num_time_steps()

# Get all time values
times = reader.times()

# Get specific time value
time_5 = reader.time(5)

# Get variable names
nodal_vars = reader.variable_names(EntityType.Nodal)
global_vars = reader.variable_names(EntityType.Global)

# Read variable at a single time step
temp_step_0 = reader.var(0, EntityType.Nodal, 0, 0)

# Read variable time series (multiple steps)
temp_all_steps = reader.var_time_series(
    start_step=0,
    end_step=num_steps,
    var_type=EntityType.Nodal,
    entity_id=0,
    var_index=0
)

# Read all variables for an entity at once
all_nodal_vars = reader.var_multi(0, EntityType.Nodal, 0)
```

### Truth Tables (Sparse Storage)

For element block variables, truth tables control which variables are defined on which blocks:

```python
# Get truth table
truth = reader.truth_table(EntityType.ElemBlock)

# Check if variable is defined on a block
if truth.is_valid(block_id=100, var_index=0):
    data = reader.var(0, EntityType.ElemBlock, 100, 0)

# Set truth table (writer only)
# Create a table: rows = blocks, cols = variables
# True = variable defined on block, False = not defined
writer.put_truth_table(EntityType.ElemBlock, truth_data)
```

## Sets

### Node Sets

Define and use node sets:

```python
# Writing node sets
writer.put_node_set(
    set_id=10,
    nodes=[1, 2, 3, 4, 5],
    dist_factors=None  # Optional distribution factors
)

# With distribution factors
writer.put_node_set(20, [10, 11, 12], [1.0, 0.5, 0.5])

# Set node set name
writer.put_node_set_name(10, "LeftBoundary")

# Reading node sets
node_set_ids = reader.get_node_set_ids()
node_set = reader.get_node_set(10)
print(f"Nodes: {node_set.nodes}")
print(f"Distribution factors: {node_set.dist_factors}")

name = reader.get_node_set_name(10)
```

### Side Sets

Define and use side sets:

```python
# Writing side sets
writer.put_side_set(
    set_id=100,
    elements=[1, 2, 3],      # Element IDs
    sides=[1, 2, 1],         # Side numbers (1-based)
    dist_factors=None
)

# Reading side sets
side_set_ids = reader.get_side_set_ids()
side_set = reader.get_side_set(100)
print(f"Elements: {side_set.elements}")
print(f"Sides: {side_set.sides}")
```

**Side Numbering:**
Side numbering depends on element topology. Generally:
- For quads: 1=bottom, 2=right, 3=top, 4=left
- For hexes: 1-6 for faces

### Edge and Face Blocks

For 3D meshes, you can also define edge and face blocks:

```python
# Edge block
edge_block = Block(
    id=1,
    entity_type=EntityType.EdgeBlock,
    topology="EDGE2",
    num_entries=10,
    num_nodes_per_entry=2,
    num_attributes=0,
)
writer.put_block(edge_block)
writer.put_connectivity(1, [1, 2, 2, 3, ...])

# Face block
face_block = Block(
    id=2,
    entity_type=EntityType.FaceBlock,
    topology="QUAD4",
    num_entries=20,
    num_nodes_per_entry=4,
    num_attributes=0,
)
writer.put_block(face_block)
```

## Entity Sets

Entity sets provide a unified interface for working with different set types:

```python
from exodus import EntitySet, EntityType

# Create an entity set
entity_set = EntitySet(
    id=1,
    entity_type=EntityType.NodeSet,
    entities=[1, 2, 3, 4],
    dist_factors=[1.0, 1.0, 0.5, 0.5]
)

# Write it
writer.put_entity_set(entity_set)

# Read it
entity_set = reader.get_entity_set(EntityType.NodeSet, 1)
```

## Assemblies

Assemblies provide hierarchical organization of entities:

```python
from exodus import Assembly, EntityType

# Create an assembly
assembly = Assembly(
    id=1,
    name="MainStructure",
    assembly_type=EntityType.Assembly,
    entity_list=[100, 101, 102],  # IDs of contained blocks/sets
)

# Write assembly
writer.put_assembly(assembly)

# Read assemblies
assembly_ids = reader.get_assembly_ids()
assembly = reader.get_assembly(1)
print(f"Assembly {assembly.name} contains: {assembly.entity_list}")
```

## Attributes

Element blocks can have attributes (per-element scalar values):

```python
# Writing block attributes with builder
block = (BlockBuilder(1, "HEX8")
    .connectivity([...])
    .attributes([100.0, 200.0, 150.0])  # One per element
    .attribute_names(["MaterialID"])
    .build())

# Reading block attributes
attrs = reader.get_block_attributes(100)
attr_names = reader.get_block_attribute_names(100)

# Writing attributes with low-level API
writer.put_block_attributes(100, [100.0, 200.0, 150.0])
writer.put_block_attribute_names(100, ["MaterialID"])
```

## Metadata

### QA Records

Quality assurance records track software that created/modified the file:

```python
from exodus import QaRecord

# Writing QA records (with builder)
builder.qa_record("MyCode", "1.0.0", "2025-01-15", "14:30:00")

# Writing QA records (with writer)
qa = QaRecord(
    code_name="MyAnalysisCode",
    code_version="2.1.0",
    date="2025-01-15",
    time="14:30:00"
)
writer.put_qa_records([qa])

# Reading QA records
qa_records = reader.get_qa_records()
for qa in qa_records:
    print(f"{qa.code_name} v{qa.code_version} ({qa.date} {qa.time})")
```

### Info Records

Information records are arbitrary text strings:

```python
# Writing
writer.put_info_records([
    "Generated from CAD model v3",
    "Material properties from database",
    "Contact: engineer@example.com"
])

# Reading
info = reader.get_info_records()
for line in info:
    print(line)
```

### File Metadata

Query file format and version information:

```python
# Get Exodus format version
major, minor = reader.version()
print(f"Exodus version: {major}.{minor}")

# Get NetCDF file format
format_str = reader.format()
print(f"File format: {format_str}")

# Get file path
path = reader.path()
```

## Maps

Maps provide alternative numbering schemes for nodes and elements:

```python
# Element number map (local to global ID mapping)
writer.put_elem_num_map([100, 101, 102, ...])
elem_map = reader.get_elem_num_map()

# Node number map
writer.put_node_num_map([1000, 1001, 1002, ...])
node_map = reader.get_node_num_map()

# Element order map (processing order)
writer.put_elem_order_map([0, 1, 2, ...])
elem_order = reader.get_elem_order_map()
```

## Blobs

Blobs store arbitrary binary data:

```python
from exodus import Blob

# Create and write a blob
blob = Blob(
    id=1,
    name="CustomData",
    data=b"Any binary data here..."
)
writer.put_blob(blob)

# Read blobs
blob_ids = reader.get_blob_ids()
blob = reader.get_blob(1)
print(f"Blob {blob.name}: {len(blob.data)} bytes")
```

## Reduction Variables

Reduction variables store aggregate values for entire objects:

```python
# Define reduction variables
writer.define_reduction_variables(
    EntityType.Assembly,
    ["TotalMass", "Momentum_X", "Momentum_Y"]
)

# Write reduction variable values
writer.put_reduction_vars(
    step=0,
    var_type=EntityType.Assembly,
    entity_id=1,
    values=[1000.0, 50.0, 25.0]
)

# Read reduction variables
reduction_vars = reader.reduction_variable_names(EntityType.Assembly)
values = reader.get_reduction_vars(0, EntityType.Assembly, 1)
```

## Performance Optimization

### Chunking and Caching

For large files, configure chunking and caching:

```python
from exodus import PyChunkConfig, PyCacheConfig, PyPerformanceConfig

chunk_config = PyChunkConfig(
    enabled=True,
    chunk_size=1024 * 1024  # 1MB chunks
)

cache_config = PyCacheConfig(
    enabled=True,
    size=100 * 1024 * 1024  # 100MB cache
)

perf_config = PyPerformanceConfig(
    chunk_config=chunk_config,
    cache_config=cache_config,
    parallel_io=True
)

# Apply configuration when creating file
# (API details may vary - check current implementation)
```

## Best Practices

### Error Handling

Always handle potential errors:

```python
try:
    reader = ExodusReader.open("mesh.exo")
    params = reader.init_params()
    reader.close()
except FileNotFoundError:
    print("File not found")
except RuntimeError as e:
    print(f"Exodus error: {e}")
```

### Resource Management

Use context managers to ensure files are properly closed:

```python
# Good - file automatically closed
with ExodusReader.open("mesh.exo") as reader:
    data = reader.get_coords()
    # File closed automatically when exiting 'with' block

# Avoid - must remember to close
reader = ExodusReader.open("mesh.exo")
data = reader.get_coords()
reader.close()  # Easy to forget!
```

### Memory Efficiency

For large files:

```python
# Read one variable at a time instead of all at once
for var_idx in range(num_vars):
    data = reader.var(step, EntityType.Nodal, 0, var_idx)
    process(data)  # Process immediately
    # Data can be garbage collected

# Instead of:
all_data = []
for var_idx in range(num_vars):
    all_data.append(reader.var(step, EntityType.Nodal, 0, var_idx))
    # All data kept in memory!
```

### Validation

Validate data before writing:

```python
# Check coordinate lengths match num_nodes
assert len(x) == params.num_nodes
assert len(y) == params.num_nodes
assert len(z) == params.num_nodes or len(z) == 0

# Check connectivity length
expected_len = num_elems * nodes_per_elem
assert len(connectivity) == expected_len
```

## Troubleshooting

### Common Issues

**"File already closed" error:**
- Make sure you haven't already called `close()` on the writer
- Use `with` statement to avoid this issue

**"AlreadyExists" error:**
- File exists and you're using `CreateMode.NoClobber`
- Use `CreateMode.Clobber` or delete the existing file

**Variable data wrong size:**
- Check that array length matches expected size:
  - Global: 1 value
  - Nodal: num_nodes values
  - Element block: num_elements_in_block values

**"Variable not defined" error:**
- Call `define_variables()` before `put_var()`
- Ensure variable index matches definition order

**Cannot import exodus:**
- Make sure package is installed: `pip install exodus-py`
- Or in development mode: `maturin develop`
