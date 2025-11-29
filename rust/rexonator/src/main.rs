//! rexonator: CLI tool for transforming Exodus mesh files
//!
//! This tool applies geometric transformations to Exodus II mesh files,
//! including translation, rotation, scaling, and mirroring. Transformations
//! are applied in the order they appear on the command line.

mod cli;
mod copy_mirror_merge;
mod man;
mod operations;
mod parsers;
mod performance;
mod progress;

use clap::Parser;
use exodus_rs::{mode, ExodusFile};
use std::path::{Path, PathBuf};

use cli::{Axis, Cli, Operation, Result, TransformError};

/// Determine the output path and whether to use in-place mode.
///
/// In-place mode is enabled when:
/// 1. --in-place flag is explicitly set, OR
/// 2. input and output paths resolve to the same file (handles relative paths, symlinks, etc.)
///
/// Returns (effective_output_path, is_in_place_mode)
fn determine_output_mode(
    input: &Path,
    output: Option<&PathBuf>,
    in_place_flag: bool,
) -> Result<(PathBuf, bool)> {
    // If --in-place is set, output defaults to input
    if in_place_flag {
        let output_path = output.cloned().unwrap_or_else(|| input.to_path_buf());
        return Ok((output_path, true));
    }

    // Output is required when not in --in-place mode
    let output = output.ok_or_else(|| {
        TransformError::InvalidFormat(
            "OUTPUT is required unless --in-place is specified".to_string(),
        )
    })?;

    // Check if input and output resolve to the same file
    // Use canonicalize to handle relative paths, symlinks, etc.
    let input_canonical = input.canonicalize().ok();
    let output_canonical = output.canonicalize().ok();

    let same_file = match (&input_canonical, &output_canonical) {
        (Some(i), Some(o)) => i == o,
        // If output doesn't exist yet, compare the paths directly
        // (could be creating a new file with same relative name)
        (Some(i), None) => {
            // Try to see if they would resolve to the same path
            // by comparing parent + filename
            if let (Some(ip), Some(op)) = (input.parent(), output.parent()) {
                let ip_canon = ip.canonicalize().ok();
                let op_canon = op.canonicalize().ok();
                match (ip_canon, op_canon) {
                    (Some(ipc), Some(opc)) => ipc == opc && input.file_name() == output.file_name(),
                    _ => i.as_path() == output,
                }
            } else {
                false
            }
        }
        _ => false,
    };

    Ok((output.clone(), same_file))
}
use copy_mirror_merge::{
    apply_operation_to_mesh_data, copy_mirror_merge, normalize_time_mesh_data, read_mesh_data,
    warn_memory_usage, write_mesh_data, VectorDetectionConfig,
};
use man::show_man_page;
use operations::{apply_simple_operation, normalize_time};
use parsers::extract_ordered_operations;
use performance::PerformanceOptions;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Build performance configuration from CLI options
    let perf_config = PerformanceOptions::from_cli(&cli);

    // Handle --man flag
    if cli.man {
        show_man_page()?;
        return Ok(());
    }

    // Handle --show-perf-config
    if cli.show_perf_config {
        println!("{}", perf_config);
        return Ok(());
    }

    // Apply HDF5 environment variables before opening any files
    // These must be set before the HDF5 library is initialized
    perf_config.apply_env_vars();

    // Unwrap input (guaranteed to be present due to required_unless_present_any)
    let input = cli.input.as_ref().unwrap();

    // Determine effective output path and whether to use in-place mode
    // In-place mode is enabled when:
    //   1. --in-place flag is set (output defaults to input), OR
    //   2. input and output resolve to the same file
    let (output, in_place_mode) = determine_output_mode(input, cli.output.as_ref(), cli.in_place)?;

    // Extract operations in command-line order
    let operations = extract_ordered_operations(&cli, cli.verbose)?;

    // Handle --dry-run
    if cli.dry_run {
        println!("Dry-Run Mode Enabled:");
        println!("Input:  {}", input.display());
        println!("Output: {}", output.display());
        println!("Operations to apply: {}", operations.len());
        for (i, op) in operations.iter().enumerate() {
            println!("  {}: {:?}", i + 1, op);
        }
        println!();

        // Read input file stats
        let file = ExodusFile::open(input)?;
        let params = file.init_params()?;
        println!("Input Mesh Statistics:");
        println!("  Nodes:     {}", params.num_nodes);
        println!("  Elements:  {}", params.num_elems);
        println!("  Dimensions: {}", params.num_dim);
        println!("  Time Steps: {}", file.num_time_steps()?);
        println!();
        println!("No output file will be written in dry-run mode.");
        return Ok(());
    }

    if cli.verbose {
        println!("Input:  {}", input.display());
        println!("Output: {}", output.display());
        if in_place_mode {
            println!("Mode:   In-place (no file copy needed)");
        } else {
            println!("Mode:   Copy to output");
        }
        println!("Operations to apply: {}", operations.len());
        println!();
        println!("{}", perf_config);
        println!();
    }
    // Check if any CopyMirrorMerge operations are present
    let has_cmm = operations
        .iter()
        .any(|op| matches!(op, Operation::CopyMirrorMerge(_, _)));

    if has_cmm {
        // Optimized CMM path: all operations done in-memory with single read/write
        //
        // Instead of:
        //   file copy → open append → pre-CMM ops → close → open read → read mesh →
        //   close → CMM in memory → write → open append → post-CMM ops → close
        //
        // We do:
        //   open read → read mesh → close → pre-CMM in memory → CMM in memory →
        //   post-CMM in memory → write once
        //
        // This eliminates unnecessary file copies and multiple open/close cycles.

        // Split operations into groups: before CMM, CMM itself, after CMM
        let mut pre_cmm_ops = Vec::new();
        let mut cmm_op: Option<(Axis, f64)> = None;
        let mut post_cmm_ops = Vec::new();
        let mut found_cmm = false;

        for op in &operations {
            if let Operation::CopyMirrorMerge(axis, tolerance) = op {
                if found_cmm {
                    return Err(TransformError::InvalidFormat(
                        "Only one --copy-mirror-merge operation is supported per invocation"
                            .to_string(),
                    ));
                }
                cmm_op = Some((*axis, *tolerance));
                found_cmm = true;
            } else if found_cmm {
                post_cmm_ops.push(op.clone());
            } else {
                pre_cmm_ops.push(op.clone());
            }
        }

        let (cmm_axis, cmm_tolerance) = cmm_op.unwrap();

        // Build vector detection config from CLI options
        let vector_config = VectorDetectionConfig::from_cli_options(
            cli.vector_fields.as_deref(),
            cli.scalar_fields.as_deref(),
            cli.no_auto_vector_detection,
        );

        // Step 1: Read input mesh into memory
        if cli.verbose {
            println!("Reading input mesh into memory...");
        }
        let input_file = ExodusFile::<mode::Read>::open(input)?;
        let mut mesh_data = read_mesh_data(&input_file, cli.verbose)?;
        drop(input_file);

        // Check memory usage and warn if needed
        warn_memory_usage(&mesh_data, cli.verbose);

        // Step 2: Apply pre-CMM operations in memory
        if !pre_cmm_ops.is_empty() {
            if cli.verbose {
                println!("Applying pre-merge transformations in memory:");
            }
            for op in &pre_cmm_ops {
                apply_operation_to_mesh_data(&mut mesh_data, op, cli.verbose)?;
            }
        }

        // Step 3: Apply CopyMirrorMerge in memory
        if cli.verbose {
            println!("Applying copy-mirror-merge:");
        }
        let mut mesh_data = copy_mirror_merge(
            &mesh_data,
            cmm_axis,
            cmm_tolerance,
            &vector_config,
            cli.verbose,
        )?;

        // Step 4: Apply post-CMM operations in memory
        if !post_cmm_ops.is_empty() {
            if cli.verbose {
                println!("Applying post-merge transformations in memory:");
            }
            for op in &post_cmm_ops {
                apply_operation_to_mesh_data(&mut mesh_data, op, cli.verbose)?;
            }
        }

        // Step 5: Apply time normalization in memory if requested
        if cli.zero_time {
            if cli.verbose {
                println!("Normalizing time values:");
            }
            normalize_time_mesh_data(&mut mesh_data, cli.verbose);
        }

        // Step 6: Write output mesh (single write operation)
        if cli.verbose {
            println!("Writing output mesh...");
        }
        write_mesh_data(
            &output,
            &mesh_data,
            Some(perf_config.to_exodus_config()),
            cli.verbose,
        )?;
    } else {
        // Simple path: no CopyMirrorMerge
        // Use in-place mode when possible to avoid expensive file copy
        let target_path = if in_place_mode {
            if cli.verbose {
                println!("Operating in-place on input file (no copy needed)...");
            }
            input.to_path_buf()
        } else {
            if cli.verbose {
                println!("Copying input file to output location...");
            }
            std::fs::copy(input, &output)?;
            output.clone()
        };

        // Open the target file in append mode for modifications
        let mut file = ExodusFile::append(&target_path)?;

        if cli.verbose {
            let params = file.init_params()?;
            println!(
                "Mesh: {} nodes, {} elements, {} dimensions",
                params.num_nodes, params.num_elems, params.num_dim
            );
        }

        // Apply transformations in order
        if cli.verbose && !operations.is_empty() {
            println!("Applying transformations:");
        }

        for op in &operations {
            apply_simple_operation(&mut file, op, cli.verbose)?;
        }

        // Apply time normalization if requested
        if cli.zero_time {
            if cli.verbose {
                println!("Normalizing time values:");
            }
            normalize_time(&mut file, cli.verbose)?;
        }

        // Ensure all changes are written to disk
        file.sync()?;
    }

    if cli.verbose {
        println!("Done!");
    }

    Ok(())
}
