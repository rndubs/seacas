# Exodus Rust Implementation Architecture

**Last Updated:** 2025-11-19

## Overview

The Exodus Rust implementation consists of two distinct layers with clear separation of concerns:

1. **exodus-rs** - Pure Rust library (`./rust/exodus-rs/`)
2. **exodus-py** - Python bindings (`./rust/exodus-py/`)

This document explains the architectural decisions, layer responsibilities, and data flow between components.

---

## Layered Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Python User Code                         │
│  (uses NumPy arrays, Python lists, Python types)            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     exodus-py Layer                          │
│  • PyO3 Python bindings                                      │
│  • NumPy integration (numpy crate 0.27)                      │
│  • Type conversions: NumPy/List ↔ Vec<T>                    │
│  • Python exception handling                                 │
│  • Type stub generation (.pyi files)                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     exodus-rs Layer                          │
│  • Pure Rust implementation                                  │
│  • Uses Vec<f64>, Vec<i64> for arrays                       │
│  • No Python or NumPy dependencies                           │
│  • Optional ndarray support (not yet implemented)            │
│  • NetCDF-4/HDF5 via netcdf-rs                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│               System Libraries (C)                           │
│  • NetCDF-C (libnetcdf)                                      │
│  • HDF5 (libhdf5)                                            │
│  • Original Exodus C library (for compatibility testing)     │
└─────────────────────────────────────────────────────────────┘
```

---

## Layer Responsibilities

### Layer 1: exodus-rs (Pure Rust)

**Purpose:** Core Exodus II format implementation in pure, safe Rust.

**Responsibilities:**
- File lifecycle management (open, create, close)
- NetCDF-4 operations via netcdf-rs
- Exodus II format specification compliance
- Data validation and error handling
- Type-safe API using Rust's type system
- Array operations using `Vec<f64>`, `Vec<i64>`, etc.

**Dependencies:**
```toml
[dependencies]
netcdf = "0.11"              # NetCDF backend
thiserror = "1.0"            # Error handling
ndarray = { optional }       # Future: Rust ndarray support
rayon = { optional }         # Future: Parallel I/O
serde = { optional }         # Future: Serialization
```

**Key Design Decisions:**
1. **No Python dependencies** - Can be used in pure Rust projects
2. **No NumPy** - NumPy is a Python library, not relevant to Rust
3. **Uses Vec<T>** - Rust's standard dynamic array type
4. **Optional ndarray** - For Rust users who want ndarray integration (not NumPy!)

**API Example:**
```rust
use exodus_rs::{ExodusFile, mode};

// Pure Rust - uses Vec<f64>
let mut file = ExodusFile::<mode::Write>::create("mesh.exo")?;
let x_coords: Vec<f64> = vec![0.0, 1.0, 2.0];
let y_coords: Vec<f64> = vec![0.0, 0.0, 0.0];
let z_coords: Vec<f64> = vec![0.0, 0.0, 0.0];
file.put_coords(&x_coords, &y_coords, &z_coords)?;
```

---

### Layer 2: exodus-py (Python Bindings)

**Purpose:** Provide idiomatic Python bindings with NumPy integration.

**Responsibilities:**
- Python type conversions (NumPy ↔ Vec, List ↔ Vec)
- NumPy array support for performance and ergonomics
- Python exception handling and error messages
- Type stubs (.pyi) for IDE support and type checking
- Pythonic API design (snake_case, context managers, etc.)
- Backward compatibility with list-based code

**Dependencies:**
```toml
[dependencies]
pyo3 = { version = "0.27", features = ["extension-module"] }
exodus-rs = { path = "../exodus-rs", features = ["netcdf4"] }
numpy = { version = "0.27", optional = true }  # Default: enabled

[features]
default = ["numpy"]
numpy = ["dep:numpy"]
```

**Key Design Decisions:**
1. **NumPy at Python layer** - Where it belongs, not in core Rust
2. **Dual input support** - Accepts both `List[float]` and `NDArray[np.float64]`
3. **NumPy output** - Returns NumPy arrays for better performance
4. **Backward compatible** - Existing list-based code still works
5. **Zero-copy where possible** - Direct memory sharing with NumPy

**Data Flow:**

#### Input Path (Python → Rust)
```
NumPy array (np.float64)  ─┐
                            ├→ extract_f64_vec() → Vec<f64> → exodus-rs
