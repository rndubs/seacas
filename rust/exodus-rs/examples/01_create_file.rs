//! Example: Creating and opening Exodus files
//!
//! This example demonstrates Phase 1 functionality:
//! - Creating new Exodus files
//! - Opening existing files for reading
//! - Using different creation modes
//! - Querying file properties

use exodus_rs::{CreateMode, CreateOptions, ExodusFile, FloatSize, Int64Mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Exodus File Creation Example ===\n");

    // Example 1: Create a file with default options
    println!("1. Creating file with default options...");
    {
        let file = ExodusFile::create_default("example_default.exo")?;
        println!("   Created: {:?}", file.path());
        println!("   Format: {:?}", file.format()?);
        println!("   Version: {:?}", file.version()?);
    } // File is automatically closed when it goes out of scope

    // Example 2: Create a file with custom options
    println!("\n2. Creating file with custom options...");
    {
        let options = CreateOptions {
            mode: CreateMode::Clobber, // Overwrite if exists
            float_size: FloatSize::Float32,
            int64_mode: Int64Mode::Int64,
            compression: None,
            parallel: false,
            performance: None,
        };

        let file = ExodusFile::create("example_custom.exo", options)?;
        println!("   Created: {:?}", file.path());
    }

    // Example 3: Try to create with NoClobber (should succeed first time)
    println!("\n3. Creating file with NoClobber mode...");
    {
        let options = CreateOptions {
            mode: CreateMode::NoClobber,
            ..Default::default()
        };

        let file = ExodusFile::create("example_noclobber.exo", options)?;
        println!("   Created: {:?}", file.path());
    }

    // Example 4: Try to create again with NoClobber (should fail)
    println!("\n4. Trying to create existing file with NoClobber...");
    {
        let options = CreateOptions {
            mode: CreateMode::NoClobber,
            ..Default::default()
        };

        match ExodusFile::create("example_noclobber.exo", options) {
            Ok(_) => println!("   Unexpectedly succeeded!"),
            Err(e) => println!("   Expected error: {}", e),
        }
    }

    // Example 5: Open an existing file for reading
    println!("\n5. Opening file for reading...");
    {
        use exodus_rs::mode::Read;
        let file = ExodusFile::<Read>::open("example_default.exo")?;
        println!("   Opened: {:?}", file.path());
        println!("   Format: {:?}", file.format()?);
        println!("   Version: {:?}", file.version()?);
    }

    // Example 6: Open file in append mode
    println!("\n6. Opening file in append mode...");
    {
        use exodus_rs::mode::Append;
        let file = ExodusFile::<Append>::append("example_default.exo")?;
        println!("   Opened for append: {:?}", file.path());
    }

    // Example 7: Explicit close
    println!("\n7. Explicitly closing a file...");
    {
        let file = ExodusFile::create_default("example_close.exo")?;
        println!("   Created: {:?}", file.path());
        file.close()?;
        println!("   File closed explicitly");
    }

    println!("\n=== Example completed successfully ===");

    // Cleanup: Remove created files
    let files = [
        "example_default.exo",
        "example_custom.exo",
        "example_noclobber.exo",
        "example_close.exo",
    ];

    for file in &files {
        if std::path::Path::new(file).exists() {
            std::fs::remove_file(file)?;
            println!("Cleaned up: {}", file);
        }
    }

    Ok(())
}
