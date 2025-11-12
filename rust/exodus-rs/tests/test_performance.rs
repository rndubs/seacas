//! Integration tests for performance configuration
//!
//! Tests that performance settings are correctly applied to file creation
//! and that node detection works as expected.

use exodus_rs::*;
use tempfile::NamedTempFile;

#[test]
fn test_file_creation_with_auto_performance() {
    let tmp = NamedTempFile::new().unwrap();

    // Create file with auto-detected performance config
    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(PerformanceConfig::auto()),
        ..Default::default()
    };

    let file = ExodusFile::create(tmp.path(), options).unwrap();
    assert_eq!(file.path(), tmp.path());

    // Verify file was created
    assert!(tmp.path().exists());
}

#[test]
fn test_file_creation_with_conservative_performance() {
    let tmp = NamedTempFile::new().unwrap();

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(PerformanceConfig::conservative()),
        ..Default::default()
    };

    let file = ExodusFile::create(tmp.path(), options).unwrap();
    assert_eq!(file.path(), tmp.path());
}

#[test]
fn test_file_creation_with_aggressive_performance() {
    let tmp = NamedTempFile::new().unwrap();

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(PerformanceConfig::aggressive()),
        ..Default::default()
    };

    let file = ExodusFile::create(tmp.path(), options).unwrap();
    assert_eq!(file.path(), tmp.path());
}

#[test]
fn test_file_creation_with_custom_cache() {
    let tmp = NamedTempFile::new().unwrap();

    let perf = PerformanceConfig::auto()
        .with_cache_mb(64)
        .with_node_chunk_size(10_000);

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(perf),
        ..Default::default()
    };

    let file = ExodusFile::create(tmp.path(), options).unwrap();
    assert_eq!(file.path(), tmp.path());
}

#[test]
fn test_file_creation_with_custom_preemption() {
    let tmp = NamedTempFile::new().unwrap();

    let perf = PerformanceConfig::auto()
        .with_preemption(0.3);

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(perf),
        ..Default::default()
    };

    let file = ExodusFile::create(tmp.path(), options).unwrap();
    assert_eq!(file.path(), tmp.path());
}

#[test]
fn test_file_creation_with_none_performance() {
    let tmp = NamedTempFile::new().unwrap();

    // None should auto-detect
    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: None,
        ..Default::default()
    };

    let file = ExodusFile::create(tmp.path(), options).unwrap();
    assert_eq!(file.path(), tmp.path());
}

#[test]
fn test_default_create_uses_auto_performance() {
    let tmp = NamedTempFile::new().unwrap();

    // create_default should auto-detect performance
    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let file = ExodusFile::create(tmp.path(), options).unwrap();
    assert_eq!(file.path(), tmp.path());
}

#[test]
fn test_performance_config_with_mesh_operations() {
    let tmp = NamedTempFile::new().unwrap();

    let perf = PerformanceConfig::aggressive()
        .with_cache_mb(128)
        .with_node_chunk_size(10_000);

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(perf),
        ..Default::default()
    };

    let mut file = ExodusFile::create(tmp.path(), options).unwrap();

    // Initialize with mesh data
    let params = InitParams {
        title: "Performance Test".to_string(),
        num_dim: 3,
        num_nodes: 1000,
        num_elems: 800,
        num_elem_blocks: 1,
        ..Default::default()
    };

    file.init(&params).unwrap();

    // Write coordinates
    let x: Vec<f64> = (0..1000).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..1000).map(|i| i as f64).collect();
    let z: Vec<f64> = (0..1000).map(|i| i as f64).collect();

    file.put_coords(&x, Some(&y), Some(&z)).unwrap();

    // Verify data can be read back
    drop(file);

    let file = ExodusFile::open(tmp.path()).unwrap();
    let params = file.init_params().unwrap();
    assert_eq!(params.num_nodes, 1000);
}

#[test]
fn test_node_type_for_specific_types() {
    let compute_perf = PerformanceConfig::for_node_type(NodeType::Compute);
    assert_eq!(compute_perf.node_type, NodeType::Compute);
    assert_eq!(compute_perf.cache.cache_size, 128 * 1024 * 1024);

    let login_perf = PerformanceConfig::for_node_type(NodeType::Login);
    assert_eq!(login_perf.node_type, NodeType::Login);
    assert_eq!(login_perf.cache.cache_size, 4 * 1024 * 1024);

    let unknown_perf = PerformanceConfig::for_node_type(NodeType::Unknown);
    assert_eq!(unknown_perf.node_type, NodeType::Unknown);
    assert_eq!(unknown_perf.cache.cache_size, 16 * 1024 * 1024);
}

