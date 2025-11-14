# Exomerge Python Bindings Review

**Date:** 2025-11-14
**Reviewer:** Claude Code
**File Analyzed:** `./rust/exodus-py/python/exodus/exomerge.py` (5,624 lines)

## Executive Summary

The `exomerge.py` module currently operates as a **pure Python implementation** with minimal use of the exodus-py Rust bindings. While the exodus bindings are imported, they are only used for basic file I/O operations (import_model and export_model). The vast majority of the module's functionality is implemented in pure Python with extensive use of nested loops, list comprehensions, and manual data manipulation. This represents a significant opportunity for optimization by leveraging the existing exodus-py Rust bindings and adding new compute-intensive operations to the Rust layer.

## Current State Analysis

### 1. Exodus Binding Usage

The exodus module is imported on line 25:
```python
from . import exodus
```

However, it's only used in **2 locations** (11 total usages):

1. **`import_model()` method (lines 721-853)**
   - Uses: `ExodusReader`, `EntityType`
   - Opens files and reads basic data (nodes, blocks, QA records)
   - **Incomplete implementation**: Node sets and side sets reading is stubbed out (lines 809-822)

2. **`export_model()` method (lines 855-959)**
   - Uses: `ExodusWriter`, `CreateOptions`, `InitParams`, `CreateMode`, `Block`
   - Writes basic data to Exodus files
   - **Incomplete implementation**: Side sets, node sets, and field data are not being written

### 2. Missing Exodus Binding Integration

The following operations are implemented in pure Python but **should use exodus bindings**:

#### File I/O Operations (High Priority)
- **Node Set Reading/Writing**: Currently stubbed in import_model (lines 809-813), not implemented in export_model
  - Available bindings: `get_node_set()`, `put_node_set()`, `get_node_set_ids()`

- **Side Set Reading/Writing**: Currently stubbed in import_model (lines 816-822), not implemented in export_model
  - Available bindings: `get_side_set()`, `put_side_set()`, `get_side_set_ids()`

- **Variable/Field Data**: Not read or written in import/export
  - Available bindings: `var()`, `put_var()`, `variable_names()`, `define_variables()`
  - Available bindings: `var_multi()`, `put_var_multi()`, `var_time_series()`, `put_var_time_series()`

- **Timestep Data**: Currently creates sequential timesteps (line 833), doesn't read actual values
  - Available bindings: `times()`, `time()`, `put_time()`

- **Block Names**: Not being read or written properly
  - Available bindings: `get_name()`, `put_name()`, `get_names()`, `put_names()`

- **Element Block Attributes**: Not being read or written
  - Available bindings: `get_block_attributes()`, `put_block_attributes()`
  - Available bindings: `get_block_attribute_names()`, `put_block_attribute_names()`

#### Data Structure Operations (Medium Priority)
The exomerge module maintains in-memory data structures as Python lists and dicts:
- `self.nodes`: List of [x, y, z] coordinates (pure Python lists)
- `self.element_blocks`: Dict with connectivity as Python lists
- `self.node_fields`: Dict of field data as Python lists
- `self.global_variables`: Dict of variable data
- `self.side_sets`: Dict of side set data
- `self.node_sets`: Dict of node set data

These could be stored using exodus bindings to reduce memory overhead and improve access patterns.

## 3. Compute-Intensive Operations for Rust Migration

The following operations are implemented in pure Python with nested loops and are **prime candidates for Rust optimization**:

### Critical Performance Issues (O(n²) or worse)

#### A. Node Merging (`merge_nodes()` - lines 4271-4352)
**Current Implementation:**
- O(n²) algorithm comparing all node pairs
- Pure Python distance calculations
- Manual connectivity updates across all element blocks
- Manual node set updates

**Performance Impact:** For a mesh with 100,000 nodes, this requires ~5 billion distance comparisons

**Recommended Rust Implementation:**
```rust
// Use spatial hashing or KD-tree for O(n log n) performance
fn merge_nodes(coords: &[[f64; 3]], tolerance: f64) -> (Vec<[f64; 3]>, HashMap<usize, usize>)
```

