//! Comprehensive tests for Phase 6: Variables and Time Steps (All Types)

use exodus_rs::{mode, Block, CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, Set};
use tempfile::NamedTempFile;

#[test]
fn test_edge_block_variables() {
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
            title: "Edge Block Vars".into(),
            num_dim: 2,
            num_nodes: 4,
            num_edges: 2,
            num_edge_blocks: 1,
            ..Default::default()
        })
        .unwrap();

        let block = Block {
            id: 100,
            entity_type: EntityType::EdgeBlock,
            topology: "EDGE2".into(),
            num_entries: 2,
            num_nodes_per_entry: 2,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };
        file.put_block(&block).unwrap();

        file.define_variables(EntityType::EdgeBlock, &["EdgeTemp"])
            .unwrap();

        file.put_time(0, 0.0).unwrap();
        file.put_var(0, EntityType::EdgeBlock, 100, 0, &[25.0, 30.0])
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let vars = file.variable_names(EntityType::EdgeBlock).unwrap();
        assert_eq!(vars, vec!["EdgeTemp"]);

        let values = file.var(0, EntityType::EdgeBlock, 100, 0).unwrap();
        assert_eq!(values, vec![25.0, 30.0]);
    }
}

#[test]
fn test_face_block_variables() {
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
            title: "Face Block Vars".into(),
            num_dim: 3,
            num_nodes: 8,
            num_faces: 2,
            num_face_blocks: 1,
            ..Default::default()
        })
        .unwrap();

        let block = Block {
            id: 200,
            entity_type: EntityType::FaceBlock,
            topology: "QUAD4".into(),
            num_entries: 2,
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };
        file.put_block(&block).unwrap();

        file.define_variables(EntityType::FaceBlock, &["FaceStress"])
            .unwrap();

        file.put_time(0, 0.0).unwrap();
        file.put_var(0, EntityType::FaceBlock, 200, 0, &[100.0, 150.0])
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let vars = file.variable_names(EntityType::FaceBlock).unwrap();
        assert_eq!(vars, vec!["FaceStress"]);

        let values = file.var(0, EntityType::FaceBlock, 200, 0).unwrap();
        assert_eq!(values, vec![100.0, 150.0]);
    }
}

#[test]
fn test_node_set_variables() {
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
            title: "Node Set Vars".into(),
            num_dim: 2,
            num_nodes: 10,
            num_node_sets: 1,
            ..Default::default()
        })
        .unwrap();

        let set = Set {
            id: 10,
            entity_type: EntityType::NodeSet,
            num_entries: 3,
            num_dist_factors: 0,
        };
        file.put_set(&set).unwrap();
        file.put_node_set(10, &[1, 5, 9], None).unwrap();

        file.define_variables(EntityType::NodeSet, &["BCValue"])
            .unwrap();

        file.put_time(0, 0.0).unwrap();
        file.put_var(0, EntityType::NodeSet, 10, 0, &[5.0, 10.0, 15.0])
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let vars = file.variable_names(EntityType::NodeSet).unwrap();
        assert_eq!(vars, vec!["BCValue"]);

        let values = file.var(0, EntityType::NodeSet, 10, 0).unwrap();
        assert_eq!(values, vec![5.0, 10.0, 15.0]);
    }
}

#[test]
fn test_side_set_variables() {
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
            title: "Side Set Vars".into(),
            num_dim: 2,
            num_nodes: 4,
            num_elems: 1,
            num_elem_blocks: 1,
            num_side_sets: 1,
            ..Default::default()
        })
        .unwrap();

        // Need an element block first
        let block = Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "QUAD4".into(),
            num_entries: 1,
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };
        file.put_block(&block).unwrap();

        let set = Set {
            id: 20,
            entity_type: EntityType::SideSet,
            num_entries: 2,
            num_dist_factors: 0,
        };
        file.put_set(&set).unwrap();

        file.put_side_set(20, &[1, 1], &[1, 2], None).unwrap();

        file.define_variables(EntityType::SideSet, &["Pressure"])
            .unwrap();

        file.put_time(0, 0.0).unwrap();
        file.put_var(0, EntityType::SideSet, 20, 0, &[101.3, 102.5])
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let vars = file.variable_names(EntityType::SideSet).unwrap();
        assert_eq!(vars, vec!["Pressure"]);

        let values = file.var(0, EntityType::SideSet, 20, 0).unwrap();
        assert_eq!(values, vec![101.3, 102.5]);
    }
}

