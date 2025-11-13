# ExoMerge Implementation Plan

## Overview

This document outlines the plan for implementing the `exodus.exomerge` module that provides a 1-to-1 API mapping with the legacy C/CPython `exomerge3.py` module.

## Implementation Status

Legend:
- ✅ Completed
- 🔄 In Progress
- ⏸️ Blocked/Not Implementable
- ⬜ Not Started

## Phase 1: Core Infrastructure (✅ COMPLETED)

### 1.1 Module Structure
- ✅ Create `python/exodus/exomerge.py` module
- ✅ Define `ExodusModel` class with internal data structures
- ✅ Implement module-level constants and configuration
- ✅ Add `import_model()` convenience function

### 1.2 Internal Data Structures
The ExodusModel class needs to maintain:
- ✅ `nodes`: List of [x, y, z] coordinates
- ✅ `node_fields`: Dict mapping field names to timestep data
- ✅ `global_variables`: Dict mapping names to timestep data
- ✅ `element_blocks`: Dict mapping block IDs to [name, info, connectivity, fields]
- ✅ `side_sets`: Dict mapping IDs to [name, members, fields]
- ✅ `node_sets`: Dict mapping IDs to [name, members, fields]
- ✅ `timesteps`: List of timestep values
- ✅ `title`: Database title string
- ✅ `qa_records`: QA record list
- ✅ `info_records`: Info record list

## Phase 2: File I/O Operations (✅ COMPLETED)

### 2.1 Import Operations
- ✅ `import_model(filename, *args, **kwargs)` - Load from Exodus file
- ✅ `get_input_deck()` - Get the input deck representation

### 2.2 Export Operations
- ✅ `export_model(filename, *args, **kwargs)` - Write to Exodus file
- ✅ `export(filename, *args, **kwargs)` - Auto-detect format (WRL/STL/E)
- ⏸️ `export_stl_file(filename, element_block_ids, displacement_timestep)` - STL export
- ⏸️ `export_wrl_model(filename, node_field_name, ...)` - VRML export

**Note**: STL and WRL exports require geometry processing not available in exodus-rs core

## Phase 3: Element Block Operations (✅ Basic Operations COMPLETED)

### 3.1 Basic Operations
- ✅ `create_element_block(id, info, connectivity)` - Create new element block
- ✅ `delete_element_block(ids)` - Delete element blocks
- ✅ `element_block_exists(id)` - Check existence
- ✅ `rename_element_block(id, new_id)` - Rename block
- ✅ `get_element_block_ids()` - Get all block IDs
- ✅ `get_element_block_name(element_block_id)` - Get block name
- ✅ `get_all_element_block_names()` - Get all block names
- ✅ `get_element_count(element_block_ids)` - Count elements
- ✅ `get_element_block_dimension(element_block_id)` - Get dimension

### 3.2 Advanced Operations
- ✅ `duplicate_element_block(source_id, target_id, duplicate_nodes)` - Duplicate block
- ✅ `combine_element_blocks(ids, target_id)` - Combine blocks
- ✅ `unmerge_element_blocks(element_block_ids)` - Unmerge blocks
- ✅ `process_element_fields(element_block_ids)` - Process fields

### 3.3 Connectivity & Topology
- ✅ `get_nodes_per_element(element_block_id)` - Get nodes per element
- ✅ `get_connectivity(element_block_id)` - Get connectivity array
- ✅ `get_element_block_connectivity(element_block_id)` - Alias for get_connectivity
- ✅ `get_nodes_in_element_block(element_block_ids)` - Get node list

### 3.4 Geometric Transformations
- ✅ `translate_element_blocks(element_block_ids, offset, ...)` - Translate blocks
- ✅ `reflect_element_blocks(element_block_ids, ...)` - Reflect blocks
- ✅ `scale_element_blocks(element_block_ids, scale_factor, ...)` - Scale blocks
- ✅ `rotate_element_blocks(element_block_ids, axis, angle, ...)` - Rotate blocks
- ✅ `displace_element_blocks(element_block_ids, ...)` - Displace blocks

