//! Example 06: Variables and Time Steps
//!
//! This example demonstrates how to work with variables and time-dependent data in Exodus files.
//! It shows:
//! - Defining global, nodal, and element variables
//! - Writing variable data across multiple time steps
//! - Using truth tables for sparse element variable storage
//! - Reading variable data
//! - Time step management

use exodus_rs::{
    mode, Block, EntityType, ExodusFile, InitParams, TruthTable,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Exodus Variables and Time Steps Example ===\n");

    // Example 1: Global variables (simulation-level scalars)
    println!("1. Creating global variables");
    create_global_variables()?;

    // Example 2: Nodal variables (per-node data)
    println!("\n2. Creating nodal variables");
    create_nodal_variables()?;

    // Example 3: Element variables with truth tables
    println!("\n3. Creating element variables with truth tables");
    create_element_variables()?;

    // Example 4: Time series data
    println!("\n4. Writing time series data");
    write_time_series()?;

    // Example 5: Reading variables
    println!("\n5. Reading variable data");
    read_variables()?;

    println!("\n=== Example Complete ===");
    Ok(())
}

/// Create global variables (simulation-level scalars)
fn create_global_variables() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = ExodusFile::create_default("global_vars.exo")?;

    // Initialize
    let params = InitParams {
        title: "Global Variables Example".into(),
        num_dim: 2,
        num_nodes: 4,
        ..Default::default()
    };
    file.init(&params)?;

    // Simple coordinates
    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0];
    file.put_coords(&x, Some(&y), None)?;

    // Define 3 global variables
    let var_names = vec!["time_step_size", "total_energy", "max_temperature"];
    file.define_variables(EntityType::Global, &var_names)?;

    println!("  ✓ Defined {} global variables:", var_names.len());
    for (i, name) in var_names.iter().enumerate() {
        println!("    {}: {}", i + 1, name);
    }

    // Write values for 3 time steps
    for time_step in 0..3 {
        let time = time_step as f64 * 0.1;
        file.put_time(time_step, time)?;

        // Simulate some physics
        let dt = 0.01 * (time_step + 1) as f64;
        let energy = 100.0 + (time_step + 1) as f64 * 5.0;
        let max_temp = 300.0 + (time_step + 1) as f64 * 10.0;

        file.put_var(time_step, EntityType::Global, 0, 0, &[dt])?;
        file.put_var(time_step, EntityType::Global, 0, 1, &[energy])?;
        file.put_var(time_step, EntityType::Global, 0, 2, &[max_temp])?;
    }

    println!("\n  ✓ Wrote values for 3 time steps");
    Ok(())
}

/// Create nodal variables (per-node data)
fn create_nodal_variables() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = ExodusFile::create_default("nodal_vars.exo")?;

    // Initialize with a simple 3x3 grid
    let params = InitParams {
        title: "Nodal Variables Example".into(),
        num_dim: 2,
        num_nodes: 9,
        ..Default::default()
    };
    file.init(&params)?;

    // Write coordinates for a 3x3 grid
    let x = vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];
    file.put_coords(&x, Some(&y), None)?;

    // Define nodal variables
    let var_names = vec!["temperature", "displacement_x", "displacement_y"];
    file.define_variables(EntityType::Nodal, &var_names)?;

    println!("  ✓ Defined {} nodal variables:", var_names.len());
    for (i, name) in var_names.iter().enumerate() {
        println!("    {}: {}", i + 1, name);
    }

    // Write values for 2 time steps
    for time_step in 0..2 {
        let time = (time_step + 1) as f64 * 0.5;
        file.put_time(time_step, time)?;

        // Temperature: simulate heat diffusion pattern
        let temp: Vec<f64> = x
            .iter()
            .zip(&y)
            .map(|(xi, yi)| 300.0 + time * (xi * 20.0 + yi * 10.0))
            .collect();
        file.put_var(time_step, EntityType::Nodal, 0, 0, &temp)?;

        // Displacement X: simulate expansion
        let disp_x: Vec<f64> = x.iter().map(|xi| xi * 0.01 * time).collect();
        file.put_var(time_step, EntityType::Nodal, 0, 1, &disp_x)?;

        // Displacement Y: simulate expansion
        let disp_y: Vec<f64> = y.iter().map(|yi| yi * 0.01 * time).collect();
        file.put_var(time_step, EntityType::Nodal, 0, 2, &disp_y)?;
    }

    println!("\n  ✓ Wrote values for 2 time steps across 9 nodes");
    Ok(())
}