#[test]
fn test_elem_set_variables() {
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
            title: "Elem Set Vars".into(),
            num_dim: 2,
            num_nodes: 4,
            num_elems: 2,
            num_elem_blocks: 1,
            num_elem_sets: 1,
            ..Default::default()
        })
        .unwrap();

        let block = Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "TRI3".into(),
            num_entries: 2,
            num_nodes_per_entry: 3,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };
        file.put_block(&block).unwrap();

        let set = Set {
            id: 30,
            entity_type: EntityType::ElemSet,
            num_entries: 2,
            num_dist_factors: 0,
        };
        file.put_set(&set).unwrap();

        file.put_entity_set(EntityType::ElemSet, 30, &[1, 2])
            .unwrap();

        file.define_variables(EntityType::ElemSet, &["Quality"])
            .unwrap();

        file.put_time(0, 0.0).unwrap();
        file.put_var(0, EntityType::ElemSet, 30, 0, &[0.9, 0.95])
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let vars = file.variable_names(EntityType::ElemSet).unwrap();
        assert_eq!(vars, vec!["Quality"]);

        let values = file.var(0, EntityType::ElemSet, 30, 0).unwrap();
        assert_eq!(values, vec![0.9, 0.95]);
    }
}

#[test]
fn test_multiple_time_steps_all_types() {
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
            title: "Multi-Step All Types".into(),
            num_dim: 2,
            num_nodes: 4,
            num_elems: 1,
            num_elem_blocks: 1,
            ..Default::default()
        })
        .unwrap();

        let block = Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "QUAD4".into(),
            num_entries: 1,
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };
        file.put_block(&block).unwrap();

        file.define_variables(EntityType::Global, &["Time"])
            .unwrap();
        file.define_variables(EntityType::Nodal, &["Temp"]).unwrap();
        file.define_variables(EntityType::ElemBlock, &["Stress"])
            .unwrap();

        // Time step 0
        file.put_time(0, 0.0).unwrap();
        file.put_var(0, EntityType::Global, 0, 0, &[0.0]).unwrap();
        file.put_var(0, EntityType::Nodal, 0, 0, &[20.0, 20.0, 20.0, 20.0])
            .unwrap();
        file.put_var(0, EntityType::ElemBlock, 1, 0, &[100.0])
            .unwrap();

        // Time step 1
        file.put_time(1, 1.0).unwrap();
        file.put_var(1, EntityType::Global, 0, 0, &[1.0]).unwrap();
        file.put_var(1, EntityType::Nodal, 0, 0, &[25.0, 25.0, 25.0, 25.0])
            .unwrap();
        file.put_var(1, EntityType::ElemBlock, 1, 0, &[150.0])
            .unwrap();

        // Time step 2
        file.put_time(2, 2.0).unwrap();
        file.put_var(2, EntityType::Global, 0, 0, &[2.0]).unwrap();
        file.put_var(2, EntityType::Nodal, 0, 0, &[30.0, 30.0, 30.0, 30.0])
            .unwrap();
        file.put_var(2, EntityType::ElemBlock, 1, 0, &[200.0])
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

        assert_eq!(file.num_time_steps().unwrap(), 3);

        let times = file.times().unwrap();
        assert_eq!(times, vec![0.0, 1.0, 2.0]);

        // Check global variable progression
        for step in 0..3 {
            let val = file.var(step, EntityType::Global, 0, 0).unwrap();
            assert_eq!(val[0], step as f64);
        }

        // Check nodal variable progression
        let temp0 = file.var(0, EntityType::Nodal, 0, 0).unwrap();
        assert_eq!(temp0, vec![20.0; 4]);

        let temp2 = file.var(2, EntityType::Nodal, 0, 0).unwrap();
        assert_eq!(temp2, vec![30.0; 4]);

        // Check element variable progression
        let stress1 = file.var(1, EntityType::ElemBlock, 1, 0).unwrap();
        assert_eq!(stress1, vec![150.0]);
    }
}

