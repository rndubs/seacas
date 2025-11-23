"""
NumPy integration tests for exodus-py

These tests verify that the NumPy bindings work correctly and return
the expected array shapes and types.

Note: Tests marked with xfail require Phase 1 Rust ndarray methods
(coords_array, var_time_series_array, connectivity_array) which are
not yet implemented in this branch. They will pass once merged with
the exodus-numpy-support branch.
"""

import pytest
import numpy as np
import tempfile
import os

# These tests will run once exodus-py is built with NumPy support
pytest.importorskip("exodus")
import exodus

# Mark for features requiring Phase 1 Rust implementation
requires_phase1 = pytest.mark.xfail(
    reason="Requires Phase 1 Rust ndarray methods (coords_array, var_time_series_array, connectivity_array) not yet in this branch",
    strict=False
)


class TestNumpyCoordinates:
    """Test NumPy integration for coordinate operations"""

    @requires_phase1
    def test_get_coords_returns_numpy_array(self, simple_mesh_file):
        """Test that get_coords() returns a 2D NumPy array"""
        reader = exodus.ExodusReader.open(simple_mesh_file)
        coords = reader.get_coords()

        assert isinstance(coords, np.ndarray), "get_coords should return NumPy array"
        assert coords.ndim == 2, "coords should be 2D"
        assert coords.shape[1] == 3, "coords should have 3 columns (x, y, z)"
        assert coords.dtype == np.float64, "coords should be float64"
        assert coords.flags['C_CONTIGUOUS'], "coords should be C-contiguous"

    def test_get_coords_list_backward_compat(self, simple_mesh_file):
        """Test that get_coords_list() still works for backward compatibility"""
        reader = exodus.ExodusReader.open(simple_mesh_file)
        x, y, z = reader.get_coords_list()

        assert isinstance(x, list), "get_coords_list should return lists"
        assert isinstance(y, list)
        assert isinstance(z, list)
        assert len(x) == len(y) == len(z)

    @requires_phase1
    def test_get_coord_x_returns_numpy(self, simple_mesh_file):
        """Test that individual coordinate getters return NumPy arrays"""
        reader = exodus.ExodusReader.open(simple_mesh_file)
        x = reader.get_coord_x()

        assert isinstance(x, np.ndarray)
        assert x.ndim == 1
        assert x.dtype == np.float64

    @requires_phase1
    def test_coords_values_match_list_version(self, simple_mesh_file):
        """Verify NumPy and list versions return same values"""
        reader = exodus.ExodusReader.open(simple_mesh_file)

        # Get both versions
        coords_np = reader.get_coords()
        x_list, y_list, z_list = reader.get_coords_list()

        # Compare values
        np.testing.assert_array_equal(coords_np[:, 0], np.array(x_list))
        np.testing.assert_array_equal(coords_np[:, 1], np.array(y_list))
        np.testing.assert_array_equal(coords_np[:, 2], np.array(z_list))

    @requires_phase1
    def test_put_coords_accepts_numpy(self, tmp_path):
        """Test that put_coords() accepts NumPy arrays"""
        filename = str(tmp_path / "test_write.exo")

        # Create writer
        writer = exodus.ExodusWriter.create(filename)

        # Initialize
        params = exodus.InitParams(
            title="NumPy Write Test",
            num_dim=3,
            num_nodes=4,
            num_elems=0,
            num_elem_blocks=0,
            num_node_sets=0,
            num_side_sets=0
        )
        writer.put_init_params(params)

        # Write coordinates as NumPy arrays
        x = np.array([0.0, 1.0, 1.0, 0.0])
        y = np.array([0.0, 0.0, 1.0, 1.0])
        z = np.array([0.0, 0.0, 0.0, 0.0])

        writer.put_coords(x, y, z)  # Should accept NumPy arrays
        writer.close()

        # Verify written correctly
        reader = exodus.ExodusReader.open(filename)
        coords = reader.get_coords()
        np.testing.assert_array_equal(coords[:, 0], x)
        np.testing.assert_array_equal(coords[:, 1], y)
        np.testing.assert_array_equal(coords[:, 2], z)


