"""
Tests for geometry utilities (element volumes and centroids)
"""

import pytest
import math
import tempfile
import os


# Import from installed package after building
try:
    from exodus import element_volume, element_centroid, ExodusReader, MeshBuilder, BlockBuilder
    EXODUS_AVAILABLE = True
except ImportError:
    EXODUS_AVAILABLE = False
    pytestmark = pytest.mark.skip("exodus module not built")


class TestElementVolume:
    """Tests for element_volume function"""

    def test_hex8_unit_cube(self):
        """Test volume of a unit cube"""
        coords = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ]
        volume = element_volume("HEX8", coords)
        assert abs(volume - 1.0) < 1e-10, f"Expected 1.0, got {volume}"

    def test_hex8_scaled(self):
        """Test volume of a scaled box (2x3x4)"""
        coords = [
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 3.0, 0.0],
            [0.0, 3.0, 0.0],
            [0.0, 0.0, 4.0],
            [2.0, 0.0, 4.0],
            [2.0, 3.0, 4.0],
            [0.0, 3.0, 4.0],
        ]
        volume = element_volume("HEX8", coords)
        expected = 2.0 * 3.0 * 4.0
        assert abs(volume - expected) < 1e-10, f"Expected {expected}, got {volume}"

    def test_tet4_unit(self):
        """Test volume of a unit tetrahedron"""
        coords = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
        volume = element_volume("TET4", coords)
        expected = 1.0 / 6.0
        assert abs(volume - expected) < 1e-10, f"Expected {expected}, got {volume}"

    def test_wedge6_unit(self):
        """Test volume of a unit wedge"""
        coords = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
        ]
        volume = element_volume("WEDGE6", coords)
        # Wedge volume = base_area * height = (1/2 * 1 * 1) * 1 = 0.5
        expected = 0.5
        assert abs(volume - expected) < 1e-10, f"Expected {expected}, got {volume}"

    def test_pyramid5_unit(self):
        """Test volume of a pyramid"""
        coords = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ]
        volume = element_volume("PYRAMID5", coords)
        # Pyramid volume = (1/3) * base_area * height = (1/3) * 1 * 1 = 1/3
        expected = 1.0 / 3.0
        assert abs(volume - expected) < 0.01, f"Expected ~{expected}, got {volume}"

    def test_invalid_topology(self):
        """Test error handling for unsupported topology"""
        coords = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]
        # SPHERE is not supported for volume calculations
        with pytest.raises(Exception):  # Should raise RuntimeError
            element_volume("SPHERE", coords)

    def test_insufficient_coords(self):
        """Test error handling for insufficient coordinates"""
        coords = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]  # Only 2 coords for HEX8
        with pytest.raises(Exception):  # Should raise RuntimeError
            element_volume("HEX8", coords)

    def test_invalid_coord_length(self):
        """Test error handling for invalid coordinate dimensions"""
        coords = [[0.0, 0.0], [1.0, 0.0]]  # Only 2D coords
        with pytest.raises(Exception):  # Should raise ValueError
            element_volume("TET4", coords)