#[test]
fn test_var_time_series_all_types() {
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
            title: "Time Series Test".into(),
            num_dim: 2,
            num_nodes: 2,
            num_node_sets: 1,
            ..Default::default()
        })
        .unwrap();

        let set = Set {
            id: 5,
            entity_type: EntityType::NodeSet,
            num_entries: 2,
            num_dist_factors: 0,
        };
        file.put_set(&set).unwrap();
        file.put_node_set(5, &[1, 2], None).unwrap();

        file.define_variables(EntityType::NodeSet, &["Load"])
            .unwrap();

        // Write time series data
        for step in 0..5 {
            file.put_time(step, step as f64 * 0.1).unwrap();
            file.put_var(
                step,
                EntityType::NodeSet,
                5,
                0,
                &[step as f64 * 10.0, step as f64 * 20.0],
            )
            .unwrap();
        }
    }

    // Read time series
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

        let series = file
            .var_time_series(0, 5, EntityType::NodeSet, 5, 0)
            .unwrap();

        // Should be 5 time steps * 2 nodes = 10 values
        assert_eq!(series.len(), 10);

        // Check first time step values
        assert_eq!(series[0], 0.0);
        assert_eq!(series[1], 0.0);

        // Check last time step values
        assert_eq!(series[8], 40.0);
        assert_eq!(series[9], 80.0);
    }
}

#[test]
fn test_multiple_variables_per_entity() {
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
            title: "Multi-Var Test".into(),
            num_dim: 2,
            num_nodes: 4,
            ..Default::default()
        })
        .unwrap();

        file.define_variables(EntityType::Nodal, &["Temp", "Pressure", "Velocity"])
            .unwrap();

        file.put_time(0, 0.0).unwrap();

        file.put_var(0, EntityType::Nodal, 0, 0, &[100.0, 200.0, 300.0, 400.0])
            .unwrap();
        file.put_var(0, EntityType::Nodal, 0, 1, &[1.0, 2.0, 3.0, 4.0])
            .unwrap();
        file.put_var(0, EntityType::Nodal, 0, 2, &[10.0, 20.0, 30.0, 40.0])
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

        let vars = file.variable_names(EntityType::Nodal).unwrap();
        assert_eq!(vars.len(), 3);
        assert_eq!(vars, vec!["Temp", "Pressure", "Velocity"]);

        let temp = file.var(0, EntityType::Nodal, 0, 0).unwrap();
        assert_eq!(temp, vec![100.0, 200.0, 300.0, 400.0]);

        let pressure = file.var(0, EntityType::Nodal, 0, 1).unwrap();
        assert_eq!(pressure, vec![1.0, 2.0, 3.0, 4.0]);

        let velocity = file.var(0, EntityType::Nodal, 0, 2).unwrap();
        assert_eq!(velocity, vec![10.0, 20.0, 30.0, 40.0]);
    }
}

#[test]
fn test_empty_variables() {
    let tmp = NamedTempFile::new().unwrap();

    // Write file with no variables
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
            title: "No Variables".into(),
            num_dim: 2,
            num_nodes: 4,
            ..Default::default()
        })
        .unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

        let global_vars = file.variable_names(EntityType::Global).unwrap();
        assert_eq!(global_vars.len(), 0);

        let nodal_vars = file.variable_names(EntityType::Nodal).unwrap();
        assert_eq!(nodal_vars.len(), 0);

        let elem_vars = file.variable_names(EntityType::ElemBlock).unwrap();
        assert_eq!(elem_vars.len(), 0);
    }
}

#[test]
fn test_variable_name_length_limits() {
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
        title: "Long Names".into(),
        num_dim: 2,
        num_nodes: 1,
        ..Default::default()
    })
    .unwrap();

    // Very long variable name (will be truncated to 32 chars)
    let long_name = "ThisIsAVeryLongVariableNameThatExceedsThirtyTwoCharacters";

    file.define_variables(EntityType::Nodal, &[long_name])
        .unwrap();

    drop(file);

    // Read back
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let vars = file.variable_names(EntityType::Nodal).unwrap();

    // Should be truncated to 32 characters
    assert!(vars[0].len() <= 32);
}

#[test]
fn test_invalid_time_step_access() {
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
            title: "Invalid Step Test".into(),
            num_dim: 2,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();

        file.define_variables(EntityType::Nodal, &["Var1"]).unwrap();

        file.put_time(0, 0.0).unwrap();
        file.put_var(0, EntityType::Nodal, 0, 0, &[1.0]).unwrap();
    }

    // Read
    {
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

        // Try to access time step that doesn't exist
        let result = file.time(5);
        assert!(result.is_err(), "Should fail accessing invalid time step");
    }
}