#### B. Closest Node Distance (`get_closest_node_distance()` - lines 4354-4377)
**Current Implementation:**
- O(n²) brute force algorithm
- Compares every node pair

**Performance Impact:** Same as merge_nodes - extremely slow for large meshes

**Recommended Rust Implementation:**
```rust
// Use spatial data structure for O(n log n) performance
fn get_closest_node_distance(coords: &[[f64; 3]]) -> f64
```

### Heavy Computation Operations (O(n*m) or O(n*k))

#### C. Element Volume Calculation (`calculate_element_volumes()` - lines 2377-2511)
**Current Implementation:**
- Nested loops over elements and nodes
- Manual vector math for cross products and dot products
- Repeated coordinate lookups
- Python list operations for building coordinate arrays

**Performance Impact:** For 1M elements, involves millions of Python loops and vector operations

**Recommended Rust Implementation:**
```rust
fn calculate_element_volumes(
    connectivity: &[Vec<usize>],
    coords: &[[f64; 3]],
    element_type: &str
) -> Vec<f64>
```

#### D. Element Centroid Calculation (`calculate_element_centroids()` - lines 2311-2375)
**Current Implementation:**
- Triple nested loops (blocks, elements, nodes)
- Manual summation and averaging
- Per-coordinate processing

**Recommended Rust Implementation:**
```rust
fn calculate_element_centroids(
    connectivity: &[Vec<usize>],
    coords: &[[f64; 3]]
) -> Vec<[f64; 3]>
```

#### E. Element Edge Length Analysis (`get_element_edge_length_info()` - lines 2694-2754)
**Current Implementation:**
- Nested loops over blocks, elements, and edges
- Distance calculations for all edges
- Min/average tracking

**Recommended Rust Implementation:**
```rust
fn get_element_edge_lengths(
    connectivity: &[Vec<usize>],
    coords: &[[f64; 3]],
    edge_indices: &[(usize, usize)]
) -> (f64, f64) // (min, avg)
```

### Field Conversion Operations

#### F. Element-to-Node Field Conversion (`convert_element_field_to_node_field()` - lines 3139-3203)
**Current Implementation:**
- Triple nested loops (timesteps, blocks, elements)
- Accumulation arrays for averaging
- Node index lookups

**Performance Impact:** For 10 timesteps × 100k elements × 8 nodes/element = 8M iterations

**Recommended Rust Implementation:**
```rust
fn element_field_to_node_field(
    elem_values: &[Vec<f64>],  // per timestep
    connectivity: &[Vec<usize>],
    num_nodes: usize
) -> Vec<Vec<f64>>  // per timestep
```

#### G. Node-to-Element Field Conversion (`convert_node_field_to_element_field()` - lines 3205-3264)
**Current Implementation:**
- Similar nested loop structure
- Node value averaging per element

**Recommended Rust Implementation:**
```rust
fn node_field_to_element_field(
    node_values: &[Vec<f64>],  // per timestep
    connectivity: &[Vec<usize>]
) -> Vec<Vec<f64>>  // per timestep
```

### Geometric Transformation Operations

#### H. Node Transformations (lines 379-505)
**Current Implementation:**
- `_translate_nodes()`: List comprehensions or loops
- `_scale_nodes()`: List comprehensions or loops
- `_rotate_nodes()`: Complex rotation matrix math in Python loops

**Performance Impact:** For 1M nodes, involves millions of Python arithmetic operations

**Recommended Rust Implementation:**
```rust
fn translate_nodes(coords: &mut [[f64; 3]], offset: [f64; 3])
fn scale_nodes(coords: &mut [[f64; 3]], factor: f64)
fn rotate_nodes(coords: &mut [[f64; 3]], axis: [f64; 3], angle_deg: f64)
```

### Additional Operations

#### I. Duplicate Element Detection (`delete_duplicate_elements()` - lines 2267-2309)
- Nested loops comparing element connectivity
- O(n²) in worst case

