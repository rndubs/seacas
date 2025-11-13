//! QA and Info record compatibility tests
//!
//! Tests that QA records and info records are correctly written
//! and can be read by the C Exodus library.

use anyhow::Result;
use exodus_rs::{CreateMode, CreateOptions, ExodusFile, InitParams, InfoRecord, QaRecord};
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
        num_elem: 1,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Add multiple QA records
    let qa_records = vec![
        QaRecord {
            code: "exodus-rs".to_string(),
            version: "1.0.0".to_string(),
            date: "2025-11-13".to_string(),
            time: "12:00:00".to_string(),
        },
        QaRecord {
            code: "rust-to-c-test".to_string(),
            version: "0.1.0".to_string(),
            date: "2025-11-13".to_string(),
            time: "12:00:01".to_string(),
        },
        QaRecord {
            code: "compatibility".to_string(),
            version: "1.0".to_string(),
            date: "2025-11-13".to_string(),
            time: "12:00:02".to_string(),
        },
    ];

    file.put_qa_records(&qa_records)?;

    // Write coordinates
    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0];
    file.put_coords(&x, &y, &[])?;

    // Write element block
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        1,
        "QUAD4",
        1,
        4,
        0,
        0,
    )?;

    let connectivity = vec![1, 2, 3, 4];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 1, &connectivity)?;

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
        num_elem: 1,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Add multiple info records
    let info_records = vec![
        InfoRecord {
            text: "This file tests info record compatibility".to_string(),
        },
        InfoRecord {
            text: "Info records store arbitrary text information".to_string(),
        },
        InfoRecord {
            text: "Line 3: Created by exodus-rs Rust implementation".to_string(),
        },
        InfoRecord {
            text: "Line 4: Testing C library interoperability".to_string(),
        },
        InfoRecord {
            text: "Line 5: Should be readable by C Exodus library".to_string(),
        },
    ];

    file.put_info_records(&info_records)?;

    // Write coordinates
    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0];
    file.put_coords(&x, &y, &[])?;

    // Write element block
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        1,
        "QUAD4",
        1,
        4,
        0,
        0,
    )?;

    let connectivity = vec![1, 2, 3, 4];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 1, &connectivity)?;

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
        num_elem: 1,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Add QA records
    let qa_records = vec![
        QaRecord {
            code: "exodus-rs".to_string(),
            version: "1.0.0".to_string(),
            date: "2025-11-13".to_string(),
            time: "12:00:00".to_string(),
        },
        QaRecord {
            code: "combined-test".to_string(),
            version: "0.1.0".to_string(),
            date: "2025-11-13".to_string(),
            time: "12:00:01".to_string(),
        },
    ];

    file.put_qa_records(&qa_records)?;

    // Add info records
    let info_records = vec![
        InfoRecord {
            text: "This file contains both QA and info records".to_string(),
        },
        InfoRecord {
            text: "Testing combined metadata functionality".to_string(),
        },
        InfoRecord {
            text: "Should preserve both record types".to_string(),
        },
    ];

    file.put_info_records(&info_records)?;

    // Write coordinates
    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0];
    file.put_coords(&x, &y, &[])?;

    // Write element block
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        1,
        "QUAD4",
        1,
        4,
        0,
        0,
    )?;

    let connectivity = vec![1, 2, 3, 4];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 1, &connectivity)?;

    Ok(())
}
