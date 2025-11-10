//! Example 10: NetCDF Define Mode Operations
//!
//! This example demonstrates best practices for managing NetCDF define/data
//! mode transitions when working with Exodus files.
//!
//! # NetCDF Mode Overview
//!
//! NetCDF files operate in two modes:
//! - **Define Mode**: Add dimensions, variables, and attributes
//! - **Data Mode**: Write data to variables
//!
//! While NetCDF-4 format is flexible and allows automatic mode switching,
//! explicitly managing modes can improve performance and code clarity.
//!
//! # Best Practices
//!
//! 1. Group all definitions together
//! 2. Call `end_define()` before writing data
//! 3. Only `reenter_define()` if absolutely necessary
//! 4. Use `sync()` to ensure data is flushed to disk
//!
//! # Running
//!
//! ```bash
//! cargo run --example 10_define_mode_operations --features netcdf4
//! ```

use exodus_rs::error::Result;
use exodus_rs::types::{Block, CreateMode, CreateOptions, EntityType, InitParams, QaRecord};
use exodus_rs::ExodusFile;

fn main() -> Result<()> {
    println!("=== Example 10: NetCDF Define Mode Operations ===\n");

    // Example 1: Recommended workflow - all definitions upfront
    println!("Example 1: Recommended workflow (all definitions first)");
    recommended_workflow()?;

    // Example 2: Re-entering define mode (less efficient but sometimes necessary)
    println!("\nExample 2: Re-entering define mode (less efficient)");
    reenter_define_workflow()?;

    // Example 3: Using is_define_mode() to check current state
    println!("\nExample 3: Checking define mode state");
    check_define_mode_state()?;

    println!("\n=== All examples completed successfully! ===");
    Ok(())
}

/// Example 1: Recommended workflow - define everything upfront
fn recommended_workflow() -> Result<()> {
    let file = tempfile::NamedTempFile::new().unwrap();
    let mut exo = ExodusFile::create(
        file.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )?;

    println!("  Step 1: File created - in define mode");
    assert!(exo.is_define_mode());

    // ===================================================================
    // DEFINE MODE: Define all structure upfront
    // ===================================================================

    println!("  Step 2: Initialize database parameters");
    let params = InitParams {
        title: "Define Mode Example".into(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };
    exo.init(&params)?;

    println!("  Step 3: Define element block");
    let block = Block {
        id: 100,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".into(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    exo.put_block(&block)?;

    println!("  Step 4: Define variables");
    exo.define_variables(EntityType::Nodal, &["Temperature", "Pressure"])?;
    exo.define_variables(EntityType::ElemBlock, &["Stress"])?;

    // Optional: Add QA records
    let qa = QaRecord {
        code_name: "exodus-rs".into(),
        code_version: "1.0".into(),
        date: "2025-01-01".into(),
        time: "12:00:00".into(),
    };
    exo.put_qa_records(&[qa])?;

    println!("  Step 5: End define mode");
    exo.end_define()?;
    assert!(!exo.is_define_mode());

    // ===================================================================
    // DATA MODE: Write all data values
    // ===================================================================

    println!("  Step 6: Write coordinates");
    let x = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y = vec![0.0_f64, 0.0, 1.0, 1.0];
    exo.put_coords(&x, Some(&y), None)?;

    println!("  Step 7: Write connectivity");
    let conn = vec![1_i64, 2, 3, 4];
    exo.put_connectivity(100, &conn)?;

    println!("  Step 8: Write time steps and variable values");
    exo.put_time(0, 0.0)?;
    let temp_vals = vec![100.0_f64, 150.0, 200.0, 175.0];
    let press_vals = vec![1.0_f64, 1.5, 2.0, 1.75];
    let stress_vals = vec![50.0_f64];

    exo.put_var(0, EntityType::Nodal, 0, 0, &temp_vals)?;
    exo.put_var(0, EntityType::Nodal, 1, 0, &press_vals)?;
    exo.put_var(0, EntityType::ElemBlock, 100, 0, &stress_vals)?;

    println!("  Step 9: Sync to ensure data is written");
    exo.sync()?;

    println!("  ✓ Recommended workflow completed successfully");
    Ok(())
}

/// Example 2: Re-entering define mode (less efficient but sometimes necessary)
fn reenter_define_workflow() -> Result<()> {
    let file = tempfile::NamedTempFile::new().unwrap();
    let mut exo = ExodusFile::create(
        file.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )?;

    println!("  Step 1: Initial definitions");
    let params = InitParams {
        title: "Reenter Define Mode Example".into(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };
    exo.init(&params)?;

    let block = Block {
        id: 100,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".into(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    exo.put_block(&block)?;

    println!("  Step 2: Define variables");
    exo.define_variables(EntityType::Nodal, &["Temperature"])?;

    println!("  Step 3: End define mode");
    exo.end_define()?;

    println!("  Step 4: Write coordinates");
    let x = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y = vec![0.0_f64, 0.0, 1.0, 1.0];
    exo.put_coords(&x, Some(&y), None)?;

    println!("  Step 5: Write connectivity");
    let conn = vec![1_i64, 2, 3, 4];
    exo.put_connectivity(100, &conn)?;

    println!("  Step 6: Write variable data");
    exo.put_time(0, 0.0)?;
    let temp_vals = vec![100.0_f64, 150.0, 200.0, 175.0];
    exo.put_var(0, EntityType::Nodal, 0, 0, &temp_vals)?;

    // Demonstrate that we can re-enter define mode if needed
    println!("  Step 7: Re-enter define mode to add QA record");
    exo.reenter_define()?;
    assert!(exo.is_define_mode());

    println!("  Step 8: Add QA record after writing data");
    let qa = QaRecord {
        code_name: "post-processor".into(),
        code_version: "2.0".into(),
        date: "2025-01-02".into(),
        time: "13:00:00".into(),
    };
    exo.put_qa_records(&[qa])?;

    println!("  Step 9: End define mode again");
    exo.end_define()?;

    println!("  Step 10: Sync to ensure all data is written");
    exo.sync()?;

    println!("  ✓ Re-enter define mode workflow completed");
    println!("  Note: This approach works but is less efficient than defining everything upfront");
    Ok(())
}

/// Example 3: Checking define mode state
fn check_define_mode_state() -> Result<()> {
    let file = tempfile::NamedTempFile::new().unwrap();
    let mut exo = ExodusFile::create(
        file.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )?;

    println!("  Checking mode states during workflow:");

    println!("    After creation: {}", mode_str(&exo));
    assert!(exo.is_define_mode());

    let params = InitParams {
        title: "Mode State Example".into(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };
    exo.init(&params)?;

    println!("    After init: {}", mode_str(&exo));
    assert!(exo.is_define_mode());

    exo.end_define()?;
    println!("    After end_define: {}", mode_str(&exo));
    assert!(!exo.is_define_mode());

    let x = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y = vec![0.0_f64, 0.0, 1.0, 1.0];
    exo.put_coords(&x, Some(&y), None)?;

    println!("    After writing data: {}", mode_str(&exo));
    assert!(!exo.is_define_mode());

    exo.reenter_define()?;
    println!("    After reenter_define: {}", mode_str(&exo));
    assert!(exo.is_define_mode());

    println!("  ✓ Mode state checking completed");
    Ok(())
}

/// Helper to get mode string
fn mode_str(exo: &ExodusFile<exodus_rs::mode::Write>) -> &'static str {
    if exo.is_define_mode() {
        "Define Mode"
    } else {
        "Data Mode"
    }
}
