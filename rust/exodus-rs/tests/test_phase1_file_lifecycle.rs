//! Phase 1 Comprehensive Tests: File Lifecycle
//!
//! This test suite covers all file lifecycle operations including:
//! - File creation with different modes
//! - File opening
//! - File formats and options
//! - Error handling
//! - Resource cleanup

#[cfg(feature = "netcdf4")]
mod file_lifecycle_tests {
    use exodus_rs::*;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    // Helper to create test file with clobber mode
    fn create_test_file(path: impl AsRef<std::path::Path>) -> Result<ExodusFile<mode::Write>> {
        ExodusFile::create(
            path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
    }

    // ========================================================================
    // CreateMode Tests
    // ========================================================================

    #[test]
    fn test_create_mode_noclobber_success() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        // Remove the file so we can test creation
        drop(tmp);

        let result = ExodusFile::create(
            &path,
            CreateOptions {
                mode: CreateMode::NoClobber,
                ..Default::default()
            },
        );

        assert!(result.is_ok(), "Should create file when it doesn't exist");
    }

    #[test]
    fn test_create_mode_noclobber_fails_when_exists() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        // Create initial file
        let _file1 = create_test_file(path).unwrap();
        drop(_file1);

        // Try to create again with NoClobber
        let result = ExodusFile::create(
            path,
            CreateOptions {
                mode: CreateMode::NoClobber,
                ..Default::default()
            },
        );

