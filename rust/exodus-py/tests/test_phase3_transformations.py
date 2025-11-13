"""
Test Phase 3 geometric transformations and advanced operations in exomerge module.
Tests: transformation operations, duplicate_element_block, get_element_block_extents
"""

import pytest
import sys
sys.path.insert(0, 'python')

from exodus.exomerge import ExodusModel


@pytest.fixture
def quad_model():
    """Create a model with two separate quad blocks."""
    model = ExodusModel()

    # Create two separate blocks with their own nodes
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]])
    model.create_element_block(1, ['quad4', 1, 4, 0], [[1, 2, 3, 4]])

    model.create_nodes([[5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [6.0, 1.0, 0.0], [5.0, 1.0, 0.0]])
    model.create_element_block(2, ['quad4', 1, 4, 0], [[5, 6, 7, 8]])

    return model


def test_translate_element_blocks(quad_model):
    """Test translating element blocks."""
    # Translate block 1
    quad_model.translate_element_blocks(1, [10.0, 0.0, 0.0])

    # Check that block 1 nodes moved
    assert abs(quad_model.nodes[0][0] - 10.0) < 1e-10, f"Node 0 x should be 10.0, got {quad_model.nodes[0][0]}"
    # Check that block 2 nodes didn't move
    assert abs(quad_model.nodes[4][0] - 5.0) < 1e-10, f"Node 4 x should be 5.0, got {quad_model.nodes[4][0]}"


def test_scale_element_blocks():
    """Test scaling element blocks."""
    model = ExodusModel()

    # Create a simple block
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]])
    model.create_element_block(1, ['quad4', 1, 4, 0], [[1, 2, 3, 4]])

    # Scale by 2
    model.scale_element_blocks(1, 2.0)

    # Check scaled coordinates
    assert abs(model.nodes[1][0] - 2.0) < 1e-10, f"Node 1 x should be 2.0, got {model.nodes[1][0]}"
    assert abs(model.nodes[2][1] - 2.0) < 1e-10, f"Node 2 y should be 2.0, got {model.nodes[2][1]}"


def test_rotate_element_blocks():
    """Test rotating element blocks."""
    model = ExodusModel()

    # Create a simple block
    model.create_nodes([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])
    model.create_element_block(1, ['tri3', 1, 3, 0], [[1, 2, 3]])

    # Rotate 90 degrees around Z-axis
    model.rotate_element_blocks(1, [0, 0, 1], 90)

    # After 90° rotation around Z: (1,0,0) -> (0,1,0)
    assert abs(model.nodes[0][0] - 0.0) < 1e-10, f"Node 0 x should be ~0.0, got {model.nodes[0][0]}"
    assert abs(model.nodes[0][1] - 1.0) < 1e-10, f"Node 0 y should be ~1.0, got {model.nodes[0][1]}"


def test_duplicate_element_block():
    """Test duplicating element blocks."""
    model = ExodusModel()

    # Create a block
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]])
    model.create_element_block(1, ['quad4', 1, 4, 0], [[1, 2, 3, 4]])

    original_node_count = len(model.nodes)

    # Duplicate with new nodes
    model.duplicate_element_block(1, 2, duplicate_nodes=True)

    assert model.element_block_exists(2), "Element block 2 should exist"
    assert len(model.nodes) == original_node_count + 4, "Should have 4 new nodes"

    # Duplicate without new nodes
    model.duplicate_element_block(1, 3, duplicate_nodes=False)

    assert model.element_block_exists(3), "Element block 3 should exist"
    assert len(model.nodes) == original_node_count + 4, "Node count should not change"


def test_get_element_block_extents():
    """Test getting element block extents."""
    model = ExodusModel()

    # Create a block with known extents
    model.create_nodes([
        [0.0, 0.0, 0.0],
        [5.0, 0.0, 0.0],
        [5.0, 3.0, 0.0],
        [0.0, 3.0, 2.0]
    ])
    model.create_element_block(1, ['quad4', 1, 4, 0], [[1, 2, 3, 4]])

    extents = model.get_element_block_extents(1)

    assert len(extents) == 3, "Should have 3 dimensions"
    assert extents[0] == [0.0, 5.0], f"X extents should be [0.0, 5.0], got {extents[0]}"
    assert extents[1] == [0.0, 3.0], f"Y extents should be [0.0, 3.0], got {extents[1]}"
    assert extents[2] == [0.0, 2.0], f"Z extents should be [0.0, 2.0], got {extents[2]}"


def test_combined_operations():
    """Test combined operations: duplicate, translate, and scale."""
    model = ExodusModel()

    # Create a cube
    nodes = [
        [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]
    ]
    model.create_nodes(nodes)
    model.create_element_block(1, ['hex8', 1, 8, 0], [[1, 2, 3, 4, 5, 6, 7, 8]])

    # Duplicate
    model.duplicate_element_block(1, 2, duplicate_nodes=True)

    # Translate the duplicate
    model.translate_element_blocks(2, [2.0, 0.0, 0.0])

    # Scale the duplicate
    model.scale_element_blocks(2, 0.5)

    # Check extents of both blocks
    extents1 = model.get_element_block_extents(1)
    extents2 = model.get_element_block_extents(2)

    # Block 1 should still be unit cube
    assert extents1[0] == [0.0, 1.0], "Block 1 X extents incorrect"

    # Block 2 should be translated and scaled
    # Original (0,1) -> translate (+2) -> (2,3) -> scale (0.5) -> (1, 1.5)
    assert abs(extents2[0][0] - 1.0) < 1e-10, f"Block 2 min X should be 1.0, got {extents2[0][0]}"
    assert abs(extents2[0][1] - 1.5) < 1e-10, f"Block 2 max X should be 1.5, got {extents2[0][1]}"
