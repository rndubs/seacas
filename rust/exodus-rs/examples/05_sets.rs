//! Example 05: Sets (Node Sets, Side Sets, Element Sets)
//!
//! This example demonstrates how to create and use different types of sets in Exodus files.
//! It shows:
//! - Creating node sets (with and without distribution factors)
//! - Creating side sets for boundary conditions
//! - Creating element sets for grouping
//! - Iterating over sets
//! - Reading set information

use exodus_rs::{mode, EntityType, ExodusFile, InitParams, Set};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Exodus Sets Example ===\n");

    // Example 1: Create node sets
    println!("1. Creating node sets");
    create_node_sets()?;

    // Example 2: Create side sets
    println!("\n2. Creating side sets for boundary conditions");
    create_side_sets()?;

    // Example 3: Create element sets
    println!("\n3. Creating element sets");
    create_element_sets()?;

    // Example 4: Read and iterate over sets
    println!("\n4. Reading set information");
    read_sets()?;

    // Example 5: Distribution factors
    println!("\n5. Using distribution factors");
    distribution_factors_example()?;

    println!("\n=== Example Complete ===");
    Ok(())
}

/// Create node sets for marking specific groups of nodes
fn create_node_sets() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = ExodusFile::create_default("node_sets.exo")?;

    // Initialize with a simple 2D mesh
    let params = InitParams {
        title: "Node Sets Example".into(),
        num_dim: 2,
        num_nodes: 9,
        num_node_sets: 3,
        ..Default::default()
    };
    file.init(&params)?;

    // Write coordinates for a 3x3 grid
    let x = vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];
    file.put_coords(&x, Some(&y), None)?;

    // Node set 1: Left edge nodes (1, 4, 7)
    let set1 = Set {
        id: 10,
        entity_type: EntityType::NodeSet,
        num_entries: 3,
        num_dist_factors: 0,
    };
    file.put_set(&set1)?;
    file.put_node_set(10, &vec![1, 4, 7], None)?;

    // Node set 2: Bottom edge nodes (1, 2, 3)
    let set2 = Set {
        id: 20,
        entity_type: EntityType::NodeSet,
        num_entries: 3,
        num_dist_factors: 0,
    };
    file.put_set(&set2)?;
    file.put_node_set(20, &vec![1, 2, 3], None)?;

    // Node set 3: Corner nodes (1, 3, 7, 9)
    let set3 = Set {
        id: 30,
        entity_type: EntityType::NodeSet,
        num_entries: 4,
        num_dist_factors: 0,
    };
    file.put_set(&set3)?;
    file.put_node_set(30, &vec![1, 3, 7, 9], None)?;

    println!("  ✓ Created 3 node sets:");
    println!("    - Set 10: Left edge (3 nodes)");
    println!("    - Set 20: Bottom edge (3 nodes)");
    println!("    - Set 30: Corners (4 nodes)");
    Ok(())
}

/// Create side sets for boundary conditions
fn create_side_sets() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = ExodusFile::create_default("side_sets.exo")?;

    // Initialize with a quad mesh
    let params = InitParams {
        title: "Side Sets Example".into(),
        num_dim: 2,
        num_nodes: 6,
        num_elems: 2,
        num_elem_blocks: 1,
        num_side_sets: 2,
        ..Default::default()
    };
    file.init(&params)?;

    // Write coordinates
    let x = vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    file.put_coords(&x, Some(&y), None)?;

    // Create element block
    use exodus_rs::Block;
    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".into(),
        num_entries: 2,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &vec![1, 2, 5, 4, 2, 3, 6, 5])?;

    // Side set 1: Left boundary (element 1, side 4)
    let ss1 = Set {
        id: 100,
        entity_type: EntityType::SideSet,
        num_entries: 1,
        num_dist_factors: 0,
    };
    file.put_set(&ss1)?;
    file.put_side_set(100, &vec![1], &vec![4], None)?;

    // Side set 2: Bottom boundary (both elements, side 1)
    let ss2 = Set {
        id: 200,
        entity_type: EntityType::SideSet,
        num_entries: 2,
        num_dist_factors: 0,
    };
    file.put_set(&ss2)?;
    file.put_side_set(200, &vec![1, 2], &vec![1, 1], None)?;

    println!("  ✓ Created 2 side sets:");
    println!("    - Set 100: Left boundary (1 side)");
    println!("    - Set 200: Bottom boundary (2 sides)");
    Ok(())
}

