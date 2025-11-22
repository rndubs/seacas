"""
Tests for reading sample mesh data from the data directory.

This test demonstrates reading an exodus file containing a meshed surface
with nodes, element connectivity, node sets, and side sets.
"""

import pytest
from pathlib import Path

pytest.importorskip("exodus")

from exodus import ExodusReader, EntityType


def get_sample_mesh_path():
    """Get the path to the sample mesh file in the data directory."""
    # Navigate from tests directory to rust/data
    tests_dir = Path(__file__).parent
    data_dir = tests_dir.parent.parent / "data"
    mesh_path = data_dir / "square_surface_pressure_temperature_notime.e"
    return mesh_path


def test_read_sample_mesh():
    """Test reading the sample mesh file with nodes, connectivity, and sets."""
    mesh_path = get_sample_mesh_path()
    assert mesh_path.exists(), f"Sample mesh file not found at {mesh_path}"

    # Open the file for reading
    reader = ExodusReader.open(str(mesh_path))

    try:
        # Read initialization parameters
        params = reader.init_params()
        print(f"\nTitle: {params.title}")
        print(f"Dimensions: {params.num_dim}")
        print(f"Nodes: {params.num_nodes}")
        print(f"Elements: {params.num_elems}")
        print(f"Element blocks: {params.num_elem_blocks}")

        # Verify expected mesh structure
        assert params.num_dim == 3
        assert params.num_nodes == 961
        assert params.num_elems == 900
        assert params.num_elem_blocks == 1

        # Read coordinates
        x_coords, y_coords, z_coords = reader.get_coords()
        assert len(x_coords) == params.num_nodes
        assert len(y_coords) == params.num_nodes
        assert len(z_coords) == params.num_nodes

        print(
            f"Coordinate ranges: X=[{min(x_coords):.3f}, {max(x_coords):.3f}], "
            f"Y=[{min(y_coords):.3f}, {max(y_coords):.3f}], "
            f"Z=[{min(z_coords):.3f}, {max(z_coords):.3f}]"
        )

        # Read element blocks and connectivity
        block_ids = reader.get_block_ids()
        assert len(block_ids) == 1
        print(f"Element block IDs: {block_ids}")

        for block_id in block_ids:
            block = reader.get_block(block_id)
            print(
                f"Block {block_id}: topology={block.topology}, "
                f"entries={block.num_entries}, nodes_per_entry={block.num_nodes_per_entry}"
            )

            assert block.topology == "SHELL4"
            assert block.num_entries == 900
            assert block.num_nodes_per_entry == 4

            # Read connectivity
            connectivity = reader.get_connectivity(block_id)
            assert len(connectivity) == block.num_entries * block.num_nodes_per_entry
            print(f"  First element connectivity: {connectivity[:block.num_nodes_per_entry]}")

        # Check for node sets
        nodeset_ids = reader.get_node_set_ids()
        assert len(nodeset_ids) == 1
        print(f"Node set IDs: {nodeset_ids}")

        for ns_id in nodeset_ids:
            node_set = reader.get_node_set(ns_id)
            print(
                f"Node set {node_set.id}: {len(node_set.nodes)} nodes, "
                f"{len(node_set.dist_factors)} dist factors"
            )
            assert len(node_set.nodes) == 961  # All nodes

        # Check for side sets
        sideset_ids = reader.get_side_set_ids()
        assert len(sideset_ids) == 1
        print(f"Side set IDs: {sideset_ids}")

        for ss_id in sideset_ids:
            side_set = reader.get_side_set(ss_id)
            print(
                f"Side set {side_set.id}: {len(side_set.elements)} entries, "
                f"{len(side_set.dist_factors)} dist factors"
            )
            assert len(side_set.elements) == 900  # All elements

        print("Successfully read sample mesh file!")

    finally:
        reader.close()


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
