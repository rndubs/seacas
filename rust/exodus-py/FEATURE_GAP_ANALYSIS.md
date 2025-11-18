# Feature Gap Analysis: Sandia exodus.py vs. Rust exodus-py

**Date:** 2025-01-15
**Purpose:** Identify features in Sandia's official exodus.py (C-based) that are not yet available in the Rust-based exodus-py package

---

## Executive Summary

The Rust-based exodus-py package provides excellent coverage of core Exodus II functionality with modern APIs (Builder pattern, ExoMerge). However, the Sandia exodus.py package includes several specialized features and utilities that are currently missing. This document catalogs these gaps organized by priority and category.

**Coverage Overview:**
- ✅ **Complete:** File I/O, Coordinates, Blocks, Basic Sets, Variables, Metadata, Assemblies/Blobs
- ⚠️ **Partial:** Advanced variable operations, edge/face blocks, polyhedra
- ❌ **Missing:** Several utility functions, connectivity analysis, partial data retrieval, advanced I/O options

---

## Priority 1: Critical Missing Features

### 1.1 Partial Data Retrieval (Large Dataset Support)

**Sandia exodus.py provides:**
```python
# Partial coordinate retrieval
exo.__ex_get_partial_coord(startNodeId, numNodes)

# Partial variable retrieval
exo.get_partial_node_variable_values(name, step, start_index, num_nodes)
exo.get_partial_element_variable_values(blockId, name, step, start_index, num_elements)
exo.get_partial_node_set_variable_values(object_id, name, step, start_index, num_nodes)
exo.get_partial_side_set_variable_values(object_id, name, step, start_index, num_sides)
```

**Rust exodus-py:**
- ❌ No partial data retrieval
- Must load entire arrays into memory

**Impact:** Cannot efficiently process very large meshes (>10M nodes/elements) that don't fit in memory

**Use Cases:**
- Processing massive simulation results
- Extracting subregions from large models
- Streaming data processing

---

### 1.2 Multi-Timestep Variable Retrieval

**Sandia exodus.py provides:**
```python
# Get variable values across time range for a specific entity
exo.get_variable_values_time(objType, entityId, var_name, start_step, end_step)

# Get multi-time values for element variables
exo.get_variable_values_multi_time(objType, entityId, name, begin_step, end_step)

# C-level multi-time retrieval
exo.__ex_get_var_multi_time(start_step, end_step, varType, varId, blkId, numValues)
exo.__ex_get_var_time(varType, varId, entityID, start_step, end_step)
```

**Rust exodus-py:**
- ❌ Only single timestep retrieval
- Must iterate manually through timesteps

**Impact:** Inefficient time-series analysis and post-processing

**Use Cases:**
- Time-history extraction for specific nodes/elements
- Transient analysis visualization
- Result trend analysis

---

### 1.3 Side Set Node List Generation

**Sandia exodus.py provides:**
```python
# Get nodes that form the sides in a side set
node_list = exo.get_side_set_node_list(side_set_id)

# Query node list length
length = exo.__ex_get_side_set_node_list_len(object_id)
```

**Rust exodus-py:**
- ❌ No side set node list computation
- Only provides element+side pairs

**Impact:** Cannot easily extract boundary nodes for side sets

**Use Cases:**
- Extracting boundary nodes for BC application
- Surface mesh extraction
- Boundary layer analysis

---

### 1.4 Truth Table Management

**Sandia exodus.py provides:**
```python
# Get truth tables for any entity type
table = exo.get_element_variable_truth_table(entId=None)
table = exo.get_node_set_variable_truth_table(entId=None)
table = exo.get_side_set_variable_truth_table(entId=None)
table = exo.get_variable_truth_table(objType, entId=None)

# Set truth tables
exo.set_element_variable_truth_table(table)
exo.set_node_set_variable_truth_table(table)
exo.set_side_set_variable_truth_table(table)
exo.set_variable_truth_table(objType, table)

# Get per-entity truth vectors
vector = exo.__ex_get_object_truth_vector(objType, entId)
```

**Rust exodus-py:**
```python
# Has basic TruthTable type
table = TruthTable.new(var_type, num_blocks, num_vars)
table.set(block_idx, var_idx, exists)
value = table.get(block_idx, var_idx)

# Can write truth tables
writer.put_variable_truth_table(var_type, truth_table)
```

**Gap:**
- ❌ Cannot **read** truth tables from existing files
- ❌ No convenience methods for specific entity types
- ❌ No truth vector retrieval

**Impact:** Cannot query which variables exist in which blocks/sets before attempting retrieval

---

### 1.5 Property Management

