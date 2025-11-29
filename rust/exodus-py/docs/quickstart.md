# Quickstart Guide

This guide will get you started with the most common exodus-py operations.

## Opening an Existing File

The most common operation is reading an existing Exodus file to extract mesh data and results.

### Basic File Reading

```python
from exodus import ExodusReader

# Open file using context manager (recommended)
with ExodusReader.open("mesh.exo") as reader:
    # Get basic mesh information
    params = reader.init_params()

    # Get coordinates
    x, y, z = reader.get_coords()

    # Get element blocks
    block_ids = reader.get_block_ids()
    for block_id in block_ids:
        block = reader.get_block(block_id)
        conn = reader.get_connectivity(block_id)
```

### Reading Time Series Data

If your Exodus file contains time-dependent results:

```python
from exodus import (
    ExodusReader, EntityType, ExodusWriter,
    InitParams, Block, MeshBuilder, BlockBuilder,
    CreateMode, CreateOptions
)

# First create a sample file with time series data
builder = MeshBuilder("Results Mesh")
builder.dimensions(2)
builder.coordinates(
    x=[0.0, 1.0, 1.0, 0.0],
    y=[0.0, 0.0, 1.0, 1.0],
    z=[]
)

block = BlockBuilder(1, "QUAD4")
block.connectivity([1, 2, 3, 4])
builder.add_block(block.build())
builder.write("/tmp/quickstart_results1.exo")

# Add variables and time steps using Writer
options = CreateOptions(mode=CreateMode.Clobber)
writer = ExodusWriter.create("/tmp/quickstart_results_temp1.exo", options)

# Copy the mesh
params = InitParams(
    title="Results Mesh",
    num_dim=2,
    num_nodes=4,
    num_elems=1,
    num_elem_blocks=1
)
writer.put_init_params(params)
writer.put_coords([0.0, 1.0, 1.0, 0.0], [0.0, 0.0, 1.0, 1.0], [])

block_info = Block(
    id=1,
    entity_type=EntityType.ElemBlock,
    topology="QUAD4",
    num_entries=1,
    num_nodes_per_entry=4,
    num_attributes=0
)
writer.put_block(block_info)
writer.put_connectivity(1, [1, 2, 3, 4])
# Define and write variables
writer.define_variables(EntityType.Nodal, ["Temperature"])
writer.define_variables(EntityType.Global, ["TotalEnergy"])

# Write first time step
writer.put_time(0, 1.0)
writer.put_var(0, EntityType.Nodal, 0, 0, [100.0, 150.0, 150.0, 100.0])
writer.put_var(0, EntityType.Global, 0, 0, [1000.0])

# Write second time step
writer.put_time(1, 2.0)
writer.put_var(1, EntityType.Nodal, 0, 0, [110.0, 160.0, 160.0, 110.0])
writer.put_var(1, EntityType.Global, 0, 0, [1100.0])
writer.close()

# Read the data back
with ExodusReader.open("/tmp/quickstart_results_temp1.exo") as reader:
    num_steps = reader.num_time_steps()
    times = reader.times()

    nodal_vars = reader.variable_names(EntityType.Nodal)
    global_vars = reader.variable_names(EntityType.Global)

    # Read nodal variable at first time step
    temperature = reader.var(0, EntityType.Nodal, 0, 0)

    # Read global variable at first time step
    energy = reader.var(0, EntityType.Global, 0, 0)

    # Read variable time series
    temp_history = reader.var_time_series(0, num_steps, EntityType.Nodal, 0, 0)
```

## Writing a New File After Adding Time Series Data

A common workflow is to read an existing mesh, add new time step data, and write it to a new file.

### Complete Workflow Example