#### J. Degenerate Element Counting (`count_degenerate_elements()` - lines 2160-2199)
- Loops through all elements checking for collapsed nodes
- Distance calculations

#### K. Disconnected Block Counting (`count_disconnected_blocks()` - lines 2201-2265)
- Graph traversal implemented in Python
- Node-to-element mapping construction

## 4. Data Structure Optimization Opportunities

### Current In-Memory Representation

All data is stored as Python lists and dicts:
```python
self.nodes = [[x, y, z], [x, y, z], ...]  # List of lists
self.element_blocks[id] = [name, info, connectivity, fields]
self.node_fields = {name: [[values], [values], ...]}  # Per timestep
```

### Memory Issues
1. **Python list overhead**: Each list has object overhead (~56 bytes + pointers)
2. **Fragmented memory**: Lists of lists are not contiguous in memory
3. **Cache inefficiency**: Poor spatial locality for vectorized operations
4. **GC pressure**: Millions of small Python objects

### Recommended Approach

Create Rust-backed data structures that can be shared with Python via:
- NumPy arrays (via PyO3's numpy integration)
- Rust Vec<T> exposed as Python buffers
- Memory-mapped representations for large datasets

Example:
```rust
#[pyclass]
struct MeshData {
    coords: Vec<f64>,  // Flat array: [x1,y1,z1,x2,y2,z2,...]
    connectivity: Vec<i64>,  // Flat connectivity array
    offsets: Vec<usize>,  // Element start offsets
}
```

## 5. API Completeness Issues

### Missing Exodus Binding Features

The exomerge module doesn't expose several features available in exodus-py:

1. **Truth Tables**: For sparse variable storage
   - Available: `truth_table()`, `put_truth_table()`

2. **Reduction Variables**: For summary data
   - Available: `reduction_variable_names()`, `get_reduction_vars()`, `put_reduction_vars()`

3. **Assemblies and Blobs**: For hierarchical mesh organization
   - Available: `get_assembly()`, `put_assembly()`, `get_blob()`, `put_blob()`

4. **ID Maps**: For element/node numbering
   - Available: `get_id_map()`, `put_id_map()`, `get_elem_order_map()`

5. **Properties**: For entity metadata
   - Available: `get_property()`, `put_property()`, `get_property_array()`

6. **Attributes**: For block/set attributes
   - Available: `get_attribute()`, `put_attribute()`, `get_attribute_names()`

## 6. Correctness Issues Found

### Import/Export Data Loss

1. **Node Sets**: Read as empty dict (line 806-813), not written (export_model)
2. **Side Sets**: Read as empty dict (line 816-822), not written (export_model)
3. **Timestep Values**: Created as sequential integers, not actual time values (line 833)
4. **Element Fields**: Not read from file (import_model)
5. **Node Fields**: Not read from file (import_model)
6. **Global Variables**: Not read from file (import_model)
7. **Set Fields**: Not read or written

This means **exomerge currently loses data** when round-tripping files through import/export!

## Recommendations

### Phase 1: Complete Exodus Binding Integration (High Priority)

1. **Fix import_model()** to properly read:
   - Node sets and side sets using `get_node_set_ids()`, `get_node_set()`, etc.
   - Timestep values using `times()` or `time(step)`
   - All variable/field data using `var()`, `var_multi()`, etc.
   - Block names using `get_names()`
   - Element block attributes

2. **Fix export_model()** to properly write:
   - Node sets using `put_node_set()`
   - Side sets using `put_side_set()`
   - Timestep values using `put_time()`
   - All field data using `put_var()`, `put_var_multi()`, etc.
   - Block names using `put_names()`
   - Element block attributes

3. **Add comprehensive round-trip tests** to verify no data loss

### Phase 2: Add Compute Operations to Rust (Medium Priority)

Implement these operations in `exodus-py/src/compute.rs`:

1. **Spatial Operations** (Critical - O(n²) → O(n log n))
   ```rust
   fn merge_nodes(coords, tolerance) -> (new_coords, node_map)
   fn closest_node_distance(coords) -> f64
   fn length_scale(coords) -> f64
   ```

2. **Element Geometry** (High Impact)
   ```rust
   fn calculate_volumes(connectivity, coords, elem_type) -> Vec<f64>
   fn calculate_centroids(connectivity, coords) -> Vec<[f64; 3]>
   fn calculate_edge_lengths(connectivity, coords, edges) -> (f64, f64)
   ```

3. **Field Conversions** (High Impact)
   ```rust
   fn elem_to_node_field(elem_vals, connectivity, num_nodes) -> Vec<f64>
   fn node_to_elem_field(node_vals, connectivity) -> Vec<f64>
   ```

4. **Geometric Transforms** (Medium Impact)
   ```rust
   fn transform_coords(coords, transform_type, params) -> Vec<[f64; 3]>
   // transform_type: translate, scale, rotate, reflect
   ```

5. **Mesh Quality** (Medium Impact)
   ```rust
   fn find_degenerate_elements(connectivity, coords, tolerance) -> Vec<usize>
   fn find_duplicate_elements(connectivity) -> Vec<usize>
   fn find_disconnected_regions(connectivity) -> Vec<Vec<usize>>
   ```

### Phase 3: Memory Optimization (Lower Priority)

1. **Use NumPy arrays** for coordinate storage via PyO3 numpy
2. **Implement zero-copy views** where possible
3. **Add chunked operations** for very large meshes
4. **Consider memory-mapped I/O** for extremely large files

### Phase 4: API Extensions (Future)

Expose currently missing exodus features:
- Truth tables
- Reduction variables
- Assemblies and blobs
- ID maps and properties
- Attributes

## Performance Impact Estimates

Based on typical mesh sizes:

| Operation | Small Mesh (10K nodes) | Large Mesh (1M nodes) | Rust Speedup Estimate |
|-----------|------------------------|----------------------|----------------------|
| merge_nodes | ~0.5s | ~500s (8+ min) | **100-1000x** |
| closest_distance | ~0.3s | ~300s (5 min) | **100-1000x** |
| calculate_volumes | ~0.1s | ~5s | **10-50x** |
| calculate_centroids | ~0.05s | ~3s | **10-50x** |
| field_conversions | ~0.2s | ~10s | **10-50x** |
| transforms | ~0.02s | ~2s | **5-20x** |

**Total potential speedup for typical workflow: 50-500x**

## Code Quality Observations

### Positive Aspects
- Well-documented with docstrings
- Comprehensive API surface
- Good error handling structure
- Maintains backward compatibility with exomerge3.py

### Areas for Improvement
- Heavy reliance on pure Python implementations
- Incomplete file I/O (data loss on round-trip)
- No use of NumPy or other numerical libraries
- No vectorization of operations
- Large file (5,624 lines) - could benefit from modularization

## Testing Recommendations

1. **Add round-trip tests**: Import → modify → export → import → verify
2. **Add performance benchmarks**: Compare Rust vs Python implementations
3. **Add memory profiling**: Track memory usage for large meshes
4. **Add correctness tests**: Verify geometric operations against known results
5. **Add edge case tests**: Empty meshes, single element, degenerate cases

## Conclusion

The exomerge module has significant opportunities for optimization and correctness improvements:

1. **Critical**: Fix data loss in import/export by properly using exodus bindings
2. **High Impact**: Migrate O(n²) spatial operations to Rust (100-1000x speedup)
3. **Medium Impact**: Migrate geometric calculations to Rust (10-50x speedup)
4. **Long Term**: Optimize memory layout and add missing API features

The existing exodus-py Rust bindings already provide most of the necessary foundation - the main work is integrating them properly into exomerge and adding compute-intensive operations to the Rust layer.

**Estimated total development effort:** 4-6 weeks
- Phase 1 (Fix I/O): 1-2 weeks
- Phase 2 (Rust compute): 2-3 weeks
- Phase 3 (Memory optimization): 1 week
- Testing throughout: Ongoing