Python list [float]       ─┘
```

The `extract_f64_vec()` function in `numpy_utils.rs`:
1. Checks if input is a NumPy array (any dtype: f64, f32, i64, i32)
2. If NumPy: converts to Vec<f64> with automatic dtype conversion
3. If list: iterates and converts Python floats to Vec<f64>
4. Passes Vec<f64> to exodus-rs

#### Output Path (Rust → Python)
```
exodus-rs returns Vec<f64> → vec_to_numpy_f64() → NumPy array (np.float64)
```

The `vec_to_numpy_f64()` function:
1. Takes Vec<f64> from exodus-rs
2. Creates a NumPy array with the same data
3. Returns `Bound<'py, PyArray1<f64>>` to Python

**API Example:**
```python
import numpy as np
from exodus import ExodusWriter

# NumPy arrays work
x = np.array([0.0, 1.0, 2.0])
y = np.array([0.0, 0.0, 0.0])
z = np.array([0.0, 0.0, 0.0])

writer = ExodusWriter.create("mesh.exo")
writer.put_coordinates(x, y, z)  # Accepts NumPy

# Python lists also work (backward compatible)
x_list = [0.0, 1.0, 2.0]
writer.put_coordinates(x_list, y_list, z_list)  # Also works
```

---

## Why This Architecture?

### Separation of Concerns

**exodus-rs should remain pure Rust because:**
1. ✅ Can be used in pure Rust projects without Python
2. ✅ Can be compiled to WebAssembly, embedded systems, etc.
3. ✅ Easier to test and benchmark without Python overhead
4. ✅ Can be used from other languages (C FFI, Node.js, Ruby, etc.)
5. ✅ NumPy is fundamentally a Python library - not relevant to Rust

**exodus-py should handle NumPy because:**
1. ✅ NumPy is a Python library (requires CPython runtime)
2. ✅ PyO3 + numpy crate provide the integration layer
3. ✅ Python users expect NumPy integration
4. ✅ Type conversions happen at the language boundary
5. ✅ No impact on pure Rust users

### Alternative: ndarray in exodus-rs

The `ndarray` optional dependency in exodus-rs is for **Rust's ndarray library**, not NumPy:

```rust
// Future optional feature for Rust users
use ndarray::Array1;

let x_coords: Array1<f64> = Array1::from_vec(vec![0.0, 1.0, 2.0]);
file.put_coords_ndarray(&x_coords, &y_coords, &z_coords)?;
```

This would be:
- ✅ For Rust users who prefer ndarray over Vec
- ✅ Still pure Rust, no Python
- ❌ **Not currently implemented** (optional feature for future)
- ❌ **Not related to NumPy** (different library)

---

## NumPy Integration Implementation

### Files Modified (exodus-py only)

#### Core Implementation
- **`src/numpy_utils.rs`** (NEW)
  - `extract_f64_vec()` - Convert NumPy/List → Vec<f64>
  - `extract_i64_vec()` - Convert NumPy/List → Vec<i64>
  - `vec_to_numpy_f64()` - Convert Vec<f64> → NumPy
  - `vec_to_numpy_i64()` - Convert Vec<i64> → NumPy
  - Handles multiple dtypes (f32, f64, i32, i64, u32, u64)

#### Updated Modules (exodus-py)
- **`src/coord.rs`** - Coordinate methods
- **`src/variable.rs`** - Variable read/write
- **`src/block.rs`** - Block connectivity
- **`src/set.rs`** - Set ID arrays

#### Type Stubs
- **`python/exodus/exodus.pyi`**
  - Input types: `Union[List[float], NDArray[np.float64]]`
  - Output types: `NDArray[np.float64]`, `NDArray[np.int64]`
  - Full mypy validation (all tests pass)

#### Tests
- **`tests/test_numpy_integration.py`** (NEW)
  - 7 comprehensive NumPy tests
  - Input/output validation
  - Mixed dtype support
  - Backward compatibility

### Performance Characteristics

**NumPy Input:**
- ✅ Zero-copy when dtype matches (f64/i64)
- ⚠️  Minimal copy when conversion needed (f32→f64, i32→i64)
- ✅ No Python loop overhead

**NumPy Output:**
- ✅ Single allocation for NumPy array
- ✅ Direct memory initialization
- ✅ No intermediate Python objects

