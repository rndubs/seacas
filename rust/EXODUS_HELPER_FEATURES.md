# Exodus-Helper Feature Comparison

This document tracks the implementation status of features from Sandia's exodus-helper package in exodus-py.

## Overview

The Sandia exodus-helper package provides Python tools for working with ExodusII files. This comparison identifies which features are missing from exodus-py and tracks their implementation status.

## Completed Features ✅

### 1. Element Volume Calculations

**Implementation**: `rust/exodus-rs/src/geometry.rs` + Python bindings

**Functions**:
- `element_volume(topology, coords)` - Compute volume of hex, tet, wedge, and pyramid elements
- `tetrahedron_volume(coords)` - Volume of tetrahedral element
- `hexahedron_volume(coords)` - Volume of hex element via tetrahedral decomposition
- `wedge_volume(coords)` - Volume of wedge/prism element
- `pyramid_volume(coords)` - Volume of pyramid element

**Supported Element Types**:
- HEX8, HEX20, HEX27 (hexahedral elements)
- TET4, TET8, TET10, TET14, TET15 (tetrahedral elements)
- WEDGE6, WEDGE15, WEDGE18 (wedge/prism elements)
- PYRAMID5, PYRAMID13, PYRAMID14 (pyramidal elements)

**Tests**:
- Rust tests: `rust/exodus-rs/src/geometry.rs` (18 tests, all passing)
- Python tests: `rust/exodus-py/tests/test_geometry.py` (comprehensive test suite)

**Use Cases**:
- Quality metrics for mesh validation
- Physics calculations requiring element volumes
- Post-processing and analysis

### 2. Element Centroid Calculations

**Implementation**: `rust/exodus-rs/src/geometry.rs` + Python bindings

**Functions**:
- `element_centroid(coords)` - Compute geometric center of any element
- `compute_face_center(coords)` - Compute face centroid (already existed)

**Tests**:
- Rust tests: Verified for hex and tet elements
- Python tests: Tests for translated, scaled, and edge-case geometries

**Use Cases**:
- Post-processing visualization
- Spatial queries
- Element location and sorting

## Pending Features 📋

### 3. Mesh Element Type Conversions ⏳

**Status**: Not yet implemented

**Required Functions**:
- `convert_tet4_tet10(filename_from, filename_to)` - Convert linear to quadratic tets

**Use Cases**:
- Mesh refinement for higher-order finite element analysis
- Converting linear elements to quadratic elements

**Priority**: Medium

**Complexity**: Medium (requires mid-edge node generation and connectivity updating)

### 4. Canonical Set Generation ⏳

**Status**: Not yet implemented

**Required Functions**:
- `create_sets_canonical(filename)` - Generate 14 node sets (6 surfaces + 8 corners) and 6 side sets for rectangular prism geometries

**Use Cases**:
- Automated boundary condition setup for structured meshes
- Standardized mesh preparation workflows

**Priority**: Medium

**Complexity**: Medium (requires surface detection logic)

### 5. Mesh Rendering and Visualization ⏳

**Status**: Not yet implemented

**Required Functions**:
- `render_mesh(mesh, surface, sampling_ratio)` - Create binary image of mesh surface
- `map_points_to_elements(mesh, points)` - Spatial mapping of points to containing elements

**Use Cases**:
- Quick visualization without external tools
- Spatial queries for analysis
- Point location for interpolation

**Priority**: Low (visualization alternatives exist)

**Complexity**: High (requires Delaunay triangulation, image generation)

### 6. Specialized Topology Utilities ⏳

**Status**: Not yet implemented

**Required Classes/Functions**:
- `RectangularPrism` class with specialized queries:
  - `get_nodes_on_surface(surface_id)`
  - `get_elems_on_surface(surface_id)`
  - `get_elems_sides_on_surface(surface_id)`
  - `get_patches_on_surface(surface_id)`
  - `get_resolution()` - Calculate element spacing
  - `get_shape()` - Determine mesh dimensions
  - Surface identification (x-, x+, y-, y+, z-, z+ planes)

**Use Cases**:
- Structured mesh queries
- Boundary extraction
- Surface-based analysis

**Priority**: Medium

**Complexity**: Medium (specialized for rectangular prisms only)

## Feature Comparison Matrix

| Feature Category | exodus-helper | exodus-py | Status |
|-----------------|---------------|-----------|--------|
| **Core Exodus II API** | ✅ | ✅ | Complete |
| **Element Volume Calculations** | ✅ | ✅ | **Complete** |
| **Element Centroid Calculations** | ✅ | ✅ | **Complete** |
| **Mesh Manipulation (exomerge)** | ✅ | ✅ | Complete |
| **TET4 to TET10 Conversion** | ✅ | ❌ | Pending |
| **Canonical Set Generation** | ✅ | ❌ | Pending |
| **Mesh Rendering** | ✅ | ❌ | Pending |
| **RectangularPrism Utilities** | ✅ | ❌ | Pending |
| **Direct NetCDF4 Access** | ✅ | ⚠️ | Architectural difference |

## Recommendations

### High Priority (Most Requested)
1. **✅ Element volume calculations** - COMPLETED
2. **✅ Element centroid calculations** - COMPLETED

### Medium Priority (Specialized but Useful)
3. TET4 to TET10 conversion - Mesh refinement workflows
4. RectangularPrism utilities - Structured mesh workflows
5. Canonical set generation - Automated setup

### Low Priority (Niche Features)
6. Mesh rendering - Visualization alternatives exist
7. Direct NetCDF4 access - Architectural decision (abstracted in exodus-rs)

## Implementation Notes

### Rust Implementation
- All volume calculations use tetrahedral decomposition
- Methods work for both regular and distorted elements
- Full error handling for invalid inputs
- Comprehensive test coverage

### Python Bindings
- Clean Pythonic API using PyO3
- Proper error messages (ValueError for invalid input)
- Type hints in docstrings
- Examples in documentation

### Performance
- Rust-backed implementation provides excellent performance
- No Python/Rust boundary overhead for batch operations
- Volume calculations are O(1) per element

## Testing

### Rust Tests
Location: `rust/exodus-rs/src/geometry.rs`

Test coverage:
- Unit tetrahedron volume (1/6)
- Unit cube volume (1.0)
- Scaled box volume (2×3×4 = 24.0)
- Wedge volume
- Pyramid volume
- Centroids for hex and tet elements

### Python Tests
Location: `rust/exodus-py/tests/test_geometry.py`

Test coverage:
- Volume calculations for all supported element types
- Centroid calculations with various geometries
- Error handling for invalid inputs
- Edge cases (empty coordinates, single points)

## Documentation

### API Documentation
- Rust documentation: `cargo doc --features netcdf4 --open`
- Python documentation: Docstrings with examples

### User Guide
Location: `rust/exodus-py/docs/user_guide.md` (to be updated)

Topics to cover:
- Element volume calculation examples
- Element centroid calculation examples
- Supported element types
- Error handling
- Performance considerations

## Summary

**Completed**: 2 of 6 feature categories (33%)

**Key Achievements**:
- ✅ High-performance element volume calculations (most requested)
- ✅ Element centroid calculations
- ✅ Full test coverage in Rust and Python
- ✅ Clean Python API with proper error handling

**Remaining Work**:
- TET4 to TET10 mesh conversion (medium priority)
- Canonical set generation (medium priority)
- Mesh rendering utilities (low priority)
- RectangularPrism topology utilities (medium priority)

The highest-priority features (element volume and centroid calculations) are now complete and fully tested, providing the most commonly requested functionality from exodus-helper.
