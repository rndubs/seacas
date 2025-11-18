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

### Converting NodeSets to SideSets

exodus-py provides convenient methods to automatically convert nodesets (collections of nodes) into sidesets (collections of element faces). This conversion:
- Identifies element faces where all nodes are in the nodeset
- Filters for boundary faces only (faces on the mesh exterior)
- Verifies normals point outward from the mesh center
- Checks for consistent normal orientations

This is particularly useful for defining boundary conditions on surfaces specified by nodal coordinates.

#### Basic Conversion

**Read-only conversion** (using ExodusReader):

```python
from exodus import ExodusReader

# Read and convert without modifying the file
with ExodusReader.open("mesh.exo") as reader:
    # Convert nodeset 10 to a new sideset with ID 100
    sideset = reader.convert_nodeset_to_sideset(
        nodeset_id=10,
        new_sideset_id=100
    )
    print(f"Created sideset {sideset.id} with {len(sideset.elements)} faces")
```

**Write to file** (using ExodusAppender):

```python
from exodus import ExodusAppender

# Convert and save to file
with ExodusAppender.append("mesh.exo") as appender:
    appender.create_sideset_from_nodeset(
        nodeset_id=10,
        new_sideset_id=100
    )
```

#### Auto-Increment IDs

Automatically assign sideset IDs by finding the maximum existing ID and incrementing:

```python
with ExodusAppender.append("mesh.exo") as appender:
    # Sideset ID is automatically assigned
    sideset_id = appender.create_sideset_from_nodeset_auto(10)
    print(f"Created sideset with auto-assigned ID: {sideset_id}")

    # Convert multiple nodesets without ID conflicts
    id1 = appender.create_sideset_from_nodeset_auto(20)
    id2 = appender.create_sideset_from_nodeset_auto(30)
```

#### Name-Based Conversion

Work with string names instead of numeric IDs:

```python
with ExodusAppender.append("mesh.exo") as appender:
    # Convert by name - automatically copies the nodeset's name to the sideset
    sideset_id = appender.create_sideset_from_nodeset_by_name("inlet")
    print(f"Converted nodeset 'inlet' to sideset {sideset_id}")

    # Create a sideset with an explicit name
    sideset_id = appender.create_sideset_from_nodeset_named(10, "outlet")
    print(f"Converted nodeset 10 to sideset named 'outlet'")
```

#### Conversion Examples

For complete examples demonstrating all conversion features, see:
- `rust/exodus-py/examples/nodeset_to_sideset.py`

The example includes:
- Multiple conversion approaches (read-only vs. write)
- Auto-increment ID assignment
- Name-based conversion
- Using context managers
- Inspecting conversion results

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

## Geometry Utilities

exodus-py provides high-performance geometry calculations for mesh analysis, powered by native Rust implementations.

### Element Volume Calculations

Calculate the volume of 3D finite elements using optimized tetrahedral decomposition methods.

**Supported Element Types:**
- `HEX8`, `HEX20`, `HEX27` - Hexahedral elements
- `TET4`, `TET8`, `TET10`, `TET14`, `TET15` - Tetrahedral elements
- `WEDGE6`, `WEDGE15`, `WEDGE18` - Wedge/prism elements
- `PYRAMID5`, `PYRAMID13`, `PYRAMID14` - Pyramidal elements

```python
from exodus import element_volume

# Compute volume of a unit cube hex element
hex_coords = [
    [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
]
volume = element_volume("HEX8", hex_coords)
print(f"Hex volume: {volume}")  # 1.0

# Compute volume of a tetrahedron
tet_coords = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
]
volume = element_volume("TET4", tet_coords)
print(f"Tet volume: {volume}")  # 0.16666... (1/6)

# Works with distorted/irregular elements too
distorted_hex = [
    [0.0, 0.0, 0.0], [1.2, 0.0, 0.0], [1.0, 1.1, 0.0], [0.0, 1.0, 0.0],
    [0.1, 0.0, 1.0], [1.0, 0.0, 0.9], [1.0, 1.0, 1.0], [0.0, 1.0, 1.1],
]
volume = element_volume("HEX8", distorted_hex)
```

**Computing volumes for an entire mesh (manual approach):**

