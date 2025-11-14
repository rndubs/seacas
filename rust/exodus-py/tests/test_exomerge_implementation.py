"""
Comprehensive tests for the implemented features of exodus.exomerge module.

These tests validate the File I/O and basic element block operations.
"""

try:
    import pytest
except ImportError:
    # Create a minimal pytest stub for running tests without pytest
    class PytestStub:
        @staticmethod
        def skip(msg):
            raise RuntimeError(f"Test skipped: {msg}")

        @staticmethod
        def fail(msg):
            raise AssertionError(msg)

        @staticmethod
        def approx(expected, abs=None):
            class Approx:
                def __init__(self, expected, tolerance):
                    self.expected = expected
                    self.tolerance = tolerance or 1e-6

                def __eq__(self, actual):
                    if isinstance(self.expected, list):
                        return all(abs(a - e) <= self.tolerance for a, e in zip(actual, self.expected))
                    return abs(actual - self.expected) <= self.tolerance

            return Approx(expected, abs)

        class raises:
            def __init__(self, exc_type):
                self.exc_type = exc_type
                self.value = None

            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc_val, exc_tb):
                if exc_type is None:
                    raise AssertionError(f"Expected {self.exc_type.__name__} but no exception was raised")
                if not issubclass(exc_type, self.exc_type):
                    return False
                self.value = exc_val
                return True

        @staticmethod
        def main(args):
            pass

    pytest = PytestStub()

import tempfile
import os


def create_simple_mesh_file():
    """Helper to create a simple test mesh file using exodus-py."""
    try:
        import exodus
    except ImportError:
        pytest.skip("exodus module not available")

    # Create a temporary file
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name
    os.unlink(tmp_path)

    # Create a simple 2D mesh with one quad element
    writer = exodus.ExodusWriter.create(tmp_path)
    params = exodus.InitParams(
        title="Test Mesh",
        num_dim=2,
        num_nodes=4,
        num_elems=1,
        num_elem_blocks=1,
    )
    writer.put_init_params(params)

    # Write coordinates (unit square)
    writer.put_coords([0.0, 1.0, 1.0, 0.0], [0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0])

    # Write element block
    block = exodus.Block(
        id=1,
        entity_type=exodus.EntityType.ElemBlock,
        topology="QUAD4",
        num_entries=1,
        num_nodes_per_entry=4,
        num_attributes=0
    )
    writer.put_block(block)
    writer.put_connectivity(1, [1, 2, 3, 4])  # 1-indexed

    writer.close()
    return tmp_path


def test_import_simple_mesh():
    """Test importing a simple mesh file."""
    import exodus.exomerge as exomerge

    tmp_path = create_simple_mesh_file()
    try:
        # Import the model
        model = exomerge.import_model(tmp_path)

        # Validate basic properties
        assert model is not None
        assert isinstance(model, exomerge.ExodusModel)
        assert model.get_title() == "Test Mesh"

        # Validate nodes
        assert model.get_node_count() == 4
        nodes = model.get_nodes()
        assert len(nodes) == 4
        assert nodes[0] == [0.0, 0.0, 0.0]
        assert nodes[1] == [1.0, 0.0, 0.0]

        # Validate element blocks
        block_ids = model.get_element_block_ids()
        assert block_ids == [1]
        assert model.element_block_exists(1)
        assert not model.element_block_exists(999)

        # Validate element count
        assert model.get_element_count() == 1
        assert model.get_element_count("all") == 1
        assert model.get_element_count([1]) == 1

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_element_block_info():
    """Test element block information retrieval."""
    import exodus.exomerge as exomerge

    tmp_path = create_simple_mesh_file()
    try:
        model = exomerge.import_model(tmp_path)

        # Test element block properties
        assert model.get_nodes_per_element(1) == 4
        assert model.get_element_block_dimension(1) == 2  # QUAD4 is 2D

        # Test connectivity
        conn = model.get_connectivity(1)
        assert len(conn) > 0

        # Test get_connectivity with "auto" (single block)
        conn_auto = model.get_connectivity("auto")
        assert conn_auto == conn

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_export_simple_mesh():
    """Test exporting a mesh file."""
    import exodus.exomerge as exomerge

    tmp_path_in = create_simple_mesh_file()
    tmp_path_out = tmp_path_in.replace(".exo", "_out.exo")

    try:
        # Import then export
        model = exomerge.import_model(tmp_path_in)
        model.export_model(tmp_path_out)

        # Verify output file was created
        assert os.path.exists(tmp_path_out)

        # Re-import and validate
        model2 = exomerge.import_model(tmp_path_out)
        assert model2.get_node_count() == model.get_node_count()
        assert model2.get_element_count() == model.get_element_count()
        assert model2.get_element_block_ids() == model.get_element_block_ids()

    finally:
        for path in [tmp_path_in, tmp_path_out]:
            if os.path.exists(path):
                os.unlink(path)


