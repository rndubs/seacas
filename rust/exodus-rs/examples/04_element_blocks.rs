//! Example 04: Element Blocks
//!
//! This example demonstrates how to create element blocks and write connectivity data.
//! It shows:
//! - Creating different types of element blocks (hex, quad, tet)
//! - Writing connectivity arrays
//! - Reading block information back
//! - Iterating over connectivity data

use exodus_rs::{
    Block, EntityType, ExodusFile, InitParams, Topology,
    mode,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Exodus Element Blocks Example ===\n");

    // Example 1: Create a simple hex mesh
    println!("1. Creating a simple hex mesh with 1 element");
    create_hex_mesh()?;

    // Example 2: Create a quad mesh
    println!("\n2. Creating a quad mesh with 2 elements");
    create_quad_mesh()?;

    // Example 3: Create a mixed mesh
    println!("\n3. Creating a mixed mesh with different element types");
    create_mixed_mesh()?;

    // Example 4: Read and inspect blocks
    println!("\n4. Reading block information");
    read_blocks()?;

    println!("\n=== Example Complete ===");
    Ok(())
}

/// Create a simple mesh with one hex element
fn create_hex_mesh() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = ExodusFile::create_default("hex_mesh.exo")?;

    // Initialize the database
    let params = InitParams {
        title: "Single Hex Element".into(),
        num_dim: 3,
        num_nodes: 8,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };
    file.init(&params)?;

    // Write coordinates for a unit cube
    let x = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
    let z = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    file.put_coords(&x, Some(&y), Some(&z))?;

    // Define hex block
    let block = Block {
        id: 100,
        entity_type: EntityType::ElemBlock,
        topology: "HEX8".into(),
        num_entries: 1,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;

    // Write connectivity (1-based node IDs)
    let conn = vec![1, 2, 3, 4, 5, 6, 7, 8];
    file.put_connectivity(100, &conn)?;

    println!("  ✓ Created hex_mesh.exo with 1 HEX8 element");
    Ok(())
}

/// Create a quad mesh with multiple elements
fn create_quad_mesh() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = ExodusFile::create_default("quad_mesh.exo")?;

    // Initialize the database
    let params = InitParams {
        title: "Quad Mesh".into(),
        num_dim: 2,
        num_nodes: 6,
        num_elems: 2,
        num_elem_blocks: 1,
        ..Default::default()
    };
    file.init(&params)?;

    // Write coordinates for 2x1 rectangular mesh
    let x = vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let z = vec![0.0; 6];
    file.put_coords(&x, Some(&y), Some(&z))?;

    // Define quad block
    let block = Block {
        id: 10,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".into(),
        num_entries: 2,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;

    // Write connectivity for 2 elements
    // Element 1: nodes 1,2,5,4
    // Element 2: nodes 2,3,6,5
    let conn = vec![
        1, 2, 5, 4,  // Element 1
        2, 3, 6, 5,  // Element 2
    ];
    file.put_connectivity(10, &conn)?;

    println!("  ✓ Created quad_mesh.exo with 2 QUAD4 elements");
    Ok(())
}

/// Create a mesh with different element types
fn create_mixed_mesh() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = ExodusFile::create_default("mixed_mesh.exo")?;

    // Initialize with 2 element blocks
    let params = InitParams {
        title: "Mixed Element Mesh".into(),
        num_dim: 3,
        num_nodes: 14,
        num_elems: 3,
        num_elem_blocks: 2,
        ..Default::default()
    };
    file.init(&params)?;

    // Write coordinates (simplified for example)
    let x = vec![0.0, 1.0, 0.5, 0.5, 2.0, 3.0, 3.0, 2.0, 2.0, 3.0, 3.0, 2.0, 2.0, 3.0];
    let y = vec![0.0, 0.0, 1.0, 0.5, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.5, 0.5];
    let z = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.5, 0.5];
    file.put_coords(&x, Some(&y), Some(&z))?;

    // Block 1: Tetrahedra
    let tet_block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "TET4".into(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&tet_block)?;
    file.put_connectivity(1, &vec![1, 2, 3, 4])?;

    // Block 2: Hexahedra
    let hex_block = Block {
        id: 2,
        entity_type: EntityType::ElemBlock,
        topology: "HEX8".into(),
        num_entries: 2,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&hex_block)?;
    file.put_connectivity(2, &vec![
        5, 6, 7, 8, 9, 10, 11, 12,   // Hex 1
        9, 10, 11, 12, 5, 6, 7, 13,  // Hex 2 (simplified)
    ])?;

    println!("  ✓ Created mixed_mesh.exo with TET4 and HEX8 elements");
    Ok(())
}

/// Read and display block information
fn read_blocks() -> Result<(), Box<dyn std::error::Error>> {
    // Read the quad mesh
    let file = ExodusFile::<mode::Read>::open("quad_mesh.exo")?;

    // Get all element block IDs
    let block_ids = file.block_ids(EntityType::ElemBlock)?;
    println!("  Found {} element block(s)", block_ids.len());

    for block_id in block_ids {
        // Get block parameters
        let block = file.block(block_id)?;
        println!("\n  Block ID: {}", block.id);
        println!("    Topology: {}", block.topology);
        println!("    Elements: {}", block.num_entries);
        println!("    Nodes per element: {}", block.num_nodes_per_entry);

        // Get connectivity
        let conn = file.connectivity(block_id)?;
        println!("    Connectivity ({} values): {:?}", conn.len(), conn);

        // Get structured connectivity
        let conn_struct = file.connectivity_structured(block_id)?;
        println!("    Structured connectivity:");
        for (i, elem_conn) in conn_struct.iter().enumerate() {
            println!("      Element {}: {:?}", i + 1, elem_conn);
        }
    }

    // Demonstrate topology parsing
    println!("\n  Topology information:");
    let topology = Topology::from_str("QUAD4");
    println!("    QUAD4 expects {} nodes", topology.expected_nodes().unwrap());
    println!("    String representation: {}", topology);

    Ok(())
}
