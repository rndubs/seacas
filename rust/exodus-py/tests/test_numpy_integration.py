"""
Tests for NumPy integration

This test file verifies that exodus-py properly integrates with NumPy arrays
for both input and output operations.
"""

import pytest
import tempfile
import os
import numpy as np

pytest.importorskip("exodus")

from exodus import (
    ExodusWriter,
    ExodusReader,
    InitParams,
    EntityType,
    Block,
)


def test_numpy_coordinate_input_output():
    """Test that we can write and read coordinates using NumPy arrays"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        # Create file using NumPy arrays for input
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="NumPy Coords", num_dim=3, num_nodes=8)
        writer.put_init_params(params)

        # Use NumPy arrays for coordinates
        x_coords = np.array([0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0])
        y_coords = np.array([0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0])
        z_coords = np.array([0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0])
        writer.put_coords(x_coords, y_coords, z_coords)
        writer.close()

        # Read back and verify we get NumPy arrays
        reader = ExodusReader.open(tmp_path)
        x_read, y_read, z_read = reader.get_coords()

        # Verify the returned values are NumPy arrays
        assert isinstance(x_read, np.ndarray)
        assert isinstance(y_read, np.ndarray)
        assert isinstance(z_read, np.ndarray)

        # Verify values
        np.testing.assert_array_almost_equal(x_read, x_coords)
        np.testing.assert_array_almost_equal(y_read, y_coords)
        np.testing.assert_array_almost_equal(z_read, z_coords)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_numpy_connectivity_input_output():
    """Test that connectivity works with NumPy arrays"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="NumPy Connectivity",
            num_dim=3,
            num_nodes=8,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        # Use NumPy arrays for input
        writer.put_coords(
            np.array([0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0]),
            np.array([0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0]),
            np.array([0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]),
        )

        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="HEX8",
            num_entries=1,
            num_nodes_per_entry=8,
            num_attributes=0,
        )
        writer.put_block(block)

        # Use NumPy array for connectivity
        connectivity = np.array([1, 2, 3, 4, 5, 6, 7, 8], dtype=np.int64)
        writer.put_connectivity(1, connectivity)
        writer.close()

        # Read back and verify
        reader = ExodusReader.open(tmp_path)
        conn_read = reader.get_connectivity(1)

        # Verify it's a NumPy array
        assert isinstance(conn_read, np.ndarray)

        # Verify values
        np.testing.assert_array_equal(conn_read, connectivity)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_numpy_variables_input_output():
    """Test that variable operations work with NumPy arrays"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="NumPy Variables",
            num_dim=2,
            num_nodes=4,
        )
        writer.put_init_params(params)

        # Define variables
        writer.define_variables(EntityType.Global, ["Energy"])
        writer.define_variables(EntityType.Nodal, ["Temperature", "Pressure"])

        # Write time steps using NumPy arrays
        writer.put_time(0, 0.0)
        writer.put_var(0, EntityType.Global, 0, 0, np.array([100.0]))
        writer.put_var(
            0, EntityType.Nodal, 0, 0, np.array([20.0, 21.0, 22.0, 23.0])
        )
        writer.put_var(
            0, EntityType.Nodal, 0, 1, np.array([1.0, 1.1, 1.2, 1.3])
        )

        writer.put_time(1, 1.0)
        writer.put_var(1, EntityType.Global, 0, 0, np.array([110.0]))
        writer.put_var(
            1, EntityType.Nodal, 0, 0, np.array([25.0, 26.0, 27.0, 28.0])
        )
        writer.close()

        # Read back and verify
        reader = ExodusReader.open(tmp_path)

        energy_0 = reader.var(0, EntityType.Global, 0, 0)
        assert isinstance(energy_0, np.ndarray)
        np.testing.assert_array_almost_equal(energy_0, [100.0])

        temp_0 = reader.var(0, EntityType.Nodal, 0, 0)
        assert isinstance(temp_0, np.ndarray)
        np.testing.assert_array_almost_equal(temp_0, [20.0, 21.0, 22.0, 23.0])

        press_0 = reader.var(0, EntityType.Nodal, 0, 1)
        assert isinstance(press_0, np.ndarray)
        np.testing.assert_array_almost_equal(press_0, [1.0, 1.1, 1.2, 1.3])

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_numpy_mixed_dtypes():
    """Test that different NumPy dtypes are properly converted"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Mixed dtypes", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Try different float dtypes
        x_float32 = np.array([0.0, 1.0, 1.0, 0.0], dtype=np.float32)
        y_float64 = np.array([0.0, 0.0, 1.0, 1.0], dtype=np.float64)
        z_empty = np.array([], dtype=np.float64)

        writer.put_coords(x_float32, y_float64, z_empty)
        writer.close()

        reader = ExodusReader.open(tmp_path)
        x_read, y_read, z_read = reader.get_coords()

        # Should all be float64 (f64)
        assert x_read.dtype == np.float64
        assert y_read.dtype == np.float64

        np.testing.assert_array_almost_equal(x_read, [0.0, 1.0, 1.0, 0.0])
        np.testing.assert_array_almost_equal(y_read, [0.0, 0.0, 1.0, 1.0])

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_numpy_connectivity_mixed_int_dtypes():
    """Test that different integer NumPy dtypes work for connectivity"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Mixed int dtypes",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        writer.put_coords(
            np.array([0.0, 1.0, 1.0, 0.0]),
            np.array([0.0, 0.0, 1.0, 1.0]),
            np.array([]),
        )

        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Try int32
        connectivity = np.array([1, 2, 3, 4], dtype=np.int32)
        writer.put_connectivity(1, connectivity)
        writer.close()

        reader = ExodusReader.open(tmp_path)
        conn_read = reader.get_connectivity(1)

        # Should be int64 (i64)
        assert conn_read.dtype == np.int64
        np.testing.assert_array_equal(conn_read, [1, 2, 3, 4])

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_numpy_set_operations():
    """Test that set operations work with NumPy arrays"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="NumPy Sets",
            num_dim=2,
            num_nodes=4,
            num_node_sets=1,
            num_side_sets=1,
        )
        writer.put_init_params(params)

        # Node set with NumPy arrays
        writer.put_set(EntityType.NodeSet, 10, 2, 2)
        nodes = np.array([1, 2], dtype=np.int64)
        dist_factors = np.array([1.0, 1.0], dtype=np.float64)
        writer.put_node_set(10, nodes, dist_factors)

        # Side set with NumPy arrays
        writer.put_set(EntityType.SideSet, 20, 1, 0)
        elements = np.array([1], dtype=np.int64)
        sides = np.array([3], dtype=np.int64)
        writer.put_side_set(20, elements, sides, None)

        writer.close()

        # Read and verify
        reader = ExodusReader.open(tmp_path)

        node_set_ids = reader.get_node_set_ids()
        assert isinstance(node_set_ids, np.ndarray)
        assert 10 in node_set_ids

        side_set_ids = reader.get_side_set_ids()
        assert isinstance(side_set_ids, np.ndarray)
        assert 20 in side_set_ids

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_backward_compatibility_with_lists():
    """Verify that lists still work (backward compatibility)"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="List Compat", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Use Python lists (old API)
        writer.put_coords([0.0, 1.0, 1.0, 0.0], [0.0, 0.0, 1.0, 1.0], [])
        writer.close()

        reader = ExodusReader.open(tmp_path)
        x, y, z = reader.get_coords()

        # Should return NumPy arrays
        assert isinstance(x, np.ndarray)
        assert isinstance(y, np.ndarray)

        # But values should match
        np.testing.assert_array_almost_equal(x, [0.0, 1.0, 1.0, 0.0])
        np.testing.assert_array_almost_equal(y, [0.0, 0.0, 1.0, 1.0])

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
