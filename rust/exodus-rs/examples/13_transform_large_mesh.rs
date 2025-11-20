///! Transform large exodus meshes with performance monitoring
///!
///! This example demonstrates applying transformations (rotation, translation, scaling)
///! to large Exodus files with comprehensive performance tuning options.
///!
///! Features:
///! - Rotate mesh and stress tensors by 90 degrees around Z axis
///! - Scale mesh by 5x
///! - Scale scalar values by 10x
///! - Performance monitoring for different cache/chunk configurations
///! - CLI arguments for tuning HDF5 cache and chunking

use clap::Parser;
use exodus_rs::{
    CreateMode, CreateOptions, EntityType, ExodusError, ExodusFile, InitParams, PerformanceConfig,
};
use exodus_rs::transformations::{rotate_symmetric_tensor, rotation_matrix_z};
use std::f64::consts::PI;
use std::time::{Duration, Instant};

/// CLI arguments for transformation with performance tuning
#[derive(Parser, Debug)]
#[clap(
    name = "transform_large_mesh",
    about = "Transform large Exodus meshes with performance monitoring"
)]
struct Args {
    /// Input Exodus file path
    #[clap(short, long)]
    input: String,

    /// Output Exodus file path
    #[clap(short, long)]
    output: String,

    /// HDF5 cache size in MB (default: 128)
    #[clap(long, default_value = "128")]
    cache_mb: usize,

    /// Node chunk size for HDF5 (default: 10000)
    #[clap(long, default_value = "10000")]
    node_chunk_size: usize,

    /// Element chunk size for HDF5 (default: 8000)
    #[clap(long, default_value = "8000")]
    element_chunk_size: usize,

    /// Time step chunk size for HDF5 (default: 10)
    #[clap(long, default_value = "10")]
    time_chunk_size: usize,

    /// Cache preemption policy (0.0-1.0, default: 0.75)
    #[clap(long, default_value = "0.75")]
    preemption: f64,

    /// Rotation angle in degrees (default: 90)
    #[clap(long, default_value = "90.0")]
    rotation: f64,

    /// Mesh scale factor (default: 5.0)
    #[clap(long, default_value = "5.0")]
    mesh_scale: f64,

    /// Scalar variable scale factor (default: 10.0)
    #[clap(long, default_value = "10.0")]
    scalar_scale: f64,
}

/// Performance timing data
#[derive(Default, Debug)]
struct TimingData {
    read_metadata: Duration,
    copy_mesh: Duration,
    transform_coords: Duration,
    transform_variables: Duration,
    write_output: Duration,
    total: Duration,
}

impl TimingData {
    fn print_summary(&self) {
        println!("\n{}", "=".repeat(70));
        println!("PERFORMANCE SUMMARY");
        println!("{}", "=".repeat(70));
        println!("{:<30} {:>12.2} seconds", "Read metadata:", self.read_metadata.as_secs_f64());
        println!("{:<30} {:>12.2} seconds", "Copy mesh:", self.copy_mesh.as_secs_f64());
        println!("{:<30} {:>12.2} seconds", "Transform coordinates:", self.transform_coords.as_secs_f64());
        println!("{:<30} {:>12.2} seconds", "Transform variables:", self.transform_variables.as_secs_f64());
        println!("{:<30} {:>12.2} seconds", "Write output:", self.write_output.as_secs_f64());
        println!("{}", "-".repeat(70));
        println!("{:<30} {:>12.2} seconds", "TOTAL:", self.total.as_secs_f64());
        println!("{}", "=".repeat(70));
    }
}

