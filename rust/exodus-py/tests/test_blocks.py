"""
Tests for block operations
"""

import pytest
import tempfile
import os

pytest.importorskip("exodus")

from exodus import (
    ExodusWriter,
    ExodusReader,
    InitParams,
    Block,
    EntityType,
)


def test_define_and_get_elem_block():
    """Test defining and reading element blocks"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        # Create file with element block
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Block Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        # Define element block
        writer.put_elem_block(1, "QUAD4", 1, 4, 0)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        block = reader.get_elem_block(1)
        assert block.entity_type == "QUAD4"
        assert block.num_entries == 1
        assert block.nodes_per_entry == 4
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_elem_block_connectivity():
    """Test writing and reading element connectivity"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Connectivity Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        # Define block
        writer.put_elem_block(1, "QUAD4", 1, 4, 0)

        # Write connectivity
        connectivity = [1, 2, 3, 4]
        writer.put_elem_connectivity(1, connectivity)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        conn_read = reader.get_elem_connectivity(1)
        assert len(conn_read) == 4
        assert conn_read == connectivity
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_elem_blocks():
    """Test multiple element blocks"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Multi Block",
            num_dim=3,
            num_nodes=12,
            num_elems=2,
            num_elem_blocks=2,
        )
        writer.put_init_params(params)

        # Define two blocks
        writer.put_elem_block(100, "HEX8", 1, 8, 0)
        writer.put_elem_block(200, "TET4", 1, 4, 0)

        # Write connectivity for both
        writer.put_elem_connectivity(100, [1, 2, 3, 4, 5, 6, 7, 8])
        writer.put_elem_connectivity(200, [9, 10, 11, 12])
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)

        # Check block IDs
        block_ids = reader.get_elem_block_ids()
        assert len(block_ids) == 2
        assert 100 in block_ids
        assert 200 in block_ids

        # Check first block
        block1 = reader.get_elem_block(100)
        assert block1.entity_type == "HEX8"
        assert block1.num_entries == 1

        # Check second block
        block2 = reader.get_elem_block(200)
        assert block2.entity_type == "TET4"
        assert block2.num_entries == 1

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_elem_block_attributes():
    """Test element block attributes"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Attributes Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        # Define block with attributes
        writer.put_elem_block(1, "QUAD4", 1, 4, 2)

        # Write attributes
        attributes = [1.5, 2.5]
        writer.put_elem_attr(1, attributes)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        block = reader.get_elem_block(1)
        assert block.num_attrs == 2

        attrs_read = reader.get_elem_attr(1)
        assert len(attrs_read) == 2
        assert attrs_read == pytest.approx(attributes, abs=1e-6)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_elem_block_names():
    """Test element block naming"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Block Names",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        writer.put_elem_block(1, "QUAD4", 1, 4, 0)

        # Set block name
        writer.put_elem_block_name(1, "MaterialBlock1")
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        name = reader.get_elem_block_name(1)
        assert name == "MaterialBlock1"
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_edge_blocks():
    """Test edge blocks"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Edge Block Test",
            num_dim=2,
            num_nodes=4,
            num_edges=2,
            num_edge_blocks=1,
        )
        writer.put_init_params(params)

        # Define edge block
        writer.put_edge_block(1, "EDGE2", 2, 2)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        block = reader.get_edge_block(1)
        assert block.entity_type == "EDGE2"
        assert block.num_entries == 2
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_face_blocks():
    """Test face blocks"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Face Block Test",
            num_dim=3,
            num_nodes=4,
            num_faces=1,
            num_face_blocks=1,
        )
        writer.put_init_params(params)

        # Define face block
        writer.put_face_block(1, "TRI3", 1, 3)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        block = reader.get_face_block(1)
        assert block.entity_type == "TRI3"
        assert block.num_entries == 1
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
