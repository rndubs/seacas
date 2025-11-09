//! Comprehensive tests for Phase 3: Coordinate Operations

use exodus_rs::{mode, CreateMode, CreateOptions, ExodusFile, InitParams};
use tempfile::NamedTempFile;

#[test]
fn test_1d_coordinates() {
    let tmp = NamedTempFile::new().unwrap();

    let x_coords = vec![0.0, 1.0, 2.0, 3.0];

    // Write
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
            title: "1D Test".into(),
            num_dim: 1,
            num_nodes: 4,
            ..Default::default()
        })
        .unwrap();

        file.put_coords(&x_coords, &[], &[]).unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let (x, y, z) = file.coords().unwrap();
        assert_eq!(x, x_coords);
        assert_eq!(y.len(), 0);
        assert_eq!(z.len(), 0);
    }
}

#[test]
fn test_2d_coordinates() {
    let tmp = NamedTempFile::new().unwrap();

    let x_coords = vec![0.0, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0, 0.0, 1.0, 1.0];

    // Write
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
            title: "2D Test".into(),
            num_dim: 2,
            num_nodes: 4,
            ..Default::default()
        })
        .unwrap();

        file.put_coords(&x_coords, &y_coords, &[]).unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let (x, y, z) = file.coords().unwrap();
        assert_eq!(x, x_coords);
        assert_eq!(y, y_coords);
        assert_eq!(z.len(), 0);
    }
}

#[test]
fn test_3d_coordinates() {
    let tmp = NamedTempFile::new().unwrap();

    let x_coords = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
    let z_coords = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];

    // Write
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
            title: "3D Test".into(),
            num_dim: 3,
            num_nodes: 8,
            ..Default::default()
        })
        .unwrap();

        file.put_coords(&x_coords, &y_coords, &z_coords)
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let (x, y, z) = file.coords().unwrap();
        assert_eq!(x, x_coords);
        assert_eq!(y, y_coords);
        assert_eq!(z, z_coords);
    }
}

#[test]
fn test_coordinate_array_length_validation() {
    let tmp = NamedTempFile::new().unwrap();

    let mut file = ExodusFile::create(
        tmp.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
    .unwrap();

    file.init(&InitParams {
        title: "Length Test".into(),
        num_dim: 2,
        num_nodes: 4,
        ..Default::default()
    })
    .unwrap();

    // Wrong x length
    let result = file.put_coords(&vec![0.0, 1.0], &vec![0.0, 0.0, 1.0, 1.0], &[]);
    assert!(result.is_err(), "Should fail with wrong x length");

    // Wrong y length
    let result = file.put_coords(&vec![0.0, 1.0, 1.0, 0.0], &vec![0.0, 0.0], &[]);
    assert!(result.is_err(), "Should fail with wrong y length");
}

#[test]
fn test_single_node_coordinates() {
    let tmp = NamedTempFile::new().unwrap();

    // Write
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
            title: "Single Node".into(),
            num_dim: 3,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();

        file.put_coords(&vec![5.0], &vec![10.0], &vec![15.0])
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let (x, y, z) = file.coords().unwrap();
        assert_eq!(x, vec![5.0]);
        assert_eq!(y, vec![10.0]);
        assert_eq!(z, vec![15.0]);
    }
}

#[test]
fn test_large_coordinate_array() {
    let tmp = NamedTempFile::new().unwrap();

    let num_nodes = 10000;
    let x_coords: Vec<f64> = (0..num_nodes).map(|i| i as f64).collect();
    let y_coords: Vec<f64> = (0..num_nodes).map(|i| (i * 2) as f64).collect();

    // Write
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
            title: "Large Array".into(),
            num_dim: 2,
            num_nodes: num_nodes,
            ..Default::default()
        })
        .unwrap();

        file.put_coords(&x_coords, &y_coords, &[]).unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let (x, y, _z) = file.coords().unwrap();
        assert_eq!(x.len(), num_nodes);
        assert_eq!(y.len(), num_nodes);
        assert_eq!(x[0], 0.0);
        assert_eq!(x[num_nodes - 1], (num_nodes - 1) as f64);
        assert_eq!(y[num_nodes - 1], ((num_nodes - 1) * 2) as f64);
    }
}

