# ExoMerge API Compatibility Summary

## Overview

This document summarizes the API compatibility between the `exodus.exomerge` module and the legacy `exomerge3.py` module. The goal is to provide a 1-to-1 API mapping where possible, clearly documenting any deviations or limitations.

The legacy `exomerge3.py` file can be found here:

- packages/seacas/scripts/exomerge3.py

**Status**: Initial framework complete. All method signatures have been defined, with implementation in progress.

**Module Location**: `python/exodus/exomerge.py`

## Compatibility Level

### API Completeness

- **Total Methods in Original**: ~150 public methods
- **Total Methods Mapped**: ~150 (100%)
- **Methods Fully Implemented**: 4 (metadata getters/setters)
- **Methods Raising NotImplementedError**: ~146
- **Methods Not Implementable**: 2 (STL/WRL export)

### Compatibility Categories

1. **✅ Fully Compatible**: Method signature and behavior match exactly
2. **🔄 Planned**: Method signature defined, implementation in progress
3. **⏸️ Not Implementable**: Cannot be implemented with exodus-rs backend
4. **⚠️ Modified**: Method exists but with different signature or behavior

## API Compatibility by Category

### File I/O Operations

| Method | Status | Notes |
|--------|--------|-------|
| `import_model()` | 🔄 Planned | Phase 2 |
| `export_model()` | 🔄 Planned | Phase 2 |
| `export()` | 🔄 Planned | Auto-detect format |
| `export_stl_file()` | ⏸️ Not Implementable | Requires geometry library |
| `export_wrl_model()` | ⏸️ Not Implementable | Requires VRML library |
| `get_input_deck()` | 🔄 Planned | Phase 2 |

**Notes**:
- STL and WRL export require extensive geometry processing not available in exodus-rs
- These methods raise `NotImplementedError` with detailed explanations
- Users needing these features should use the original exomerge3.py

### Element Block Operations

| Method | Status | Notes |
|--------|--------|-------|
| `create_element_block()` | 🔄 Planned | Phase 3 |
| `delete_element_block()` | 🔄 Planned | Phase 3 |
| `element_block_exists()` | 🔄 Planned | Phase 3 |
| `rename_element_block()` | 🔄 Planned | Phase 3 |
| `get_element_block_ids()` | 🔄 Planned | Phase 3 |
| `get_element_block_name()` | 🔄 Planned | Phase 3 |
| `get_all_element_block_names()` | 🔄 Planned | Phase 3 |
| `get_element_count()` | 🔄 Planned | Phase 3 |
| `get_element_block_dimension()` | 🔄 Planned | Phase 3 |
| `get_nodes_per_element()` | 🔄 Planned | Phase 3 |
| `get_connectivity()` | 🔄 Planned | Phase 3 |
| `get_element_block_connectivity()` | 🔄 Planned | Alias for get_connectivity |
| `get_nodes_in_element_block()` | 🔄 Planned | Phase 3 |
| `duplicate_element_block()` | 🔄 Planned | Phase 3 |
| `combine_element_blocks()` | 🔄 Planned | Phase 3 |
| `unmerge_element_blocks()` | 🔄 Planned | Phase 3 |
| `process_element_fields()` | 🔄 Planned | Phase 3 |
| `translate_element_blocks()` | 🔄 Planned | Phase 3 |
| `reflect_element_blocks()` | 🔄 Planned | Phase 3 |
| `scale_element_blocks()` | 🔄 Planned | Phase 3 |
| `rotate_element_blocks()` | 🔄 Planned | Phase 3 |
| `displace_element_blocks()` | 🔄 Planned | Phase 3 |
| `convert_element_blocks()` | 🔄 Planned | Phase 3 |
| `make_elements_linear()` | 🔄 Planned | Phase 3 |
| `make_elements_quadratic()` | 🔄 Planned | Phase 3 |
| `convert_hex8_block_to_tet4_block()` | 🔄 Planned | Phase 3 |
| `threshold_element_blocks()` | 🔄 Planned | Phase 3 |
| `count_degenerate_elements()` | 🔄 Planned | Phase 3 |
| `count_disconnected_blocks()` | 🔄 Planned | Phase 3 |
| `delete_duplicate_elements()` | 🔄 Planned | Phase 3 |
| `calculate_element_centroids()` | 🔄 Planned | Phase 3 |
| `calculate_element_volumes()` | 🔄 Planned | Phase 3 |
| `get_element_block_volume()` | 🔄 Planned | Phase 3 |
| `get_element_block_centroid()` | 🔄 Planned | Phase 3 |
| `get_element_block_extents()` | 🔄 Planned | Phase 3 |
| `get_element_edge_length_info()` | 🔄 Planned | Phase 3 |