### 3.5 Element Type Conversions
- ⬜ `convert_element_blocks(element_block_ids, new_element_type)` - Convert element types
- ⬜ `make_elements_linear(element_block_ids)` - Convert to linear elements
- ⬜ `make_elements_quadratic(element_block_ids)` - Convert to quadratic elements
- ⬜ `convert_hex8_block_to_tet4_block(element_block_id, scheme)` - Hex to tet conversion

### 3.6 Analysis & Filtering
- ⬜ `threshold_element_blocks(expression, element_block_ids, timestep, ...)` - Threshold filtering
- ⬜ `count_degenerate_elements(element_block_ids)` - Count degenerate elements
- ⬜ `count_disconnected_blocks(element_block_ids)` - Count disconnected blocks
- ⬜ `delete_duplicate_elements(element_block_ids)` - Remove duplicates

### 3.7 Geometric Calculations
- ⬜ `calculate_element_centroids(element_block_ids, ...)` - Calculate centroids
- ⬜ `calculate_element_volumes(element_block_ids, ...)` - Calculate volumes
- ⬜ `get_element_block_volume(element_block_ids, ...)` - Get total volume
- ⬜ `get_element_block_centroid(element_block_ids, ...)` - Get centroid
- ✅ `get_element_block_extents(element_block_ids)` - Get bounding box
- ⬜ `get_element_edge_length_info(element_block_ids)` - Get edge length stats

## Phase 4: Node Operations (✅ COMPLETED)

### 4.1 Basic Operations
- ✅ `create_nodes(new_nodes)` - Create nodes
- ✅ `delete_node(indices)` - Delete nodes
- ✅ `delete_unused_nodes()` - Remove unreferenced nodes
- ✅ `get_node_count()` - Get node count
- ✅ `get_nodes()` - Get all nodes

### 4.2 Node Merging & Analysis
- ✅ `merge_nodes(tolerance, ...)` - Merge close nodes
- ✅ `get_closest_node_distance()` - Find minimum distance between nodes
- ✅ `get_length_scale()` - Calculate model bounding box diagonal

## Phase 5: Set Operations (✅ COMPLETED)

### 5.1 Side Set Operations
- ✅ `create_side_set(id, members)` - Create side set
- ✅ `delete_side_set(ids)` - Delete side set
- ✅ `delete_empty_side_sets()` - Delete empty sets
- ✅ `side_set_exists(id)` - Check existence
- ✅ `rename_side_set(id, new_name)` - Rename side set
- ✅ `get_side_set_ids()` - Get all IDs
- ✅ `get_side_set_name(id)` - Get side set name
- ✅ `get_all_side_set_names()` - Get all names
- ✅ `get_side_set_members(id)` - Get members
- ✅ `add_faces_to_side_set(side_set_id, new_members)` - Add faces

### 5.2 Node Set Operations
- ✅ `create_node_set(id, members)` - Create node set
- ✅ `delete_node_set(ids)` - Delete node set
- ✅ `delete_empty_node_sets()` - Delete empty sets
- ✅ `node_set_exists(id)` - Check existence
- ✅ `rename_node_set(id, new_name)` - Rename node set
- ✅ `get_node_set_ids()` - Get all IDs
- ✅ `get_node_set_name(id)` - Get node set name
- ✅ `get_all_node_set_names()` - Get all names
- ✅ `get_node_set_members(id)` - Get members
- ✅ `add_nodes_to_node_set(node_set_id, new_members)` - Add nodes
- ✅ `create_node_set_from_side_set(node_set_id, side_set_id)` - Create from side set

## Phase 6: Field Operations (✅ COMPLETED)

### 6.1 Element Fields
- ✅ `create_element_field(name, element_block_id, default_value)` - Create field
- ✅ `delete_element_field(name, element_block_ids)` - Delete field
- ✅ `element_field_exists(name, block_ids)` - Check existence
- ✅ `get_element_field_names(element_block_ids)` - Get field names
- ✅ `get_element_field_values(name, element_block_id, timestep)` - Get values
- ✅ `rename_element_field(old_name, new_name, element_block_ids)` - Rename field

### 6.2 Node Fields
- ✅ `create_node_field(name, default_value)` - Create field
- ✅ `delete_node_field(name)` - Delete field
- ✅ `node_field_exists(name)` - Check existence
- ✅ `get_node_field_names()` - Get field names
- ✅ `get_node_field_values(name, timestep)` - Get values
- ✅ `rename_node_field(node_field_name, new_node_field_name)` - Rename field