```python
from exodus import (
    ExodusReader,
    ExodusWriter,
    ExodusAppender,
    EntityType,
    CreateMode,
    CreateOptions,
    MeshBuilder,
    BlockBuilder
)

# First create an example mesh file
builder = MeshBuilder("Original Mesh")
builder.dimensions(3)
builder.coordinates(
    x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
    y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
    z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
)

block = BlockBuilder(1, "HEX8")
block.connectivity([1, 2, 3, 4, 5, 6, 7, 8])
builder.add_block(block.build())
builder.write("/tmp/quickstart_original_mesh.exo")

# Step 1: Read the original mesh
with ExodusReader.open("/tmp/quickstart_original_mesh.exo") as reader:
    # Get all mesh parameters
    params = reader.init_params()
    coords = reader.get_coords()
    block_ids = reader.get_block_ids()

    # Store blocks and connectivity
    blocks = []
    connectivities = {}
    for block_id in block_ids:
        block = reader.get_block(block_id)
        blocks.append(block)
        connectivities[block_id] = reader.get_connectivity(block_id)

    # Read existing variable data if any
    existing_vars = {}
    if reader.num_time_steps() > 0:
        nodal_vars = reader.variable_names(EntityType.Nodal)
        existing_vars['nodal'] = nodal_vars
        # Read existing time step data...

# Step 2: Create a new file with the same mesh
options = CreateOptions(mode=CreateMode.Clobber)
writer = ExodusWriter.create("/tmp/quickstart_new_results.exo", options)

# Initialize with original parameters, but update time step count
writer.put_init_params(params)

# Write mesh geometry
writer.put_coords(coords[0], coords[1], coords[2])

# Write element blocks
for block in blocks:
    writer.put_block(block)
    writer.put_connectivity(block.id, connectivities[block.id])

# Step 3: Define new variables for time series data
writer.define_variables(EntityType.Global, ["TotalEnergy", "MaxStress"])
writer.define_variables(EntityType.Nodal, ["Temperature", "Pressure", "Displacement"])
writer.define_variables(EntityType.ElemBlock, ["Stress", "Strain"])

# Step 4: Write time series data
num_nodes = params.num_nodes

# Write data for multiple time steps
for step in range(5):
    time_value = step * 0.1  # Time increment

    # Write time value for this step
    writer.put_time(step, time_value)

    # Write global variables (single values)
    total_energy = 1000.0 - step * 10.0
    max_stress = 500.0 + step * 5.0
    writer.put_var(step, EntityType.Global, 0, 0, [total_energy])
    writer.put_var(step, EntityType.Global, 0, 1, [max_stress])

    # Write nodal variables (one value per node)
    # Generate sample data: temperature varies with node index and time
    temperatures = [20.0 + i * 0.5 + step * 2.0 for i in range(num_nodes)]
    pressures = [100.0 + i * 0.1 + step * 0.5 for i in range(num_nodes)]
    displacements = [0.0 + i * 0.01 + step * 0.02 for i in range(num_nodes)]

    writer.put_var(step, EntityType.Nodal, 0, 0, temperatures)
    writer.put_var(step, EntityType.Nodal, 0, 1, pressures)
    writer.put_var(step, EntityType.Nodal, 0, 2, displacements)

    # Write element block variables (one value per element in the block)
    for block in blocks:
        num_elems = block.num_entries
        stresses = [100.0 + j * 1.0 + step * 5.0 for j in range(num_elems)]
        strains = [0.001 + j * 0.0001 + step * 0.0005 for j in range(num_elems)]

        writer.put_var(step, EntityType.ElemBlock, block.id, 0, stresses)
        writer.put_var(step, EntityType.ElemBlock, block.id, 1, strains)

# Close the writer
writer.close()

# Step 5: Verify the new file
with ExodusReader.open("/tmp/quickstart_new_results.exo") as reader:
    params_new = reader.init_params()
    num_steps = reader.num_time_steps()

    # Check that variables were written
    nodal_vars = reader.variable_names(EntityType.Nodal)

    # Read back first time step
    temp_0 = reader.var(0, EntityType.Nodal, 0, 0)
```

### Simpler Approach: Using MeshBuilder

If you're creating a mesh from scratch rather than copying an existing one:

