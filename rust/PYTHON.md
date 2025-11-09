# Python Bindings for exodus-rs

## Overview

This document outlines the plan for creating Python bindings for the exodus-rs Rust library. The bindings will expose the public-facing portions of the exodus-rs API to Python, with a focus on the high-level builder API and essential file operations.

## Goals

1. **Expose Builder API**: Provide ergonomic Python interface using `MeshBuilder` and `BlockBuilder`
2. **File Operations**: Support creating, opening, and reading Exodus files
3. **Data Access**: Enable reading/writing coordinates, blocks, sets, and metadata
4. **Type Safety**: Maintain Rust's type safety while providing Pythonic interfaces
5. **Error Handling**: Convert Rust errors to Python exceptions appropriately
6. **Documentation**: Provide comprehensive docstrings and examples

## Technology Stack

- **PyO3**: Rust bindings for Python (https://pyo3.rs/)
- **maturin**: Build tool for PyO3 projects
- **Python 3.8+**: Target Python version

## API Surface Reference

### Existing Python API (exodus.py)
The current C-based Python bindings expose:
- File operations: `exodus()` constructor, `close()`
- Initialization: `put_init_params()`, `get_init_params()`
- Coordinates: `get_coords()`, `put_coords()`
- Element blocks: `get_ids()`, `get_block()`, `put_block()`, `get_elem_connectivity()`
- Node/Side sets: `get_node_set()`, `put_node_set()`, `get_side_set()`
- Variables: `get_node_variable_values()`, `put_node_variable_values()`
- Maps: `get_id_map()`, `put_id_map()`
- Metadata: `get_qa_records()`, `put_qa_records()`, `get_info_records()`
- Assemblies/Blobs: `get_assembly()`, `put_assembly()`, `get_blob()`
- Entity types: Enums for EX_ELEM_BLOCK, EX_NODE_SET, etc.

### Rust exodus-rs Public API
Core types and modules to expose:
- **Builder API**: `MeshBuilder`, `BlockBuilder`
- **File Operations**: `ExodusFile<Read>`, `ExodusFile<Write>`, `ExodusFile<Append>`
- **Core Types**: `EntityType`, `InitParams`, `Block`, `Set`, `NodeSet`, `SideSet`, `EntitySet`, `Assembly`, `Blob`
- **Enums**: `CreateMode`, `FloatSize`, `Int64Mode`, `FileFormat`, `AttributeType`
- **Coordinates**: `Coordinates`, coordinate read/write methods
- **Metadata**: `QaRecord`, `InfoRecord`

## Implementation Plan

### ✅ Phase 1: Project Setup
- [x] Research existing Python API
- [x] Research Rust API structure
- [ ] Create detailed implementation plan
- [ ] Set up PyO3 project structure
- [ ] Configure maturin build system
- [ ] Add Python bindings directory structure

**Files to create:**
- `./rust/exodus-py/` - Python bindings package directory
- `./rust/exodus-py/Cargo.toml` - Package configuration
- `./rust/exodus-py/pyproject.toml` - Python project metadata
- `./rust/exodus-py/src/lib.rs` - Main bindings entry point
- `./rust/exodus-py/README.md` - Python bindings documentation

**Estimated time:** 1 hour

---

### Phase 2: Core Type Bindings
Expose fundamental Rust types as Python classes.

**Tasks:**
- [ ] Implement `EntityType` enum binding
  - [ ] Map Rust enum variants to Python enum
  - [ ] Implement string conversions
  - [ ] Add docstrings

- [ ] Implement `InitParams` struct binding
  - [ ] Create Python class with properties
  - [ ] Implement `__init__` with default values
  - [ ] Add validation for dimensions (1, 2, or 3)
  - [ ] Implement `__repr__` for debugging

- [ ] Implement `CreateOptions` binding
  - [ ] Expose `CreateMode` enum (Clobber, NoClobber)
  - [ ] Expose `FloatSize` enum (Float32, Float64)
  - [ ] Expose `Int64Mode` enum (Int32, Int64)
  - [ ] Implement builder pattern or keyword arguments

- [ ] Implement `Block` struct binding
  - [ ] Create Python class with properties
  - [ ] Implement validation

- [ ] Implement `Set` types bindings
  - [ ] `NodeSet` class
  - [ ] `SideSet` class
  - [ ] `EntitySet` class

- [ ] Implement `Assembly` and `Blob` bindings
  - [ ] `Assembly` class
  - [ ] `Blob` class

**Files to create/modify:**
- `./rust/exodus-py/src/types.rs` - Type bindings implementation
- `./rust/exodus-py/tests/test_types.py` - Python tests for types

**Estimated time:** 4 hours

---

### Phase 3: File Operations
Expose file creation, opening, and basic operations.

**Tasks:**
- [ ] Implement `ExodusFile` Python wrapper
  - [ ] Create separate classes for different modes: `ExodusReader`, `ExodusWriter`, `ExodusAppender`
  - [ ] Implement `create()` class method
  - [ ] Implement `open()` class method
  - [ ] Implement `append()` class method
  - [ ] Implement context manager protocol (`__enter__`, `__exit__`)
  - [ ] Add `path` property
  - [ ] Add `close()` method

- [ ] Implement initialization methods
  - [ ] `init()` - Initialize database parameters
  - [ ] `init_params()` - Read initialization parameters

- [ ] Error handling
  - [ ] Create `ExodusError` exception class hierarchy
  - [ ] Map Rust errors to Python exceptions
  - [ ] Add helpful error messages

**Files to create/modify:**
- `./rust/exodus-py/src/file.rs` - File operations implementation
- `./rust/exodus-py/src/error.rs` - Error handling
- `./rust/exodus-py/tests/test_file_ops.py` - File operation tests

**Example usage:**
```python
from exodus import ExodusWriter, CreateOptions, CreateMode

# Create a file
with ExodusWriter.create("mesh.exo", CreateOptions(mode=CreateMode.CLOBBER)) as exo:
    exo.init(params)
    # ... write data ...

# Open for reading
with ExodusReader.open("mesh.exo") as exo:
    params = exo.init_params()
    print(f"Mesh has {params.num_nodes} nodes")
```

**Estimated time:** 5 hours

---

### Phase 4: Coordinate Operations
Expose coordinate read/write functionality.

**Tasks:**
- [ ] Implement `put_coords()` method
  - [ ] Accept Python lists or numpy arrays
  - [ ] Handle 1D, 2D, and 3D coordinates
  - [ ] Validate coordinate lengths match

- [ ] Implement `get_coords()` method
  - [ ] Return as numpy arrays (if numpy available)
  - [ ] Return as Python lists (fallback)
  - [ ] Support getting individual coordinate arrays (x, y, z)

- [ ] Implement coordinate names
  - [ ] `put_coord_names()` method
  - [ ] `get_coord_names()` method

**Files to create/modify:**
- `./rust/exodus-py/src/coord.rs` - Coordinate operations
- `./rust/exodus-py/tests/test_coords.py` - Coordinate tests

**Example usage:**
```python
# Write coordinates
x = [0.0, 1.0, 1.0, 0.0]
y = [0.0, 0.0, 1.0, 1.0]
z = [0.0, 0.0, 0.0, 0.0]
exo.put_coords(x, y, z)

# Read coordinates
x, y, z = exo.get_coords()
```

**Estimated time:** 3 hours

---

### Phase 5: Block Operations
Expose element block functionality.

**Tasks:**
- [ ] Implement `put_block()` method
  - [ ] Accept `Block` objects
  - [ ] Validate block parameters

- [ ] Implement `get_block()` method
  - [ ] Return `Block` object

- [ ] Implement `get_block_ids()` method
  - [ ] Return list of block IDs

- [ ] Implement connectivity operations
  - [ ] `put_connectivity()` method
  - [ ] `get_connectivity()` method

- [ ] Implement block attributes
  - [ ] `put_block_attributes()` method
  - [ ] `get_block_attributes()` method
  - [ ] `put_block_attribute_names()` method
  - [ ] `get_block_attribute_names()` method

**Files to create/modify:**
- `./rust/exodus-py/src/block.rs` - Block operations
- `./rust/exodus-py/tests/test_blocks.py` - Block tests

**Example usage:**
```python
from exodus import Block, EntityType

# Define a block
block = Block(
    id=100,
    entity_type=EntityType.ELEM_BLOCK,
    topology="HEX8",
    num_entries=10,
    num_nodes_per_entry=8
)

exo.put_block(block)
exo.put_connectivity(100, connectivity_array)
```

**Estimated time:** 4 hours

---

### Phase 6: Set Operations
Expose node set and side set functionality.

**Tasks:**
- [ ] Implement node set operations
  - [ ] `put_node_set()` method
  - [ ] `get_node_set()` method
  - [ ] `get_node_set_ids()` method

- [ ] Implement side set operations
  - [ ] `put_side_set()` method
  - [ ] `get_side_set()` method
  - [ ] `get_side_set_ids()` method

- [ ] Implement entity set operations
  - [ ] `put_entity_set()` method (for edge/face/elem sets)
  - [ ] `get_entity_set()` method

- [ ] Implement set names
  - [ ] `put_set_name()` method
  - [ ] `get_set_name()` method

**Files to create/modify:**
- `./rust/exodus-py/src/set.rs` - Set operations
- `./rust/exodus-py/tests/test_sets.py` - Set tests

**Example usage:**
```python
from exodus import NodeSet

# Create and write a node set
ns = NodeSet(
    id=10,
    nodes=[1, 2, 3, 4, 5],
    dist_factors=[]
)
exo.put_node_set(ns)

# Read a node set
ns = exo.get_node_set(10)
```

**Estimated time:** 4 hours

---

### Phase 7: Builder API (High Priority)
Expose the ergonomic `MeshBuilder` and `BlockBuilder` API.

**Tasks:**
- [ ] Implement `BlockBuilder` Python wrapper
  - [ ] `__init__()` with id and topology
  - [ ] `connectivity()` method (fluent interface)
  - [ ] `attributes()` method
  - [ ] `attribute_names()` method
  - [ ] `build()` method

- [ ] Implement `MeshBuilder` Python wrapper
  - [ ] `__init__()` with title
  - [ ] `dimensions()` method (fluent interface)
  - [ ] `coordinates()` method
  - [ ] `add_block()` method
  - [ ] `qa_record()` method
  - [ ] `info()` method
  - [ ] `write()` method
  - [ ] `write_with_options()` method

- [ ] Add comprehensive examples
  - [ ] Simple 2D quad mesh
  - [ ] 3D hex mesh
  - [ ] Multi-block mesh
  - [ ] Mesh with attributes

**Files to create/modify:**
- `./rust/exodus-py/src/builder.rs` - Builder API bindings
- `./rust/exodus-py/tests/test_builder.py` - Builder tests
- `./rust/exodus-py/examples/simple_mesh.py` - Example script

**Example usage (key feature):**
```python
from exodus import MeshBuilder, BlockBuilder

# Create a mesh using fluent API
(MeshBuilder("Simple Quad Mesh")
    .dimensions(2)
    .coordinates(
        x=[0.0, 1.0, 1.0, 0.0],
        y=[0.0, 0.0, 1.0, 1.0],
        z=[]
    )
    .add_block(
        BlockBuilder(1, "QUAD4")
            .connectivity([1, 2, 3, 4])
            .build()
    )
    .write("output.exo"))
```

**Estimated time:** 5 hours

---

### Phase 8: Metadata Operations
Expose QA records, info records, and entity naming.

**Tasks:**
- [ ] Implement QA record operations
  - [ ] `put_qa_records()` method
  - [ ] `get_qa_records()` method
  - [ ] Create `QaRecord` Python class

- [ ] Implement info record operations
  - [ ] `put_info_records()` method
  - [ ] `get_info_records()` method

- [ ] Implement entity naming
  - [ ] `put_name()` method
  - [ ] `get_name()` method
  - [ ] `put_names()` method
  - [ ] `get_names()` method

**Files to create/modify:**
- `./rust/exodus-py/src/metadata.rs` - Metadata operations
- `./rust/exodus-py/tests/test_metadata.py` - Metadata tests

**Example usage:**
```python
from exodus import QaRecord

qa = QaRecord(
    code_name="MyApp",
    code_version="1.0.0",
    date="2025-01-15",
    time="14:30:00"
)
exo.put_qa_records([qa])
```

**Estimated time:** 3 hours

---

### Phase 9: Map Operations
Expose ID maps and order maps.

**Tasks:**
- [ ] Implement ID map operations
  - [ ] `put_id_map()` method (node, elem, etc.)
  - [ ] `get_id_map()` method

- [ ] Implement order map operations
  - [ ] `put_order_map()` method
  - [ ] `get_order_map()` method

- [ ] Implement numbered map operations
  - [ ] `put_num_map()` method
  - [ ] `get_num_map()` method

**Files to create/modify:**
- `./rust/exodus-py/src/map.rs` - Map operations
- `./rust/exodus-py/tests/test_maps.py` - Map tests

**Estimated time:** 3 hours

---

### Phase 10: Assembly and Blob Operations
Expose advanced hierarchical grouping features.

**Tasks:**
- [ ] Implement Assembly operations
  - [ ] `put_assembly()` method
  - [ ] `get_assembly()` method
  - [ ] `get_assembly_ids()` method

- [ ] Implement Blob operations
  - [ ] `put_blob()` method
  - [ ] `get_blob()` method
  - [ ] `get_blob_ids()` method

**Files to create/modify:**
- `./rust/exodus-py/src/assembly.rs` - Assembly operations
- `./rust/exodus-py/src/blob.rs` - Blob operations
- `./rust/exodus-py/tests/test_assembly_blob.py` - Tests

**Example usage:**
```python
from exodus import Assembly, EntityType

assembly = Assembly(
    id=1,
    name="LeftWing",
    entity_type=EntityType.ELEM_BLOCK,
    entity_list=[100, 101, 102]
)
exo.put_assembly(assembly)
```

**Estimated time:** 3 hours

---

### Phase 11: Variable Operations (Optional, Future Work)
If time permits, expose variable read/write functionality.

**Tasks:**
- [ ] Implement time step operations
  - [ ] `put_time()` method
  - [ ] `get_times()` method

- [ ] Implement variable definition
  - [ ] `put_variable_param()` method
  - [ ] `put_variable_names()` method
  - [ ] `get_variable_names()` method

- [ ] Implement variable I/O
  - [ ] `put_nodal_var()` method
  - [ ] `get_nodal_var()` method
  - [ ] `put_elem_var()` method
  - [ ] `get_elem_var()` method
  - [ ] `put_global_var()` method
  - [ ] `get_global_var()` method

**Files to create/modify:**
- `./rust/exodus-py/src/variable.rs` - Variable operations
- `./rust/exodus-py/tests/test_variables.py` - Variable tests

**Estimated time:** 6 hours

---

### Phase 12: NumPy Integration
Enhance integration with NumPy for efficient array operations.

**Tasks:**
- [ ] Add numpy feature flag
- [ ] Implement automatic numpy array conversion
  - [ ] Convert Python lists to Rust Vec
  - [ ] Convert numpy arrays to Rust Vec
  - [ ] Return numpy arrays from Rust

- [ ] Add type stubs for better IDE support
  - [ ] Create `.pyi` stub files
  - [ ] Add type hints to docstrings

**Files to create/modify:**
- `./rust/exodus-py/src/numpy_support.rs` - NumPy integration
- `./rust/exodus-py/exodus.pyi` - Type stubs

**Estimated time:** 3 hours

---

### Phase 13: Testing and Documentation
Comprehensive testing and documentation.

**Tasks:**
- [ ] Unit tests for all Python bindings
  - [ ] Test all type conversions
  - [ ] Test error handling
  - [ ] Test edge cases

- [ ] Integration tests
  - [ ] Test complete workflows (create -> write -> read)
  - [ ] Test compatibility with existing exodus files

- [ ] Documentation
  - [ ] API reference documentation
  - [ ] User guide with examples
  - [ ] Migration guide from exodus.py
  - [ ] Update README.md

- [ ] Examples
  - [ ] Simple mesh creation
  - [ ] Reading and modifying existing files
  - [ ] Multi-block meshes
  - [ ] Using assemblies and blobs

**Files to create/modify:**
- `./rust/exodus-py/tests/*.py` - Comprehensive test suite
- `./rust/exodus-py/docs/` - Documentation directory
- `./rust/exodus-py/examples/*.py` - Example scripts

**Estimated time:** 6 hours

---

### Phase 14: Build and Distribution
Prepare for distribution and installation.

**Tasks:**
- [ ] Configure maturin for wheel building
  - [ ] Set up CI/CD for multiple platforms (Linux, macOS, Windows)
  - [ ] Configure Python version matrix (3.8, 3.9, 3.10, 3.11, 3.12)

- [ ] Create installation documentation
  - [ ] pip install instructions
  - [ ] Building from source
  - [ ] Troubleshooting guide

- [ ] Performance benchmarking
  - [ ] Compare with C-based exodus.py
  - [ ] Optimize bottlenecks

- [ ] Package metadata
  - [ ] Set up pyproject.toml properly
  - [ ] Add classifiers and keywords
  - [ ] License information

**Files to create/modify:**
- `.github/workflows/python-wheels.yml` - CI/CD workflow
- `./rust/exodus-py/INSTALL.md` - Installation guide
- `./rust/exodus-py/pyproject.toml` - Update with metadata

**Estimated time:** 4 hours

---

## Total Estimated Time

- **Core implementation (Phases 1-10):** ~35 hours
- **Optional features (Phase 11):** ~6 hours
- **Testing & docs (Phase 12-13):** ~9 hours
- **Distribution (Phase 14):** ~4 hours
- **Total (minimum viable product):** ~48 hours
- **Total (with variables):** ~54 hours

## Success Criteria

1. ✅ Python users can create Exodus files using the builder API
2. ✅ Python users can read existing Exodus files
3. ✅ All core data types are accessible from Python
4. ✅ Comprehensive test coverage (>80%)
5. ✅ Complete API documentation
6. ✅ Installation via pip works on Linux, macOS, and Windows
7. ✅ Performance is comparable or better than C-based bindings

## Implementation Progress

### Completed Features
- [x] Planning and research
- [ ] (remaining tasks TBD)

### Current Status
Currently in **Phase 1: Project Setup**

### Next Steps
1. Set up PyO3 project structure
2. Configure maturin build
3. Implement core type bindings

---

## Notes and Considerations

### Design Decisions

1. **Fluent vs. Traditional API**: Prioritize the builder API (fluent) as it's more Pythonic and ergonomic
2. **Error Handling**: Convert Rust `Result<T, ExodusError>` to Python exceptions with helpful messages
3. **Memory Management**: PyO3 handles memory management automatically; be careful with large data arrays
4. **NumPy Integration**: Optional but recommended for performance with large datasets
5. **File Mode Safety**: Use separate classes for Read/Write/Append modes rather than a single class with mode parameter

### Open Questions

1. Should we support both the builder API and lower-level file operations, or focus on one?
   - **Decision**: Support both, with builder API as primary/recommended interface

2. How should we handle very large datasets (memory mapping, chunking)?
   - **Decision**: Start with full in-memory operations, add streaming in future if needed

3. Should we maintain 1:1 API compatibility with exodus.py or create a new Pythonic API?
   - **Decision**: Create new Pythonic API, consider compatibility layer later

### Dependencies

**Rust dependencies:**
- `pyo3 = { version = "0.20", features = ["extension-module"] }`
- `numpy = "0.20"` (for NumPy integration)
- `exodus-rs` (local dependency)

**Python dependencies:**
- `numpy >= 1.20` (optional but recommended)
- `pytest` (for testing)
- `pytest-cov` (for coverage)

### File Structure

```
./rust/exodus-py/
├── Cargo.toml                 # Rust package config
├── pyproject.toml             # Python package config
├── README.md                  # Package documentation
├── INSTALL.md                 # Installation guide
├── src/
│   ├── lib.rs                # Main entry point, module definitions
│   ├── types.rs              # Core type bindings
│   ├── file.rs               # File operation bindings
│   ├── error.rs              # Error handling
│   ├── coord.rs              # Coordinate operations
│   ├── block.rs              # Block operations
│   ├── set.rs                # Set operations
│   ├── builder.rs            # Builder API bindings
│   ├── metadata.rs           # Metadata operations
│   ├── map.rs                # Map operations
│   ├── assembly.rs           # Assembly operations
│   ├── blob.rs               # Blob operations
│   ├── variable.rs           # Variable operations (optional)
│   └── numpy_support.rs      # NumPy integration
├── tests/
│   ├── test_types.py
│   ├── test_file_ops.py
│   ├── test_coords.py
│   ├── test_blocks.py
│   ├── test_sets.py
│   ├── test_builder.py
│   ├── test_metadata.py
│   ├── test_maps.py
│   └── test_assembly_blob.py
├── examples/
│   ├── simple_mesh.py
│   ├── read_mesh.py
│   ├── multi_block.py
│   └── builder_api.py
└── docs/
    ├── api_reference.md
    ├── user_guide.md
    └── migration_guide.md
```

---

## References

- [PyO3 Documentation](https://pyo3.rs/)
- [maturin Documentation](https://www.maturin.rs/)
- [Exodus II Format Specification](https://sandialabs.github.io/seacas-docs/)
- [Current Python exodus.py API](https://sandialabs.github.io/seacas-docs/sphinx/html/exodus.html)
- [exodus-rs Rust Implementation](../exodus-rs/)
