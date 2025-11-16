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
writer = ExodusWriter.create("/tmp/ug_writer_example.exo", CreateOptions(mode=CreateMode.Clobber))

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

# Or with custom options (using different filename to avoid conflict)
import os
if os.path.exists("/tmp/ug_writer_example2.exo"):
    os.remove("/tmp/ug_writer_example2.exo")
options = CreateOptions(mode=CreateMode.NoClobber)  # Fail if file exists
writer2 = ExodusWriter.create("/tmp/ug_writer_example2.exo", options)
writer2.put_init_params(params)
writer2.close()
```

#### ExodusAppender (Read-Write)

Use `ExodusAppender` for modifying existing files:

```python
from exodus import ExodusAppender

# Open existing file for read-write access
with ExodusAppender.append("mesh.exo") as appender:
    # Can read existing data
    params = appender.init_params()

    # Appender provides read-only access to the file
    # For writing new data, use ExodusWriter to create a new file
    # or reopen with appropriate write mode
```

### Create Options

Control file creation behavior with `CreateOptions`:

```python
from exodus import CreateOptions, CreateMode, FloatSize, Int64Mode, ExodusWriter, InitParams

options = CreateOptions(
    mode=CreateMode.Clobber,        # Overwrite existing file
    float_size=FloatSize.Float64,   # Use 64-bit floats
    int64_mode=Int64Mode.Int64,     # Use 64-bit integers
)

with ExodusWriter.create("create_options_example.exo", options) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=8, num_elems=1, num_elem_blocks=1)
    writer.put_init_params(params)
```

**Create Modes:**
- `CreateMode.Clobber`: Overwrite existing file (default)
- `CreateMode.NoClobber`: Fail if file exists

**Float Sizes:**
- `FloatSize.Float32`: 32-bit floating point
- `FloatSize.Float64`: 64-bit floating point (default)

**Int64 Modes:**
- `Int64Mode.Int32`: Use 32-bit integers
- `Int64Mode.Int64`: Use 64-bit integers (default)

## Creating Meshes

### Using MeshBuilder (Recommended)

The builder API provides a fluent interface for creating meshes:

```python
from exodus import MeshBuilder, BlockBuilder

