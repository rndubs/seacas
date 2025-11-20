//! Memory-efficient Exodus file processing example
//!
//! This example demonstrates how to process large Exodus files (100GB+) with limited memory
//! by processing data sequentially and minimizing allocations.
//!
//! Operations performed:
//! 1. Read and transform mesh coordinates (e.g., coordinate system transformation)
//! 2. Read, scale, and write time-history field values for all time steps
//! 3. Minimize memory usage by processing one time step at a time
//!
//! Usage:
//!   cargo run --release --example 12_process_large_file --features netcdf4 -- INPUT.exo OUTPUT.exo [SCALE_FACTOR]
//!
//! This Rust version should be significantly faster and more memory-efficient than the Python version
//! due to:
//! - No PyO3 marshaling overhead
//! - Zero-copy operations where possible
//! - Better memory control and reuse
//! - Compiler optimizations

use std::env;
use std::path::Path;
use std::time::Instant;

use exodus_rs::{
    CreateMode, CreateOptions, EntityType, ExodusError, ExodusFile, FloatSize, InitParams, Int64Mode,
};
use exodus_rs::performance::PerformanceConfig;

/// Apply coordinate transformation to mesh coordinates
///
/// Example: Rotate, translate, or scale the mesh.
/// This example applies a simple scaling and translation.
fn transform_coordinates(
    x: &[f64],
    y: &[f64],
    z: &[f64],
    num_dim: usize,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    println!("  Applying coordinate transformation...");

    let scale_factor = 2.0;
    let translation = [1.0, 2.0, 3.0];

    // Transform X coordinates
    let x_new: Vec<f64> = x.iter().map(|&xi| xi * scale_factor + translation[0]).collect();

    // Transform Y coordinates
    let y_new: Vec<f64> = if num_dim >= 2 {
        y.iter()
            .map(|&yi| yi * scale_factor + translation[1])
            .collect()
    } else {
        y.to_vec()
    };

    // Transform Z coordinates
    let z_new: Vec<f64> = if num_dim >= 3 {
        z.iter()
            .map(|&zi| zi * scale_factor + translation[2])
            .collect()
    } else {
        z.to_vec()
    };

    (x_new, y_new, z_new)
}

/// Apply scaling to field values in-place
///
/// This modifies the input vector instead of allocating a new one
fn scale_field_values_inplace(values: &mut [f64], scale_factor: f64) {
    for v in values.iter_mut() {
        *v *= scale_factor;
    }
}

