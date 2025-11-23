# Mesh Transformations in exodus-rs

The exodus-rs library provides comprehensive support for applying spatial transformations to Exodus meshes and their associated field data.

## Overview

Transformations allow you to:
- **Translate** meshes by a vector offset
- **Rotate** meshes using axis-aligned rotations or Euler angles
- **Scale** meshes uniformly or non-uniformly
- **Transform field data** (vectors and tensors) to match mesh orientation

## Basic Usage

### Translation

Move a mesh by a vector offset:

```rust
use exodus_rs::ExodusFile;

let mut file = ExodusFile::append("mesh.exo")?;
file.translate(&[10.0, 5.0, 0.0])?; // Move 10 units in X, 5 in Y
```

### Rotation

#### Axis-Aligned Rotations

Rotate around the X, Y, or Z axis:

```rust
// Rotate 45 degrees around Z axis
file.rotate_z(45.0)?;

// Rotate 90 degrees around X axis
file.rotate_x(90.0)?;

// Rotate 30 degrees around Y axis
file.rotate_y(30.0)?;
```

**Note**: All angles are in degrees and are automatically converted to radians internally.

#### Euler Angle Rotations

Apply complex rotations using Euler angles following the scipy convention:

```rust
// Extrinsic XYZ rotation (rotate around fixed axes)
file.rotate_euler("XYZ", &[30.0, 45.0, 60.0], true)?;

// Intrinsic xyz rotation (rotate around body axes)
file.rotate_euler("xyz", &[30.0, 45.0, 60.0], true)?;

// Single-axis rotation using Euler notation
file.rotate_euler("Z", &[90.0], true)?;
```

**Euler Sequence Notation**:
- **Uppercase** (e.g., "XYZ"): Extrinsic rotations - applied in the fixed/global frame
- **Lowercase** (e.g., "xyz"): Intrinsic rotations - applied in the body/local frame

### Scaling

Scale a mesh uniformly or non-uniformly:

```rust
// Uniform scaling (double all dimensions)
file.scale_uniform(2.0)?;

// Non-uniform scaling (scale each axis independently)
file.scale(&[2.0, 1.0, 0.5])?; // Double X, keep Y, halve Z
```

## Field Data Transformations

When you rotate or transform a mesh, you may also need to transform associated field variables (like displacement, velocity, stress, strain) to maintain consistency.

### Vector Field Transformations

For 3-component vector fields (displacement, velocity, etc.):

```rust
use exodus_rs::transformations::rotation_matrix_z;
use std::f64::consts::PI;

// Create rotation matrix (90 degrees around Z)
let rotation = rotation_matrix_z(PI / 2.0);

// Manually read and transform nodal variable data
let var_names = file.variable_names(EntityType::Nodal)?;
let num_time_steps = file.num_time_steps()?;

for time_step in 1..=num_time_steps {
    // Read variable components at this time step
    let disp_x_idx = var_names.iter().position(|n| n == "disp_x").unwrap();
    let disp_y_idx = var_names.iter().position(|n| n == "disp_y").unwrap();
    let disp_z_idx = var_names.iter().position(|n| n == "disp_z").unwrap();

    let x_data = file.var(time_step, EntityType::Nodal, 0, disp_x_idx)?;
    let y_data = file.var(time_step, EntityType::Nodal, 0, disp_y_idx)?;
    let z_data = file.var(time_step, EntityType::Nodal, 0, disp_z_idx)?;

    // Transform each vector
    use exodus_rs::transformations::apply_rotation_to_vector;
    let mut new_x = Vec::new();
    let mut new_y = Vec::new();
    let mut new_z = Vec::new();

    for i in 0..x_data.len() {
        let vec = [x_data[i], y_data[i], z_data[i]];
        let rotated = apply_rotation_to_vector(&rotation, &vec);
        new_x.push(rotated[0]);
        new_y.push(rotated[1]);
        new_z.push(rotated[2]);
    }

    // Write back transformed data
    file.put_var(time_step, EntityType::Nodal, 0, disp_x_idx, &new_x)?;
    file.put_var(time_step, EntityType::Nodal, 0, disp_y_idx, &new_y)?;
    file.put_var(time_step, EntityType::Nodal, 0, disp_z_idx, &new_z)?;
}
```

### Tensor Field Transformations

For 6-component symmetric tensor fields (stress, strain) in Voigt notation:

```rust
use exodus_rs::transformations::{rotation_matrix_z, rotate_symmetric_tensor};
use std::f64::consts::PI;

// Create rotation matrix
let rotation = rotation_matrix_z(PI / 4.0); // 45 degrees

// Read tensor components (Voigt notation: XX, YY, ZZ, XY, YZ, XZ)
let stress_xx = file.var(time_step, EntityType::Nodal, 0, stress_xx_idx)?;
let stress_yy = file.var(time_step, EntityType::Nodal, 0, stress_yy_idx)?;
let stress_zz = file.var(time_step, EntityType::Nodal, 0, stress_zz_idx)?;
let stress_xy = file.var(time_step, EntityType::Nodal, 0, stress_xy_idx)?;
let stress_yz = file.var(time_step, EntityType::Nodal, 0, stress_yz_idx)?;
let stress_xz = file.var(time_step, EntityType::Nodal, 0, stress_xz_idx)?;

// Transform each tensor
let mut new_xx = Vec::new();
let mut new_yy = Vec::new();
let mut new_zz = Vec::new();
let mut new_xy = Vec::new();
let mut new_yz = Vec::new();
let mut new_xz = Vec::new();

for i in 0..stress_xx.len() {
    let tensor = [
        stress_xx[i], stress_yy[i], stress_zz[i],
        stress_xy[i], stress_yz[i], stress_xz[i]
    ];
    let rotated = rotate_symmetric_tensor(&rotation, &tensor);
    new_xx.push(rotated[0]);
    new_yy.push(rotated[1]);
    new_zz.push(rotated[2]);
    new_xy.push(rotated[3]);
    new_yz.push(rotated[4]);
    new_xz.push(rotated[5]);
}

// Write back transformed tensors
file.put_var(time_step, EntityType::Nodal, 0, stress_xx_idx, &new_xx)?;
// ... write other components
```

## Memory-Efficient Processing

For large datasets with hundreds of thousands of nodes and many time steps, process one time step at a time to minimize memory usage:

```rust
let num_time_steps = file.num_time_steps()?;

for time_step in 1..=num_time_steps {
    // Transform field data for this time step only
    // This keeps memory usage constant regardless of total time steps
    transform_variables_at_timestep(&mut file, time_step, &rotation)?;
}
```

## Low-Level Transformation Utilities

The `transformations` module provides low-level utilities for direct matrix operations:

```rust
use exodus_rs::transformations::*;

// Create rotation matrices
let rot_x = rotation_matrix_x(deg_to_rad(45.0));
let rot_y = rotation_matrix_y(deg_to_rad(30.0));
let rot_z = rotation_matrix_z(deg_to_rad(60.0));

// Compose matrices
let combined = multiply_matrices(&rot_x, &rot_y);

// Create from Euler angles
let matrix = rotation_matrix_from_euler("XYZ", &[30.0, 45.0, 60.0], true)?;

// Apply to vectors
let point = [1.0, 2.0, 3.0];
let rotated = apply_rotation_to_vector(&matrix, &point);

// Rotate tensors
let stress = [100.0, 50.0, 25.0, 10.0, 5.0, 2.0]; // Voigt notation
let rotated_stress = rotate_symmetric_tensor(&matrix, &stress);
```

## Example: Complete Workflow

See `examples/11_mesh_transformations.rs` for a complete example demonstrating:
- Basic coordinate transformations
- Euler angle rotations
- Direct matrix application
- Tensor transformations

Run with:
```bash
cargo run --example 11_mesh_transformations --features netcdf4
```

## API Reference

### High-Level Methods (on `ExodusFile<mode::Append>`)

- `translate(&mut self, translation: &[f64; 3]) -> Result<()>`
- `rotate_x(&mut self, angle_degrees: f64) -> Result<()>`
- `rotate_y(&mut self, angle_degrees: f64) -> Result<()>`
- `rotate_z(&mut self, angle_degrees: f64) -> Result<()>`
- `rotate_euler(&mut self, seq: &str, angles: &[f64], degrees: bool) -> Result<()>`
- `apply_rotation(&mut self, rotation_matrix: &Matrix3x3) -> Result<()>`
- `scale_uniform(&mut self, scale_factor: f64) -> Result<()>`
- `scale(&mut self, scale_factors: &[f64; 3]) -> Result<()>`

### Low-Level Utilities (`transformations` module)

- `rotation_matrix_x(angle_rad: f64) -> Matrix3x3`
- `rotation_matrix_y(angle_rad: f64) -> Matrix3x3`
- `rotation_matrix_z(angle_rad: f64) -> Matrix3x3`
- `rotation_matrix_from_euler(seq: &str, angles: &[f64], degrees: bool) -> Result<Matrix3x3>`
- `apply_rotation_to_vector(matrix: &Matrix3x3, vec: &[f64; 3]) -> [f64; 3]`
- `rotate_symmetric_tensor(rotation: &Matrix3x3, tensor: &[f64; 6]) -> [f64; 6]`
- `multiply_matrices(a: &Matrix3x3, b: &Matrix3x3) -> Matrix3x3`
- `deg_to_rad(degrees: f64) -> f64`

## Notes

- All high-level transformation methods work only on files opened in `Append` mode
- Angles are always in degrees for high-level methods (automatically converted to radians)
- Low-level utilities use radians for consistency with standard math libraries
- Transformations modify the file in-place
- For large datasets, process time steps individually to manage memory efficiently
