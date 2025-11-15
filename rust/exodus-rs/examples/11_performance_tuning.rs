//! Example demonstrating performance tuning for HPC environments
//!
//! This example shows how to configure exodus-rs for optimal I/O performance
//! on different types of compute nodes (login vs compute nodes).

use exodus_rs::*;
use std::env;

fn main() -> Result<()> {
    println!("=== Exodus Performance Tuning Example ===\n");

    // Example 1: Automatic node detection
    println!("1. Automatic Node Detection");
    println!("   Detecting node type...");

    let node_type = NodeType::detect();
    println!("   Detected: {:?}", node_type);

    match node_type {
        NodeType::Compute => {
            println!("   ✓ Running on compute node - using aggressive settings");
            println!(
                "     - Cache: {} MB",
                node_type.default_cache_size() / (1024 * 1024)
            );
            println!("     - Chunk: {} nodes", node_type.default_chunk_nodes());
        }
        NodeType::Login => {
            println!("   ✓ Running on login node - using conservative settings");
            println!(
                "     - Cache: {} MB",
                node_type.default_cache_size() / (1024 * 1024)
            );
            println!("     - Chunk: {} nodes", node_type.default_chunk_nodes());
        }
        NodeType::Unknown => {
            println!("   ⚠ Running on unknown node - using moderate defaults");
            println!(
                "     - Cache: {} MB",
                node_type.default_cache_size() / (1024 * 1024)
            );
            println!("     - Chunk: {} nodes", node_type.default_chunk_nodes());
        }
    }
    println!();

    // Example 2: Auto-configured file creation
    println!("2. Auto-Configured File Creation (Recommended)");
    println!("   Creating file with automatic performance tuning...");

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(PerformanceConfig::auto()), // Auto-detect and optimize
        ..Default::default()
    };

    let _file1 = ExodusFile::create("example_auto_perf.exo", options)?;
    println!("   ✓ File created with auto-detected settings");
    println!();

    // Example 3: Conservative settings (for login nodes or shared resources)
    println!("3. Conservative Settings (Login Node)");
    println!("   Creating file with conservative settings...");

    let conservative = PerformanceConfig::conservative();
    println!("   {}", conservative.summary());

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(conservative),
        ..Default::default()
    };

    let _file2 = ExodusFile::create("example_conservative.exo", options)?;
    println!("   ✓ File created with conservative settings");
    println!();

    // Example 4: Aggressive settings (for dedicated compute nodes)
    println!("4. Aggressive Settings (Compute Node)");
    println!("   Creating file with aggressive settings...");

    let aggressive = PerformanceConfig::aggressive();
    println!("   {}", aggressive.summary());

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(aggressive),
        ..Default::default()
    };

    let _file3 = ExodusFile::create("example_aggressive.exo", options)?;
    println!("   ✓ File created with aggressive settings");
    println!();

    // Example 5: Custom tuning for specific workload
    println!("5. Custom Performance Tuning");
    println!("   Creating file with custom settings for large mesh...");

    let custom = PerformanceConfig::auto()
        .with_cache_mb(256) // 256 MB cache (large node)
        .with_node_chunk_size(20_000) // 20k nodes per chunk
        .with_preemption(0.5); // Balanced write/read

    println!("   {}", custom.summary());

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(custom),
        ..Default::default()
    };

    let mut file4 = ExodusFile::create("example_custom.exo", options)?;
    println!("   ✓ File created with custom settings");
    println!();

    // Example 6: Initialize and write data (performance config applies)
    println!("6. Writing Data with Performance Tuning");
    println!("   Initializing mesh...");

    let params = InitParams {
        title: "Performance Tuning Example".to_string(),
        num_dim: 3,
        num_nodes: 100_000, // Large mesh
        num_elems: 90_000,
        num_elem_blocks: 1,
        ..Default::default()
    };

    file4.init(&params)?;
    println!("   ✓ Initialized mesh with {} nodes", params.num_nodes);

    // Generate coordinate data
    let x: Vec<f64> = (0..100_000).map(|i| (i as f64) * 0.1).collect();
    let y: Vec<f64> = (0..100_000).map(|i| (i as f64) * 0.1).collect();
    let z: Vec<f64> = (0..100_000).map(|i| (i as f64) * 0.1).collect();

    println!("   Writing coordinates...");
    file4.put_coords(&x, Some(&y), Some(&z))?;
    println!("   ✓ Coordinates written (cache optimized)");
    println!();

    // Example 7: Environment variable override
    println!("7. Environment Variable Override");
    println!("   You can also set HDF5 environment variables manually:");
    println!("   ```bash");
    println!("   export HDF5_CHUNK_CACHE_NBYTES=268435456  # 256 MB");
    println!("   export HDF5_CHUNK_CACHE_NSLOTS=10007      # Prime number");
    println!("   export HDF5_CHUNK_CACHE_W0=0.75           # Preemption policy");
    println!("   cargo run --example 11_performance_tuning");
    println!("   ```");
    println!();

    // Check if any env vars are set
    if env::var("HDF5_CHUNK_CACHE_NBYTES").is_ok() {
        println!(
            "   ✓ HDF5_CHUNK_CACHE_NBYTES is set: {}",
            env::var("HDF5_CHUNK_CACHE_NBYTES").unwrap()
        );
    }
    if env::var("HDF5_CHUNK_CACHE_NSLOTS").is_ok() {
        println!(
            "   ✓ HDF5_CHUNK_CACHE_NSLOTS is set: {}",
            env::var("HDF5_CHUNK_CACHE_NSLOTS").unwrap()
        );
    }
    if env::var("HDF5_CHUNK_CACHE_W0").is_ok() {
        println!(
            "   ✓ HDF5_CHUNK_CACHE_W0 is set: {}",
            env::var("HDF5_CHUNK_CACHE_W0").unwrap()
        );
    }
    println!();

    // Example 8: Recommendations
    println!("8. Performance Recommendations");
    println!("   ┌─────────────────────────────────────────────────────────┐");
    println!("   │ Node Type      │ Cache Size │ Chunk Size │ Use Case    │");
    println!("   ├─────────────────────────────────────────────────────────┤");
    println!("   │ Login Node     │    4 MB    │  1,000     │ Development │");
    println!("   │ Small Compute  │   16 MB    │  5,000     │ Testing     │");
    println!("   │ Large Compute  │  128 MB    │ 10,000     │ Production  │");
    println!("   │ HPC (256GB+)   │  256+ MB   │ 20,000+    │ Large Scale │");
    println!("   └─────────────────────────────────────────────────────────┘");
    println!();

    println!("   For your detected node type ({:?}):", node_type);
    match node_type {
        NodeType::Compute => {
            println!("   → Use PerformanceConfig::aggressive() or auto()");
            println!("   → You have dedicated resources - maximize cache!");
        }
        NodeType::Login => {
            println!("   → Use PerformanceConfig::conservative()");
            println!("   → Be respectful of shared resources");
        }
        NodeType::Unknown => {
            println!("   → Use PerformanceConfig::auto() (safe default)");
            println!("   → Or customize based on your system specs");
        }
    }
    println!();

    // Example 9: Job scheduler detection
    println!("9. Job Scheduler Detection");
    println!("   Checking for job scheduler environments...");

    let schedulers = [
        ("SLURM_JOB_ID", "Slurm"),
        ("FLUX_URI", "Flux"),
        ("PBS_JOBID", "PBS"),
        ("LSB_JOBID", "LSF"),
    ];

    let mut found_scheduler = false;
    for (env_var, name) in &schedulers {
        if let Ok(value) = env::var(env_var) {
            println!("   ✓ Detected {} scheduler: {} = {}", name, env_var, value);
            found_scheduler = true;
        }
    }

    if !found_scheduler {
        println!("   ⚠ No job scheduler detected (running locally)");
    }
    println!();

    println!("=== Example Complete ===");
    println!();
    println!("Files created:");
    println!("  - example_auto_perf.exo (auto-detected settings)");
    println!("  - example_conservative.exo (login node settings)");
    println!("  - example_aggressive.exo (compute node settings)");
    println!("  - example_custom.exo (custom tuned settings with data)");

    Ok(())
}
