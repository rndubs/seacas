#!/usr/bin/env python3
"""
Simple tests for Phases 4-6 that don't require the Rust extension module.
These tests verify the pure Python logic in exomerge.
"""

import sys
import os

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))


def test_phase4_node_operations():
    """Test Phase 4: Node operations."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.set_title("Node Operations Test")

    # Test create_nodes
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0], [1.0, 1.0, 0.0]])
    assert model.get_node_count() == 3
    assert model.nodes[1] == [1.0, 0.0, 0.0]  # 2D node expanded to 3D

    # Test delete_node
    model.create_nodes([[2.0, 2.0, 0.0]])
    assert model.get_node_count() == 4
    model.delete_node(3)  # Delete the 4th node (0-indexed)
    assert model.get_node_count() == 3

    # Test get_length_scale
    length_scale = model.get_length_scale()
    assert length_scale > 0

    # Test get_closest_node_distance
    min_dist = model.get_closest_node_distance()
    assert min_dist > 0

    print("✓ test_phase4_node_operations passed")


def test_phase4_merge_nodes():
    """Test Phase 4: Node merging."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Create nodes that are close together
    model.create_nodes([
        [0.0, 0.0, 0.0],
        [0.0000001, 0.0, 0.0],  # Very close to first node
        [1.0, 0.0, 0.0],
    ])

    initial_count = model.get_node_count()
    assert initial_count == 3

    # Merge nodes with tolerance
    model.merge_nodes(tolerance=0.001)

    # Should have merged the first two nodes
    assert model.get_node_count() == 2

    print("✓ test_phase4_merge_nodes passed")


def test_phase5_side_sets():
    """Test Phase 5: Side set operations."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Create side set
    model.create_side_set(1, [(1, 1), (2, 2)])
    assert model.side_set_exists(1)
    assert not model.side_set_exists(999)

    # Test get side set IDs
    ids = model.get_side_set_ids()
    assert 1 in ids

    # Test rename
    model.rename_side_set(1, "MySideSet")
    assert model.get_side_set_name(1) == "MySideSet"

    # Test get all names
    names = model.get_all_side_set_names()
    assert names[1] == "MySideSet"

    # Test members
    members = model.get_side_set_members(1)
    assert len(members) == 2

    # Test delete
    model.delete_side_set(1)
    assert not model.side_set_exists(1)

    print("✓ test_phase5_side_sets passed")


def test_phase5_node_sets():
    """Test Phase 5: Node set operations."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])

    # Create node set
    model.create_node_set(1, [1, 2])
    assert model.node_set_exists(1)
    assert not model.node_set_exists(999)

    # Test get node set IDs
    ids = model.get_node_set_ids()
    assert 1 in ids

    # Test rename
    model.rename_node_set(1, "MyNodeSet")
    assert model.get_node_set_name(1) == "MyNodeSet"

    # Test get all names
    names = model.get_all_node_set_names()
    assert names[1] == "MyNodeSet"

    # Test members
    members = model.get_node_set_members(1)
    assert len(members) == 2

    # Test add nodes
    model.add_nodes_to_node_set(1, [3])
    members = model.get_node_set_members(1)
    assert 3 in members

    # Test delete
    model.delete_node_set(1)
    assert not model.node_set_exists(1)

    print("✓ test_phase5_node_sets passed")


def test_phase6_element_fields():
    """Test Phase 6: Element field operations."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.element_blocks[1] = ["Block1", ["HEX8", 10, 8, 0], [], {}]

    # Test create field
    model.create_element_field("stress", 1, 100.0)
    assert model.element_field_exists("stress", 1)

    # Test get field names
    names = model.get_element_field_names(1)
    assert "stress" in names

    # Test get field values
    values = model.get_element_field_values("stress", 1)
    assert len(values) == 10
    assert values[0] == 100.0

    # Test rename field
    model.rename_element_field("stress", "stress_new", 1)
    assert model.element_field_exists("stress_new", 1)
    assert not model.element_field_exists("stress", 1)

    # Test delete field
    model.delete_element_field("stress_new", 1)
    assert not model.element_field_exists("stress_new", 1)

    print("✓ test_phase6_element_fields passed")


def test_phase6_node_fields():
    """Test Phase 6: Node field operations."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])

    # Test create field
    model.create_node_field("temperature", 25.0)
    assert model.node_field_exists("temperature")

    # Test get field names
    names = model.get_node_field_names()
    assert "temperature" in names

    # Test get field values
    values = model.get_node_field_values("temperature")
    assert len(values) == 2
    assert values[0] == 25.0

    # Test rename field
    model.rename_node_field("temperature", "temp")
    assert model.node_field_exists("temp")
    assert not model.node_field_exists("temperature")

    # Test delete field
    model.delete_node_field("temp")
    assert not model.node_field_exists("temp")

    print("✓ test_phase6_node_fields passed")