**Sandia exodus.py provides:**
```python
# Element block properties
names = exo.get_element_property_names()
value = exo.get_element_property_value(block_id, name)
exo.put_element_property_value(block_id, name, value)

# Node set properties
names = exo.get_node_set_property_names()
value = exo.get_node_set_property_value(set_id, name)
exo.put_node_set_property_value(set_id, name, value)

# Side set properties
names = exo.get_side_set_property_names()
value = exo.get_side_set_property_value(set_id, name)
exo.put_side_set_property_value(set_id, name, value)

# Generic property access
names = exo.__ex_get_prop_names(varType, inqType)
value = exo.__ex_get_prop(objType, objId, propName)
exo.__ex_put_prop(objType, objId, propName, propVal)
```

**Rust exodus-py:**
- ❌ No property access at all
- Only ID-based entity lookup

**Impact:** Cannot work with property-based metadata (user-defined entity attributes)

**Use Cases:**
- Material property assignment
- Block categorization
- User metadata storage

---

## Priority 2: Important Missing Features

### 2.1 Inquiry System

**Sandia exodus.py provides:**
```python
# 50+ inquiry codes via ex_inquiry enum
value = exo.inquire(ex_inquiry.EX_INQ_DIM)
value = exo.inquire(ex_inquiry.EX_INQ_NODES)
value = exo.inquire(ex_inquiry.EX_INQ_TIME)
value = exo.inquire(ex_inquiry.EX_INQ_DB_MAX_ALLOWED_NAME_LENGTH)
# ... many more

# Internal inquiry functions
value = exo.__ex_inquire_int(inq_id)
value = exo.__ex_inquire_float(inq_id)
```

**Rust exodus-py:**
- ❌ No inquiry system
- Must access InitParams for basic counts
- Cannot query advanced metadata

**Impact:** Less flexible metadata queries

---

### 2.2 All Node/Element ID Maps

**Sandia exodus.py provides:**
```python
# All map types
node_map = exo.get_node_num_map()
elem_map = exo.get_elem_num_map()
elem_order = exo.get_elem_order_map()

# Generic map access
map_data = exo.get_num_map(mapType, idx)
exo.put_num_map(mapType, idx, num_map)

# Block-specific ID maps
block_map = exo.get_block_id_map(obj_type, entity_id)
```

**Rust exodus-py:**
```python
# Only basic ID maps
writer.put_id_map(entity_type, ids)
reader.get_id_map(entity_type)
```

**Gap:**
- ❌ No element order maps
- ❌ No numbered map retrieval (multiple maps of same type)
- ❌ No block-specific ID maps

---

### 2.3 Connectivity Analysis Utilities

**Sandia exodus.py provides:**
```python
# Build element connectivity list
connectivity = []
collectElemConnectivity(exo, connectivity)

# Build node-to-element mapping (inverse connectivity)
localNodeToLocalElems = {}
collectLocalNodeToLocalElems(exo, connectivity, localNodeToLocalElems)

# Build element-to-element adjacency graph
localElemToLocalElems = {}
collectLocalElemToLocalElems(exo, connectivity, localNodeToLocalElems, localElemToLocalElems)
```

**Rust exodus-py:**
- ❌ No connectivity analysis utilities
- Users must implement graph algorithms themselves

**Impact:** More difficult to perform mesh topology analysis

**Use Cases:**
- Mesh partitioning
- Element neighbor finding
- Mesh quality analysis

---

### 2.4 High-Level File Copy/Transfer Utilities

**Sandia exodus.py provides:**
```python
# Copy mesh structure only
new_exo = copy_mesh(fromFile, toFile, additionalElementAttributes=['stress'])

# Transfer variables with optional additions
transfer_variables(exoFrom, exoTo,
    additionalGlobalVariables=['energy'],
    additionalNodalVariables=[('displacement', None)],  # all blocks
    additionalElementVariables=[('stress', [1, 2])])     # specific blocks

# Add variables to existing file
add_variables(exo,
    global_vars=['time', 'energy'],
    nodal_vars=['temperature'],
    element_vars=[('stress', [1, 2, 3])])

# Complete copy with additions
new_exo = copyTransfer(fromFile, toFile,
    additionalGlobalVariables=['computed_energy'],
    additionalElementAttributes={'stress_xx': 0.0})
```

**Rust exodus-py:**
```python
# Only basic file copy
exo.copy(new_filename, include_transient=False)
```

**Gap:**
- ❌ No selective variable transfer
- ❌ Cannot add variables during copy
- ❌ No helper for adding variables to existing files

**Impact:** Complex post-processing workflows require more code

---

### 2.5 File Format and Version Information

