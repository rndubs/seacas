//! Phase 5 Comprehensive Tests: Sets
//!
//! This test suite covers all set operations including:
//! - Node sets with/without distribution factors
//! - Side sets (element-side pairs)
//! - Side set distribution factors
//! - Element sets
//! - Edge sets
//! - Face sets
//! - Empty sets
//! - Set iteration
//! - Error cases

#[cfg(feature = "netcdf4")]
mod set_tests {
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
    // Node Set Tests (with distribution factors)
    // ========================================================================

    #[test]
    fn test_node_set_with_dist_factors() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "NodeSet with DF".to_string(),
                num_dim: 3,
                num_nodes: 10,
                num_node_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let node_set = NodeSet {
                id: 100,
                nodes: vec![1, 3, 5, 7, 9],
                dist_factors: vec![1.0, 1.5, 2.0, 2.5, 3.0],
            };
            file.put_node_set(node_set.id, &node_set.nodes, Some(&node_set.dist_factors))
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let node_set = file.node_set(100).unwrap();

        assert_eq!(node_set.id, 100);
        assert_eq!(node_set.nodes.len(), 5);
        assert_eq!(node_set.nodes[0], 1);
        assert_eq!(node_set.nodes[4], 9);

        assert_eq!(node_set.dist_factors.len(), 5);
        assert!((node_set.dist_factors[0] - 1.0).abs() < 1e-10);
        assert!((node_set.dist_factors[4] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_node_set_multiple_with_dist_factors() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Multiple NodeSets".to_string(),
                num_dim: 3,
                num_nodes: 20,
                num_node_sets: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            for i in 1..=3 {
                let node_set = NodeSet {
                    id: i,
                    nodes: vec![i, i + 3, i + 6],
                    dist_factors: vec![1.0, 2.0, 3.0],
                };
                file.put_node_set(node_set.id, &node_set.nodes, Some(&node_set.dist_factors))
                    .unwrap();
            }
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let ns2 = file.node_set(2).unwrap();

        assert_eq!(ns2.id, 2);
        assert_eq!(ns2.nodes, vec![2, 5, 8]);
    }

    // ========================================================================
    // Node Set Tests (without distribution factors)
    // ========================================================================

    #[test]
    fn test_node_set_without_dist_factors() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "NodeSet no DF".to_string(),
                num_dim: 3,
                num_nodes: 10,
                num_node_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let node_set = NodeSet {
                id: 1,
                nodes: vec![1, 2, 3, 4],
                dist_factors: vec![],
            };
            file.put_node_set(node_set.id, &node_set.nodes, None)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let node_set = file.node_set(1).unwrap();

        assert_eq!(node_set.nodes.len(), 4);
        assert!(node_set.dist_factors.is_empty());
    }

    #[test]
    fn test_node_set_large() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        let num_nodes_in_set = 1000;

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Large NodeSet".to_string(),
                num_dim: 3,
                num_nodes: num_nodes_in_set,
                num_node_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let nodes: Vec<i64> = (1..=num_nodes_in_set as i64).collect();
            let df: Vec<f64> = (0..num_nodes_in_set).map(|i| i as f64 * 0.1).collect();

            let node_set = NodeSet {
                id: 1,
                nodes,
                dist_factors: df,
            };
            file.put_node_set(node_set.id, &node_set.nodes, Some(&node_set.dist_factors))
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let node_set = file.node_set(1).unwrap();

