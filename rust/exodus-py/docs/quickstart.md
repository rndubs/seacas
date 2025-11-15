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
    print(f"Title: {params.title}")
    print(f"Dimensions: {params.num_dim}D")
    print(f"Nodes: {params.num_nodes}")
    print(f"Elements: {params.num_elems}")

    # Get coordinates
    x, y, z = reader.get_coords()
    print(f"X coordinates: {x}")
    print(f"Y coordinates: {y}")

    # Get element blocks
    block_ids = reader.get_block_ids()
    for block_id in block_ids:
        block = reader.get_block(block_id)
        print(f"Block {block_id}: {block.topology}, {block.num_entries} elements")

        # Get connectivity for this block
        conn = reader.get_connectivity(block_id)
        print(f"  Connectivity: {conn}")
```

### Reading Time Series Data

If your Exodus file contains time-dependent results:

```python
import os

# This example shows how to read time series data
# We'll use mesh.exo which may not have time series
with ExodusReader.open("mesh.exo") as reader:
    # Get number of time steps
    num_steps = reader.num_time_steps()
    print(f"File has {num_steps} time steps")

    # Get all time values
    times = reader.times() if num_steps > 0 else []
    print(f"Time values: {times}")

    # Get variable names
    nodal_vars = reader.variable_names(EntityType.Nodal)
    print(f"Nodal variables: {nodal_vars}")

    global_vars = reader.variable_names(EntityType.Global)
    print(f"Global variables: {global_vars}")

    # Read nodal variable at first time step (if variables exist)
    if num_steps > 0 and len(nodal_vars) > 0:
        # Arguments: step (0-based), var_type, entity_id, var_index (0-based)
        temperature = reader.var(0, EntityType.Nodal, 0, 0)
        print(f"Temperature at t=0: {temperature}")

        # Read global variable at first time step (if variables exist)
        if len(global_vars) > 0:
            energy = reader.var(0, EntityType.Global, 0, 0)
            print(f"Total energy at t=0: {energy[0]}")

            # Read variable time series (all time steps for one variable)
            temp_history = reader.var_time_series(0, num_steps, EntityType.Nodal, 0, 0)
            print(f"Temperature history length: {len(temp_history)}")
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
    CreateOptions
)

# Step 1: Read the original mesh
print("Reading original mesh...")
with ExodusReader.open("original_mesh.exo") as reader:
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
print("Creating new mesh file...")
writer = ExodusWriter.create("new_results.exo", CreateOptions(mode=CreateMode.Clobber))

# Initialize with original parameters, but update time step count
writer.put_init_params(params)

# Write mesh geometry
writer.put_coords(coords[0], coords[1], coords[2])

# Write element blocks
for block in blocks:
    writer.put_block(block)
    writer.put_connectivity(block.id, connectivities[block.id])

# Step 3: Define new variables for time series data
print("Defining variables...")
writer.define_variables(EntityType.Global, ["TotalEnergy", "MaxStress"])
writer.define_variables(EntityType.Nodal, ["Temperature", "Pressure", "Displacement"])
writer.define_variables(EntityType.ElemBlock, ["Stress", "Strain"])

# Step 4: Write time series data
print("Writing time series data...")
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
print("✓ Successfully created new_results.exo with time series data")

# Step 5: Verify the new file
print("\nVerifying new file...")
with ExodusReader.open("new_results.exo") as reader:
    params_new = reader.init_params()
    num_steps = reader.num_time_steps()
    print(f"Verification: {num_steps} time steps written")

    # Check that variables were written
    nodal_vars = reader.variable_names(EntityType.Nodal)
    print(f"Nodal variables: {nodal_vars}")

    # Read back first time step
    temp_0 = reader.var(0, EntityType.Nodal, 0, 0)
    print(f"First node temperature at t=0: {temp_0[0]:.2f}")
```

### Simpler Approach: Using MeshBuilder

If you're creating a mesh from scratch rather than copying an existing one:

```python
# Note: Example of creating mesh from scratch
# skip
from exodus import MeshBuilder, BlockBuilder

# Create mesh with builder API
builder = MeshBuilder("Analysis Results")
builder.dimensions(3)
builder.coordinates(
    x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
    y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
    z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
)
builder.add_block(
    BlockBuilder(1, "HEX8")
        .connectivity([1, 2, 3, 4, 5, 6, 7, 8])
        .build()
)

# Write to file (this creates the file with geometry only)
builder.write("hex_mesh.exo")

# Time series data would need to be added using ExodusWriter
# with the complete workflow shown above
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
