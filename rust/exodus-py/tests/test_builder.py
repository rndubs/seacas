"""
Tests for the builder API
"""

import pytest
import tempfile
import os

# Test will be skipped if exodus module is not built
pytest.importorskip("exodus")

from exodus import (
    MeshBuilder,
    BlockBuilder,
    NodeSetBuilder,
    SideSetBuilder,
    AppendBuilder,
    ExodusReader,
    ExodusWriter,
    EntityType,
    CreateOptions,
    CreateMode,
    InitParams,
)


def test_block_builder():
    """Test BlockBuilder creation and methods"""
    block = (
        BlockBuilder(1, "HEX8")
        .connectivity([1, 2, 3, 4, 5, 6, 7, 8])
        .attributes([100.0])
        .attribute_names(["MaterialID"])
        .build()
    )
    assert block is not None


def test_simple_mesh_builder():
    """Test creating a simple 2D quad mesh"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        # Create mesh
        (
            MeshBuilder("Test Quad")
            .dimensions(2)
            .coordinates(
                x=[0.0, 1.0, 1.0, 0.0],
                y=[0.0, 0.0, 1.0, 1.0],
                z=[],
            )
            .add_block(BlockBuilder(1, "QUAD4").connectivity([1, 2, 3, 4]).build())
            .write(tmp_path)
        )

        # Verify the file was created
        assert os.path.exists(tmp_path)

        # Read and verify
        with ExodusReader.open(tmp_path) as reader:
            params = reader.init_params()
            assert params.num_nodes == 4
            assert params.num_elems == 1
            assert params.num_elem_blocks == 1
            assert params.num_dim == 2

            # Check coordinates (using backward-compatible list API)
            x, y, z = reader.get_coords_list()
            assert len(x) == 4
            assert len(y) == 4

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_hex_mesh_builder():
    """Test creating a 3D hex mesh"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        # Create mesh
        (
            MeshBuilder("Test Hex")
            .dimensions(3)
            .coordinates(
                x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
                y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
                z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            )
            .add_block(
                BlockBuilder(100, "HEX8")
                .connectivity([1, 2, 3, 4, 5, 6, 7, 8])
                .attributes([1.0])
                .attribute_names(["MaterialID"])
                .build()
            )
            .qa_record("pytest", "1.0", "2025-01-15", "12:00:00")
            .info("Test mesh")
            .write(tmp_path)
        )

        # Verify
        with ExodusReader.open(tmp_path) as reader:
            params = reader.init_params()
            assert params.num_nodes == 8
            assert params.num_elems == 1
            assert params.num_dim == 3

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multi_block_mesh():
    """Test creating a mesh with multiple blocks"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        (
            MeshBuilder("Multi-Block Mesh")
            .dimensions(3)
            .coordinates(
                x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 2.0, 2.0],
                y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0],
                z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0],
            )
            .add_block(
                BlockBuilder(1, "HEX8")
                .connectivity([1, 2, 3, 4, 5, 6, 7, 8])
                .build()
            )
            .add_block(
                BlockBuilder(2, "TRI3")
                .connectivity([2, 9, 10])
                .build()
            )
            .write(tmp_path)
        )

        with ExodusReader.open(tmp_path) as reader:
            params = reader.init_params()
            assert params.num_nodes == 10
            assert params.num_elems == 2
            assert params.num_elem_blocks == 2

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_create_options():
    """Test using CreateOptions"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        opts = CreateOptions(mode=CreateMode.Clobber)

        (
            MeshBuilder("Options Test")
            .dimensions(2)
            .coordinates(x=[0.0, 1.0], y=[0.0, 0.0], z=[])
            .write_with_options(tmp_path, opts)
        )

        assert os.path.exists(tmp_path)

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_node_set_builder():
    """Test NodeSetBuilder creation and methods"""
    node_set = (
        NodeSetBuilder(10)
        .nodes([1, 2, 3, 4])
        .name("inlet")
        .dist_factors([1.0, 1.0, 1.0, 1.0])
        .build()
    )
    assert node_set is not None


