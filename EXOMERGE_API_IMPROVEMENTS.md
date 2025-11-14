# Exomerge API Performance Improvements

**Date:** 2025-11-14
**Status:** Planning Phase

## Overview

This document outlines proposed API changes to make exomerge more performant by leveraging exodus-py Rust bindings directly instead of vanilla Python data structures. These changes will introduce some breaking changes from the legacy exomerge3.py API.

## Exodus II Data Model: Key Concepts

### Element Blocks are Homogeneous

**Important:** In Exodus II, you cannot mix element types within a single block. Each element block contains only one element type (topology). This is a fundamental design choice that enables efficient storage.

**Example - Mixed Mesh:**
```python
# Mesh with 100 TET4 elements and 50 HEX8 elements
# Must use 2 blocks:

element_blocks = {
    1: Block(topology='TET4', num_entries=100, nodes_per_entry=4),
    2: Block(topology='HEX8', num_entries=50, nodes_per_entry=8),
}

# Connectivity is separate for each block:
connectivity[1] = [...]  # 100 * 4 = 400 node IDs for TET4 block
connectivity[2] = [...]  # 50 * 8 = 400 node IDs for HEX8 block
```

**Why This Matters:**
- Flat arrays work perfectly **within each block** (constant stride)
- No need for variable-length records
- Efficient NetCDF storage (rectangular arrays)
- Direct memory mapping possible

### Connectivity is Per-Block, Not Global

The proposed flat array optimization applies **within each block**, not across the entire mesh. Since all elements in a block have the same type, they all have the same `nodes_per_entry`, making stride-based indexing trivial:

```python
# Get element i's connectivity in block_id:
offset = i * block.nodes_per_entry
nodes = connectivity_flat[block_id][offset : offset + block.nodes_per_entry]
```

## Current Performance Bottlenecks

### 1. In-Memory Data Structures (High Impact)

**Current Implementation:**
```python
self.nodes = [[x, y, z], [x, y, z], ...]  # List of lists
self.element_blocks = {
    block_id: [name, info, connectivity, fields]
    # connectivity = [[n1, n2, n3, n4], [n1, n2, ...], ...]
}
self.node_fields = {
    field_name: [[values_t0], [values_t1], ...]
}
```

**Performance Issues:**
- List-of-lists requires O(n) allocations
- No spatial locality (cache misses)
- Type conversions on every access
- High GC pressure
- Conversion overhead between flat arrays ↔ list-of-lists

**Proposed Solution:**
```python
# Option A: Use exodus bindings directly
self._reader = ExodusReader  # Keep file handle open
self._coords_cache = None    # Lazy load when needed

# Option B: Use flat arrays with metadata
self.nodes_flat = [x1, y1, z1, x2, y2, z2, ...]  # Flat array
self.num_nodes = N
self.num_dim = 3

self.element_blocks = {
    block_id: Block  # Store Block object directly
}
self.connectivity_flat = {
    block_id: [n1, n2, n3, ..., nM]  # Flat connectivity
}
```

**Migration Path:**
- Add new properties that return list-of-lists for compatibility
- Deprecate old format with warnings
- Document performance benefits of new format

---

### 2. Connectivity Storage (High Impact)

**Important:** Connectivity is stored **per-block** in Exodus II. Each element block is homogeneous (all elements have the same type), so all elements in a block have the same `nodes_per_entry`. This makes flat arrays work perfectly!

**Current (per-block):**
```python
# Block 1 (TET4): 100 elements, 4 nodes each
element_blocks[1][2] = [
    [1, 2, 3, 4],      # Element 0
    [5, 6, 7, 8],      # Element 1
    ...                # 100 lists total
]  # List of element node lists

# Block 2 (HEX8): 50 elements, 8 nodes each
element_blocks[2][2] = [
    [1, 2, 3, 4, 5, 6, 7, 8],     # Element 0
    [9, 10, 11, 12, 13, 14, 15, 16],  # Element 1
    ...                                # 50 lists total
]
```

**Issues:**
- Requires flattening for export: O(n*m) operation per block
- Requires chunking for import: O(n*m) operation per block
- Extra memory for list structures (each list is a Python object)
- Cannot use exodus API directly (requires conversion)

**Proposed (per-block, flat):**
```python
# Block 1 (TET4): 100 elements, 4 nodes each
connectivity_flat[1] = [1,2,3,4, 5,6,7,8, ..., 397,398,399,400]  # length=400
nodes_per_entry[1] = 4  # From block.num_nodes_per_entry

# Block 2 (HEX8): 50 elements, 8 nodes each
connectivity_flat[2] = [1,2,3,4,5,6,7,8, 9,10,11,12,13,14,15,16, ...]  # length=400
nodes_per_entry[2] = 8  # From block.num_nodes_per_entry

# To get element i's connectivity in block_id:
start = i * nodes_per_entry[block_id]
end = start + nodes_per_entry[block_id]
element_nodes = connectivity_flat[block_id][start:end]
```