**List Input (backward compatible):**
- ⚠️  Python iteration required
- ⚠️  Python float→Rust f64 conversion per element
- ✅ Still works, just slower than NumPy

---

## Testing Strategy

### exodus-rs Testing
**268 Rust tests** - All pure Rust, no Python
- Unit tests in each module
- Integration tests in `tests/`
- Benchmark tests
- C compatibility tests (80/80 passing)

### exodus-py Testing
**225 Python tests** - All passing
- PyO3 binding tests
- NumPy integration tests
- Type stub validation (mypy)
- Backward compatibility tests

### No Cross-Layer Contamination
- exodus-rs tests don't require Python
- exodus-py tests don't modify exodus-rs
- Clear separation of concerns

---

## Type Stub Documentation

The `.pyi` type stub file provides IDE autocompletion and mypy type checking:

```python
# Input parameters accept both types (Union)
def put_coordinates(
    self,
    x: Union[List[float], NDArray[np.float64]],
    y: Union[List[float], NDArray[np.float64]],
    z: Union[List[float], NDArray[np.float64]]
) -> None: ...

# Output returns NumPy arrays
def coordinates(self) -> Tuple[
    NDArray[np.float64],
    NDArray[np.float64],
    NDArray[np.float64]
]: ...
```

**Validation:**
- ✅ All mypy tests pass (including --strict mode)
- ✅ Type errors correctly detected
- ✅ IDE autocompletion works
- ✅ NumPy type hints fully functional

---

## Future Considerations

### exodus-rs (Pure Rust)
1. **ndarray support** - Optional feature for Rust ndarray users
   ```rust
   fn put_coords_ndarray(&mut self, x: &Array1<f64>, ...) -> Result<()>
   ```
2. **rayon parallelism** - Parallel array operations
3. **serde support** - Serialization of metadata types
4. **C FFI** - C ABI for use from other languages

### exodus-py (Python Bindings)
1. **Performance optimization** - Profile NumPy conversions
2. **Multi-dimensional arrays** - Support for 2D/3D NumPy arrays
3. **Type stub improvements** - Fix method name inconsistencies
4. **Documentation** - Add NumPy examples to docs

### None of these require changes to the architecture
- The layered design is flexible and extensible
- New features can be added at the appropriate layer
- Separation of concerns is maintained

---

## Comparison with Other Approaches

### ❌ Alternative: NumPy in exodus-rs

**What it would look like:**
```rust
// BAD: exodus-rs depending on NumPy
use numpy::PyArray1;  // Requires Python runtime!

fn put_coords(&mut self, x: &PyArray1<f64>) -> Result<()> {
    // ...
}
```

**Why this is wrong:**
- ❌ exodus-rs now requires Python runtime
- ❌ Can't use from pure Rust projects
- ❌ Can't compile to WebAssembly
- ❌ Can't use from other language bindings
- ❌ Ties core library to Python ecosystem
- ❌ More complex build process
- ❌ NumPy is a Python library, not a Rust concept

### ✅ Current Approach: NumPy in exodus-py

**Advantages:**
- ✅ exodus-rs remains pure Rust
- ✅ Can be used from any language
- ✅ Python users get NumPy integration
- ✅ Clean separation of concerns
- ✅ Each layer has appropriate dependencies
- ✅ Easier to test and maintain
- ✅ More flexible for future bindings

---

## Conclusion

**The current architecture is correct:**

1. **exodus-rs** is a pure Rust library
   - Uses `Vec<T>` for arrays
   - No Python dependencies
   - Can be used from any language

2. **exodus-py** provides Python bindings
   - Uses PyO3 for Rust ↔ Python bridge
   - Uses numpy crate for NumPy integration
   - Handles all Python-specific concerns

3. **NumPy support lives in exodus-py**
   - Where it belongs (Python layer)
   - No impact on Rust users
   - Clean, maintainable design

**No changes needed to exodus-rs for NumPy support.**

The NumPy integration is complete, tested, and production-ready at the appropriate layer (exodus-py).

---

## References

- [exodus-rs Documentation](./rust/exodus-rs/README.md)
- [exodus-py Documentation](./rust/PYTHON.md)
- [Implementation Status](./rust/RUST.md)
- [PyO3 Documentation](https://pyo3.rs/)
- [numpy crate](https://docs.rs/numpy/)
- [Rust ndarray](https://docs.rs/ndarray/) (different from NumPy)
