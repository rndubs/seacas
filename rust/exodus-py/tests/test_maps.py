"""
Tests for map operations (ID maps, element order maps, properties)
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


def test_node_id_map():
    """Test writing and reading node ID map"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Node Map Test", num_dim=2, num_nodes=5)
        writer.put_init_params(params)

        # Write custom node ID map (nodes numbered 10, 20, 30, 40, 50)
        node_map = [10, 20, 30, 40, 50]
        writer.put_id_map("node", node_map)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        map_read = reader.get_id_map("node")
        assert map_read == node_map
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_elem_id_map():
    """Test writing and reading element ID map"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Elem Map Test",
            num_dim=2,
            num_nodes=8,
            num_elems=3,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        # Define element block
        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=3,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Write custom element ID map
        elem_map = [100, 101, 102]
        writer.put_id_map("elem", elem_map)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        map_read = reader.get_id_map("elem")
        assert map_read == elem_map
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_element_order_map():
    """Test writing and reading element order map"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Order Map Test",
            num_dim=2,
            num_nodes=12,
            num_elems=4,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=4,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Write element order map (process elements in reverse order)
        order_map = [4, 3, 2, 1]
        writer.put_elem_order_map(order_map)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        order_read = reader.get_elem_order_map()
        assert order_read == order_map
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_entity_names():
    """Test setting and getting entity names"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Names Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
            num_node_sets=1,
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
        writer.put_node_set(1, [1, 2, 3, 4])

        # Set individual names
        writer.put_name("elem_block", 0, "MainBlock")
        writer.put_name("node_set", 0, "Boundary")
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        block_name = reader.get_name("elem_block", 0)
        assert block_name == "MainBlock"

        ns_name = reader.get_name("node_set", 0)
        assert ns_name == "Boundary"
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_entity_names_batch():
    """Test setting multiple entity names at once"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Batch Names",
            num_dim=2,
            num_nodes=8,
            num_elems=3,
            num_elem_blocks=3,
        )
        writer.put_init_params(params)

        # Define three blocks
        for i in range(3):
            block = Block(
                id=i + 1,
                entity_type=EntityType.ElemBlock,
                topology="QUAD4",
                num_entries=1,
                num_nodes_per_entry=4,
                num_attributes=0,
            )
            writer.put_block(block)

        # Set all block names at once
        names = ["Block1", "Block2", "Block3"]
        writer.put_names("elem_block", names)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        for i, expected_name in enumerate(names):
            name = reader.get_name("elem_block", i)
            assert name == expected_name
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_entity_property():
    """Test setting and getting entity properties"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Property Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        block = Block(
            id=100,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Set property for specific block
        writer.put_property("elem_block", 100, "material_id", 42)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        prop_value = reader.get_property("elem_block", 100, "material_id")
        assert prop_value == 42
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_property_array():
    """Test setting and getting property arrays"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Property Array",
            num_dim=2,
            num_nodes=12,
            num_elems=3,
            num_elem_blocks=3,
        )
        writer.put_init_params(params)

        # Define three blocks
        for i in range(3):
            block = Block(
                id=i + 1,
                entity_type=EntityType.ElemBlock,
                topology="QUAD4",
                num_entries=1,
                num_nodes_per_entry=4,
                num_attributes=0,
            )
            writer.put_block(block)

        # Set property for all blocks at once
        mat_ids = [1, 2, 3]
        writer.put_property_array("elem_block", "material_id", mat_ids)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        for block_id, expected_mat_id in enumerate(mat_ids, start=1):
            prop_value = reader.get_property("elem_block", block_id, "material_id")
            assert prop_value == expected_mat_id
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
