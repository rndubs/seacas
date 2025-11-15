"""
Pytest configuration for testing documentation code examples using Sybil.

This conftest.py file configures Sybil to test all Python code blocks
in the MyST markdown documentation files.
"""

import pytest
import tempfile
import os
from pathlib import Path
from sybil import Sybil
from sybil.parsers.myst import (
    DocTestDirectiveParser,
    PythonCodeBlockParser,
)


@pytest.fixture(scope="session", autouse=True)
def setup_test_environment():
    """Setup test environment with sample .exo files and cleanup after tests."""
    # Create sample .exo files in the docs directory
    docs_dir = Path(__file__).parent

    # Create sample .exo files for testing
    sample_files = []
    try:
        from exodus import MeshBuilder, BlockBuilder, ExodusWriter, InitParams, EntityType, CreateMode, Block

        # Create a simple mesh.exo file
        mesh_path = docs_dir / "mesh.exo"
        builder = MeshBuilder("Test Mesh")
        builder.dimensions(3)
        builder.coordinates(
            x=[0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
            y=[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
            z=[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
        )
        builder.add_block(
            BlockBuilder(100, "HEX8")
                .connectivity([1, 2, 3, 4, 5, 6, 7, 8])
                .build()
        )
        builder.write(str(mesh_path))
        sample_files.append(mesh_path)

        # Create original_mesh.exo (same as mesh.exo)
        original_mesh_path = docs_dir / "original_mesh.exo"
        builder.write(str(original_mesh_path))
        sample_files.append(original_mesh_path)

        # Create results.exo with time series data
        results_path = docs_dir / "results.exo"
        writer = ExodusWriter.create(str(results_path))
        params = InitParams(
            title="Results Mesh",
            num_dim=3,
            num_nodes=8,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)
        writer.put_coords(
            [0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
            [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
            [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
        )
        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="HEX8",
            num_entries=1,
            num_nodes_per_entry=8,
            num_attributes=0,
        )
        writer.put_block(block)
        writer.put_connectivity(1, [1, 2, 3, 4, 5, 6, 7, 8])

        # Add time series data
        writer.define_variables(EntityType.Nodal, ["Temperature", "Pressure"])
        writer.define_variables(EntityType.Global, ["TotalEnergy"])

        for step in range(3):
            writer.put_time(step, step * 0.1)
            writer.put_var(step, EntityType.Global, 0, 0, [1000.0 - step * 10.0])
            temps = [300.0 + i * 10.0 + step * 5.0 for i in range(8)]
            pressures = [100.0 + i * 1.0 + step * 0.5 for i in range(8)]
            writer.put_var(step, EntityType.Nodal, 0, 0, temps)
            writer.put_var(step, EntityType.Nodal, 0, 1, pressures)

        writer.close()
        sample_files.append(results_path)

    except Exception as e:
        import traceback
        print(f"Warning: Could not create sample .exo files: {e}")
        traceback.print_exc()

    yield

    # Cleanup - remove sample files
    for file_path in sample_files:
        try:
            if file_path.exists():
                file_path.unlink()
        except:
            pass


def setup_namespace(namespace):
    """Pre-populate namespace with common imports for all code examples."""
    # Import all common exodus modules
    try:
        from exodus import (
            ExodusReader, ExodusWriter, ExodusAppender,
            MeshBuilder, BlockBuilder,
            InitParams, CreateOptions, CreateMode, FloatSize, Int64Mode,
            Block, EntityType,
            Assembly, Blob, QaRecord,
            EntitySet,
        )

        namespace.update({
            'ExodusReader': ExodusReader,
            'ExodusWriter': ExodusWriter,
            'ExodusAppender': ExodusAppender,
            'MeshBuilder': MeshBuilder,
            'BlockBuilder': BlockBuilder,
            'InitParams': InitParams,
            'CreateOptions': CreateOptions,
            'CreateMode': CreateMode,
            'FloatSize': FloatSize,
            'Int64Mode': Int64Mode,
            'Block': Block,
            'EntityType': EntityType,
            'Assembly': Assembly,
            'Blob': Blob,
            'QaRecord': QaRecord,
            'EntitySet': EntitySet,
        })
    except ImportError as e:
        print(f"Warning: Could not import exodus modules: {e}")


# Configure Sybil to test Python code blocks in MyST markdown files
pytest_collect_file = Sybil(
    parsers=[
        PythonCodeBlockParser(future_imports=['annotations']),
        DocTestDirectiveParser(),
    ],
    patterns=['*.md'],
    excludes=[],
    setup=setup_namespace,
).pytest()
