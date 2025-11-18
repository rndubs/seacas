"""
Tests for geometry utilities (element volumes and centroids)
"""

import pytest
import math


# Import from installed package after building
try:
    from exodus import element_volume, element_centroid
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