# Simple 2D quadrilateral mesh
builder = MeshBuilder("2D Mesh")
builder.dimensions(2)
builder.coordinates(
    x=[0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
    y=[0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    z=[]
)

block = BlockBuilder(1, "QUAD4")
block.connectivity([1, 2, 5, 4, 2, 3, 6, 5])
builder.add_block(block.build())
builder.write("builder_mesh.exo")
```

### Using Low-Level Writer API

For more control, use the writer directly:

```python
from exodus import ExodusWriter, InitParams, Block, EntityType

# Create writer
with ExodusWriter.create("/tmp/ug_lowlevel_mesh.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
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
```

## Reading Mesh Data

### Initialization Parameters

Get basic mesh information:

```python
with ExodusReader.open("mesh.exo") as reader:
    params = reader.init_params()

    title = params.title
    num_dim = params.num_dim
    num_nodes = params.num_nodes
    num_elems = params.num_elems
    num_elem_blocks = params.num_elem_blocks
    num_node_sets = params.num_node_sets
    num_side_sets = params.num_side_sets
```

### Coordinates

Read nodal coordinates:

```python
from exodus import ExodusReader, ExodusWriter, InitParams, MeshBuilder, BlockBuilder

# First create a sample mesh
builder = MeshBuilder("Sample Mesh")
builder.dimensions(3)
builder.coordinates(
    x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
    y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
    z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
)

block = BlockBuilder(1, "HEX8")
block.connectivity([1, 2, 3, 4, 5, 6, 7, 8])
builder.add_block(block.build())
builder.write("/tmp/ug_coords_mesh.exo")

with ExodusReader.open("/tmp/ug_coords_mesh.exo") as reader:
    # Get all coordinates
    x, y, z = reader.get_coords()

    # Get coordinate names if available
    coord_names = reader.get_coord_names()  # e.g., ["X", "Y", "Z"]

# Set coordinate names (writer only)
with ExodusWriter.create("/tmp/ug_coord_names_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=8, num_elems=1, num_elem_blocks=1)
    writer.put_init_params(params)
    writer.put_coord_names(["X", "Y", "Z"])
```

### Element Blocks

Work with element blocks:

```python
from exodus import ExodusReader, ExodusWriter, InitParams, Block, EntityType, MeshBuilder, BlockBuilder

# First create a sample mesh to read from
builder = MeshBuilder("Block Example Mesh")
builder.dimensions(3)
builder.coordinates(
    x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
    y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
    z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
)

block = BlockBuilder(1, "HEX8")
block.connectivity([1, 2, 3, 4, 5, 6, 7, 8])
builder.add_block(block.build())
builder.write("/tmp/ug_blocks_mesh.exo")

with ExodusReader.open("/tmp/ug_blocks_mesh.exo") as reader:
    # Get all block IDs
    block_ids = reader.get_block_ids()

    # Get a specific block
    block = reader.get_block(block_ids[0])
    topology = block.topology
    num_entries = block.num_entries
    nodes_per_entry = block.num_nodes_per_entry

    # Get connectivity
    conn = reader.get_connectivity(block_ids[0])
    # Returns flat list: [n1, n2, ..., nk, n1, n2, ..., nk, ...]
    # where k = nodes_per_element

# Block names can be set when writing
with ExodusWriter.create("/tmp/ug_block_name_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=8, num_elems=1, num_elem_blocks=1)
    writer.put_init_params(params)
    block = Block(
        id=100,
        entity_type=EntityType.ElemBlock,
        topology="HEX8",
        num_entries=1,
        num_nodes_per_entry=8,
        num_attributes=0,
    )
    writer.put_block(block)
    # Block names may be set through block properties or other methods
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
from exodus import EntityType, ExodusWriter, InitParams

# Create a writer for demonstration
with ExodusWriter.create("/tmp/ug_define_vars_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=8, num_elems=1, num_elem_blocks=1)
    writer.put_init_params(params)

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

with ExodusWriter.create("/tmp/ug_timesteps_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=num_nodes, num_elems=50, num_elem_blocks=1)
    writer.put_init_params(params)

    # Define block
    block = Block(
        id=100,
        entity_type=EntityType.ElemBlock,
        topology="HEX8",
        num_entries=50,
        num_nodes_per_entry=8,
        num_attributes=0,
    )
    writer.put_block(block)

    # Define variables
    writer.define_variables(EntityType.Global, ["TotalEnergy", "KineticEnergy"])
    writer.define_variables(EntityType.Nodal, ["Temperature", "Pressure"])
    writer.define_variables(EntityType.ElemBlock, ["Stress"])

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
# Open a file with time series data
with ExodusReader.open("mesh.exo") as reader:
    # Get number of time steps
    num_steps = reader.num_time_steps()

    # Get all time values
    times = reader.times()

    # Get specific time value (use valid index)
    if num_steps > 0:
        time_0 = reader.time(0)

    # Get variable names
    nodal_vars = reader.variable_names(EntityType.Nodal)
    global_vars = reader.variable_names(EntityType.Global)

    # Read variable at a single time step
    if len(nodal_vars) > 0 and num_steps > 0:
        temp_step_0 = reader.var(0, EntityType.Nodal, 0, 0)

    # Read variable time series (multiple steps)
    if len(nodal_vars) > 0 and num_steps > 0:
        temp_all_steps = reader.var_time_series(
            start_step=0,
            end_step=num_steps,
            var_type=EntityType.Nodal,
            entity_id=0,
            var_index=0
        )

    # Read all variables for an entity at once
    if len(nodal_vars) > 0 and num_steps > 0:
        all_nodal_vars = reader.var_multi(0, EntityType.Nodal, 0)
```

### Truth Tables (Sparse Storage)

For element block variables, truth tables control which variables are defined on which blocks:

```python
# Example showing truth table usage (conceptual - truth tables have specific API)
# Truth tables are automatically managed when you define variables
# The API may use different methods depending on implementation

# Reading: Check which variables exist for which blocks
with ExodusReader.open("mesh.exo") as reader:
    block_ids = reader.get_block_ids()
    elem_vars = reader.variable_names(EntityType.ElemBlock)

    # Try to read variable if it exists
    if len(elem_vars) > 0 and len(block_ids) > 0:
        try:
            data = reader.var(0, EntityType.ElemBlock, block_ids[0], 0)
        except:
            pass  # Variable not defined for this block
```

## Sets

### Node Sets

Define and use node sets:

```python
from exodus import ExodusWriter, ExodusReader, InitParams

# Writing node sets
with ExodusWriter.create("/tmp/ug_nodesets_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=20, num_elems=1, num_elem_blocks=1, num_node_sets=2)
    writer.put_init_params(params)

    # Put node sets
    writer.put_node_set(
        set_id=10,
        nodes=[1, 2, 3, 4, 5],
        dist_factors=None  # Optional distribution factors
    )

    # With distribution factors
    writer.put_node_set(20, [10, 11, 12], [1.0, 0.5, 0.5])

    # Set node set name (not yet implemented)
    # writer.put_node_set_name(10, "LeftBoundary")

# Reading node sets
with ExodusReader.open("/tmp/ug_nodesets_example.exo") as reader:
    node_set_ids = reader.get_node_set_ids()
    if len(node_set_ids) > 0:
        node_set = reader.get_node_set(node_set_ids[0])
        nodes = node_set.nodes
        dist_factors = node_set.dist_factors
        # Not yet implemented:
        # name = reader.get_node_set_name(node_set_ids[0])
```

### Side Sets

Define and use side sets:

```python
from exodus import ExodusWriter, ExodusReader, InitParams

# Writing side sets
with ExodusWriter.create("/tmp/ug_sidesets_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=8, num_elems=3, num_elem_blocks=1, num_side_sets=1)
    writer.put_init_params(params)

    writer.put_side_set(
        set_id=100,
        elements=[1, 2, 3],      # Element IDs
        sides=[1, 2, 1],         # Side numbers (1-based)
        dist_factors=None
    )

# Reading side sets
with ExodusReader.open("/tmp/ug_sidesets_example.exo") as reader:
    side_set_ids = reader.get_side_set_ids()
    if len(side_set_ids) > 0:
        side_set = reader.get_side_set(side_set_ids[0])
        elements = side_set.elements
        sides = side_set.sides
```

**Side Numbering:**
Side numbering depends on element topology. Generally:
- For quads: 1=bottom, 2=right, 3=top, 4=left
- For hexes: 1-6 for faces

### Edge and Face Blocks

For 3D meshes, you can also define edge and face blocks:

**Note:** Edge and face blocks may have special dimension requirements in NetCDF and may require additional configuration.

```python
from exodus import ExodusWriter, InitParams, Block, EntityType, CreateOptions, CreateMode

with ExodusWriter.create("/tmp/ug_edgeface_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=20, num_elems=1, num_elem_blocks=1, num_edge_blocks=1, num_face_blocks=1)
    writer.put_init_params(params)

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
    writer.put_connectivity(1, [1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11])

    # Face block
    face_block = Block(
        id=2,
        entity_type=EntityType.FaceBlock,
        topology="QUAD4",
        num_entries=5,
        num_nodes_per_entry=4,
        num_attributes=0,
    )
    writer.put_block(face_block)
    writer.put_connectivity(2, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20])
```

## Entity Sets

Entity sets provide a unified interface for working with different set types. In exodus-py, use the specialized `put_node_set` and `put_side_set` methods for working with sets:

```python
from exodus import ExodusWriter, ExodusReader, InitParams, CreateOptions, CreateMode

# Writing sets
with ExodusWriter.create("/tmp/ug_entity_sets_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=20, num_elems=5, num_elem_blocks=1, num_node_sets=1, num_side_sets=1)
    writer.put_init_params(params)

    # For node sets, use put_node_set:
    writer.put_node_set(set_id=1, nodes=[1, 2, 3, 4], dist_factors=[1.0, 1.0, 0.5, 0.5])

    # For side sets, use put_side_set:
    writer.put_side_set(set_id=1, elements=[1, 2], sides=[1, 2], dist_factors=None)

# Reading works through get_node_set and get_side_set:
with ExodusReader.open("/tmp/ug_entity_sets_example.exo") as reader:
    node_set = reader.get_node_set(1)
    side_set = reader.get_side_set(1)
```

## Assemblies

Assemblies provide hierarchical organization of entities, allowing you to group related blocks and sets together.

**Note:** The assembly API is not yet fully implemented in exodus-py. Check the API documentation for current implementation status. When available, assemblies will allow you to organize mesh entities into logical groups for better data organization.

## Attributes

Element blocks can have attributes (per-element scalar values):

```python
from exodus import BlockBuilder, ExodusReader, ExodusWriter, InitParams, Block, EntityType, MeshBuilder

# First create a sample mesh to read from
builder = MeshBuilder("Attributes Mesh")
builder.dimensions(3)
builder.coordinates(
    x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 2.0, 3.0],
    y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
    z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0]
)

block = BlockBuilder(1, "HEX8")
block.connectivity([1, 2, 3, 4, 5, 6, 7, 8, 2, 3, 4, 5, 6, 7, 8, 9, 3, 4, 5, 6, 7, 8, 9, 10])
block.attributes([100.0, 200.0, 150.0])
block.attribute_names(["MaterialID"])
builder.add_block(block.build())
builder.write("/tmp/ug_attrs_mesh.exo")

# Reading block attributes
with ExodusReader.open("/tmp/ug_attrs_mesh.exo") as reader:
    block_ids = reader.get_block_ids()
    if len(block_ids) > 0:
        block_id = block_ids[0]
        attrs = reader.get_block_attributes(block_id)
        attr_names = reader.get_block_attribute_names(block_id)

# Writing attributes with low-level API
with ExodusWriter.create("/tmp/ug_attributes_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=10, num_elems=3, num_elem_blocks=1)
    writer.put_init_params(params)
    block = Block(
        id=100,
        entity_type=EntityType.ElemBlock,
        topology="HEX8",
        num_entries=3,
        num_nodes_per_entry=8,
        num_attributes=1,
    )
    writer.put_block(block)
    writer.put_block_attributes(100, [100.0, 200.0, 150.0])
    writer.put_block_attribute_names(100, ["MaterialID"])
```

## Metadata

### QA Records

Quality assurance records track software that created/modified the file:

```python
from exodus import QaRecord, MeshBuilder, ExodusWriter, ExodusReader, InitParams, BlockBuilder

# Writing QA records (with builder)
builder = MeshBuilder("Test Mesh")
builder.dimensions(2)
builder.coordinates(x=[0.0, 1.0], y=[0.0, 0.0], z=[])

block = BlockBuilder(1, "BAR2")
block.connectivity([1, 2])
builder.add_block(block.build())
builder.qa_record("MyCode", "1.0.0", "2025-01-15", "14:30:00")
builder.write("/tmp/ug_qa_builder.exo")

# Writing QA records (with writer)
with ExodusWriter.create("/tmp/ug_qa_records_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=8, num_elems=1, num_elem_blocks=1)
    writer.put_init_params(params)

    qa = QaRecord(
        code_name="MyAnalysisCode",
        code_version="2.1.0",
        date="2025-01-15",
        time="14:30:00"
    )
    writer.put_qa_records([qa])

# Reading QA records
with ExodusReader.open("/tmp/ug_qa_builder.exo") as reader:
    qa_records = reader.get_qa_records()
    for qa in qa_records:
        code_name = qa.code_name
        code_version = qa.code_version
        date = qa.date
        time = qa.time
```

### Info Records

Information records are arbitrary text strings:

```python
from exodus import ExodusWriter, ExodusReader, InitParams

# Writing
with ExodusWriter.create("/tmp/ug_info_records_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=8, num_elems=1, num_elem_blocks=1)
    writer.put_init_params(params)

    writer.put_info_records([
        "Generated from CAD model v3",
        "Material properties from database",
        "Contact: engineer@example.com"
    ])

# Reading
with ExodusReader.open("/tmp/ug_info_records_example.exo") as reader:
    info = reader.get_info_records()
```

### File Metadata

Query file format and version information:

```python
# Get Exodus format version
major, minor = reader.version()

# Get NetCDF file format
format_str = reader.format()

# Get file path
path = reader.path()
```

## Maps

Maps provide alternative numbering schemes for nodes and elements.

**Note:** Most map write methods are not yet implemented. Element order maps can be written, but element number maps and node number maps cannot be written yet.

```python
from exodus import ExodusWriter, ExodusReader, InitParams, CreateOptions, CreateMode

# Writing element order map (processing order)
with ExodusWriter.create("/tmp/ug_maps_example.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(title="Test", num_dim=3, num_nodes=10, num_elems=5, num_elem_blocks=1)
    writer.put_init_params(params)

    # Element order map (processing order) - this works
    writer.put_elem_order_map([0, 1, 2, 3, 4])

    # Element number map (local to global ID mapping) - not yet implemented
    # writer.put_elem_num_map([100, 101, 102, 103, 104])

    # Node number map - not yet implemented
    # writer.put_node_num_map([1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007, 1008, 1009])

# Reading maps (works for existing files with maps)
with ExodusReader.open("/tmp/ug_maps_example.exo") as reader:
    elem_order = reader.get_elem_order_map()
    # elem_map = reader.get_elem_num_map()  # May return None if not present
    # node_map = reader.get_node_num_map()  # May return None if not present
```

## Blobs

Blobs allow storing arbitrary binary data within an Exodus II file.

**Note:** The blob API is not yet implemented in exodus-py. Check the API documentation for current implementation status.

## Reduction Variables

Reduction variables store aggregate values for entire objects (e.g., total mass for an assembly).

**Note:** Reduction variables are not yet fully implemented in exodus-py. Check the API documentation for current implementation status. When available, reduction variables will allow you to store summary statistics and aggregate values at various entity levels.

## Performance Optimization

### Chunking and Caching

**Note:** Advanced performance configuration API (chunking, caching) is not yet available in exodus-py. The underlying NetCDF library handles file I/O operations.

For optimal performance with the current implementation:
- Use context managers to ensure files are properly closed
- Read variables one at a time instead of loading all into memory
- Use appropriate data types (`Int64Mode`, `FloatSize`)

Advanced chunking and caching controls may be added in future releases.

## Best Practices

### Error Handling

Always handle potential errors:

```python
try:
    reader = ExodusReader.open("mesh.exo")
    params = reader.init_params()
    reader.close()
except FileNotFoundError:
    pass  # Handle file not found error
except RuntimeError as e:
    pass  # Handle Exodus error
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
with ExodusReader.open("mesh.exo") as reader:
    num_vars = len(reader.variable_names(EntityType.Nodal))
    num_steps = reader.num_time_steps()

    if num_vars > 0 and num_steps > 0:
        # Read one variable at a time instead of all at once
        for var_idx in range(num_vars):
            data = reader.var(0, EntityType.Nodal, 0, var_idx)
            # process(data)  # Process immediately
            # Data can be garbage collected

        # Instead of:
        # all_data = []
        # for var_idx in range(num_vars):
        #     all_data.append(reader.var(0, EntityType.Nodal, 0, var_idx))
        #     # All data kept in memory!
```

### Validation

Validate data before writing:

```python
# Example validation when writing a mesh
params = InitParams(title="Test", num_dim=3, num_nodes=8, num_elems=1, num_elem_blocks=1)

# Prepare coordinate data
x = [0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
y = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0]
z = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]

# Check coordinate lengths match num_nodes
assert len(x) == params.num_nodes
assert len(y) == params.num_nodes
assert len(z) == params.num_nodes or len(z) == 0

# Check connectivity length
num_elems = 1
nodes_per_elem = 8
connectivity = [1, 2, 3, 4, 5, 6, 7, 8]
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
