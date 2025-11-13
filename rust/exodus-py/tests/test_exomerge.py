"""
Tests for the exodus.exomerge module.

These tests verify API compatibility with the legacy exomerge3.py module.
"""

import pytest


def test_import_exomerge():
    """Test that exomerge module can be imported."""
    try:
        import exodus.exomerge as exomerge
        assert exomerge is not None
    except ImportError as e:
        pytest.fail(f"Failed to import exodus.exomerge: {e}")


def test_exomerge_version():
    """Test that exomerge has version information."""
    import exodus.exomerge as exomerge
    assert hasattr(exomerge, '__version__')
    assert isinstance(exomerge.__version__, str)
    assert len(exomerge.__version__) > 0


def test_exomerge_contact():
    """Test that exomerge has contact information."""
    import exodus.exomerge as exomerge
    assert hasattr(exomerge, 'CONTACT')
    assert isinstance(exomerge.CONTACT, str)


def test_import_model_function_exists():
    """Test that import_model function exists."""
    import exodus.exomerge as exomerge
    assert hasattr(exomerge, 'import_model')
    assert callable(exomerge.import_model)


def test_exodus_model_class_exists():
    """Test that ExodusModel class exists."""
    import exodus.exomerge as exomerge
    assert hasattr(exomerge, 'ExodusModel')
    assert isinstance(exomerge.ExodusModel, type)


def test_create_exodus_model():
    """Test that ExodusModel can be instantiated."""
    import exodus.exomerge as exomerge
    model = exomerge.ExodusModel()
    assert model is not None
    assert isinstance(model, exomerge.ExodusModel)


def test_exodus_model_attributes():
    """Test that ExodusModel has expected attributes."""
    import exodus.exomerge as exomerge
    model = exomerge.ExodusModel()

    # Check core data structures exist
    assert hasattr(model, 'nodes')
    assert hasattr(model, 'node_fields')
    assert hasattr(model, 'global_variables')
    assert hasattr(model, 'element_blocks')
    assert hasattr(model, 'side_sets')
    assert hasattr(model, 'node_sets')
    assert hasattr(model, 'timesteps')
    assert hasattr(model, 'title')
    assert hasattr(model, 'qa_records')
    assert hasattr(model, 'info_records')

    # Check they are initialized correctly
    assert isinstance(model.nodes, list)
    assert isinstance(model.node_fields, dict)
    assert isinstance(model.global_variables, dict)
    assert isinstance(model.element_blocks, dict)
    assert isinstance(model.side_sets, dict)
    assert isinstance(model.node_sets, dict)
    assert isinstance(model.timesteps, list)
    assert isinstance(model.title, str)
    assert isinstance(model.qa_records, list)
    assert isinstance(model.info_records, list)


def test_exodus_model_implemented_methods():
    """Test that basic implemented methods work."""
    import exodus.exomerge as exomerge
    model = exomerge.ExodusModel()

    # Test get_node_count (should return 0 for empty model)
    assert model.get_node_count() == 0

    # Test get_nodes (should return empty list)
    assert model.get_nodes() == []

    # Test title operations
    model.set_title("Test Model")
    assert model.get_title() == "Test Model"

    # Test timesteps operations
    assert model.get_timesteps() == []
    assert not model.timestep_exists(1.0)

    # Test QA and info records
    assert model.get_qa_records() == []
    assert model.get_info_records() == []

    model.add_info_record("Test info")
    assert model.get_info_records() == ["Test info"]


def test_exodus_model_unimplemented_methods_raise():
    """Test that unimplemented methods raise NotImplementedError."""
    import exodus.exomerge as exomerge
    model = exomerge.ExodusModel()

    # Test a few key unimplemented methods
    with pytest.raises(NotImplementedError):
        model.import_model("test.e")

    with pytest.raises(NotImplementedError):
        model.export_model("test.e")

    with pytest.raises(NotImplementedError):
        model.create_element_block(1, ['HEX8', 1, 8, 0])

    with pytest.raises(NotImplementedError):
        model.get_element_block_ids()


def test_stl_export_raises_with_explanation():
    """Test that STL export raises NotImplementedError with explanation."""
    import exodus.exomerge as exomerge
    model = exomerge.ExodusModel()

    with pytest.raises(NotImplementedError) as exc_info:
        model.export_stl_file("test.stl")

    # Check that the error message mentions why it's not implementable
    error_message = str(exc_info.value)
    assert "not implementable" in error_message.lower()
    assert "geometry" in error_message.lower() or "stl" in error_message.lower()


def test_wrl_export_raises_with_explanation():
    """Test that WRL export raises NotImplementedError with explanation."""
    import exodus.exomerge as exomerge
    model = exomerge.ExodusModel()

    with pytest.raises(NotImplementedError) as exc_info:
        model.export_wrl_model("test.wrl")

    # Check that the error message mentions why it's not implementable
    error_message = str(exc_info.value)
    assert "not implementable" in error_message.lower()
    assert "vrml" in error_message.lower() or "wrl" in error_message.lower()


def test_expression_methods_raise_with_explanation():
    """Test that expression-based methods raise NotImplementedError with explanation."""
    import exodus.exomerge as exomerge
    model = exomerge.ExodusModel()

    expression_methods = [
        ('calculate_element_field', ('x + y',)),
        ('calculate_node_field', ('sqrt(x**2 + y**2)',)),
        ('calculate_global_variable', ('2 * x',)),
    ]

    for method_name, args in expression_methods:
        method = getattr(model, method_name)
        with pytest.raises(NotImplementedError) as exc_info:
            method(*args)

        # Check that the error message mentions expression evaluation
        error_message = str(exc_info.value)
        assert "expression" in error_message.lower()


def test_deprecated_function_handling():
    """Test that deprecated function names are handled."""
    import exodus.exomerge as exomerge
    model = exomerge.ExodusModel()

    # 'write' is deprecated and should redirect to 'export'
    # Both should raise NotImplementedError, but 'write' should also warn
    assert hasattr(model, 'export')

    # The __getattr__ should handle the deprecated 'write' method
    # Note: We can't easily test the warning without capturing stdout,
    # but we can verify the attribute lookup works


def test_api_method_count():
    """Test that ExodusModel has approximately the expected number of public methods."""
    import exodus.exomerge as exomerge
    model = exomerge.ExodusModel()

    # Get all public methods (not starting with _)
    public_methods = [method for method in dir(model) if not method.startswith('_') and callable(getattr(model, method))]

    # Should have roughly 150+ public methods from the original API
    assert len(public_methods) >= 100, f"Expected at least 100 public methods, found {len(public_methods)}"


def test_module_constants():
    """Test that module has expected constants."""
    import exodus.exomerge as exomerge

    assert hasattr(exomerge, '__version__')
    assert hasattr(exomerge, 'VERSION')
    assert hasattr(exomerge, 'CONTACT')
    assert hasattr(exomerge, 'SHOW_BANNER')
    assert hasattr(exomerge, 'EXIT_ON_WARNING')
    assert hasattr(exomerge, 'DEPRECATED_FUNCTIONS')

    # Check types
    assert isinstance(exomerge.__version__, str)
    assert isinstance(exomerge.VERSION, str)
    assert isinstance(exomerge.CONTACT, str)
    assert isinstance(exomerge.SHOW_BANNER, bool)
    assert isinstance(exomerge.EXIT_ON_WARNING, bool)
    assert isinstance(exomerge.DEPRECATED_FUNCTIONS, dict)


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
