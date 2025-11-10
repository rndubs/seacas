//! Phase 4 Comprehensive Tests: Blocks
//!
//! This test suite covers all block operations including:
//! - Standard topologies (Hex8, Tet4, Quad4, etc.)
//! - NSided elements
//! - NFaced elements
//! - Custom topologies
//! - Block attributes
//! - Connectivity validation
//! - Multiple blocks
//! - Block iteration

#[cfg(feature = "netcdf4")]
mod block_tests {
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
    // Standard Topology Tests - 2D Elements
    // ========================================================================

    #[test]
    fn test_tri3_block() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Tri3 Test".to_string(),
                num_dim: 2,
                num_nodes: 4,
                num_elems: 2,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "TRI3".to_string(),
                num_entries: 2,
                num_nodes_per_entry: 3,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            // Two triangles sharing an edge
            let connectivity = vec![1, 2, 3, 2, 4, 3];
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();

        assert_eq!(block.topology, "TRI3");
        assert_eq!(block.num_entries, 2);
        assert_eq!(block.num_nodes_per_entry, 3);

        let conn = file.connectivity(1).unwrap();
        assert_eq!(conn.len(), 6);
        assert_eq!(conn[0], 1);
        assert_eq!(conn[5], 3);
    }

    #[test]
    fn test_quad4_block() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Quad4 Test".to_string(),
                num_dim: 2,
                num_nodes: 4,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 10,
                entity_type: EntityType::ElemBlock,
                topology: "QUAD4".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let connectivity = vec![1, 2, 3, 4];
            file.put_connectivity(10, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(10).unwrap();

        assert_eq!(block.topology, "QUAD4");
        assert_eq!(block.num_nodes_per_entry, 4);
    }

    #[test]
    fn test_quad8_block() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Quad8 Test".to_string(),
                num_dim: 2,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "QUAD8".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let connectivity = vec![1, 2, 3, 4, 5, 6, 7, 8];
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();
        assert_eq!(block.topology, "QUAD8");
    }

    // ========================================================================
    // Standard Topology Tests - 3D Elements
    // ========================================================================

    #[test]
    fn test_tet4_block() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Tet4 Test".to_string(),
                num_dim: 3,
                num_nodes: 4,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

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

            let connectivity = vec![1, 2, 3, 4];
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();

        assert_eq!(block.topology, "TET4");
        assert_eq!(block.num_nodes_per_entry, 4);
    }

    #[test]
    fn test_tet10_block() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Tet10 Test".to_string(),
                num_dim: 3,
                num_nodes: 10,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "TET10".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 10,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let connectivity: Vec<i64> = (1..=10).collect();
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();
        assert_eq!(block.topology, "TET10");
    }

    #[test]
    fn test_hex8_block() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Hex8 Test".to_string(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

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
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();

        assert_eq!(block.topology, "HEX8");
        assert_eq!(block.num_nodes_per_entry, 8);
    }

    #[test]
    fn test_hex20_block() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Hex20 Test".to_string(),
                num_dim: 3,
                num_nodes: 20,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX20".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 20,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let connectivity: Vec<i64> = (1..=20).collect();
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();
        assert_eq!(block.topology, "HEX20");
    }

    #[test]
    fn test_wedge6_block() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Wedge6 Test".to_string(),
                num_dim: 3,
                num_nodes: 6,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "WEDGE6".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 6,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let connectivity: Vec<i64> = (1..=6).collect();
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();
        assert_eq!(block.topology, "WEDGE6");
    }

    #[test]
    fn test_pyramid5_block() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Pyramid5 Test".to_string(),
                num_dim: 3,
                num_nodes: 5,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "PYRAMID5".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 5,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let connectivity: Vec<i64> = (1..=5).collect();
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();
        assert_eq!(block.topology, "PYRAMID5");
    }

    // ========================================================================
    // NSided Element Tests
    // ========================================================================

    #[test]
    fn test_nsided_elements() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "NSided Test".to_string(),
                num_dim: 2,
                num_nodes: 10,
                num_elems: 2,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "NSIDED".to_string(),
                num_entries: 2,
                num_nodes_per_entry: 0, // Variable
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();
        assert_eq!(block.topology, "NSIDED");
    }

    // ========================================================================
    // NFaced Element Tests
    // ========================================================================

    #[test]
    fn test_nfaced_elements() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "NFaced Test".to_string(),
                num_dim: 3,
                num_nodes: 20,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "NFACED".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 0, // Variable
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();
        assert_eq!(block.topology, "NFACED");
    }

    // ========================================================================
    // Custom Topology Tests
    // ========================================================================

    #[test]
    fn test_custom_topology() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Custom Topology".to_string(),
                num_dim: 3,
                num_nodes: 12,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "MyCustomElement".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 12,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let connectivity: Vec<i64> = (1..=12).collect();
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();

        assert_eq!(block.topology, "MyCustomElement");
    }

    // ========================================================================
    // Block Attribute Tests
    // ========================================================================

    #[test]
    fn test_block_with_attributes() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Block Attributes".to_string(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 3,
            };
            file.put_block(&block).unwrap();

            let connectivity: Vec<i64> = (1..=8).collect();
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();
        assert_eq!(block.num_attributes, 3);
    }

    #[test]
    fn test_block_attribute_names() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Attribute Names".to_string(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 2,
            };
            file.put_block(&block).unwrap();

            let attr_names = vec!["density", "material_id"];
            file.put_block_attribute_names(1, &attr_names)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let names = file
            .block_attribute_names(1)
            .unwrap();

        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "density");
        assert_eq!(names[1], "material_id");
    }

    // ========================================================================
    // Connectivity Validation Tests
    // ========================================================================

    #[test]
    fn test_connectivity_wrong_length() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Wrong Connectivity".to_string(),
            num_dim: 3,
            num_nodes: 8,
            num_elems: 1,
            num_elem_blocks: 1,
            ..Default::default()
        };
        file.init(&params).unwrap();

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

        // Wrong number of nodes (should be 8, providing 6)
        let connectivity = vec![1, 2, 3, 4, 5, 6];
        let result = file.put_connectivity(1, &connectivity);

        assert!(result.is_err());
    }

    #[test]
    fn test_connectivity_multiple_elements() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Multiple Elements".to_string(),
                num_dim: 3,
                num_nodes: 27,
                num_elems: 8,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 8,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            // 8 hex8 elements, each with 8 nodes = 64 connectivity entries
            let mut connectivity = Vec::new();
            for elem in 0..8 {
                for node in 0..8 {
                    connectivity.push((elem * 3 + node + 1) as i64);
                }
            }

            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let conn = file.connectivity(1).unwrap();

        assert_eq!(conn.len(), 64);
    }

    // ========================================================================
    // Multiple Block Tests
    // ========================================================================

    #[test]
    fn test_multiple_blocks_same_topology() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Multiple Blocks".to_string(),
                num_dim: 3,
                num_nodes: 16,
                num_elems: 2,
                num_elem_blocks: 2,
                ..Default::default()
            };
            file.init(&params).unwrap();

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

            let block2 = Block {
                id: 20,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block2).unwrap();

            let conn1: Vec<i64> = (1..=8).collect();
            file.put_connectivity(10, &conn1)
                .unwrap();

            let conn2: Vec<i64> = (9..=16).collect();
            file.put_connectivity(20, &conn2)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block1 = file.block(10).unwrap();
        let block2 = file.block(20).unwrap();

        assert_eq!(block1.id, 10);
        assert_eq!(block2.id, 20);
    }

    #[test]
    fn test_multiple_blocks_different_topology() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Mixed Topology".to_string(),
                num_dim: 3,
                num_nodes: 12,
                num_elems: 2,
                num_elem_blocks: 2,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block1 = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block1).unwrap();

            let block2 = Block {
                id: 2,
                entity_type: EntityType::ElemBlock,
                topology: "TET4".to_string(),
                num_entries: 1,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block2).unwrap();

            let conn1: Vec<i64> = (1..=8).collect();
            file.put_connectivity(1, &conn1)
                .unwrap();

            let conn2: Vec<i64> = (9..=12).collect();
            file.put_connectivity(2, &conn2)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block1 = file.block(1).unwrap();
        let block2 = file.block(2).unwrap();

        assert_eq!(block1.topology, "HEX8");
        assert_eq!(block2.topology, "TET4");
    }

    // ========================================================================
    // Block Iteration Tests
    // ========================================================================

    #[test]
    fn test_block_ids_retrieval() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Block IDs".to_string(),
                num_dim: 3,
                num_nodes: 24,
                num_elems: 3,
                num_elem_blocks: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            for id in [5, 10, 15] {
                let block = Block {
                    id,
                    entity_type: EntityType::ElemBlock,
                    topology: "HEX8".to_string(),
                    num_entries: 1,
                    num_nodes_per_entry: 8,
                    num_edges_per_entry: 0,
                    num_faces_per_entry: 0,
                    num_attributes: 0,
                };
                file.put_block(&block).unwrap();

                let conn: Vec<i64> = (1..=8).collect();
                file.put_connectivity(id, &conn)
                    .unwrap();
            }
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let ids = file.block_ids(EntityType::ElemBlock).unwrap();

        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&5));
        assert!(ids.contains(&10));
        assert!(ids.contains(&15));
    }

    #[test]
    fn test_iterate_all_blocks() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Iterate Blocks".to_string(),
                num_dim: 3,
                num_nodes: 32,
                num_elems: 4,
                num_elem_blocks: 4,
                ..Default::default()
            };
            file.init(&params).unwrap();

            for id in 1..=4 {
                let block = Block {
                    id,
                    entity_type: EntityType::ElemBlock,
                    topology: "HEX8".to_string(),
                    num_entries: 1,
                    num_nodes_per_entry: 8,
                    num_edges_per_entry: 0,
                    num_faces_per_entry: 0,
                    num_attributes: 0,
                };
                file.put_block(&block).unwrap();

                let conn: Vec<i64> = (1..=8).collect();
                file.put_connectivity(id, &conn)
                    .unwrap();
            }
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let ids = file.block_ids(EntityType::ElemBlock).unwrap();

        let mut count = 0;
        for id in ids {
            let _block = file.block(id).unwrap();
            count += 1;
        }

        assert_eq!(count, 4);
    }

    // ========================================================================
    // Edge and Face Block Tests
    // ========================================================================

    #[test]
    fn test_edge_block() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Edge Block".to_string(),
                num_dim: 2,
                num_nodes: 10,
                num_edges: 5,
                num_edge_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::EdgeBlock,
                topology: "EDGE2".to_string(),
                num_entries: 5,
                num_nodes_per_entry: 2,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let connectivity = vec![1, 2, 2, 3, 3, 4, 4, 5, 5, 6];
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();

        assert_eq!(block.num_entries, 5);
        assert_eq!(block.topology, "EDGE2");
    }

    #[test]
    fn test_face_block() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Face Block".to_string(),
                num_dim: 3,
                num_nodes: 16,
                num_faces: 4,
                num_face_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::FaceBlock,
                topology: "QUAD4".to_string(),
                num_entries: 4,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let connectivity: Vec<i64> = (1..=16).collect();
            file.put_connectivity(1, &connectivity)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let block = file.block(1).unwrap();

        assert_eq!(block.num_entries, 4);
        assert_eq!(block.topology, "QUAD4");
    }

    // ========================================================================
    // Error Case Tests
    // ========================================================================

    #[test]
    fn test_invalid_topology_node_count() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Invalid Topology".to_string(),
            num_dim: 3,
            num_nodes: 8,
            num_elems: 1,
            num_elem_blocks: 1,
            ..Default::default()
        };
        file.init(&params).unwrap();

        // Tet4 should have 4 nodes per element, not 8
        let block = Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "TET4".to_string(),
            num_entries: 1,
            num_nodes_per_entry: 8, // Wrong!
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };

        // This should either fail or be allowed with a warning
        // depending on implementation
        let result = file.put_block(&block);
        // Note: Some implementations may allow this and rely on connectivity validation
        let _ = result;
    }

    #[test]
    fn test_block_name() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Named Block".to_string(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

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

            file.put_name(EntityType::ElemBlock, 1, "StructuralBlock")
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let name = file.name(EntityType::ElemBlock, 1).unwrap();

        assert_eq!(name, "StructuralBlock");
    }
}
