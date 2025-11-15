//! Example demonstrating coordinate operations
//!
//! This example shows how to write and read nodal coordinates in an Exodus file.
//!
//! Run with:
//! ```bash
//! cargo run --example 03_coordinates --features netcdf4
//! ```

#[cfg(feature = "netcdf4")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use exodus_rs::{mode, ExodusFile};

    // Create a new Exodus file
    let mut file = ExodusFile::create_default("coordinates_example.exo")?;

    // Initialize the file with a 2D mesh of 4 nodes
    file.builder()
        .title("Coordinate Example - 2D Square")
        .dimensions(2)
        .nodes(4)
        .finish()?;

    println!("Created Exodus file with 4 nodes in 2D");

    // Define coordinates for a unit square
    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0];

    // Write coordinates
    file.put_coords(&x, Some(&y), None)?;

    println!("Wrote coordinates:");
    for (i, (&xi, &yi)) in x.iter().zip(y.iter()).enumerate() {
        println!("  Node {}: ({}, {})", i + 1, xi, yi);
    }

    // Close the file
    drop(file);

    // Re-open the file for reading
    let file = ExodusFile::<mode::Read>::open("coordinates_example.exo")?;

    // Read all coordinates
    let coords = file.coords::<f64>()?;

    println!("\nRead coordinates back:");
    println!("  Number of nodes: {}", coords.len());
    println!("  Number of dimensions: {}", coords.num_dim);

    // Iterate over coordinates
    for (i, coord) in coords.iter().enumerate() {
        if coords.num_dim == 2 {
            println!("  Node {}: ({}, {})", i + 1, coord[0], coord[1]);
        } else {
            println!(
                "  Node {}: ({}, {}, {})",
                i + 1,
                coord[0],
                coord[1],
                coord[2]
            );
        }
    }

    // Access individual coordinates
    if let Some(coord) = coords.get(0) {
        println!("\nFirst node coordinate: ({}, {})", coord[0], coord[1]);
    }

    // Read individual dimensions
    let x_coords = file.get_coord_x::<f64>()?;
    let y_coords = file.get_coord_y::<f64>()?;

    println!("\nX coordinates: {:?}", x_coords);
    println!("Y coordinates: {:?}", y_coords);

    println!("\n=== 3D Example ===\n");

    // Create a 3D example - a unit cube
    let mut file = ExodusFile::create_default("coordinates_3d_example.exo")?;

    file.builder()
        .title("Coordinate Example - 3D Cube")
        .dimensions(3)
        .nodes(8)
        .finish()?;

    // Define coordinates for a unit cube
    let x = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
    let z = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];

    file.put_coords(&x, Some(&y), Some(&z))?;

    println!("Created 3D cube with 8 nodes");

    // You can also write coordinates separately
    // file.put_coord_x(&x)?;
    // file.put_coord_y(&y)?;
    // file.put_coord_z(&z)?;

    drop(file);

    // Read 3D coordinates
    let file = ExodusFile::<mode::Read>::open("coordinates_3d_example.exo")?;
    let coords = file.coords::<f64>()?;

    println!("\n3D coordinates:");
    for (i, coord) in coords.iter().enumerate() {
        println!(
            "  Node {}: ({}, {}, {})",
            i + 1,
            coord[0],
            coord[1],
            coord[2]
        );
    }

    println!("\n=== Partial I/O Example ===\n");

    // Create a file with many nodes
    let mut file = ExodusFile::create_default("coordinates_partial_example.exo")?;

    file.builder()
        .title("Partial I/O Example")
        .dimensions(2)
        .nodes(10)
        .finish()?;

    // Write first 5 nodes
    let x1 = vec![0.0, 1.0, 2.0, 3.0, 4.0];
    let y1 = vec![0.0, 0.0, 0.0, 0.0, 0.0];
    file.put_partial_coords(0, 5, &x1, Some(&y1), None)?;

    println!("Wrote first 5 nodes");

    // Write next 5 nodes
    let x2 = vec![5.0, 6.0, 7.0, 8.0, 9.0];
    let y2 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
    file.put_partial_coords(5, 5, &x2, Some(&y2), None)?;

    println!("Wrote next 5 nodes");

    drop(file);

    // Read partial coordinates
    let file = ExodusFile::<mode::Read>::open("coordinates_partial_example.exo")?;

    // Read middle section (nodes 3-6)
    let partial_coords = file.get_partial_coords::<f64>(3, 4)?;

    println!("\nRead partial coordinates (nodes 4-7):");
    for (i, coord) in partial_coords.iter().enumerate() {
        println!("  Node {}: ({}, {})", i + 4, coord[0], coord[1]);
    }

    println!("\n=== Type Conversion Example ===\n");

    // You can write as f32 and read as f64, or vice versa
    let mut file = ExodusFile::create_default("coordinates_conversion_example.exo")?;

    file.builder()
        .title("Type Conversion Example")
        .dimensions(2)
        .nodes(3)
        .finish()?;

    // Write as f32
    let x: Vec<f32> = vec![0.0, 1.0, 2.0];
    let y: Vec<f32> = vec![0.0, 1.0, 2.0];
    file.put_coords(&x, Some(&y), None)?;

    println!("Wrote coordinates as f32");

    drop(file);

    // Read as f64
    let file = ExodusFile::<mode::Read>::open("coordinates_conversion_example.exo")?;
    let coords_f64 = file.coords::<f64>()?;

    println!("Read coordinates as f64: {:?}", coords_f64.x);

    // Read as f32
    let coords_f32 = file.coords::<f32>()?;

    println!("Read coordinates as f32: {:?}", coords_f32.x);

    // Clean up example files
    println!("\n=== Cleaning up example files ===");
    std::fs::remove_file("coordinates_example.exo")?;
    std::fs::remove_file("coordinates_3d_example.exo")?;
    std::fs::remove_file("coordinates_partial_example.exo")?;
    std::fs::remove_file("coordinates_conversion_example.exo")?;

    println!("\nExample completed successfully!");

    Ok(())
}

#[cfg(not(feature = "netcdf4"))]
fn main() {
    println!("This example requires the 'netcdf4' feature to be enabled.");
    println!("Run with: cargo run --example 03_coordinates --features netcdf4");
}
