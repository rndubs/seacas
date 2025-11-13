#!/usr/bin/env python3
"""
Tests for Phases 7, 8, and 9 of exodus.exomerge module.
These tests verify advanced set operations, timestep operations, and metadata operations.
"""

import sys
import os

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))


def test_phase7_get_nodes_in_side_set():
    """Test Phase 7: get_nodes_in_side_set."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Create some nodes
    model.create_nodes([
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ])

    # Create an element block with one quad element
    model.element_blocks[1] = [
        "Block1",
        ["QUAD4", 1, 4, 0],  # 1 quad element with 4 nodes
        [[1, 2, 3, 4]],  # connectivity (1-indexed)
        {}
    ]

    # Create a side set referencing the element
    model.create_side_set(1, [(1, 1)])  # Element 1, face 1

    # Get nodes in side set
    nodes = model.get_nodes_in_side_set(1)

    # Should contain all 4 nodes from the element
    assert len(nodes) == 4
    assert nodes == [1, 2, 3, 4]

    print("✓ test_phase7_get_nodes_in_side_set passed")


def test_phase7_get_nodes_in_node_set():
    """Test Phase 7: get_nodes_in_node_set (alias)."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])

    # Create a node set
    model.create_node_set(1, [1, 2])

    # Test the alias method
    nodes = model.get_nodes_in_node_set(1)
    assert len(nodes) == 2
    assert 1 in nodes
    assert 2 in nodes

    # Verify it's the same as get_node_set_members
    assert nodes == model.get_node_set_members(1)

    print("✓ test_phase7_get_nodes_in_node_set passed")


def test_phase8_create_timestep():
    """Test Phase 8: create_timestep."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])

    # Create a node field
    model.create_node_field("temp", 25.0)

    # Initially should have 1 timestep of data
    assert len(model.node_fields["temp"]) == 1

    # Create a new timestep
    model.create_timestep(1.0)

    # Should now have 2 timesteps
    assert len(model.timesteps) == 1
    assert 1.0 in model.timesteps

    # Node field should have been extended
    assert len(model.node_fields["temp"]) == 2

    # New timestep data should be zeros
    assert model.node_fields["temp"][1] == [0.0, 0.0]

    print("✓ test_phase8_create_timestep passed")


def test_phase8_delete_timestep():
    """Test Phase 8: delete_timestep."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.create_nodes([[0.0, 0.0, 0.0]])

    # Create timesteps manually
    model.timesteps = [0.0, 1.0, 2.0]

    # Create a node field with data for each timestep
    model.node_fields["temp"] = [
        [10.0],  # t=0
        [20.0],  # t=1
        [30.0],  # t=2
    ]

    # Delete middle timestep
    model.delete_timestep(1.0)

    # Should have 2 timesteps now
    assert len(model.timesteps) == 2
    assert model.timesteps == [0.0, 2.0]

    # Field data should be updated
    assert len(model.node_fields["temp"]) == 2
    assert model.node_fields["temp"][0] == [10.0]
    assert model.node_fields["temp"][1] == [30.0]

    print("✓ test_phase8_delete_timestep passed")


def test_phase8_copy_timestep():
    """Test Phase 8: copy_timestep."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])

    # Create timesteps
    model.timesteps = [0.0, 1.0]

    # Create a node field with different data for each timestep
    model.node_fields["temp"] = [
        [10.0, 15.0],  # t=0
        [20.0, 25.0],  # t=1
    ]

    # Create a global variable
    model.global_variables["time"] = [0.0, 1.0]

    # Copy timestep 1.0 to 2.0
    model.copy_timestep(1.0, 2.0)

    # Should have 3 timesteps now
    assert len(model.timesteps) == 3
    assert 2.0 in model.timesteps

    # Field data should be copied
    assert len(model.node_fields["temp"]) == 3
    # The copied data should match the source
    assert model.node_fields["temp"][2] == [20.0, 25.0]

    # Global variable should be copied
    assert len(model.global_variables["time"]) == 3
    assert model.global_variables["time"][2] == 1.0

    print("✓ test_phase8_copy_timestep passed")


def test_phase8_timestep_exists_get_timesteps():
    """Test Phase 8: timestep_exists and get_timesteps."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Create some timesteps
    model.create_timestep(0.0)
    model.create_timestep(1.0)
    model.create_timestep(2.0)

    # Test timestep_exists
    assert model.timestep_exists(0.0)
    assert model.timestep_exists(1.0)
    assert model.timestep_exists(2.0)
    assert not model.timestep_exists(3.0)

    # Test get_timesteps
    timesteps = model.get_timesteps()
    assert len(timesteps) == 3
    assert timesteps == [0.0, 1.0, 2.0]

    print("✓ test_phase8_timestep_exists_get_timesteps passed")


