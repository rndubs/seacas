# Exodus-RS Cookbook

Common recipes and patterns for working with exodus-rs.

## Table of Contents

1. [File Operations](#file-operations)
2. [Mesh Creation](#mesh-creation)
3. [Reading Meshes](#reading-meshes)
4. [Variables and Results](#variables-and-results)
5. [Sets and Boundary Conditions](#sets-and-boundary-conditions)
6. [Mesh Manipulation](#mesh-manipulation)
7. [Performance Optimization](#performance-optimization)
8. [Integration](#integration)

## File Operations

### Recipe: Create File with Compression

```rust
use exodus_rs::{CreateOptions, ExodusFile, FileFormat};

let options = CreateOptions {
    format: FileFormat::NetCDF4,
    compression: Some(6),  // Compression level 1-9
    ..Default::default()
};

let file = ExodusFile::create("compressed.exo", options)?;
```

### Recipe: Check if File Exists Before Creating

```rust
use std::path::Path;
use exodus_rs::*;

fn create_if_not_exists(path: &str) -> Result<ExodusFile, ExodusError> {
    if Path::new(path).exists() {
        eprintln!("File {} already exists", path);
        return Err(ExodusError::Other("File exists".to_string()));
    }
    ExodusFile::create_default(path)
}
```

### Recipe: Safely Update Existing File

```rust
use exodus_rs::ExodusFile;

// Open in append mode
let mut file = ExodusFile::append("existing.exo")?;

// Read existing data
let params = file.init_params()?;
let num_steps = file.num_times()?;

// Add new time step
file.put_time(num_steps + 1, 1.0)?;
// ... write new variables
```

### Recipe: Copy File with Modifications

```rust
fn copy_and_modify(input: &str, output: &str) -> Result<(), ExodusError> {
    // Read input
    let input_file = ExodusFile::open(input)?;
    let params = input_file.init_params()?;
    let coords = input_file.coords::<f64>()?;

    // Create output
    let mut output_file = ExodusFile::create_default(output)?;
    output_file.init(&params)?;

    // Modify coordinates (e.g., scale by 2)
    let x_scaled: Vec<f64> = coords.x.iter().map(|&v| v * 2.0).collect();
    let y_scaled: Vec<f64> = coords.y.iter().map(|&v| v * 2.0).collect();
    let z_scaled: Vec<f64> = coords.z.iter().map(|&v| v * 2.0).collect();

    output_file.put_coords(&x_scaled, Some(&y_scaled), Some(&z_scaled))?;

    // Copy blocks, sets, etc...
    Ok(())
}
```

## Mesh Creation

### Recipe: Create 2D Structured Quad Mesh

```rust
use exodus_rs::*;

fn create_quad_mesh(nx: usize, ny: usize, lx: f64, ly: f64) -> Result<(), ExodusError> {
    let mut file = ExodusFile::create_default("quad_mesh.exo")?;

    let num_nodes = (nx + 1) * (ny + 1);
    let num_elems = nx * ny;

    let params = InitParams {
        title: format!("{}x{} Quad Mesh", nx, ny),
        num_dim: 2,
        num_nodes,
        num_elems,
        num_elem_blocks: 1,
        ..Default::default()
    };
    file.init(&params)?;

    // Generate coordinates
    let dx = lx / nx as f64;
    let dy = ly / ny as f64;

    let mut x = Vec::new();
    let mut y = Vec::new();

    for j in 0..=ny {
        for i in 0..=nx {
            x.push(i as f64 * dx);
            y.push(j as f64 * dy);
        }
    }

    file.put_coords(&x, Some(&y), None)?;

    // Generate connectivity
    let mut connectivity = Vec::new();
    for j in 0..ny {
        for i in 0..nx {
            let n0 = (j * (nx + 1) + i) as i64 + 1;
            let n1 = n0 + 1;
            let n2 = n1 + (nx + 1) as i64;
            let n3 = n0 + (nx + 1) as i64;
            connectivity.extend_from_slice(&[n0, n1, n2, n3]);
        }
    }

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".to_string(),
        num_entries: num_elems,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;

    Ok(())
}

// Usage:
create_quad_mesh(10, 10, 1.0, 1.0)?;  // 10x10 mesh, 1x1 domain
```

### Recipe: Create 3D Structured Hex Mesh

```rust
fn create_hex_mesh(nx: usize, ny: usize, nz: usize) -> Result<(), ExodusError> {
    let mut file = ExodusFile::create_default("hex_mesh.exo")?;

    let num_nodes = (nx + 1) * (ny + 1) * (nz + 1);
    let num_elems = nx * ny * nz;

    let params = InitParams {
        title: format!("{}x{}x{} Hex Mesh", nx, ny, nz),
        num_dim: 3,
        num_nodes,
        num_elems,
        num_elem_blocks: 1,
        ..Default::default()
    };
    file.init(&params)?;

    // Generate coordinates
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut z = Vec::new();

    for k in 0..=nz {
        for j in 0..=ny {
            for i in 0..=nx {
                x.push(i as f64);
                y.push(j as f64);
                z.push(k as f64);
            }
        }
    }

    file.put_coords(&x, Some(&y), Some(&z))?;

    // Generate connectivity (HEX8)
    let mut connectivity = Vec::new();
    let nx1 = (nx + 1) as i64;
    let ny1 = (ny + 1) as i64;

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let base = (k * ny1 * nx1 + j * nx1 + i) as i64 + 1;
                connectivity.extend_from_slice(&[
                    base,
                    base + 1,
                    base + nx1 + 1,
                    base + nx1,
                    base + ny1 * nx1,
                    base + ny1 * nx1 + 1,
                    base + ny1 * nx1 + nx1 + 1,
                    base + ny1 * nx1 + nx1,
                ]);
            }
        }
    }

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "HEX8".to_string(),
        num_entries: num_elems,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;

    Ok(())
}
```

### Recipe: Create Mixed Element Mesh

```rust
fn create_mixed_mesh() -> Result<(), ExodusError> {
    let mut file = ExodusFile::create_default("mixed_mesh.exo")?;

    let params = InitParams {
        title: "Mixed Element Mesh".to_string(),
        num_dim: 2,
        num_nodes: 6,
        num_elems: 3,
        num_elem_blocks: 2,  // One for tris, one for quads
        ..Default::default()
    };
    file.init(&params)?;

    // Coordinates
    let x = vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    file.put_coords(&x, Some(&y), None)?;

    // Triangle block
    let tri_block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "TRI3".to_string(),
        num_entries: 2,
        num_nodes_per_entry: 3,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&tri_block)?;
    file.put_connectivity(1, &vec![1, 2, 4, 2, 5, 4])?;

    // Quad block
    let quad_block = Block {
        id: 2,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".to_string(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&quad_block)?;
    file.put_connectivity(2, &vec![2, 3, 6, 5])?;

    Ok(())
}
```

### Recipe: Using High-Level Builder API

```rust
use exodus_rs::{MeshBuilder, BlockBuilder, Topology};

MeshBuilder::new("Simple Mesh")
    .dimensions(3)
    .coordinates(
        vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
    )
    .add_block(
        BlockBuilder::new(1, Topology::Hex8)
            .connectivity(vec![1, 2, 3, 4, 5, 6, 7, 8])
            .build()
    )
    .qa_record("MyApp", "1.0", "2025-11-10", "12:00:00")
    .info("Generated automatically")
    .write("mesh.exo")?;
```

## Reading Meshes

### Recipe: Print Mesh Summary

```rust
fn print_mesh_summary(filename: &str) -> Result<(), ExodusError> {
    let file = ExodusFile::open(filename)?;

    let params = file.init_params()?;
    println!("Title: {}", params.title);
    println!("Dimensions: {}", params.num_dim);
    println!("Nodes: {}", params.num_nodes);
    println!("Elements: {}", params.num_elems);
    println!("Element Blocks: {}", params.num_elem_blocks);

    // Print block info
    let block_ids = file.elem_block_ids()?;
    for block_id in block_ids {
        let block = file.block(EntityType::ElemBlock, block_id)?;
        println!("  Block {}: {} {} elements",
            block.id,
            block.num_entries,
            block.topology
        );
    }

    // Print variable info
    if let Ok(var_names) = file.variable_names(EntityType::Nodal) {
        println!("Nodal variables: {}", var_names.len());
        for (i, name) in var_names.iter().enumerate() {
            println!("  {}: {}", i, name);
        }
    }

    // Print time steps
    if let Ok(times) = file.times() {
        println!("Time steps: {}", times.len());
        if !times.is_empty() {
            println!("  First: {}, Last: {}", times[0], times[times.len()-1]);
        }
    }

    Ok(())
}
```

### Recipe: Extract Boundary Nodes

```rust
fn extract_boundary_nodes(file: &ExodusFile) -> Result<Vec<i64>, ExodusError> {
    let coords = file.coords::<f64>()?;
    let params = file.init_params()?;

    // Find nodes on boundaries (simple box mesh example)
    let mut boundary_nodes = Vec::new();

    // Assuming box mesh: 0 <= x,y,z <= 1
    const TOL: f64 = 1e-10;

    for i in 0..params.num_nodes {
        let x = coords.x[i];
        let y = coords.y[i];
        let z = coords.z.get(i).copied().unwrap_or(0.0);

        // Check if on any face
        if x.abs() < TOL || (x - 1.0).abs() < TOL ||
           y.abs() < TOL || (y - 1.0).abs() < TOL ||
           z.abs() < TOL || (z - 1.0).abs() < TOL
        {
            boundary_nodes.push((i + 1) as i64);  // 1-based
        }
    }

    Ok(boundary_nodes)
}
```

### Recipe: Find Elements Containing a Point

```rust
fn find_elements_containing_point(
    file: &ExodusFile,
    point: [f64; 3]
) -> Result<Vec<i64>, ExodusError> {
    let coords = file.coords::<f64>()?;
    let block_ids = file.elem_block_ids()?;

    let mut containing_elems = Vec::new();

    for block_id in block_ids {
        let block = file.block(EntityType::ElemBlock, block_id)?;
        let connectivity = file.connectivity(block_id)?;

        let num_elems = block.num_entries;
        let nodes_per_elem = block.num_nodes_per_entry;

        for elem_idx in 0..num_elems {
            // Get element nodes
            let start = elem_idx * nodes_per_elem;
            let elem_nodes = &connectivity.nodes[start..start + nodes_per_elem];

            // Compute element bounding box
            let mut min = [f64::MAX; 3];
            let mut max = [f64::MIN; 3];

            for &node_id in elem_nodes {
                let idx = (node_id - 1) as usize;  // Convert to 0-based
                min[0] = min[0].min(coords.x[idx]);
                max[0] = max[0].max(coords.x[idx]);
                min[1] = min[1].min(coords.y[idx]);
                max[1] = max[1].max(coords.y[idx]);
                if coords.num_dim >= 3 {
                    min[2] = min[2].min(coords.z[idx]);
                    max[2] = max[2].max(coords.z[idx]);
                }
            }

            // Check if point is in bounding box
            if point[0] >= min[0] && point[0] <= max[0] &&
               point[1] >= min[1] && point[1] <= max[1] &&
               point[2] >= min[2] && point[2] <= max[2]
            {
                containing_elems.push((elem_idx + 1) as i64);
            }
        }
    }

    Ok(containing_elems)
}
```

## Variables and Results

### Recipe: Write Time-Dependent Solution

```rust
fn write_heat_diffusion_results(
    file: &mut ExodusFile,
    num_steps: usize,
    dt: f64,
) -> Result<(), ExodusError> {
    let params = file.init_params()?;
    let num_nodes = params.num_nodes;

    // Define variables
    file.define_variables(EntityType::Nodal, &["temperature"])?;
    file.define_variables(EntityType::Global, &["max_temp", "min_temp"])?;

    for step in 0..num_steps {
        let time = step as f64 * dt;
        file.put_time(step + 1, time)?;

        // Compute temperature field (example: heat diffusion)
        let temps: Vec<f64> = (0..num_nodes)
            .map(|i| {
                let phase = 2.0 * std::f64::consts::PI * i as f64 / num_nodes as f64;
                20.0 + 10.0 * (phase + 0.1 * time).sin()
            })
            .collect();

        // Write nodal temperature
        file.put_var(step + 1, EntityType::Nodal, 0, 0, &temps)?;

        // Write global statistics
        let max_temp = temps.iter().copied().fold(f64::MIN, f64::max);
        let min_temp = temps.iter().copied().fold(f64::MAX, f64::min);

        file.put_var(step + 1, EntityType::Global, 0, 0, &[max_temp])?;
        file.put_var(step + 1, EntityType::Global, 0, 1, &[min_temp])?;
    }

    Ok(())
}
```

### Recipe: Read and Plot Time History

```rust
fn extract_time_history(
    filename: &str,
    node_id: i64,
    var_name: &str,
) -> Result<Vec<(f64, f64)>, ExodusError> {
    let file = ExodusFile::open(filename)?;

    // Find variable index
    let var_names = file.variable_names(EntityType::Nodal)?;
    let var_idx = var_names.iter()
        .position(|name| name == var_name)
        .ok_or_else(|| ExodusError::Other(format!("Variable {} not found", var_name)))?;

    // Get time values
    let times = file.times()?;
    let num_steps = times.len();

    // Extract values
    let mut history = Vec::new();
    for step in 0..num_steps {
        let values = file.var(step + 1, EntityType::Nodal, 0, var_idx)?;
        let node_value = values[(node_id - 1) as usize];
        history.push((times[step], node_value));
    }

    Ok(history)
}
```

### Recipe: Compute Derived Quantities

```rust
fn compute_von_mises_stress(
    file: &ExodusFile,
    step: usize,
) -> Result<Vec<f64>, ExodusError> {
    // Read stress components
    let var_names = file.variable_names(EntityType::ElemBlock)?;

    let sx_idx = var_names.iter().position(|n| n == "stress_xx").unwrap();
    let sy_idx = var_names.iter().position(|n| n == "stress_yy").unwrap();
    let sz_idx = var_names.iter().position(|n| n == "stress_zz").unwrap();
    let sxy_idx = var_names.iter().position(|n| n == "stress_xy").unwrap();
    let syz_idx = var_names.iter().position(|n| n == "stress_yz").unwrap();
    let sxz_idx = var_names.iter().position(|n| n == "stress_xz").unwrap();

    let sx = file.var(step, EntityType::ElemBlock, 0, sx_idx)?;
    let sy = file.var(step, EntityType::ElemBlock, 0, sy_idx)?;
    let sz = file.var(step, EntityType::ElemBlock, 0, sz_idx)?;
    let sxy = file.var(step, EntityType::ElemBlock, 0, sxy_idx)?;
    let syz = file.var(step, EntityType::ElemBlock, 0, syz_idx)?;
    let sxz = file.var(step, EntityType::ElemBlock, 0, sxz_idx)?;

    // Compute von Mises stress
    let von_mises: Vec<f64> = (0..sx.len())
        .map(|i| {
            let s1 = sx[i] - sy[i];
            let s2 = sy[i] - sz[i];
            let s3 = sz[i] - sx[i];
            ((s1*s1 + s2*s2 + s3*s3 + 6.0*(sxy[i]*sxy[i] + syz[i]*syz[i] + sxz[i]*sxz[i])) / 2.0).sqrt()
        })
        .collect();

    Ok(von_mises)
}
```

## Sets and Boundary Conditions

### Recipe: Create Boundary Sets for Box Mesh

```rust
fn create_box_boundary_sets(
    file: &mut ExodusFile
) -> Result<(), ExodusError> {
    let coords = file.coords::<f64>()?;
    let params = file.init_params()?;
    const TOL: f64 = 1e-10;

    // Find nodes on each face
    let mut x0_nodes = Vec::new();  // x = 0
    let mut x1_nodes = Vec::new();  // x = 1
    let mut y0_nodes = Vec::new();  // y = 0
    let mut y1_nodes = Vec::new();  // y = 1
    let mut z0_nodes = Vec::new();  // z = 0
    let mut z1_nodes = Vec::new();  // z = 1

    for i in 0..params.num_nodes {
        let node_id = (i + 1) as i64;
        let x = coords.x[i];
        let y = coords.y[i];
        let z = coords.z.get(i).copied().unwrap_or(0.0);

        if x.abs() < TOL { x0_nodes.push(node_id); }
        if (x - 1.0).abs() < TOL { x1_nodes.push(node_id); }
        if y.abs() < TOL { y0_nodes.push(node_id); }
        if (y - 1.0).abs() < TOL { y1_nodes.push(node_id); }
        if z.abs() < TOL { z0_nodes.push(node_id); }
        if (z - 1.0).abs() < TOL { z1_nodes.push(node_id); }
    }

    // Create node sets
    let sets = vec![
        (1, "x0", x0_nodes),
        (2, "x1", x1_nodes),
        (3, "y0", y0_nodes),
        (4, "y1", y1_nodes),
        (5, "z0", z0_nodes),
        (6, "z1", z1_nodes),
    ];

    for (set_id, name, nodes) in sets {
        if !nodes.is_empty() {
            let set = Set {
                id: set_id,
                entity_type: EntityType::NodeSet,
                num_entries: nodes.len(),
                num_dist_factors: 0,
            };
            file.put_set(&set)?;
            file.put_set_params(set_id, EntityType::NodeSet, &nodes, None)?;
            file.put_name(EntityType::NodeSet, set_id, name)?;
        }
    }

    Ok(())
}
```

### Recipe: Create Side Set for Pressure Load

```rust
fn create_pressure_surface(
    file: &mut ExodusFile,
    surface_normal: [f64; 3],
) -> Result<(), ExodusError> {
    // Find elements and sides on the surface
    let block_ids = file.elem_block_ids()?;
    let coords = file.coords::<f64>()?;

    let mut elem_list = Vec::new();
    let mut side_list = Vec::new();

    for block_id in block_ids {
        let block = file.block(EntityType::ElemBlock, block_id)?;
        let connectivity = file.connectivity(block_id)?;

        // For each element, check each side
        for elem_idx in 0..block.num_entries {
            let start = elem_idx * block.num_nodes_per_entry;
            let elem_nodes = &connectivity.nodes[start..start + block.num_nodes_per_entry];

            // Check if any side matches the surface
            // (Simplified: just check first side)
            if is_side_on_surface(&coords, elem_nodes, &surface_normal) {
                elem_list.push((elem_idx + 1) as i64);
                side_list.push(1);  // Side 1 (simplified)
            }
        }
    }

    // Create side set
    if !elem_list.is_empty() {
        let set = Set {
            id: 1,
            entity_type: EntityType::SideSet,
            num_entries: elem_list.len(),
            num_dist_factors: 0,
        };
        file.put_set(&set)?;
        file.put_side_set(1, &elem_list, &side_list)?;
        file.put_name(EntityType::SideSet, 1, "pressure_surface")?;
    }

    Ok(())
}

fn is_side_on_surface(
    coords: &Coordinates<f64>,
    _nodes: &[i64],
    _normal: &[f64; 3]
) -> bool {
    // Implement actual surface detection logic
    // This is problem-specific
    true  // Placeholder
}
```

## Mesh Manipulation

### Recipe: Merge Two Meshes

```rust
fn merge_meshes(file1: &str, file2: &str, output: &str) -> Result<(), ExodusError> {
    // Read first mesh
    let f1 = ExodusFile::open(file1)?;
    let params1 = f1.init_params()?;
    let coords1 = f1.coords::<f64>()?;

    // Read second mesh
    let f2 = ExodusFile::open(file2)?;
    let params2 = f2.init_params()?;
    let coords2 = f2.coords::<f64>()?;

    // Create merged mesh
    let mut out = ExodusFile::create_default(output)?;

    let merged_params = InitParams {
        title: format!("Merged: {} + {}", params1.title, params2.title),
        num_dim: params1.num_dim,
        num_nodes: params1.num_nodes + params2.num_nodes,
        num_elems: params1.num_elems + params2.num_elems,
        num_elem_blocks: params1.num_elem_blocks + params2.num_elem_blocks,
        ..Default::default()
    };
    out.init(&merged_params)?;

    // Merge coordinates
    let mut x = coords1.x.clone();
    let mut y = coords1.y.clone();
    let mut z = coords1.z.clone();
    x.extend_from_slice(&coords2.x);
    y.extend_from_slice(&coords2.y);
    z.extend_from_slice(&coords2.z);

    out.put_coords(&x, Some(&y), Some(&z))?;

    // Copy blocks with renumbered connectivity
    // ... (implementation details)

    Ok(())
}
```

### Recipe: Refine Mesh (Uniform Subdivision)

```rust
fn refine_quad_mesh(input: &str, output: &str) -> Result<(), ExodusError> {
    let file = ExodusFile::open(input)?;
    let params = file.init_params()?;
    let coords = file.coords::<f64>()?;

    // For each quad, create 4 new quads by adding midpoint nodes
    // This is a simplified example

    let block = file.block(EntityType::ElemBlock, 1)?;
    let connectivity = file.connectivity(1)?;

    let num_orig_nodes = params.num_nodes;
    let num_orig_elems = block.num_entries;

    // New mesh has: original nodes + edge midpoints + element centers
    let num_new_nodes = num_orig_nodes + num_orig_elems * 5;  // Approx.
    let num_new_elems = num_orig_elems * 4;

    // Create refined mesh
    let mut out = ExodusFile::create_default(output)?;
    let new_params = InitParams {
        title: format!("{} (refined)", params.title),
        num_dim: params.num_dim,
        num_nodes: num_new_nodes,
        num_elems: num_new_elems,
        num_elem_blocks: 1,
        ..Default::default()
    };
    out.init(&new_params)?;

    // Generate new coordinates
    // ... (compute midpoints and element centers)

    Ok(())
}
```

## Performance Optimization

### Recipe: Use Compression for Large Files

```rust
let options = CreateOptions {
    compression: Some(6),  // Good balance
    ..Default::default()
};

// For maximum compression:
let options = CreateOptions {
    compression: Some(9),  // Slower write, smaller file
    ..Default::default()
};

// For faster writes:
let options = CreateOptions {
    compression: Some(1),  // Faster write, larger file
    ..Default::default()
};
```

### Recipe: Batch Variable Writes

```rust
// Less efficient: one variable at a time
for step in 0..num_steps {
    file.put_time(step + 1, times[step])?;
    file.put_var(step + 1, EntityType::Nodal, 0, 0, &temp_data[step])?;
    file.put_var(step + 1, EntityType::Nodal, 0, 1, &pres_data[step])?;
}

// More efficient: prepare data first, then write
for step in 0..num_steps {
    file.put_time(step + 1, times[step])?;
}

for step in 0..num_steps {
    file.put_var(step + 1, EntityType::Nodal, 0, 0, &temp_data[step])?;
}

for step in 0..num_steps {
    file.put_var(step + 1, EntityType::Nodal, 0, 1, &pres_data[step])?;
}
```

### Recipe: Use Float32 for Large Datasets

```rust
let options = CreateOptions {
    float_size: FloatSize::Float,  // Use f32 instead of f64
    compression: Some(6),
    ..Default::default()
};

let file = ExodusFile::create("large_mesh.exo", options)?;

// Can still write f64, will be converted
let coords_f64: Vec<f64> = /* ... */;
file.put_coords(&coords_f64, Some(&y_f64), Some(&z_f64))?;
```

## Integration

### Recipe: Convert from Another Format

```rust
// Example: Convert from simple text format to Exodus
fn convert_from_text(input_txt: &str, output_exo: &str) -> Result<(), ExodusError> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(input_txt)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Read nodes
    let num_nodes: usize = lines.next().unwrap()?.parse().unwrap();
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut z = Vec::new();

    for _ in 0..num_nodes {
        let line = lines.next().unwrap()?;
        let vals: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        x.push(vals[0]);
        y.push(vals[1]);
        z.push(vals[2]);
    }

    // Read elements
    let num_elems: usize = lines.next().unwrap()?.parse().unwrap();
    let mut connectivity = Vec::new();

    for _ in 0..num_elems {
        let line = lines.next().unwrap()?;
        let nodes: Vec<i64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        connectivity.extend(nodes);
    }

    // Write to Exodus
    let mut exo = ExodusFile::create_default(output_exo)?;
    let params = InitParams {
        title: "Converted Mesh".to_string(),
        num_dim: 3,
        num_nodes,
        num_elems,
        num_elem_blocks: 1,
        ..Default::default()
    };
    exo.init(&params)?;
    exo.put_coords(&x, Some(&y), Some(&z))?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "HEX8".to_string(),
        num_entries: num_elems,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    exo.put_block(&block)?;
    exo.put_connectivity(1, &connectivity)?;

    Ok(())
}
```

### Recipe: Generate Mesh Statistics

```rust
use std::collections::HashMap;

fn compute_mesh_statistics(filename: &str) -> Result<(), ExodusError> {
    let file = ExodusFile::open(filename)?;
    let params = file.init_params()?;
    let coords = file.coords::<f64>()?;

    println!("=== Mesh Statistics ===");
    println!("Nodes: {}", params.num_nodes);
    println!("Elements: {}", params.num_elems);

    // Bounding box
    let x_min = coords.x.iter().copied().fold(f64::MAX, f64::min);
    let x_max = coords.x.iter().copied().fold(f64::MIN, f64::max);
    let y_min = coords.y.iter().copied().fold(f64::MAX, f64::min);
    let y_max = coords.y.iter().copied().fold(f64::MIN, f64::max);

    println!("Bounding box:");
    println!("  X: [{:.6}, {:.6}]", x_min, x_max);
    println!("  Y: [{:.6}, {:.6}]", y_min, y_max);

    // Element types
    let block_ids = file.elem_block_ids()?;
    let mut element_types: HashMap<String, usize> = HashMap::new();

    for block_id in block_ids {
        let block = file.block(EntityType::ElemBlock, block_id)?;
        *element_types.entry(block.topology.clone()).or_insert(0) += block.num_entries;
    }

    println!("Element types:");
    for (topo, count) in element_types {
        println!("  {}: {}", topo, count);
    }

    Ok(())
}
```

## Additional Resources

- [User Guide](guide.md) - Comprehensive guide
- [Migration Guide](migration.md) - For C API users
- [Examples](../examples/) - Complete working examples
- [API Documentation](https://docs.rs/exodus-rs) - Detailed API reference