```python
from exodus import MeshBuilder, BlockBuilder, EntityType, ExodusWriter

# Create mesh with builder API
builder = MeshBuilder("Analysis Results")
builder.dimensions(3)
builder.coordinates(
    x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
    y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
    z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
)

block = BlockBuilder(1, "HEX8")
block.connectivity([1, 2, 3, 4, 5, 6, 7, 8])
builder.add_block(block.build())

# Write to file and define variables in one step
options = CreateOptions(mode=CreateMode.Clobber)
writer = ExodusWriter.create("/tmp/quickstart_hex_mesh_final.exo", options)

params2 = InitParams(
    title="Analysis Results",
    num_dim=3,
    num_nodes=8,
    num_elems=1,
    num_elem_blocks=1
)
writer.put_init_params(params2)
writer.put_coords(
    [0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
    [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
    [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
)

hex_block = Block(
    id=1,
    entity_type=EntityType.ElemBlock,
    topology="HEX8",
    num_entries=1,
    num_nodes_per_entry=8,
    num_attributes=0
)
writer.put_block(hex_block)
writer.put_connectivity(1, [1, 2, 3, 4, 5, 6, 7, 8])
writer.define_variables(EntityType.Nodal, ["Temperature"])

# Write variable data for each time step
for step in range(10):
    writer.put_time(step, step * 0.5)
    temps = [300.0 + i * 10.0 + step * 5.0 for i in range(8)]
    writer.put_var(step, EntityType.Nodal, 0, 0, temps)

writer.close()
```

## Adding Sets with Builder API

exodus-py provides a fluent builder API for adding node sets and side sets to files:

```python
from exodus import (
    AppendBuilder, NodeSetBuilder, SideSetBuilder,
    ExodusWriter, ExodusReader, InitParams, CreateOptions, CreateMode
)

# First, create a mesh with reserved space for sets
with ExodusWriter.create("/tmp/quickstart_sets.exo", CreateOptions(mode=CreateMode.Clobber)) as writer:
    params = InitParams(
        title="Mesh with Sets",
        num_dim=2,
        num_nodes=4,
        num_elems=1,
        num_elem_blocks=1,
        num_node_sets=2,  # Reserve space for node sets
        num_side_sets=1,  # Reserve space for side sets
    )
    writer.put_init_params(params)
    writer.put_coords(x=[0.0, 1.0, 1.0, 0.0], y=[0.0, 0.0, 1.0, 1.0])

# Add sets using the fluent builder API
builder = AppendBuilder.open("/tmp/quickstart_sets.exo")
builder.add_node_set(NodeSetBuilder(10).nodes([1, 2]).name("bottom").build())
builder.add_node_set(NodeSetBuilder(20).nodes([3, 4]).name("top").build())
builder.add_side_set(SideSetBuilder(100).sides([(1, 1)]).name("left_edge").build())
builder.apply()

# Verify the sets were added
with ExodusReader.open("/tmp/quickstart_sets.exo") as reader:
    node_set_ids = reader.get_node_set_ids()
    side_set_ids = reader.get_side_set_ids()
    print(f"Node sets: {node_set_ids}")  # [10, 20]
    print(f"Side sets: {side_set_ids}")  # [100]
```

## Key Concepts

### Entity Types

Variables in Exodus are associated with different entity types:

- **`EntityType.Global`**: Single values per time step (e.g., total energy)
- **`EntityType.Nodal`**: Values at each node (e.g., temperature, displacement)
- **`EntityType.ElemBlock`**: Values for each element in a block (e.g., stress, strain)
- **`EntityType.NodeSet`**: Values for nodes in a node set
- **`EntityType.SideSet`**: Values for sides in a side set

### Variable Indexing

When reading/writing variables:
- **Time steps** are 0-based: first step is 0
- **Variable indices** are 0-based: first variable is 0
- **Entity IDs**: For blocks/sets, use the actual ID (e.g., block 100)
- **For global/nodal variables**: Use entity_id = 0

### Context Managers

Always use context managers (`with` statement) when opening files:

```python
# Good - file is automatically closed
with ExodusReader.open("mesh.exo") as reader:
    data = reader.get_coords()

# Also acceptable - manual close
reader = ExodusReader.open("mesh.exo")
try:
    data = reader.get_coords()
finally:
    reader.close()
```

## Next Steps

- Learn more in the {doc}`user_guide`
- Explore the complete {doc}`api_reference`
- Check out example files in the `examples/` directory
