//! Example demonstrating mesh transformations
//!
//! This example shows how to apply various transformations to Exodus meshes,
//! including translation, rotation (axis-aligned and Euler angles), and scaling.

use exodus_rs::{CreateMode, CreateOptions, ExodusFile, InitParams};
use std::f64::consts::PI;

fn main() -> Result<(), exodus_rs::ExodusError> {
    println!("=== Mesh Transformation Example ===\n");

    // Create a simple mesh with a single element
    let file_path = "transform_example.exo";

    println!("1. Creating initial mesh...");
    {
        let mut file = ExodusFile::create(
            file_path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )?;

        // Initialize with 4 nodes (a square in XY plane)
        let params = InitParams {
            num_dim: 3,
            num_nodes: 4,
            num_elems: 1,
            num_elem_blocks: 1,
            ..Default::default()
        };
        file.init(&params)?;

        // Define a unit square centered at origin
        let x = vec![-0.5, 0.5, 0.5, -0.5];
        let y = vec![-0.5, -0.5, 0.5, 0.5];
        let z = vec![0.0, 0.0, 0.0, 0.0];
        file.put_coords(&x, Some(&y), Some(&z))?;

        println!("   Initial coordinates:");
        for i in 0..4 {
            println!("   Node {}: ({:.2}, {:.2}, {:.2})", i + 1, x[i], y[i], z[i]);
        }
    }

    // Example 1: Translation
    println!("\n2. Applying translation (+10, +5, +2)...");
    {
        let mut file = ExodusFile::append(file_path)?;
        file.translate(&[10.0, 5.0, 2.0])?;

        let coords = file.coords::<f64>()?;
        println!("   After translation:");
        for i in 0..4 {
            println!(
                "   Node {}: ({:.2}, {:.2}, {:.2})",
                i + 1,
                coords.x[i],
                coords.y[i],
                coords.z[i]
            );
        }
    }

    // Example 2: Rotation around Z axis
    println!("\n3. Applying 45-degree rotation around Z axis...");
    {
        let mut file = ExodusFile::append(file_path)?;
        file.rotate_z(45.0)?;

        let coords = file.coords::<f64>()?;
        println!("   After rotation:");
        for i in 0..4 {
            println!(
                "   Node {}: ({:.2}, {:.2}, {:.2})",
                i + 1,
                coords.x[i],
                coords.y[i],
                coords.z[i]
            );
        }
    }

    // Example 3: Scaling
    println!("\n4. Applying non-uniform scaling (2x, 1x, 0.5x)...");
    {
        let mut file = ExodusFile::append(file_path)?;
        file.scale(&[2.0, 1.0, 0.5])?;

        let coords = file.coords::<f64>()?;
        println!("   After scaling:");
        for i in 0..4 {
            println!(
                "   Node {}: ({:.2}, {:.2}, {:.2})",
                i + 1,
                coords.x[i],
                coords.y[i],
                coords.z[i]
            );
        }
    }

    // Reset and demonstrate Euler angles
    println!("\n5. Resetting mesh and applying Euler angle rotation...");
    {
        // Recreate mesh
        let mut file = ExodusFile::create(
            file_path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )?;

        let params = InitParams {
            num_dim: 3,
            num_nodes: 4,
            ..Default::default()
        };
        file.init(&params)?;

        let x = vec![1.0, 0.0, 0.0, 0.0];
        let y = vec![0.0, 1.0, 0.0, 0.0];
        let z = vec![0.0, 0.0, 1.0, 0.0];
        file.put_coords(&x, Some(&y), Some(&z))?;
    }

    // Example 4: Extrinsic Euler rotation
    println!("   Applying extrinsic XYZ Euler rotation (30°, 45°, 60°)...");
    {
        let mut file = ExodusFile::append(file_path)?;
        file.rotate_euler("XYZ", &[30.0, 45.0, 60.0], true)?;

        let coords = file.coords::<f64>()?;
        println!("   After Euler rotation:");
        for i in 0..4 {
            println!(
                "   Node {}: ({:.2}, {:.2}, {:.2})",
                i + 1,
                coords.x[i],
                coords.y[i],
                coords.z[i]
            );
        }
    }

    // Example 5: Direct rotation matrix application
    println!("\n6. Using transformations module directly...");
    {
        use exodus_rs::transformations::{apply_rotation_to_vector, rotation_matrix_z};

        let rotation = rotation_matrix_z(PI / 4.0); // 45 degrees in radians
        let point = [1.0, 0.0, 0.0];
        let rotated = apply_rotation_to_vector(&rotation, &point);

        println!(
            "   Original point: ({:.2}, {:.2}, {:.2})",
            point[0], point[1], point[2]
        );
        println!(
            "   After 45° Z rotation: ({:.2}, {:.2}, {:.2})",
            rotated[0], rotated[1], rotated[2]
        );
    }

    // Example 6: Tensor transformation
    println!("\n7. Tensor transformation example...");
    {
        use exodus_rs::transformations::{rotate_symmetric_tensor, rotation_matrix_z};

        // Example stress tensor (in Voigt notation: XX, YY, ZZ, XY, YZ, XZ)
        let stress_tensor = [100.0, 50.0, 25.0, 10.0, 5.0, 2.0];

        println!("   Original stress tensor (Voigt):");
        println!(
            "   σ_xx={:.2}, σ_yy={:.2}, σ_zz={:.2}",
            stress_tensor[0], stress_tensor[1], stress_tensor[2]
        );
        println!(
            "   σ_xy={:.2}, σ_yz={:.2}, σ_xz={:.2}",
            stress_tensor[3], stress_tensor[4], stress_tensor[5]
        );

        let rotation = rotation_matrix_z(PI / 4.0); // 45 degrees
        let rotated_tensor = rotate_symmetric_tensor(&rotation, &stress_tensor);

        println!("   After 45° Z rotation:");
        println!(
            "   σ_xx={:.2}, σ_yy={:.2}, σ_zz={:.2}",
            rotated_tensor[0], rotated_tensor[1], rotated_tensor[2]
        );
        println!(
            "   σ_xy={:.2}, σ_yz={:.2}, σ_xz={:.2}",
            rotated_tensor[3], rotated_tensor[4], rotated_tensor[5]
        );
    }

    println!("\n=== Transformation example completed ===");
    println!("Output file: {}", file_path);

    Ok(())
}