```python
from exodus import ExodusReader, element_volume

# Read mesh and compute element volumes manually
with ExodusReader.open("mesh.exo") as reader:
    # Get element block information
    block_ids = reader.get_block_ids()
    coords_x, coords_y, coords_z = reader.get_coords()

    # Process each element block
    for block_id in block_ids:
        block = reader.get_block(block_id)
        connectivity = reader.get_connectivity(block_id)

        # Get topology and nodes per element
        topology = block.topology
        nodes_per_elem = block.num_nodes_per_entry
        num_elems = block.num_entries

        # Compute volume for each element
        volumes = []
        for elem_idx in range(num_elems):
            # Extract node indices for this element
            start = elem_idx * nodes_per_elem
            end = start + nodes_per_elem
            node_indices = connectivity[start:end]

            # Get coordinates for this element's nodes
            elem_coords = []
            for node_id in node_indices:
                # Node IDs are 1-based in Exodus, convert to 0-based for array access
                idx = node_id - 1
                elem_coords.append([coords_x[idx], coords_y[idx], coords_z[idx]])

            # Compute volume
            vol = element_volume(topology, elem_coords)
            volumes.append(vol)

        total_volume = sum(volumes)
        print(f"Block {block_id}: {num_elems} elements, total volume = {total_volume}")
```

**Computing volumes for an entire mesh (simplified approach):**

The above manual approach requires deeply nested loops. exodus-py provides high-level methods
that eliminate this complexity:

```python
from exodus import ExodusReader

# Compute volumes for entire mesh with one method call
with ExodusReader.open("mesh.exo") as reader:
    # Get all element volumes at once
    all_volumes = reader.all_element_volumes()
    total_volume = sum(all_volumes)
    print(f"Total mesh volume: {total_volume}")
    print(f"Number of elements: {len(all_volumes)}")

    # Or compute volumes for a specific block
    block_ids = reader.get_block_ids()
    block_volumes = reader.block_element_volumes(block_ids[0])
    print(f"Block {block_ids[0]} volume: {sum(block_volumes)}")
```

**Use Cases:**
- Mesh quality metrics and validation
- Physics calculations (mass, density)
- Post-processing and analysis
- Volume-weighted averaging

### Element Centroid Calculations

Calculate the geometric center (centroid) of an element as the average of all node positions.

```python
from exodus import element_centroid

# Compute centroid of a cube
hex_coords = [
    [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
]
centroid = element_centroid(hex_coords)
print(f"Centroid: {centroid}")  # [0.5, 0.5, 0.5]

# Works with any element type
tet_coords = [
    [0.0, 0.0, 0.0],
    [3.0, 0.0, 0.0],
    [0.0, 3.0, 0.0],
    [0.0, 0.0, 3.0],
]
centroid = element_centroid(tet_coords)
print(f"Tet centroid: {centroid}")  # [0.75, 0.75, 0.75]
```

**Computing centroids for mesh visualization (manual approach):**

```python
from exodus import ExodusReader, element_centroid
import numpy as np

# Read mesh and compute element centroids manually
with ExodusReader.open("mesh.exo") as reader:
    block_ids = reader.get_block_ids()
    coords_x, coords_y, coords_z = reader.get_coords()

    all_centroids = []

    for block_id in block_ids:
        block = reader.get_block(block_id)
        connectivity = reader.get_connectivity(block_id)

        nodes_per_elem = block.num_nodes_per_entry
        num_elems = block.num_entries

        for elem_idx in range(num_elems):
            # Extract element coordinates
            start = elem_idx * nodes_per_elem
            end = start + nodes_per_elem
            node_indices = connectivity[start:end]

            elem_coords = []
            for node_id in node_indices:
                idx = node_id - 1
                elem_coords.append([coords_x[idx], coords_y[idx], coords_z[idx]])

            # Compute centroid
            centroid = element_centroid(elem_coords)
            all_centroids.append(centroid)

    # Convert to numpy array for further processing
    centroids = np.array(all_centroids)

    # Useful for:
    # - Spatial queries (find elements in a region)
    # - Element-based visualization
    # - Sorting elements by location
    # - Computing distances between elements
```

**Computing centroids for mesh visualization (simplified approach):**

The above manual approach requires deeply nested loops. exodus-py provides high-level methods
that eliminate this complexity:

```python
from exodus import ExodusReader
import numpy as np

# Compute centroids for entire mesh with one method call
with ExodusReader.open("mesh.exo") as reader:
    # Get all element centroids at once
    all_centroids = reader.all_element_centroids()
    centroids = np.array(all_centroids)

    print(f"Mesh has {len(all_centroids)} elements")
    print(f"First element centroid: {all_centroids[0]}")

    # Or compute centroids for a specific block
    block_ids = reader.get_block_ids()
    block_centroids = reader.block_element_centroids(block_ids[0])
    print(f"Block {block_ids[0]} has {len(block_centroids)} elements")

    # Useful for:
    # - Spatial queries (find elements in a region)
    # - Element-based visualization
    # - Sorting elements by location
    # - Computing distances between elements
```