#[test]
fn test_zero_coordinates() {
    let tmp = NamedTempFile::new().unwrap();

    // Write
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
            title: "Zero Coords".into(),
            num_dim: 3,
            num_nodes: 4,
            ..Default::default()
        })
        .unwrap();

        file.put_coords(&vec![0.0; 4], &vec![0.0; 4], &vec![0.0; 4])
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let (x, y, z) = file.coords().unwrap();
        assert_eq!(x, vec![0.0; 4]);
        assert_eq!(y, vec![0.0; 4]);
        assert_eq!(z, vec![0.0; 4]);
    }
}

#[test]
fn test_negative_coordinates() {
    let tmp = NamedTempFile::new().unwrap();

    let x_coords = vec![-1.0, -2.0, -3.0];
    let y_coords = vec![-4.0, -5.0, -6.0];

    // Write
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
            title: "Negative Coords".into(),
            num_dim: 2,
            num_nodes: 3,
            ..Default::default()
        })
        .unwrap();

        file.put_coords(&x_coords, &y_coords, &[]).unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let (x, y, _z) = file.coords().unwrap();
        assert_eq!(x, x_coords);
        assert_eq!(y, y_coords);
    }
}

#[test]
fn test_fractional_coordinates() {
    let tmp = NamedTempFile::new().unwrap();

    let x_coords = vec![0.123456789, 1.987654321, 2.555555555];

    // Write
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
            title: "Fractional Coords".into(),
            num_dim: 1,
            num_nodes: 3,
            ..Default::default()
        })
        .unwrap();

        file.put_coords(&x_coords, &[], &[]).unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let (x, _y, _z) = file.coords().unwrap();
        // Check with small epsilon for floating point comparison
        for (i, &coord) in x_coords.iter().enumerate() {
            assert!((x[i] - coord).abs() < 1e-10, "Coordinate mismatch");
        }
    }
}

#[test]
fn test_coordinate_names() {
    let tmp = NamedTempFile::new().unwrap();

    // Write
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
            title: "Coord Names Test".into(),
            num_dim: 3,
            num_nodes: 4,
            ..Default::default()
        })
        .unwrap();

        file.put_coord_names(&["X", "Y", "Z"]).unwrap();

        file.put_coords(&vec![0.0; 4], &vec![0.0; 4], &vec![0.0; 4])
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let names = file.coord_names().unwrap();
        assert_eq!(names, vec!["X", "Y", "Z"]);
    }
}

#[test]
fn test_custom_coordinate_names() {
    let tmp = NamedTempFile::new().unwrap();

    // Write
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
            title: "Custom Names".into(),
            num_dim: 2,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();

        file.put_coord_names(&["Radius", "Angle"]).unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let names = file.coord_names().unwrap();
        assert_eq!(names, vec!["Radius", "Angle"]);
    }
}

#[test]
fn test_coords_f32_precision() {
    let tmp = NamedTempFile::new().unwrap();

    let x_coords_f32 = vec![1.5f32, 2.5f32, 3.5f32];

    // Write as f32
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
            title: "F32 Test".into(),
            num_dim: 1,
            num_nodes: 3,
            ..Default::default()
        })
        .unwrap();

        file.put_coords_f32(&x_coords_f32, &[], &[]).unwrap();
    }

    // Read as f64
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let (x, _y, _z) = file.coords().unwrap();

        // Should convert correctly
        for (i, &val) in x_coords_f32.iter().enumerate() {
            assert!((x[i] - val as f64).abs() < 1e-6);
        }
    }
}

#[test]
fn test_mixed_precision_read_write() {
    let tmp = NamedTempFile::new().unwrap();

    // Write as f64
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
            title: "Mixed Precision".into(),
            num_dim: 2,
            num_nodes: 2,
            ..Default::default()
        })
        .unwrap();

        file.put_coords(&vec![1.0, 2.0], &vec![3.0, 4.0], &[])
            .unwrap();
    }

    // Read as f32
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let (x, y, _z) = file.coords_f32().unwrap();
        assert_eq!(x, vec![1.0f32, 2.0f32]);
        assert_eq!(y, vec![3.0f32, 4.0f32]);
    }
}

#[test]
fn test_extreme_coordinate_values() {
    let tmp = NamedTempFile::new().unwrap();

    let x_coords = vec![f64::MAX, f64::MIN, 0.0];

    // Write
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
            title: "Extreme Values".into(),
            num_dim: 1,
            num_nodes: 3,
            ..Default::default()
        })
        .unwrap();

        file.put_coords(&x_coords, &[], &[]).unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let (x, _y, _z) = file.coords().unwrap();
        assert_eq!(x[0], f64::MAX);
        assert_eq!(x[1], f64::MIN);
        assert_eq!(x[2], 0.0);
    }
}