/// Process an Exodus file with minimal memory usage
fn process_exodus_file<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    field_scale_factor: f64,
    perf_config: Option<PerformanceConfig>,
) -> Result<(), ExodusError> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    println!("Processing Exodus file: {}", input_path.display());
    println!("Output file: {}", output_path.display());
    println!("Field scale factor: {}", field_scale_factor);

    // Print performance configuration
    let perf = perf_config.unwrap_or_else(PerformanceConfig::auto);
    println!("\n{}", perf.summary());
    println!();

    let total_start = Instant::now();

    // Step 1: Open input file for reading
    println!("[1/6] Opening input file...");
    let reader = ExodusFile::open(input_path)?;

    // Step 2: Read metadata (minimal memory)
    println!("[2/6] Reading metadata...");
    let init_params = reader.init_params()?;
    let num_nodes = init_params.num_nodes;
    let num_elem = init_params.num_elems;
    let num_dim = init_params.num_dim;
    let num_blocks = init_params.num_elem_blocks;
    let num_time_steps = reader.num_time_steps()?;

    println!("  Nodes: {}", format_number(num_nodes));
    println!("  Elements: {}", format_number(num_elem));
    println!("  Dimensions: {}", num_dim);
    println!("  Element Blocks: {}", num_blocks);
    println!("  Time Steps: {}", format_number(num_time_steps));

    // Get variable info
    let nodal_var_names = reader.variable_names(EntityType::Nodal)?;
    let num_nodal_vars = nodal_var_names.len();
    println!("  Nodal Variables: {} - {:?}", num_nodal_vars, nodal_var_names);

    // Estimate memory usage
    let bytes_per_node = 8; // f64
    let mem_per_step_mb = (num_nodes * num_nodal_vars * bytes_per_node) as f64 / (1024.0 * 1024.0);
    println!(
        "  Estimated memory per time step: {:.1} MB",
        mem_per_step_mb
    );
    println!(
        "  Peak memory usage: ~{:.1} MB (2-3x per step in Rust vs 4x in Python)",
        mem_per_step_mb * 2.5
    );
    println!();

    // Step 3: Process coordinates (load once, transform, write)
    println!("[3/6] Processing coordinates...");
    println!("  Reading coordinates from input...");
    let coords = reader.coords::<f64>()?;

    println!("  Loaded {} nodes", format_number(coords.x.len()));
    let (x_new, y_new, z_new) = transform_coordinates(&coords.x, &coords.y, &coords.z, num_dim);

    // coords will be dropped here, freeing memory
    drop(coords);

    // Step 4: Create output file and write metadata
    println!("[4/6] Creating output file...");
    let options = CreateOptions {
        mode: CreateMode::Clobber,
        float_size: FloatSize::Float64,
        int64_mode: Int64Mode::Int64,
        performance: Some(perf),
        ..Default::default()
    };
    let mut writer = ExodusFile::create(output_path, options)?;

    // Initialize output file
    writer.init(&InitParams {
        title: init_params.title.clone(),
        num_nodes,
        num_dim,
        num_elems: num_elem,
        num_elem_blocks: num_blocks,
        num_node_sets: init_params.num_node_sets,
        num_side_sets: init_params.num_side_sets,
        ..Default::default()
    })?;

    // Write transformed coordinates
    println!("  Writing transformed coordinates...");
    writer.put_coords(
        &x_new,
        if num_dim >= 2 { Some(&y_new) } else { None },
        if num_dim >= 3 { Some(&z_new) } else { None },
    )?;

    // Free transformed coordinates
    drop(x_new);
    drop(y_new);
    drop(z_new);

    // Copy coordinate names
    let coord_names = reader.coord_names()?;
    if !coord_names.is_empty() {
        let coord_name_refs: Vec<&str> = coord_names.iter().map(|s| s.as_str()).collect();
        writer.put_coord_names(&coord_name_refs)?;
    }

    // Copy element blocks (connectivity)
    println!("  Copying element blocks...");
    let block_ids = reader.block_ids(EntityType::ElemBlock)?;
    for block_id in block_ids {
        let block = reader.block(block_id)?;

        writer.put_block(&block)?;

        // Copy connectivity
        let connectivity = reader.connectivity(block_id)?;
        writer.put_connectivity(block_id, &connectivity)?;
    }

    // Define variables in output file
    if num_nodal_vars > 0 {
        println!("  Defining variables...");
        let var_name_refs: Vec<&str> = nodal_var_names.iter().map(|s| s.as_str()).collect();
        writer.define_variables(EntityType::Nodal, &var_name_refs)?;
    }

    // Step 5: Process time steps sequentially (CRITICAL for memory efficiency)
    println!(
        "[5/6] Processing {} time steps...",
        format_number(num_time_steps)
    );
    println!("  Processing one time step at a time to minimize memory usage...");

    let scale_factor = field_scale_factor;
    let processing_start = Instant::now();

    for step in 0..num_time_steps {
        if step % 100 == 0 || step == num_time_steps - 1 {
            let progress = (step + 1) as f64 / num_time_steps as f64 * 100.0;
            let elapsed = processing_start.elapsed().as_secs_f64();
            let rate = (step + 1) as f64 / elapsed;
            let eta = (num_time_steps - step - 1) as f64 / rate;
            println!(
                "  Progress: {}/{} ({:.1}%) - {:.1} steps/sec - ETA: {:.0}s",
                format_number(step + 1),
                format_number(num_time_steps),
                progress,
                rate,
                eta
            );
        }

        // Read time value
        let time_val = reader.time(step)?;
        writer.put_time(step, time_val)?;

        // Process each nodal variable
        for var_idx in 0..num_nodal_vars {
            // Read variable data for this time step
            // Memory: 1x allocation
            let mut data: Vec<f64> = reader.var(step, EntityType::Nodal, 0, var_idx)?;

            // Scale the data IN-PLACE (no additional allocation!)
            scale_field_values_inplace(&mut data, scale_factor);

            // Write immediately (passes reference, no copy)
            writer.put_var(step, EntityType::Nodal, 0, var_idx, &data)?;

            // data is dropped here, freeing memory
        }
    }

    println!();
    println!("[6/6] Finalizing output file...");

    // Close files (explicit drop for clarity)
    drop(writer);
    drop(reader);

    let total_elapsed = total_start.elapsed();
    println!();
    println!("✓ Processing complete!");
    println!("  Output written to: {}", output_path.display());
    println!("  Total time: {:.2}s", total_elapsed.as_secs_f64());
    println!(
        "  Average: {:.1} steps/sec",
        num_time_steps as f64 / total_elapsed.as_secs_f64()
    );

    Ok(())
}