class TestNumpyVariables:
    """Test NumPy integration for variable operations"""

    @requires_phase1
    def test_var_returns_numpy_array(self, mesh_with_vars):
        """Test that var() returns a 1D NumPy array"""
        reader = exodus.ExodusReader.open(mesh_with_vars)
        data = reader.var(0, exodus.EntityType.Nodal, 0, 0)

        assert isinstance(data, np.ndarray)
        assert data.ndim == 1
        assert data.dtype == np.float64

    def test_var_list_backward_compat(self, mesh_with_vars):
        """Test that var_list() still works"""
        reader = exodus.ExodusReader.open(mesh_with_vars)
        data = reader.var_list(0, exodus.EntityType.Nodal, 0, 0)

        assert isinstance(data, list)

    @requires_phase1
    def test_var_time_series_returns_2d_numpy(self, mesh_with_vars):
        """Test that var_time_series() returns a 2D NumPy array"""
        reader = exodus.ExodusReader.open(mesh_with_vars)
        num_steps = reader.num_time_steps()
        data = reader.var_time_series(0, num_steps, exodus.EntityType.Nodal, 0, 0)

        assert isinstance(data, np.ndarray)
        assert data.ndim == 2
        assert data.shape[0] == num_steps
        assert data.dtype == np.float64
        assert data.flags['C_CONTIGUOUS']

    def test_var_time_series_list_backward_compat(self, mesh_with_vars):
        """Test that var_time_series_list() still works"""
        reader = exodus.ExodusReader.open(mesh_with_vars)
        num_steps = reader.num_time_steps()
        data = reader.var_time_series_list(0, num_steps, exodus.EntityType.Nodal, 0, 0)

        assert isinstance(data, list)

    @requires_phase1
    def test_var_time_series_indexing(self, mesh_with_vars):
        """Test that 2D time series array is properly indexed"""
        reader = exodus.ExodusReader.open(mesh_with_vars)
        num_steps = reader.num_time_steps()
        params = reader.init_params()

        data = reader.var_time_series(0, num_steps, exodus.EntityType.Nodal, 0, 0)

        # Access specific time step (all nodes)
        step_0 = data[0, :]
        assert step_0.shape[0] == params.num_nodes

        # Access specific node (all time steps)
        node_history = data[:, 0]
        assert node_history.shape[0] == num_steps

    @requires_phase1
    def test_put_var_accepts_numpy(self, tmp_path):
        """Test that put_var() accepts NumPy arrays"""
        filename = str(tmp_path / "test_var_write.exo")

        # Create and initialize
        writer = exodus.ExodusWriter.create(filename)
        params = exodus.InitParams(
            title="Var Write Test",
            num_dim=2,
            num_nodes=4,
            num_elems=0,
            num_elem_blocks=0,
            num_node_sets=0,
            num_side_sets=0,
            num_time_steps=1
        )
        writer.put_init_params(params)

        # Define variables
        writer.define_variables(exodus.EntityType.Nodal, ["Temperature"])

        # Write time
        writer.put_time(0, 0.0)

        # Write variable as NumPy array
        temp = np.array([100.0, 200.0, 300.0, 400.0])
        writer.put_var(0, exodus.EntityType.Nodal, 0, 0, temp)
        writer.close()

        # Verify
        reader = exodus.ExodusReader.open(filename)
        read_temp = reader.var(0, exodus.EntityType.Nodal, 0, 0)
        np.testing.assert_array_equal(read_temp, temp)