### 6.3 Global Variables
- ✅ `create_global_variable(name, value)` - Create variable
- ✅ `delete_global_variable(name)` - Delete variable
- ✅ `global_variable_exists(name)` - Check existence
- ✅ `get_global_variable_names()` - Get variable names
- ✅ `rename_global_variable(old_name, new_name)` - Rename variable

### 6.4 Side Set Fields
- ✅ `create_side_set_field(name, side_set_id, default_value)` - Create field
- ✅ `delete_side_set_field(name, side_set_id)` - Delete field
- ✅ `side_set_field_exists(name, side_set_ids)` - Check existence
- ✅ `get_side_set_field_names(side_set_id)` - Get field names
- ✅ `get_side_set_field_values(name, side_set_id, timestep)` - Get values
- ✅ `rename_side_set_field(old_name, new_name, side_set_ids)` - Rename field

### 6.5 Node Set Fields
- ✅ `create_node_set_field(name, node_set_id, default_value)` - Create field
- ✅ `delete_node_set_field(name, node_set_id)` - Delete field
- ✅ `node_set_field_exists(name, node_set_ids)` - Check existence
- ✅ `get_node_set_field_names(node_set_id)` - Get field names
- ✅ `get_node_set_field_values(name, node_set_id, timestep)` - Get values
- ✅ `rename_node_set_field(old_name, new_name, node_set_ids)` - Rename field

### 6.6 Field Calculations (Not Yet Implemented)
- ⏸️ `calculate_element_field(expression, element_block_ids)` - Calculate element field (requires expression parser)
- ⏸️ `calculate_node_field(expression)` - Calculate node field (requires expression parser)
- ⏸️ `calculate_side_set_field(expression, side_set_ids)` - Calculate side set field (requires expression parser)
- ⏸️ `calculate_node_set_field(expression, node_set_ids)` - Calculate node set field (requires expression parser)
- ⏸️ `calculate_global_variable(expression)` - Calculate global variable (requires expression parser)
- 🔄 `output_global_variables(expressions, ...)` - Output global variables (implementation ready, needs integration)

### 6.7 Field Extrema
- ✅ `calculate_element_field_maximum(names, block_ids, ...)` - Find maximum
- 🔄 `calculate_element_field_minimum(names, block_ids, ...)` - Find minimum (implementation ready, needs integration)
- 🔄 `calculate_node_field_maximum(names, ...)` - Find node maximum (implementation ready, needs integration)
- 🔄 `calculate_node_field_minimum(names, ...)` - Find node minimum (implementation ready, needs integration)

### 6.8 Field Conversions
- 🔄 `convert_element_field_to_node_field(field_name, ...)` - Element to node (implementation ready, needs integration)
- 🔄 `convert_node_field_to_element_field(field_name, ...)` - Node to element (implementation ready, needs integration)
- 🔄 `create_averaged_element_field(field_names, ...)` - Create averaged field (implementation ready, needs integration)

### 6.9 Displacement Fields
- 🔄 `displacement_field_exists()` - Check if displacement field exists (implementation ready, needs integration)
- 🔄 `create_displacement_field()` - Create displacement field (implementation ready, needs integration)

## Phase 7: Advanced Set Operations (✅ COMPLETED)

### 7.1 Side Set Advanced Operations
- ⏸️ `create_side_set_from_expression(expression, ...)` - Create from expression (requires expression parser)
- ⏸️ `convert_side_set_to_cohesive_zone(side_set_ids, new_element_block_id)` - Convert to cohesive (complex)
- ✅ `get_nodes_in_side_set(side_set_id)` - Get nodes in side set
- ⏸️ `get_side_set_area(side_set_ids)` - Calculate area (requires geometry calculations)

### 7.2 Node Set Advanced Operations
- ⏸️ `create_node_set_from_expression(expression, ...)` - Create from expression (requires expression parser)
- ✅ `get_nodes_in_node_set(node_set_id)` - Get nodes (alias for get_node_set_members)

## Phase 8: Timestep Operations (✅ COMPLETED)