class TestElementCentroid:
    """Tests for element_centroid function"""

    def test_hex8_unit_cube(self):
        """Test centroid of a unit cube"""
        coords = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ]
        centroid = element_centroid(coords)
        assert len(centroid) == 3
        assert abs(centroid[0] - 0.5) < 1e-10
        assert abs(centroid[1] - 0.5) < 1e-10
        assert abs(centroid[2] - 0.5) < 1e-10

    def test_tet4_centroid(self):
        """Test centroid of a tetrahedron"""
        coords = [
            [0.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [0.0, 3.0, 0.0],
            [0.0, 0.0, 3.0],
        ]
        centroid = element_centroid(coords)
        # Centroid is average of vertices: (3/4, 3/4, 3/4)
        assert abs(centroid[0] - 0.75) < 1e-10
        assert abs(centroid[1] - 0.75) < 1e-10
        assert abs(centroid[2] - 0.75) < 1e-10

    def test_translated_hex(self):
        """Test centroid of a translated hexahedron"""
        # Unit cube translated by (5, 10, 15)
        offset = [5.0, 10.0, 15.0]
        coords = [
            [0.0 + offset[0], 0.0 + offset[1], 0.0 + offset[2]],
            [1.0 + offset[0], 0.0 + offset[1], 0.0 + offset[2]],
            [1.0 + offset[0], 1.0 + offset[1], 0.0 + offset[2]],
            [0.0 + offset[0], 1.0 + offset[1], 0.0 + offset[2]],
            [0.0 + offset[0], 0.0 + offset[1], 1.0 + offset[2]],
            [1.0 + offset[0], 0.0 + offset[1], 1.0 + offset[2]],
            [1.0 + offset[0], 1.0 + offset[1], 1.0 + offset[2]],
            [0.0 + offset[0], 1.0 + offset[1], 1.0 + offset[2]],
        ]
        centroid = element_centroid(coords)
        assert abs(centroid[0] - (0.5 + offset[0])) < 1e-10
        assert abs(centroid[1] - (0.5 + offset[1])) < 1e-10
        assert abs(centroid[2] - (0.5 + offset[2])) < 1e-10

    def test_empty_coords(self):
        """Test centroid with empty coordinates returns origin"""
        coords = []
        centroid = element_centroid(coords)
        assert centroid == [0.0, 0.0, 0.0]

    def test_single_point(self):
        """Test centroid of a single point"""
        coords = [[1.5, 2.5, 3.5]]
        centroid = element_centroid(coords)
        assert centroid == [1.5, 2.5, 3.5]


class TestBlockElementVolumes:
    """Tests for ExodusReader.block_element_volumes method"""

    def test_single_block_volumes(self):
        """Test computing volumes for all elements in a single block"""
        with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as f:
            mesh_file = f.name

        try:
            # Create a simple mesh with 2 unit cubes
            builder = MeshBuilder("Test Mesh")
            builder.dimensions(3)
            builder.coordinates(
                x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0,  # First cube
                   2.0, 3.0, 3.0, 2.0, 2.0, 3.0, 3.0, 2.0], # Second cube
                y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0,
                   0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
                z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0,
                   0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
            )

            block = BlockBuilder(100, "HEX8")
            block.connectivity([
                1, 2, 3, 4, 5, 6, 7, 8,    # First cube
                9, 10, 11, 12, 13, 14, 15, 16  # Second cube
            ])
            builder.add_block(block.build())
            builder.write(mesh_file)

            # Read and compute volumes
            with ExodusReader.open(mesh_file) as reader:
                volumes = reader.block_element_volumes(100)

                assert len(volumes) == 2, f"Expected 2 volumes, got {len(volumes)}"
                assert abs(volumes[0] - 1.0) < 1e-10, f"Expected 1.0, got {volumes[0]}"
                assert abs(volumes[1] - 1.0) < 1e-10, f"Expected 1.0, got {volumes[1]}"
        finally:
            if os.path.exists(mesh_file):
                os.unlink(mesh_file)

    def test_multi_block_volumes(self):
        """Test computing volumes for specific block in multi-block mesh"""
        with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as f:
            mesh_file = f.name

        try:
            # Create a mesh with 2 blocks (hex and tet)
            builder = MeshBuilder("Multi-Block Test")
            builder.dimensions(3)
            builder.coordinates(
                x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0,  # Hex nodes 1-8
                   2.0, 3.0, 2.5, 2.5],  # Tet nodes 9-12
                y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0,
                   0.0, 0.0, 1.0, 0.5],
                z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0,
                   0.0, 0.0, 0.0, 1.0]
            )

            # Block 1: 1 hex
            hex_block = BlockBuilder(100, "HEX8")
            hex_block.connectivity([1, 2, 3, 4, 5, 6, 7, 8])
            builder.add_block(hex_block.build())

            # Block 2: 1 tet
            tet_block = BlockBuilder(200, "TET4")
            tet_block.connectivity([9, 10, 11, 12])
            builder.add_block(tet_block.build())

            builder.write(mesh_file)

            # Test volumes for each block separately
            with ExodusReader.open(mesh_file) as reader:
                hex_volumes = reader.block_element_volumes(100)
                tet_volumes = reader.block_element_volumes(200)

                assert len(hex_volumes) == 1
                assert abs(hex_volumes[0] - 1.0) < 1e-10

                assert len(tet_volumes) == 1
                assert tet_volumes[0] > 0.0  # Just verify it's positive
        finally:
            if os.path.exists(mesh_file):
                os.unlink(mesh_file)


