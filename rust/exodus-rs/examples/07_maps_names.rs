//! Example 07: Maps and Names
//!
//! This example demonstrates Phase 7 functionality:
//! - Entity ID maps
//! - Element order maps
//! - Entity naming
//! - Property arrays

use exodus_rs::{
    mode, CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, Result,
};

fn main() -> Result<()> {
    println!("=== Phase 7: Maps and Names Example ===\n");

    let filename = "example_07_maps_names.exo";

    // ========================================================================
    // Part 1: Create file with maps and names
    // ========================================================================
    println!("1. Creating Exodus file with maps and names...");
    {
        let mut file = ExodusFile::create(
            filename,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )?;

        // Initialize with some entities
        let params = InitParams {
            title: "Maps and Names Example".into(),
            num_dim: 3,
            num_nodes: 8,
            num_elems: 2,
            num_elem_blocks: 2,
            ..Default::default()
        };
        file.init(&params)?;

        // ====================================================================
        // ID Maps
        // ====================================================================
        println!("   - Setting custom node numbering (100-107)...");
        let node_map = vec![100, 101, 102, 103, 104, 105, 106, 107];
        file.put_id_map(EntityType::NodeMap, &node_map)?;

        println!("   - Setting custom element numbering (1000, 2000)...");
        let elem_map = vec![1000, 2000];
        file.put_id_map(EntityType::ElemMap, &elem_map)?;

        // ====================================================================
        // Order Maps
        // ====================================================================
        println!("   - Setting element processing order (reverse)...");
        let elem_order = vec![2, 1];
        file.put_elem_order_map(&elem_order)?;

        // ====================================================================
        // Entity Names
        // ====================================================================
        println!("   - Naming element blocks...");
        file.put_names(EntityType::ElemBlock, &["Steel_Block", "Aluminum_Block"])?;

        // ====================================================================
        // Property Arrays
        // ====================================================================
        println!("   - Setting material IDs...");
        file.put_property_array(EntityType::ElemBlock, "MATERIAL_ID", &[1, 2])?;

        println!("   - Setting processor IDs...");
        file.put_property_array(EntityType::ElemBlock, "PROCESSOR_ID", &[0, 1])?;

        println!("File created successfully!");
    }

    // ========================================================================
    // Part 2: Read back the maps and names
    // ========================================================================
    println!("\n2. Reading maps and names from file...");
    {
        let file = ExodusFile::<mode::Read>::open(filename)?;

        // ====================================================================
        // Read ID Maps
        // ====================================================================
        println!("   - Node ID map:");
        let node_map = file.id_map(EntityType::NodeMap)?;
        println!("     {:?}", node_map);

        println!("   - Element ID map:");
        let elem_map = file.id_map(EntityType::ElemMap)?;
        println!("     {:?}", elem_map);

        // ====================================================================
        // Read Order Maps
        // ====================================================================
        println!("   - Element order map:");
        let elem_order = file.elem_order_map()?;
        println!("     {:?}", elem_order);

        // ====================================================================
        // Read Entity Names
        // ====================================================================
        println!("   - Element block names:");
        let block_names = file.names(EntityType::ElemBlock)?;
        for (i, name) in block_names.iter().enumerate() {
            println!("     Block {}: {}", i + 1, name);
        }

        // Read individual name
        let name = file.name(EntityType::ElemBlock, 0)?;
        println!("   - First block name (via index): {}", name);

        // ====================================================================
        // Read Property Arrays
        // ====================================================================
        println!("   - Material IDs:");
        let material_ids = file.property_array(EntityType::ElemBlock, "MATERIAL_ID")?;
        println!("     {:?}", material_ids);

        println!("   - Processor IDs:");
        let proc_ids = file.property_array(EntityType::ElemBlock, "PROCESSOR_ID")?;
        println!("     {:?}", proc_ids);

        println!("   - All property names for element blocks:");
        let prop_names = file.property_names(EntityType::ElemBlock)?;
        println!("     {:?}", prop_names);
    }

    println!("\n=== Example completed successfully! ===");
    println!("Output file: {}", filename);

    Ok(())
}
