# exodus-py Documentation

Welcome to the documentation for **exodus-py**, a Python package providing fast, type-safe bindings to the Exodus II finite element data format through a pure Rust implementation.

## About exodus-py

exodus-py is a Python interface to the exodus-rs Rust library, enabling efficient creation, reading, and manipulation of Exodus II mesh files directly from Python. Built with [PyO3](https://pyo3.rs/), it combines Python's ease-of-use with Rust's performance and safety guarantees.

### Key Features

- **High-Performance**: Rust-powered operations for fast file I/O
- **Type Safety**: Leverages Rust's type system to prevent common errors
- **Pure Rust Backend**: No dependency on legacy C libraries
- **Modern Format Support**: Full NetCDF-4/HDF5 compatibility
- **Pythonic API**: Fluent builder interface and context managers
- **Complete Coverage**: Support for all Exodus II entities (blocks, sets, assemblies, variables, etc.)

## Documentation Structure

- **{doc}`quickstart`** - Get up and running quickly with common use cases
- **{doc}`user_guide`** - Comprehensive guide to using exodus-py
- **{doc}`api_reference`** - Detailed API documentation for all classes and methods

## Example Usage

### Creating a Simple Mesh

```python
from exodus import MeshBuilder, BlockBuilder

(MeshBuilder("My Mesh")
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
    .write("mesh.exo"))
```

### Reading an Existing Mesh

```python
from exodus import ExodusReader

with ExodusReader.open("mesh.exo") as reader:
    params = reader.init_params()
    print(f"Mesh has {params.num_nodes} nodes")
    x, y, z = reader.get_coords()
```

## Table of Contents

```{toctree}
:maxdepth: 2
:caption: Contents

quickstart
user_guide
api_reference
```

## License

exodus-py is distributed under the BSD 3-Clause License.

## Support

For issues, questions, or contributions, please visit the [SEACAS GitHub repository](https://github.com/sandialabs/seacas).
