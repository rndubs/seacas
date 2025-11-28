# Rexonator Production Readiness Plan

## Progress Checklist

### High Priority
- [x] Refactor `copy_mirror_merge()` into smaller focused functions
- [x] Add memory usage warnings for large mesh operations

### Medium Priority
- [x] Fix vector component detection false positives
- [x] Reduce excessive cloning in copy_mirror_merge
- [x] Complete side set side number mapping
- [x] Consolidate test helpers with builder pattern

### Low Priority
- [ ] Replace `exit()` with proper error returns in man.rs [XFAIL TEST: `test_man_page_missing_returns_error`]
- [x] Remove or implement unused performance config fields
- [ ] Add parallel processing with rayon for large meshes [XFAIL TEST: `test_cmm_parallel_processing`]
- [ ] Add benchmarks for performance-critical operations
- [x] Add progress indicators for verbose mode on large operations
- [ ] Preserve 2D mesh dimensionality in CMM [XFAIL TEST: `test_cmm_preserves_2d_dimensionality`]

---

## Integration Test Coverage

Integration tests have been added in `tests/` covering:

### Basic Transformations (`tests/integration_basic.rs`)
- Translation: positive, negative, 3D combined, 2D mesh
- Scaling: up, down, unit conversion
- Mirroring: X, Y, Z axes (upper and lowercase)
- Rotation: single axis, multi-axis, extrinsic/intrinsic
- Field scaling: single, multiple, scientific notation
- Time normalization: zero-time flag
- Operation ordering: translate-then-scale vs scale-then-translate
- Error handling: invalid inputs
- Data preservation: nodes, elements, time steps, variable names

### Copy-Mirror-Merge (`tests/integration_cmm.rs`)
- Basic CMM for all supported element types: HEX8, TET4, WEDGE6, PYRAMID5, QUAD4, TRI3
- Coordinate bounds verification (symmetric after CMM)
- CMM about all three axes (X, Y, Z)
- Node set and side set duplication with `_mirror` suffix
- Element block duplication
- Vector field component negation
- Scalar field preservation
- Custom merge tolerance
- CMM with pre-operations (translate, scale)
- CMM with post-operations
- Time step preservation
- Variable names preservation

### XFail Tests (`tests/xfail_planned.rs`)
These tests document expected behavior for features not yet implemented.
When implementing a feature, remove the `#[ignore]` attribute and verify the test passes.

| Test Name | Planned Feature | Priority | Status |
|-----------|-----------------|----------|--------|
| `test_vector_detection_should_not_match_max_x` | Vector component false positives | Medium | **FIXED** |
| `test_vector_detection_should_not_match_index_x` | Vector component false positives | Medium | **FIXED** |
| `test_vector_detection_velocity_x_is_negated` | Vector component detection | Medium | **FIXED** |
| `test_cmm_side_numbers_properly_mapped` | Side set side number mapping | Medium | **FIXED** |
| `test_cmm_side_numbers_mapped_for_perpendicular_faces` | Side set side number mapping (perpendicular faces) | Medium | **FIXED** |
| `test_cmm_warns_on_large_mesh` | Memory usage warnings | High | Pending |
| `test_man_page_missing_returns_error` | Proper error handling in man.rs | Low | Pending |
| `test_cmm_parallel_processing` | Parallel processing with rayon | Low | Pending |
| `test_verbose_progress_indicators` | Progress indicators | Low | **FIXED** |
| `test_cmm_preserves_2d_dimensionality` | 2D mesh handling | Low | Pending |

### Running Tests

```bash
# Run all integration tests
cargo test --features netcdf4

# Run basic transformation tests only
cargo test --features netcdf4 integration_basic

# Run CMM tests only
cargo test --features netcdf4 integration_cmm

# Run xfail tests (these are ignored by default)
cargo test --features netcdf4 xfail -- --ignored

# Run all tests including ignored
cargo test --features netcdf4 -- --include-ignored
```

### Test Fixtures

Test meshes are created programmatically in `tests/fixtures.rs`:
- `create_quad4_mesh` - 2D 4-element grid
- `create_tri3_mesh` - 2D 2-triangle mesh
- `create_hex8_mesh` - 3D half-symmetry hex mesh with nodal variables
- `create_tet4_mesh` - 3D tetrahedral mesh
- `create_wedge6_mesh` - 3D wedge element
- `create_pyramid5_mesh` - 3D pyramid element
- `create_hex8_with_elem_vars` - HEX8 with element variables
- `create_mesh_with_time_steps` - Multi-timestep mesh for zero-time testing
- `create_simple_cube` - Single HEX8 unit cube
- `create_mesh_with_global_vars` - Mesh with global variables