**Sandia exodus.py provides:**
```python
# Get exodus library version
version = getExodusVersion()  # Returns integer like 808

# Get database version
db_version = exo.version_num()

# Get file type information
file_type = exo.inquire(ex_inquiry.EX_INQ_FILE_TYPE)
```

**Rust exodus-py:**
```python
# Version info
major, minor = reader.version()

# File format
format_str = reader.format()
```

**Gap:**
- ❌ No library version query
- ✅ Has database version (different API)
- ✅ Has format information

---

### 2.6 Database Summary

**Sandia exodus.py provides:**
```python
# Print comprehensive summary
exo.summarize()
```
Outputs:
- Title, dimensions, version
- Node/element/block counts
- Set counts (node, side, edge, face, elem)
- Map counts
- Variable counts (global, nodal, element, set)
- Assembly/blob counts

**Rust exodus-py:**
- ❌ No summary method
- Must manually format InitParams

---

### 2.7 Sierra Input Deck Handling

**Sandia exodus.py provides:**
```python
# Read Sierra input deck from info records
sierra_input = exo.get_sierra_input(inpFileName='input.i')
```

**Rust exodus-py:**
- ❌ No Sierra-specific functionality
- Can access info records but no parsing

**Impact:** Less integration with Sandia codes

---

## Priority 3: Advanced/Specialized Features

### 3.1 Polyhedra Support

**Sandia exodus.py provides:**
```python
# Define polyhedra element blocks
exo.put_polyhedra_elem_blk(blkID, num_polyhedra, num_faces,
    polyhedra_face_counts, polyhedra_node_counts)

# Define polyhedra face blocks
exo.put_polyhedra_face_blk(blkID, num_faces, num_nodes,
    face_node_counts, face_edge_counts)

# Set connectivity
exo.put_elem_face_conn(blkId, elemFaceConn)
exo.put_face_node_conn(blkId, faceNodeConn)
exo.put_face_count_per_polyhedra(blkID, entityCounts)
exo.put_node_count_per_face(blkID, entityCounts)
```

**Rust exodus-py:**
- ❌ No explicit polyhedra support
- Only standard element topologies

**Impact:** Cannot work with arbitrary polyhedral meshes

**Use Cases:**
- CFD with polyhedral cells
- Advanced meshing schemes
- Cut-cell methods

---

### 3.2 Edge and Face Block Support

**Sandia exodus.py provides:**
Full support for:
- Edge blocks (`EX_EDGE_BLOCK`)
- Face blocks (`EX_FACE_BLOCK`)
- Edge sets (`EX_EDGE_SET`)
- Face sets (`EX_FACE_SET`)
- Edge variables
- Face variables

**Rust exodus-py:**
```python
# Type definitions exist
EntityType.EdgeBlock
EntityType.FaceBlock
EntityType.EdgeSet
EntityType.FaceSet

# Block definition possible
Block(id, EntityType.EdgeBlock, "LINE2", ...)
```

**Gap:**
- ⚠️ Types defined but **limited testing/documentation**
- ⚠️ No edge/face-specific convenience methods
- ⚠️ Unclear if variables fully work

---

### 3.3 Coordinate Frame Support

**Sandia exodus.py provides:**
```python
# Query coordinate frames
num_frames = exo.inquire(ex_inquiry.EX_INQ_COORD_FRAMES)
```

**Rust exodus-py:**
- ❌ No coordinate frame support

**Impact:** Cannot work with multiple coordinate systems

**Use Cases:**
- Multi-body dynamics
- Cylindrical/spherical coordinate systems
- Local coordinate transformations

---

### 3.4 Concatenated Block Definition

**Sandia exodus.py provides:**
```python
# Define multiple element blocks at once (efficient)
exo.put_concat_elem_blk(
    elem_blk_ids=[1, 2, 3],
    elem_type=['HEX8', 'TET4', 'QUAD4'],
    num_blk_elems=[100, 50, 200],
    num_nodes_per_elem=[8, 4, 4],
    num_attr=[2, 1, 0]
)
```

**Rust exodus-py:**
- ❌ Must define blocks one at a time
- Less efficient for many blocks

---

### 3.5 Array Type Flexibility

**Sandia exodus.py provides:**
```python
# Choose array backend
exo = exodus('file.exo', array_type='numpy')  # Use NumPy arrays
exo = exodus('file.exo', array_type='ctype')  # Use ctypes arrays

# Conversion utility
numpy_array = ctype_to_numpy(exo, c_array)
```

**Rust exodus-py:**
- ❌ Always returns Python lists
- No NumPy array option

**Impact:** Less efficient for numerical computing (users must convert to numpy manually)