class TestNumpyConnectivity:
    """Test NumPy integration for connectivity operations"""

    @requires_phase1
    def test_get_connectivity_returns_2d_numpy(self, mesh_with_blocks):
        """Test that get_connectivity() returns a 2D NumPy array"""
        reader = exodus.ExodusReader.open(mesh_with_blocks)
        block_ids = reader.get_block_ids()

        conn = reader.get_connectivity(block_ids[0])

        assert isinstance(conn, np.ndarray)
        assert conn.ndim == 2
        assert conn.dtype == np.int64
        assert conn.flags['C_CONTIGUOUS']

    def test_get_connectivity_list_backward_compat(self, mesh_with_blocks):
        """Test that get_connectivity_list() still works"""
        reader = exodus.ExodusReader.open(mesh_with_blocks)
        block_ids = reader.get_block_ids()

        conn = reader.get_connectivity_list(block_ids[0])

        assert isinstance(conn, list)

    @requires_phase1
    def test_get_connectivity_shape(self, mesh_with_blocks):
        """Test that connectivity array has correct shape"""
        reader = exodus.ExodusReader.open(mesh_with_blocks)
        block_ids = reader.get_block_ids()
        block = reader.get_block(block_ids[0])

        conn = reader.get_connectivity(block_ids[0])

        assert conn.shape[0] == block.num_entries
        assert conn.shape[1] == block.num_nodes_per_entry

    @requires_phase1
    def test_put_connectivity_accepts_numpy(self, tmp_path):
        """Test that put_connectivity() accepts NumPy arrays"""
        filename = str(tmp_path / "test_conn_write.exo")

        writer = exodus.ExodusWriter.create(filename)
        params = exodus.InitParams(
            title="Connectivity Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
            num_node_sets=0,
            num_side_sets=0
        )
        writer.put_init_params(params)

        # Write coords
        x = np.array([0.0, 1.0, 1.0, 0.0])
        y = np.array([0.0, 0.0, 1.0, 1.0])
        z = np.array([0.0, 0.0, 0.0, 0.0])
        writer.put_coords(x, y, z)

        # Define block
        block = exodus.Block(
            id=100,
            entity_type=exodus.EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_edges_per_entry=0,
            num_faces_per_entry=0,
            num_attributes=0
        )
        writer.put_block(block)

        # Write connectivity as NumPy array
        conn = np.array([1, 2, 3, 4], dtype=np.int64)
        writer.put_connectivity(100, conn)
        writer.close()

        # Verify
        reader = exodus.ExodusReader.open(filename)
        read_conn = reader.get_connectivity(100)
        np.testing.assert_array_equal(read_conn.flatten(), conn)


class TestNumpyMemoryEfficiency:
    """Test memory efficiency of NumPy integration"""

    @requires_phase1
    def test_coords_returns_same_data(self, simple_mesh_file):
        """Verify that multiple calls return equivalent data (not testing true zero-copy due to ownership)"""
        reader = exodus.ExodusReader.open(simple_mesh_file)

        coords1 = reader.get_coords()
        coords2 = reader.get_coords()

        # Should return equivalent data
        np.testing.assert_array_equal(coords1, coords2)

        # Both should be NumPy arrays with same properties
        assert coords1.shape == coords2.shape
        assert coords1.dtype == coords2.dtype

    @requires_phase1
    def test_var_c_contiguous(self, mesh_with_vars):
        """Verify arrays are C-contiguous for efficient computation"""
        reader = exodus.ExodusReader.open(mesh_with_vars)

        coords = reader.get_coords()
        assert coords.flags['C_CONTIGUOUS']

        data = reader.var(0, exodus.EntityType.Nodal, 0, 0)
        assert data.flags['C_CONTIGUOUS']

        time_series = reader.var_time_series(0, 10, exodus.EntityType.Nodal, 0, 0)
        assert time_series.flags['C_CONTIGUOUS']


