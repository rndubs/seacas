//! Comprehensive tests for Phase 1: File Lifecycle Operations

use exodus_rs::{mode, CreateMode, CreateOptions, ExodusFile, FloatSize, InitParams, Int64Mode};
use std::path::Path;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_create_with_clobber_mode() {
    let tmp = NamedTempFile::new().unwrap();

    // Create first file
    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "First".into(),
            num_dim: 2,
            num_nodes: 4,
            ..Default::default()
        })
        .unwrap();
    }

    // Clobber it with new file
    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "Second".into(),
            num_dim: 3,
            num_nodes: 8,
            ..Default::default()
        })
        .unwrap();
    }

    // Verify second file
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let params = file.init_params().unwrap();
        assert_eq!(params.title, "Second");
        assert_eq!(params.num_dim, 3);
        assert_eq!(params.num_nodes, 8);
    }
}

#[test]
fn test_create_with_noclobber_mode() {
    let tmp = NamedTempFile::new().unwrap();

    // Create first file
    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::NoClobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "First".into(),
            num_dim: 2,
            num_nodes: 4,
            ..Default::default()
        })
        .unwrap();
    }

    // Try to create again - should fail
    let result = ExodusFile::create(
        tmp.path(),
        CreateOptions {
            mode: CreateMode::NoClobber,
            ..Default::default()
        },
    );

    assert!(result.is_err(), "NoClobber should fail on existing file");
}

#[test]
fn test_float_size_float32() {
    let tmp = NamedTempFile::new().unwrap();

    {
        let file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                float_size: FloatSize::Float32,
                ..Default::default()
            },
        )
        .unwrap();

        // File created successfully
        drop(file);
    }

    // Should be able to read it
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    drop(file);
}

#[test]
fn test_float_size_float64() {
    let tmp = NamedTempFile::new().unwrap();

    {
        let file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                float_size: FloatSize::Float64,
                ..Default::default()
            },
        )
        .unwrap();

        drop(file);
    }

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    drop(file);
}

#[test]
fn test_int64_mode_default() {
    let tmp = NamedTempFile::new().unwrap();

    {
        let file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                int64_mode: Int64Mode::Default,
                ..Default::default()
            },
        )
        .unwrap();

        drop(file);
    }

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    drop(file);
}

#[test]
fn test_int64_mode_all() {
    let tmp = NamedTempFile::new().unwrap();

    {
        let file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                int64_mode: Int64Mode::All64Bit,
                ..Default::default()
            },
        )
        .unwrap();

        drop(file);
    }

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    drop(file);
}

#[test]
fn test_open_nonexistent_file() {
    let result = ExodusFile::<mode::Read>::open("/nonexistent/path/file.exo");
    assert!(result.is_err(), "Opening nonexistent file should fail");
}

#[test]
fn test_file_version() {
    let tmp = NamedTempFile::new().unwrap();

    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "Version Test".into(),
            num_dim: 2,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();
    }

    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let version = file.version().unwrap();
        assert!(version > 0.0, "Version should be positive");
    }
}

#[test]
fn test_drop_closes_file() {
    let tmp = NamedTempFile::new().unwrap();

    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "Drop Test".into(),
            num_dim: 2,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();

        // File dropped here
    }

    // Should be able to open again
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    drop(file);
}

#[test]
fn test_create_in_directory() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.exo");

    {
        let mut file = ExodusFile::create(
            &file_path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "Dir Test".into(),
            num_dim: 2,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();
    }

    assert!(file_path.exists(), "File should exist");

    let file = ExodusFile::<mode::Read>::open(&file_path).unwrap();
    drop(file);
}

#[test]
fn test_multiple_files_simultaneously() {
    let tmp1 = NamedTempFile::new().unwrap();
    let tmp2 = NamedTempFile::new().unwrap();

    let mut file1 = ExodusFile::create(
        tmp1.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
    .unwrap();

    let mut file2 = ExodusFile::create(
        tmp2.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
    .unwrap();

    file1
        .init(&InitParams {
            title: "File1".into(),
            num_dim: 2,
            num_nodes: 4,
            ..Default::default()
        })
        .unwrap();

    file2
        .init(&InitParams {
            title: "File2".into(),
            num_dim: 3,
            num_nodes: 8,
            ..Default::default()
        })
        .unwrap();

    drop(file1);
    drop(file2);

    // Verify both files
    let f1 = ExodusFile::<mode::Read>::open(tmp1.path()).unwrap();
    let p1 = f1.init_params().unwrap();
    assert_eq!(p1.title, "File1");

    let f2 = ExodusFile::<mode::Read>::open(tmp2.path()).unwrap();
    let p2 = f2.init_params().unwrap();
    assert_eq!(p2.title, "File2");
}

#[test]
fn test_read_write_modes_are_separate() {
    let tmp = NamedTempFile::new().unwrap();

    // Write mode
    {
        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "Mode Test".into(),
            num_dim: 2,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();
    }

    // Read mode
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let params = file.init_params().unwrap();
        assert_eq!(params.title, "Mode Test");
    }
}

#[test]
fn test_file_path_with_spaces() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test file with spaces.exo");

    {
        let mut file = ExodusFile::create(
            &file_path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            title: "Spaces Test".into(),
            num_dim: 2,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();
    }

    let file = ExodusFile::<mode::Read>::open(&file_path).unwrap();
    drop(file);
}

#[test]
fn test_create_default_helper() {
    let tmp = NamedTempFile::new().unwrap();

    {
        let file = ExodusFile::create_default(tmp.path()).unwrap();
        drop(file);
    }

    // Should create valid file
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    drop(file);
}