---

## Detailed Analysis

### Overall Assessment

The rexonator CLI is well-structured with good separation of concerns across modules. The code is readable, has reasonable test coverage, and handles many edge cases. However, there are several opportunities for performance optimization and maintainability improvements that would be valuable before a full production release.

---

## Performance Optimizations

### 1. Memory-Intensive Copy-Mirror-Merge (High Priority)

**File:** `copy_mirror_merge.rs:351-694`

The entire mesh is loaded into memory, duplicated, and written out. For large meshes with many time steps, this is O(2N) memory usage.

**Current approach:**
```rust
// Lines 410-412: Full coordinate cloning
let mut new_x = data.x.clone();
let mut new_y = data.y.clone();
let mut new_z = data.z.clone();
```

**Recommendation:** Consider a streaming approach for very large meshes, or add a memory budget check with warnings:
```rust
let estimated_memory = data.params.num_nodes * data.times.len() * 8 * num_vars;
if estimated_memory > 1_000_000_000 && verbose {
    eprintln!("WARNING: Estimated memory usage: {} GB", estimated_memory / 1_000_000_000);
}
```

### 2. Excessive Cloning in copy_mirror_merge() (Medium Priority)

**File:** `copy_mirror_merge.rs:439-441, 516-517, 556-557`

Multiple large data structures are cloned when they could potentially be moved or built incrementally:
```rust
let mut new_blocks = data.blocks.clone();         // Line 439
let mut new_connectivities = data.connectivities.clone();
let mut new_block_names = data.block_names.clone();
```

**Recommendation:** Use `with_capacity()` and build vectors incrementally instead of cloning and appending.

### 3. Parallel Processing Opportunity (Low Priority)

**File:** `copy_mirror_merge.rs:597-624`

The nodal variable mirroring loops through each variable x time step x node sequentially:
```rust
for (var_idx, var_name) in data.nodal_var_names.iter().enumerate() {
    for step in 0..data.times.len() {
        // ...process each node
    }
}
```

**Recommendation:** Consider using `rayon` for parallel iteration on large meshes with many time steps.

### 4. Unused Performance Configuration Fields

**File:** `performance.rs:98-102`

The `node_chunk_size`, `element_chunk_size`, and `time_chunk_size` fields are stored but only displayed, never actually used for chunking operations:
```rust
node_chunk_size: usize,      // Stored but not used
element_chunk_size: usize,   // Stored but not used
time_chunk_size: usize,      // Stored but not used
```

**Recommendation:** Either implement actual chunking using these values or remove them from the configuration.

---

## Maintainability Issues

### 1. Oversized Function (High Priority)

**File:** `copy_mirror_merge.rs:351-694`

The `copy_mirror_merge()` function is 343 lines long with multiple responsibilities.

**Recommendation:** Break into smaller, focused functions:
```rust
fn create_mirrored_coordinates(
    data: &MeshData,
    axis: Axis,
    symmetry_nodes: &HashSet<usize>
) -> (Vec<f64>, Vec<f64>, Vec<f64>);

fn create_mirrored_blocks(
    data: &MeshData,
    axis: Axis,
    mirror_node_map: &HashMap<usize, i64>
) -> Result<(Vec<Block>, Vec<Vec<i64>>, Vec<String>)>;

fn create_mirrored_node_sets(
    data: &MeshData,
    mirror_node_map: &HashMap<usize, i64>
) -> (Vec<NodeSetData>, Vec<String>);

fn create_mirrored_side_sets(
    data: &MeshData,
    offset: usize
) -> (Vec<SideSetData>, Vec<String>);

fn create_mirrored_nodal_vars(
    data: &MeshData,
    axis: Axis,
    symmetry_nodes: &HashSet<usize>
) -> Vec<Vec<Vec<f64>>>;

fn create_mirrored_elem_vars(
    data: &MeshData,
    axis: Axis
) -> Vec<Vec<Vec<Vec<f64>>>>;
```

### 2. Test Helper Duplication (Medium Priority)

**File:** `parsers.rs:353-410`

Two nearly identical test helpers exist:
```rust
fn make_test_cli(...) -> Cli { ... }           // Lines 353-379
fn make_test_cli_with_cmm(...) -> Cli { ... }  // Lines 382-410
```