/// Format a number with thousands separators
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}

fn print_usage(program: &str) {
    eprintln!("Usage: {} INPUT.exo OUTPUT.exo [OPTIONS]", program);
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  INPUT.exo       - Input Exodus file path");
    eprintln!("  OUTPUT.exo      - Output Exodus file path");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --scale FACTOR           - Scale factor for field values (default: 1.5)");
    eprintln!();
    eprintln!("Performance Tuning:");
    eprintln!("  --auto                   - Auto-detect node type (default)");
    eprintln!("  --conservative           - Conservative settings for login nodes");
    eprintln!("  --aggressive             - Aggressive settings for compute nodes");
    eprintln!("  --cache-mb SIZE          - HDF5 chunk cache size in MB (e.g., 128)");
    eprintln!("  --cache-preemption VAL   - Cache preemption policy 0.0-1.0 (default: 0.75)");
    eprintln!("                             0.0 = favor writes, 1.0 = favor reads");
    eprintln!("  --node-chunk SIZE        - Nodes per chunk (e.g., 10000)");
    eprintln!("  --elem-chunk SIZE        - Elements per chunk (e.g., 10000)");
    eprintln!("  --time-chunk SIZE        - Time steps per chunk (default: 0 = no chunking)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  # Basic usage with auto-detection");
    eprintln!("  {} input.exo output.exo", program);
    eprintln!();
    eprintln!("  # Custom scale factor");
    eprintln!("  {} input.exo output.exo --scale 2.0", program);
    eprintln!();
    eprintln!("  # Aggressive performance for large compute node");
    eprintln!("  {} input.exo output.exo --aggressive", program);
    eprintln!();
    eprintln!("  # Custom cache and chunk sizes");
    eprintln!("  {} input.exo output.exo --cache-mb 256 --node-chunk 20000", program);
    eprintln!();
    eprintln!("  # Full customization");
    eprintln!("  {} input.exo output.exo --scale 1.5 \\", program);
    eprintln!("    --cache-mb 512 --cache-preemption 0.5 \\");
    eprintln!("    --node-chunk 50000 --elem-chunk 50000 --time-chunk 0");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    // Parse positional arguments
    let input_path = &args[1];
    let output_path = &args[2];

    // Parse optional arguments
    let mut scale_factor = 1.5;
    let mut perf_config = PerformanceConfig::auto();
    let mut perf_preset: Option<&str> = None;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--scale" => {
                if i + 1 < args.len() {
                    scale_factor = args[i + 1].parse::<f64>().unwrap_or_else(|_| {
                        eprintln!("ERROR: Invalid scale factor: {}", args[i + 1]);
                        std::process::exit(1);
                    });
                    i += 2;
                } else {
                    eprintln!("ERROR: --scale requires a value");
                    std::process::exit(1);
                }
            }
            "--auto" => {
                perf_preset = Some("auto");
                i += 1;
            }
            "--conservative" => {
                perf_preset = Some("conservative");
                i += 1;
            }
            "--aggressive" => {
                perf_preset = Some("aggressive");
                i += 1;
            }
            "--cache-mb" => {
                if i + 1 < args.len() {
                    let cache_mb = args[i + 1].parse::<usize>().unwrap_or_else(|_| {
                        eprintln!("ERROR: Invalid cache size: {}", args[i + 1]);
                        std::process::exit(1);
                    });
                    perf_config = perf_config.with_cache_mb(cache_mb);
                    i += 2;
                } else {
                    eprintln!("ERROR: --cache-mb requires a value");
                    std::process::exit(1);
                }
            }
            "--cache-preemption" => {
                if i + 1 < args.len() {
                    let preemption = args[i + 1].parse::<f64>().unwrap_or_else(|_| {
                        eprintln!("ERROR: Invalid preemption value: {}", args[i + 1]);
                        std::process::exit(1);
                    });
                    perf_config = perf_config.with_preemption(preemption);
                    i += 2;
                } else {
                    eprintln!("ERROR: --cache-preemption requires a value");
                    std::process::exit(1);
                }
            }
            "--node-chunk" => {
                if i + 1 < args.len() {
                    let chunk_size = args[i + 1].parse::<usize>().unwrap_or_else(|_| {
                        eprintln!("ERROR: Invalid node chunk size: {}", args[i + 1]);
                        std::process::exit(1);
                    });
                    perf_config = perf_config.with_node_chunk_size(chunk_size);
                    i += 2;
                } else {
                    eprintln!("ERROR: --node-chunk requires a value");
                    std::process::exit(1);
                }
            }
            "--elem-chunk" => {
                if i + 1 < args.len() {
                    let chunk_size = args[i + 1].parse::<usize>().unwrap_or_else(|_| {
                        eprintln!("ERROR: Invalid element chunk size: {}", args[i + 1]);
                        std::process::exit(1);
                    });
                    perf_config = perf_config.with_element_chunk_size(chunk_size);
                    i += 2;
                } else {
                    eprintln!("ERROR: --elem-chunk requires a value");
                    std::process::exit(1);
                }
            }
            "--time-chunk" => {
                if i + 1 < args.len() {
                    let chunk_size = args[i + 1].parse::<usize>().unwrap_or_else(|_| {
                        eprintln!("ERROR: Invalid time chunk size: {}", args[i + 1]);
                        std::process::exit(1);
                    });
                    perf_config = perf_config.with_time_chunk_size(chunk_size);
                    i += 2;
                } else {
                    eprintln!("ERROR: --time-chunk requires a value");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_usage(&args[0]);
                std::process::exit(0);
            }
            other => {
                eprintln!("ERROR: Unknown option: {}", other);
                eprintln!();
                print_usage(&args[0]);
                std::process::exit(1);
            }
        }
    }

    // Apply preset if specified (overrides previous settings)
    if let Some(preset) = perf_preset {
        perf_config = match preset {
            "auto" => PerformanceConfig::auto(),
            "conservative" => PerformanceConfig::conservative(),
            "aggressive" => PerformanceConfig::aggressive(),
            _ => perf_config,
        };
    }

    if !Path::new(input_path).exists() {
        eprintln!("ERROR: Input file not found: {}", input_path);
        std::process::exit(1);
    }

    if Path::new(output_path).exists() {
        eprintln!(
            "WARNING: Output file exists and will be overwritten: {}",
            output_path
        );
        eprint!("Continue? [y/N] ");
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input).unwrap();
        if input.trim().to_lowercase() != "y" {
            println!("Aborted.");
            std::process::exit(0);
        }
    }

    if let Err(e) = process_exodus_file(input_path, output_path, scale_factor, Some(perf_config))
    {
        eprintln!("\nERROR: {}", e);
        std::process::exit(1);
    }
}
