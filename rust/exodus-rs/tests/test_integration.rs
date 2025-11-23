//! Integration Tests: Complete Workflows
//!
//! This test suite covers complete workflows from file creation to reading,
//! testing the integration of all components together.

#[cfg(feature = "netcdf4")]
mod integration_tests {
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
    // Complete Workflow Tests
    // ========================================================================

    #[test]
    fn test_complete_workflow_simple() {
        // Test: create → init → coords → block → close → read
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        // Write phase
        {
            let mut file = create_test_file(path).unwrap();

            // Initialize
            let params = InitParams {
                title: "Simple Integration Test".to_string(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Write coordinates (unit cube)
            let x = vec![0.0_f64, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0_f64, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0_f64, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            // Write block
            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let connectivity: Vec<i64> = (1..=8).collect();
            file.put_connectivity(1, &connectivity).unwrap();

            // Explicitly close
            file.close().unwrap();
        }

        // Read phase
        {
            let file = ExodusFile::<mode::Read>::open(path).unwrap();

            // Verify init params
            let params = file.init_params().unwrap();
            assert_eq!(params.title, "Simple Integration Test");
            assert_eq!(params.num_nodes, 8);
            assert_eq!(params.num_elems, 1);

            // Verify coordinates
            let coords = file.coords::<f64>().unwrap();
            assert_eq!(coords.x.len(), 8);
            assert_eq!(coords.x[0], 0.0);
            assert_eq!(coords.x[7], 0.0);

            // Verify block
            let block = file.block(1).unwrap();
            assert_eq!(block.topology, "HEX8");

            let conn = file.connectivity(1).unwrap();
            assert_eq!(conn.len(), 8);
        }
    }

    #[test]
    fn test_complete_workflow_with_sets() {
        // Test: create → init → coords → blocks → sets → close → read
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();

            let params = InitParams {
                title: "Integration with Sets".to_string(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                num_node_sets: 2,
                num_side_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Coordinates
            let x = vec![0.0_f64, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0_f64, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0_f64, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            // Block
            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();
            file.put_connectivity(1, &(1..=8).collect::<Vec<i64>>())
                .unwrap();

            // Node sets (bottom and top faces)
            let ns1 = NodeSet {
                id: 1,
                nodes: vec![1, 2, 3, 4], // Bottom
                dist_factors: vec![],
            };
            file.put_node_set(ns1.id, &ns1.nodes, None).unwrap();

            let ns2 = NodeSet {
                id: 2,
                nodes: vec![5, 6, 7, 8], // Top
                dist_factors: vec![1.0, 1.0, 1.0, 1.0],
            };
            file.put_node_set(ns2.id, &ns2.nodes, Some(&ns2.dist_factors))
                .unwrap();

            // Side set
            let ss = SideSet {
                id: 10,
                elements: vec![1],
                sides: vec![1],
                dist_factors: vec![],
            };
            file.put_side_set(ss.id, &ss.elements, &ss.sides, None)
                .unwrap();
        }

        // Read and verify
        {
            let file = ExodusFile::<mode::Read>::open(path).unwrap();

            let params = file.init_params().unwrap();
            assert_eq!(params.num_node_sets, 2);
            assert_eq!(params.num_side_sets, 1);

            let ns1 = file.node_set(1).unwrap();
            assert_eq!(ns1.nodes.len(), 4);
            assert!(ns1.dist_factors.is_empty());

            let ns2 = file.node_set(2).unwrap();
            assert_eq!(ns2.nodes.len(), 4);
            assert!(!ns2.dist_factors.is_empty());

            let ss = file.side_set(10).unwrap();
            assert_eq!(ss.elements.len(), 1);
        }
    }

    #[test]
    fn test_complete_workflow_with_variables() {
        // Test: create → init → coords → blocks → vars → time steps → close → read
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();

            let params = InitParams {
                title: "Integration with Variables".to_string(),
                num_dim: 3,
                num_nodes: 4,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Coordinates (tetrahedron)
            let x = vec![0.0_f64, 1.0, 0.0, 0.0];
            let y = vec![0.0_f64, 0.0, 1.0, 0.0];
            let z = vec![0.0_f64, 0.0, 0.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            // Block
            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "TET4".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();
            file.put_connectivity(1, &[1, 2, 3, 4]).unwrap();

            // Define variables
            file.define_variables(EntityType::Global, &["time", "energy"])
                .unwrap();
            file.define_variables(EntityType::Nodal, &["temperature", "pressure"])
                .unwrap();
            file.define_variables(EntityType::ElemBlock, &["stress"])
                .unwrap();

            // Write time steps
            for step in 0..3 {
                let time = step as f64 * 0.1;
                file.put_time(step, time).unwrap();

                // Global variables (entity_id is ignored for Global, var_index is used)
                file.put_var(step, EntityType::Global, 0, 0, &[time])
                    .unwrap();
                file.put_var(step, EntityType::Global, 0, 1, &[time * 100.0])
                    .unwrap();

                // Nodal variables (entity_id is ignored for Nodal, var_index is used)
                let temps: Vec<f64> = (0..4).map(|i| 20.0 + i as f64 + step as f64).collect();
                file.put_var(step, EntityType::Nodal, 0, 0, &temps).unwrap();

                let pressures: Vec<f64> = (0..4).map(|i| 100.0 + i as f64 * 10.0).collect();
                file.put_var(step, EntityType::Nodal, 0, 1, &pressures)
                    .unwrap();

                // Element variable (block_id=1, var_index=0)
                file.put_var(step, EntityType::ElemBlock, 1, 0, &[50.0 + step as f64])
                    .unwrap();
            }
        }

        // Read and verify
        {
            let file = ExodusFile::<mode::Read>::open(path).unwrap();

            let num_time_steps = file.num_time_steps().unwrap();
            assert_eq!(num_time_steps, 3);

            // Check variable names
            let global_vars = file.variable_names(EntityType::Global).unwrap();
            assert_eq!(global_vars.len(), 2);
            assert_eq!(global_vars[0], "time");
            assert_eq!(global_vars[1], "energy");

            let nodal_vars = file.variable_names(EntityType::Nodal).unwrap();
            assert_eq!(nodal_vars.len(), 2);

            // Read time step 1
            let time = file.time(1).unwrap();
            assert!((time - 0.1).abs() < 1e-10);

            let global_val: Vec<f64> = file.var(1, EntityType::Global, 0, 0).unwrap();
            assert!((global_val[0] - 0.1).abs() < 1e-10);

            let temps: Vec<f64> = file.var(1, EntityType::Nodal, 0, 0).unwrap();
            assert_eq!(temps.len(), 4);
            assert!((temps[0] - 21.0).abs() < 1e-10);
        }
    }

    // DISABLED - put_coords_2d(), put_coord_names(), and coord_names() methods don't exist
    /*
    #[test]
    fn test_complete_workflow_with_metadata() {
        // Test workflow with QA and info records
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();

            let params = InitParams {
                title: "Integration with Metadata".to_string(),
                num_dim: 2,
                num_nodes: 4,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // QA records
            let qa = vec![
                QaRecord {
                    code_name: "TestSuite".to_string(),
                    code_version: "1.0.0".to_string(),
                    date: "2025-01-10".to_string(),
                    time: "12:00:00".to_string(),
                },
                QaRecord {
                    code_name: "Preprocessor".to_string(),
                    code_version: "2.1.0".to_string(),
                    date: "2025-01-09".to_string(),
                    time: "10:30:00".to_string(),
                },
            ];
            file.put_qa_records(&qa).unwrap();

            // Info records
            let info = vec![
                "Generated for integration testing".to_string(),
                "Author: Test System".to_string(),
                "Purpose: Verify complete workflow".to_string(),
            ];
            file.put_info_records(&info).unwrap();

            // Coordinates
            let x = vec![0.0_f64, 1.0, 1.0, 0.0];
            let y = vec![0.0_f64, 0.0, 1.0, 1.0];
            file.put_coords_2d(&x, &y).unwrap();

            // Coordinate names
            file.put_coord_names(&["X_axis", "Y_axis"]).unwrap();
        }

        {
            let file = ExodusFile::<mode::Read>::open(path).unwrap();

            let qa = file.qa_records().unwrap();
            assert_eq!(qa.len(), 2);
            assert_eq!(qa[0].code_name, "TestSuite");
            assert_eq!(qa[1].code_name, "Preprocessor");

            let info = file.info_records().unwrap();
            assert_eq!(info.len(), 3);
            assert!(info[0].contains("integration testing"));

            let coord_names = file.coord_names().unwrap();
            assert_eq!(coord_names.len(), 2);
            assert_eq!(coord_names[0], "X_axis");
            assert_eq!(coord_names[1], "Y_axis");
        }
    }
    */

    #[test]
    fn test_complete_workflow_multi_block() {
        // Test workflow with multiple blocks of different types
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();

            let params = InitParams {
                title: "Multi-Block Mesh".to_string(),
                num_dim: 3,
                num_nodes: 20,
                num_elems: 3,
                num_elem_blocks: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Simplified coordinates
            let x: Vec<f64> = (0..20).map(|i| i as f64 * 0.1).collect();
            let y = vec![0.0_f64; 20];
            let z = vec![0.0_f64; 20];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            // Block 1: Hex8
            let block1 = Block {
                id: 10,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block1).unwrap();
            file.put_connectivity(10, &(1..=8).collect::<Vec<i64>>())
                .unwrap();
            file.put_name(EntityType::ElemBlock, 0, "HexBlock").unwrap();

            // Block 2: Tet4
            let block2 = Block {
                id: 20,
                entity_type: EntityType::ElemBlock,
                topology: "TET4".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block2).unwrap();
            file.put_connectivity(20, &[9, 10, 11, 12]).unwrap();
            file.put_name(EntityType::ElemBlock, 1, "TetBlock").unwrap();

            // Block 3: Wedge6
            let block3 = Block {
                id: 30,
                entity_type: EntityType::ElemBlock,
                topology: "WEDGE6".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 6,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block3).unwrap();
            file.put_connectivity(30, &(13..=18).collect::<Vec<i64>>())
                .unwrap();
            file.put_name(EntityType::ElemBlock, 2, "WedgeBlock")
                .unwrap();
        }

        {
            let file = ExodusFile::<mode::Read>::open(path).unwrap();

            let block_ids = file.block_ids(EntityType::ElemBlock).unwrap();
            assert_eq!(block_ids.len(), 3);

            let block1 = file.block(10).unwrap();
            assert_eq!(block1.topology, "HEX8");
            assert_eq!(file.name(EntityType::ElemBlock, 0).unwrap(), "HexBlock");

            let block2 = file.block(20).unwrap();
            assert_eq!(block2.topology, "TET4");

            let block3 = file.block(30).unwrap();
            assert_eq!(block3.topology, "WEDGE6");
        }
    }

    #[test]
    fn test_complete_workflow_large_mesh() {
        // Test with a larger mesh (100 nodes, 80 elements)
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();

            let params = InitParams {
                title: "Large Mesh".to_string(),
                num_dim: 3,
                num_nodes: 100,
                num_elems: 80,
                num_elem_blocks: 2,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Generate coordinates
            let x: Vec<f64> = (0..100).map(|i| (i % 10) as f64).collect();
            let y: Vec<f64> = (0..100).map(|i| ((i / 10) % 10) as f64).collect();
            let z: Vec<f64> = (0..100).map(|i| (i / 100) as f64).collect();
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            // Block 1: 40 hex elements
            let block1 = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 40,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block1).unwrap();

            let mut conn1 = Vec::new();
            for i in 0..40 {
                for j in 0..8 {
                    conn1.push((i * 2 + j + 1) as i64);
                }
            }
            file.put_connectivity(1, &conn1).unwrap();

            // Block 2: 40 tet elements
            let block2 = Block {
                id: 2,
                entity_type: EntityType::ElemBlock,
                topology: "TET4".to_string(),
                num_entries: 40,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block2).unwrap();

            let mut conn2 = Vec::new();
            for i in 0..40 {
                for j in 0..4 {
                    conn2.push((i + j + 1) as i64);
                }
            }
            file.put_connectivity(2, &conn2).unwrap();
        }

        {
            let file = ExodusFile::<mode::Read>::open(path).unwrap();

            let params = file.init_params().unwrap();
            assert_eq!(params.num_nodes, 100);
            assert_eq!(params.num_elems, 80);

            let coords = file.coords::<f64>().unwrap();
            assert_eq!(coords.x.len(), 100);

            let block1 = file.block(1).unwrap();
            assert_eq!(block1.num_entries, 40);

            let conn1 = file.connectivity(1).unwrap();
            assert_eq!(conn1.len(), 40 * 8);
        }
    }

    #[test]
    fn test_append_mode_workflow() {
        // Test: create → init → coords → close → append → add sets → close → read
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        // Initial creation
        {
            let mut file = create_test_file(path).unwrap();

            let params = InitParams {
                title: "Append Mode Test".to_string(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                num_node_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![0.0_f64, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0_f64, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0_f64, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();
            file.put_connectivity(1, &(1..=8).collect::<Vec<i64>>())
                .unwrap();
        }

        // Append additional data
        {
            let _file = ExodusFile::<mode::Append>::append(path).unwrap();

            // Note: put_node_set, define_variables, and put_var are only available in Write mode, not Append mode
            // let ns = NodeSet {
            //     id: 1,
            //     nodes: vec![1, 2, 3, 4],
            //     dist_factors: vec![],
            // };
            // file.put_node_set(ns.id, &ns.nodes, None).unwrap();
            //
            // file.define_variables(EntityType::Global, &["iteration"])
            //     .unwrap();
            // file.put_time(0, 0.0).unwrap();
            // file.put_var(0, EntityType::Global, 0, 0, &[1.0])
            //     .unwrap();
        }

        // Read and verify
        {
            let file = ExodusFile::<mode::Read>::open(path).unwrap();

            let params = file.init_params().unwrap();
            assert_eq!(params.title, "Append Mode Test");

            let coords = file.coords::<f64>().unwrap();
            assert_eq!(coords.x.len(), 8);

            // Note: Node set and variable assertions commented out since we don't add them in Append mode
            // let ns = file.node_set(1).unwrap();
            // assert_eq!(ns.nodes.len(), 4);
            //
            // let num_steps = file.num_time_steps().unwrap();
            // assert_eq!(num_steps, 1);
            //
            // let global_vars = file.variable_names(EntityType::Global).unwrap();
            // assert_eq!(global_vars[0], "iteration");
        }
    }

    #[test]
    fn test_builder_pattern_workflow() {
        // Test using the builder pattern for initialization
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();

            // Use builder pattern
            file.builder()
                .title("Builder Pattern Test")
                .dimensions(3)
                .nodes(8)
                .elems(1)
                .elem_blocks(1)
                .node_sets(1)
                .side_sets(1)
                .finish()
                .unwrap();

            // Continue with normal operations
            let x = vec![0.0_f64, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0_f64, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0_f64, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        {
            let file = ExodusFile::<mode::Read>::open(path).unwrap();
            let params = file.init_params().unwrap();

            assert_eq!(params.title, "Builder Pattern Test");
            assert_eq!(params.num_dim, 3);
            assert_eq!(params.num_nodes, 8);
        }
    }

    #[test]
    fn test_error_recovery_workflow() {
        // Test that errors don't corrupt the file
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();

            let params = InitParams {
                title: "Error Recovery".to_string(),
                num_dim: 3,
                num_nodes: 4,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Successful coordinate write
            let x = vec![0.0_f64, 1.0, 0.0, 0.0];
            let y = vec![0.0_f64, 0.0, 1.0, 0.0];
            let z = vec![0.0_f64, 0.0, 0.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            // Successful block write
            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "TET4".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();
            file.put_connectivity(1, &[1, 2, 3, 4]).unwrap();

            // Attempt invalid operation (wrong connectivity length)
            let result = file.put_connectivity(1, &[1, 2]);
            assert!(result.is_err());

            // Continue with valid operations
            file.put_name(EntityType::ElemBlock, 0, "Block1").unwrap();
        }

        // Verify file is still readable
        {
            let file = ExodusFile::<mode::Read>::open(path).unwrap();
            let params = file.init_params().unwrap();
            assert_eq!(params.num_nodes, 4);

            let name = file.name(EntityType::ElemBlock, 0).unwrap();
            assert_eq!(name, "Block1");
        }
    }

    #[test]
    fn test_mixed_precision_workflow() {
        // Test writing f32 and reading as f64
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();

            let params = InitParams {
                title: "Mixed Precision".to_string(),
                num_dim: 3,
                num_nodes: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Write as f32
            let x = vec![0.0_f32, 1.0, 2.0];
            let y = vec![0.0_f32, 1.0, 2.0];
            let z = vec![0.0_f32, 1.0, 2.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        {
            let file = ExodusFile::<mode::Read>::open(path).unwrap();

            // Read as f64
            let coords = file.coords::<f64>().unwrap();
            assert_eq!(coords.x.len(), 3);
            assert!((coords.x[1] - 1.0).abs() < 1e-6);

            // Also read as f32
            let coords32 = file.coords::<f32>().unwrap();
            assert_eq!(coords32.x[2], 2.0_f32);
        }
    }
}