**Use Cases:**
- Element location and spatial queries
- Visualization and post-processing
- Element sorting and organization
- Distance calculations
- Interpolation to element centers

### Performance Notes

Both geometry functions are implemented in native Rust for optimal performance:
- **element_volume**: Uses efficient tetrahedral decomposition
  - Hexahedron: 5 tetrahedra
  - Wedge: 3 tetrahedra
  - Pyramid: 2 tetrahedra
- **element_centroid**: Simple average of node positions
- No Python/Rust boundary overhead for batch operations
- Works correctly for both regular and distorted elements

### Error Handling

Both functions provide clear error messages for invalid inputs:

```python
from exodus import element_volume, element_centroid

# Invalid topology
try:
    volume = element_volume("SPHERE", [[0, 0, 0]])
except RuntimeError as e:
    print(f"Error: {e}")  # "Volume calculation not supported for topology: SPHERE"

# Insufficient coordinates
try:
    volume = element_volume("HEX8", [[0, 0, 0], [1, 0, 0]])  # Need 8 coords
except RuntimeError as e:
    print(f"Error: {e}")  # "HEX element requires at least 8 coordinates, got 2"

# Invalid coordinate dimensions
try:
    volume = element_volume("TET4", [[0, 0], [1, 0], [0, 1], [0, 0]])
except ValueError as e:
    print(f"Error: {e}")  # "Each coordinate must have 3 values (x, y, z), got 2"
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

exodus-py provides comprehensive performance tuning through HDF5 chunk cache configuration. These settings can dramatically improve I/O performance (1-50x speedups for large meshes) by optimizing how NetCDF-4/HDF5 manages data access patterns.

#### Quick Start

**Automatic Configuration (Recommended)**

The simplest approach is to let exodus-py auto-detect your environment and apply optimal settings:

```python
from exodus import ExodusWriter, CreateOptions, PerformanceConfig, CreateMode, InitParams

# Auto-detect node type and apply appropriate settings
perf = PerformanceConfig.auto()
opts = CreateOptions(mode=CreateMode.Clobber, performance=perf)
file = ExodusWriter.create("mesh.exo", opts)

# Initialize and use normally
params = InitParams(
    title="Optimized Mesh",
    num_dim=3,
    num_nodes=100000,
    num_elems=90000,
    num_elem_blocks=10
)
file.put_init_params(params)
# ... continue with mesh creation
file.close()
```

**Preset Configurations**

Use predefined presets for common scenarios:

```python
# Conservative settings (4MB cache) - safe for login nodes
perf = PerformanceConfig.conservative()

# Aggressive settings (128MB cache) - for compute nodes
perf = PerformanceConfig.aggressive()

opts = CreateOptions(mode=CreateMode.Clobber, performance=perf)
file = ExodusWriter.create("mesh.exo", opts)
```

**Custom Configuration**

Fine-tune performance for your specific workload:

```python
# Custom settings for large compute node
perf = PerformanceConfig.auto() \
    .with_cache_mb(256) \
    .with_node_chunk_size(20000) \
    .with_element_chunk_size(15000) \
    .with_preemption(0.5)

opts = CreateOptions(mode=CreateMode.Clobber, performance=perf)
file = ExodusWriter.create("mesh.exo", opts)
```

#### Node Type Detection

exodus-py automatically detects whether you're running on:
- **Compute nodes** (inside a job scheduler) - 128MB cache, 10k chunk sizes
- **Login nodes** (on HPC system but not in job) - 4MB cache, 1k chunk sizes (conservative)
- **Unknown** (local development machine) - 16MB cache, 5k chunk sizes (moderate)

Supported job schedulers: SLURM, Flux, PBS, LSF

```python
from exodus import NodeType

# Check current environment
node_type = NodeType.detect()
print(f"Detected: {node_type}")

# Get defaults for this node type
print(f"Default cache size: {node_type.default_cache_size()} bytes")
print(f"Default node chunk size: {node_type.default_chunk_nodes()}")

# Create configuration for specific node type
perf = PerformanceConfig.for_node_type(node_type)
```

#### Performance Classes

**PerformanceConfig**

High-level configuration combining cache and chunk settings:

```python
# Factory methods
PerformanceConfig.auto()           # Auto-detect environment
PerformanceConfig.conservative()   # Safe for login nodes (4MB)
PerformanceConfig.aggressive()     # Optimized for compute (128MB)
PerformanceConfig.for_node_type(node_type)  # Node-specific