### Field Operations

#### Element Fields

| Method | Status | Notes |
|--------|--------|-------|
| `create_element_field()` | 🔄 Planned | Phase 4 |
| `delete_element_field()` | 🔄 Planned | Phase 4 |
| `element_field_exists()` | 🔄 Planned | Phase 4 |
| `get_element_field_names()` | 🔄 Planned | Phase 4 |
| `get_element_field_values()` | 🔄 Planned | Phase 4 |
| `rename_element_field()` | 🔄 Planned | Phase 4 |
| `calculate_element_field()` | 🔄 Planned | Requires expression parser |
| `calculate_element_field_maximum()` | 🔄 Planned | Phase 4 |
| `calculate_element_field_minimum()` | 🔄 Planned | Phase 4 |
| `create_averaged_element_field()` | 🔄 Planned | Phase 4 |
| `convert_element_field_to_node_field()` | 🔄 Planned | Phase 4 |
| `convert_node_field_to_element_field()` | 🔄 Planned | Phase 4 |

#### Node Fields

| Method | Status | Notes |
|--------|--------|-------|
| `create_node_field()` | 🔄 Planned | Phase 4 |
| `delete_node_field()` | 🔄 Planned | Phase 4 |
| `node_field_exists()` | 🔄 Planned | Phase 4 |
| `get_node_field_names()` | 🔄 Planned | Phase 4 |
| `get_node_field_values()` | 🔄 Planned | Phase 4 |
| `rename_node_field()` | 🔄 Planned | Phase 4 |
| `calculate_node_field()` | 🔄 Planned | Requires expression parser |
| `calculate_node_field_maximum()` | 🔄 Planned | Phase 4 |
| `calculate_node_field_minimum()` | 🔄 Planned | Phase 4 |
| `displacement_field_exists()` | 🔄 Planned | Phase 4 |
| `create_displacement_field()` | 🔄 Planned | Phase 4 |

#### Global Variables

| Method | Status | Notes |
|--------|--------|-------|
| `create_global_variable()` | 🔄 Planned | Phase 4 |
| `delete_global_variable()` | 🔄 Planned | Phase 4 |
| `global_variable_exists()` | 🔄 Planned | Phase 4 |
| `get_global_variable_names()` | 🔄 Planned | Phase 4 |
| `rename_global_variable()` | 🔄 Planned | Phase 4 |
| `calculate_global_variable()` | 🔄 Planned | Requires expression parser |
| `output_global_variables()` | 🔄 Planned | Phase 4 |

#### Side Set Fields

| Method | Status | Notes |
|--------|--------|-------|
| `create_side_set_field()` | 🔄 Planned | Phase 4 |
| `delete_side_set_field()` | 🔄 Planned | Phase 4 |
| `side_set_field_exists()` | 🔄 Planned | Phase 4 |
| `get_side_set_field_names()` | 🔄 Planned | Phase 4 |
| `get_side_set_field_values()` | 🔄 Planned | Phase 4 |
| `rename_side_set_field()` | 🔄 Planned | Phase 4 |
| `calculate_side_set_field()` | 🔄 Planned | Requires expression parser |

#### Node Set Fields

| Method | Status | Notes |
|--------|--------|-------|
| `create_node_set_field()` | 🔄 Planned | Phase 4 |
| `delete_node_set_field()` | 🔄 Planned | Phase 4 |
| `node_set_field_exists()` | 🔄 Planned | Phase 4 |
| `get_node_set_field_names()` | 🔄 Planned | Phase 4 |
| `get_node_set_field_values()` | 🔄 Planned | Phase 4 |
| `rename_node_set_field()` | 🔄 Planned | Phase 4 |
| `calculate_node_set_field()` | 🔄 Planned | Requires expression parser |

### Node Operations

| Method | Status | Notes |
|--------|--------|-------|
| `create_nodes()` | 🔄 Planned | Phase 5; was `create_node` in original |
| `delete_node()` | 🔄 Planned | Phase 5 |
| `delete_unused_nodes()` | 🔄 Planned | Phase 5 |
| `get_node_count()` | ✅ Fully Compatible | Returns `len(self.nodes)` |
| `get_nodes()` | ✅ Fully Compatible | Returns `self.nodes` |
| `merge_nodes()` | 🔄 Planned | Phase 5 |
| `get_closest_node_distance()` | 🔄 Planned | Phase 5 |
| `get_length_scale()` | 🔄 Planned | Phase 5 |