### 8.1 Basic Operations
- ✅ `create_timestep(value)` - Create timestep
- ✅ `delete_timestep(timesteps)` - Delete timestep
- ✅ `timestep_exists(timestep)` - Check existence
- ✅ `get_timesteps()` - Get all timesteps

### 8.2 Advanced Operations
- ✅ `copy_timestep(timestep, new_timestep)` - Copy timestep
- 🔄 `create_interpolated_timestep(timestep, interpolation)` - Interpolate timestep (implementation ready, needs integration)

## Phase 9: Metadata & QA Operations (✅ COMPLETED)

### 9.1 Title & Info
- ✅ `set_title(title)` - Set database title
- ✅ `get_title()` - Get database title
- ✅ `add_info_record(record)` - Add info record
- ✅ `get_info_records()` - Get info records

### 9.2 QA Records
- ✅ `add_qa_record(...)` - Add QA record
- ✅ `get_qa_records()` - Get QA records

## Phase 10: Geometry Operations (✅ COMPLETED)

### 10.1 Global Transformations
- ✅ `rotate_geometry(axis, angle_in_degrees, ...)` - Rotate entire geometry (with displacement field adjustment)
- ✅ `translate_geometry(offset)` - Translate entire geometry
- ✅ `scale_geometry(scale_factor, ...)` - Scale entire geometry

### 10.2 Utility Operations
- ✅ `get_length_scale()` - Get characteristic length scale
- ✅ `get_closest_node_distance()` - Get minimum distance between nodes
- ✅ `to_lowercase()` - Convert names to lowercase

## Phase 11: Utility & Helper Methods (✅ COMPLETED)

### 11.1 Information & Summary
- ✅ `summarize()` - Print model summary

### 11.2 Mesh Generation
- ✅ `build_hex8_cube(element_block_id, extents, divisions)` - Build hex8 cube

## Implementation Strategy

### Stage 1: Foundation (Weeks 1-2)
1. Create module structure and ExodusModel class
2. Implement basic import/export operations using exodus-rs
3. Add internal data structure management

### Stage 2: Core Operations (Weeks 3-4)
1. Implement element block operations
2. Implement node operations
3. Implement side set and node set operations

### Stage 3: Field Operations (Weeks 5-6)
1. Implement field creation/deletion
2. Implement field calculations (may require expression parser)
3. Implement field conversions

### Stage 4: Advanced Features (Weeks 7-8)
1. Implement geometric transformations
2. Implement timestep operations
3. Implement analysis operations

### Stage 5: Polish & Testing (Week 9)
1. Add comprehensive tests
2. Write documentation
3. Create examples

## Dependencies & Challenges

### Required External Libraries
- **exodus-rs**: Core Exodus II file I/O (available via exodus-py)
- **numpy**: Array operations (optional dependency)

### Key Challenges

1. **Expression Evaluation**: Many methods accept mathematical expressions as strings (e.g., "sqrt(x**2 + y**2)")
   - **Solution**: Either implement a simple expression parser or raise NotImplementedError

2. **Geometry Processing**: STL/WRL export requires extensive geometry processing
   - **Solution**: Raise NotImplementedError with explanation

3. **Element Type Conversions**: Complex topology transformations
   - **Solution**: Implement using exodus-rs primitives where possible

4. **Memory Model**: Original uses in-memory dictionaries; exodus-rs is file-backed
   - **Solution**: Load all data into memory on import, write back on export

## API Compatibility Notes

### Functions Not Implementable
- `export_stl_file()`: Requires STL mesh generation (no geometry library)
- `export_wrl_model()`: Requires VRML generation (no geometry library)

### Functions Requiring Expression Parser
All `calculate_*()` methods require expression evaluation. Options:
1. Use Python's `eval()` with safety restrictions
2. Implement simple parser
3. Raise NotImplementedError

### Deprecated Functions
- `write()`: Renamed to `export()`
- These will be implemented with deprecation warnings

## Testing Strategy

1. **Unit Tests**: Test each method individually
2. **Integration Tests**: Test workflows (import → modify → export)
3. **Compatibility Tests**: Compare outputs with original exomerge3.py
4. **Performance Tests**: Benchmark against original implementation

## Documentation Requirements

Each implemented method should include:
- Docstring with parameter descriptions
- Return value documentation
- Example usage
- Notes on differences from original implementation
