//! Tests for reduction variables functionality
//!
//! Reduction variables store aggregated/summary values for entire objects
//! (e.g., assemblies, blocks, sets) rather than for individual entities within those objects.

use approx::assert_abs_diff_eq;
use exodus_rs::types::{Assembly, Block, EntityType};
use exodus_rs::{CreateMode, CreateOptions, ExodusFile, InitParams, Set};
use tempfile::NamedTempFile;

#[test]
#[cfg(feature = "netcdf4")]
fn test_assembly_reduction_variables() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path();

    // Write reduction variables
    {
        let mut file = ExodusFile::create(
            file_path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        // Initialize with assemblies
        let params = InitParams {
            title: "Assembly Reduction Test".to_string(),
            num_dim: 3,
            num_nodes: 3,
            num_elems: 1,
            num_elem_blocks: 1,
            num_assemblies: 2,
            ..Default::default()
        };
        file.init(&params).unwrap();

        // Put coords (required)
        file.put_coords(
            &[0.0, 1.0, 2.0],
            Some(&[0.0, 0.0, 0.0]),
            Some(&[0.0, 0.0, 0.0]),
        )
        .unwrap();

        // Put element block (required)
        let block = Block {
            id: 10,
            entity_type: EntityType::ElemBlock,
            topology: "tri".to_string(),
            num_entries: 1,
            num_nodes_per_entry: 3,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };
        file.put_block(&block).unwrap();
        file.put_connectivity(10, &[1, 2, 3]).unwrap();

        // Put assemblies
        let assembly1 = Assembly {
            id: 100,
            name: "Assembly_A".to_string(),
            entity_type: EntityType::ElemBlock,
            entity_list: vec![10],
        };
        file.put_assembly(&assembly1).unwrap();

        let assembly2 = Assembly {
            id: 200,
            name: "Assembly_B".to_string(),
            entity_type: EntityType::ElemBlock,
            entity_list: vec![10],
        };
        file.put_assembly(&assembly2).unwrap();

        // Define reduction variables
        file.define_reduction_variables(
            EntityType::Assembly,
            &["Momentum_X", "Momentum_Y", "Momentum_Z", "Kinetic_Energy"],
        )
        .unwrap();

        // Write time steps with reduction variables
        for ts in 0..3 {
            let time_val = (ts + 1) as f64 * 0.1;
            file.put_time(ts, time_val).unwrap();

            // Assembly 100
            let values_100 = [
                time_val * 1.0,  // Momentum_X
                time_val * 2.0,  // Momentum_Y
                time_val * 3.0,  // Momentum_Z
                time_val * 10.0, // Kinetic_Energy
            ];
            file.put_reduction_vars(ts, EntityType::Assembly, 100, &values_100)
                .unwrap();

            // Assembly 200
            let values_200 = [
                time_val * 4.0,  // Momentum_X
                time_val * 5.0,  // Momentum_Y
                time_val * 6.0,  // Momentum_Z
                time_val * 20.0, // Kinetic_Energy
            ];
            file.put_reduction_vars(ts, EntityType::Assembly, 200, &values_200)
                .unwrap();
        }
    }

    // Read reduction variables
    {
        let file = ExodusFile::open(file_path).unwrap();

        // Read variable names
        let names = file.reduction_variable_names(EntityType::Assembly).unwrap();
        assert_eq!(names.len(), 4);
        assert_eq!(names[0], "Momentum_X");
        assert_eq!(names[1], "Momentum_Y");
        assert_eq!(names[2], "Momentum_Z");
        assert_eq!(names[3], "Kinetic_Energy");

        // Read values for assembly 100 at time step 0
        let values = file
            .get_reduction_vars(0, EntityType::Assembly, 100)
            .unwrap();
        assert_eq!(values.len(), 4);
        assert_abs_diff_eq!(values[0], 0.1, epsilon = 1e-10);
        assert_abs_diff_eq!(values[1], 0.2, epsilon = 1e-10);
        assert_abs_diff_eq!(values[2], 0.3, epsilon = 1e-10);
        assert_abs_diff_eq!(values[3], 1.0, epsilon = 1e-10);

        // Read values for assembly 200 at time step 2
        let values = file
            .get_reduction_vars(2, EntityType::Assembly, 200)
            .unwrap();
        assert_eq!(values.len(), 4);
        assert_abs_diff_eq!(values[0], 1.2, epsilon = 1e-10);
        assert_abs_diff_eq!(values[1], 1.5, epsilon = 1e-10);
        assert_abs_diff_eq!(values[2], 1.8, epsilon = 1e-10);
        assert_abs_diff_eq!(values[3], 6.0, epsilon = 1e-10);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_element_block_reduction_variables() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path();

    // Write
    {
        let mut file = ExodusFile::create(
            file_path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        let params = InitParams {
            title: "Element Block Reduction Test".to_string(),
            num_dim: 3,
            num_nodes: 3,
            num_elems: 6,
            num_elem_blocks: 2,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_coords(
            &[0.0, 1.0, 2.0],
            Some(&[0.0, 0.0, 0.0]),
            Some(&[0.0, 0.0, 0.0]),
        )
        .unwrap();

        // Block 1: 2 triangles
        let block1 = Block {
            id: 10,
            entity_type: EntityType::ElemBlock,
            topology: "tri".to_string(),
            num_entries: 2,
            num_nodes_per_entry: 3,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };
        file.put_block(&block1).unwrap();
        file.put_connectivity(10, &[1, 2, 3, 2, 3, 1]).unwrap();

        // Block 2: 4 triangles
        let block2 = Block {
            id: 20,
            entity_type: EntityType::ElemBlock,
            topology: "tri".to_string(),
            num_entries: 4,
            num_nodes_per_entry: 3,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };
        file.put_block(&block2).unwrap();
        file.put_connectivity(20, &[1, 2, 3, 2, 3, 1, 1, 2, 3, 2, 3, 1])
            .unwrap();

        // Define reduction variables for element blocks
        file.define_reduction_variables(EntityType::ElemBlock, &["AvgStrain", "MaxStress"])
            .unwrap();

        // Write time step 0
        file.put_time(0, 0.0).unwrap();
        file.put_reduction_vars(0, EntityType::ElemBlock, 10, &[0.01, 100.0])
            .unwrap();
        file.put_reduction_vars(0, EntityType::ElemBlock, 20, &[0.02, 200.0])
            .unwrap();
    }

    // Read
    {
        let file = ExodusFile::open(file_path).unwrap();

        let names = file
            .reduction_variable_names(EntityType::ElemBlock)
            .unwrap();
        assert_eq!(names, vec!["AvgStrain", "MaxStress"]);

        let values_10 = file
            .get_reduction_vars(0, EntityType::ElemBlock, 10)
            .unwrap();
        assert_abs_diff_eq!(values_10[0], 0.01, epsilon = 1e-10);
        assert_abs_diff_eq!(values_10[1], 100.0, epsilon = 1e-10);

        let values_20 = file
            .get_reduction_vars(0, EntityType::ElemBlock, 20)
            .unwrap();
        assert_abs_diff_eq!(values_20[0], 0.02, epsilon = 1e-10);
        assert_abs_diff_eq!(values_20[1], 200.0, epsilon = 1e-10);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_node_set_reduction_variables() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path();

    {
        let mut file = ExodusFile::create(
            file_path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        let params = InitParams {
            title: "Node Set Reduction Test".to_string(),
            num_dim: 3,
            num_nodes: 3,
            num_elems: 5,
            num_elem_blocks: 1,
            num_node_sets: 2,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_coords(
            &[0.0, 1.0, 2.0],
            Some(&[0.0, 0.0, 0.0]),
            Some(&[0.0, 0.0, 0.0]),
        )
        .unwrap();

        let block = Block {
            id: 10,
            entity_type: EntityType::ElemBlock,
            topology: "tri".to_string(),
            num_entries: 5,
            num_nodes_per_entry: 3,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };
        file.put_block(&block).unwrap();
        file.put_connectivity(10, &[1, 2, 3, 2, 3, 1, 1, 2, 3, 2, 3, 1, 1, 2, 3])
            .unwrap();

        // Node sets
        let set1 = Set {
            id: 100,
            entity_type: EntityType::NodeSet,
            num_entries: 2,
            num_dist_factors: 0,
        };
        file.put_set(&set1).unwrap();
        file.put_node_set(100, &[1, 2], None).unwrap();

        let set2 = Set {
            id: 200,
            entity_type: EntityType::NodeSet,
            num_entries: 1,
            num_dist_factors: 0,
        };
        file.put_set(&set2).unwrap();
        file.put_node_set(200, &[3], None).unwrap();

        // Define reduction variables for node sets
        file.define_reduction_variables(EntityType::NodeSet, &["MaxTemp", "AvgPress"])
            .unwrap();

        file.put_time(0, 0.0).unwrap();
        file.put_reduction_vars(0, EntityType::NodeSet, 100, &[300.0, 101.0])
            .unwrap();
        file.put_reduction_vars(0, EntityType::NodeSet, 200, &[350.0, 102.0])
            .unwrap();
    }

    {
        let file = ExodusFile::open(file_path).unwrap();

        let names = file.reduction_variable_names(EntityType::NodeSet).unwrap();
        assert_eq!(names, vec!["MaxTemp", "AvgPress"]);

        let values = file
            .get_reduction_vars(0, EntityType::NodeSet, 100)
            .unwrap();
        assert_abs_diff_eq!(values[0], 300.0, epsilon = 1e-10);
        assert_abs_diff_eq!(values[1], 101.0, epsilon = 1e-10);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_side_set_reduction_variables() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path();

    {
        let mut file = ExodusFile::create(
            file_path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        let params = InitParams {
            title: "Side Set Reduction Test".to_string(),
            num_dim: 3,
            num_nodes: 4,
            num_elems: 1,
            num_elem_blocks: 1,
            num_side_sets: 1,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_coords(
            &[0.0, 1.0, 1.0, 0.0],
            Some(&[0.0, 0.0, 1.0, 1.0]),
            Some(&[0.0, 0.0, 0.0, 0.0]),
        )
        .unwrap();

        let block = Block {
            id: 10,
            entity_type: EntityType::ElemBlock,
            topology: "quad".to_string(),
            num_entries: 1,
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };
        file.put_block(&block).unwrap();
        file.put_connectivity(10, &[1, 2, 3, 4]).unwrap();

        // Side set
        let side_set = Set {
            id: 300,
            entity_type: EntityType::SideSet,
            num_entries: 2,
            num_dist_factors: 0,
        };
        file.put_set(&side_set).unwrap();
        file.put_side_set(300, &[1, 1], &[1, 2], None).unwrap();

        // Define reduction variables for side sets
        file.define_reduction_variables(EntityType::SideSet, &["AvgFlux"])
            .unwrap();

        file.put_time(0, 0.0).unwrap();
        file.put_reduction_vars(0, EntityType::SideSet, 300, &[42.5])
            .unwrap();
    }

    {
        let file = ExodusFile::open(file_path).unwrap();

        let names = file.reduction_variable_names(EntityType::SideSet).unwrap();
        assert_eq!(names, vec!["AvgFlux"]);

        let values = file
            .get_reduction_vars(0, EntityType::SideSet, 300)
            .unwrap();
        assert_abs_diff_eq!(values[0], 42.5, epsilon = 1e-10);
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_multiple_timesteps_reduction_variables() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path();

    {
        let mut file = ExodusFile::create(
            file_path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        let params = InitParams {
            title: "Multi-Timestep Reduction Test".to_string(),
            num_dim: 3,
            num_nodes: 3,
            num_elems: 1,
            num_elem_blocks: 1,
            num_assemblies: 1,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_coords(
            &[0.0, 1.0, 2.0],
            Some(&[0.0, 0.0, 0.0]),
            Some(&[0.0, 0.0, 0.0]),
        )
        .unwrap();

        let block = Block {
            id: 10,
            entity_type: EntityType::ElemBlock,
            topology: "tri".to_string(),
            num_entries: 1,
            num_nodes_per_entry: 3,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        };
        file.put_block(&block).unwrap();
        file.put_connectivity(10, &[1, 2, 3]).unwrap();

        let assembly = Assembly {
            id: 100,
            name: "TestAssembly".to_string(),
            entity_type: EntityType::ElemBlock,
            entity_list: vec![10],
        };
        file.put_assembly(&assembly).unwrap();

        file.define_reduction_variables(EntityType::Assembly, &["Energy", "Power"])
            .unwrap();

        // Write 10 time steps
        for ts in 0..10 {
            let time = ts as f64 * 0.1;
            file.put_time(ts, time).unwrap();

            let energy = 100.0 + ts as f64 * 10.0;
            let power = 50.0 + ts as f64 * 5.0;
            file.put_reduction_vars(ts, EntityType::Assembly, 100, &[energy, power])
                .unwrap();
        }
    }

    {
        let file = ExodusFile::open(file_path).unwrap();

        // Verify all time steps
        for ts in 0..10 {
            let values = file
                .get_reduction_vars(ts, EntityType::Assembly, 100)
                .unwrap();
            let expected_energy = 100.0 + ts as f64 * 10.0;
            let expected_power = 50.0 + ts as f64 * 5.0;
            assert_abs_diff_eq!(values[0], expected_energy, epsilon = 1e-10);
            assert_abs_diff_eq!(values[1], expected_power, epsilon = 1e-10);
        }
    }
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_empty_reduction_variables() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path();

    {
        let mut file = ExodusFile::create(
            file_path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();
        let params = InitParams {
            title: "Empty Reduction Test".to_string(),
            num_dim: 3,
            num_nodes: 3,
            num_assemblies: 2,
            ..Default::default()
        };
        file.init(&params).unwrap();

        file.put_coords(
            &[0.0, 1.0, 2.0],
            Some(&[0.0, 0.0, 0.0]),
            Some(&[0.0, 0.0, 0.0]),
        )
        .unwrap();

        // Put assemblies without reduction variables
        let assembly = Assembly {
            id: 100,
            name: "Assembly_A".to_string(),
            entity_type: EntityType::ElemBlock,
            entity_list: vec![],
        };
        file.put_assembly(&assembly).unwrap();
    }

    {
        let file = ExodusFile::open(file_path).unwrap();

        // Should return empty vector if no reduction variables defined
        let names = file.reduction_variable_names(EntityType::Assembly).unwrap();
        assert_eq!(names.len(), 0);
    }
}
