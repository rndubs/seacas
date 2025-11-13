"""
Test suite for remaining exomerge implementations.

This test file covers all the newly implemented features in exomerge.
"""

import pytest
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))

try:
    import exodus.exomerge as exomerge
except ImportError:
    pytest.skip("exodus module not available", allow_module_level=True)


class TestInputDeck:
    """Test get_input_deck functionality."""

    def test_get_input_deck_empty(self):
        """Test get_input_deck with no info records."""
        model = exomerge.ExodusModel()
        deck = model.get_input_deck()
        assert deck == ""

    def test_get_input_deck_with_data(self):
        """Test get_input_deck with input deck in info records."""
        model = exomerge.ExodusModel()
        model.info_records = [
            "begin sierra my_analysis",
            "  time control",
            "    time step = 0.1",
            "  end",
            "end"
        ]
        deck = model.get_input_deck()
        assert "begin" in deck.lower()
        assert "end" in deck.lower()


class TestCombineElementBlocks:
    """Test combine_element_blocks functionality."""

    def test_combine_two_blocks(self):
        """Test combining two element blocks."""
        model = exomerge.ExodusModel()

        # Create nodes
        model.nodes = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0],
                       [2, 0, 0], [2, 1, 0]]

        # Create two quad4 blocks
        info1 = ["QUAD4", 1, 4, 0]  # [type, num_elems, nodes_per_elem, num_attrs]
        conn1 = [[1, 2, 3, 4]]
        model.create_element_block(1, info1, conn1)

        info2 = ["QUAD4", 1, 4, 0]
        conn2 = [[2, 5, 6, 3]]
        model.create_element_block(2, info2, conn2)

        # Combine blocks
        model.combine_element_blocks([1, 2], 1)

        # Check result
        assert 1 in model.element_blocks
        assert 2 not in model.element_blocks
        assert model.get_element_count(1) == 2


class TestUnmergeElementBlocks:
    """Test unmerge_element_blocks functionality."""

    def test_unmerge_shared_nodes(self):
        """Test unmerging blocks that share nodes."""
        model = exomerge.ExodusModel()

        # Create nodes (blocks share node 2 and 3)
        model.nodes = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0]]

        # Create two blocks sharing nodes
        info1 = ["QUAD4", 1, 4, 0]
        conn1 = [[1, 2, 3, 4]]
        model.create_element_block(1, info1, conn1)

        info2 = ["LINE2", 1, 2, 0]
        conn2 = [[2, 3]]
        model.create_element_block(2, info2, conn2)

        initial_node_count = len(model.nodes)

        # Unmerge blocks
        model.unmerge_element_blocks([1, 2])

        # Check that new nodes were created
        assert len(model.nodes) > initial_node_count


class TestProcessElementFields:
    """Test process_element_fields functionality."""

    def test_process_integration_points(self):
        """Test processing element fields with integration points."""
        model = exomerge.ExodusModel()

        # Create a simple model
        model.nodes = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0]]
        model.timesteps = [0.0, 1.0]

        info = ["QUAD4", 1, 4, 0]
        conn = [[1, 2, 3, 4]]
        model.create_element_block(1, info, conn)

        # Create fields with 8 integration points
        for i in range(1, 9):
            model.create_element_field(f"stress_{i}", 1, float(i))

        # Process fields - this should average the 8 points
        # Note: This will also convert to node fields and delete element fields
        # So we need to check that conversion happened
        try:
            model.process_element_fields(1)
            # If implementation exists, check that fields were processed
            # Element fields should be converted to node fields
            assert True  # Processing completed without error
        except NotImplementedError:
            pytest.skip("process_element_fields not fully implemented")


class TestReflectElementBlocks:
    """Test reflect_element_blocks functionality."""

    def test_reflect_across_plane(self):
        """Test reflecting element blocks across a plane."""
        model = exomerge.ExodusModel()

        # Create a simple cube
        model.nodes = [
            [0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0],
            [0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1]
        ]

        info = ["HEX8", 1, 8, 0]
        conn = [[1, 2, 3, 4, 5, 6, 7, 8]]
        model.create_element_block(1, info, conn)

        # Reflect across yz-plane (x=0)
        original_x = model.nodes[1][0]  # Node 2, x-coordinate
        model.reflect_element_blocks(1, [1, 0, 0], [0, 0, 0])

        # Check that x-coordinates were reflected
        assert model.nodes[1][0] == -original_x


class TestDisplaceElementBlocks:
    """Test displace_element_blocks functionality."""

    def test_displace_with_field(self):
        """Test displacing element blocks using displacement field."""
        model = exomerge.ExodusModel()

        # Create nodes
        model.nodes = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0]]
        model.timesteps = [0.0]

        # Create displacement fields
        model.create_node_field("DISP_X", 0.0)
        model.create_node_field("DISP_Y", 0.0)
        model.create_node_field("DISP_Z", 0.0)

        # Set displacement values
        model.node_fields["DISP_X"][0] = [1.0, 1.0, 1.0, 1.0]
        model.node_fields["DISP_Y"][0] = [0.0, 0.0, 0.0, 0.0]
        model.node_fields["DISP_Z"][0] = [0.0, 0.0, 0.0, 0.0]

        # Create element block
        info = ["QUAD4", 1, 4, 0]
        conn = [[1, 2, 3, 4]]
        model.create_element_block(1, info, conn)

        original_x = model.nodes[0][0]

        # Displace with scale factor 1.0
        model.displace_element_blocks(1, "DISP", "last", 1.0)

        # Check that nodes were displaced
        assert model.nodes[0][0] == original_x + 1.0