fn main() -> Result<(), ExodusError> {
    let args = Args::parse();
    let mut timing = TimingData::default();

    let start_total = Instant::now();

    println!("{}", "=".repeat(70));
    println!("TRANSFORM LARGE EXODUS MESH");
    println!("{}", "=".repeat(70));
    println!("Input:  {}", args.input);
    println!("Output: {}", args.output);
    println!("\nTransformations:");
    println!("  - Rotate {} degrees around Z axis", args.rotation);
    println!("  - Scale mesh by {}", args.mesh_scale);
    println!("  - Scale scalar variables by {}", args.scalar_scale);
    println!("\nPerformance settings:");
    println!("  - Cache size: {} MB", args.cache_mb);
    println!("  - Node chunk size: {}", args.node_chunk_size);
    println!("  - Element chunk size: {}", args.element_chunk_size);
    println!("  - Time chunk size: {}", args.time_chunk_size);
    println!("  - Cache preemption: {}", args.preemption);
    println!();

    // Step 1: Read metadata from input file
    println!("Step 1: Reading metadata from input file...");
    let start = Instant::now();

    let input_file = ExodusFile::open_read(&args.input)?;
    let params = input_file.init_params()?;
    let coords = input_file.coords::<f64>()?;
    let coord_names = input_file.coord_names()?;
    let block_ids = input_file.block_ids()?;
    let nodal_var_names = input_file.variable_names(EntityType::Nodal)?;
    let elem_var_names = input_file.variable_names(EntityType::ElemBlock)?;
    let num_time_steps = input_file.num_time_steps()?;

    timing.read_metadata = start.elapsed();

    println!("  Nodes: {}", params.num_nodes);
    println!("  Elements: {}", params.num_elems);
    println!("  Element blocks: {}", params.num_elem_blocks);
    println!("  Nodal variables: {}", nodal_var_names.len());
    println!("  Element variables: {}", elem_var_names.len());
    println!("  Time steps: {}", num_time_steps);
    println!("  Duration: {:.2}s", timing.read_metadata.as_secs_f64());

    // Step 2: Create output file with performance config
    println!("\nStep 2: Creating output file...");
    let start = Instant::now();

    let perf = PerformanceConfig::auto()
        .with_cache_mb(args.cache_mb)
        .with_node_chunk_size(args.node_chunk_size)
        .with_element_chunk_size(args.element_chunk_size)
        .with_time_chunk_size(args.time_chunk_size)
        .with_preemption(args.preemption);

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(perf),
        ..Default::default()
    };

    let mut output_file = ExodusFile::create(&args.output, options)?;

    // Initialize with same parameters
    output_file.init(&params)?;
    output_file.put_coord_names(&coord_names)?;

    // Copy element blocks
    for block_id in &block_ids {
        let block = input_file.block(*block_id)?;
        output_file.put_block(&block)?;

        let connectivity = input_file.connectivity(*block_id)?;
        output_file.put_connectivity(*block_id, &connectivity)?;
    }

    // Define variables
    if !nodal_var_names.is_empty() {
        output_file.define_variables(EntityType::Nodal, &nodal_var_names)?;
    }
    if !elem_var_names.is_empty() {
        output_file.define_variables(EntityType::ElemBlock, &elem_var_names)?;
    }

    timing.copy_mesh = start.elapsed();
    println!("  Duration: {:.2}s", timing.copy_mesh.as_secs_f64());

    // Step 3: Transform coordinates
    println!("\nStep 3: Transforming coordinates...");
    let start = Instant::now();

    let rotation_matrix = rotation_matrix_z((args.rotation * PI) / 180.0);

    // Apply rotation to coordinates
    let num_nodes = coords.x.len();
    let mut new_x = Vec::with_capacity(num_nodes);
    let mut new_y = Vec::with_capacity(num_nodes);
    let mut new_z = Vec::with_capacity(num_nodes);

    for i in 0..num_nodes {
        let point = [coords.x[i], coords.y[i], coords.z[i]];
        let rotated = exodus_rs::transformations::apply_rotation_to_vector(&rotation_matrix, &point);
        new_x.push(rotated[0] * args.mesh_scale);
        new_y.push(rotated[1] * args.mesh_scale);
        new_z.push(rotated[2] * args.mesh_scale);
    }

    output_file.put_coords(&new_x, Some(&new_y), Some(&new_z))?;

    timing.transform_coords = start.elapsed();
    println!("  Duration: {:.2}s", timing.transform_coords.as_secs_f64());

    // Step 4: Transform and write variables
    println!("\nStep 4: Transforming and writing variables...");
    println!("  Processing {} time steps...", num_time_steps);
    let start = Instant::now();

    let stress_var_indices = find_stress_tensor_indices(&elem_var_names);

    for step in 1..=num_time_steps {
        // Write time value
        let time_value = input_file.time(step)?;
        output_file.put_time(step, time_value)?;

        // Transform nodal variables (scale scalars)
        for (var_idx, _var_name) in nodal_var_names.iter().enumerate() {
            let mut values = input_file.var::<f64>(step, EntityType::Nodal, 0, var_idx + 1)?;
            for val in &mut values {
                *val *= args.scalar_scale;
            }
            output_file.put_var(step, EntityType::Nodal, 0, var_idx + 1, &values)?;
        }

        // Transform element variables
        for block_id in &block_ids {
            for (var_idx, _var_name) in elem_var_names.iter().enumerate() {
                let mut values = input_file.var::<f64>(step, EntityType::ElemBlock, *block_id, var_idx + 1)?;

                // If this variable is part of stress tensor, rotate it
                if let Some((xx, yy, zz, xy, yz, xz)) = &stress_var_indices {
                    if var_idx == *xx || var_idx == *yy || var_idx == *zz ||
                       var_idx == *xy || var_idx == *yz || var_idx == *xz {
                        // Read all stress components for this element
                        let stress_xx = input_file.var::<f64>(step, EntityType::ElemBlock, *block_id, xx + 1)?;
                        let stress_yy = input_file.var::<f64>(step, EntityType::ElemBlock, *block_id, yy + 1)?;
                        let stress_zz = input_file.var::<f64>(step, EntityType::ElemBlock, *block_id, zz + 1)?;
                        let stress_xy = input_file.var::<f64>(step, EntityType::ElemBlock, *block_id, xy + 1)?;
                        let stress_yz = input_file.var::<f64>(step, EntityType::ElemBlock, *block_id, yz + 1)?;
                        let stress_xz = input_file.var::<f64>(step, EntityType::ElemBlock, *block_id, xz + 1)?;

                        // Transform each element's stress tensor
                        for elem_idx in 0..values.len() {
                            let tensor = [
                                stress_xx[elem_idx],
                                stress_yy[elem_idx],
                                stress_zz[elem_idx],
                                stress_xy[elem_idx],
                                stress_yz[elem_idx],
                                stress_xz[elem_idx],
                            ];
                            let rotated = rotate_symmetric_tensor(&rotation_matrix, &tensor);

                            // Update the value for this component
                            values[elem_idx] = match var_idx {
                                _ if var_idx == *xx => rotated[0],
                                _ if var_idx == *yy => rotated[1],
                                _ if var_idx == *zz => rotated[2],
                                _ if var_idx == *xy => rotated[3],
                                _ if var_idx == *yz => rotated[4],
                                _ if var_idx == *xz => rotated[5],
                                _ => values[elem_idx],
                            };
                        }
                    }
                }

                output_file.put_var(step, EntityType::ElemBlock, *block_id, var_idx + 1, &values)?;
            }
        }

        if step % 1000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let progress = (step as f64) / (num_time_steps as f64) * 100.0;
            let steps_per_sec = (step as f64) / elapsed;
            let eta_sec = ((num_time_steps - step) as f64) / steps_per_sec;
            println!(
                "  Step {}/{} ({:.1}%) - {:.1} steps/sec - ETA: {:.1} min",
                step, num_time_steps, progress, steps_per_sec, eta_sec / 60.0
            );
        }
    }

    timing.transform_variables = start.elapsed();
    println!("  Duration: {:.2}s", timing.transform_variables.as_secs_f64());

    // Close files
    println!("\nStep 5: Finalizing output file...");
    let start = Instant::now();
    drop(output_file);
    drop(input_file);
    timing.write_output = start.elapsed();
    println!("  Duration: {:.2}s", timing.write_output.as_secs_f64());

    timing.total = start_total.elapsed();
    timing.print_summary();

    // Calculate throughput
    let file_size_gb = std::fs::metadata(&args.input)?.len() as f64 / (1024.0 * 1024.0 * 1024.0);
    let throughput = file_size_gb / timing.total.as_secs_f64();

    println!("\nThroughput: {:.2} GB/s", throughput);
    println!("\nOutput file: {}", args.output);

    Ok(())
}

/// Find indices of stress tensor components in variable names
fn find_stress_tensor_indices(var_names: &[String]) -> Option<(usize, usize, usize, usize, usize, usize)> {
    let xx = var_names.iter().position(|n| n.contains("stress_xx"))?;
    let yy = var_names.iter().position(|n| n.contains("stress_yy"))?;
    let zz = var_names.iter().position(|n| n.contains("stress_zz"))?;
    let xy = var_names.iter().position(|n| n.contains("stress_xy"))?;
    let yz = var_names.iter().position(|n| n.contains("stress_yz"))?;
    let xz = var_names.iter().position(|n| n.contains("stress_xz"))?;

    Some((xx, yy, zz, xy, yz, xz))
}
