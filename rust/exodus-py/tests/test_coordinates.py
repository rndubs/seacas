"""
Tests for coordinate operations
"""

import pytest
import tempfile
import os

pytest.importorskip("exodus")

from exodus import ExodusWriter, ExodusReader, InitParams, CreateMode, CreateOptions


def test_put_and_get_coords_2d():
    """Test writing and reading 2D coordinates"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        # Create file and write coordinates
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="2D Coords", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        x_coords = [0.0, 1.0, 1.0, 0.0]
        y_coords = [0.0, 0.0, 1.0, 1.0]
        z_coords = []
        writer.put_coords(x_coords, y_coords, z_coords)
        writer.close()

        # Read back coordinates
        reader = ExodusReader.open(tmp_path)
        x_read, y_read, z_read = reader.get_coords()
        assert len(x_read) == 4
        assert len(y_read) == 4
        assert len(z_read) == 0
        assert x_read == pytest.approx(x_coords, abs=1e-6)
        assert y_read == pytest.approx(y_coords, abs=1e-6)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_put_and_get_coords_3d():
    """Test writing and reading 3D coordinates"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        # Create file and write coordinates
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="3D Coords", num_dim=3, num_nodes=8)
        writer.put_init_params(params)

        x_coords = [0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
        y_coords = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0]
        z_coords = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
        writer.put_coords(x_coords, y_coords, z_coords)
        writer.close()

        # Read back coordinates
        reader = ExodusReader.open(tmp_path)
        x_read, y_read, z_read = reader.get_coords()
        assert len(x_read) == 8
        assert len(y_read) == 8
        assert len(z_read) == 8
        assert x_read == pytest.approx(x_coords, abs=1e-6)
        assert y_read == pytest.approx(y_coords, abs=1e-6)
        assert z_read == pytest.approx(z_coords, abs=1e-6)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_coord_names():
    """Test setting and reading coordinate names"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        # Create file with coordinate names
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Coord Names", num_dim=3, num_nodes=8)
        writer.put_init_params(params)

        # Set coordinate names
        coord_names = ["X", "Y", "Z"]
        writer.put_coord_names(coord_names)

        # Write coordinates
        x_coords = [0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0]
        y_coords = [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0]
        z_coords = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
        writer.put_coords(x_coords, y_coords, z_coords)
        writer.close()

        # Read back coordinate names
        reader = ExodusReader.open(tmp_path)
        names_read = reader.get_coord_names()
        assert len(names_read) == 3
        assert names_read == coord_names
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_empty_coords():
    """Test file with zero nodes"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Empty Coords", num_dim=2, num_nodes=0)
        writer.put_init_params(params)
        writer.close()

        reader = ExodusReader.open(tmp_path)
        x, y, z = reader.get_coords()
        assert len(x) == 0
        assert len(y) == 0
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_large_coordinate_set():
    """Test with larger number of nodes"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        num_nodes = 1000
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Large Coords", num_dim=3, num_nodes=num_nodes)
        writer.put_init_params(params)

        x_coords = [float(i) for i in range(num_nodes)]
        y_coords = [float(i * 2) for i in range(num_nodes)]
        z_coords = [float(i * 3) for i in range(num_nodes)]
        writer.put_coords(x_coords, y_coords, z_coords)
        writer.close()

        reader = ExodusReader.open(tmp_path)
        x_read, y_read, z_read = reader.get_coords()
        assert len(x_read) == num_nodes
        assert x_read == pytest.approx(x_coords, abs=1e-6)
        assert y_read == pytest.approx(y_coords, abs=1e-6)
        assert z_read == pytest.approx(z_coords, abs=1e-6)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