# Builder methods (chainable)
perf.with_cache_mb(256)           # Set cache size in megabytes
perf.with_node_chunk_size(20000)  # Nodes per chunk
perf.with_element_chunk_size(15000)  # Elements per chunk
perf.with_time_chunk_size(10)     # Time steps per chunk (for transient data)
perf.with_preemption(0.5)         # Cache eviction policy (0.0-1.0)

# Inspection
perf.summary()  # Human-readable configuration summary
```

**CacheConfig**

Fine-grained HDF5 chunk cache control:

```python
from exodus import CacheConfig

# Create with default size (auto-detected)
cache = CacheConfig()

# Create with specific size
cache = CacheConfig(cache_size=64 * 1024 * 1024)  # 64MB

# Configure cache parameters
cache = cache.with_cache_mb(128)     # 128MB cache
cache = cache.with_preemption(0.75)  # Preemption policy

# Access properties
print(cache.cache_size)   # Size in bytes
print(cache.preemption)   # Preemption value (0.0-1.0)
```

**ChunkConfig**

Configure chunk dimensions for spatial and temporal data:

```python
from exodus import ChunkConfig

# Auto-detect based on node type
chunks = ChunkConfig()

# Custom chunk sizes
chunks = ChunkConfig(
    node_chunk_size=20000,
    element_chunk_size=15000,
    time_chunk_size=10
)

# Builder pattern
chunks = chunks.with_node_chunk_size(25000)
chunks = chunks.with_element_chunk_size(20000)
chunks = chunks.with_time_chunk_size(5)

# Access properties
print(chunks.node_chunk_size)
print(chunks.element_chunk_size)
print(chunks.time_chunk_size)
```

#### Cache Size Guidelines

| Node RAM | Recommended Cache | Use Case |
|----------|-------------------|----------|
| 16 GB | 4-16 MB | Development/login nodes |
| 64 GB | 32-128 MB | Small compute nodes |
| 256 GB | 128-512 MB | Large compute nodes |
| 512+ GB | 1+ GB | HPC nodes |

**Rule of thumb**: Larger cache = better performance (up to available memory)

#### Preemption Policy

Controls which chunks get evicted from cache first:

| Value | Behavior | Use Case |
|-------|----------|----------|
| 0.0 | Never evict write-only chunks | Write-heavy workloads |
| 0.75 | Balanced (default) | Mixed read/write |
| 1.0 | Aggressively evict write-only | Read-heavy workloads |

#### Example: Large Mesh with Optimized Performance

```python
import numpy as np
from exodus import (
    ExodusWriter, CreateOptions, CreateMode,
    PerformanceConfig, InitParams
)

# Configure for large compute node
perf = PerformanceConfig.aggressive() \
    .with_cache_mb(512) \
    .with_node_chunk_size(50000) \
    .with_element_chunk_size(40000)

opts = CreateOptions(mode=CreateMode.Clobber, performance=perf)

with ExodusWriter.create("large_mesh.exo", opts) as file:
    # Initialize large mesh
    params = InitParams(
        title="Large Mesh - Optimized Performance",
        num_dim=3,
        num_nodes=1000000,
        num_elems=900000,
        num_elem_blocks=10
    )
    file.put_init_params(params)

    # Write coordinates efficiently
    x = np.random.rand(1000000)
    y = np.random.rand(1000000)
    z = np.random.rand(1000000)
    file.put_coords(x, y, z)

    # Large cache significantly speeds up these operations
    # ... continue with mesh creation
```

#### Performance Tips

For optimal performance with exodus-py:
- **Use performance config**: Always specify performance settings for large meshes
- **Auto-detect when possible**: `PerformanceConfig.auto()` handles most cases well
- **Tune for workload**: Write-heavy? Use lower preemption (0.0-0.5)
- **Match chunk sizes**: Set chunks to match your typical read/write patterns
- **Use context managers**: Ensure files are properly closed to flush caches
- **Avoid over-allocation**: Don't set cache larger than available RAM
- **Test your settings**: Run benchmarks to find optimal values for your data

#### Additional Resources

For detailed performance tuning guidance, benchmark results, and troubleshooting:
- See `rust/exodus-rs/PERFORMANCE.md` in the repository for comprehensive performance guide
- Example: `rust/exodus-py/tests/test_performance.py` for complete test coverage

#### General Best Practices

Beyond chunking and caching, follow these practices:
- Use context managers to ensure files are properly closed
- Read variables one at a time instead of loading all into memory
- Use appropriate data types (`Int64Mode`, `FloatSize`) for your data range

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
