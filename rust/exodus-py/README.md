# exodus-py

Python bindings for exodus-rs - A pure Rust implementation of the Exodus II finite element data format.

## Overview

`exodus-py` provides ergonomic Python bindings for the exodus-rs Rust library, enabling efficient creation and manipulation of Exodus II mesh files directly from Python.

## Features

- **High-level Builder API**: Fluent interface for creating meshes
- **Type Safety**: Leverages Rust's type system for correctness
- **Performance**: Fast operations powered by Rust
- **Pure Rust Backend**: No C library dependencies
- **NetCDF-4/HDF5 Support**: Modern file format support

## Installation

### From PyPI (when published)

```bash
pip install exodus-py
```

### From Source

```bash
# Install maturin
pip install maturin

# Build and install
cd rust/exodus-py
maturin develop
```

## Quick Start

### Creating a Mesh with Builder API

```python
from exodus import MeshBuilder, BlockBuilder

# Create a simple 2D quad mesh
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

### Reading a Mesh

```python
from exodus import ExodusReader

# Open and read a mesh
with ExodusReader.open("mesh.exo") as exo:
    params = exo.init_params()
    print(f"Mesh has {params.num_nodes} nodes")
    print(f"Mesh has {params.num_elems} elements")

    # Get coordinates
    x, y, z = exo.get_coords()
```

### Creating a 3D Hex Mesh

```python
from exodus import MeshBuilder, BlockBuilder

# Create a single hex element
(MeshBuilder("Hex Mesh")
    .dimensions(3)
    .coordinates(
        x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
        y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
        z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
    )
    .add_block(
        BlockBuilder(100, "HEX8")
            .connectivity([1, 2, 3, 4, 5, 6, 7, 8])
            .build()
    )
    .qa_record("MyApp", "1.0.0", "2025-01-15", "14:30:00")
    .info("Generated with exodus-py")
    .write("hex_mesh.exo"))
```

## API Overview

### Core Classes

- **MeshBuilder**: High-level mesh creation with fluent API
- **BlockBuilder**: Element block builder
- **ExodusReader**: Read-only file access
- **ExodusWriter**: Write-only file creation
- **ExodusAppender**: Read-write file access

### Data Types

- **EntityType**: Enumeration of entity types (blocks, sets, etc.)
- **InitParams**: Database initialization parameters
- **Block**: Element/edge/face block definition
- **NodeSet**: Node set with optional distribution factors
- **SideSet**: Side set definition
- **Assembly**: Hierarchical grouping of entities
- **Blob**: Arbitrary binary data storage

### File Operations

- `ExodusWriter.create()`: Create a new file
- `ExodusReader.open()`: Open existing file for reading
- `ExodusAppender.append()`: Open for read-write access

## Documentation

For detailed documentation, see:
- [API Reference](docs/api_reference.md)
- [User Guide](docs/user_guide.md)
- [Examples](examples/)

## Requirements

- Python 3.8+
- NetCDF-4 libraries (system dependency)
- NumPy (optional, for array operations)

## License

BSD 3-Clause License. See LICENSE for details.

## Contributing

Contributions are welcome! Please see the main SEACAS repository for contribution guidelines.

## Acknowledgments

This package is part of the SEACAS project (Sandia Engineering Analysis Code Access System).