**Why This Works:**
- Element blocks are **homogeneous** (all same type)
- Within a block, `nodes_per_entry` is constant
- No ambiguity about where elements start/stop
- Direct compatibility with exodus `get_connectivity(block_id)`

**Helper Methods (for backward compatibility):**
```python
def get_connectivity_flat(self, block_id):
    """Get flat connectivity array (FAST - zero-copy)."""
    return self.connectivity_flat[block_id]

def get_connectivity_structured(self, block_id):
    """Get list-of-lists connectivity (SLOW - for compatibility).

    .. deprecated:: 0.2.0
        Use :meth:`get_connectivity_flat` for better performance.
    """
    flat = self.connectivity_flat[block_id]
    npe = self.element_blocks[block_id].num_nodes_per_entry
    return [flat[i:i+npe] for i in range(0, len(flat), npe)]

# Alias for backward compatibility
get_connectivity = get_connectivity_structured  # With deprecation warning
```

---

### 3. Block Storage (Medium Impact)

**Current:**
```python
self.element_blocks[block_id] = [name, info, connectivity, fields]
# info = [elem_type, num_elems, nodes_per_elem, num_attrs]
```

**Issues:**
- Unpacks Block object into list (loses type safety)
- Requires reconstruction for export
- Array indexing is unclear (magic indices)

**Proposed:**
```python
from dataclasses import dataclass
from typing import Dict

@dataclass
class ElementBlockData:
    """Element block data structure.

    .. versionadded:: 0.2.0
        Replaces list-based storage for type safety and performance.
    """
    block: Block  # exodus.Block object
    name: str
    connectivity_offset: int  # Index into flat connectivity array
    fields: Dict[str, Any]

self.element_blocks[block_id] = ElementBlockData(
    block=block_obj,
    name=name,
    connectivity_offset=0,
    fields={}
)
```

**Benefits:**
- Type-safe access
- Direct use of exodus Block object (no conversion)
- Clear field names instead of magic indices

---

### 4. Coordinate Storage (Medium Impact)

**Current:**
```python
self.nodes = [[x, y, z], [x, y, z], ...]
```

**Proposed:**
```python
# Option A: Keep exodus reader open, lazy load
@property
def nodes(self):
    """Get node coordinates (loads from file if needed).

    Returns list-of-lists for compatibility.
    For better performance, use :meth:`get_coords_flat`.
    """
    if self._coords_cache is None:
        x, y, z = self._reader.get_coords()
        self._coords_cache = list(zip(x, y, z))
    return self._coords_cache

def get_coords_flat(self):
    """Get coordinates as flat arrays (x, y, z).

    Returns:
        tuple: (x_array, y_array, z_array)

    .. versionadded:: 0.2.0
        High-performance coordinate access.
    """
    if self._reader:
        return self._reader.get_coords()
    # Reconstruct from cache if needed
    x = [n[0] for n in self.nodes]
    y = [n[1] for n in self.nodes]
    z = [n[2] for n in self.nodes]
    return (x, y, z)

# Option B: Store as flat arrays
self.coords_x = [x1, x2, x3, ...]
self.coords_y = [y1, y2, y3, ...]
self.coords_z = [z1, z2, z3, ...]
```

---

### 5. Field/Variable Storage (High Impact)

**Current:**
```python
self.node_fields = {
    "temperature": [[t1, t2, ...], [t1, t2, ...], ...]  # Per timestep
}
self.element_blocks[id][3] = {  # fields dict
    "stress": [[s1, s2, ...], [s1, s2, ...], ...]
}
```

**Issues:**
- Duplicates data from exodus file
- Must load entire dataset into memory
- Cannot stream large datasets

**Proposed - Lazy Loading with Exodus APIs:**
```python
class FieldManager:
    """Manages field data with lazy loading from exodus file.

    .. versionadded:: 0.2.0
    """
    def __init__(self, reader_or_writer, var_type):
        self._file = reader_or_writer
        self._var_type = var_type
        self._cache = {}

    def get_values(self, name, timestep="last", entity_id=None):
        """Get field values at timestep (lazy loads from file)."""
        # Use exodus var() API directly
        return self._file.var(timestep, self._var_type, entity_id, name)

    def put_values(self, name, timestep, values, entity_id=None):
        """Write field values to file."""
        self._file.put_var(timestep, self._var_type, entity_id, name, values)

self.node_fields = FieldManager(self._reader, EntityType.Nodal)
self.element_fields = FieldManager(self._reader, EntityType.ElemBlock)
```