def test_phase6_global_variables():
    """Test Phase 6: Global variable operations."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Test create variable
    model.create_global_variable("timestep", 0.01)
    assert model.global_variable_exists("timestep")

    # Test get variable names
    names = model.get_global_variable_names()
    assert "timestep" in names

    # Test rename variable
    model.rename_global_variable("timestep", "dt")
    assert model.global_variable_exists("dt")
    assert not model.global_variable_exists("timestep")

    # Test delete variable
    model.delete_global_variable("dt")
    assert not model.global_variable_exists("dt")

    print("✓ test_phase6_global_variables passed")


def test_phase6_side_set_fields():
    """Test Phase 6: Side set field operations."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.create_side_set(1, [(1, 1), (2, 2)])

    # Test create field
    model.create_side_set_field("pressure", 1, 50.0)
    assert model.side_set_field_exists("pressure", 1)

    # Test get field names
    names = model.get_side_set_field_names(1)
    assert "pressure" in names

    # Test get field values
    values = model.get_side_set_field_values("pressure", 1)
    assert len(values) == 2
    assert values[0] == 50.0

    # Test rename field
    model.rename_side_set_field("pressure", "press", 1)
    assert model.side_set_field_exists("press", 1)
    assert not model.side_set_field_exists("pressure", 1)

    # Test delete field
    model.delete_side_set_field("press", 1)
    assert not model.side_set_field_exists("press", 1)

    print("✓ test_phase6_side_set_fields passed")


def test_phase6_node_set_fields():
    """Test Phase 6: Node set field operations."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])
    model.create_node_set(1, [1, 2])

    # Test create field
    model.create_node_set_field("velocity", 1, 10.0)
    assert model.node_set_field_exists("velocity", 1)

    # Test get field names
    names = model.get_node_set_field_names(1)
    assert "velocity" in names

    # Test get field values
    values = model.get_node_set_field_values("velocity", 1)
    assert len(values) == 2
    assert values[0] == 10.0

    # Test rename field
    model.rename_node_set_field("velocity", "vel", 1)
    assert model.node_set_field_exists("vel", 1)
    assert not model.node_set_field_exists("velocity", 1)

    # Test delete field
    model.delete_node_set_field("vel", 1)
    assert not model.node_set_field_exists("vel", 1)

    print("✓ test_phase6_node_set_fields passed")


def test_delete_empty_sets():
    """Test delete_empty_side_sets and delete_empty_node_sets."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Create empty and non-empty side sets
    model.create_side_set(1, [])  # Empty
    model.create_side_set(2, [(1, 1)])  # Non-empty

    # Create empty and non-empty node sets
    model.create_node_set(10, [])  # Empty
    model.create_node_set(20, [1, 2])  # Non-empty

    # Delete empty sets
    model.delete_empty_side_sets()
    model.delete_empty_node_sets()

    # Check results
    assert not model.side_set_exists(1)  # Empty was deleted
    assert model.side_set_exists(2)  # Non-empty remains

    assert not model.node_set_exists(10)  # Empty was deleted
    assert model.node_set_exists(20)  # Non-empty remains

    print("✓ test_delete_empty_sets passed")


def test_delete_unused_nodes():
    """Test delete_unused_nodes functionality."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.create_nodes([
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
    ])

    # Create element block that uses only some nodes
    model.element_blocks[1] = [
        "Block1",
        ["LINE2", 1, 2, 0],
        [[1, 2]],  # Only uses nodes 1 and 2 (1-indexed)
        {}
    ]

    # Delete unused nodes (nodes 3 and 4 should be deleted)
    model.delete_unused_nodes()

    # Should only have 2 nodes left
    assert model.get_node_count() == 2

    print("✓ test_delete_unused_nodes passed")


def main():
    """Run all tests."""
    tests = [
        test_phase4_node_operations,
        test_phase4_merge_nodes,
        test_phase5_side_sets,
        test_phase5_node_sets,
        test_phase6_element_fields,
        test_phase6_node_fields,
        test_phase6_global_variables,
        test_phase6_side_set_fields,
        test_phase6_node_set_fields,
        test_delete_empty_sets,
        test_delete_unused_nodes,
    ]

    print("Running Phases 4-6 tests...")
    print("=" * 60)

    passed = 0
    failed = 0

    for test in tests:
        try:
            test()
            passed += 1
        except AssertionError as e:
            import traceback
            print(f"✗ {test.__name__}: Assertion failed: {e}")
            traceback.print_exc()
            failed += 1
        except Exception as e:
            import traceback
            print(f"✗ {test.__name__}: {type(e).__name__}: {e}")
            traceback.print_exc()
            failed += 1

    print("=" * 60)
    print(f"Results: {passed} passed, {failed} failed")

    return 0 if failed == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