class TestBlockElementCentroids:
    """Tests for ExodusReader.block_element_centroids method"""

    def test_single_block_centroids(self):
        """Test computing centroids for all elements in a single block"""
        with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as f:
            mesh_file = f.name

        try:
            # Create a simple mesh with 2 unit cubes at different positions
            builder = MeshBuilder("Test Mesh")
            builder.dimensions(3)
            builder.coordinates(
                x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0,  # First cube at origin
                   2.0, 3.0, 3.0, 2.0, 2.0, 3.0, 3.0, 2.0], # Second cube offset
                y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0,
                   0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
                z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0,
                   0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
            )

            block = BlockBuilder(100, "HEX8")
            block.connectivity([
                1, 2, 3, 4, 5, 6, 7, 8,
                9, 10, 11, 12, 13, 14, 15, 16
            ])
            builder.add_block(block.build())
            builder.write(mesh_file)

            # Read and compute centroids
            with ExodusReader.open(mesh_file) as reader:
                centroids = reader.block_element_centroids(100)

                assert len(centroids) == 2
                # First cube centroid at (0.5, 0.5, 0.5)
                assert abs(centroids[0][0] - 0.5) < 1e-10
                assert abs(centroids[0][1] - 0.5) < 1e-10
                assert abs(centroids[0][2] - 0.5) < 1e-10
                # Second cube centroid at (2.5, 0.5, 0.5)
                assert abs(centroids[1][0] - 2.5) < 1e-10
                assert abs(centroids[1][1] - 0.5) < 1e-10
                assert abs(centroids[1][2] - 0.5) < 1e-10
        finally:
            if os.path.exists(mesh_file):
                os.unlink(mesh_file)


class TestAllElementVolumes:
    """Tests for ExodusReader.all_element_volumes method"""

    def test_all_volumes_single_block(self):
        """Test computing volumes for entire mesh (single block)"""
        with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as f:
            mesh_file = f.name

        try:
            # Create mesh with 3 elements in one block
            builder = MeshBuilder("Test Mesh")
            builder.dimensions(3)
            # Simple coordinates for 3 unit cubes
            builder.coordinates(
                x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0,
                   1.0, 2.0, 2.0, 1.0, 1.0, 2.0, 2.0, 1.0,
                   2.0, 3.0, 3.0, 2.0, 2.0, 3.0, 3.0, 2.0],
                y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0] * 3,
                z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0] * 3
            )

            block = BlockBuilder(100, "HEX8")
            block.connectivity([
                1, 2, 3, 4, 5, 6, 7, 8,
                9, 10, 11, 12, 13, 14, 15, 16,
                17, 18, 19, 20, 21, 22, 23, 24
            ])
            builder.add_block(block.build())
            builder.write(mesh_file)

            # Read and compute all volumes
            with ExodusReader.open(mesh_file) as reader:
                all_volumes = reader.all_element_volumes()

                assert len(all_volumes) == 3
                for vol in all_volumes:
                    assert abs(vol - 1.0) < 1e-10
        finally:
            if os.path.exists(mesh_file):
                os.unlink(mesh_file)

    def test_all_volumes_multi_block(self):
        """Test computing volumes for entire mesh (multiple blocks)"""
        with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as f:
            mesh_file = f.name

        try:
            # Create mesh with 2 blocks
            builder = MeshBuilder("Multi-Block Test")
            builder.dimensions(3)
            builder.coordinates(
                x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0,
                   2.0, 3.0, 2.5, 2.5],
                y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0,
                   0.0, 0.0, 1.0, 0.5],
                z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0,
                   0.0, 0.0, 0.0, 1.0]
            )

            # Block 1: 1 hex
            hex_block = BlockBuilder(100, "HEX8")
            hex_block.connectivity([1, 2, 3, 4, 5, 6, 7, 8])
            builder.add_block(hex_block.build())

            # Block 2: 1 tet
            tet_block = BlockBuilder(200, "TET4")
            tet_block.connectivity([9, 10, 11, 12])
            builder.add_block(tet_block.build())

            builder.write(mesh_file)

            # Read and compute all volumes
            with ExodusReader.open(mesh_file) as reader:
                all_volumes = reader.all_element_volumes()

                # Should have 2 elements total (1 hex + 1 tet)
                assert len(all_volumes) == 2
                # First element (hex) should be 1.0
                assert abs(all_volumes[0] - 1.0) < 1e-10
                # Second element (tet) should be positive
                assert all_volumes[1] > 0.0
        finally:
            if os.path.exists(mesh_file):
                os.unlink(mesh_file)


