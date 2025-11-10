"""
Tests for set operations (node sets, side sets, element sets)
"""

import pytest
import tempfile
import os

pytest.importorskip("exodus")

from exodus import (
    ExodusWriter,
    ExodusReader,
    InitParams,
    NodeSet,
    SideSet,
    EntitySet,
    Block,
    EntityType,
)


def test_node_set():
    """Test creating and reading node sets"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Node Set Test",
            num_dim=2,
            num_nodes=8,
            num_node_sets=1,
        )
        writer.put_init_params(params)

        # Define node set
        node_ids = [1, 3, 5, 7]
        writer.put_node_set(1, node_ids)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        node_set = reader.get_node_set(1)
        assert len(node_set) == 4
        assert node_set == node_ids
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_node_set_with_dist_factors():
    """Test node set with distribution factors"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Node Set DF",
            num_dim=2,
            num_nodes=8,
            num_node_sets=1,
        )
        writer.put_init_params(params)

        # Define node set with distribution factors
        node_ids = [1, 2, 3, 4]
        writer.put_node_set(1, node_ids)

        dist_factors = [1.0, 2.0, 3.0, 4.0]
        writer.put_node_set_dist_fact(1, dist_factors)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        node_set = reader.get_node_set(1)
        assert len(node_set) == 4

        df_read = reader.get_node_set_dist_fact(1)
        assert len(df_read) == 4
        assert df_read == pytest.approx(dist_factors, abs=1e-6)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_side_set():
    """Test creating and reading side sets"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Side Set Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
            num_side_sets=1,
        )
        writer.put_init_params(params)

        # Define element block first
        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Define side set
        elem_ids = [1]
        side_ids = [1]
        writer.put_side_set(1, elem_ids, side_ids)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        elem_list, side_list = reader.get_side_set(1)
        assert len(elem_list) == 1
        assert len(side_list) == 1
        assert elem_list == elem_ids
        assert side_list == side_ids
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_side_set_with_dist_factors():
    """Test side set with distribution factors"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Side Set DF",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
            num_side_sets=1,
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

        # Define side set
        elem_ids = [1, 1]
        side_ids = [1, 2]
        writer.put_side_set(1, elem_ids, side_ids)

        # Distribution factors for 2 sides, assuming 2 nodes per side
        dist_factors = [1.0, 2.0, 3.0, 4.0]
        writer.put_side_set_dist_fact(1, dist_factors)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        df_read = reader.get_side_set_dist_fact(1)
        assert len(df_read) == 4
        assert df_read == pytest.approx(dist_factors, abs=1e-6)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_elem_set():
    """Test creating and reading element sets"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Elem Set Test",
            num_dim=2,
            num_nodes=8,
            num_elems=4,
            num_elem_blocks=1,
            num_elem_sets=1,
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

        # Define element set
        elem_ids = [1, 3]
        writer.put_elem_set(1, elem_ids)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        elem_set = reader.get_elem_set(1)
        assert len(elem_set) == 2
        assert elem_set == elem_ids
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_node_sets():
    """Test multiple node sets"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Multi Node Sets",
            num_dim=2,
            num_nodes=10,
            num_node_sets=2,
        )
        writer.put_init_params(params)

        # Define two node sets
        writer.put_node_set(1, [1, 2, 3])
        writer.put_node_set(2, [7, 8, 9])
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        set_ids = reader.get_node_set_ids()
        assert len(set_ids) == 2
        assert 1 in set_ids
        assert 2 in set_ids

        set1 = reader.get_node_set(1)
        set2 = reader.get_node_set(2)
        assert len(set1) == 3
        assert len(set2) == 3
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_set_names():
    """Test setting and reading set names"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Set Names",
            num_dim=2,
            num_nodes=8,
            num_node_sets=1,
        )
        writer.put_init_params(params)

        writer.put_node_set(1, [1, 2, 3, 4])
        writer.put_node_set_name(1, "BoundaryNodes")
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        name = reader.get_node_set_name(1)
        assert name == "BoundaryNodes"
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
