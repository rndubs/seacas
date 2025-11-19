"""
Tests for spatial search utilities
"""

import pytest
import tempfile
import os

# Import from installed package after building
try:
    from exodus import ExodusReader, SpatialSearchResult
    EXODUS_AVAILABLE = True
except ImportError:
    EXODUS_AVAILABLE = False
    pytestmark = pytest.mark.skip("exodus module not built")


def create_test_mesh_with_variables():
    """Helper to create a test mesh with nodal and element variables."""
    temp_file = tempfile.NamedTemporaryFile(delete=False, suffix=".exo")
    temp_file.close()
    path = temp_file.name

    # Delete the temp file so ExodusWriter can create it
    os.unlink(path)

    from exodus import ExodusWriter, InitParams, Block, EntityType

    # Create file with ExodusWriter
    writer = ExodusWriter.create(path)

    # Initialize with 8 nodes, 1 element
    params = InitParams(
        title="Test Mesh",
        num_dim=3,
        num_nodes=8,
        num_elems=1,
        num_elem_blocks=1
    )
    writer.put_init_params(params)

    # Set coordinates - cube
    coords_x = [0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
    coords_y = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0]
    coords_z = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
    writer.put_coords(coords_x, coords_y, coords_z)

    # Define hex block
    block = Block(
        id=100,
        entity_type=EntityType.ElemBlock,
        topology="HEX8",
        num_entries=1,
        num_nodes_per_entry=8,
        num_edges_per_entry=0,
        num_faces_per_entry=0,
        num_attributes=0
    )
    writer.put_block(block)
    writer.put_connectivity(100, [1, 2, 3, 4, 5, 6, 7, 8])

    # Define variables
    writer.define_variables(EntityType.Nodal, ["temperature"])
    writer.define_variables(EntityType.ElemBlock, ["stress"])

    # Write time steps
    for step in range(5):
        writer.put_time(step, float(step) * 0.1)

        # Nodal variable: different values for each node
        nodal_values = [
            10.0 + step,  # Node 1 at (0,0,0)
            20.0 + step,  # Node 2 at (1,0,0)
            30.0 + step,  # Node 3 at (1,1,0)
            40.0 + step,  # Node 4 at (0,1,0)
            50.0 + step,  # Node 5 at (0,0,1)
            60.0 + step,  # Node 6 at (1,0,1)
            70.0 + step,  # Node 7 at (1,1,1)
            80.0 + step,  # Node 8 at (0,1,1)
        ]
        writer.put_var(step, EntityType.Nodal, 0, 0, nodal_values)

        # Element variable
        elem_values = [100.0 + step]
        writer.put_var(step, EntityType.ElemBlock, 100, 0, elem_values)

    writer.close()

    return path


