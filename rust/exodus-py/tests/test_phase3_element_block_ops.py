"""
Test Phase 3.1 element block operations in exomerge module.
Tests: create_element_block, delete_element_block, rename_element_block, get_nodes_in_element_block
"""

import pytest
import sys
sys.path.insert(0, 'python')

from exodus.exomerge import ExodusModel


@pytest.fixture
def basic_model():
    """Create a basic model with nodes and an element block."""
    model = ExodusModel()

    # Create nodes for a hex8 element
    model.create_nodes([
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ])

    # Create an element block with connectivity
    # HEX8 element: 1 element, 8 nodes per element
    connectivity = [[1, 2, 3, 4, 5, 6, 7, 8]]  # 1-based indexing
    model.create_element_block(1, ['hex8', 1, 8, 0], connectivity)

    return model


def test_create_element_block():
    """Test creating element blocks."""
    model = ExodusModel()

    # Create some nodes first
    model.create_nodes([
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ])

    # Create an element block with connectivity
    connectivity = [[1, 2, 3, 4, 5, 6, 7, 8]]  # 1-based indexing
    model.create_element_block(1, ['hex8', 1, 8, 0], connectivity)

    assert model.element_block_exists(1), "Element block 1 should exist"
    assert len(model.get_element_block_ids()) == 1, "Should have 1 element block"


def test_get_nodes_in_element_block(basic_model):
    """Test getting nodes from element blocks."""
    nodes = basic_model.get_nodes_in_element_block(1)
    assert len(nodes) == 8, f"Should have 8 nodes, got {len(nodes)}"
    assert sorted(nodes) == [1, 2, 3, 4, 5, 6, 7, 8], f"Nodes should be 1-8, got {nodes}"

    # Test "all" option
    all_nodes = basic_model.get_nodes_in_element_block("all")
    assert all_nodes == nodes, "All nodes should match block 1 nodes"


def test_rename_element_block(basic_model):
    """Test renaming element blocks."""
    # Rename ID from 1 to 100
    basic_model.rename_element_block(1, 100)
    assert not basic_model.element_block_exists(1), "Element block 1 should not exist"
    assert basic_model.element_block_exists(100), "Element block 100 should exist"

    # Rename back and set a name
    basic_model.rename_element_block(100, 1)
    basic_model.rename_element_block(1, "my_block")

    name = basic_model.get_element_block_name(1)
    assert name == "my_block", f"Block name should be 'my_block', got '{name}'"


def test_delete_element_block(basic_model):
    """Test deleting element blocks."""
    # Create another element block first
    basic_model.create_element_block(2, ['hex8', 0, 8, 0], [])

    initial_count = len(basic_model.get_element_block_ids())
    basic_model.delete_element_block(2)

    assert len(basic_model.get_element_block_ids()) == initial_count - 1, "Should have one less block"
    assert not basic_model.element_block_exists(2), "Element block 2 should not exist"
    assert basic_model.element_block_exists(1), "Element block 1 should still exist"
