//! Example 08: Assemblies, Blobs, and Attributes
//!
//! This example demonstrates Phase 8 functionality:
//! - Assemblies for hierarchical grouping
//! - Blobs for binary data storage
//! - Attributes for entity metadata (integer, double, and character types)

use exodus_rs::{
    mode, Assembly, AttributeData, AttributeType, Blob, Block, CreateMode, CreateOptions,
    EntityType, ExodusFile, InitParams, Result, Set,
};

fn main() -> Result<()> {
    println!("=== Phase 8: Assemblies, Blobs, and Attributes Example ===\n");

    let filename = "example_08_assemblies_blobs_attributes.exo";

    // ========================================================================
    // Part 1: Create file with assemblies, blobs, and attributes
    // ========================================================================
    println!("1. Creating Exodus file with assemblies, blobs, and attributes...");
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
            title: "Assemblies, Blobs, and Attributes Example".into(),
            num_dim: 3,
            num_nodes: 12,
            num_elems: 5,
            num_elem_blocks: 3,
            num_node_sets: 2,
            num_side_sets: 1,
            ..Default::default()
        };
        file.init(&params)?;

        // Define element blocks
        println!("\n   Creating element blocks...");
        file.put_block(&Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "HEX8".into(),
            num_entries: 2,
            num_nodes_per_entry: 8,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })?;

        file.put_block(&Block {
            id: 2,
            entity_type: EntityType::ElemBlock,
            topology: "HEX8".into(),
            num_entries: 2,
            num_nodes_per_entry: 8,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })?;

        file.put_block(&Block {
            id: 3,
            entity_type: EntityType::ElemBlock,
            topology: "HEX8".into(),
            num_entries: 1,
            num_nodes_per_entry: 8,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })?;

        // Define node sets
        println!("   Creating node sets...");
        file.put_set(&Set {
            id: 1,
            entity_type: EntityType::NodeSet,
            num_entries: 0,
            num_dist_factors: 0,
        })?;

        file.put_set(&Set {
            id: 2,
            entity_type: EntityType::NodeSet,
            num_entries: 0,
            num_dist_factors: 0,
        })?;

        // Define side set
        file.put_set(&Set {
            id: 10,
            entity_type: EntityType::SideSet,
            num_entries: 0,
            num_dist_factors: 0,
        })?;

        // ====================================================================
        // Assemblies - Hierarchical Grouping
        // ====================================================================
        println!("\n   Creating assemblies...");

        // Assembly 1: Group of element blocks representing steel components
        println!("   - Assembly 1: Steel components (elem blocks 1-2)");
        let steel_assembly = Assembly {
            id: 100,
            name: "Steel_Components".into(),
            entity_type: EntityType::ElemBlock,
            entity_list: vec![1, 2],
        };
        file.put_assembly(&steel_assembly)?;

        // Assembly 2: Group of element blocks representing aluminum components
        println!("   - Assembly 2: Aluminum components (elem block 3)");
        let aluminum_assembly = Assembly {
            id: 200,
            name: "Aluminum_Parts".into(),
            entity_type: EntityType::ElemBlock,
            entity_list: vec![3],
        };
        file.put_assembly(&aluminum_assembly)?;

        // Assembly 3: Group of node sets representing boundary conditions
        println!("   - Assembly 3: Boundary conditions (node sets 1-2)");
        let bc_assembly = Assembly {
            id: 300,
            name: "Boundary_Conditions".into(),
            entity_type: EntityType::NodeSet,
            entity_list: vec![1, 2],
        };
        file.put_assembly(&bc_assembly)?;

        // ====================================================================
        // Blobs - Binary Data Storage
        // ====================================================================
        println!("\n   Storing blobs...");

        // Blob 1: Material properties (JSON-like data)
        println!("   - Blob 1: Material properties");
        let material_data =
            br#"{"steel": {"E": 200e9, "nu": 0.3}, "aluminum": {"E": 70e9, "nu": 0.33}}"#;
        let material_blob = Blob {
            id: 1,
            name: "material_properties".into(),
        };
        file.put_blob(&material_blob, material_data)?;

        // Blob 2: Thumbnail image (fake PNG header)
        println!("   - Blob 2: Thumbnail image");
        let image_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG signature
        let thumbnail_blob = Blob {
            id: 2,
            name: "thumbnail".into(),
        };
        file.put_blob(&thumbnail_blob, &image_data)?;

        // Blob 3: Configuration data
        println!("   - Blob 3: Simulation config");
        let config_data = b"timesteps=100\ndt=0.01\noutput_freq=10";
        let config_blob = Blob {
            id: 3,
            name: "sim_config".into(),
        };
        file.put_blob(&config_blob, config_data)?;

        // ====================================================================
        // Attributes - Entity Metadata
        // ====================================================================
        println!("\n   Adding attributes...");

        // Integer attributes - material IDs for element blocks
        println!("   - Integer attributes: Material IDs");
        file.put_attribute(
            EntityType::ElemBlock,
            1,
            "material_id",
            AttributeType::Integer,
            AttributeData::Integer(vec![1]), // Steel
        )?;

        file.put_attribute(
            EntityType::ElemBlock,
            2,
            "material_id",
            AttributeType::Integer,
            AttributeData::Integer(vec![1]), // Steel
        )?;

        file.put_attribute(
            EntityType::ElemBlock,
            3,
            "material_id",
            AttributeType::Integer,
            AttributeData::Integer(vec![2]), // Aluminum
        )?;

        // Double attributes - material properties
        println!("   - Double attributes: Material properties");
        file.put_attribute(
            EntityType::ElemBlock,
            1,
            "density",
            AttributeType::Double,
            AttributeData::Double(vec![7850.0]), // kg/m^3 for steel
        )?;

        file.put_attribute(
            EntityType::ElemBlock,
            2,
            "density",
            AttributeType::Double,
            AttributeData::Double(vec![7850.0]), // kg/m^3 for steel
        )?;

        file.put_attribute(
            EntityType::ElemBlock,
            3,
            "density",
            AttributeType::Double,
            AttributeData::Double(vec![2700.0]), // kg/m^3 for aluminum
        )?;

        // Multi-value double attributes - elastic properties
        println!("   - Multi-value double attributes: Elastic properties (E, nu)");
        file.put_attribute(
            EntityType::ElemBlock,
            1,
            "elastic_props",
            AttributeType::Double,
            AttributeData::Double(vec![200e9, 0.3]), // E, nu for steel
        )?;

        file.put_attribute(
            EntityType::ElemBlock,
            3,
            "elastic_props",
            AttributeType::Double,
            AttributeData::Double(vec![70e9, 0.33]), // E, nu for aluminum
        )?;

        // Character attributes - material names and boundary conditions
        println!("   - Character attributes: Names and descriptions");
        file.put_attribute(
            EntityType::ElemBlock,
            1,
            "material_name",
            AttributeType::Char,
            AttributeData::Char("AISI_1045_Steel".to_string()),
        )?;

        file.put_attribute(
            EntityType::ElemBlock,
            3,
            "material_name",
            AttributeType::Char,
            AttributeData::Char("Aluminum_6061".to_string()),
        )?;

        file.put_attribute(
            EntityType::NodeSet,
            1,
            "bc_type",
            AttributeType::Char,
            AttributeData::Char("fixed_displacement".to_string()),
        )?;

        file.put_attribute(
            EntityType::NodeSet,
            2,
            "bc_type",
            AttributeType::Char,
            AttributeData::Char("applied_load".to_string()),
        )?;

        file.put_attribute(
            EntityType::SideSet,
            10,
            "surface_type",
            AttributeType::Char,
            AttributeData::Char("contact_surface".to_string()),
        )?;

        println!("\nFile created successfully!");
    }

    // ========================================================================
    // Part 2: Read back assemblies, blobs, and attributes
    // ========================================================================
    println!("\n2. Reading assemblies, blobs, and attributes from file...\n");
    {
        let file = ExodusFile::<mode::Read>::open(filename)?;

        // ====================================================================
        // Read Assemblies
        // ====================================================================
        println!("   Assemblies:");
        let assembly_ids = file.assembly_ids()?;
        println!(
            "   - Found {} assemblies with IDs: {:?}",
            assembly_ids.len(),
            assembly_ids
        );

        for &id in &assembly_ids {
            let assembly = file.assembly(id)?;
            println!("\n   Assembly {}:", id);
            println!("     Name: {}", assembly.name);
            println!("     Type: {:?}", assembly.entity_type);
            println!("     Entities: {:?}", assembly.entity_list);
        }

        // ====================================================================
        // Read Blobs
        // ====================================================================
        println!("\n   Blobs:");
        let blob_ids = file.blob_ids()?;
        println!(
            "   - Found {} blobs with IDs: {:?}",
            blob_ids.len(),
            blob_ids
        );

        for &id in &blob_ids {
            let (blob, data) = file.blob(id)?;
            println!("\n   Blob {}:", id);
            println!("     Name: {}", blob.name);
            println!("     Size: {} bytes", data.len());

            // Try to display data as text if it's ASCII
            if data.iter().all(|&b| b.is_ascii()) && !data.is_empty() {
                if let Ok(text) = std::str::from_utf8(&data) {
                    let preview = if text.len() > 60 {
                        format!("{}...", &text[..60])
                    } else {
                        text.to_string()
                    };
                    println!("     Content: {}", preview);
                }
            } else {
                // Show hex dump for binary data
                let preview: String = data
                    .iter()
                    .take(8)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("     Hex: {} ...", preview);
            }
        }

        // ====================================================================
        // Read Attributes
        // ====================================================================
        println!("\n   Attributes:");

        // Read attributes from element block 1
        println!("\n   Element Block 1 attributes:");
        let eb1_attrs = file.entity_attributes(EntityType::ElemBlock, 1)?;
        for (name, data) in &eb1_attrs {
            match data {
                AttributeData::Integer(values) => {
                    println!("     {}: {:?}", name, values);
                }
                AttributeData::Double(values) => {
                    if values.len() == 1 {
                        println!("     {}: {}", name, values[0]);
                    } else {
                        println!("     {}: {:?}", name, values);
                    }
                }
                AttributeData::Char(text) => {
                    println!("     {}: \"{}\"", name, text);
                }
            }
        }

        // Read attributes from element block 3
        println!("\n   Element Block 3 attributes:");
        let eb3_attrs = file.entity_attributes(EntityType::ElemBlock, 3)?;
        for (name, data) in &eb3_attrs {
            match data {
                AttributeData::Integer(values) => {
                    println!("     {}: {:?}", name, values);
                }
                AttributeData::Double(values) => {
                    if values.len() == 1 {
                        println!("     {}: {}", name, values[0]);
                    } else {
                        println!("     {}: {:?}", name, values);
                    }
                }
                AttributeData::Char(text) => {
                    println!("     {}: \"{}\"", name, text);
                }
            }
        }

        // Read attributes from node sets
        println!("\n   Node Set 1 attributes:");
        let ns1_attr = file.attribute(EntityType::NodeSet, 1, "bc_type")?;
        if let AttributeData::Char(text) = ns1_attr {
            println!("     bc_type: \"{}\"", text);
        }

        println!("\n   Node Set 2 attributes:");
        let ns2_attr = file.attribute(EntityType::NodeSet, 2, "bc_type")?;
        if let AttributeData::Char(text) = ns2_attr {
            println!("     bc_type: \"{}\"", text);
        }

        // Read attributes from side set
        println!("\n   Side Set 10 attributes:");
        let ss10_attr = file.attribute(EntityType::SideSet, 10, "surface_type")?;
        if let AttributeData::Char(text) = ss10_attr {
            println!("     surface_type: \"{}\"", text);
        }
    }

    // ========================================================================
    // Summary
    // ========================================================================
    println!("\n=== Example completed successfully! ===");
    println!("\nKey Features Demonstrated:");
    println!("  - Assemblies provide hierarchical organization of mesh entities");
    println!("  - Different assembly types (ElemBlock, NodeSet, SideSet)");
    println!("  - Blobs store arbitrary binary data (configs, images, etc.)");
    println!("  - Attributes store typed metadata (Integer, Double, Char)");
    println!("  - Multi-value attributes for storing arrays");
    println!("  - All features use NetCDF-4 variables for flexible storage");
    println!("\nOutput file: {}", filename);

    Ok(())
}
