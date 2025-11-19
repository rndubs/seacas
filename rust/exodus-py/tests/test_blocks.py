"""
Tests for block operations
"""

import pytest
import tempfile
import os
import numpy as np

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

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

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

        # Define element block using Block object
        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        block_read = reader.get_block(1)
        assert block_read.topology == "QUAD4"
        assert block_read.num_entries == 1
        assert block_read.num_nodes_per_entry == 4
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_elem_block_connectivity():
    """Test writing and reading element connectivity"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

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
        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Write connectivity
        connectivity = [1, 2, 3, 4]
        writer.put_connectivity(1, connectivity)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        conn_read = reader.get_connectivity(1)
        assert len(conn_read) == 4
        np.testing.assert_array_equal(conn_read, connectivity)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_elem_blocks():
    """Test multiple element blocks"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

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

        # Define two blocks using Block objects
        block1 = Block(
            id=100,
            entity_type=EntityType.ElemBlock,
            topology="HEX8",
            num_entries=1,
            num_nodes_per_entry=8,
            num_attributes=0,
        )
        block2 = Block(
            id=200,
            entity_type=EntityType.ElemBlock,
            topology="TET4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block1)
        writer.put_block(block2)

        # Write connectivity for both
        writer.put_connectivity(100, [1, 2, 3, 4, 5, 6, 7, 8])
        writer.put_connectivity(200, [9, 10, 11, 12])
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)

        # Check block IDs
        block_ids = reader.get_block_ids()
        assert len(block_ids) == 2
        assert 100 in block_ids
        assert 200 in block_ids

        # Check first block
        block1_read = reader.get_block(100)
        assert block1_read.topology == "HEX8"
        assert block1_read.num_entries == 1

        # Check second block
        block2_read = reader.get_block(200)
        assert block2_read.topology == "TET4"
        assert block2_read.num_entries == 1

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_elem_block_attributes():
    """Test element block attributes"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

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
        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=2,
        )
        writer.put_block(block)

        # Write attributes
        attributes = [1.5, 2.5]
        writer.put_block_attributes(1, attributes)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        block_read = reader.get_block(1)
        assert block_read.num_attributes == 2

        attrs_read = reader.get_block_attributes(1)
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

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

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

        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Set block name using generic naming API
        # Block with ID 1 is at index 0 (first block)
        writer.put_name("elem_block", 0, "MaterialBlock1")
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        name = reader.get_name("elem_block", 0)
        assert name == "MaterialBlock1"
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_edge_blocks():
    """Test edge blocks"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

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

        # Define edge block using Block object
        block = Block(
            id=1,
            entity_type=EntityType.EdgeBlock,
            topology="EDGE2",
            num_entries=2,
            num_nodes_per_entry=2,
            num_attributes=0,
        )
        writer.put_block(block)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        block_read = reader.get_block(1)
        assert block_read.topology == "EDGE2"
        assert block_read.num_entries == 2
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_face_blocks():
    """Test face blocks"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

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

        # Define face block using Block object
        block = Block(
            id=1,
            entity_type=EntityType.FaceBlock,
            topology="TRI3",
            num_entries=1,
            num_nodes_per_entry=3,
            num_attributes=0,
        )
        writer.put_block(block)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        block_read = reader.get_block(1)
        assert block_read.topology == "TRI3"
        assert block_read.num_entries == 1
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