/// Create element variables with truth tables
fn create_element_variables() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = ExodusFile::create_default("element_vars.exo")?;

    // Initialize with 2 element blocks
    let params = InitParams {
        title: "Element Variables with Truth Tables".into(),
        num_dim: 2,
        num_nodes: 12,
        num_elems: 6,
        num_elem_blocks: 2,
        ..Default::default()
    };
    file.init(&params)?;

    // Simplified coordinates
    let coords: Vec<f64> = (0..12).map(|i| i as f64 * 0.1).collect();
    file.put_coords(&coords, Some(&coords), None)?;

    // Element block 1: 4 QUAD4 elements
    let block1 = Block {
        id: 100,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".into(),
        num_entries: 4,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block1)?;
    file.put_connectivity(
        100,
        &vec![1, 2, 5, 4, 2, 3, 6, 5, 4, 5, 8, 7, 5, 6, 9, 8],
    )?;

    // Element block 2: 2 QUAD4 elements
    let block2 = Block {
        id: 200,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".into(),
        num_entries: 2,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block2)?;
    file.put_connectivity(200, &vec![7, 8, 11, 10, 8, 9, 12, 11])?;

    // Define element variables
    let var_names = vec!["stress_xx", "stress_yy", "strain_energy"];
    file.define_variables(EntityType::ElemBlock, &var_names)?;

    println!("  ✓ Defined {} element variables:", var_names.len());
    for (i, name) in var_names.iter().enumerate() {
        println!("    {}: {}", i + 1, name);
    }

    // Create truth table (sparse storage)
    // Block 1 has all 3 variables, Block 2 only has stress_xx and stress_yy
    let mut truth_table = TruthTable::new(EntityType::ElemBlock, 2, 3);
    truth_table.set(1, 2, false); // Block 2, var 3 (strain_energy) = false

    println!("\n  ✓ Truth table (which blocks have which variables):");
    println!("    Block 100: stress_xx ✓, stress_yy ✓, strain_energy ✓");
    println!("    Block 200: stress_xx ✓, stress_yy ✓, strain_energy ✗");

    file.put_truth_table(EntityType::ElemBlock, &truth_table)?;

    // Write variable data for 2 time steps
    for time_step in 0..2 {
        let time = (time_step + 1) as f64 * 0.25;
        file.put_time(time_step, time)?;

        // Block 1: stress_xx (4 elements)
        let stress_xx_1: Vec<f64> = (0..4).map(|i| 100.0 + time * (10.0 + i as f64)).collect();
        file.put_var(time_step, EntityType::ElemBlock, 100, 0, &stress_xx_1)?;

        // Block 1: stress_yy (4 elements)
        let stress_yy_1: Vec<f64> = (0..4).map(|i| 80.0 + time * (8.0 + i as f64)).collect();
        file.put_var(time_step, EntityType::ElemBlock, 100, 1, &stress_yy_1)?;

        // Block 1: strain_energy (4 elements)
        let strain_energy_1: Vec<f64> = (0..4).map(|i| 50.0 + time * (5.0 + i as f64)).collect();
        file.put_var(time_step, EntityType::ElemBlock, 100, 2, &strain_energy_1)?;

        // Block 2: stress_xx (2 elements)
        let stress_xx_2: Vec<f64> = (0..2).map(|i| 90.0 + time * (9.0 + i as f64)).collect();
        file.put_var(time_step, EntityType::ElemBlock, 200, 0, &stress_xx_2)?;

        // Block 2: stress_yy (2 elements)
        let stress_yy_2: Vec<f64> = (0..2).map(|i| 70.0 + time * (7.0 + i as f64)).collect();
        file.put_var(time_step, EntityType::ElemBlock, 200, 1, &stress_yy_2)?;

        // Note: Block 2 does not have strain_energy (truth table = false)
    }

    println!("\n  ✓ Wrote values for 2 time steps");
    Ok(())
}