class TestSpatialSearch:
    """Tests for spatial search functionality"""

    def test_average_element_size(self):
        """Test average element size computation"""
        path = create_test_mesh_with_variables()
        try:
            reader = ExodusReader.open(path)
            avg_size = reader.average_element_size()

            # Unit cube has volume 1.0, so cbrt(1.0) = 1.0
            assert abs(avg_size - 1.0) < 1e-10, f"Expected 1.0, got {avg_size}"

            reader.close()
        finally:
            os.unlink(path)

    def test_find_nearest_node(self):
        """Test finding nearest node"""
        path = create_test_mesh_with_variables()
        try:
            reader = ExodusReader.open(path)

            # Search near origin (should find node 1 at (0,0,0))
            node_id, distance = reader.find_nearest_node(0.1, 0.1, 0.0, -1.0)
            assert node_id == 1, f"Expected node 1, got {node_id}"
            expected_dist = (0.1**2 + 0.1**2) ** 0.5
            assert abs(distance - expected_dist) < 1e-10

            # Search near (1, 0, 0) (should find node 2)
            node_id, distance = reader.find_nearest_node(0.9, 0.1, 0.0, -1.0)
            assert node_id == 2, f"Expected node 2, got {node_id}"

            # Test distance limit - should fail for far point
            with pytest.raises(Exception):
                reader.find_nearest_node(10.0, 10.0, 10.0, 1.0)

            # Test distance limit - should succeed for nearby point
            node_id, distance = reader.find_nearest_node(0.5, 0.5, 0.0, 1.0)
            assert node_id in [1, 2, 3, 4]  # Should find one of the bottom nodes

            reader.close()
        finally:
            os.unlink(path)

    def test_find_nearest_element(self):
        """Test finding nearest element"""
        path = create_test_mesh_with_variables()
        try:
            reader = ExodusReader.open(path)

            # Search near centroid (0.5, 0.5, 0.5)
            elem_id, distance = reader.find_nearest_element(0.5, 0.5, 0.5, -1.0)
            assert elem_id == 1, f"Expected element 1, got {elem_id}"
            assert distance < 0.1, f"Distance should be very small, got {distance}"

            # Test distance limit
            with pytest.raises(Exception):
                reader.find_nearest_element(10.0, 10.0, 10.0, 1.0)

            reader.close()
        finally:
            os.unlink(path)

    def test_search_nodal_variable(self):
        """Test searching for nodal variable"""
        path = create_test_mesh_with_variables()
        try:
            reader = ExodusReader.open(path)

            # Search near node 1 (0, 0, 0)
            result = reader.search_nodal_variable(0.1, 0.1, 0.0, "temperature", -1.0)

            assert isinstance(result, SpatialSearchResult)
            assert result.id == 1, f"Expected node 1, got {result.id}"
            assert len(result.time_history) == 5, f"Expected 5 time steps, got {len(result.time_history)}"

            # Check time history values for node 1
            expected_values = [10.0, 11.0, 12.0, 13.0, 14.0]
            for i, (actual, expected) in enumerate(zip(result.time_history, expected_values)):
                assert abs(actual - expected) < 1e-10, \
                    f"Time step {i}: expected {expected}, got {actual}"

            # Search near node 3 (1, 1, 0)
            result = reader.search_nodal_variable(0.9, 0.9, 0.0, "temperature", -1.0)
            assert result.id == 3, f"Expected node 3, got {result.id}"
            expected_values = [30.0, 31.0, 32.0, 33.0, 34.0]
            for i, (actual, expected) in enumerate(zip(result.time_history, expected_values)):
                assert abs(actual - expected) < 1e-10

            # Test with default distance limit (None)
            result = reader.search_nodal_variable(0.1, 0.1, 0.0, "temperature", None)
            assert result.id == 1

            reader.close()
        finally:
            os.unlink(path)

    def test_search_element_variable(self):
        """Test searching for element variable"""
        path = create_test_mesh_with_variables()
        try:
            reader = ExodusReader.open(path)

            # Search near element centroid
            result = reader.search_element_variable(0.5, 0.5, 0.5, "stress", -1.0)

            assert isinstance(result, SpatialSearchResult)
            assert result.id == 1, f"Expected element 1, got {result.id}"
            assert len(result.time_history) == 5

            # Check time history values
            expected_values = [100.0, 101.0, 102.0, 103.0, 104.0]
            for i, (actual, expected) in enumerate(zip(result.time_history, expected_values)):
                assert abs(actual - expected) < 1e-10

            # Test with default distance limit
            result = reader.search_element_variable(0.5, 0.5, 0.5, "stress", None)
            assert result.id == 1

            reader.close()
        finally:
            os.unlink(path)

    def test_search_result_slice(self):
        """Test slicing time history by indices"""
        path = create_test_mesh_with_variables()
        try:
            reader = ExodusReader.open(path)

            result = reader.search_nodal_variable(0.1, 0.1, 0.0, "temperature", -1.0)

            # Test basic slicing: get steps 1-3
            sliced = result.slice(1, 3, 1)
            assert len(sliced.time_history) == 2
            assert abs(sliced.time_history[0] - 11.0) < 1e-10
            assert abs(sliced.time_history[1] - 12.0) < 1e-10
            assert sliced.id == result.id
            assert sliced.distance == result.distance

            # Test slicing with step=2
            sliced = result.slice(0, 5, 2)
            assert len(sliced.time_history) == 3
            assert abs(sliced.time_history[0] - 10.0) < 1e-10
            assert abs(sliced.time_history[1] - 12.0) < 1e-10
            assert abs(sliced.time_history[2] - 14.0) < 1e-10

            # Test slicing to end (None)
            sliced = result.slice(3, None, 1)
            assert len(sliced.time_history) == 2
            assert abs(sliced.time_history[0] - 13.0) < 1e-10
            assert abs(sliced.time_history[1] - 14.0) < 1e-10

            reader.close()
        finally:
            os.unlink(path)

    def test_search_result_slice_by_time(self):
        """Test slicing time history by time values"""
        path = create_test_mesh_with_variables()
        try:
            reader = ExodusReader.open(path)

            result = reader.search_nodal_variable(0.1, 0.1, 0.0, "temperature", -1.0)

            # Slice by time range: 0.1 to 0.31 (slightly beyond 0.3 to handle floating point)
            sliced = result.slice_by_time(reader, 0.1, 0.31)

            # Should get steps 1, 2, 3 (times 0.1, 0.2, 0.3)
            assert len(sliced.time_history) == 3, f"Expected 3 time steps, got {len(sliced.time_history)}"
            assert abs(sliced.time_history[0] - 11.0) < 1e-10
            assert abs(sliced.time_history[1] - 12.0) < 1e-10
            assert abs(sliced.time_history[2] - 13.0) < 1e-10

            # Test edge case: exact time match at start
            sliced = result.slice_by_time(reader, 0.0, 0.01)
            assert len(sliced.time_history) == 1
            assert abs(sliced.time_history[0] - 10.0) < 1e-10

            # Test slicing middle range
            sliced = result.slice_by_time(reader, 0.15, 0.25)
            assert len(sliced.time_history) == 1
            assert abs(sliced.time_history[0] - 12.0) < 1e-10

            reader.close()
        finally:
            os.unlink(path)

    def test_search_result_repr(self):
        """Test string representation of search result"""
        path = create_test_mesh_with_variables()
        try:
            reader = ExodusReader.open(path)

            result = reader.search_nodal_variable(0.1, 0.1, 0.0, "temperature", -1.0)

            repr_str = repr(result)
            assert "SpatialSearchResult" in repr_str
            assert "id=1" in repr_str
            assert "time_steps=5" in repr_str

            str_str = str(result)
            assert "SpatialSearchResult" in str_str

            reader.close()
        finally:
            os.unlink(path)

    def test_search_nonexistent_variable(self):
        """Test error handling for nonexistent variable"""
        path = create_test_mesh_with_variables()
        try:
            reader = ExodusReader.open(path)

            # Try to search for a variable that doesn't exist
            with pytest.raises(Exception):
                reader.search_nodal_variable(0.1, 0.1, 0.0, "nonexistent", -1.0)

            with pytest.raises(Exception):
                reader.search_element_variable(0.5, 0.5, 0.5, "nonexistent", -1.0)

            reader.close()
        finally:
            os.unlink(path)

    def test_search_with_distance_limit(self):
        """Test that distance limits work correctly"""
        path = create_test_mesh_with_variables()
        try:
            reader = ExodusReader.open(path)

            # Search with very small distance limit - should fail
            with pytest.raises(Exception):
                reader.search_nodal_variable(5.0, 5.0, 5.0, "temperature", 0.1)

            # Search with large distance limit - should succeed
            result = reader.search_nodal_variable(5.0, 5.0, 5.0, "temperature", 100.0)
            assert result.id in [1, 2, 3, 4, 5, 6, 7, 8]  # Should find some node

            # Test with default (None) - should use 5x avg element size
            result = reader.search_nodal_variable(0.5, 0.5, 0.0, "temperature", None)
            assert result.id in [1, 2, 3, 4]

            reader.close()
        finally:
            os.unlink(path)