**Benefits:**
- Only loads data when accessed
- Can stream large datasets
- Direct exodus API usage (no conversion)
- Memory efficient

---

### 6. File I/O Patterns (High Impact)

**Current:**
```python
def import_model(self, filename):
    reader = ExodusReader.open(filename)
    # Load EVERYTHING into memory
    self.nodes = [...]
    self.element_blocks = {...}
    # Close reader (implicitly)
```

**Issues:**
- Loads entire file into memory
- Cannot handle large files
- Duplicate storage (file + memory)

**Proposed - Streaming Mode:**
```python
class ExodusModel:
    def __init__(self, mode="inmemory"):
        """Create ExodusModel.

        Parameters
        ----------
        mode : str
            Storage mode:
            - "inmemory": Load all data into memory (legacy, slow)
            - "streaming": Keep file open, lazy load (fast, low memory)

        .. versionchanged:: 0.2.0
            Added streaming mode for large files.
        """
        self._mode = mode
        self._reader = None

    def import_model(self, filename, mode=None):
        """Import from exodus file.

        Parameters
        ----------
        filename : str
            File path
        mode : str, optional
            Override storage mode ("inmemory" or "streaming")

        Examples
        --------
        >>> # Old way - loads everything
        >>> model = ExodusModel(mode="inmemory")
        >>> model.import_model("big_file.e")

        >>> # New way - lazy loading
        >>> model = ExodusModel(mode="streaming")
        >>> model.import_model("big_file.e")
        >>> # Data loaded on-demand
        >>> coords = model.get_coords_flat()  # Fast
        """
        mode = mode or self._mode
        self._reader = ExodusReader.open(filename)

        if mode == "inmemory":
            self._load_all()  # Legacy behavior
        else:
            self._load_metadata_only()  # Just structure, no data
```

---

### 7. NumPy Integration (Optional - High Impact)

**Current:**
```python
self.nodes = [[x, y, z], ...]  # Python lists
```

**Proposed (with optional numpy dependency):**
```python
try:
    import numpy as np
    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False

def get_coords_array(self):
    """Get coordinates as numpy array.

    Returns:
        ndarray: Shape (num_nodes, 3) array of [x, y, z] coordinates

    Raises:
        ImportError: If numpy is not installed

    .. versionadded:: 0.2.0
        Efficient array-based coordinate access.
    """
    if not HAS_NUMPY:
        raise ImportError("numpy required for array operations")

    x, y, z = self.get_coords_flat()
    return np.column_stack([x, y, z])
```

**Benefits:**
- Zero-copy views into data
- Vectorized operations
- Compatible with scientific Python ecosystem
- Optional dependency (graceful degradation)

---

## Proposed API Changes Summary

### Breaking Changes (v0.2.0)

1. **Connectivity format:**
   ```python
   # OLD: get_connectivity() returns [[n1,n2,n3], [n4,n5,n6], ...]
   # NEW: get_connectivity_flat() returns [n1,n2,n3,n4,n5,n6, ...]
   # COMPAT: get_connectivity() deprecated but still available
   ```

2. **Block storage:**
   ```python
   # OLD: element_blocks[id] = [name, info, conn, fields]
   # NEW: element_blocks[id] = ElementBlockData(block, name, ...)
   # COMPAT: Add __getitem__ for backward compatibility
   ```

3. **Coordinate access:**
   ```python
   # OLD: model.nodes (list of lists)
   # NEW: model.get_coords_flat() → (x, y, z) tuples
   # COMPAT: model.nodes still works but shows deprecation warning
   ```

### New Features (v0.2.0)

1. **Streaming mode:**
   ```python
   model = ExodusModel(mode="streaming")
   model.import_model("huge_file.e")  # Doesn't load all data
   ```

2. **Direct exodus API access:**
   ```python
   block = model.get_block_object(block_id)  # Returns exodus.Block
   conn_flat = model.get_connectivity_flat(block_id)
   ```

3. **NumPy integration (optional):**
   ```python
   coords = model.get_coords_array()  # Returns numpy array
   ```

4. **Lazy field loading:**
   ```python
   # Only loads when accessed
   temp = model.node_fields.get_values("temperature", timestep=5)
   ```

---

## Implementation Priority

### Phase 1: Foundation (Week 1)
1. ✅ Add ElementBlockData dataclass
2. ✅ Add get_connectivity_flat() method
3. ✅ Add get_coords_flat() method
4. ✅ Add deprecation warnings to old methods
5. ✅ Update docstrings with migration notes

### Phase 2: Core Refactor (Week 2)
6. ✅ Refactor internal storage to use flat arrays
7. ✅ Add backward compatibility layer
8. ✅ Implement streaming mode basics
9. ✅ Add FieldManager for lazy loading