**Recommendation:** Use a builder pattern:
```rust
struct TestCliBuilder {
    translate: Vec<String>,
    rotate: Vec<String>,
    scale_len: Vec<f64>,
    mirror: Vec<String>,
    copy_mirror_merge: Vec<String>,
    merge_tolerance: f64,
}

impl TestCliBuilder {
    fn new() -> Self { ... }
    fn translate(mut self, v: Vec<String>) -> Self { self.translate = v; self }
    fn rotate(mut self, v: Vec<String>) -> Self { self.rotate = v; self }
    fn scale_len(mut self, v: Vec<f64>) -> Self { self.scale_len = v; self }
    fn mirror(mut self, v: Vec<String>) -> Self { self.mirror = v; self }
    fn copy_mirror_merge(mut self, v: Vec<String>, tol: f64) -> Self { ... }
    fn build(self) -> Cli { ... }
}

// Usage:
let cli = TestCliBuilder::new()
    .translate(vec!["1,0,0".to_string()])
    .rotate(vec!["Z,90".to_string()])
    .build();
```

### 3. Incomplete Feature with TODO (Medium Priority) - FIXED

**File:** `copy_mirror_merge.rs:378-497`

**Status:** ✅ Fixed with proper side number mapping implementation:

1. **New function `get_side_number_mapping()`:**
   - Returns topology-specific side number mappings for each mirror axis
   - Supports HEX8, TET4, WEDGE6, PYRAMID5, QUAD4, TRI3
   - Handles face swaps for perpendicular faces (e.g., HEX8 X-axis: sides 2↔4)

2. **Updated `create_mirrored_side_sets()`:**
   - Builds element-to-block mapping for topology lookup
   - Applies correct side mapping based on element topology and mirror axis
   - Reports remapped side counts in verbose mode

**Tests:** See `tests/xfail_planned.rs`:
- `test_cmm_side_numbers_properly_mapped` - verifies sides parallel to mirror axis stay unchanged
- `test_cmm_side_numbers_mapped_for_perpendicular_faces` - verifies perpendicular faces are remapped

### 4. Unused Function Parameter - FIXED

**File:** `copy_mirror_merge.rs:306`

The unused `_axis` parameter has been removed from `find_symmetry_plane_nodes()`. The axis selection now happens entirely in the caller via `get_axis_coords()`, making the function signature clearer and more accurate.

---

## Potential Bugs and Edge Cases

### 1. Vector Component Detection False Positives (Medium Priority) - FIXED

**File:** `copy_mirror_merge.rs:17-160`

**Status:** ✅ Fixed with a hybrid approach:

1. **Stricter default heuristics:**
   - Requires underscore separator (`_x`, `_y`, `_z` not just `x`)
   - Excludes known scalar prefixes (`max`, `min`, `index`, `avg`, etc.)
   - Single-letter variables (`u`, `v`, `w`) must be exact matches

2. **User-specified control via CLI options:**
   - `--vector-fields "velocity,displacement"` - explicitly mark base names as vectors
   - `--scalar-fields "flux_x,special_y"` - override detection for specific fields
   - `--no-auto-vector-detection` - disable auto-detection, only use explicit list

**Implementation:** See `VectorDetectionConfig` struct in `copy_mirror_merge.rs:26-160`

**Tests:** See `tests/xfail_planned.rs` for integration tests that verify:
- `max_x`, `index_x` are NOT treated as vector components
- `velocity_x` IS properly negated during CMM

### 2. Hard Exit in man.rs (Low Priority)

**File:** `man.rs:27, 35`
```rust
std::process::exit(1);  // Bypasses normal error handling
```

**Recommendation:** Return proper errors instead:
```rust
if !man_page.exists() {
    return Err(TransformError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!(
            "Man page not found at: {}. Please ensure rexonator.1 is in the same directory as the executable.",
            man_page.display()
        ),
    )));
}

// ...

if !status.success() {
    return Err(TransformError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed to display man page",
    )));
}
```

### 3. Unwrap on Empty Sequence (Low Priority)

**File:** `operations.rs:39`
```rust
let rotation_type = if sequence.chars().next().unwrap().is_uppercase() {
```

While the parser validates this, defensive coding would be safer:
```rust
let rotation_type = sequence.chars().next()
    .map(|c| if c.is_uppercase() { "extrinsic" } else { "intrinsic" })
    .unwrap_or("extrinsic");
```

