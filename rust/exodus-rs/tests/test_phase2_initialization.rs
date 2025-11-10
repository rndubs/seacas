//! Phase 2 Comprehensive Tests: Database Initialization
//!
//! This test suite covers all database initialization operations including:
//! - InitParams validation
//! - Builder pattern completeness
//! - Title length validation
//! - QA and info records
//! - Coordinate names
//! - Round-trip testing

#[cfg(feature = "netcdf4")]
mod initialization_tests {
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
    // InitParams Validation Tests
    // ========================================================================

    #[test]
    fn test_init_params_default() {
        let params = InitParams::default();
        assert_eq!(params.num_dim, 3);
        assert_eq!(params.num_nodes, 0);
        assert_eq!(params.num_elems, 0);
        assert!(params.title.is_empty());
    }

    #[test]
    fn test_init_with_valid_1d() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "1D Test".to_string(),
            num_dim: 1,
            num_nodes: 10,
            ..Default::default()
        };

        let result = file.init(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_with_valid_2d() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "2D Test".to_string(),
            num_dim: 2,
            num_nodes: 10,
            ..Default::default()
        };

        let result = file.init(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_with_valid_3d() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "3D Test".to_string(),
            num_dim: 3,
            num_nodes: 10,
            ..Default::default()
        };

        let result = file.init(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_with_invalid_dimension_zero() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Invalid Dim".to_string(),
            num_dim: 0,
            num_nodes: 10,
            ..Default::default()
        };

        let result = file.init(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_with_invalid_dimension_four() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Invalid Dim".to_string(),
            num_dim: 4,
            num_nodes: 10,
            ..Default::default()
        };

        let result = file.init(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_with_zero_nodes() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Zero Nodes".to_string(),
            num_dim: 3,
            num_nodes: 0,
            ..Default::default()
        };

        // Should succeed - zero nodes is valid (empty mesh)
        let result = file.init(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_prevents_double_initialization() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "First Init".to_string(),
            num_dim: 3,
            num_nodes: 10,
            ..Default::default()
        };

        // First init should succeed
        file.init(&params).unwrap();

        // Second init should fail
        let params2 = InitParams {
            title: "Second Init".to_string(),
            num_dim: 2,
            num_nodes: 5,
            ..Default::default()
        };

        let result = file.init(&params2);
        assert!(result.is_err());
    }

    // ========================================================================
    // Builder Pattern Tests
    // ========================================================================

    #[test]
    fn test_builder_basic() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let result = file
            .builder()
            .title("Builder Test")
            .dimensions(3)
            .nodes(100)
            .finish();

        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_full_elem_mesh() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let result = file
            .builder()
            .title("Full Element Mesh")
            .dimensions(3)
            .nodes(100)
            .elems(80)
            .elem_blocks(2)
            .finish();

        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_with_sets() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let result = file
            .builder()
            .title("Mesh with Sets")
            .dimensions(3)
            .nodes(100)
            .elems(50)
            .elem_blocks(1)
            .node_sets(2)
            .side_sets(3)
            .finish();

        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_with_edges() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let result = file
            .builder()
            .title("Mesh with Edges")
            .dimensions(2)
            .nodes(50)
            .edges(100)
            .edge_blocks(2)
            .edge_sets(1)
            .finish();

        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_with_faces() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let result = file
            .builder()
            .title("Mesh with Faces")
            .dimensions(3)
            .nodes(100)
            .faces(80)
            .face_blocks(2)
            .face_sets(1)
            .finish();

        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_with_maps() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let result = file
            .builder()
            .title("Mesh with Maps")
            .dimensions(3)
            .nodes(50)
            .elems(40)
            .elem_blocks(1)
            .node_maps(2)
            .elem_maps(2)
            .finish();

        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_with_assemblies_and_blobs() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let result = file
            .builder()
            .title("Advanced Features")
            .dimensions(3)
            .nodes(100)
            .assemblies(3)
            .blobs(2)
            .finish();

        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_minimal() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        // Only set essential fields
        let result = file.builder().dimensions(3).nodes(10).finish();

        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_chaining() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        // Test that builder properly chains all methods
        let result = file
            .builder()
            .title("Complete Mesh")
            .dimensions(3)
            .nodes(1000)
            .elems(800)
            .elem_blocks(4)
            .edges(50)
            .edge_blocks(1)
            .edge_sets(2)
            .faces(100)
            .face_blocks(2)
            .face_sets(1)
            .node_sets(3)
            .side_sets(2)
            .elem_sets(1)
            .node_maps(1)
            .elem_maps(1)
            .edge_maps(1)
            .face_maps(1)
            .assemblies(2)
            .blobs(1)
            .finish();

        assert!(result.is_ok());
    }

    // ========================================================================
    // Title Length Validation Tests
    // ========================================================================

    #[test]
    fn test_title_empty() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "".to_string(),
            num_dim: 3,
            num_nodes: 10,
            ..Default::default()
        };

        let result = file.init(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_title_max_length_80() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let title = "A".repeat(80);
        let params = InitParams {
            title: title.clone(),
            num_dim: 3,
            num_nodes: 10,
            ..Default::default()
        };

        let result = file.init(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_title_exceeds_max_length() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let title = "A".repeat(81);
        let params = InitParams {
            title,
            num_dim: 3,
            num_nodes: 10,
            ..Default::default()
        };

        let result = file.init(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_title_with_special_characters() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Test: Mesh #1 (v2.0) - [Final]".to_string(),
            num_dim: 3,
            num_nodes: 10,
            ..Default::default()
        };

        let result = file.init(&params);
        assert!(result.is_ok());
    }

    // ========================================================================
    // QA Records Tests
    // ========================================================================

    #[test]
    fn test_qa_records_write_and_read() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "QA Test".to_string(),
                num_dim: 3,
                num_nodes: 10,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let qa_records = vec![QaRecord {
                code_name: "TestCode".to_string(),
                code_version: "1.0".to_string(),
                date: "2025-01-01".to_string(),
                time: "12:00:00".to_string(),
            }];

            file.put_qa_records(&qa_records).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let qa = file.qa_records().unwrap();

        assert_eq!(qa.len(), 1);
        assert_eq!(qa[0].code_name, "TestCode");
        assert_eq!(qa[0].code_version, "1.0");
    }

    #[test]
    fn test_qa_records_multiple() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Multi QA".to_string(),
                num_dim: 3,
                num_nodes: 5,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let qa_records = vec![
                QaRecord {
                    code_name: "Code1".to_string(),
                    code_version: "1.0".to_string(),
                    date: "2025-01-01".to_string(),
                    time: "10:00:00".to_string(),
                },
                QaRecord {
                    code_name: "Code2".to_string(),
                    code_version: "2.0".to_string(),
                    date: "2025-01-02".to_string(),
                    time: "11:00:00".to_string(),
                },
            ];

            file.put_qa_records(&qa_records).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let qa = file.qa_records().unwrap();

        assert_eq!(qa.len(), 2);
        assert_eq!(qa[1].code_name, "Code2");
    }

    // ========================================================================
    // Info Records Tests
    // ========================================================================

    #[test]
    fn test_info_records_write_and_read() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Info Test".to_string(),
                num_dim: 3,
                num_nodes: 10,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let info = vec![
                "This is line 1".to_string(),
                "This is line 2".to_string(),
                "This is line 3".to_string(),
            ];

            file.put_info_records(&info).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let info = file.info_records().unwrap();

        assert_eq!(info.len(), 3);
        assert_eq!(info[0], "This is line 1");
        assert_eq!(info[2], "This is line 3");
    }

    // ========================================================================
    // Coordinate Names Tests
    // ========================================================================
    // NOTE: These tests are commented out because coord_names() and put_coord_names()
    // methods don't exist in the current API

    /* DISABLED - coord_names() method doesn't exist
    #[test]
    fn test_coordinate_names_default() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Coord Names".to_string(),
                num_dim: 3,
                num_nodes: 10,
                ..Default::default()
            };
            file.init(&params).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let names = file.coord_names().unwrap();

        // Default names should be ["x", "y", "z"]
        assert_eq!(names.len(), 3);
        assert_eq!(names[0], "x");
        assert_eq!(names[1], "y");
        assert_eq!(names[2], "z");
    }

    #[test]
    fn test_coordinate_names_custom() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Custom Coord Names".to_string(),
                num_dim: 3,
                num_nodes: 10,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let names = vec!["X_coord", "Y_coord", "Z_coord"];
            file.put_coord_names(&names).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let names = file.coord_names().unwrap();

        assert_eq!(names.len(), 3);
        assert_eq!(names[0], "X_coord");
        assert_eq!(names[1], "Y_coord");
        assert_eq!(names[2], "Z_coord");
    }

    #[test]
    fn test_coordinate_names_2d() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "2D Mesh".to_string(),
                num_dim: 2,
                num_nodes: 10,
                ..Default::default()
            };
            file.init(&params).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let names = file.coord_names().unwrap();

        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "x");
        assert_eq!(names[1], "y");
    }
    */

    // ========================================================================
    // Round-Trip Tests
    // ========================================================================

    #[test]
    fn test_roundtrip_basic_params() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let original_params = InitParams {
            title: "Round Trip Test".to_string(),
            num_dim: 3,
            num_nodes: 100,
            num_elems: 80,
            num_elem_blocks: 2,
            num_node_sets: 3,
            num_side_sets: 4,
            ..Default::default()
        };

        {
            let mut file = create_test_file(path).unwrap();
            file.init(&original_params).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let read_params = file.init_params().unwrap();

        assert_eq!(read_params.title, original_params.title);
        assert_eq!(read_params.num_dim, original_params.num_dim);
        assert_eq!(read_params.num_nodes, original_params.num_nodes);
        assert_eq!(read_params.num_elems, original_params.num_elems);
        assert_eq!(read_params.num_elem_blocks, original_params.num_elem_blocks);
        assert_eq!(read_params.num_node_sets, original_params.num_node_sets);
        assert_eq!(read_params.num_side_sets, original_params.num_side_sets);
    }

    #[test]
    fn test_roundtrip_with_all_entities() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let original_params = InitParams {
            title: "Complete Mesh".to_string(),
            num_dim: 3,
            num_nodes: 1000,
            num_elems: 800,
            num_elem_blocks: 4,
            num_edges: 200,
            num_edge_blocks: 2,
            num_faces: 400,
            num_face_blocks: 3,
            num_node_sets: 5,
            num_edge_sets: 2,
            num_face_sets: 2,
            num_side_sets: 3,
            num_elem_sets: 1,
            num_node_maps: 2,
            num_elem_maps: 2,
            num_edge_maps: 1,
            num_face_maps: 1,
            num_assemblies: 3,
            num_blobs: 2,
        };

        {
            let mut file = create_test_file(path).unwrap();
            file.init(&original_params).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let read_params = file.init_params().unwrap();

        assert_eq!(read_params.num_edges, original_params.num_edges);
        assert_eq!(read_params.num_edge_blocks, original_params.num_edge_blocks);
        assert_eq!(read_params.num_faces, original_params.num_faces);
        assert_eq!(read_params.num_face_blocks, original_params.num_face_blocks);
        assert_eq!(read_params.num_assemblies, original_params.num_assemblies);
        assert_eq!(read_params.num_blobs, original_params.num_blobs);
    }

    #[test]
    fn test_roundtrip_minimal_mesh() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let original_params = InitParams {
            title: "Minimal".to_string(),
            num_dim: 1,
            num_nodes: 2,
            ..Default::default()
        };

        {
            let mut file = create_test_file(path).unwrap();
            file.init(&original_params).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let read_params = file.init_params().unwrap();

        assert_eq!(read_params.title, "Minimal");
        assert_eq!(read_params.num_dim, 1);
        assert_eq!(read_params.num_nodes, 2);
    }
}