class TestNumpyCoordinatesComplete:
    """Additional tests for complete coordinate coverage"""

    @requires_phase1
    def test_get_coord_y_returns_numpy(self, simple_mesh_file):
        """Test that get_coord_y() returns NumPy array"""
        reader = exodus.ExodusReader.open(simple_mesh_file)
        y = reader.get_coord_y()

        assert isinstance(y, np.ndarray)
        assert y.ndim == 1
        assert y.dtype == np.float64
        assert len(y) == reader.init_params().num_nodes

    @requires_phase1
    def test_get_coord_z_returns_numpy(self, simple_mesh_file):
        """Test that get_coord_z() returns NumPy array"""
        reader = exodus.ExodusReader.open(simple_mesh_file)
        z = reader.get_coord_z()

        assert isinstance(z, np.ndarray)
        assert z.ndim == 1
        assert z.dtype == np.float64
        assert len(z) == reader.init_params().num_nodes

    @requires_phase1
    def test_all_coord_components_match(self, simple_mesh_file):
        """Verify individual coord getters match combined getter"""
        reader = exodus.ExodusReader.open(simple_mesh_file)

        # Get combined
        coords = reader.get_coords()

        # Get individual components
        x = reader.get_coord_x()
        y = reader.get_coord_y()
        z = reader.get_coord_z()

        # Verify they match
        np.testing.assert_array_equal(coords[:, 0], x)
        np.testing.assert_array_equal(coords[:, 1], y)
        np.testing.assert_array_equal(coords[:, 2], z)

    @requires_phase1
    def test_put_coords_numpy_f32(self, tmp_path):
        """Test that put_coords() accepts float32 NumPy arrays"""
        filename = str(tmp_path / "test_f32.exo")

        writer = exodus.ExodusWriter.create(filename)
        params = exodus.InitParams(
            title="Float32 Test",
            num_dim=3,
            num_nodes=4,
            num_elems=0,
            num_elem_blocks=0,
            num_node_sets=0,
            num_side_sets=0
        )
        writer.put_init_params(params)

        # Write coordinates as float32 NumPy arrays
        x = np.array([0.0, 1.0, 1.0, 0.0], dtype=np.float32)
        y = np.array([0.0, 0.0, 1.0, 1.0], dtype=np.float32)
        z = np.array([0.0, 0.0, 0.0, 0.0], dtype=np.float32)

        writer.put_coords(x, y, z)
        writer.close()

        # Verify written correctly
        reader = exodus.ExodusReader.open(filename)
        coords = reader.get_coords()
        np.testing.assert_array_almost_equal(coords[:, 0], x.astype(np.float64))