/// Write time series data efficiently
fn write_time_series() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = ExodusFile::create_default("time_series.exo")?;

    // Initialize
    let params = InitParams {
        title: "Time Series Example".into(),
        num_dim: 2,
        num_nodes: 4,
        ..Default::default()
    };
    file.init(&params)?;

    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0];
    file.put_coords(&x, Some(&y), None)?;

    // Define nodal variable
    file.define_variables(EntityType::Nodal, &["pressure"])?;

    println!("  ✓ Writing 10 time steps...");

    // Write 10 time steps
    for time_step in 0..10 {
        let time = (time_step + 1) as f64 * 0.1;
        file.put_time(time_step, time)?;

        // Simulate pressure wave
        let pressure: Vec<f64> = (0..4)
            .map(|i| 100.0 * (time * 10.0 + i as f64 * 0.5).sin())
            .collect();
        file.put_var(time_step, EntityType::Nodal, 0, 0, &pressure)?;

        if (time_step + 1) % 5 == 0 {
            println!("    Step {}: t = {:.1}s", time_step + 1, time);
        }
    }

    println!("\n  ✓ Completed time series (10 steps)");
    Ok(())
}

/// Read variables from file
fn read_variables() -> Result<(), Box<dyn std::error::Error>> {
    // Read global variables
    println!("  Reading global variables:");
    {
        let file = ExodusFile::<mode::Read>::open("global_vars.exo")?;
        let var_names = file.variable_names(EntityType::Global)?;
        let num_time_steps = file.num_time_steps()?;

        println!("    Variables: {:?}", var_names);
        println!("    Time steps: {}", num_time_steps);

        for time_step in 0..num_time_steps {
            let time = file.time(time_step)?;
            let energy = file.var(time_step, EntityType::Global, 0, 1)?; // total_energy
            println!("      Step {}: t={:.2}, energy={:.2}", time_step + 1, time, energy[0]);
        }
    }

    // Read nodal variables
    println!("\n  Reading nodal variables:");
    {
        let file = ExodusFile::<mode::Read>::open("nodal_vars.exo")?;
        let var_names = file.variable_names(EntityType::Nodal)?;
        println!("    Variables: {:?}", var_names);

        let time_step = 1;
        let temp = file.var(time_step, EntityType::Nodal, 0, 0)?; // temperature
        println!("\n    Temperature at step {} (9 nodes):", time_step + 1);
        println!("      Min: {:.1}°C, Max: {:.1}°C, Avg: {:.1}°C",
            temp.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            temp.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            temp.iter().sum::<f64>() / temp.len() as f64
        );
    }

    // Read element variables
    println!("\n  Reading element variables:");
    {
        let file = ExodusFile::<mode::Read>::open("element_vars.exo")?;
        let var_names = file.variable_names(EntityType::ElemBlock)?;
        println!("    Variables: {:?}", var_names);

        // Check truth table
        let truth_table = file.truth_table(EntityType::ElemBlock)?;
        println!("\n    Truth table:");
        for block_idx in 0..truth_table.num_blocks {
            print!("      Block {}: ", block_idx);
            for var_idx in 0..truth_table.num_vars {
                let has_var = truth_table.get(block_idx, var_idx);
                print!("{} ", if has_var { "✓" } else { "✗" });
            }
            println!();
        }

        let time_step = 0;
        let stress_xx = file.var(time_step, EntityType::ElemBlock, 100, 0)?; // Block 100
        println!("\n    Stress XX at block 100, step {}:", time_step + 1);
        println!("      Values: {:?}", stress_xx);
    }

    // Read time series
    println!("\n  Reading time series:");
    {
        let file = ExodusFile::<mode::Read>::open("time_series.exo")?;
        let all_times = file.times()?;
        println!("    Total time steps: {}", all_times.len());
        println!("    Time range: {:.2}s to {:.2}s", all_times[0], all_times[all_times.len() - 1]);

        // Sample times at steps 1, 5, 10
        for &step in &[0, 4, 9] {
            let time = file.time(step)?;
            let pressure = file.var(step, EntityType::Nodal, 0, 0)?;
            let max_pressure = pressure.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            println!("      Step {}: t={:.1}s, max_pressure={:.1}", step + 1, time, max_pressure);
        }
    }

    Ok(())
}
