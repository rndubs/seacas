"""
Test transformation operations on Exodus files.

Tests spatial transformations including translation, rotation, scaling,
and Euler angle rotations.
"""

import pytest
import sys
import os
import tempfile
import math

sys.path.insert(0, 'python')

from exodus import ExodusWriter, ExodusAppender, InitParams, CreateOptions, CreateMode


@pytest.fixture
def simple_mesh_file():
    """Create a simple 3D mesh file for testing."""
    fd, path = tempfile.mkstemp(suffix='.exo')
    os.close(fd)

    # Create a file with a simple mesh
    writer = ExodusWriter.create(path, CreateOptions(mode=CreateMode.Clobber))

    params = InitParams(
        title="Test Mesh",
        num_dim=3,
        num_nodes=8,
        num_elems=1,
        num_elem_blocks=1,
        num_node_sets=0,
        num_side_sets=0
    )
    writer.put_init_params(params)

    # Unit cube: corners at (0,0,0) to (1,1,1)
    x = [0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
    y = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0]
    z = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
    writer.put_coords(x, y, z)

    writer.close()

    yield path

    # Cleanup
    if os.path.exists(path):
        os.unlink(path)


@pytest.fixture
def point_on_x_axis():
    """Create a mesh with a single point on the X axis."""
    fd, path = tempfile.mkstemp(suffix='.exo')
    os.close(fd)

    writer = ExodusWriter.create(path, CreateOptions(mode=CreateMode.Clobber))

    params = InitParams(
        title="Point on X",
        num_dim=3,
        num_nodes=1,
        num_elems=0,
        num_elem_blocks=0
    )
    writer.put_init_params(params)

    # Point at (1, 0, 0)
    writer.put_coords([1.0], [0.0], [0.0])
    writer.close()

    yield path

    # Cleanup
    if os.path.exists(path):
        os.unlink(path)


def test_translate(simple_mesh_file):
    """Test translating mesh coordinates."""
    appender = ExodusAppender.append(simple_mesh_file)

    # Translate by (10, 20, 30)
    appender.translate([10.0, 20.0, 30.0])

    # Read back coordinates
    x, y, z = appender.get_coords_list()

    # First point should have moved from (0,0,0) to (10,20,30)
    assert abs(x[0] - 10.0) < 1e-10
    assert abs(y[0] - 20.0) < 1e-10
    assert abs(z[0] - 30.0) < 1e-10

    # Second point should have moved from (1,0,0) to (11,20,30)
    assert abs(x[1] - 11.0) < 1e-10
    assert abs(y[1] - 20.0) < 1e-10
    assert abs(z[1] - 30.0) < 1e-10

    appender.close()


def test_rotate_x(simple_mesh_file):
    """Test rotation around X axis."""
    appender = ExodusAppender.append(simple_mesh_file)

    # Rotate 90 degrees around X axis
    # Point (0, 1, 0) should become (0, 0, 1)
    appender.rotate_x(90.0)

    x, y, z = appender.get_coords_list()

    # Find the point that was at (0, 1, 0) - that's index 3
    # After 90° rotation around X: (0, 1, 0) -> (0, 0, 1)
    assert abs(x[3] - 0.0) < 1e-10
    assert abs(y[3] - 0.0) < 1e-10
    assert abs(z[3] - 1.0) < 1e-10

    appender.close()


def test_rotate_y(simple_mesh_file):
    """Test rotation around Y axis."""
    appender = ExodusAppender.append(simple_mesh_file)

    # Rotate 90 degrees around Y axis
    # Point (1, 0, 0) should become (0, 0, -1)
    appender.rotate_y(90.0)

    x, y, z = appender.get_coords_list()

    # Find the point that was at (1, 0, 0) - that's index 1
    # After 90° rotation around Y: (1, 0, 0) -> (0, 0, -1)
    assert abs(x[1] - 0.0) < 1e-10
    assert abs(y[1] - 0.0) < 1e-10
    assert abs(z[1] - (-1.0)) < 1e-10

    appender.close()


def test_rotate_z(point_on_x_axis):
    """Test rotation around Z axis."""
    appender = ExodusAppender.append(point_on_x_axis)

    # Rotate 90 degrees around Z axis
    # Point (1, 0, 0) should become (0, 1, 0)
    appender.rotate_z(90.0)

    x, y, z = appender.get_coords_list()

    # After 90° rotation around Z: (1, 0, 0) -> (0, 1, 0)
    assert abs(x[0] - 0.0) < 1e-10
    assert abs(y[0] - 1.0) < 1e-10
    assert abs(z[0] - 0.0) < 1e-10

    appender.close()


def test_rotate_euler_extrinsic(point_on_x_axis):
    """Test Euler angle rotation with extrinsic sequence."""
    appender = ExodusAppender.append(point_on_x_axis)

    # Single 90-degree rotation around Z (extrinsic)
    appender.rotate_euler("Z", [90.0], degrees=True)

    x, y, z = appender.get_coords_list()

    # After 90° rotation around Z: (1, 0, 0) -> (0, 1, 0)
    assert abs(x[0] - 0.0) < 1e-10
    assert abs(y[0] - 1.0) < 1e-10
    assert abs(z[0] - 0.0) < 1e-10

    appender.close()


