//! Comprehensive tests for Phase 7: Maps and Names
//!
//! Tests for:
//! - Entity ID maps
//! - Element order maps
//! - Entity naming
//! - Property arrays

#[cfg(feature = "netcdf4")]
use exodus_rs::*;
#[cfg(feature = "netcdf4")]
use tempfile::NamedTempFile;

// ============================================================================
// ID Map Tests
// ============================================================================

#[test]
#[cfg(feature = "netcdf4")]
fn test_node_id_map_roundtrip() {
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

        let params = InitParams {
            title: "Node ID Map Test".into(),
            num_dim: 3,
            num_nodes: 4,
            ..Default::default()
        };
        file.init(&params).unwrap();

        // Custom node numbering starting from 100
        let node_map = vec![100, 101, 102, 103];
        file.put_id_map(EntityType::NodeMap, &node_map).unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let node_map = file.id_map(EntityType::NodeMap).unwrap();
        assert_eq!(node_map, vec![100, 101, 102, 103]);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_elem_id_map_roundtrip() {
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

        let params = InitParams {
            title: "Element ID Map Test".into(),
            num_dim: 3,
            num_elems: 5,
            num_elem_blocks: 1,
            ..Default::default()
        };
        file.init(&params).unwrap();

        // Custom element numbering
        let elem_map = vec![1000, 1001, 1002, 1003, 1004];
        file.put_id_map(EntityType::ElemMap, &elem_map).unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let elem_map = file.id_map(EntityType::ElemMap).unwrap();
        assert_eq!(elem_map, vec![1000, 1001, 1002, 1003, 1004]);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_edge_id_map_roundtrip() {
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

        let params = InitParams {
            title: "Edge ID Map Test".into(),
            num_dim: 3,
            num_edges: 3,
            ..Default::default()
        };
        file.init(&params).unwrap();

        let edge_map = vec![200, 201, 202];
        file.put_id_map(EntityType::EdgeMap, &edge_map).unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let edge_map = file.id_map(EntityType::EdgeMap).unwrap();
        assert_eq!(edge_map, vec![200, 201, 202]);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_face_id_map_roundtrip() {
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

        let params = InitParams {
            title: "Face ID Map Test".into(),
            num_dim: 3,
            num_faces: 3,
            ..Default::default()
        };
        file.init(&params).unwrap();

        let face_map = vec![300, 301, 302];
        file.put_id_map(EntityType::FaceMap, &face_map).unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let face_map = file.id_map(EntityType::FaceMap).unwrap();
        assert_eq!(face_map, vec![300, 301, 302]);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_id_map_invalid_length() {
    let tmp = NamedTempFile::new().unwrap();

    let mut file = ExodusFile::create(
        tmp.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
    .unwrap();

    let params = InitParams {
        title: "Invalid Length Test".into(),
        num_dim: 3,
        num_nodes: 4,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Try to write map with wrong length
    let node_map = vec![100, 101, 102]; // Only 3 instead of 4
    let result = file.put_id_map(EntityType::NodeMap, &node_map);
    assert!(result.is_err());
}

// ============================================================================
// Element Order Map Tests
// ============================================================================

#[test]
#[cfg(feature = "netcdf4")]
fn test_elem_order_map_roundtrip() {
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

        let params = InitParams {
            title: "Order Map Test".into(),
            num_dim: 3,
            num_elems: 4,
            num_elem_blocks: 1,
            ..Default::default()
        };
        file.init(&params).unwrap();

        // Reverse element ordering
        let order = vec![4, 3, 2, 1];
        file.put_elem_order_map(&order).unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let order = file.elem_order_map().unwrap();
        assert_eq!(order, vec![4, 3, 2, 1]);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_elem_order_map_custom() {
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

        let params = InitParams {
            title: "Custom Order Test".into(),
            num_dim: 3,
            num_elems: 6,
            num_elem_blocks: 1,
            ..Default::default()
        };
        file.init(&params).unwrap();

        // Custom ordering: process elements 3, 1, 5, 2, 6, 4
        let order = vec![3, 1, 5, 2, 6, 4];
        file.put_elem_order_map(&order).unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let order = file.elem_order_map().unwrap();
        assert_eq!(order, vec![3, 1, 5, 2, 6, 4]);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_elem_order_map_invalid_length() {
    let tmp = NamedTempFile::new().unwrap();

    let mut file = ExodusFile::create(
        tmp.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
    .unwrap();

    let params = InitParams {
        title: "Invalid Order Length Test".into(),
        num_dim: 3,
        num_elems: 4,
        num_elem_blocks: 1,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Try to write order map with wrong length
    let order = vec![1, 2, 3]; // Only 3 instead of 4
    let result = file.put_elem_order_map(&order);
    assert!(result.is_err());
}

// ============================================================================
// Entity Naming Tests
// ============================================================================

#[test]
#[cfg(feature = "netcdf4")]
fn test_block_names_roundtrip() {
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

        let params = InitParams {
            title: "Block Names Test".into(),
            num_dim: 3,
            num_elem_blocks: 3,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_names(EntityType::ElemBlock, &["Block1", "Block2", "Block3"])
            .unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let names = file.names(EntityType::ElemBlock).unwrap();
        assert_eq!(names, vec!["Block1", "Block2", "Block3"]);

        let name = file.name(EntityType::ElemBlock, 0).unwrap();
        assert_eq!(name, "Block1");

        let name = file.name(EntityType::ElemBlock, 2).unwrap();
        assert_eq!(name, "Block3");
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_single_name_update() {
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

        let params = InitParams {
            title: "Single Name Update Test".into(),
            num_dim: 3,
            num_elem_blocks: 3,
            ..Default::default()
        };
        file.init(&params).unwrap();

        // Set all names first
        file.put_names(EntityType::ElemBlock, &["A", "B", "C"])
            .unwrap();

        // Update just the second one
        file.put_name(EntityType::ElemBlock, 1, "Updated").unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let names = file.names(EntityType::ElemBlock).unwrap();
        assert_eq!(names, vec!["A", "Updated", "C"]);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_nodeset_names() {
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

        let params = InitParams {
            title: "NodeSet Names Test".into(),
            num_dim: 3,
            num_node_sets: 2,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_names(EntityType::NodeSet, &["Inlet", "Outlet"])
            .unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let names = file.names(EntityType::NodeSet).unwrap();
        assert_eq!(names, vec!["Inlet", "Outlet"]);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_sideset_names() {
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

        let params = InitParams {
            title: "SideSet Names Test".into(),
            num_dim: 3,
            num_side_sets: 2,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_names(EntityType::SideSet, &["Wall", "Boundary"])
            .unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let names = file.names(EntityType::SideSet).unwrap();
        assert_eq!(names, vec!["Wall", "Boundary"]);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_name_too_long() {
    let tmp = NamedTempFile::new().unwrap();

    let mut file = ExodusFile::create(
        tmp.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
    .unwrap();

    let params = InitParams {
        title: "Long Name Test".into(),
        num_dim: 3,
        num_elem_blocks: 1,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Try to set a name that's too long (max is 32 characters)
    let long_name = "This_is_a_very_long_name_that_exceeds_the_limit";
    let result = file.put_names(EntityType::ElemBlock, &[long_name]);
    assert!(result.is_err());
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_names_invalid_length() {
    let tmp = NamedTempFile::new().unwrap();

    let mut file = ExodusFile::create(
        tmp.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
    .unwrap();

    let params = InitParams {
        title: "Invalid Names Length Test".into(),
        num_dim: 3,
        num_elem_blocks: 3,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Try to write names with wrong count
    let result = file.put_names(EntityType::ElemBlock, &["A", "B"]); // Only 2 instead of 3
    assert!(result.is_err());
}

// ============================================================================
// Property Array Tests
// ============================================================================

#[test]
#[cfg(feature = "netcdf4")]
fn test_property_array_roundtrip() {
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

        let params = InitParams {
            title: "Property Test".into(),
            num_dim: 3,
            num_elem_blocks: 3,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_property_array(EntityType::ElemBlock, "MATERIAL_ID", &[1, 2, 3])
            .unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let props = file
            .property_array(EntityType::ElemBlock, "MATERIAL_ID")
            .unwrap();
        assert_eq!(props, vec![1, 2, 3]);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_multiple_properties() {
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

        let params = InitParams {
            title: "Multiple Properties Test".into(),
            num_dim: 3,
            num_elem_blocks: 3,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_property_array(EntityType::ElemBlock, "MATERIAL_ID", &[1, 2, 3])
            .unwrap();
        file.put_property_array(EntityType::ElemBlock, "PROCESSOR_ID", &[0, 1, 0])
            .unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

        let material_ids = file
            .property_array(EntityType::ElemBlock, "MATERIAL_ID")
            .unwrap();
        assert_eq!(material_ids, vec![1, 2, 3]);

        let processor_ids = file
            .property_array(EntityType::ElemBlock, "PROCESSOR_ID")
            .unwrap();
        assert_eq!(processor_ids, vec![0, 1, 0]);

        let prop_names = file.property_names(EntityType::ElemBlock).unwrap();
        assert!(prop_names.contains(&"material_id".to_string()));
        assert!(prop_names.contains(&"processor_id".to_string()));
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_nodeset_properties() {
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

        let params = InitParams {
            title: "NodeSet Properties Test".into(),
            num_dim: 3,
            num_node_sets: 2,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_property_array(EntityType::NodeSet, "BC_TYPE", &[1, 2])
            .unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let props = file.property_array(EntityType::NodeSet, "BC_TYPE").unwrap();
        assert_eq!(props, vec![1, 2]);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_property_names() {
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

        let params = InitParams {
            title: "Property Names Test".into(),
            num_dim: 3,
            num_elem_blocks: 2,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_property_array(EntityType::ElemBlock, "MATERIAL_ID", &[1, 2])
            .unwrap();
        file.put_property_array(EntityType::ElemBlock, "DENSITY", &[100, 200])
            .unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let prop_names = file.property_names(EntityType::ElemBlock).unwrap();

        assert_eq!(prop_names.len(), 2);
        assert!(prop_names.contains(&"material_id".to_string()));
        assert!(prop_names.contains(&"density".to_string()));
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_property_invalid_length() {
    let tmp = NamedTempFile::new().unwrap();

    let mut file = ExodusFile::create(
        tmp.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
    .unwrap();

    let params = InitParams {
        title: "Invalid Property Length Test".into(),
        num_dim: 3,
        num_elem_blocks: 3,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Try to write properties with wrong length
    let result = file.put_property_array(EntityType::ElemBlock, "MATERIAL_ID", &[1, 2]); // Only 2 instead of 3
    assert!(result.is_err());
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
#[cfg(feature = "netcdf4")]
fn test_all_maps_and_names_together() {
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

        let params = InitParams {
            title: "Complete Maps & Names Test".into(),
            num_dim: 3,
            num_nodes: 4,
            num_elems: 2,
            num_elem_blocks: 2,
            num_node_sets: 2,
            ..Default::default()
        };
        file.init(&params).unwrap();

        // Set ID maps
        file.put_id_map(EntityType::NodeMap, &[100, 101, 102, 103])
            .unwrap();
        file.put_id_map(EntityType::ElemMap, &[1000, 1001]).unwrap();

        // Set order map
        file.put_elem_order_map(&[2, 1]).unwrap();

        // Set names
        file.put_names(EntityType::ElemBlock, &["Material1", "Material2"])
            .unwrap();
        file.put_names(EntityType::NodeSet, &["Inlet", "Outlet"])
            .unwrap();

        // Set properties
        file.put_property_array(EntityType::ElemBlock, "MATERIAL_ID", &[1, 2])
            .unwrap();
        file.put_property_array(EntityType::NodeSet, "BC_TYPE", &[10, 20])
            .unwrap();
    }

    // Read and verify
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

        // Verify ID maps
        let node_map = file.id_map(EntityType::NodeMap).unwrap();
        assert_eq!(node_map, vec![100, 101, 102, 103]);

        let elem_map = file.id_map(EntityType::ElemMap).unwrap();
        assert_eq!(elem_map, vec![1000, 1001]);

        // Verify order map
        let order = file.elem_order_map().unwrap();
        assert_eq!(order, vec![2, 1]);

        // Verify names
        let block_names = file.names(EntityType::ElemBlock).unwrap();
        assert_eq!(block_names, vec!["Material1", "Material2"]);

        let nodeset_names = file.names(EntityType::NodeSet).unwrap();
        assert_eq!(nodeset_names, vec!["Inlet", "Outlet"]);

        // Verify properties
        let mat_ids = file
            .property_array(EntityType::ElemBlock, "MATERIAL_ID")
            .unwrap();
        assert_eq!(mat_ids, vec![1, 2]);

        let bc_types = file.property_array(EntityType::NodeSet, "BC_TYPE").unwrap();
        assert_eq!(bc_types, vec![10, 20]);
    }
}
