#!/usr/bin/env python3
"""
Test script for element block operations in exomerge module.
Tests: create_element_block, delete_element_block, rename_element_block, get_nodes_in_element_block
"""

import sys
sys.path.insert(0, 'python')

from exodus.exomerge import ExodusModel

def test_create_element_block():
    """Test creating element blocks."""
    print("Testing create_element_block...")
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
    # HEX8 element: 1 element, 8 nodes per element
    connectivity = [[1, 2, 3, 4, 5, 6, 7, 8]]  # 1-based indexing
    model.create_element_block(1, ['hex8', 1, 8, 0], connectivity)

    assert model.element_block_exists(1), "Element block 1 should exist"
    assert len(model.get_element_block_ids()) == 1, "Should have 1 element block"
    print("✓ create_element_block passed")

    return model

def test_get_nodes_in_element_block(model):
    """Test getting nodes from element blocks."""
    print("\nTesting get_nodes_in_element_block...")

    nodes = model.get_nodes_in_element_block(1)
    assert len(nodes) == 8, f"Should have 8 nodes, got {len(nodes)}"
    assert sorted(nodes) == [1, 2, 3, 4, 5, 6, 7, 8], f"Nodes should be 1-8, got {nodes}"

    # Test "all" option
    all_nodes = model.get_nodes_in_element_block("all")
    assert all_nodes == nodes, "All nodes should match block 1 nodes"

    print("✓ get_nodes_in_element_block passed")

def test_rename_element_block(model):
    """Test renaming element blocks."""
    print("\nTesting rename_element_block...")

    # Rename ID from 1 to 100
    model.rename_element_block(1, 100)
    assert not model.element_block_exists(1), "Element block 1 should not exist"
    assert model.element_block_exists(100), "Element block 100 should exist"

    # Rename back and set a name
    model.rename_element_block(100, 1)
    model.rename_element_block(1, "my_block")

    name = model.get_element_block_name(1)
    assert name == "my_block", f"Block name should be 'my_block', got '{name}'"

    print("✓ rename_element_block passed")

def test_delete_element_block(model):
    """Test deleting element blocks."""
    print("\nTesting delete_element_block...")

    # Create another element block first
    model.create_element_block(2, ['hex8', 0, 8, 0], [])

    initial_count = len(model.get_element_block_ids())
    model.delete_element_block(2)

    assert len(model.get_element_block_ids()) == initial_count - 1, "Should have one less block"
    assert not model.element_block_exists(2), "Element block 2 should not exist"
    assert model.element_block_exists(1), "Element block 1 should still exist"

    print("✓ delete_element_block passed")

def main():
    """Run all tests."""
    print("="*60)
    print("Testing Element Block Operations")
    print("="*60)

    try:
        model = test_create_element_block()
        test_get_nodes_in_element_block(model)
        test_rename_element_block(model)
        test_delete_element_block(model)

        print("\n" + "="*60)
        print("All tests passed! ✓")
        print("="*60)
        return 0
    except AssertionError as e:
        print(f"\n✗ Test failed: {e}")
        return 1
    except Exception as e:
        print(f"\n✗ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())