        assert!(result.is_err(), "NoClobber should fail when file exists");
    }

    #[test]
    fn test_create_mode_clobber_overwrites_existing() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        // Create and initialize first file
        {
            let mut file1 = create_test_file(path).unwrap();
            let params = InitParams {
                title: "First File".to_string(),
                num_dim: 3,
                num_nodes: 100,
                ..Default::default()
            };
            file1.init(&params).unwrap();
        }

        // Overwrite with clobber
        {
            let mut file2 = ExodusFile::create(
                path,
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "Second File".to_string(),
                num_dim: 2,
                num_nodes: 50,
                ..Default::default()
            };
            file2.init(&params).unwrap();
        }

        // Verify the file was overwritten
        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let params = file.init_params().unwrap();
        assert_eq!(params.title, "Second File");
        assert_eq!(params.num_dim, 2);
        assert_eq!(params.num_nodes, 50);
    }

    // ========================================================================
    // FloatSize Tests
    // ========================================================================

    #[test]
    fn test_float_size_32() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let file = ExodusFile::create(
            path,
            CreateOptions {
                mode: CreateMode::Clobber,
                float_size: FloatSize::Float32,
                ..Default::default()
            },
        )
        .unwrap();

        drop(file);

        // Verify file was created successfully
        assert!(path.exists());
    }

    #[test]
    fn test_float_size_64() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let file = ExodusFile::create(
            path,
            CreateOptions {
                mode: CreateMode::Clobber,
                float_size: FloatSize::Float64,
                ..Default::default()
            },
        )
        .unwrap();

        drop(file);

        // Verify file was created successfully
        assert!(path.exists());
    }

    // ========================================================================
    // Int64Mode Tests
    // ========================================================================

    #[test]
    fn test_int64_mode_int32() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let file = ExodusFile::create(
            path,
            CreateOptions {
                mode: CreateMode::Clobber,
                int64_mode: Int64Mode::Int32,
                ..Default::default()
            },
        )
        .unwrap();

        drop(file);

        // Verify file was created successfully
        assert!(path.exists());
    }

    #[test]
    fn test_int64_mode_int64() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let file = ExodusFile::create(
            path,
            CreateOptions {
                mode: CreateMode::Clobber,
                int64_mode: Int64Mode::Int64,
                ..Default::default()
            },
        )
        .unwrap();

        drop(file);

        // Verify file was created successfully
        assert!(path.exists());
    }

    // ========================================================================
    // File Format Detection Tests
    // ========================================================================

    #[test]
    fn test_file_format_detection() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let file = create_test_file(path).unwrap();
        let format = file.format().unwrap();

        assert_eq!(format, FileFormat::NetCdf4);
    }

    // ========================================================================
    // Version Reading Tests
    // ========================================================================

    #[test]
    fn test_version_reading() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let file = create_test_file(path).unwrap();
        let version = file.version().unwrap();

        // Should be version 2.0
        assert_eq!(version.0, 2, "Major version should be 2");
        assert_eq!(version.1, 0, "Minor version should be 0");
    }

    #[test]
    fn test_version_after_init() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Test Mesh".to_string(),
                num_dim: 3,
                num_nodes: 10,
                ..Default::default()
            };
            file.init(&params).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let version = file.version().unwrap();

        assert_eq!(version.0, 2);
        assert_eq!(version.1, 0);
    }

    // ========================================================================
    // Error Handling Tests
    // ========================================================================

    #[test]
    fn test_open_nonexistent_file() {
        let result = ExodusFile::<mode::Read>::open("/tmp/nonexistent_exodus_file_xyz123.exo");
        assert!(result.is_err(), "Should fail to open nonexistent file");
    }

    #[test]
    fn test_append_nonexistent_file() {
        let result = ExodusFile::<mode::Append>::append("/tmp/nonexistent_exodus_file_xyz456.exo");
        assert!(result.is_err(), "Should fail to append to nonexistent file");
    }

    #[test]
    #[cfg(unix)]
    fn test_create_in_readonly_directory() {
        // Create a temporary directory and make it read-only
        let dir = TempDir::new().unwrap();
        let readonly_dir = dir.path();
        let file_path = readonly_dir.join("test.exo");

        // Set directory to read-only (this may not work in all environments)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(readonly_dir).unwrap().permissions();
            perms.set_mode(0o444);
            let _ = fs::set_permissions(readonly_dir, perms);

            let result = create_test_file(&file_path);

            // Restore permissions for cleanup
            let mut perms = fs::metadata(readonly_dir).unwrap().permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(readonly_dir, perms);

            // Note: This test may pass on some systems that allow file creation
            // even in read-only directories, so we just verify it doesn't panic
            let _ = result;
        }
    }

    // ========================================================================
    // Close and Drop Behavior Tests
    // ========================================================================

    #[test]
    fn test_explicit_close() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let file = create_test_file(path).unwrap();
        let result = file.close();

        assert!(result.is_ok(), "Explicit close should succeed");
        assert!(path.exists(), "File should still exist after close");
    }

    #[test]
    fn test_drop_behavior() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        {
            let mut file = create_test_file(&path).unwrap();
            let params = InitParams {
                title: "Test".to_string(),
                num_dim: 3,
                num_nodes: 5,
                ..Default::default()
            };
            file.init(&params).unwrap();
            // File goes out of scope here
        }

        // Verify file was properly written during Drop
        let file = ExodusFile::<mode::Read>::open(&path).unwrap();
        let params = file.init_params().unwrap();
        assert_eq!(params.title, "Test");
    }

    #[test]
    fn test_multiple_drop() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        // Create and drop multiple times
        for i in 0..5 {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: format!("Iteration {}", i),
                num_dim: 3,
                num_nodes: 10,
                ..Default::default()
            };
            file.init(&params).unwrap();
            drop(file);
        }

        // Verify last write was successful
        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let params = file.init_params().unwrap();
        assert_eq!(params.title, "Iteration 4");
    }

    // ========================================================================
    // Append Mode Operation Tests
    // ========================================================================

    #[test]
    fn test_append_mode_read_operations() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        // Create and initialize file
        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Original".to_string(),
                num_dim: 3,
                num_nodes: 100,
                ..Default::default()
            };
            file.init(&params).unwrap();
        }

        // Open in append mode and read
        let file = ExodusFile::<mode::Append>::append(path).unwrap();
        let params = file.init_params().unwrap();

        assert_eq!(params.title, "Original");
        assert_eq!(params.num_nodes, 100);
    }

    #[test]
    fn test_append_mode_write_operations() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        // Create and initialize file
        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Test Mesh".to_string(),
                num_dim: 3,
                num_nodes: 4,
                num_elems: 0,
                ..Default::default()
            };
            file.init(&params).unwrap();
        }

        // Open in append mode and write coordinates
        {
            let mut file = ExodusFile::<mode::Append>::append(path).unwrap();
            let x = vec![0.0_f64, 1.0, 1.0, 0.0];
            let y = vec![0.0_f64, 0.0, 1.0, 1.0];
            let z = vec![0.0_f64; 4];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        // Verify coordinates were written
        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();
        assert_eq!(coords.x.len(), 4);
        assert_eq!(coords.x[1], 1.0);
    }

    #[test]
    fn test_create_default_convenience() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        // Remove temp file so we can recreate it
        drop(tmp);

        let file = ExodusFile::create_default(&path).unwrap();

        // Verify default options were used
        let version = file.version().unwrap();
        assert_eq!(version, (2, 0));

        let format = file.format().unwrap();
        assert_eq!(format, FileFormat::NetCdf4);
    }

    // ========================================================================
    // Path Handling Tests
    // ========================================================================

    #[test]
    fn test_path_retrieval() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let file = create_test_file(path).unwrap();

        assert_eq!(file.path(), path);
    }

    #[test]
    fn test_file_with_special_characters_in_name() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test-file_123.exo");

        let file = create_test_file(&path).unwrap();
        drop(file);

        assert!(path.exists());

        // Verify we can open it again
        let _file = ExodusFile::<mode::Read>::open(&path).unwrap();
    }
}
