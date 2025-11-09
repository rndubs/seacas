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
    ExodusReader,
    EntityType,
    CreateOptions,
    CreateMode,
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

            # Check coordinates
            x, y, z = reader.get_coords()
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


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
