//! Example 08: Assemblies and Blobs
//!
//! This example demonstrates Phase 8 functionality:
//! - Assemblies for hierarchical grouping
//! - Blobs for binary data storage

use exodus_rs::{
    mode, Assembly, Blob, CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, Result,
};

fn main() -> Result<()> {
    println!("=== Phase 8: Assemblies and Blobs Example ===\n");

    let filename = "example_08_assemblies_blobs.exo";

    // ========================================================================
    // Part 1: Create file with assemblies and blobs
    // ========================================================================
    println!("1. Creating Exodus file with assemblies and blobs...");
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
            title: "Assemblies and Blobs Example".into(),
            num_dim: 3,
            num_nodes: 12,
            num_elems: 5,
            num_elem_blocks: 3,
            num_node_sets: 2,
            ..Default::default()
        };
        file.init(&params)?;

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
        let material_data = br#"{"steel": {"E": 200e9, "nu": 0.3}, "aluminum": {"E": 70e9, "nu": 0.33}}"#;
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

        println!("\nFile created successfully!");
    }

    // ========================================================================
    // Part 2: Read back assemblies and blobs
    // ========================================================================
    println!("\n2. Reading assemblies and blobs from file...\n");
    {
        let file = ExodusFile::<mode::Read>::open(filename)?;

        // ====================================================================
        // Read Assemblies
        // ====================================================================
        println!("   Assemblies:");
        let assembly_ids = file.assembly_ids()?;
        println!("   - Found {} assemblies with IDs: {:?}", assembly_ids.len(), assembly_ids);

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
        println!("   - Found {} blobs with IDs: {:?}", blob_ids.len(), blob_ids);

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
                let preview: String = data.iter()
                    .take(8)
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("     Hex: {} ...", preview);
            }
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
    println!("  - Both features use NetCDF-4 attributes for metadata");
    println!("\nOutput file: {}", filename);

    Ok(())
}