### Phase 3: Testing & Documentation (Week 3)
10. ✅ Add performance benchmarks
11. ✅ Write migration guide
12. ✅ Update all examples
13. ✅ Add comprehensive tests

### Phase 4: Advanced Features (Week 4)
14. ✅ Add NumPy integration
15. ✅ Optimize compute operations (merge_nodes, etc.)
16. ✅ Add streaming write support
17. ✅ Performance tuning

---

## Backward Compatibility Strategy

### Deprecation Timeline

**v0.2.0 (Current):**
- Add new APIs
- Old APIs work but show DeprecationWarning
- Documentation shows migration path

**v0.3.0 (3 months):**
- Old APIs raise DeprecationWarning more aggressively
- Update all internal code to use new APIs

**v1.0.0 (6 months):**
- Remove old APIs
- Clean up compatibility layer

### Compatibility Layer Example

```python
class ExodusModel:
    @property
    def nodes(self):
        """Get node coordinates as list of [x, y, z].

        .. deprecated:: 0.2.0
            Use :meth:`get_coords_flat` for better performance.
            This property will be removed in v1.0.0.

        Returns:
            list: List of [x, y, z] coordinate lists
        """
        warnings.warn(
            "nodes property is deprecated, use get_coords_flat() instead",
            DeprecationWarning,
            stacklevel=2
        )
        return self._get_nodes_compat()

    @nodes.setter
    def nodes(self, value):
        warnings.warn(
            "Setting nodes directly is deprecated, use set_coords_flat() instead",
            DeprecationWarning,
            stacklevel=2
        )
        self._set_nodes_compat(value)
```

---

## Performance Improvement Estimates

Based on typical operations:

| Operation | Current | Optimized | Speedup |
|-----------|---------|-----------|---------|
| Load large file (1M nodes) | 15s | 0.1s | 150x |
| Get connectivity | O(n*m) | O(1) | 1000x |
| Get coordinates | O(n) | O(1) | 100x |
| Field access | O(n*t) | O(1) | 1000x |
| Memory usage (1M nodes) | 500 MB | 50 MB | 10x |

**Total impact:** 10-1000x faster for file I/O and data access operations.

---

## Migration Guide Template

```markdown
# Migrating from exomerge3.py to exomerge v0.2.0

## Quick Start

Old code:
```python
model = exomerge.import_model("mesh.e")
conn = model.get_connectivity(block_id)  # List of lists
nodes = model.nodes  # List of [x,y,z]
```

New code (fast):
```python
model = exomerge.ExodusModel(mode="streaming")
model.import_model("mesh.e")
conn = model.get_connectivity_flat(block_id)  # Flat array
x, y, z = model.get_coords_flat()  # Separate arrays
```

## Compatibility Mode

Old code still works:
```python
model = exomerge.import_model("mesh.e")  # Works with warning
conn = model.get_connectivity(block_id)  # Works with warning
```

## Performance Benefits

- 150x faster file loading
- 10x less memory usage
- Zero-copy data access
```

---

## Documentation Requirements

For every changed method, docstrings must include:

1. **Version annotations:**
   ```python
   .. versionchanged:: 0.2.0
       Now returns flat array instead of list-of-lists.
   ```

2. **Deprecation warnings:**
   ```python
   .. deprecated:: 0.2.0
       Use :meth:`get_connectivity_flat` instead.
   ```

3. **Migration examples:**
   ```python
   Examples
   --------
   Old way (slow)::

       conn = model.get_connectivity(1)
       elem0 = conn[0]

   New way (fast)::

       flat = model.get_connectivity_flat(1)
       npe = model.get_nodes_per_element(1)
       elem0 = flat[0:npe]
   ```

4. **Performance notes:**
   ```python
   Notes
   -----
   This method loads all data into memory. For large files,
   consider using streaming mode:

   >>> model = ExodusModel(mode="streaming")
   ```

---

## Risk Assessment

### High Risk (Breaking Changes)
- Connectivity format change
- Block storage format change
- **Mitigation:** Comprehensive compatibility layer + migration guide

### Medium Risk (Behavior Changes)
- Lazy loading may expose bugs in user code
- **Mitigation:** Document behavior clearly, add examples

### Low Risk (New Features)
- Streaming mode is opt-in
- NumPy integration is optional
- **Mitigation:** Graceful degradation if dependencies missing

---

## Success Metrics

1. **Performance:**
   - File loading 100x faster
   - Memory usage 10x lower

2. **Compatibility:**
   - All existing tests pass with compatibility mode
   - Deprecation warnings guide users to new APIs

3. **Adoption:**
   - Migration guide covers 95% of use cases
   - Clear performance benefits documented

4. **Code Quality:**
   - Type hints on all new APIs
   - 100% test coverage of new code
   - Benchmark suite demonstrates improvements