def test_side_set_builder():
    """Test SideSetBuilder creation and methods"""
    side_set = (
        SideSetBuilder(20)
        .sides([(1, 1), (1, 2), (2, 3)])
        .name("outlet")
        .build()
    )
    assert side_set is not None


def test_side_set_builder_separate_arrays():
    """Test SideSetBuilder with separate element/side arrays"""
    side_set = (
        SideSetBuilder(30)
        .elements_and_sides([1, 1, 2], [1, 2, 3])
        .build()
    )
    assert side_set is not None


def test_append_builder_add_node_set():
    """Test AppendBuilder adding a node set"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        # First create a mesh with node set capacity
        with ExodusWriter.create(
            tmp_path, CreateOptions(mode=CreateMode.Clobber)
        ) as writer:
            params = InitParams(
                title="Test Mesh",
                num_dim=2,
                num_nodes=4,
                num_elems=1,
                num_elem_blocks=1,
                num_node_sets=1,  # Reserve space for node sets
            )
            writer.put_init_params(params)
            writer.put_coords(
                x=[0.0, 1.0, 1.0, 0.0],
                y=[0.0, 0.0, 1.0, 1.0],
            )

        # Then add a node set using AppendBuilder
        (
            AppendBuilder.open(tmp_path)
            .add_node_set(
                NodeSetBuilder(10)
                .nodes([1, 2])
                .name("boundary")
                .build()
            )
            .apply()
        )

        # Verify the node set was added
        with ExodusReader.open(tmp_path) as reader:
            ids = reader.get_node_set_ids()
            assert 10 in ids

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_append_builder_add_side_set():
    """Test AppendBuilder adding a side set"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        # First create a mesh with side set capacity
        with ExodusWriter.create(
            tmp_path, CreateOptions(mode=CreateMode.Clobber)
        ) as writer:
            params = InitParams(
                title="Test Mesh",
                num_dim=2,
                num_nodes=4,
                num_elems=1,
                num_elem_blocks=1,
                num_side_sets=1,  # Reserve space for side sets
            )
            writer.put_init_params(params)
            writer.put_coords(
                x=[0.0, 1.0, 1.0, 0.0],
                y=[0.0, 0.0, 1.0, 1.0],
            )

        # Then add a side set using AppendBuilder
        (
            AppendBuilder.open(tmp_path)
            .add_side_set(
                SideSetBuilder(20)
                .sides([(1, 1), (1, 2)])
                .name("surface")
                .build()
            )
            .apply()
        )

        # Verify the side set was added
        with ExodusReader.open(tmp_path) as reader:
            ids = reader.get_side_set_ids()
            assert 20 in ids

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_append_builder_fluent_chain():
    """Test AppendBuilder with multiple operations chained"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        # Create a mesh with space for both node and side sets
        with ExodusWriter.create(
            tmp_path, CreateOptions(mode=CreateMode.Clobber)
        ) as writer:
            params = InitParams(
                title="Test Mesh",
                num_dim=2,
                num_nodes=4,
                num_elems=1,
                num_elem_blocks=1,
                num_node_sets=2,
                num_side_sets=2,
            )
            writer.put_init_params(params)
            writer.put_coords(
                x=[0.0, 1.0, 1.0, 0.0],
                y=[0.0, 0.0, 1.0, 1.0],
            )

        # Add multiple sets using fluent builder
        (
            AppendBuilder.open(tmp_path)
            .add_node_set(NodeSetBuilder(10).nodes([1, 2]).name("inlet").build())
            .add_node_set(NodeSetBuilder(11).nodes([3, 4]).name("outlet").build())
            .add_side_set(SideSetBuilder(20).sides([(1, 1)]).name("wall1").build())
            .add_side_set(SideSetBuilder(21).sides([(1, 2)]).name("wall2").build())
            .apply()
        )

        # Verify all sets were added
        with ExodusReader.open(tmp_path) as reader:
            node_set_ids = reader.get_node_set_ids()
            side_set_ids = reader.get_side_set_ids()
            assert 10 in node_set_ids
            assert 11 in node_set_ids
            assert 20 in side_set_ids
            assert 21 in side_set_ids

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
