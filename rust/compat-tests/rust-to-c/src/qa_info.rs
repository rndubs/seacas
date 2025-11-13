//! QA and Info record compatibility tests
//!
//! Tests that QA records and info records are correctly written
//! and can be read by the C Exodus library.

use anyhow::Result;
use exodus_rs::{Block, CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, QaRecord, Topology};
use std::path::Path;

/// Generate a file with QA records
pub fn generate_qa_records(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with basic 2D mesh
    let params = InitParams {
        title: "QA Records Test".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Add multiple QA records
    let qa_records = vec![
        QaRecord {
            code_name: "exodus-rs".to_string(),
            code_version: "1.0.0".to_string(),
            date: "2025-11-13".to_string(),
            time: "12:00:00".to_string(),
        },
        QaRecord {
            code_name: "rust-to-c-test".to_string(),
            code_version: "0.1.0".to_string(),
            date: "2025-11-13".to_string(),
            time: "12:00:01".to_string(),
        },
        QaRecord {
            code_name: "compatibility".to_string(),
            code_version: "1.0".to_string(),
            date: "2025-11-13".to_string(),
            time: "12:00:02".to_string(),
        },
    ];

    file.put_qa_records(&qa_records)?;

    // Write coordinates
    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0];
    file.put_coords(&x, Some(&y), None)?;

    // Write element block
    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: Topology::Quad4.to_string(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;

    let connectivity = vec![1_i64, 2, 3, 4];
    file.put_connectivity(1, &connectivity)?;

    Ok(())
}

/// Generate a file with info records
pub fn generate_info_records(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with basic 2D mesh
    let params = InitParams {
        title: "Info Records Test".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Add multiple info records (InfoRecord is just a String)
    let info_records = vec![
        "This file tests info record compatibility".to_string(),
        "Info records store arbitrary text information".to_string(),
        "Line 3: Created by exodus-rs Rust implementation".to_string(),
        "Line 4: Testing C library interoperability".to_string(),
        "Line 5: Should be readable by C Exodus library".to_string(),
    ];

    file.put_info_records(&info_records)?;

    // Write coordinates
    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0];
    file.put_coords(&x, Some(&y), None)?;

    // Write element block
    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: Topology::Quad4.to_string(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;

    let connectivity = vec![1_i64, 2, 3, 4];
    file.put_connectivity(1, &connectivity)?;

    Ok(())
}

/// Generate a file with both QA and info records
pub fn generate_qa_and_info(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with basic 2D mesh
    let params = InitParams {
        title: "QA and Info Records Test".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Add QA records
    let qa_records = vec![
        QaRecord {
            code_name: "exodus-rs".to_string(),
            code_version: "1.0.0".to_string(),
            date: "2025-11-13".to_string(),
            time: "12:00:00".to_string(),
        },
        QaRecord {
            code_name: "combined-test".to_string(),
            code_version: "0.1.0".to_string(),
            date: "2025-11-13".to_string(),
            time: "12:00:01".to_string(),
        },
    ];

    file.put_qa_records(&qa_records)?;

    // Add info records
    let info_records = vec![
        "This file contains both QA and info records".to_string(),
        "Testing combined metadata functionality".to_string(),
        "Should preserve both record types".to_string(),
    ];

    file.put_info_records(&info_records)?;

    // Write coordinates
    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0];
    file.put_coords(&x, Some(&y), None)?;

    // Write element block
    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: Topology::Quad4.to_string(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;

    let connectivity = vec![1_i64, 2, 3, 4];
    file.put_connectivity(1, &connectivity)?;

    Ok(())
}