class TestMultipleElements:
    """Tests with more complex meshes"""

    def create_multi_element_mesh(self):
        """Create a mesh with multiple elements"""
        temp_file = tempfile.NamedTemporaryFile(delete=False, suffix=".exo")
        temp_file.close()
        path = temp_file.name

        # Delete the temp file so ExodusWriter can create it
        os.unlink(path)

        from exodus import ExodusWriter, InitParams, Block, EntityType

        # Create file with ExodusWriter
        writer = ExodusWriter.create(path)

        # Initialize with 8 nodes, 2 elements
        params = InitParams(
            title="Multi Element Mesh",
            num_dim=3,
            num_nodes=8,
            num_elems=2,
            num_elem_blocks=1
        )
        writer.put_init_params(params)

        # Set coordinates
        coords_x = [0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
        coords_y = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0]
        coords_z = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
        writer.put_coords(coords_x, coords_y, coords_z)

        # Define hex block with 2 elements
        block = Block(
            id=100,
            entity_type=EntityType.ElemBlock,
            topology="HEX8",
            num_entries=2,
            num_nodes_per_entry=8,
            num_edges_per_entry=0,
            num_faces_per_entry=0,
            num_attributes=0
        )
        writer.put_block(block)
        writer.put_connectivity(100, [
            1, 2, 3, 4, 5, 6, 7, 8,  # Element 1
            1, 2, 3, 4, 5, 6, 7, 8,  # Element 2 (same nodes for simplicity)
        ])

        # Define element variable
        writer.define_variables(EntityType.ElemBlock, ["pressure"])

        # Write time steps
        for step in range(3):
            writer.put_time(step, float(step))
            elem_values = [50.0 + step, 150.0 + step]
            writer.put_var(step, EntityType.ElemBlock, 100, 0, elem_values)

        writer.close()
        return path

    def test_search_multi_element(self):
        """Test searching with multiple elements"""
        path = self.create_multi_element_mesh()
        try:
            reader = ExodusReader.open(path)

            # Search should find one of the elements
            result = reader.search_element_variable(0.5, 0.5, 0.5, "pressure", -1.0)
            assert result.id in [1, 2]
            assert len(result.time_history) == 3

            # Verify time history matches expected values
            if result.id == 1:
                expected = [50.0, 51.0, 52.0]
            else:
                expected = [150.0, 151.0, 152.0]

            for actual, exp in zip(result.time_history, expected):
                assert abs(actual - exp) < 1e-10

            reader.close()
        finally:
            os.unlink(path)