class TestNumpyVariablesComplete:
    """Additional tests for complete variable coverage"""

    @requires_phase1
    def test_put_var_time_series_1d_numpy(self, tmp_path):
        """Test that put_var_time_series() accepts 1D NumPy array"""
        filename = str(tmp_path / "test_ts_write.exo")

        writer = exodus.ExodusWriter.create(filename)
        params = exodus.InitParams(
            title="Time Series Write Test",
            num_dim=2,
            num_nodes=4,
            num_elems=0,
            num_elem_blocks=0,
            num_node_sets=0,
            num_side_sets=0,
            num_time_steps=3
        )
        writer.put_init_params(params)

        # Write coordinates
        x = [0.0, 1.0, 1.0, 0.0]
        y = [0.0, 0.0, 1.0, 1.0]
        z = [0.0, 0.0, 0.0, 0.0]
        writer.put_coords(x, y, z)

        # Define global variable
        writer.define_variables(exodus.EntityType.Global, ["Energy"])

        # Write time steps and variable using NumPy array
        for step in range(3):
            writer.put_time(step, float(step))

        # Write all time steps at once using 1D NumPy array (flattened)
        energy_values = np.array([10.0, 20.0, 30.0])  # One value per time step
        writer.put_var_time_series(0, 3, exodus.EntityType.Global, 0, 0, energy_values)
        writer.close()

        # Verify
        reader = exodus.ExodusReader.open(filename)
        ts = reader.var_time_series(0, 3, exodus.EntityType.Global, 0, 0)
        assert ts.shape[0] == 3  # 3 time steps
        np.testing.assert_array_almost_equal(ts.flatten(), energy_values)

    @requires_phase1
    def test_put_var_numpy_int_array(self, tmp_path):
        """Test that put_var() converts integer NumPy arrays to float"""
        filename = str(tmp_path / "test_int.exo")

        writer = exodus.ExodusWriter.create(filename)
        params = exodus.InitParams(
            title="Integer Test",
            num_dim=2,
            num_nodes=4,
            num_elems=0,
            num_elem_blocks=0,
            num_node_sets=0,
            num_side_sets=0,
            num_time_steps=1
        )
        writer.put_init_params(params)

        # Write coordinates
        writer.put_coords([0.0, 1.0, 1.0, 0.0], [0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0])

        # Define variable
        writer.define_variables(exodus.EntityType.Nodal, ["Count"])
        writer.put_time(0, 0.0)

        # Write variable as int NumPy array
        count = np.array([1, 2, 3, 4], dtype=np.int32)
        writer.put_var(0, exodus.EntityType.Nodal, 0, 0, count)
        writer.close()

        # Verify
        reader = exodus.ExodusReader.open(filename)
        read_count = reader.var(0, exodus.EntityType.Nodal, 0, 0)
        np.testing.assert_array_equal(read_count, count.astype(np.float64))

    @requires_phase1
    def test_var_time_series_slicing(self, mesh_with_vars):
        """Test that 2D time series array supports advanced slicing"""
        reader = exodus.ExodusReader.open(mesh_with_vars)
        data = reader.var_time_series(0, 10, exodus.EntityType.Nodal, 0, 0)

        # Slice time steps 2-5, all nodes
        subset = data[2:5, :]
        assert subset.shape == (3, 4)

        # Slice all time steps, nodes 1-3
        node_subset = data[:, 1:3]
        assert node_subset.shape == (10, 2)

        # Fancy indexing
        specific = data[[0, 5, 9], :]
        assert specific.shape == (3, 4)


class TestNumpyConnectivityComplete:
    """Additional tests for complete connectivity coverage"""

    @requires_phase1
    def test_put_connectivity_2d_numpy(self, tmp_path):
        """Test that put_connectivity() accepts 2D NumPy array"""
        filename = str(tmp_path / "test_conn_2d.exo")

        writer = exodus.ExodusWriter.create(filename)
        params = exodus.InitParams(
            title="2D Connectivity Test",
            num_dim=2,
            num_nodes=6,
            num_elems=2,
            num_elem_blocks=1,
            num_node_sets=0,
            num_side_sets=0
        )
        writer.put_init_params(params)

        # Write coords
        x = np.array([0.0, 1.0, 0.5, 1.0, 2.0, 1.5])
        y = np.array([0.0, 0.0, 1.0, 0.0, 0.0, 1.0])
        z = np.zeros(6)
        writer.put_coords(x, y, z)

        # Define block
        block = exodus.Block(
            id=100,
            entity_type=exodus.EntityType.ElemBlock,
            topology="TRI3",
            num_entries=2,
            num_nodes_per_entry=3,
            num_edges_per_entry=0,
            num_faces_per_entry=0,
            num_attributes=0
        )
        writer.put_block(block)

        # Write connectivity as 2D NumPy array (num_elems, nodes_per_elem)
        conn_2d = np.array([[1, 2, 3], [4, 5, 6]], dtype=np.int64)
        writer.put_connectivity(100, conn_2d.flatten())  # Flatten for API
        writer.close()

        # Verify
        reader = exodus.ExodusReader.open(filename)
        read_conn = reader.get_connectivity(100)
        np.testing.assert_array_equal(read_conn, conn_2d)

    @requires_phase1
    def test_get_connectivity_values_correct(self, mesh_with_blocks):
        """Verify connectivity values are 1-indexed node IDs"""
        reader = exodus.ExodusReader.open(mesh_with_blocks)
        block_ids = reader.get_block_ids()
        conn = reader.get_connectivity(block_ids[0])

        # All node indices should be >= 1 (1-indexed)
        assert np.all(conn >= 1)

        # All node indices should be <= num_nodes
        params = reader.init_params()
        assert np.all(conn <= params.num_nodes)

    @requires_phase1
    def test_get_connectivity_element_access(self, mesh_with_blocks):
        """Test accessing individual elements from connectivity array"""
        reader = exodus.ExodusReader.open(mesh_with_blocks)
        block_ids = reader.get_block_ids()
        conn = reader.get_connectivity(block_ids[0])

        # Access single element nodes
        elem0_nodes = conn[0, :]
        assert len(elem0_nodes) == 3  # TRI3 has 3 nodes

        # Verify node IDs are integers
        assert elem0_nodes.dtype == np.int64


