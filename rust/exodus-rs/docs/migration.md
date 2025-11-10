# Migration Guide: From C Exodus II API to exodus-rs

This guide helps developers familiar with the C Exodus II library transition to the Rust exodus-rs library.

## Table of Contents

1. [Key Differences](#key-differences)
2. [Function Mapping](#function-mapping)
3. [Type Conversions](#type-conversions)
4. [Error Handling](#error-handling)
5. [Common Patterns](#common-patterns)
6. [Memory Management](#memory-management)
7. [Examples](#examples)

## Key Differences

### Philosophy

| Aspect | C API | Rust API |
|--------|-------|----------|
| **Error Handling** | Return codes | `Result<T, ExodusError>` |
| **Memory Management** | Manual (malloc/free) | Automatic (RAII) |
| **Null Safety** | Null pointers possible | No null pointers |
| **Type Safety** | Weak (casts common) | Strong (compile-time checks) |
| **File Handles** | Integer IDs | Type-safe structs |
| **Indexing** | 0-based or 1-based | Consistently 1-based for entity IDs |

### Type-State Pattern

The Rust API uses type-state pattern to prevent errors at compile time:

**C:**
```c
int exoid = ex_create("mesh.exo", EX_CLOBBER, ...);
// Can accidentally call read functions
ex_get_coord(exoid, ...);  // Error: file not initialized
```

**Rust:**
```rust
let mut file = ExodusFile::create_default("mesh.exo")?;
// Type system prevents calling read functions
// file.coords()?;  // Compile error: wrong file mode
```

### Builder Pattern

Rust provides a high-level builder API for common operations:

**C:**
```c
ex_init_params params;
params.title = "Mesh";
params.num_dim = 3;
// ... set many fields ...
ex_put_init_ext(exoid, &params);

// Then write coordinates, blocks, etc.
```

**Rust:**
```rust
MeshBuilder::new("Mesh")
    .dimensions(3)
    .coordinates(x, y, z)
    .add_block(...)
    .write("mesh.exo")?;
```

## Function Mapping

### File Operations

| C Function | Rust Equivalent | Notes |
|------------|-----------------|-------|
| `ex_create` | `ExodusFile::create` | Returns `Result<ExodusFile>` |
| `ex_create_int` | `ExodusFile::create` with options | Specify int64 mode in options |
| `ex_open` | `ExodusFile::open` | Returns read-only file |
| `ex_close` | Automatic (Drop) | Or call `file.close()` explicitly |
| `ex_update` | `ExodusFile::append` | Opens in read-write mode |

**Example:**
```rust
// C: int exoid = ex_create("mesh.exo", EX_CLOBBER, &ws, &ws);
let mut file = ExodusFile::create_default("mesh.exo")?;

// C: int exoid = ex_open("mesh.exo", EX_READ, &ws, &ws, &version);
let file = ExodusFile::open("mesh.exo")?;

// C: ex_close(exoid);
// Rust: automatic when file goes out of scope
drop(file);  // Or just let it go out of scope
```

### Initialization

| C Function | Rust Equivalent |
|------------|-----------------|
| `ex_put_init` | `file.init(&params)` |
| `ex_put_init_ext` | `file.init(&params)` |
| `ex_get_init` | `file.init_params()` |
| `ex_get_init_ext` | `file.init_params()` |
| `ex_put_qa` | `file.put_qa_records(&records)` |
| `ex_get_qa` | `file.qa_records()` |
| `ex_put_info` | `file.put_info_records(&records)` |
| `ex_get_info` | `file.info_records()` |

**Example:**
```rust
// C:
// ex_init_params params;
// strcpy(params.title, "My Mesh");
// params.num_dim = 3;
// params.num_nodes = 100;
// ex_put_init_ext(exoid, &params);

// Rust:
let params = InitParams {
    title: "My Mesh".to_string(),
    num_dim: 3,
    num_nodes: 100,
    ..Default::default()
};
file.init(&params)?;
```

### Coordinates

| C Function | Rust Equivalent |
|------------|-----------------|
| `ex_put_coord` | `file.put_coords(&x, Some(&y), Some(&z))` |
| `ex_get_coord` | `file.coords::<f64>()` |
| `ex_put_partial_coord` | Not yet implemented |
| `ex_get_partial_coord` | Not yet implemented |
| `ex_put_coord_names` | `file.put_coord_names(&names)` |
| `ex_get_coord_names` | `file.coord_names()` |

**Example:**
```rust
// C:
// double *x = malloc(num_nodes * sizeof(double));
// double *y = malloc(num_nodes * sizeof(double));
// double *z = malloc(num_nodes * sizeof(double));
// // Fill arrays...
// ex_put_coord(exoid, x, y, z);
// free(x); free(y); free(z);

// Rust:
let x: Vec<f64> = (0..num_nodes).map(|i| i as f64).collect();
let y: Vec<f64> = (0..num_nodes).map(|i| i as f64 * 2.0).collect();
let z: Vec<f64> = vec![0.0; num_nodes];
file.put_coords(&x, Some(&y), Some(&z))?;
// Memory automatically freed when vectors go out of scope
```

### Element Blocks

| C Function | Rust Equivalent |
|------------|-----------------|
| `ex_put_block` | `file.put_block(&block)` |
| `ex_get_block` | `file.block(entity_type, id)` |
| `ex_put_conn` | `file.put_connectivity(id, &conn)` |
| `ex_get_conn` | `file.connectivity(id)` |
| `ex_get_ids` | `file.elem_block_ids()` |
| `ex_put_attr` | `file.put_block_attributes(id, &attrs)` |
| `ex_get_attr` | `file.block_attributes(id)` |

**Example:**
```rust
// C:
// ex_put_block(exoid, EX_ELEM_BLOCK, 1, "HEX8", 100, 8, 0, 0, 0);
// int *conn = malloc(100 * 8 * sizeof(int));
// // Fill connectivity...
// ex_put_conn(exoid, EX_ELEM_BLOCK, 1, conn, NULL, NULL);
// free(conn);

// Rust:
let block = Block {
    id: 1,
    entity_type: EntityType::ElemBlock,
    topology: "HEX8".to_string(),
    num_entries: 100,
    num_nodes_per_entry: 8,
    num_edges_per_entry: 0,
    num_faces_per_entry: 0,
    num_attributes: 0,
};
file.put_block(&block)?;

let connectivity: Vec<i64> = /* ... */;
file.put_connectivity(1, &connectivity)?;
```

### Sets

| C Function | Rust Equivalent |
|------------|-----------------|
| `ex_put_set_param` | `file.put_set(&set)` |
| `ex_get_set_param` | `file.set(entity_type, id)` |
| `ex_put_set` | `file.put_set_params(id, type, &nodes, dist_factors)` |
| `ex_get_set` | `file.set(entity_type, id)` |
| `ex_get_ids` | `file.set_ids(entity_type)` |

**Example:**
```rust
// C:
// ex_put_set_param(exoid, EX_NODE_SET, 1, 4, 0);
// int nodes[4] = {1, 2, 3, 4};
// ex_put_set(exoid, EX_NODE_SET, 1, nodes, NULL);

// Rust:
let set = Set {
    id: 1,
    entity_type: EntityType::NodeSet,
    num_entries: 4,
    num_dist_factors: 0,
};
file.put_set(&set)?;
file.put_set_params(1, EntityType::NodeSet, &[1, 2, 3, 4], None)?;
```

### Variables

| C Function | Rust Equivalent |
|------------|-----------------|
| `ex_put_variable_param` | `file.define_variables(type, &names)` |
| `ex_get_variable_param` | `file.variable_names(type)` |
| `ex_put_variable_names` | Implicit in `define_variables` |
| `ex_get_variable_names` | `file.variable_names(type)` |
| `ex_put_var` | `file.put_var(step, type, entity_id, var_idx, &values)` |
| `ex_get_var` | `file.var(step, type, entity_id, var_idx)` |
| `ex_put_time` | `file.put_time(step, time)` |
| `ex_get_time` | `file.times()` or `file.times()? [step-1]` |

**Example:**
```rust
// C:
// int num_vars = 2;
// ex_put_variable_param(exoid, EX_NODAL, num_vars);
// char *names[] = {"temperature", "pressure"};
// ex_put_variable_names(exoid, EX_NODAL, num_vars, names);
//
// double *values = malloc(num_nodes * sizeof(double));
// // Fill values...
// ex_put_var(exoid, 1, EX_NODAL, 1, 1, num_nodes, values);
// free(values);

// Rust:
file.define_variables(EntityType::Nodal, &["temperature", "pressure"])?;

let values: Vec<f64> = /* ... */;
file.put_time(1, 0.0)?;
file.put_var(1, EntityType::Nodal, 0, 0, &values)?;
// Memory automatically managed
```

### Truth Tables

| C Function | Rust Equivalent |
|------------|-----------------|
| `ex_put_truth_table` | `file.put_truth_table(type, &table)` |
| `ex_get_truth_table` | `file.truth_table(type)` |

**Example:**
```rust
// C:
// int truth_table[num_blocks * num_vars];
// // Fill truth table...
// ex_put_truth_table(exoid, EX_ELEM_BLOCK, num_blocks, num_vars, truth_table);

// Rust:
let truth_table: Vec<i32> = /* ... */;
file.put_truth_table(EntityType::ElemBlock, &truth_table)?;
```

## Type Conversions

### Integer Types

| C Type | Rust Type | Notes |
|--------|-----------|-------|
| `int` (entity ID) | `i64` (EntityId) | Always 64-bit in Rust |
| `int` (count) | `usize` | For array sizes |
| `int` (index) | `usize` | For array indexing |
| `int64_t` | `i64` | Direct mapping |

### Float Types

| C Type | Rust Type | Notes |
|--------|-----------|-------|
| `float` | `f32` | Single precision |
| `double` | `f64` | Double precision |

Rust API is generic over float types:
```rust
// Write as f32
file.put_coords(&x_f32, Some(&y_f32), Some(&z_f32))?;

// Read as f64 (automatic conversion)
let coords = file.coords::<f64>()?;
```

### String Types

| C Type | Rust Type | Conversion |
|--------|-----------|------------|
| `char *` | `String` | Owned string |
| `const char *` | `&str` | String slice |
| `char[MAX_LEN]` | `String` | Convert with `to_string()` |

```rust
// C: char title[MAX_STR_LENGTH+1];
// Rust:
let title: String = "My Mesh".to_string();

// C: const char *name = "temperature";
// Rust:
let name: &str = "temperature";
```

### Entity Type Constants

| C Constant | Rust Enum |
|------------|-----------|
| `EX_NODAL` | `EntityType::Nodal` |
| `EX_ELEM_BLOCK` | `EntityType::ElemBlock` |
| `EX_NODE_SET` | `EntityType::NodeSet` |
| `EX_SIDE_SET` | `EntityType::SideSet` |
| `EX_EDGE_BLOCK` | `EntityType::EdgeBlock` |
| `EX_FACE_BLOCK` | `EntityType::FaceBlock` |
| `EX_EDGE_SET` | `EntityType::EdgeSet` |
| `EX_FACE_SET` | `EntityType::FaceSet` |
| `EX_ELEM_SET` | `EntityType::ElemSet` |
| `EX_GLOBAL` | `EntityType::Global` |

## Error Handling

### C API

```c
int status = ex_put_coord(exoid, x, y, z);
if (status < 0) {
    fprintf(stderr, "Error: %s\n", ex_error_msg);
    // Manual cleanup
    free(x);
    free(y);
    free(z);
    ex_close(exoid);
    return -1;
}
```

### Rust API

```rust
// Result type with ? operator
file.put_coords(&x, Some(&y), Some(&z))?;
// Automatic cleanup on error
// File closed automatically
// Memory freed automatically

// Or handle explicitly
match file.put_coords(&x, Some(&y), Some(&z)) {
    Ok(()) => println!("Success"),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Error Types

All errors are instances of `ExodusError`:

```rust
use exodus_rs::ExodusError;

match file.block(EntityType::ElemBlock, 999) {
    Ok(block) => println!("Found block"),
    Err(ExodusError::EntityNotFound { entity_type, id }) => {
        eprintln!("Block {} not found", id);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Memory Management

### C: Manual Management

```c
// Allocate
double *coords = malloc(num_nodes * sizeof(double));
if (coords == NULL) {
    // Handle error
    return -1;
}

// Use
ex_get_coord(exoid, coords, NULL, NULL);

// Must manually free
free(coords);
```

### Rust: Automatic Management

```rust
// Allocate (automatic)
let coords = file.coords::<f64>()?;

// Use
println!("First coord: ({}, {})", coords.x[0], coords.y[0]);

// Automatically freed when coords goes out of scope
```

### Rust: Ownership

```rust
fn process_mesh(file: ExodusFile) -> Result<(), ExodusError> {
    // file is moved here, caller can't use it anymore
    let coords = file.coords::<f64>()?;
    Ok(())
    // file automatically closed here
}

// Alternative: borrow instead of move
fn process_mesh_ref(file: &ExodusFile) -> Result<(), ExodusError> {
    // file is borrowed, caller can still use it
    let coords = file.coords::<f64>()?;
    Ok(())
    // file NOT closed here, caller still owns it
}
```

## Common Patterns

### Pattern 1: Create and Write Mesh

**C:**
```c
int exoid = ex_create("mesh.exo", EX_CLOBBER, &ws, &ws);

ex_init_params params = {0};
strcpy(params.title, "Mesh");
params.num_dim = 3;
params.num_nodes = 100;
params.num_elem = 50;
params.num_elem_blk = 1;
ex_put_init_ext(exoid, &params);

double *x = malloc(100 * sizeof(double));
double *y = malloc(100 * sizeof(double));
double *z = malloc(100 * sizeof(double));
// Fill coordinates...
ex_put_coord(exoid, x, y, z);
free(x); free(y); free(z);

ex_put_block(exoid, EX_ELEM_BLOCK, 1, "HEX8", 50, 8, 0, 0, 0);

int *conn = malloc(50 * 8 * sizeof(int));
// Fill connectivity...
ex_put_conn(exoid, EX_ELEM_BLOCK, 1, conn, NULL, NULL);
free(conn);

ex_close(exoid);
```

**Rust:**
```rust
let mut file = ExodusFile::create_default("mesh.exo")?;

let params = InitParams {
    title: "Mesh".to_string(),
    num_dim: 3,
    num_nodes: 100,
    num_elems: 50,
    num_elem_blocks: 1,
    ..Default::default()
};
file.init(&params)?;

let x: Vec<f64> = /* ... */;
let y: Vec<f64> = /* ... */;
let z: Vec<f64> = /* ... */;
file.put_coords(&x, Some(&y), Some(&z))?;

let block = Block {
    id: 1,
    entity_type: EntityType::ElemBlock,
    topology: "HEX8".to_string(),
    num_entries: 50,
    num_nodes_per_entry: 8,
    num_edges_per_entry: 0,
    num_faces_per_entry: 0,
    num_attributes: 0,
};
file.put_block(&block)?;

let connectivity: Vec<i64> = /* ... */;
file.put_connectivity(1, &connectivity)?;

// File automatically closed
```

### Pattern 2: Read and Process Mesh

**C:**
```c
int exoid = ex_open("mesh.exo", EX_READ, &ws, &ws, &version);

ex_init_params params;
ex_get_init_ext(exoid, &params);

double *x = malloc(params.num_nodes * sizeof(double));
double *y = malloc(params.num_nodes * sizeof(double));
double *z = malloc(params.num_nodes * sizeof(double));
ex_get_coord(exoid, x, y, z);

// Process coordinates...
for (int i = 0; i < params.num_nodes; i++) {
    printf("Node %d: (%f, %f, %f)\n", i+1, x[i], y[i], z[i]);
}

free(x); free(y); free(z);
ex_close(exoid);
```

**Rust:**
```rust
let file = ExodusFile::open("mesh.exo")?;

let params = file.init_params()?;
let coords = file.coords::<f64>()?;

// Process coordinates...
for i in 0..params.num_nodes {
    let coord = coords.get(i).unwrap();
    println!("Node {}: {:?}", i+1, coord);
}

// Memory and file automatically cleaned up
```

### Pattern 3: Write Time-Dependent Variables

**C:**
```c
int num_vars = 1;
ex_put_variable_param(exoid, EX_NODAL, num_vars);

char *var_names[] = {"temperature"};
ex_put_variable_names(exoid, EX_NODAL, num_vars, var_names);

double *values = malloc(num_nodes * sizeof(double));
for (int step = 1; step <= 10; step++) {
    double time = step * 0.1;
    ex_put_time(exoid, step, &time);

    // Fill values...
    for (int i = 0; i < num_nodes; i++) {
        values[i] = 20.0 + i + step;
    }
    ex_put_var(exoid, step, EX_NODAL, 1, 1, num_nodes, values);
}
free(values);
```

**Rust:**
```rust
file.define_variables(EntityType::Nodal, &["temperature"])?;

for step in 1..=10 {
    let time = step as f64 * 0.1;
    file.put_time(step, time)?;

    let values: Vec<f64> = (0..num_nodes)
        .map(|i| 20.0 + i as f64 + step as f64)
        .collect();
    file.put_var(step, EntityType::Nodal, 0, 0, &values)?;
}
```

## Examples

### Complete Migration Example

See the [examples/](../examples/) directory for complete working examples:

- `examples/01_create_file.rs` - Basic file creation
- `examples/02_initialize.rs` - Database initialization
- `examples/03_coordinates.rs` - Coordinate operations
- `examples/04_element_blocks.rs` - Element block management
- `examples/05_sets.rs` - Node/side/element sets
- `examples/06_variables.rs` - Time-dependent variables
- `examples/09_high_level_builder.rs` - High-level builder API

### Performance Comparison

In general, Rust exodus-rs performs comparably to the C library:

- **File I/O**: Similar (both use NetCDF backend)
- **Memory Usage**: Often lower (no over-allocation, efficient collections)
- **Safety**: Much higher (no segfaults, no memory leaks)
- **Development Speed**: Faster (better error messages, no manual memory management)

## Additional Resources

- [User Guide](guide.md) - Complete guide to exodus-rs
- [Cookbook](cookbook.md) - Common recipes and patterns
- [API Documentation](https://docs.rs/exodus-rs) - Detailed API reference
- [Exodus II Specification](https://sandialabs.github.io/seacas-docs/exodusII-new.pdf)
- [C API Reference](https://sandialabs.github.io/seacas-docs/index.html)

## Getting Help

If you have questions about migration:

1. Check the [examples/](../examples/) directory
2. Read the [API documentation](https://docs.rs/exodus-rs)
3. File an issue on [GitHub](https://github.com/sandialabs/seacas/issues)