---

### 3.6 Error Handling Configuration

**Sandia exodus.py provides:**
```python
# Configure error/warning verbosity
ex_opts(ex_options.EX_VERBOSE)   # Show all messages
ex_opts(ex_options.EX_DEFAULT)   # Quiet mode
ex_opts(ex_options.EX_DEBUG)     # Debug output
ex_opts(ex_options.EX_ABORT)     # Abort on errors
```

**Rust exodus-py:**
- ❌ No error handling configuration
- Fixed error behavior

---

### 3.7 File Sharing Modes

**Sandia exodus.py provides:**
```python
# File access modes
EX_SHARE       # Shared file access
EX_NOSHARE     # Exclusive access

# Parallel I/O options
EX_MPIIO       # MPI I/O
EX_PNETCDF     # Parallel NetCDF
```

**Rust exodus-py:**
- ❌ No sharing mode control
- ❌ No parallel I/O options

**Impact:** Cannot use parallel I/O for HPC workflows

---

### 3.8 Special File Modes

**Sandia exodus.py provides:**
```python
# Memory-based I/O
EX_DISKLESS    # In-memory file
EX_MMAP        # Memory-mapped file
```

**Rust exodus-py:**
- ❌ No memory-based I/O options

---

## Priority 4: API Convenience Features

### 4.1 Context Manager Chaining

**Sandia exodus.py provides:**
```python
with exodus('file.exo', mode='r') as exo:
    # File auto-closes
    pass
```

**Rust exodus-py:**
```python
with ExodusReader.open('file.exo') as exo:
    # File auto-closes
    pass
```

**Gap:**
- ✅ Both support context managers
- ⚠️ Sandia's API allows mode parameter in single constructor
- ⚠️ Rust API requires separate classes (Reader/Writer/Appender)

**Trade-off:** Rust approach is more type-safe but less concise

---

### 4.2 Generic Variable/Set Methods

**Sandia exodus.py provides:**
```python
# Generic methods that work with any entity type
exo.get_variable_values(objType, entityId, name, step)
exo.put_variable_values(objType, entityId, name, step, values)
exo.get_variable_names(objType)
exo.get_variable_number(objType)
exo.set_variable_number(objType, number)

# Generic set operations
exo.get_set_params(object_type, object_id)
exo.put_set_params(object_type, object_id, numEntries, numDistFacts)
```

**Rust exodus-py:**
- ⚠️ Has some generic methods in ExoMerge
- ❌ No generic methods at low-level file I/O layer
- Must use specific methods (get_nodal_variable, get_element_variable, etc.)

**Trade-off:** Specific methods are more type-safe and self-documenting

---

### 4.3 File Update/Sync

**Sandia exodus.py provides:**
```python
# Flush buffers to disk
exo.__ex_update()
```

**Rust exodus-py:**
```python
# Sync to disk
writer.sync()
```

**Gap:**
- ✅ Both support flushing
- Different naming (update vs sync)

---

## Summary Tables

### Feature Coverage Matrix

