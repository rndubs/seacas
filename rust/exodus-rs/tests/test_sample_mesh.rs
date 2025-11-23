//! Tests for reading sample mesh data from the data directory.
//!
//! This test demonstrates reading an exodus file containing a meshed surface
//! with nodes, element connectivity, node sets, and side sets.

#[cfg(feature = "netcdf4")]
mod sample_mesh_tests {
    use exodus_rs::{mode, EntityType, ExodusFile};
    use std::path::PathBuf;

    fn get_sample_mesh_path() -> PathBuf {
        // Navigate from tests directory to rust/data
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("data")
            .join("square_surface_pressure_temperature_notime.e")
    }

    #[test]
    fn test_read_sample_mesh() {
        let mesh_path = get_sample_mesh_path();
        assert!(
            mesh_path.exists(),
            "Sample mesh file not found at {:?}",
            mesh_path
        );

        // Open the file for reading
        let file =
            ExodusFile::<mode::Read>::open(&mesh_path).expect("Failed to open sample mesh file");

        // Read initialization parameters
        let params = file.init_params().expect("Failed to read init params");
        println!("Title: {}", params.title);
        println!("Dimensions: {}", params.num_dim);
        println!("Nodes: {}", params.num_nodes);
        println!("Elements: {}", params.num_elems);
        println!("Element blocks: {}", params.num_elem_blocks);

        // Verify expected mesh structure
        assert_eq!(params.num_dim, 3);
        assert_eq!(params.num_nodes, 961);
        assert_eq!(params.num_elems, 900);
        assert_eq!(params.num_elem_blocks, 1);

        // Read coordinates
        let coords = file.coords::<f64>().expect("Failed to read coordinates");
        assert_eq!(coords.x.len(), params.num_nodes);
        assert_eq!(coords.y.len(), params.num_nodes);
        assert_eq!(coords.z.len(), params.num_nodes);

        println!(
            "Coordinate ranges: X=[{:.3}, {:.3}], Y=[{:.3}, {:.3}], Z=[{:.3}, {:.3}]",
            coords.x.iter().cloned().fold(f64::INFINITY, f64::min),
            coords.x.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            coords.y.iter().cloned().fold(f64::INFINITY, f64::min),
            coords.y.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            coords.z.iter().cloned().fold(f64::INFINITY, f64::min),
            coords.z.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        );

        // Read element blocks and connectivity
        let block_ids = file
            .block_ids(EntityType::ElemBlock)
            .expect("Failed to read block IDs");
        assert_eq!(block_ids.len(), 1);
        println!("Element block IDs: {:?}", block_ids);

        for &block_id in &block_ids {
            let block = file.block(block_id).expect("Failed to read block");
            println!(
                "Block {}: topology={}, entries={}, nodes_per_entry={}",
                block_id, block.topology, block.num_entries, block.num_nodes_per_entry
            );

            assert_eq!(block.topology, "SHELL4");
            assert_eq!(block.num_entries, 900);
            assert_eq!(block.num_nodes_per_entry, 4);

            // Read connectivity
            let connectivity = file
                .connectivity(block_id)
                .expect("Failed to read connectivity");
            assert_eq!(
                connectivity.len(),
                block.num_entries * block.num_nodes_per_entry
            );
            println!(
                "  First element connectivity: {:?}",
                &connectivity[..block.num_nodes_per_entry]
            );
        }

        // Check for node sets
        let nodeset_ids = file.set_ids(EntityType::NodeSet).unwrap_or_default();
        assert_eq!(nodeset_ids.len(), 1);
        println!("Node set IDs: {:?}", nodeset_ids);

        for &ns_id in &nodeset_ids {
            let node_set = file.node_set(ns_id).expect("Failed to read node set");
            println!(
                "Node set {}: {} nodes, {} dist factors",
                node_set.id,
                node_set.nodes.len(),
                node_set.dist_factors.len()
            );
            assert_eq!(node_set.nodes.len(), 961); // All nodes
        }

        // Check for side sets
        let sideset_ids = file.set_ids(EntityType::SideSet).unwrap_or_default();
        assert_eq!(sideset_ids.len(), 1);
        println!("Side set IDs: {:?}", sideset_ids);

        for &ss_id in &sideset_ids {
            let side_set = file.side_set(ss_id).expect("Failed to read side set");
            println!(
                "Side set {}: {} entries, {} dist factors",
                side_set.id,
                side_set.elements.len(),
                side_set.dist_factors.len()
            );
            assert_eq!(side_set.elements.len(), 900); // All elements
        }

        println!("Successfully read sample mesh file!");
    }
}