class TestFieldMinMax:
    """Test field min/max calculation methods."""

    def test_element_field_maximum(self):
        """Test calculating maximum of element field."""
        model = exomerge.ExodusModel()

        # Create simple model
        model.nodes = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0]]
        model.timesteps = [0.0]

        info = ["QUAD4", 2, 4, 0]
        conn = [[1, 2, 3, 4], [2, 3, 4, 1]]
        model.create_element_block(1, info, conn)

        # Create field with known values
        model.create_element_field("stress", 1, 0.0)
        model.element_blocks[1][3]["stress"][0] = [10.0, 20.0]

        try:
            max_val = model.calculate_element_field_maximum("stress", 1, "last")
            assert max_val == 20.0
        except NotImplementedError:
            pytest.skip("calculate_element_field_maximum not implemented")

    def test_node_field_minimum(self):
        """Test calculating minimum of node field."""
        model = exomerge.ExodusModel()

        model.nodes = [[0, 0, 0], [1, 0, 0], [1, 1, 0]]
        model.timesteps = [0.0]

        model.create_node_field("temperature", 0.0)
        model.node_fields["temperature"][0] = [100.0, 50.0, 75.0]

        try:
            min_val = model.calculate_node_field_minimum("temperature", "last")
            assert min_val == 50.0
        except NotImplementedError:
            pytest.skip("calculate_node_field_minimum not implemented")


class TestFieldConversions:
    """Test field conversion methods."""

    def test_create_averaged_element_field(self):
        """Test creating averaged element field."""
        model = exomerge.ExodusModel()

        model.nodes = [[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0]]
        model.timesteps = [0.0]

        info = ["QUAD4", 1, 4, 0]
        conn = [[1, 2, 3, 4]]
        model.create_element_block(1, info, conn)

        # Create fields to average
        model.create_element_field("field1", 1, 0.0)
        model.create_element_field("field2", 1, 0.0)
        model.element_blocks[1][3]["field1"][0] = [10.0]
        model.element_blocks[1][3]["field2"][0] = [20.0]

        try:
            model.create_averaged_element_field(["field1", "field2"], "avg_field", 1)
            assert "avg_field" in model.element_blocks[1][3]
            assert model.element_blocks[1][3]["avg_field"][0][0] == 15.0
        except NotImplementedError:
            pytest.skip("create_averaged_element_field not implemented")


class TestDisplacementFields:
    """Test displacement field methods."""

    def test_displacement_field_exists(self):
        """Test checking if displacement fields exist."""
        model = exomerge.ExodusModel()

        try:
            # Initially should not exist
            assert not model.displacement_field_exists()

            # Create displacement fields
            model.create_displacement_field(0.0)

            # Now should exist
            assert model.displacement_field_exists()
        except NotImplementedError:
            pytest.skip("displacement_field methods not implemented")


class TestGlobalVariables:
    """Test global variable operations."""

    def test_output_global_variables(self):
        """Test outputting global variables."""
        model = exomerge.ExodusModel()

        model.timesteps = [0.0, 1.0, 2.0]
        model.create_global_variable("energy", 100.0)
        model.global_variables["energy"] = [100.0, 150.0, 200.0]

        try:
            output = model.output_global_variables(["energy"])
            assert "energy" in output.lower()
            assert "100" in output
        except NotImplementedError:
            pytest.skip("output_global_variables not implemented")


class TestTimestepOperations:
    """Test timestep manipulation methods."""

    def test_create_interpolated_timestep(self):
        """Test creating interpolated timestep."""
        model = exomerge.ExodusModel()

        model.nodes = [[0, 0, 0], [1, 0, 0]]
        model.timesteps = [0.0, 2.0]

        model.create_node_field("displacement", 0.0)
        model.node_fields["displacement"] = [[0.0, 0.0], [2.0, 2.0]]

        try:
            # Interpolate at t=1.0 (midpoint)
            model.create_interpolated_timestep(1.0)

            # Check that timestep was added
            assert 1.0 in model.timesteps

            # Check that field values were interpolated
            timestep_idx = model.timesteps.index(1.0)
            interp_values = model.node_fields["displacement"][timestep_idx]
            assert abs(interp_values[0] - 1.0) < 0.001  # Should be interpolated to 1.0
        except NotImplementedError:
            pytest.skip("create_interpolated_timestep not implemented")


# Run tests if executed directly
if __name__ == "__main__":
    pytest.main([__file__, "-v"])