class TestNumpyEdgeCases:
    """Test edge cases and error handling for NumPy integration"""

    @requires_phase1
    def test_coords_2d_mesh_z_zeros(self, tmp_path):
        """Test that 2D mesh returns zeros for z coordinates"""
        filename = str(tmp_path / "test_2d.exo")

        writer = exodus.ExodusWriter.create(filename)
        params = exodus.InitParams(
            title="2D Mesh",
            num_dim=2,
            num_nodes=4,
            num_elems=0,
            num_elem_blocks=0,
            num_node_sets=0,
            num_side_sets=0
        )
        writer.put_init_params(params)

        x = [0.0, 1.0, 1.0, 0.0]
        y = [0.0, 0.0, 1.0, 1.0]
        z = [0.0, 0.0, 0.0, 0.0]
        writer.put_coords(x, y, z)
        writer.close()

        reader = exodus.ExodusReader.open(filename)
        coords = reader.get_coords()

        # Z column should be all zeros
        np.testing.assert_array_equal(coords[:, 2], np.zeros(4))

    @requires_phase1
    def test_1d_mesh_coords_array(self, tmp_path):
        """Test that 1D mesh returns proper coords array"""
        filename = str(tmp_path / "test_1d.exo")

        writer = exodus.ExodusWriter.create(filename)
        params = exodus.InitParams(
            title="1D Mesh",
            num_dim=1,
            num_nodes=5,
            num_elems=0,
            num_elem_blocks=0,
            num_node_sets=0,
            num_side_sets=0
        )
        writer.put_init_params(params)

        x = [0.0, 0.25, 0.5, 0.75, 1.0]
        writer.put_coords(x, None, None)
        writer.close()

        reader = exodus.ExodusReader.open(filename)
        coords = reader.get_coords()

        assert coords.shape == (5, 3)
        np.testing.assert_array_almost_equal(coords[:, 0], x)
        # Y and Z should be zeros
        np.testing.assert_array_equal(coords[:, 1], np.zeros(5))
        np.testing.assert_array_equal(coords[:, 2], np.zeros(5))

    @requires_phase1
    def test_empty_mesh_coords(self, tmp_path):
        """Test coords array for empty mesh (0 nodes)"""
        filename = str(tmp_path / "test_empty.exo")

        writer = exodus.ExodusWriter.create(filename)
        params = exodus.InitParams(
            title="Empty Mesh",
            num_dim=3,
            num_nodes=0,
            num_elems=0,
            num_elem_blocks=0,
            num_node_sets=0,
            num_side_sets=0
        )
        writer.put_init_params(params)
        writer.close()

        reader = exodus.ExodusReader.open(filename)
        coords = reader.get_coords()

        assert coords.shape == (0, 3)
        assert coords.dtype == np.float64

    @requires_phase1
    def test_large_array_c_contiguous(self, tmp_path):
        """Test that large arrays maintain C-contiguous layout"""
        filename = str(tmp_path / "test_large.exo")

        num_nodes = 10000

        writer = exodus.ExodusWriter.create(filename)
        params = exodus.InitParams(
            title="Large Mesh",
            num_dim=3,
            num_nodes=num_nodes,
            num_elems=0,
            num_elem_blocks=0,
            num_node_sets=0,
            num_side_sets=0
        )
        writer.put_init_params(params)

        x = np.linspace(0, 100, num_nodes)
        y = np.linspace(0, 100, num_nodes)
        z = np.linspace(0, 100, num_nodes)
        writer.put_coords(x, y, z)
        writer.close()

        reader = exodus.ExodusReader.open(filename)
        coords = reader.get_coords()

        assert coords.shape == (num_nodes, 3)
        assert coords.flags['C_CONTIGUOUS']
        np.testing.assert_array_almost_equal(coords[:, 0], x)
        np.testing.assert_array_almost_equal(coords[:, 1], y)
        np.testing.assert_array_almost_equal(coords[:, 2], z)