        assert_eq!(node_set.nodes.len(), num_nodes_in_set);
        assert_eq!(node_set.nodes[999], 1000);
    }

    // ========================================================================
    // Side Set Tests (element-side pairs)
    // ========================================================================

    #[test]
    fn test_side_set_basic() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "SideSet Test".to_string(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                num_side_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Create element block first
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

            let side_set = SideSet {
                id: 10,
                elements: vec![1, 1, 1],
                sides: vec![1, 2, 3],
                dist_factors: vec![],
            };
            file.put_side_set(side_set.id, &side_set.elements, &side_set.sides, None)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let side_set = file.side_set(10).unwrap();

        assert_eq!(side_set.id, 10);
        assert_eq!(side_set.elements.len(), 3);
        assert_eq!(side_set.sides.len(), 3);
        assert_eq!(side_set.elements[0], 1);
        assert_eq!(side_set.sides[2], 3);
    }

    #[test]
    fn test_side_set_with_dist_factors() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "SideSet with DF".to_string(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                num_side_sets: 1,
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

            // Hex8 has 6 faces, each with 4 nodes
            let side_set = SideSet {
                id: 1,
                elements: vec![1, 1],
                sides: vec![1, 2],
                dist_factors: vec![1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0],
            };
            file.put_side_set(
                side_set.id,
                &side_set.elements,
                &side_set.sides,
                Some(&side_set.dist_factors),
            )
            .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let side_set = file.side_set(1).unwrap();

        assert_eq!(side_set.dist_factors.len(), 8);
    }

    #[test]
    fn test_side_set_multiple_elements() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "SideSet Multi Elem".to_string(),
                num_dim: 3,
                num_nodes: 16,
                num_elems: 2,
                num_elem_blocks: 1,
                num_side_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 2,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let side_set = SideSet {
                id: 1,
                elements: vec![1, 2, 2],
                sides: vec![1, 1, 2],
                dist_factors: vec![],
            };
            file.put_side_set(side_set.id, &side_set.elements, &side_set.sides, None)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let side_set = file.side_set(1).unwrap();

        assert_eq!(side_set.elements.len(), 3);
        assert_eq!(side_set.elements[1], 2);
    }

    // ========================================================================
    // Element Set Tests
    // ========================================================================

    #[test]
    fn test_element_set_basic() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "ElemSet Test".to_string(),
                num_dim: 3,
                num_nodes: 32,
                num_elems: 4,
                num_elem_blocks: 1,
                num_elem_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 4,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let set = Set {
                id: 1,
                entity_type: EntityType::ElemSet,
                num_entries: 2,
                num_dist_factors: 0,
            };
            file.put_set(&set).unwrap();
            file.put_entity_set(EntityType::ElemSet, 1, &vec![1, 3])
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let elem_set = file.entity_set(EntityType::ElemSet, 1).unwrap();

        assert_eq!(elem_set.id, 1);
        assert_eq!(elem_set.entity_type, EntityType::ElemSet);
        assert_eq!(elem_set.entities.len(), 2);
        assert_eq!(elem_set.entities[0], 1);
        assert_eq!(elem_set.entities[1], 3);
    }

    #[test]
    fn test_element_set_with_dist_factors() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "ElemSet with DF".to_string(),
                num_dim: 3,
                num_nodes: 24,
                num_elems: 3,
                num_elem_blocks: 1,
                num_elem_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 3,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let set = Set {
                id: 50,
                entity_type: EntityType::ElemSet,
                num_entries: 3,
                num_dist_factors: 3,
            };
            file.put_set(&set).unwrap();
            file.put_entity_set(EntityType::ElemSet, 50, &vec![1, 2, 3])
                .unwrap();
            // Note: distribution factors not supported for entity sets in current API
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let elem_set = file.entity_set(EntityType::ElemSet, 50).unwrap();

        assert_eq!(elem_set.id, 50);
        assert_eq!(elem_set.entity_type, EntityType::ElemSet);
        assert_eq!(elem_set.entities, vec![1, 2, 3]);
    }

    // ========================================================================
    // Edge Set Tests
    // ========================================================================

    #[test]
    fn test_edge_set_basic() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "EdgeSet Test".to_string(),
                num_dim: 2,
                num_nodes: 10,
                num_edges: 5,
                num_edge_blocks: 1,
                num_edge_sets: 1,
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

            let set = Set {
                id: 1,
                entity_type: EntityType::EdgeSet,
                num_entries: 3,
                num_dist_factors: 0,
            };
            file.put_set(&set).unwrap();
            file.put_entity_set(EntityType::EdgeSet, 1, &vec![1, 3, 5])
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let edge_set = file.entity_set(EntityType::EdgeSet, 1).unwrap();

        assert_eq!(edge_set.entities.len(), 3);
        assert_eq!(edge_set.entities[2], 5);
    }

    #[test]
    fn test_edge_set_with_dist_factors() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "EdgeSet with DF".to_string(),
                num_dim: 2,
                num_nodes: 10,
                num_edges: 4,
                num_edge_blocks: 1,
                num_edge_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::EdgeBlock,
                topology: "EDGE2".to_string(),
                num_entries: 4,
                num_nodes_per_entry: 2,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let set = Set {
                id: 10,
                entity_type: EntityType::EdgeSet,
                num_entries: 2,
                num_dist_factors: 0,
            };
            file.put_set(&set).unwrap();
            file.put_entity_set(EntityType::EdgeSet, 10, &vec![2, 4])
                .unwrap();
            // Note: distribution factors not supported for entity sets in current API
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let edge_set = file.entity_set(EntityType::EdgeSet, 10).unwrap();

        assert_eq!(edge_set.entities.len(), 2);
        assert_eq!(edge_set.entities[0], 2);
    }

    // ========================================================================
    // Face Set Tests
    // ========================================================================

    #[test]
    fn test_face_set_basic() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "FaceSet Test".to_string(),
                num_dim: 3,
                num_nodes: 16,
                num_faces: 4,
                num_face_blocks: 1,
                num_face_sets: 1,
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

            let set = Set {
                id: 1,
                entity_type: EntityType::FaceSet,
                num_entries: 2,
                num_dist_factors: 0,
            };
            file.put_set(&set).unwrap();
            file.put_entity_set(EntityType::FaceSet, 1, &vec![1, 2])
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let face_set = file.entity_set(EntityType::FaceSet, 1).unwrap();

        assert_eq!(face_set.entities.len(), 2);
    }

    #[test]
    fn test_face_set_with_dist_factors() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "FaceSet with DF".to_string(),
                num_dim: 3,
                num_nodes: 12,
                num_faces: 3,
                num_face_blocks: 1,
                num_face_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::FaceBlock,
                topology: "TRI3".to_string(),
                num_entries: 3,
                num_nodes_per_entry: 3,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let set = Set {
                id: 20,
                entity_type: EntityType::FaceSet,
                num_entries: 3,
                num_dist_factors: 0,
            };
            file.put_set(&set).unwrap();
            file.put_entity_set(EntityType::FaceSet, 20, &vec![1, 2, 3])
                .unwrap();
            // Note: distribution factors not supported for entity sets in current API
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let face_set = file.entity_set(EntityType::FaceSet, 20).unwrap();

        assert_eq!(face_set.entities, vec![1, 2, 3]);
    }

    // ========================================================================
    // Empty Set Tests
    // ========================================================================

    #[test]
    fn test_empty_node_set() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Empty NodeSet".to_string(),
                num_dim: 3,
                num_nodes: 10,
                num_node_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let node_set = NodeSet {
                id: 1,
                nodes: vec![],
                dist_factors: vec![],
            };
            file.put_node_set(node_set.id, &node_set.nodes, None)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let node_set = file.node_set(1).unwrap();

        assert!(node_set.nodes.is_empty());
    }

    #[test]
    fn test_empty_side_set() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Empty SideSet".to_string(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                num_side_sets: 1,
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

            let side_set = SideSet {
                id: 1,
                elements: vec![],
                sides: vec![],
                dist_factors: vec![],
            };
            file.put_side_set(side_set.id, &side_set.elements, &side_set.sides, None)
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let side_set = file.side_set(1).unwrap();

        assert!(side_set.elements.is_empty());
        assert!(side_set.sides.is_empty());
    }

    #[test]
    fn test_empty_element_set() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Empty ElemSet".to_string(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                num_elem_sets: 1,
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

            let set = Set {
                id: 1,
                entity_type: EntityType::ElemSet,
                num_entries: 0,
                num_dist_factors: 0,
            };
            file.put_set(&set).unwrap();
            file.put_entity_set(EntityType::ElemSet, 1, &vec![])
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let elem_set = file.entity_set(EntityType::ElemSet, 1).unwrap();

        assert!(elem_set.entities.is_empty());
    }

    // ========================================================================
    // Set Iteration Tests
    // ========================================================================

    #[test]
    fn test_node_set_ids_retrieval() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "NodeSet IDs".to_string(),
                num_dim: 3,
                num_nodes: 20,
                num_node_sets: 4,
                ..Default::default()
            };
            file.init(&params).unwrap();

            for id in [10, 20, 30, 40] {
                let node_set = NodeSet {
                    id,
                    nodes: vec![1, 2],
                    dist_factors: vec![],
                };
                file.put_node_set(node_set.id, &node_set.nodes, None)
                    .unwrap();
            }
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let ids = file.set_ids(EntityType::NodeSet).unwrap();

        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&10));
        assert!(ids.contains(&40));
    }

    #[test]
    fn test_iterate_all_node_sets() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Iterate NodeSets".to_string(),
                num_dim: 3,
                num_nodes: 30,
                num_node_sets: 5,
                ..Default::default()
            };
            file.init(&params).unwrap();

            for i in 1..=5 {
                let node_set = NodeSet {
                    id: i,
                    nodes: vec![i, i + 10],
                    dist_factors: vec![],
                };
                file.put_node_set(node_set.id, &node_set.nodes, None)
                    .unwrap();
            }
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let ids = file.set_ids(EntityType::NodeSet).unwrap();

        let mut count = 0;
        for id in ids {
            let _ns = file.node_set(id).unwrap();
            count += 1;
        }

        assert_eq!(count, 5);
    }

    #[test]
    fn test_mixed_set_types() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Mixed Sets".to_string(),
                num_dim: 3,
                num_nodes: 30,
                num_elems: 10,
                num_elem_blocks: 1,
                num_node_sets: 2,
                num_side_sets: 2,
                num_elem_sets: 2,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 1,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".to_string(),
                num_entries: 10,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            // Node sets
            for i in 1..=2 {
                let ns = NodeSet {
                    id: i,
                    nodes: vec![i],
                    dist_factors: vec![],
                };
                file.put_node_set(ns.id, &ns.nodes, None).unwrap();
            }

            // Side sets
            for i in 1..=2 {
                let ss = SideSet {
                    id: i,
                    elements: vec![i],
                    sides: vec![1],
                    dist_factors: vec![],
                };
                file.put_side_set(ss.id, &ss.elements, &ss.sides, None)
                    .unwrap();
            }

            // Element sets
            for i in 1..=2 {
                let set = Set {
                    id: i,
                    entity_type: EntityType::ElemSet,
                    num_entries: 1,
                    num_dist_factors: 0,
                };
                file.put_set(&set).unwrap();
                file.put_entity_set(EntityType::ElemSet, i, &vec![i])
                    .unwrap();
            }
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();

        let ns_ids = file.set_ids(EntityType::NodeSet).unwrap();
        let ss_ids = file.set_ids(EntityType::SideSet).unwrap();
        let es_ids = file.set_ids(EntityType::ElemSet).unwrap();

        assert_eq!(ns_ids.len(), 2);
        assert_eq!(ss_ids.len(), 2);
        assert_eq!(es_ids.len(), 2);
    }

    // ========================================================================
    // Error Case Tests
    // ========================================================================

    #[test]
    fn test_side_set_mismatched_elem_side_length() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Mismatched SideSet".to_string(),
            num_dim: 3,
            num_nodes: 8,
            num_elems: 1,
            num_elem_blocks: 1,
            num_side_sets: 1,
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

        // Mismatched lengths
        let side_set = SideSet {
            id: 1,
            elements: vec![1, 1, 1],
            sides: vec![1, 2], // Wrong length!
            dist_factors: vec![],
        };

        let result = file.put_side_set(side_set.id, &side_set.elements, &side_set.sides, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_name() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();

        {
            let mut file = create_test_file(path).unwrap();
            let params = InitParams {
                title: "Named Set".to_string(),
                num_dim: 3,
                num_nodes: 10,
                num_node_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let node_set = NodeSet {
                id: 100,
                nodes: vec![1, 2, 3],
                dist_factors: vec![],
            };
            file.put_node_set(node_set.id, &node_set.nodes, None)
                .unwrap();

            file.put_name(EntityType::NodeSet, 0, "BoundaryNodes")
                .unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let name = file.name(EntityType::NodeSet, 0).unwrap();

        assert_eq!(name, "BoundaryNodes");
    }

    #[test]
    fn test_dist_factors_length_mismatch() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "DF Mismatch".to_string(),
            num_dim: 3,
            num_nodes: 10,
            num_node_sets: 1,
            ..Default::default()
        };
        file.init(&params).unwrap();

        // Distribution factors length doesn't match nodes length
        let node_set = NodeSet {
            id: 1,
            nodes: vec![1, 2, 3, 4, 5],
            dist_factors: vec![1.0, 2.0], // Wrong length!
        };

        let result = file.put_node_set(node_set.id, &node_set.nodes, Some(&node_set.dist_factors));
        assert!(result.is_err());
    }
}