def test_manual_model_construction():
    """Test manually constructing a model and exporting it."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.set_title("Manual Mesh")

    # Add nodes manually
    model.nodes = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ]

    # Add element block
    # Format: [name, info, connectivity, fields]
    # info: [elem_type, num_elems, nodes_per_elem, num_attrs]
    model.element_blocks[100] = [
        "MyBlock",
        ["QUAD4", 1, 4, 0],
        [[1, 2, 3, 4]],  # 1-indexed connectivity
        {}
    ]

    # Validate
    assert model.get_node_count() == 4
    assert model.get_element_count() == 1
    assert model.element_block_exists(100)
    assert model.get_nodes_per_element(100) == 4
    assert model.get_element_block_dimension(100) == 2

    # Export
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name
    os.unlink(tmp_path)

    try:
        model.export_model(tmp_path)
        assert os.path.exists(tmp_path)

        # Re-import and validate
        model2 = exomerge.import_model(tmp_path)
        assert model2.get_node_count() == 4
        assert model2.get_element_count() == 1

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_element_block_names():
    """Test element block name operations."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.element_blocks[1] = ["Block1", ["HEX8", 10, 8, 0], [], {}]
    model.element_blocks[2] = ["Block2", ["TET4", 20, 4, 0], [], {}]

    # Test get_element_block_name
    assert model.get_element_block_name(1) == "Block1"
    assert model.get_element_block_name(2) == "Block2"

    # Test get_all_element_block_names
    names = model.get_all_element_block_names()
    assert names == {1: "Block1", 2: "Block2"}


def test_element_dimensions():
    """Test _get_dimension helper for various element types."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Test various element types
    assert model._get_dimension("HEX8") == 3
    assert model._get_dimension("hex20") == 3
    assert model._get_dimension("TET4") == 3
    assert model._get_dimension("QUAD4") == 2
    assert model._get_dimension("TRI3") == 2
    assert model._get_dimension("LINE2") == 1
    assert model._get_dimension("point") == 0

    # Test case insensitivity
    assert model._get_dimension("WEDGE6") == 3
    assert model._get_dimension("wedge6") == 3


def test_timesteps():
    """Test timestep operations."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Initially empty
    assert model.get_timesteps() == []
    assert not model.timestep_exists(1.0)

    # Add timesteps
    model.timesteps = [0.0, 0.5, 1.0]
    assert model.get_timesteps() == [0.0, 0.5, 1.0]
    assert model.timestep_exists(0.5)
    assert not model.timestep_exists(0.75)


def test_metadata_operations():
    """Test metadata (title, QA, info) operations."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # Title
    assert model.get_title() == ""
    model.set_title("Test Model")
    assert model.get_title() == "Test Model"

    # Info records
    assert model.get_info_records() == []
    model.add_info_record("Info 1")
    model.add_info_record("Info 2")
    assert model.get_info_records() == ["Info 1", "Info 2"]

    # QA records
    assert model.get_qa_records() == []


def test_get_connectivity_auto_error():
    """Test that get_connectivity('auto') errors with multiple blocks."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()
    model.element_blocks[1] = ["Block1", ["HEX8", 1, 8, 0], [[1,2,3,4,5,6,7,8]], {}]
    model.element_blocks[2] = ["Block2", ["TET4", 1, 4, 0], [[1,2,3,4]], {}]

    # Should error because there are multiple blocks
    with pytest.raises(ValueError):
        model.get_connectivity("auto")


def test_element_block_not_found():
    """Test error handling for non-existent element blocks."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # get_element_block_name returns empty string for non-existent blocks
    assert model.get_element_block_name(999) == ""

    # get_nodes_per_element raises ValueError for non-existent blocks
    with pytest.raises(ValueError):
        model.get_nodes_per_element(999)

    # get_element_block_dimension raises ValueError for non-existent blocks
    with pytest.raises(ValueError):
        model.get_element_block_dimension(999)

    # get_connectivity raises ValueError for non-existent blocks
    with pytest.raises(ValueError):
        model.get_connectivity(999)


def test_roundtrip_preserve_data():
    """Test that import->export->import preserves all data."""
    import exodus.exomerge as exomerge

    tmp_path1 = create_simple_mesh_file()
    tmp_path2 = tmp_path1.replace(".exo", "_copy.exo")

    try:
        # Import, export, re-import
        model1 = exomerge.import_model(tmp_path1)
        model1.export_model(tmp_path2)
        model2 = exomerge.import_model(tmp_path2)

        # Compare key properties
        assert model1.get_title() == model2.get_title()
        assert model1.get_node_count() == model2.get_node_count()
        assert model1.get_element_count() == model2.get_element_count()
        assert model1.get_element_block_ids() == model2.get_element_block_ids()

        # Compare node coordinates
        nodes1 = model1.get_nodes()
        nodes2 = model2.get_nodes()
        assert len(nodes1) == len(nodes2)
        for n1, n2 in zip(nodes1, nodes2):
            assert n1 == pytest.approx(n2, abs=1e-10)

    finally:
        for path in [tmp_path1, tmp_path2]:
            if os.path.exists(path):
                os.unlink(path)


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


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
