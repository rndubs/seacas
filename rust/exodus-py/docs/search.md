# Spatial Search

The `exodus` Python package provides spatial search capabilities for finding and extracting nodal and element field data based on spatial coordinates.

## Overview

Spatial search allows you to:
- Find the nearest node to a given (x, y, z) coordinate
- Find the nearest element (using element centroids) to a given coordinate
- Extract complete time-history data for variables at specific spatial locations
- Control search distance limits to avoid false matches
- Slice time-history data by index or time value

## Basic Usage

### Searching for Nodal Variables

```python
from exodus import ExodusReader

# Open an Exodus file
reader = ExodusReader.open("simulation.exo")

# Search for temperature data near a point
# Default uses 5x average element size as search radius
result = reader.search_nodal_variable(
    x=1.0,
    y=2.0,
    z=3.0,
    var_name="temperature"
)

print(f"Found node {result.id} at distance {result.distance}")
print(f"Time history: {result.time_history}")

# Search with custom distance limit
result = reader.search_nodal_variable(
    x=1.0, y=2.0, z=3.0,
    var_name="pressure",
    max_distance=0.5  # Only search within 0.5 units
)

# Search without distance limit (always find nearest)
result = reader.search_nodal_variable(
    x=1.0, y=2.0, z=3.0,
    var_name="velocity_x",
    max_distance=-1.0  # Negative means no limit
)
```

### Searching for Element Variables

```python
# Search for element variable using element centroids
result = reader.search_element_variable(
    x=1.0, y=2.0, z=3.0,
    var_name="stress"
)

print(f"Found element {result.id} at distance {result.distance}")
print(f"Number of time steps: {len(result.time_history)}")
```

### Low-Level Search Methods

```python
# Find nearest node (returns tuple of node_id, distance)
node_id, distance = reader.find_nearest_node(1.0, 2.0, 3.0, max_distance=-1.0)

# Find nearest element (returns tuple of elem_id, distance)
elem_id, distance = reader.find_nearest_element(1.0, 2.0, 3.0, max_distance=-1.0)

# Get average element size (useful for setting search limits)
avg_size = reader.average_element_size()
search_radius = 3.0 * avg_size  # Custom search radius
```

## Working with Search Results

### The SpatialSearchResult Class

Search results are returned as `SpatialSearchResult` objects with these attributes:

- `id`: The matched node or element ID (1-based)
- `distance`: Distance from search point to matched location
- `time_history`: List of values at all time steps

### Slicing Time History by Index

```python
result = reader.search_nodal_variable(1.0, 2.0, 3.0, "temperature")

# Get time steps 10 through 20
sliced = result.slice(start=10, end=20, step=1)

# Get every other time step
sliced = result.slice(start=0, end=None, step=2)

# Get last 5 time steps
sliced = result.slice(start=-5, end=None, step=1)
```

### Slicing Time History by Time Value

```python
result = reader.search_nodal_variable(1.0, 2.0, 3.0, "temperature")

# Get data from time 0.5 to 1.5 seconds
sliced = result.slice_by_time(reader, start_time=0.5, end_time=1.5)

# Get single time point
sliced = result.slice_by_time(reader, start_time=1.0, end_time=1.0)
```

## Distance Limits

### Default Behavior

By default, searches use a distance limit of **5 times the average element size**. This prevents unexpected matches when the search point is far from the mesh.

```python
# Uses default limit (5x average element size)
result = reader.search_nodal_variable(1.0, 2.0, 3.0, "temperature")

# Equivalent to:
avg_size = reader.average_element_size()
result = reader.search_nodal_variable(
    1.0, 2.0, 3.0, "temperature",
    max_distance=5.0 * avg_size
)
```

### Custom Distance Limits

```python
# Tight search radius
result = reader.search_nodal_variable(
    1.0, 2.0, 3.0, "temperature",
    max_distance=0.1
)

# Unlimited search (always finds nearest, no matter how far)
result = reader.search_nodal_variable(
    1.0, 2.0, 3.0, "temperature",
    max_distance=-1.0
)
```

### Handling Search Failures

If no node/element is found within the search distance, an exception is raised:

```python
try:
    result = reader.search_nodal_variable(
        100.0, 100.0, 100.0,  # Far from mesh
        "temperature",
        max_distance=1.0  # Small search radius
    )
except RuntimeError as e:
    print(f"Search failed: {e}")
    # Handle the error appropriately
```

## Complete Example

```python
from exodus import ExodusReader

# Open file
reader = ExodusReader.open("simulation.exo")

# Find temperature at specific location
result = reader.search_nodal_variable(
    x=5.0, y=10.0, z=2.0,
    var_name="temperature",
    max_distance=None  # Use default (5x avg element size)
)

print(f"Matched node: {result.id}")
print(f"Distance from search point: {result.distance:.4f}")
print(f"Total time steps: {len(result.time_history)}")

# Analyze time history
max_temp = max(result.time_history)
min_temp = min(result.time_history)
print(f"Temperature range: {min_temp:.2f} to {max_temp:.2f}")

# Get data for specific time range
time_slice = result.slice_by_time(reader, 1.0, 5.0)
print(f"Values from t=1.0 to t=5.0: {time_slice.time_history}")

# Close file
reader.close()
```

## Notes

- Node and element IDs are 1-based (following Exodus convention)
- Distance calculations use Euclidean distance in 3D space
- For 2D meshes, z-coordinates are treated as 0.0
- Element searches use element centroids (geometric center of nodes)
- Search performance is O(n) where n is number of nodes/elements
- For large meshes with many queries, consider caching coordinate data

## See Also

- [Variables](variables.md) - Working with time-dependent field data
- [Coordinates](coordinates.md) - Accessing nodal coordinates
- [Geometry](geometry.md) - Element volumes and centroids
