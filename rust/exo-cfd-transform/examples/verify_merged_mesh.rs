//! Verify the merged mesh from copy-mirror-merge operation
//!
//! Run with: cargo run --example verify_merged_mesh

use exodus_rs::{mode, types::*, ExodusFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "test_full_symmetry.exo";

    let file = ExodusFile::<mode::Read>::open(path)?;

    let params = file.init_params()?;
    println!("Mesh: {}", path);
    println!("  Nodes: {}", params.num_nodes);
    println!("  Elements: {}", params.num_elems);
    println!("  Node sets: {}", params.num_node_sets);
    println!("  Side sets: {}", params.num_side_sets);

    // Read coordinates
    let coords = file.coords()?;
    println!("\nCoordinates (first 10 nodes):");
    for i in 0..params.num_nodes.min(10) {
        println!(
            "  Node {}: ({:.3}, {:.3}, {:.3})",
            i + 1,
            coords.x[i],
            coords.y[i],
            coords.z[i]
        );
    }
    if params.num_nodes > 10 {
        println!("  ... and {} more nodes", params.num_nodes - 10);
    }

    // Check x coordinate range to verify mirroring
    let x_min = coords.x.iter().copied().fold(f64::INFINITY, f64::min);
    let x_max = coords.x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    println!("\nX coordinate range: [{:.3}, {:.3}]", x_min, x_max);
    println!(
        "Symmetric about x=0: {}",
        (x_min.abs() - x_max.abs()).abs() < 0.01
    );

    // Read element block
    let block_ids = file.block_ids(EntityType::ElemBlock)?;
    for block_id in &block_ids {
        let block = file.block(*block_id)?;
        println!(
            "\nElement block {}: {} {} elements",
            block_id, block.num_entries, block.topology
        );
    }

    // Read node sets
    let ns_ids = file.set_ids(EntityType::NodeSet)?;
    let ns_names = file.names(EntityType::NodeSet).unwrap_or_default();
    println!("\nNode sets:");
    for (i, &set_id) in ns_ids.iter().enumerate() {
        let ns = file.node_set(set_id)?;
        let name = ns_names.get(i).map(|s| s.as_str()).unwrap_or("unnamed");
        println!("  {} (id={}): {} nodes", name, set_id, ns.nodes.len());
    }

    // Read side sets
    let ss_ids = file.set_ids(EntityType::SideSet)?;
    let ss_names = file.names(EntityType::SideSet).unwrap_or_default();
    println!("\nSide sets:");
    for (i, &set_id) in ss_ids.iter().enumerate() {
        let ss = file.side_set(set_id)?;
        let name = ss_names.get(i).map(|s| s.as_str()).unwrap_or("unnamed");
        println!("  {} (id={}): {} faces", name, set_id, ss.elements.len());
    }

    // Read nodal variables
    let var_names = file.variable_names(EntityType::Nodal)?;
    let times = file.times()?;
    println!("\nNodal variables: {:?}", var_names);
    println!("Time steps: {}", times.len());

    // Check velocity_x values for proper mirroring
    if let Some(vx_idx) = var_names.iter().position(|n| n == "velocity_x") {
        let vx = file.var(0, EntityType::Nodal, 0, vx_idx)?;
        println!("\nvelocity_x at t=0 (sample):");

        // Find nodes on symmetry plane (x=0)
        let sym_nodes: Vec<usize> = coords
            .x
            .iter()
            .enumerate()
            .filter(|(_, &x)| x.abs() < 0.01)
            .map(|(i, _)| i)
            .collect();
        println!("  Nodes on symmetry plane (x=0): {} nodes", sym_nodes.len());

        // Check that vx is 0 at symmetry plane
        let vx_at_sym: Vec<f64> = sym_nodes.iter().map(|&i| vx[i]).collect();
        let max_vx_at_sym = vx_at_sym.iter().copied().fold(0.0, f64::max);
        println!("  Max |velocity_x| at symmetry plane: {:.6}", max_vx_at_sym);

        // Find mirrored pairs and check symmetry
        let mut positive_x_nodes: Vec<(usize, f64, f64)> = vec![];
        let mut negative_x_nodes: Vec<(usize, f64, f64)> = vec![];

        for (i, &x) in coords.x.iter().enumerate() {
            if x > 0.01 {
                positive_x_nodes.push((i, x, vx[i]));
            } else if x < -0.01 {
                negative_x_nodes.push((i, x, vx[i]));
            }
        }

        println!("\nvelocity_x symmetry check:");
        println!("  Positive x nodes: {}", positive_x_nodes.len());
        println!("  Negative x nodes: {}", negative_x_nodes.len());

        // Sample a few nodes
        for (i, x, v) in positive_x_nodes.iter().take(3) {
            println!("    Node {} at x={:.3}: velocity_x = {:.6}", i + 1, x, v);
        }
        for (i, x, v) in negative_x_nodes.iter().take(3) {
            println!("    Node {} at x={:.3}: velocity_x = {:.6}", i + 1, x, v);
        }
    }

    println!("\nVerification complete!");

    Ok(())
}
