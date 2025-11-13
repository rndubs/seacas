#!/usr/bin/env python3
"""
Tests for Phases 10 and 11 of exodus.exomerge module.
These tests verify geometry operations and utility/helper methods.
"""

import sys
import os

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))


def test_phase10_translate_geometry():
    """Test Phase 10: translate_geometry."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Create some nodes
    model.create_nodes([
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
    ])

    # Translate by [10, 20, 30]
    model.translate_geometry([10.0, 20.0, 30.0])

    # Check that all nodes were translated
    assert model.nodes[0] == [10.0, 20.0, 30.0]
    assert model.nodes[1] == [11.0, 20.0, 30.0]
    assert model.nodes[2] == [11.0, 21.0, 30.0]

    print("✓ test_phase10_translate_geometry passed")


def test_phase10_scale_geometry():
    """Test Phase 10: scale_geometry."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Create some nodes
    model.create_nodes([
        [1.0, 2.0, 3.0],
        [2.0, 4.0, 6.0],
    ])

    # Scale by 2.0
    model.scale_geometry(2.0)

    # Check that all nodes were scaled
    assert model.nodes[0] == [2.0, 4.0, 6.0]
    assert model.nodes[1] == [4.0, 8.0, 12.0]

    print("✓ test_phase10_scale_geometry passed")


def test_phase10_rotate_geometry():
    """Test Phase 10: rotate_geometry."""
    import exodus.exomerge as exomerge
    import math

    model = exomerge.ExodusModel()

    # Create a simple node at [1, 0, 0]
    model.create_nodes([[1.0, 0.0, 0.0]])

    # Rotate 90 degrees around Z-axis
    model.rotate_geometry([0.0, 0.0, 1.0], 90.0)

    # After 90° rotation around Z, [1,0,0] should become approximately [0,1,0]
    x, y, z = model.nodes[0]
    assert abs(x - 0.0) < 1e-10
    assert abs(y - 1.0) < 1e-10
    assert abs(z - 0.0) < 1e-10

    print("✓ test_phase10_rotate_geometry passed")


def test_phase10_rotate_geometry_with_displacement_field():
    """Test Phase 10: rotate_geometry with displacement field adjustment."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Create nodes
    model.create_nodes([
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    ])

    # Create a displacement field with 3 components
    model.node_fields["displacement"] = [
        [[0.1, 0.0, 0.0], [0.0, 0.1, 0.0]]  # timestep 0
    ]
    model.timesteps = [0.0]

    # Rotate 90 degrees around Z-axis, adjusting displacement
    model.rotate_geometry([0.0, 0.0, 1.0], 90.0, adjust_displacement_field=True)

    # Check that displacement field was rotated
    disp = model.node_fields["displacement"][0]
    # Original [0.1, 0.0, 0.0] should become approximately [0.0, 0.1, 0.0]
    assert abs(disp[0][0] - 0.0) < 1e-10
    assert abs(disp[0][1] - 0.1) < 1e-10

    print("✓ test_phase10_rotate_geometry_with_displacement_field passed")


def test_phase10_to_lowercase():
    """Test Phase 10: to_lowercase."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Set title
    model.set_title("MY TEST MODEL")

    # Create element block with uppercase name
    model.element_blocks[1] = ["BLOCK_ONE", ["HEX8", 1, 8, 0], [], {}]

    # Create side set with uppercase name
    model.create_side_set(1, [(1, 1)])
    model.rename_side_set(1, "SIDE_SET_ONE")

    # Create node set with uppercase name
    model.create_node_set(1, [1])
    model.rename_node_set(1, "NODE_SET_ONE")

    # Create fields with uppercase names
    model.create_nodes([[0.0, 0.0, 0.0]])
    model.node_fields["TEMPERATURE"] = [[100.0]]
    model.element_blocks[1][3]["STRESS"] = [[50.0]]
    model.global_variables["TIME"] = [0.0]

    # Convert to lowercase
    model.to_lowercase()

    # Check that everything was converted
    assert model.get_title() == "my test model"
    assert model.element_blocks[1][0] == "block_one"
    assert model.get_side_set_name(1) == "side_set_one"
    assert model.get_node_set_name(1) == "node_set_one"
    assert "temperature" in model.node_fields
    assert "TEMPERATURE" not in model.node_fields
    assert "stress" in model.element_blocks[1][3]
    assert "STRESS" not in model.element_blocks[1][3]
    assert "time" in model.global_variables
    assert "TIME" not in model.global_variables

    print("✓ test_phase10_to_lowercase passed")


def test_phase11_summarize():
    """Test Phase 11: summarize."""
    import exodus.exomerge as exomerge
    import io
    import sys

    model = exomerge.ExodusModel()
    model.set_title("Test Summary Model")

    # Create a simple model
    model.create_nodes([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])
    model.element_blocks[1] = ["Block1", ["LINE2", 1, 2, 0], [[1, 2]], {}]
    model.create_side_set(1, [(1, 1)])
    model.create_node_set(1, [1, 2])
    model.create_node_field("temp", 25.0)
    model.create_element_field("stress", 1, 100.0)
    model.create_global_variable("time", 0.0)

    # Capture output
    old_stdout = sys.stdout
    sys.stdout = captured_output = io.StringIO()

    try:
        model.summarize()
        output = captured_output.getvalue()

        # Check that summary contains expected information
        assert "Test Summary Model" in output
        assert "Nodes: 2" in output
        assert "Element Blocks: 1" in output
        assert "Side Sets: 1" in output
        assert "Node Sets: 1" in output
        assert "temp" in output
        assert "stress" in output
        assert "time" in output

    finally:
        sys.stdout = old_stdout

    print("✓ test_phase11_summarize passed")