/// Create element sets for grouping elements
fn create_element_sets() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = ExodusFile::create_default("elem_sets.exo")?;

    // Initialize with multiple element blocks
    let params = InitParams {
        title: "Element Sets Example".into(),
        num_dim: 3,
        num_nodes: 20,
        num_elems: 6,
        num_elem_blocks: 2,
        num_elem_sets: 2,
        ..Default::default()
    };
    file.init(&params)?;

    // Simplified coordinates
    let coords: Vec<f64> = (0..20).map(|i| i as f64 * 0.1).collect();
    file.put_coords(&coords, Some(&coords), Some(&coords))?;

    // Element set 1: First group of elements (1, 2, 3)
    let es1 = Set {
        id: 1000,
        entity_type: EntityType::ElemSet,
        num_entries: 3,
        num_dist_factors: 0,
    };
    file.put_set(&es1)?;
    file.put_entity_set(EntityType::ElemSet, 1000, &vec![1, 2, 3])?;

    // Element set 2: Second group of elements (4, 5, 6)
    let es2 = Set {
        id: 2000,
        entity_type: EntityType::ElemSet,
        num_entries: 3,
        num_dist_factors: 0,
    };
    file.put_set(&es2)?;
    file.put_entity_set(EntityType::ElemSet, 2000, &vec![4, 5, 6])?;

    println!("  ✓ Created 2 element sets:");
    println!("    - Set 1000: Elements 1-3");
    println!("    - Set 2000: Elements 4-6");
    Ok(())
}

/// Read and iterate over sets
fn read_sets() -> Result<(), Box<dyn std::error::Error>> {
    let file = ExodusFile::<mode::Read>::open("node_sets.exo")?;

    println!("  Reading node sets from file:");

    // Method 1: Get all set IDs and iterate
    let set_ids = file.set_ids(EntityType::NodeSet)?;
    println!("\n  Method 1: Using set_ids()");
    println!("    Found {} node set(s): {:?}", set_ids.len(), set_ids);

    for set_id in &set_ids {
        let node_set = file.node_set(*set_id)?;
        println!("\n    Node Set ID: {}", node_set.id);
        println!("      Nodes: {:?}", node_set.nodes);
        println!("      Node count: {}", node_set.nodes.len());
    }

    // Method 2: Using the iterator
    println!("\n  Method 2: Using sets() iterator");
    for set_id in file.sets(EntityType::NodeSet)? {
        let node_set = file.node_set(set_id)?;
        println!("    Set {}: {} nodes", set_id, node_set.nodes.len());
    }

    // Method 3: Get set parameters
    println!("\n  Method 3: Using set() for parameters only");
    for set_id in &set_ids {
        let set_params = file.set(EntityType::NodeSet, *set_id)?;
        println!(
            "    Set {}: type={}, entries={}, dist_factors={}",
            set_params.id,
            set_params.entity_type,
            set_params.num_entries,
            set_params.num_dist_factors
        );
    }

    Ok(())
}

/// Demonstrate distribution factors
fn distribution_factors_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = ExodusFile::create_default("dist_factors.exo")?;

    // Initialize
    let params = InitParams {
        title: "Distribution Factors Example".into(),
        num_dim: 2,
        num_nodes: 4,
        num_node_sets: 1,
        ..Default::default()
    };
    file.init(&params)?;

    // Coordinates
    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0];
    file.put_coords(&x, Some(&y), None)?;

    // Node set with distribution factors (for boundary conditions)
    // Distribution factors can be used to weight nodes differently
    let set = Set {
        id: 50,
        entity_type: EntityType::NodeSet,
        num_entries: 4,
        num_dist_factors: 4,
    };
    file.put_set(&set)?;

    let nodes = vec![1, 2, 3, 4];
    let dist_factors = vec![1.0, 0.5, 0.5, 1.0]; // Corner nodes weighted more
    file.put_node_set(50, &nodes, Some(&dist_factors))?;

    println!("  ✓ Created node set with distribution factors:");
    println!("    Nodes: {:?}", nodes);
    println!("    Distribution factors: {:?}", dist_factors);

    // Read back
    let file = ExodusFile::<mode::Read>::open("dist_factors.exo")?;
    let node_set = file.node_set(50)?;
    println!("\n  Read back:");
    println!("    Nodes: {:?}", node_set.nodes);
    println!("    Distribution factors: {:?}", node_set.dist_factors);

    // Example: Apply weighted boundary condition
    println!("\n  Example application: Weighted force on boundary");
    let total_force = 100.0;
    let sum_factors: f64 = node_set.dist_factors.iter().sum();

    for (_i, (&node, &factor)) in node_set
        .nodes
        .iter()
        .zip(&node_set.dist_factors)
        .enumerate()
    {
        let node_force = total_force * factor / sum_factors;
        println!("    Node {}: Force = {:.2}", node, node_force);
    }

    Ok(())
}
