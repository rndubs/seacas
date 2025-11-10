//! Phase 3 Comprehensive Tests: Coordinates
//!
//! This test suite covers all coordinate operations including:
//! - 1D, 2D, 3D coordinates
//! - f32 and f64 coordinate types
//! - Type conversion
//! - Partial coordinate I/O
//! - Array length validation
//! - Empty coordinates
//! - Large coordinate arrays

#[cfg(feature = "netcdf4")]
mod coordinate_tests {
    use exodus_rs::*;
    use tempfile::NamedTempFile;

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
    // 1D, 2D, 3D Coordinate Tests
    // ========================================================================

    #[test]
    fn test_1d_coordinates_f64() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "1D Mesh".to_string(),
                num_dim: 1,
                num_nodes: 5,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![0.0_f64, 1.0, 2.0, 3.0, 4.0];
            file.put_coords(&x, None, None).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        assert_eq!(coords.x.len(), 5);
        assert_eq!(coords.x[0], 0.0);
        assert_eq!(coords.x[4], 4.0);
        assert!(coords.y.is_empty());
        assert!(coords.z.is_empty());
    }

    #[test]
    fn test_2d_coordinates_f64() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "2D Mesh".to_string(),
                num_dim: 2,
                num_nodes: 4,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![0.0_f64, 1.0, 1.0, 0.0];
            let y = vec![0.0_f64, 0.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), None).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        assert_eq!(coords.x.len(), 4);
        assert_eq!(coords.y.len(), 4);
        assert!(coords.z.is_empty());
        assert_eq!(coords.x[2], 1.0);
        assert_eq!(coords.y[2], 1.0);
    }

    #[test]
    fn test_3d_coordinates_f64() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "3D Mesh".to_string(),
                num_dim: 3,
                num_nodes: 8,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Unit cube
            let x = vec![0.0_f64, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0_f64, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0_f64, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        assert_eq!(coords.x.len(), 8);
        assert_eq!(coords.y.len(), 8);
        assert_eq!(coords.z.len(), 8);
        assert_eq!(coords.z[4], 1.0);
        assert_eq!(coords.z[7], 1.0);
    }

    // ========================================================================
    // f32 and f64 Coordinate Type Tests
    // ========================================================================

    #[test]
    fn test_coordinates_f32() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = ExodusFile::create(
                path,
                CreateOptions {
                    mode: CreateMode::Clobber,
                    float_size: FloatSize::Float32,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "f32 Coordinates".to_string(),
                num_dim: 3,
                num_nodes: 4,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![0.0_f32, 1.0, 2.0, 3.0];
            let y = vec![0.0_f32, 1.0, 2.0, 3.0];
            let z = vec![0.0_f32, 1.0, 2.0, 3.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f32>().unwrap();

        assert_eq!(coords.x.len(), 4);
        assert_eq!(coords.x[2], 2.0_f32);
        assert_eq!(coords.y[3], 3.0_f32);
    }

    #[test]
    fn test_coordinates_f64() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = ExodusFile::create(
                path,
                CreateOptions {
                    mode: CreateMode::Clobber,
                    float_size: FloatSize::Float64,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "f64 Coordinates".to_string(),
                num_dim: 3,
                num_nodes: 4,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![0.0_f64, 1.0, 2.0, 3.0];
            let y = vec![0.0_f64, 1.0, 2.0, 3.0];
            let z = vec![0.0_f64, 1.0, 2.0, 3.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        assert_eq!(coords.x.len(), 4);
        assert_eq!(coords.x[2], 2.0_f64);
        assert_eq!(coords.y[3], 3.0_f64);
    }

    // ========================================================================
    // Type Conversion Tests
    // ========================================================================

    #[test]
    fn test_write_f32_read_f64() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Type Conversion".to_string(),
                num_dim: 3,
                num_nodes: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![1.5_f32, 2.5, 3.5];
            let y = vec![4.5_f32, 5.5, 6.5];
            let z = vec![7.5_f32, 8.5, 9.5];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        assert!((coords.x[0] - 1.5).abs() < 1e-6);
        assert!((coords.y[1] - 5.5).abs() < 1e-6);
        assert!((coords.z[2] - 9.5).abs() < 1e-6);
    }

    #[test]
    fn test_write_f64_read_f32() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Type Conversion".to_string(),
                num_dim: 3,
                num_nodes: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![1.5_f64, 2.5, 3.5];
            let y = vec![4.5_f64, 5.5, 6.5];
            let z = vec![7.5_f64, 8.5, 9.5];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f32>().unwrap();

        assert!((coords.x[0] - 1.5_f32).abs() < 1e-6);
        assert!((coords.y[1] - 5.5_f32).abs() < 1e-6);
        assert!((coords.z[2] - 9.5_f32).abs() < 1e-6);
    }

    // ========================================================================
    // Partial Coordinate I/O Tests
    // ========================================================================

    #[test]
    fn test_partial_coordinate_write() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Partial Write".to_string(),
                num_dim: 3,
                num_nodes: 10,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Write first 5 nodes
            let x1 = vec![0.0_f64, 1.0, 2.0, 3.0, 4.0];
            let y1 = vec![0.0_f64; 5];
            let z1 = vec![0.0_f64; 5];
            file.put_partial_coords(0, 5, &x1, Some(&y1), Some(&z1)).unwrap();

            // Write next 5 nodes
            let x2 = vec![5.0_f64, 6.0, 7.0, 8.0, 9.0];
            let y2 = vec![1.0_f64; 5];
            let z2 = vec![2.0_f64; 5];
            file.put_partial_coords(5, 5, &x2, Some(&y2), Some(&z2)).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        assert_eq!(coords.x.len(), 10);
        assert_eq!(coords.x[4], 4.0);
        assert_eq!(coords.x[5], 5.0);
        assert_eq!(coords.y[7], 1.0);
        assert_eq!(coords.z[9], 2.0);
    }

    #[test]
    fn test_partial_coordinate_read() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Partial Read".to_string(),
                num_dim: 3,
                num_nodes: 10,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = (0..10).map(|i| i as f64).collect::<Vec<_>>();
            let y = vec![1.0_f64; 10];
            let z = vec![2.0_f64; 10];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();

        // Read middle 5 nodes
        let coords = file.get_partial_coords::<f64>(3, 5).unwrap();

        assert_eq!(coords.x.len(), 5);
        assert_eq!(coords.x[0], 3.0);
        assert_eq!(coords.x[4], 7.0);
    }

    // ========================================================================
    // Array Length Validation Tests
    // ========================================================================

    #[test]
    fn test_coord_length_mismatch_xy() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Length Mismatch".to_string(),
            num_dim: 2,
            num_nodes: 5,
            ..Default::default()
        };
        file.init(&params).unwrap();

        let x = vec![0.0_f64, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0_f64, 1.0, 2.0]; // Wrong length

        let result = file.put_coords(&x, Some(&y), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_coord_length_mismatch_xyz() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Length Mismatch".to_string(),
            num_dim: 3,
            num_nodes: 5,
            ..Default::default()
        };
        file.init(&params).unwrap();

        let x = vec![0.0_f64, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0_f64, 1.0, 2.0, 3.0, 4.0];
        let z = vec![0.0_f64, 1.0]; // Wrong length

        let result = file.put_coords(&x, Some(&y), Some(&z));
        assert!(result.is_err());
    }

    #[test]
    fn test_coord_length_exceeds_num_nodes() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Too Many Coords".to_string(),
            num_dim: 3,
            num_nodes: 3, // Only 3 nodes
            ..Default::default()
        };
        file.init(&params).unwrap();

        // Try to write 5 nodes
        let x = vec![0.0_f64, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0_f64, 1.0, 2.0, 3.0, 4.0];
        let z = vec![0.0_f64, 1.0, 2.0, 3.0, 4.0];

        let result = file.put_coords(&x, Some(&y), Some(&z));
        assert!(result.is_err());
    }

    // ========================================================================
    // Empty Coordinate Tests
    // ========================================================================

    #[test]
    fn test_empty_coordinates() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Empty Mesh".to_string(),
                num_dim: 3,
                num_nodes: 0,
                ..Default::default()
            };
            file.init(&params).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        assert!(coords.x.is_empty());
        assert!(coords.y.is_empty());
        assert!(coords.z.is_empty());
    }

    #[test]
    fn test_write_empty_coordinates() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Empty Coords".to_string(),
            num_dim: 3,
            num_nodes: 0,
            ..Default::default()
        };
        file.init(&params).unwrap();

        let x: Vec<f64> = vec![];
        let y: Vec<f64> = vec![];
        let z: Vec<f64> = vec![];

        // Writing empty coordinates should succeed
        let result = file.put_coords(&x, Some(&y), Some(&z));
        assert!(result.is_ok());
    }

    // ========================================================================
    // Large Coordinate Array Tests (10k+ nodes)
    // ========================================================================

    #[test]
    fn test_large_coordinate_array_10k() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let num_nodes = 10_000;

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Large Mesh".to_string(),
                num_dim: 3,
                num_nodes,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x: Vec<f64> = (0..num_nodes).map(|i| i as f64 * 0.1).collect();
            let y: Vec<f64> = (0..num_nodes).map(|i| (i as f64).sin()).collect();
            let z: Vec<f64> = (0..num_nodes).map(|i| (i as f64).cos()).collect();

            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        assert_eq!(coords.x.len(), num_nodes);
        assert_eq!(coords.y.len(), num_nodes);
        assert_eq!(coords.z.len(), num_nodes);

        // Verify some values
        assert!((coords.x[100] - 10.0).abs() < 1e-10);
        assert!((coords.y[0] - 0.0_f64.sin()).abs() < 1e-10);
    }

    #[test]
    fn test_large_coordinate_array_100k() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let num_nodes = 100_000;

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Very Large Mesh".to_string(),
                num_dim: 3,
                num_nodes,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x: Vec<f64> = (0..num_nodes).map(|i| i as f64).collect();
            let y: Vec<f64> = vec![0.0; num_nodes];
            let z: Vec<f64> = vec![0.0; num_nodes];

            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        assert_eq!(coords.x.len(), num_nodes);
        assert_eq!(coords.x[99_999], 99_999.0);
    }

    // ========================================================================
    // Additional Edge Case Tests
    // ========================================================================

    #[test]
    fn test_coordinates_with_negative_values() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Negative Coords".to_string(),
                num_dim: 3,
                num_nodes: 4,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![-1.0_f64, -0.5, 0.5, 1.0];
            let y = vec![-2.0_f64, -1.0, 1.0, 2.0];
            let z = vec![-3.0_f64, -1.5, 1.5, 3.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        assert_eq!(coords.x[0], -1.0);
        assert_eq!(coords.y[0], -2.0);
        assert_eq!(coords.z[0], -3.0);
    }

    #[test]
    fn test_coordinates_with_special_float_values() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Special Values".to_string(),
                num_dim: 3,
                num_nodes: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![0.0_f64, 1e-10, 1e10];
            let y = vec![0.0_f64, std::f64::consts::PI, std::f64::consts::E];
            let z = vec![0.0_f64, -1e-10, -1e10];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        assert!((coords.x[1] - 1e-10).abs() < 1e-15);
        assert!((coords.x[2] - 1e10).abs() < 1.0);
        assert!((coords.y[1] - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_coordinate_roundtrip_precision() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let original_x = vec![1.23456789_f64, 2.3456789, 3.456789];
        let original_y = vec![4.56789_f64, 5.6789, 6.789];
        let original_z = vec![7.89_f64, 8.9, 9.0];

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Precision Test".to_string(),
                num_dim: 3,
                num_nodes: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();
            file.put_coords(&original_x, Some(&original_y), Some(&original_z))
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let coords = file.coords::<f64>().unwrap();

        for i in 0..3 {
            assert!((coords.x[i] - original_x[i]).abs() < 1e-10);
            assert!((coords.y[i] - original_y[i]).abs() < 1e-10);
            assert!((coords.z[i] - original_z[i]).abs() < 1e-10);
        }
    }
}
