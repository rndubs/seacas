//! Example: Initializing Exodus files with metadata
//!
//! This example demonstrates Phase 2 functionality:
//! - Database initialization with parameters
//! - Builder pattern for fluent initialization
//! - QA records for software provenance
//! - Info records for arbitrary text
//! - Coordinate axis naming

use exodus_rs::{ExodusFile, InitParams, QaRecord};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Exodus Initialization Example ===\n");

    // Example 1: Direct initialization
    println!("1. Direct initialization with InitParams...");
    {
        let mut file = ExodusFile::create_default("example_init.exo")?;

        let params = InitParams {
            title: "Direct Initialization Example".into(),
            num_dim: 3,
            num_nodes: 8,
            num_elems: 1,
            num_elem_blocks: 1,
            ..Default::default()
        };

        file.init(&params)?;
        println!("   Initialized: {}", params.title);
        println!("   Dimensions: {}", params.num_dim);
        println!("   Nodes: {}", params.num_nodes);
    }

    // Example 2: Builder pattern (fluent API)
    println!("\n2. Builder pattern initialization...");
    {
        let mut file = ExodusFile::create_default("example_builder.exo")?;

        file.builder()
            .title("Builder Pattern Example")
            .dimensions(2)
            .nodes(4)
            .elems(1)
            .elem_blocks(1)
            .node_sets(1)
            .side_sets(1)
            .finish()?;

        println!("   Initialized with fluent API");
    }

    // Example 3: QA Records
    println!("\n3. Writing QA records...");
    {
        let mut file = ExodusFile::create_default("example_qa.exo")?;
        file.init(&InitParams::default())?;

        let qa_records = vec![
            QaRecord {
                code_name: "exodus-rs".into(),
                code_version: "0.1.0".into(),
                date: "2025-01-09".into(),
                time: "12:00:00".into(),
            },
            QaRecord {
                code_name: "my-app".into(),
                code_version: "1.0.0".into(),
                date: "2025-01-09".into(),
                time: "12:05:30".into(),
            },
        ];

        file.put_qa_records(&qa_records)?;
        println!("   Wrote {} QA records", qa_records.len());

        drop(file);

        // Read back QA records
        use exodus_rs::mode::Read;
        let file = ExodusFile::<Read>::open("example_qa.exo")?;
        let read_qa = file.qa_records()?;

        println!("   Read back QA records:");
        for (i, qa) in read_qa.iter().enumerate() {
            println!(
                "     [{}] {} v{} ({} {})",
                i + 1,
                qa.code_name,
                qa.code_version,
                qa.date,
                qa.time
            );
        }
    }

    // Example 4: Info Records
    println!("\n4. Writing info records...");
    {
        let mut file = ExodusFile::create_default("example_info.exo")?;
        file.init(&InitParams::default())?;

        let info = vec![
            "This is a demonstration mesh".to_string(),
            "Created for testing exodus-rs".to_string(),
            "Phase 2 functionality".to_string(),
            "Multiple lines of arbitrary text are supported".to_string(),
        ];

        file.put_info_records(&info)?;
        println!("   Wrote {} info records", info.len());

        drop(file);

        // Read back info records
        use exodus_rs::mode::Read;
        let file = ExodusFile::<Read>::open("example_info.exo")?;
        let read_info = file.info_records()?;

        println!("   Read back info records:");
        for (i, line) in read_info.iter().enumerate() {
            println!("     [{}] {}", i + 1, line);
        }
    }

    // Example 5: Coordinate Names
    println!("\n5. Setting coordinate axis names...");
    {
        let mut file = ExodusFile::create_default("example_coords.exo")?;
        file.init(&InitParams {
            num_dim: 3,
            ..Default::default()
        })?;

        // Set custom coordinate names
        // TODO: Coordinate names not yet implemented
        // file.put_coord_names(&["X_POSITION", "Y_POSITION", "Z_POSITION"])?;
        println!("   (Coordinate names not yet implemented)");

        drop(file);

        // Read back coordinate names
        // TODO: Coordinate names not yet implemented
        // use exodus_rs::mode::Read;
        // let file = ExodusFile::<Read>::open("example_coords.exo")?;
        // let names = file.coord_names()?;
        // println!("   Read back coordinate names: {:?}", names);
    }

    // Example 6: Default coordinate names
    println!("\n6. Default coordinate names...");
    {
        let mut file = ExodusFile::create_default("example_default_coords.exo")?;
        file.init(&InitParams {
            num_dim: 2,
            ..Default::default()
        })?;

        drop(file);

        // Read back - should get defaults
        // TODO: Coordinate names not yet implemented
        // use exodus_rs::mode::Read;
        // let file = ExodusFile::<Read>::open("example_default_coords.exo")?;
        // let names = file.coord_names()?;
        // println!("   Default names for 2D: {:?}", names);
    }

    // Example 7: Complete workflow
    println!("\n7. Complete initialization workflow...");
    {
        let mut file = ExodusFile::create_default("example_complete.exo")?;

        // Initialize with builder
        file.builder()
            .title("Complete Workflow Example")
            .dimensions(3)
            .nodes(100)
            .elems(20)
            .elem_blocks(2)
            .node_sets(3)
            .side_sets(2)
            .finish()?;

        // Add QA record
        let qa = vec![QaRecord {
            code_name: "example-app".into(),
            code_version: "2.0.1".into(),
            date: "2025-01-09".into(),
            time: "14:30:00".into(),
        }];
        file.put_qa_records(&qa)?;

        // Add info records
        let info = vec![
            "Complete demonstration of Phase 2 features".to_string(),
            "Includes initialization, QA, info, and coordinate names".to_string(),
        ];
        file.put_info_records(&info)?;

        // Set coordinate names
        // TODO: Coordinate names not yet implemented
        // file.put_coord_names(&["X", "Y", "Z"])?;

        println!("   Created complete file with all metadata");

        drop(file);

        // Verify by reading back
        use exodus_rs::mode::Read;
        let file = ExodusFile::<Read>::open("example_complete.exo")?;

        let params = file.init_params()?;
        println!("\n   Verification:");
        println!("     Title: {}", params.title);
        println!("     Dimensions: {}", params.num_dim);
        println!("     Nodes: {}", params.num_nodes);
        println!("     Elements: {}", params.num_elems);
        println!("     Element blocks: {}", params.num_elem_blocks);

        let qa = file.qa_records()?;
        println!("     QA records: {}", qa.len());

        let info = file.info_records()?;
        println!("     Info records: {}", info.len());

        // TODO: Coordinate names not yet implemented
        // let names = file.coord_names()?;
        // println!("     Coordinate names: {:?}", names);
    }

    println!("\n=== Example completed successfully ===");

    // Cleanup: Remove created files
    let files = [
        "example_init.exo",
        "example_builder.exo",
        "example_qa.exo",
        "example_info.exo",
        "example_coords.exo",
        "example_default_coords.exo",
        "example_complete.exo",
    ];

    for file in &files {
        if std::path::Path::new(file).exists() {
            std::fs::remove_file(file)?;
            println!("Cleaned up: {}", file);
        }
    }

    Ok(())
}
