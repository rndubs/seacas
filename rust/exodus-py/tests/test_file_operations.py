"""
Tests for file operations (Reader, Writer, Appender)
"""

import pytest
import tempfile
import os

pytest.importorskip("exodus")

from exodus import (
    ExodusReader,
    ExodusWriter,
    ExodusAppender,
    InitParams,
    CreateOptions,
    CreateMode,
    FloatSize,
    Int64Mode,
)


def test_create_options_defaults():
    """Test CreateOptions with default values"""
    opts = CreateOptions()
    assert opts is not None


def test_create_options_with_mode():
    """Test CreateOptions with specific mode"""
    opts = CreateOptions(mode=CreateMode.Clobber)
    assert opts is not None


def test_create_options_with_sizes():
    """Test CreateOptions with float and int64 sizes"""
    opts = CreateOptions(
        mode=CreateMode.Clobber,
        float_size=FloatSize.Float64,
        int64_mode=Int64Mode.Int64,
    )
    assert opts is not None


def test_writer_create_simple_file():
    """Test creating a simple Exodus file with ExodusWriter"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        # Create file with writer
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Test File",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)
        writer.close()

        # Verify file was created
        assert os.path.exists(tmp_path)

        # Read back and verify
        reader = ExodusReader.open(tmp_path)
        read_params = reader.init_params()
        assert read_params.title == "Test File"
        assert read_params.num_dim == 2
        assert read_params.num_nodes == 4
        assert read_params.num_elems == 1
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_writer_with_options():
    """Test creating file with CreateOptions"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        opts = CreateOptions(mode=CreateMode.Clobber, float_size=FloatSize.Float64)
        writer = ExodusWriter.create(tmp_path, opts)
        params = InitParams(
            title="Test Options",
            num_dim=3,
            num_nodes=8,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)
        writer.close()

        assert os.path.exists(tmp_path)

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_reader_open_existing():
    """Test opening existing file with ExodusReader"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        # Create a file first
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Reader Test", num_dim=2, num_nodes=4)
        writer.put_init_params(params)
        writer.close()

        # Open with reader
        reader = ExodusReader.open(tmp_path)
        read_params = reader.init_params()
        assert read_params.title == "Reader Test"
        assert read_params.num_dim == 2
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_appender_modify_existing():
    """Test appending to existing file with ExodusAppender"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        # Create initial file
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Appender Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)
        writer.close()

        # Open with appender
        appender = ExodusAppender.append(tmp_path)
        read_params = appender.init_params()
        assert read_params.title == "Appender Test"
        appender.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_init_params_creation():
    """Test creating InitParams with various parameters"""
    params = InitParams(
        title="Full Params Test",
        num_dim=3,
        num_nodes=100,
        num_elems=50,
        num_elem_blocks=2,
        num_node_sets=1,
        num_side_sets=2,
    )
    assert params.title == "Full Params Test"
    assert params.num_dim == 3
    assert params.num_nodes == 100
    assert params.num_elems == 50
    assert params.num_elem_blocks == 2
    assert params.num_node_sets == 1
    assert params.num_side_sets == 2


def test_context_manager_reader():
    """Test ExodusReader with context manager"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        # Create file
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Context Test", num_dim=2, num_nodes=4)
        writer.put_init_params(params)
        writer.close()

        # Use context manager
        with ExodusReader.open(tmp_path) as reader:
            read_params = reader.init_params()
            assert read_params.title == "Context Test"

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_float_size_enum():
    """Test FloatSize enum values"""
    assert FloatSize.Float32 is not None
    assert FloatSize.Float64 is not None


def test_int64_mode_enum():
    """Test Int64Mode enum values"""
    assert Int64Mode.Int32 is not None
    assert Int64Mode.Int64 is not None


def test_create_mode_enum():
    """Test CreateMode enum values"""
    assert CreateMode.Clobber is not None
    assert CreateMode.NoClobber is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