#[test]
fn test_cache_config_fluent_api() {
    let cache = CacheConfig::new(100 * 1024 * 1024)
        .with_slots(10007)
        .with_preemption(0.25);

    assert_eq!(cache.cache_size, 100 * 1024 * 1024);
    assert_eq!(cache.num_slots, 10007);
    assert_eq!(cache.preemption, 0.25);
}

#[test]
fn test_chunk_config_fluent_api() {
    let chunks = ChunkConfig::new()
        .with_node_chunk_size(25_000)
        .with_element_chunk_size(20_000)
        .with_time_chunk_size(5);

    assert_eq!(chunks.node_chunk_size, 25_000);
    assert_eq!(chunks.element_chunk_size, 20_000);
    assert_eq!(chunks.time_chunk_size, 5);
}

#[test]
fn test_performance_config_summary() {
    let perf = PerformanceConfig::auto()
        .with_cache_mb(256);

    let summary = perf.summary();
    assert!(summary.contains("256 MB"));
    assert!(summary.contains("Performance Config"));
}

#[test]
fn test_hdf5_env_vars_set() {
    use std::env;

    // Clear any existing values
    env::remove_var("HDF5_CHUNK_CACHE_NBYTES");
    env::remove_var("HDF5_CHUNK_CACHE_NSLOTS");
    env::remove_var("HDF5_CHUNK_CACHE_W0");

    let tmp = NamedTempFile::new().unwrap();

    let perf = PerformanceConfig::auto()
        .with_cache_mb(128);

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(perf),
        ..Default::default()
    };

    let _file = ExodusFile::create(tmp.path(), options).unwrap();

    // Verify environment variables were set
    assert!(env::var("HDF5_CHUNK_CACHE_NBYTES").is_ok());
    let cache_bytes = env::var("HDF5_CHUNK_CACHE_NBYTES").unwrap();
    assert_eq!(cache_bytes, (128 * 1024 * 1024).to_string());

    assert!(env::var("HDF5_CHUNK_CACHE_W0").is_ok());
    assert!(env::var("HDF5_CHUNK_CACHE_NSLOTS").is_ok());
}

#[test]
fn test_performance_config_respects_user_env_vars() {
    use std::env;

    // User sets their own value
    env::set_var("HDF5_CHUNK_CACHE_NBYTES", "999999");

    let tmp = NamedTempFile::new().unwrap();

    let perf = PerformanceConfig::auto()
        .with_cache_mb(128);

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(perf),
        ..Default::default()
    };

    let _file = ExodusFile::create(tmp.path(), options).unwrap();

    // Should NOT override user's value
    let cache_bytes = env::var("HDF5_CHUNK_CACHE_NBYTES").unwrap();
    assert_eq!(cache_bytes, "999999");

    // Clean up
    env::remove_var("HDF5_CHUNK_CACHE_NBYTES");
}

#[test]
fn test_performance_with_large_mesh() {
    let tmp = NamedTempFile::new().unwrap();

    let perf = PerformanceConfig::aggressive()
        .with_cache_mb(256)
        .with_node_chunk_size(50_000);

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        performance: Some(perf),
        ..Default::default()
    };

    let mut file = ExodusFile::create(tmp.path(), options).unwrap();

    // Large mesh
    let params = InitParams {
        title: "Large Mesh Performance Test".to_string(),
        num_dim: 3,
        num_nodes: 100_000,
        num_elems: 90_000,
        num_elem_blocks: 2,
        ..Default::default()
    };

    file.init(&params).unwrap();

    // Write large coordinate arrays
    let x: Vec<f64> = (0..100_000).map(|i| (i as f64) * 0.1).collect();
    let y: Vec<f64> = (0..100_000).map(|i| (i as f64) * 0.2).collect();
    let z: Vec<f64> = (0..100_000).map(|i| (i as f64) * 0.3).collect();

    file.put_coords(&x, Some(&y), Some(&z)).unwrap();

    // Verify
    drop(file);
    let file = ExodusFile::open(tmp.path()).unwrap();
    let params = file.init_params().unwrap();
    assert_eq!(params.num_nodes, 100_000);
}
