# Rexonator Production Readiness Plan

## Progress Checklist

### High Priority
- [x] Refactor `copy_mirror_merge()` into smaller focused functions
- [x] Add memory usage warnings for large mesh operations

### Medium Priority
- [ ] Fix vector component detection false positives
- [x] Reduce excessive cloning in copy_mirror_merge
- [ ] Complete side set side number mapping (TODO in code)
- [ ] Consolidate test helpers with builder pattern

### Low Priority
- [ ] Replace `exit()` with proper error returns in man.rs
- [ ] Remove or implement unused performance config fields
- [ ] Add parallel processing with rayon for large meshes
- [ ] Add benchmarks for performance-critical operations
- [ ] Add progress indicators for verbose mode on large operations

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

### 3. Incomplete Feature with TODO (Medium Priority)

**File:** `copy_mirror_merge.rs:575-577`
```rust
// Side numbers need adjustment based on topology and axis
// For now, keep same side numbers (this is a simplification)
// TODO: Implement proper side number mapping for different topologies
let mirror_sides = sides.clone();
```

**Recommendation:** Either implement proper side mapping or document this limitation clearly in the man page.

### 4. Unused Function Parameter

**File:** `copy_mirror_merge.rs:54`
```rust
fn find_symmetry_plane_nodes(coords: &[f64], _axis: Axis, tolerance: f64) -> Vec<usize> {
```

The `_axis` parameter is unused. This was likely intended for future use but should be removed or documented.

---

## Potential Bugs and Edge Cases

### 1. Vector Component Detection False Positives (Medium Priority)

**File:** `copy_mirror_merge.rs:127-138`

The current logic matches any field ending with `_x`, `_y`, `_z`, etc.:
```rust
fn is_vector_component(name: &str, axis: Axis) -> bool {
    let suffix = match axis {
        Axis::X => ["_x", "x", "_u", "u"],  // Will match "max_x", "suffix_x", etc.
        // ...
    };
    suffix.iter().any(|s| name_lower.ends_with(s) || ...)
}
```

**Problem:** Fields like `"max_x"`, `"index_x"`, or `"matrix"` (ends in `x`) would be incorrectly identified as vector components.

**Recommendation:** Use more specific patterns or a curated list of known vector field prefixes:
```rust
const VECTOR_PREFIXES: &[&str] = &[
    "velocity", "displacement", "force", "momentum", "acceleration",
    "stress", "strain", "flux", "gradient", "normal", "tangent"
];

fn is_vector_component(name: &str, axis: Axis) -> bool {
    let name_lower = name.to_lowercase();

    // Check if it's a known vector field with axis suffix
    let is_known_vector = VECTOR_PREFIXES.iter()
        .any(|prefix| name_lower.starts_with(prefix));

    // Check for standard single-letter velocity components (u, v, w)
    let is_single_letter = name_lower.len() == 1 && matches!(
        (name_lower.as_str(), axis),
        ("u", Axis::X) | ("v", Axis::Y) | ("w", Axis::Z)
    );

    (is_known_vector && matches_axis_suffix(&name_lower, axis)) || is_single_letter
}

fn matches_axis_suffix(name: &str, axis: Axis) -> bool {
    let suffixes = match axis {
        Axis::X => &["_x", "_1"][..],
        Axis::Y => &["_y", "_2"][..],
        Axis::Z => &["_z", "_3"][..],
    };
    suffixes.iter().any(|s| name.ends_with(s))
}
```

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
| **Medium** | Vector component false positives | copy_mirror_merge.rs:127-138 | Low | Pending |
| **Medium** | Excessive cloning | copy_mirror_merge.rs (refactored) | Medium | Complete |
| **Medium** | Complete side set mapping TODO | copy_mirror_merge.rs:575-577 | High | Pending |
| **Medium** | Test helper consolidation | parsers.rs:353-410 | Low | Pending |
| **Low** | Hard exit in man.rs | man.rs:27,35 | Low | Pending |
| **Low** | Unused performance config fields | performance.rs:98-102 | Low | Pending |
| **Low** | Add parallel processing (rayon) | copy_mirror_merge.rs | Medium | Pending |
| **Low** | Add benchmarks | new file | Medium | Pending |
| **Low** | Progress indicators | copy_mirror_merge.rs | Low | Pending |

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