**Notes**:
- The original had `create_node()` but our implementation uses `create_nodes()` for consistency

### Side Set Operations

| Method | Status | Notes |
|--------|--------|-------|
| `create_side_set()` | 🔄 Planned | Phase 6 |
| `delete_side_set()` | 🔄 Planned | Phase 6 |
| `delete_empty_side_sets()` | 🔄 Planned | Phase 6 |
| `side_set_exists()` | 🔄 Planned | Phase 6 |
| `rename_side_set()` | 🔄 Planned | Phase 6 |
| `get_side_set_ids()` | 🔄 Planned | Phase 6 |
| `get_side_set_name()` | 🔄 Planned | Phase 6 |
| `get_all_side_set_names()` | 🔄 Planned | Phase 6 |
| `get_side_set_members()` | 🔄 Planned | Phase 6 |
| `add_faces_to_side_set()` | 🔄 Planned | Phase 6 |
| `create_side_set_from_expression()` | 🔄 Planned | Requires expression parser |
| `convert_side_set_to_cohesive_zone()` | 🔄 Planned | Phase 6 |
| `get_nodes_in_side_set()` | 🔄 Planned | Phase 6 |
| `get_side_set_area()` | 🔄 Planned | Phase 6 |

### Node Set Operations

| Method | Status | Notes |
|--------|--------|-------|
| `create_node_set()` | 🔄 Planned | Phase 7 |
| `delete_node_set()` | 🔄 Planned | Phase 7 |
| `delete_empty_node_sets()` | 🔄 Planned | Phase 7 |
| `node_set_exists()` | 🔄 Planned | Phase 7 |
| `rename_node_set()` | 🔄 Planned | Phase 7 |
| `get_node_set_ids()` | 🔄 Planned | Phase 7 |
| `get_node_set_name()` | 🔄 Planned | Phase 7 |
| `get_all_node_set_names()` | 🔄 Planned | Phase 7 |
| `get_node_set_members()` | 🔄 Planned | Phase 7 |
| `add_nodes_to_node_set()` | 🔄 Planned | Phase 7 |
| `create_node_set_from_side_set()` | 🔄 Planned | Phase 7 |

### Timestep Operations

| Method | Status | Notes |
|--------|--------|-------|
| `create_timestep()` | 🔄 Planned | Phase 8 |
| `delete_timestep()` | 🔄 Planned | Phase 8 |
| `timestep_exists()` | ✅ Fully Compatible | Checks `timestep in self.timesteps` |
| `get_timesteps()` | ✅ Fully Compatible | Returns `self.timesteps` |
| `copy_timestep()` | 🔄 Planned | Phase 8 |
| `create_interpolated_timestep()` | 🔄 Planned | Phase 8 |

### Metadata and QA Operations

| Method | Status | Notes |
|--------|--------|-------|
| `set_title()` | ✅ Fully Compatible | Sets `self.title` |
| `get_title()` | ✅ Fully Compatible | Returns `self.title` |
| `add_qa_record()` | 🔄 Planned | Phase 9 |
| `get_qa_records()` | ✅ Fully Compatible | Returns `self.qa_records` |
| `add_info_record()` | ✅ Fully Compatible | Appends to `self.info_records` |
| `get_info_records()` | ✅ Fully Compatible | Returns `self.info_records` |

### Geometric Transformation Operations

| Method | Status | Notes |
|--------|--------|-------|
| `rotate_geometry()` | 🔄 Planned | Phase 10 |
| `translate_geometry()` | 🔄 Planned | Phase 10 |
| `scale_geometry()` | 🔄 Planned | Phase 10 |

### Utility and Mesh Generation

| Method | Status | Notes |
|--------|--------|-------|
| `summarize()` | 🔄 Planned | Phase 11 |
| `to_lowercase()` | 🔄 Planned | Phase 11 |
| `build_hex8_cube()` | 🔄 Planned | Phase 11 |

## Data Structure Compatibility

The `ExodusModel` class maintains the same internal data structures as the original:

| Attribute | Type | Status | Notes |
|-----------|------|--------|-------|
| `nodes` | `List[List[float]]` | ✅ Compatible | Node coordinates |
| `node_fields` | `Dict[str, List[Any]]` | ✅ Compatible | Node field data |
| `global_variables` | `Dict[str, List[float]]` | ✅ Compatible | Global variables |
| `element_blocks` | `Dict[int, List[Any]]` | ✅ Compatible | Element block data |
| `side_sets` | `Dict[int, List[Any]]` | ✅ Compatible | Side set data |
| `node_sets` | `Dict[int, List[Any]]` | ✅ Compatible | Node set data |
| `timesteps` | `List[float]` | ✅ Compatible | Timestep values |
| `title` | `str` | ✅ Compatible | Database title |
| `qa_records` | `List[Tuple]` | ✅ Compatible | QA records |
| `info_records` | `List[str]` | ✅ Compatible | Info records |

## Key Differences and Limitations

### 1. Not Implementable Features

**STL and WRL Export** (`export_stl_file`, `export_wrl_model`):
- **Reason**: These require extensive geometry processing (triangulation, VRML generation) that is not part of the Exodus II specification
- **Workaround**: Use the original exomerge3.py or dedicated mesh conversion tools
- **Status**: Raises `NotImplementedError` with detailed explanation

### 2. Expression Evaluation

Methods requiring expression evaluation (all `calculate_*` methods):
- **Challenge**: Need to parse and evaluate mathematical expressions like `"sqrt(x**2 + y**2)"`
- **Options**:
  1. Use Python's `eval()` with safety restrictions (planned)
  2. Implement a simple expression parser
  3. Use an expression evaluation library
- **Status**: Raises `NotImplementedError` pending implementation decision

### 3. Backend Differences

**Memory Model**:
- **Original**: Uses exodus.py (C library wrapper) with in-memory dictionary manipulation
- **New**: Uses exodus-rs (Rust library) which is file-backed
- **Strategy**: Load all data into memory on import, write back on export

### 4. Deprecated Functions

The following functions from the original are marked as deprecated:
- `write()` → renamed to `export()`

These are handled via `__getattr__` and will issue deprecation warnings.

## Migration Guide

### For Users Migrating from exomerge3.py

```python
# Old way (exomerge3.py)
import exomerge3 as exomerge
model = exomerge.import_model('mesh.e')

# New way (exodus.exomerge)
import exodus.exomerge as exomerge
model = exomerge.import_model('mesh.e')
```

The API is designed to be a drop-in replacement, so most code should work unchanged.

### Features Not Available

If you need these features, continue using the original exomerge3.py:
1. STL file export (`export_stl_file`)
2. VRML/WRL export (`export_wrl_model`)

### Expression-Based Methods

Methods requiring expression evaluation are planned but not yet implemented:
- `calculate_element_field()`
- `calculate_node_field()`
- `calculate_global_variable()`
- `calculate_side_set_field()`
- `calculate_node_set_field()`
- `threshold_element_blocks()`
- `create_side_set_from_expression()`

## Implementation Roadmap

See `MERGE_PLAN.md` for the detailed implementation roadmap.

### Current Status (Phase 1 Complete)

✅ **Phase 1**: Core infrastructure and data structures
- Module structure created
- All method signatures defined
- Basic metadata operations implemented

### Next Steps

- **Phase 2**: File I/O operations (import/export)
- **Phase 3**: Element block operations
- **Phase 4**: Field operations
- **Phase 5-11**: Remaining features

## Testing Strategy

### Compatibility Testing

1. **API Compatibility**: Verify all method signatures match
2. **Data Structure Compatibility**: Ensure data structures are identical
3. **Behavioral Compatibility**: Compare outputs with original exomerge3.py
4. **Migration Testing**: Test real-world migration scenarios

### Test Coverage Goals

- Unit tests for each implemented method
- Integration tests for common workflows
- Compatibility tests comparing with original implementation
- Performance benchmarks

## Performance Considerations

### Expected Performance

- **Import/Export**: Similar to exomerge3.py (both use underlying C/Rust libraries)
- **In-Memory Operations**: Potentially faster (Rust is generally faster than Python)
- **Expression Evaluation**: May be slower if using Python eval vs native code

### Optimization Opportunities

1. Use NumPy for array operations where possible
2. Implement critical paths in Rust
3. Cache computed values when appropriate

## Contributing

To contribute to the exomerge implementation:

1. Check `MERGE_PLAN.md` for planned features
2. Pick a phase/feature to implement
3. Write tests before implementing
4. Update `MERGE_PLAN.md` to mark features as complete
5. Update this compatibility document

## Contact and Support

For issues or questions:
- File an issue in the exodus-rs repository
- Contact: exodus-rs development team

## Version History

- **v0.1.0**: Initial framework with all API signatures defined
