"""
Comprehensive tests for the implemented features of exodus.exomerge module.

These tests validate the File I/O and basic element block operations.
"""

import pytest
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
        elem_type="QUAD4",
        num_elems=1,
        nodes_per_elem=4,
        num_attrs=0
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
    with pytest.raises(SystemExit):  # _error() calls sys.exit()
        model.get_connectivity("auto")


def test_element_block_not_found():
    """Test error handling for non-existent element blocks."""
    import exodus.exomerge as exomerge

    model = exomerge.ExodusModel()

    # These should error
    with pytest.raises(SystemExit):
        model.get_element_block_name(999)

    with pytest.raises(SystemExit):
        model.get_nodes_per_element(999)

    with pytest.raises(SystemExit):
        model.get_element_block_dimension(999)

    with pytest.raises(SystemExit):
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


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