### 4. 2D Mesh Z-Coordinate Handling

**File:** `copy_mirror_merge.rs:155-160`

The code fills z with zeros for 2D meshes, but then writes 3D coordinates back:
```rust
let z = if coords.z.is_empty() {
    vec![0.0; x.len()]
} else {
    coords.z
};
```

**Concern:** This might change a 2D mesh to 3D unexpectedly. The write logic should respect the original `num_dim`.

**Recommendation:** Track and preserve the original dimensionality:
```rust
// In write_mesh_data, respect original dimensions
let y_opt = if data.params.num_dim >= 2 { Some(&data.y[..]) } else { None };
let z_opt = if data.params.num_dim >= 3 { Some(&data.z[..]) } else { None };
```

---

## Code Quality Improvements

### 1. Add Benchmarks

No benchmarks exist for performance-critical operations. Create `benches/copy_mirror_merge.rs`:
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_large_mesh_mirror(c: &mut Criterion) {
    // Create a test mesh with 100k nodes
    let mesh = create_test_mesh(100_000);

    c.bench_function("mirror_100k_nodes", |b| {
        b.iter(|| copy_mirror_merge(&mesh, Axis::X, 0.001, false))
    });
}

fn bench_nodal_var_mirroring(c: &mut Criterion) {
    let mesh = create_test_mesh_with_vars(10_000, 10, 100); // 10k nodes, 10 vars, 100 timesteps

    c.bench_function("mirror_vars_10k_10v_100t", |b| {
        b.iter(|| copy_mirror_merge(&mesh, Axis::X, 0.001, false))
    });
}

criterion_group!(benches, bench_large_mesh_mirror, bench_nodal_var_mirroring);
criterion_main!(benches);
```

### 2. Progress Indicators for Large Operations

When processing large meshes in verbose mode, add progress updates:
```rust
if verbose && step % 100 == 0 {
    println!("  Processing time step {}/{}", step + 1, total_steps);
}

if verbose && var_idx % 10 == 0 && !data.nodal_var_names.is_empty() {
    println!("  Processing nodal variable {}/{}", var_idx + 1, data.nodal_var_names.len());
}
```

### 3. Consistent Error Context

Some errors are generic while others have good context. Standardize error messages:
```rust
// Good: Specific context
TransformError::InvalidFormat(format!(
    "Unsupported topology '{}' in block {} for copy-mirror-merge",
    block.topology, block_id
))

// Pattern to follow for all errors:
// - What operation failed
// - What entity was involved (block ID, variable name, etc.)
// - What the expected vs actual values were (if applicable)
```

---

## Summary Table

| Priority | Issue | File:Line | Effort | Status |
|----------|-------|-----------|--------|--------|
| **High** | Refactor 343-line function | copy_mirror_merge.rs:861-965 | Medium | Complete |
| **High** | Add memory usage warnings | copy_mirror_merge.rs:17-108 | Low | Complete |
| **Medium** | Vector component false positives | copy_mirror_merge.rs:17-160 | Low | Complete |
| **Medium** | Excessive cloning | copy_mirror_merge.rs (refactored) | Medium | Complete |
| **Medium** | Complete side set mapping TODO | copy_mirror_merge.rs:378-497 | High | Complete |
| **Medium** | Test helper consolidation | parsers.rs:352-496 | Low | Complete |
| **Low** | Hard exit in man.rs | man.rs:27,35 | Low | Pending |
| **Low** | Unused performance config fields | performance.rs:98-102 | Low | Complete |
| **Low** | Add parallel processing (rayon) | copy_mirror_merge.rs | Medium | Pending |
| **Low** | Add benchmarks | new file | Medium | Pending |
| **Low** | Progress indicators | copy_mirror_merge.rs | Low | Complete |

---

## Implementation Order

Recommended order for implementing these changes:

1. **Phase 1 - Critical Refactoring**
   - Break up `copy_mirror_merge()` into smaller functions
   - Fix vector component detection logic
   - Add memory warnings for large meshes

2. **Phase 2 - Code Quality**
   - Replace `exit()` with proper errors in man.rs
   - Consolidate test helpers
   - Remove unused `_axis` parameter

3. **Phase 3 - Performance**
   - Reduce cloning with incremental building
   - Add rayon for parallel processing (optional)
   - Add benchmarks

4. **Phase 4 - Polish**
   - Complete side set mapping or document limitation
   - Add progress indicators
   - Clean up unused performance config fields