def test_phase9_add_qa_record():
    """Test Phase 9: add_qa_record."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Add a QA record with all parameters
    model.add_qa_record("MyCode", "1.0", "2024/01/15", "10:30:00")

    # Check the record was added
    qa_records = model.get_qa_records()
    assert len(qa_records) == 1
    assert qa_records[0] == ("MyCode", "1.0", "2024/01/15", "10:30:00")

    # Add another QA record with defaults
    model.add_qa_record()

    # Should have 2 records now
    qa_records = model.get_qa_records()
    assert len(qa_records) == 2

    # Second record should have default code name and version
    assert qa_records[1][0] == "exodus.exomerge"
    assert qa_records[1][1] == exomerge.__version__
    # Date and time fields should be populated (check they're not None)
    assert qa_records[1][2] is not None
    assert qa_records[1][3] is not None

    print("✓ test_phase9_add_qa_record passed")


def test_phase9_metadata_operations():
    """Test Phase 9: title and info records."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Test title operations
    model.set_title("Test Model")
    assert model.get_title() == "Test Model"

    # Test info records
    model.add_info_record("First info record")
    model.add_info_record("Second info record")

    info_records = model.get_info_records()
    assert len(info_records) == 2
    assert info_records[0] == "First info record"
    assert info_records[1] == "Second info record"

    print("✓ test_phase9_metadata_operations passed")


def test_integration_timesteps_with_all_field_types():
    """Integration test: timestep operations with all field types."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Set up model with all field types
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])
    model.element_blocks[1] = ["Block1", ["LINE2", 1, 2, 0], [[1, 2]], {}]
    model.create_side_set(1, [(1, 1)])
    model.create_node_set(1, [1, 2])

    # Create fields of all types
    model.create_node_field("node_temp", 10.0)
    model.create_element_field("elem_stress", 1, 100.0)
    model.create_side_set_field("side_pressure", 1, 50.0)
    model.create_node_set_field("ns_velocity", 1, 5.0)
    model.create_global_variable("time", 0.0)

    # Initially 1 timestep worth of data
    assert len(model.node_fields["node_temp"]) == 1
    assert len(model.element_blocks[1][3]["elem_stress"]) == 1
    assert len(model.side_sets[1][2]["side_pressure"]) == 1
    assert len(model.node_sets[1][2]["ns_velocity"]) == 1
    assert len(model.global_variables["time"]) == 1

    # Create a new timestep
    model.create_timestep(1.0)

    # All fields should be extended
    assert len(model.node_fields["node_temp"]) == 2
    assert len(model.element_blocks[1][3]["elem_stress"]) == 2
    assert len(model.side_sets[1][2]["side_pressure"]) == 2
    assert len(model.node_sets[1][2]["ns_velocity"]) == 2
    assert len(model.global_variables["time"]) == 2

    # Delete the first timestep
    model.timesteps.insert(0, 0.0)  # Add 0.0 back manually
    model.delete_timestep(0.0)

    # Should be back to 1 timestep
    assert len(model.timesteps) == 1
    assert len(model.node_fields["node_temp"]) == 1

    print("✓ test_integration_timesteps_with_all_field_types passed")


def main():
    """Run all tests."""
    tests = [
        test_phase7_get_nodes_in_side_set,
        test_phase7_get_nodes_in_node_set,
        test_phase8_create_timestep,
        test_phase8_delete_timestep,
        test_phase8_copy_timestep,
        test_phase8_timestep_exists_get_timesteps,
        test_phase9_add_qa_record,
        test_phase9_metadata_operations,
        test_integration_timesteps_with_all_field_types,
    ]

    print("Running Phases 7-9 tests...")
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