class TestAllElementCentroids:
    """Tests for ExodusReader.all_element_centroids method"""

    def test_all_centroids_single_block(self):
        """Test computing centroids for entire mesh (single block)"""
        with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as f:
            mesh_file = f.name

        try:
            # Create mesh with 2 elements
            builder = MeshBuilder("Test Mesh")
            builder.dimensions(3)
            builder.coordinates(
                x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0,
                   2.0, 3.0, 3.0, 2.0, 2.0, 3.0, 3.0, 2.0],
                y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0,
                   0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
                z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0,
                   0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
            )

            block = BlockBuilder(100, "HEX8")
            block.connectivity([
                1, 2, 3, 4, 5, 6, 7, 8,
                9, 10, 11, 12, 13, 14, 15, 16
            ])
            builder.add_block(block.build())
            builder.write(mesh_file)

            # Read and compute all centroids
            with ExodusReader.open(mesh_file) as reader:
                all_centroids = reader.all_element_centroids()

                assert len(all_centroids) == 2
                # First cube at (0.5, 0.5, 0.5)
                assert abs(all_centroids[0][0] - 0.5) < 1e-10
                assert abs(all_centroids[0][1] - 0.5) < 1e-10
                assert abs(all_centroids[0][2] - 0.5) < 1e-10
                # Second cube at (2.5, 0.5, 0.5)
                assert abs(all_centroids[1][0] - 2.5) < 1e-10
                assert abs(all_centroids[1][1] - 0.5) < 1e-10
                assert abs(all_centroids[1][2] - 0.5) < 1e-10
        finally:
            if os.path.exists(mesh_file):
                os.unlink(mesh_file)

    def test_all_centroids_multi_block(self):
        """Test computing centroids for entire mesh (multiple blocks)"""
        with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as f:
            mesh_file = f.name

        try:
            # Create mesh with 2 blocks (hex and quad)
            builder = MeshBuilder("Multi-Block Test")
            builder.dimensions(3)
            builder.coordinates(
                x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0,
                   2.0, 3.0, 3.0, 2.0],
                y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0,
                   0.0, 0.0, 1.0, 1.0],
                z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0,
                   0.0, 0.0, 0.0, 0.0]
            )

            # Block 1: 1 hex
            hex_block = BlockBuilder(100, "HEX8")
            hex_block.connectivity([1, 2, 3, 4, 5, 6, 7, 8])
            builder.add_block(hex_block.build())

            # Block 2: 1 quad
            quad_block = BlockBuilder(200, "QUAD4")
            quad_block.connectivity([9, 10, 11, 12])
            builder.add_block(quad_block.build())

            builder.write(mesh_file)

            # Read and compute all centroids
            with ExodusReader.open(mesh_file) as reader:
                all_centroids = reader.all_element_centroids()

                # Should have 2 elements (1 hex + 1 quad)
                assert len(all_centroids) == 2
                # First element (hex) at (0.5, 0.5, 0.5)
                assert abs(all_centroids[0][0] - 0.5) < 1e-10
                assert abs(all_centroids[0][1] - 0.5) < 1e-10
                assert abs(all_centroids[0][2] - 0.5) < 1e-10
                # Second element (quad) at (2.5, 0.5, 0.0)
                assert abs(all_centroids[1][0] - 2.5) < 1e-10
                assert abs(all_centroids[1][1] - 0.5) < 1e-10
                assert abs(all_centroids[1][2] - 0.0) < 1e-10
        finally:
            if os.path.exists(mesh_file):
                os.unlink(mesh_file)