def test_phase11_build_hex8_cube_basic():
    """Test Phase 11: build_hex8_cube with basic parameters."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Build a simple 2x2x2 cube
    model.build_hex8_cube(element_block_id=1, extents=1.0, divisions=2)

    # Should have (2+1)^3 = 27 nodes
    assert model.get_node_count() == 27

    # Should have 2^3 = 8 elements
    assert 1 in model.element_blocks
    block_info = model.element_blocks[1][1]
    assert block_info[1] == 8  # num_elements

    # Check element type is HEX8
    assert block_info[0] == "HEX8"

    # Check connectivity length
    connectivity = model.element_blocks[1][2]
    assert len(connectivity) == 8

    # Each element should have 8 nodes
    for elem_conn in connectivity:
        assert len(elem_conn) == 8

    print("✓ test_phase11_build_hex8_cube_basic passed")


def test_phase11_build_hex8_cube_non_uniform():
    """Test Phase 11: build_hex8_cube with non-uniform divisions."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Build a cube with different divisions in each direction
    model.build_hex8_cube(
        element_block_id=1,
        extents=[2.0, 3.0, 4.0],
        divisions=[2, 3, 4]
    )

    # Should have (2+1) * (3+1) * (4+1) = 3 * 4 * 5 = 60 nodes
    assert model.get_node_count() == 60

    # Should have 2 * 3 * 4 = 24 elements
    assert 1 in model.element_blocks
    block_info = model.element_blocks[1][1]
    assert block_info[1] == 24

    # Check node coordinates
    nodes = model.nodes

    # Check first node (corner at origin)
    assert nodes[0] == [0.0, 0.0, 0.0]

    # Check last node (opposite corner)
    assert nodes[-1] == [2.0, 3.0, 4.0]

    print("✓ test_phase11_build_hex8_cube_non_uniform passed")


def test_phase11_build_hex8_cube_connectivity():
    """Test Phase 11: build_hex8_cube connectivity is correct."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Build a simple 1x1x1 cube (1 element)
    model.build_hex8_cube(element_block_id=1, extents=1.0, divisions=1)

    # Should have 8 nodes (corners of a cube)
    assert model.get_node_count() == 8

    # Should have 1 element
    connectivity = model.element_blocks[1][2]
    assert len(connectivity) == 1

    # Check the single element's connectivity
    elem_conn = connectivity[0]
    assert len(elem_conn) == 8

    # Verify all node IDs are in valid range (1-8 for 1-based indexing)
    for node_id in elem_conn:
        assert 1 <= node_id <= 8

    # Verify all nodes are unique in the connectivity
    assert len(set(elem_conn)) == 8

    print("✓ test_phase11_build_hex8_cube_connectivity passed")


def test_phase11_build_hex8_cube_auto_block_id():
    """Test Phase 11: build_hex8_cube with auto block ID."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Create an existing block
    model.element_blocks[1] = ["ExistingBlock", ["HEX8", 1, 8, 0], [[1,2,3,4,5,6,7,8]], {}]

    # Build cube with auto block ID
    model.build_hex8_cube(element_block_id="auto", extents=1.0, divisions=1)

    # Should have created block with ID 2 (next available)
    assert 2 in model.element_blocks
    assert model.element_blocks[2][0] == "HEX8_Cube"

    print("✓ test_phase11_build_hex8_cube_auto_block_id passed")


def test_integration_geometry_operations():
    """Integration test: multiple geometry operations in sequence."""
    import exodus.exomerge as exomerge
    import math

    model = exomerge.ExodusModel()

    # Build a cube
    model.build_hex8_cube(element_block_id=1, extents=1.0, divisions=2)

    initial_node_count = model.get_node_count()

    # Translate
    model.translate_geometry([5.0, 0.0, 0.0])

    # Scale
    model.scale_geometry(2.0)

    # Rotate 90 degrees around Z
    model.rotate_geometry([0.0, 0.0, 1.0], 90.0)

    # Node count should remain the same
    assert model.get_node_count() == initial_node_count

    # Check that nodes have been transformed
    # After translate [5,0,0], scale 2x, and rotate 90° around Z,
    # the origin corner [0,0,0] -> [5,0,0] -> [10,0,0] -> [0,10,0]
    # Find the node closest to [0,10,0]
    found_transformed_corner = False
    for node in model.nodes:
        if abs(node[0] - 0.0) < 0.1 and abs(node[1] - 10.0) < 0.1 and abs(node[2] - 0.0) < 0.1:
            found_transformed_corner = True
            break

    assert found_transformed_corner, "Expected to find transformed corner node near [0,10,0]"

    print("✓ test_integration_geometry_operations passed")


def main():
    """Run all tests."""
    tests = [
        test_phase10_translate_geometry,
        test_phase10_scale_geometry,
        test_phase10_rotate_geometry,
        test_phase10_rotate_geometry_with_displacement_field,
        test_phase10_to_lowercase,
        test_phase11_summarize,
        test_phase11_build_hex8_cube_basic,
        test_phase11_build_hex8_cube_non_uniform,
        test_phase11_build_hex8_cube_connectivity,
        test_phase11_build_hex8_cube_auto_block_id,
        test_integration_geometry_operations,
    ]

    print("Running Phases 10-11 tests...")
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