def test_rotate_euler_intrinsic(point_on_x_axis):
    """Test Euler angle rotation with intrinsic sequence."""
    appender = ExodusAppender.append(point_on_x_axis)

    # Single 90-degree rotation around z (intrinsic)
    appender.rotate_euler("z", [90.0], degrees=True)

    x, y, z = appender.get_coords_list()

    # After 90° rotation around z: (1, 0, 0) -> (0, 1, 0)
    assert abs(x[0] - 0.0) < 1e-10
    assert abs(y[0] - 1.0) < 1e-10
    assert abs(z[0] - 0.0) < 1e-10

    appender.close()


def test_rotate_euler_radians(point_on_x_axis):
    """Test Euler angle rotation with radians."""
    appender = ExodusAppender.append(point_on_x_axis)

    # 90-degree rotation using radians
    appender.rotate_euler("Z", [math.pi / 2], degrees=False)

    x, y, z = appender.get_coords_list()

    # After 90° rotation around Z: (1, 0, 0) -> (0, 1, 0)
    assert abs(x[0] - 0.0) < 1e-10
    assert abs(y[0] - 1.0) < 1e-10
    assert abs(z[0] - 0.0) < 1e-10

    appender.close()


def test_apply_rotation(point_on_x_axis):
    """Test applying a custom rotation matrix."""
    appender = ExodusAppender.append(point_on_x_axis)

    # 90-degree rotation around Z axis
    # Rotation matrix: [cos(90°), -sin(90°), 0]
    #                  [sin(90°),  cos(90°), 0]
    #                  [0,         0,        1]
    matrix = [
        0.0, -1.0, 0.0,
        1.0,  0.0, 0.0,
        0.0,  0.0, 1.0
    ]

    appender.apply_rotation(matrix)

    x, y, z = appender.get_coords_list()

    # After 90° rotation around Z: (1, 0, 0) -> (0, 1, 0)
    assert abs(x[0] - 0.0) < 1e-10
    assert abs(y[0] - 1.0) < 1e-10
    assert abs(z[0] - 0.0) < 1e-10

    appender.close()


def test_scale_uniform(simple_mesh_file):
    """Test uniform scaling."""
    appender = ExodusAppender.append(simple_mesh_file)

    # Scale by factor of 2
    appender.scale_uniform(2.0)

    x, y, z = appender.get_coords_list()

    # Point at (1, 0, 0) should become (2, 0, 0)
    assert abs(x[1] - 2.0) < 1e-10
    assert abs(y[1] - 0.0) < 1e-10
    assert abs(z[1] - 0.0) < 1e-10

    # Point at (1, 1, 1) should become (2, 2, 2)
    assert abs(x[6] - 2.0) < 1e-10
    assert abs(y[6] - 2.0) < 1e-10
    assert abs(z[6] - 2.0) < 1e-10

    appender.close()


def test_scale_non_uniform(simple_mesh_file):
    """Test non-uniform scaling."""
    appender = ExodusAppender.append(simple_mesh_file)

    # Scale by different factors: 2x in X, 3x in Y, 0.5x in Z
    appender.scale([2.0, 3.0, 0.5])

    x, y, z = appender.get_coords_list()

    # Point at (1, 1, 1) should become (2, 3, 0.5)
    assert abs(x[6] - 2.0) < 1e-10
    assert abs(y[6] - 3.0) < 1e-10
    assert abs(z[6] - 0.5) < 1e-10

    appender.close()


def test_combined_transformations(simple_mesh_file):
    """Test combining multiple transformations."""
    appender = ExodusAppender.append(simple_mesh_file)

    # First translate
    appender.translate([1.0, 0.0, 0.0])

    # Then scale
    appender.scale_uniform(2.0)

    # Then rotate
    appender.rotate_z(90.0)

    x, y, z = appender.get_coords_list()

    # Original point at (0, 0, 0)
    # After translate: (1, 0, 0)
    # After scale: (2, 0, 0)
    # After rotate 90° around Z: (0, 2, 0)
    assert abs(x[0] - 0.0) < 1e-10
    assert abs(y[0] - 2.0) < 1e-10
    assert abs(z[0] - 0.0) < 1e-10

    appender.close()


def test_euler_xyz_sequence():
    """Test XYZ Euler sequence (extrinsic)."""
    fd, path = tempfile.mkstemp(suffix='.exo')
    os.close(fd)

    try:
        writer = ExodusWriter.create(path, CreateOptions(mode=CreateMode.Clobber))
        params = InitParams(
            title="Test",
            num_dim=3,
            num_nodes=1,
            num_elems=0,
            num_elem_blocks=0
        )
        writer.put_init_params(params)
        writer.put_coords([1.0], [0.0], [0.0])
        writer.close()

        # Apply XYZ Euler rotation: 90° around each axis
        appender = ExodusAppender.append(path)
        appender.rotate_euler("XYZ", [90.0, 90.0, 90.0], degrees=True)

        x, y, z = appender.get_coords_list()

        # The result should be deterministic (verify it doesn't crash)
        assert len(x) == 1
        assert len(y) == 1
        assert len(z) == 1

        appender.close()
    finally:
        if os.path.exists(path):
            os.unlink(path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