# Fixtures

@pytest.fixture
def simple_mesh_file(tmp_path):
    """Create a simple mesh file for testing"""
    filename = str(tmp_path / "simple.exo")

    writer = exodus.ExodusWriter.create(filename)
    params = exodus.InitParams(
        title="Simple Test Mesh",
        num_dim=3,
        num_nodes=8,
        num_elems=1,
        num_elem_blocks=1,
        num_node_sets=0,
        num_side_sets=0
    )
    writer.put_init_params(params)

    # Write coordinates (cube)
    x = [0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
    y = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0]
    z = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
    writer.put_coords(x, y, z)

    writer.close()
    return filename


@pytest.fixture
def mesh_with_vars(tmp_path):
    """Create a mesh file with variables for testing"""
    filename = str(tmp_path / "with_vars.exo")

    writer = exodus.ExodusWriter.create(filename)
    params = exodus.InitParams(
        title="Mesh with Variables",
        num_dim=2,
        num_nodes=4,
        num_elems=0,
        num_elem_blocks=0,
        num_node_sets=0,
        num_side_sets=0,
        num_time_steps=10
    )
    writer.put_init_params(params)

    # Write coordinates
    x = [0.0, 1.0, 1.0, 0.0]
    y = [0.0, 0.0, 1.0, 1.0]
    z = [0.0, 0.0, 0.0, 0.0]
    writer.put_coords(x, y, z)

    # Define variables
    writer.define_variables(exodus.EntityType.Nodal, ["Temperature", "Pressure"])

    # Write time steps
    for step in range(10):
        writer.put_time(step, float(step))
        temp = [100.0 + step * 10 + i for i in range(4)]
        pressure = [1.0 + step * 0.1 + i * 0.01 for i in range(4)]
        writer.put_var(step, exodus.EntityType.Nodal, 0, 0, temp)
        writer.put_var(step, exodus.EntityType.Nodal, 0, 1, pressure)

    writer.close()
    return filename


@pytest.fixture
def mesh_with_blocks(tmp_path):
    """Create a mesh file with element blocks for testing"""
    filename = str(tmp_path / "with_blocks.exo")

    writer = exodus.ExodusWriter.create(filename)
    params = exodus.InitParams(
        title="Mesh with Blocks",
        num_dim=2,
        num_nodes=6,
        num_elems=2,
        num_elem_blocks=1,
        num_node_sets=0,
        num_side_sets=0
    )
    writer.put_init_params(params)

    # Write coordinates (two triangles)
    x = [0.0, 1.0, 0.5, 1.0, 2.0, 1.5]
    y = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0]
    z = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    writer.put_coords(x, y, z)

    # Define block
    block = exodus.Block(
        id=100,
        entity_type=exodus.EntityType.ElemBlock,
        topology="TRI3",
        num_entries=2,
        num_nodes_per_entry=3,
        num_edges_per_entry=0,
        num_faces_per_entry=0,
        num_attributes=0
    )
    writer.put_block(block)

    # Write connectivity
    conn = [1, 2, 3, 4, 5, 6]
    writer.put_connectivity(100, conn)

    writer.close()
    return filename


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
