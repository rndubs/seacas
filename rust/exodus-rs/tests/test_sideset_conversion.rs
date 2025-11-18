//! Tests for nodeset to sideset conversion functionality

#[cfg(all(test, feature = "netcdf4"))]
mod tests {
    use exodus_rs::{
        mode, Block, CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, Set, Topology,
    };
    use tempfile::NamedTempFile;

    /// Create a simple test mesh with a single HEX8 element
    fn create_test_hex_mesh() -> NamedTempFile {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut file = ExodusFile::<mode::Write>::create(
            path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        // Initialize with 1 hex element, 8 nodes
        let params = InitParams {
            title: "Test Hex Mesh".to_string(),
            num_dim: 3,
            num_nodes: 8,
            num_elems: 1,
            num_elem_blocks: 1,
            num_node_sets: 1,
            num_side_sets: 1, // Reserve space for 1 sideset
            ..Default::default()
        };
        file.init(&params).unwrap();

        // Write coordinates for a unit cube
        let x = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
        let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
        let z = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
        file.put_coords(&x, Some(&y), Some(&z)).unwrap();

        // Write element block
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

        // Write connectivity (1-based node IDs)
        let conn = vec![1, 2, 3, 4, 5, 6, 7, 8];
        file.put_connectivity(1, &conn).unwrap();

        // Write node set for top face (nodes 5,6,7,8)
        let ns = Set {
            id: 10,
            entity_type: EntityType::NodeSet,
            num_entries: 4,
            num_dist_factors: 0,
        };
        file.put_set(&ns).unwrap();
        file.put_node_set(10, &[5, 6, 7, 8], None).unwrap();

        file.sync().unwrap();
        drop(file);

        temp_file
    }

    #[test]
    fn test_convert_nodeset_to_sideset_hex() {
        let temp_file = create_test_hex_mesh();
        let path = temp_file.path();

        // Open for reading
        let file = ExodusFile::<mode::Read>::open(path).unwrap();

        // Convert nodeset 10 (top face) to sideset 100
        let sideset = file.convert_nodeset_to_sideset(10, 100).unwrap();

        // Should have exactly 1 face (the top face)
        assert_eq!(sideset.elements.len(), 1);
        assert_eq!(sideset.sides.len(), 1);

        // Element should be element 1
        assert_eq!(sideset.elements[0], 1);

        // Side should be side 6 (top face of HEX8)
        assert_eq!(sideset.sides[0], 6);

        // ID should be set correctly
        assert_eq!(sideset.id, 100);
    }

    #[test]
    fn test_create_sideset_from_nodeset_append() {
        let temp_file = create_test_hex_mesh();
        let path = temp_file.path();

        // Open for appending
        let mut file = ExodusFile::<mode::Append>::append(path).unwrap();

        // Convert and write sideset
        file.create_sideset_from_nodeset(10, 100).unwrap();
        file.sync().unwrap();
        drop(file);

        // Verify it was written
        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let sideset = file.side_set(100).unwrap();

        assert_eq!(sideset.elements.len(), 1);
        assert_eq!(sideset.elements[0], 1);
        assert_eq!(sideset.sides[0], 6);
    }

    #[test]
    fn test_empty_nodeset_warning() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut file = ExodusFile::<mode::Write>::create(
            path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        let params = InitParams {
            title: "Empty NodeSet Test".to_string(),
            num_dim: 3,
            num_nodes: 8,
            num_elems: 1,
            num_elem_blocks: 1,
            num_node_sets: 1,
            num_side_sets: 0,
            ..Default::default()
        };
        file.init(&params).unwrap();

        let x = vec![0.0; 8];
        file.put_coords(&x, Some(&x), Some(&x)).unwrap();

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
        file.put_connectivity(1, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

        // Empty node set
        let ns = Set {
            id: 10,
            entity_type: EntityType::NodeSet,
            num_entries: 0,
            num_dist_factors: 0,
        };
        file.put_set(&ns).unwrap();
        file.put_node_set(10, &[], None).unwrap();
        file.sync().unwrap();
        drop(file);

        // Convert empty nodeset
        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let sideset = file.convert_nodeset_to_sideset(10, 100).unwrap();

        // Should be empty
        assert_eq!(sideset.elements.len(), 0);
        assert_eq!(sideset.sides.len(), 0);
    }

    #[test]
    fn test_topology_face_definitions() {
        // Test HEX8
        let hex = Topology::Hex8;
        let faces = hex.faces().unwrap();
        assert_eq!(faces.len(), 6);
        assert_eq!(faces[0].side_number, 1);
        assert_eq!(faces[0].node_indices, vec![0, 1, 5, 4]);

        // Test TET4
        let tet = Topology::Tet4;
        let faces = tet.faces().unwrap();
        assert_eq!(faces.len(), 4);

        // Test WEDGE6
        let wedge = Topology::Wedge6;
        let faces = wedge.faces().unwrap();
        assert_eq!(faces.len(), 5);

        // Test PYRAMID5
        let pyramid = Topology::Pyramid5;
        let faces = pyramid.faces().unwrap();
        assert_eq!(faces.len(), 5);
    }

    #[test]
    fn test_geometry_face_normal() {
        use exodus_rs::geometry::*;

        // Square in xy-plane
        let coords = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];

        let normal = compute_face_normal(&coords);
        // Should point in +z direction
        assert!((normal[0] - 0.0).abs() < 1e-10);
        assert!((normal[1] - 0.0).abs() < 1e-10);
        assert!((normal[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_geometry_outward_facing() {
        use exodus_rs::geometry::*;

        let mesh_center = [0.0, 0.0, 0.0];
        let face_center = [1.0, 0.0, 0.0];
        let face_normal = [1.0, 0.0, 0.0]; // Points outward

        assert!(is_outward_facing(face_center, face_normal, mesh_center));

        let inward_normal = [-1.0, 0.0, 0.0];
        assert!(!is_outward_facing(
            face_center,
            inward_normal,
            mesh_center
        ));
    }
}