| Feature Category | Sandia exodus.py | Rust exodus-py | Priority | Notes |
|-----------------|------------------|----------------|----------|-------|
| **Core I/O** |
| File Open/Create | ✅ | ✅ | - | Different API style |
| Read coordinates | ✅ | ✅ | - | Complete |
| Write coordinates | ✅ | ✅ | - | Complete |
| Partial coordinate read | ✅ | ❌ | **P1** | Missing |
| **Element Blocks** |
| Define blocks | ✅ | ✅ | - | Complete |
| Concatenated blocks | ✅ | ❌ | P3 | Minor efficiency |
| Connectivity | ✅ | ✅ | - | Complete |
| Attributes | ✅ | ✅ | - | Complete |
| Polyhedra | ✅ | ❌ | P3 | Advanced feature |
| **Sets** |
| Node sets | ✅ | ✅ | - | Complete |
| Side sets | ✅ | ✅ | - | Complete |
| Side set node list | ✅ | ❌ | **P1** | Important |
| Entity sets | ✅ | ✅ | - | Complete |
| **Variables** |
| Global variables | ✅ | ✅ | - | Complete |
| Nodal variables | ✅ | ✅ | - | Complete |
| Element variables | ✅ | ✅ | - | Complete |
| Set variables | ✅ | ✅ | - | Complete |
| Reduction variables | ✅ | ✅ | - | Complete |
| Partial variable read | ✅ | ❌ | **P1** | Critical for large data |
| Multi-timestep read | ✅ | ❌ | **P1** | Time-series analysis |
| Truth tables (read) | ✅ | ❌ | **P1** | Cannot query existence |
| Truth tables (write) | ✅ | ✅ | - | Complete |
| **Maps & IDs** |
| Basic ID maps | ✅ | ✅ | - | Complete |
| Number maps | ✅ | ❌ | P2 | Multiple maps |
| Element order map | ✅ | ❌ | P2 | Ordering info |
| Block ID maps | ✅ | ❌ | P2 | Block-specific |
| **Properties** |
| All property access | ✅ | ❌ | **P1** | No property support |
| **Metadata** |
| QA records | ✅ | ✅ | - | Complete |
| Info records | ✅ | ✅ | - | Complete |
| Title | ✅ | ✅ | - | Complete |
| Version info | ✅ | ✅ | - | Complete |
| Inquiry system | ✅ | ❌ | P2 | Alternative: InitParams |
| Summary | ✅ | ❌ | P2 | Convenience |
| **Advanced** |
| Assemblies | ✅ | ✅ | - | Complete |
| Blobs | ✅ | ⚠️ | - | Partial (read-only) |
| Coordinate frames | ✅ | ❌ | P3 | Specialized |
| **Utilities** |
| Connectivity analysis | ✅ | ❌ | P2 | Topology utilities |
| File copy | ✅ | ✅ | - | Complete |
| Variable transfer | ✅ | ❌ | P2 | Workflow helper |
| Add variables | ✅ | ❌ | P2 | Post-processing |
| ExoMerge | ❌ | ✅ | - | **Rust advantage!** |
| Builder API | ❌ | ✅ | - | **Rust advantage!** |
| **Configuration** |
| Array type (numpy) | ✅ | ❌ | P3 | User can convert |
| Error handling modes | ✅ | ❌ | P3 | Nice to have |
| Parallel I/O | ✅ | ❌ | P3 | HPC feature |
| Memory-based I/O | ✅ | ❌ | P3 | Performance |

---

## Unique Advantages of Rust exodus-py

While Sandia's exodus.py has more features, the Rust implementation has **unique advantages**:

### 1. **ExoMerge High-Level API** (150+ methods)
- ❌ **Not in Sandia exodus.py**
- ✅ Comprehensive mesh manipulation
- ✅ Geometric transformations (translate, scale, rotate, reflect)
- ✅ Block operations (merge, split, convert, duplicate)
- ✅ Advanced set operations
- ✅ Field calculations and expression evaluation
- ✅ Export to STL/VRML
- ✅ Mesh quality analysis

### 2. **Modern Builder API**
- ❌ **Not in Sandia exodus.py**
- ✅ Fluent interface for mesh creation
- ✅ Type-safe construction
- ✅ Reduced boilerplate code

### 3. **Performance Features**
- ❌ **Not in Sandia exodus.py**
- ✅ HPC-aware configuration (CacheConfig, ChunkConfig)
- ✅ Automatic node type detection
- ✅ Optimized I/O profiles

### 4. **Type Safety**
- ✅ Strong typing via Rust
- ✅ Better error messages
- ✅ Less runtime errors

### 5. **Pure Rust Backend**
- ✅ No C library dependency issues
- ✅ Cross-platform consistency
- ✅ Modern HDF5/NetCDF4

---

## Recommendations

### High Priority (Should Implement)

1. **Partial data retrieval** - Essential for large meshes
2. **Multi-timestep variable retrieval** - Common in post-processing
3. **Truth table reading** - Needed to query variable existence
4. **Property management** - Standard metadata mechanism
5. **Side set node list** - Common boundary analysis operation

### Medium Priority (Nice to Have)

6. **Inquiry system** - Alternative exists (InitParams) but less flexible
7. **Connectivity analysis utilities** - Helpful but users can implement
8. **Variable transfer utilities** - Workflow convenience
9. **File summary method** - User experience improvement

### Low Priority (Specialized)

10. **Polyhedra support** - Niche use case
11. **Coordinate frames** - Rarely used
12. **Parallel I/O** - Future performance feature
13. **Array type selection** - User can convert to NumPy

### Not Recommended

- **Error handling configuration** - Modern approach uses exceptions
- **Concatenated block definition** - Minor optimization
- **Sierra-specific features** - Too application-specific

---

## Conclusion

The Rust exodus-py package provides excellent core functionality with modern APIs that often surpass the Sandia implementation (Builder, ExoMerge). However, there are **5 critical gaps** related to large dataset handling and metadata that should be addressed for feature parity:

1. Partial data retrieval
2. Multi-timestep operations
3. Truth table reading
4. Property management
5. Side set node list generation

Implementing these would make exodus-py a complete replacement for most use cases while maintaining its advantages in type safety, performance, and high-level APIs.
