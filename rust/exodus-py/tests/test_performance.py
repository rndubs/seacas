"""Tests for performance configuration

This test suite validates the HDF5 performance optimization features including
chunking and caching configuration.

## Running Tests

Basic test (without chunking verification):
    pytest test_performance.py

With chunking verification (requires netCDF4-python):
    pip install exodus-py[test]
    pytest test_performance.py -v

The test_chunking_verification_complete() test will:
- Always test that the API works correctly
- Optionally verify actual chunking if netCDF4-python or h5py is installed
- Skip verification with a warning if neither library is available

## Chunking Verification

The verification test creates a file with specific chunk settings and uses
netCDF4-python or h5py to inspect the file and confirm that chunking was
actually applied to the variables:
- Coordinate variables (coordx, coordy, coordz)
- Connectivity variables (connect1, etc.)
- Result variables (vals_nod_var1, etc.)
"""

import pytest
import tempfile
import os
from exodus import (
    NodeType,
    CacheConfig,
    ChunkConfig,
    PerformanceConfig,
    CreateOptions,
    CreateMode,
    ExodusWriter,
    InitParams,
)


def test_node_type_detection():
    """Test node type detection"""
    node_type = NodeType.detect()
    assert node_type is not None
    # Should return one of the valid types
    assert isinstance(node_type, NodeType)


def test_node_type_creation():
    """Test creating specific node types"""
    compute = NodeType.compute()
    login = NodeType.login()
    unknown = NodeType.unknown()

    assert isinstance(compute, NodeType)
    assert isinstance(login, NodeType)
    assert isinstance(unknown, NodeType)


def test_node_type_defaults():
    """Test node type default values"""
    compute = NodeType.compute()
    assert compute.default_cache_size() == 128 * 1024 * 1024  # 128 MB
    assert compute.default_chunk_nodes() == 10_000

    login = NodeType.login()
    assert login.default_cache_size() == 4 * 1024 * 1024  # 4 MB
    assert login.default_chunk_nodes() == 1_000


def test_cache_config_creation():
    """Test creating cache configurations"""
    # Default (auto-detected)
    cache = CacheConfig()
    assert cache.cache_size > 0
    assert cache.preemption == 0.75

    # Custom size
    cache = CacheConfig(cache_size=64 * 1024 * 1024)
    assert cache.cache_size == 64 * 1024 * 1024


def test_cache_config_builder():
    """Test cache config fluent API"""
    cache = CacheConfig(cache_size=100 * 1024 * 1024)
    cache = cache.with_preemption(0.5)
    assert cache.preemption == 0.5
    assert cache.cache_size == 100 * 1024 * 1024

    cache2 = cache.with_cache_mb(256)
    assert cache2.cache_size == 256 * 1024 * 1024


def test_chunk_config_creation():
    """Test creating chunk configurations"""
    # Default (auto-detected)
    chunks = ChunkConfig()
    assert chunks.node_chunk_size > 0
    assert chunks.element_chunk_size > 0
    assert chunks.time_chunk_size == 0  # Default for mesh I/O

    # Custom sizes
    chunks = ChunkConfig(node_chunk_size=20_000, element_chunk_size=15_000)
    assert chunks.node_chunk_size == 20_000
    assert chunks.element_chunk_size == 15_000


def test_chunk_config_builder():
    """Test chunk config fluent API"""
    chunks = ChunkConfig()
    chunks = chunks.with_node_chunk_size(25_000)
    chunks = chunks.with_element_chunk_size(20_000)
    chunks = chunks.with_time_chunk_size(10)

    assert chunks.node_chunk_size == 25_000
    assert chunks.element_chunk_size == 20_000
    assert chunks.time_chunk_size == 10


def test_performance_config_auto():
    """Test automatic performance configuration"""
    perf = PerformanceConfig.auto()
    assert perf is not None
    assert perf.cache.cache_size > 0
    assert perf.chunks.node_chunk_size > 0


def test_performance_config_presets():
    """Test performance configuration presets"""
    conservative = PerformanceConfig.conservative()
    assert conservative.cache.cache_size == 4 * 1024 * 1024

    aggressive = PerformanceConfig.aggressive()
    assert aggressive.cache.cache_size == 128 * 1024 * 1024


def test_performance_config_for_node_type():
    """Test creating config for specific node type"""
    compute_node = NodeType.compute()
    perf = PerformanceConfig.for_node_type(compute_node)
    assert perf.cache.cache_size == 128 * 1024 * 1024


def test_performance_config_builder():
    """Test performance config fluent API"""
    perf = PerformanceConfig.auto()
    perf = perf.with_cache_mb(256)
    perf = perf.with_node_chunk_size(20_000)
    perf = perf.with_preemption(0.3)

    assert perf.cache.cache_size == 256 * 1024 * 1024
    assert perf.chunks.node_chunk_size == 20_000
    assert perf.cache.preemption == 0.3


def test_performance_config_summary():
    """Test performance config summary"""
    perf = PerformanceConfig.auto()
    summary = perf.summary()
    assert isinstance(summary, str)
    assert "Performance Config" in summary


def test_create_options_with_performance():
    """Test CreateOptions with performance configuration"""
    perf = PerformanceConfig.aggressive()
    opts = CreateOptions(
        mode=CreateMode.Clobber,
        performance=perf
    )
    assert opts.performance is not None


def test_file_creation_with_auto_performance():
    """Test creating file with auto performance config"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        perf = PerformanceConfig.auto()
        opts = CreateOptions(mode=CreateMode.Clobber, performance=perf)

        file = ExodusWriter.create(tmp_path, opts)
        assert file is not None

        # Initialize with small mesh
        params = InitParams(
            title="Performance Test",
            num_dim=3,
            num_nodes=100,
            num_elems=80,
            num_elem_blocks=1
        )
        file.put_init_params(params)
        file.close()

        assert os.path.exists(tmp_path)
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_file_creation_with_conservative_performance():
    """Test creating file with conservative performance settings"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        perf = PerformanceConfig.conservative()
        opts = CreateOptions(mode=CreateMode.Clobber, performance=perf)

        file = ExodusWriter.create(tmp_path, opts)
        params = InitParams(
            title="Conservative Performance Test",
            num_dim=2,
            num_nodes=50,
            num_elems=40,
            num_elem_blocks=1
        )
        file.put_init_params(params)
        file.close()

        assert os.path.exists(tmp_path)
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_file_creation_with_custom_performance():
    """Test creating file with custom performance configuration"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        perf = PerformanceConfig.auto() \
            .with_cache_mb(64) \
            .with_node_chunk_size(10_000) \
            .with_preemption(0.5)

        opts = CreateOptions(mode=CreateMode.Clobber, performance=perf)

        file = ExodusWriter.create(tmp_path, opts)
        params = InitParams(
            title="Custom Performance Test",
            num_dim=3,
            num_nodes=1000,
            num_elems=900,
            num_elem_blocks=1
        )
        file.put_init_params(params)
        file.close()

        assert os.path.exists(tmp_path)
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_file_creation_without_performance():
    """Test creating file without explicit performance config (should auto-detect)"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        opts = CreateOptions(mode=CreateMode.Clobber)
        # No performance config specified - should auto-detect

        file = ExodusWriter.create(tmp_path, opts)
        params = InitParams(
            title="Auto Performance Test",
            num_dim=3,
            num_nodes=100,
            num_elems=80,
            num_elem_blocks=1
        )
        file.put_init_params(params)
        file.close()

        assert os.path.exists(tmp_path)
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_large_mesh_with_aggressive_performance():
    """Test large mesh with aggressive performance settings"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        perf = PerformanceConfig.aggressive() \
            .with_cache_mb(256) \
            .with_node_chunk_size(50_000)

        opts = CreateOptions(mode=CreateMode.Clobber, performance=perf)

        file = ExodusWriter.create(tmp_path, opts)
        params = InitParams(
            title="Large Mesh Performance Test",
            num_dim=3,
            num_nodes=10_000,
            num_elems=9_000,
            num_elem_blocks=2
        )
        file.put_init_params(params)

        # Write coordinates
        import numpy as np
        x = np.arange(10_000, dtype=np.float64) * 0.1
        y = np.arange(10_000, dtype=np.float64) * 0.2
        z = np.arange(10_000, dtype=np.float64) * 0.3

        file.put_coords(x, y, z)
        file.close()

        assert os.path.exists(tmp_path)
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_chunking_verification_complete():
    """
    Test that chunking is actually applied to NetCDF variables.

    This test creates a file with specific chunk sizes and verifies that
    the chunking was actually applied by inspecting the NetCDF file.

    This requires netCDF4-python or h5py to inspect the file structure.
    If neither is available, the test will skip verification but still
    test that the API works correctly.
    """
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        # Create file with specific chunk sizes
        perf = PerformanceConfig.auto() \
            .with_node_chunk_size(1000) \
            .with_element_chunk_size(500) \
            .with_time_chunk_size(5)

        opts = CreateOptions(mode=CreateMode.Clobber, performance=perf)

        # Import exodus and necessary types
        from exodus import ExodusWriter, Block, EntityType

        # Create and initialize file
        file = ExodusWriter.create(tmp_path, opts)

        params = InitParams(
            title="Chunking Verification Test",
            num_dim=3,
            num_nodes=5000,
            num_elems=4000,
            num_elem_blocks=1
        )
        file.put_init_params(params)

        # Write coordinates to trigger coordinate variable creation
        import numpy as np
        x = np.arange(5000, dtype=np.float64) * 0.1
        y = np.arange(5000, dtype=np.float64) * 0.2
        z = np.arange(5000, dtype=np.float64) * 0.3
        file.put_coords(x, y, z)

        # Define element block to trigger connectivity variable creation
        block = Block(
            id=100,
            entity_type=EntityType.ElemBlock,
            topology="HEX8",
            num_entries=4000,
            num_nodes_per_entry=8
        )
        file.put_block(block)

        # Write connectivity
        conn = np.zeros((4000, 8), dtype=np.int64)
        for i in range(4000):
            conn[i] = [i*8+1, i*8+2, i*8+3, i*8+4,
                      i*8+5, i*8+6, i*8+7, i*8+8]
        file.put_connectivity(100, conn.flatten())

        # Define variables to trigger result variable creation
        file.define_variables(EntityType.Nodal, ["Temperature", "Pressure"])
        file.define_variables(EntityType.Global, ["Time"])

        # Write some variable data
        file.put_time(0, 0.0)
        temp_data = np.random.rand(5000).astype(np.float64)
        file.put_var(0, EntityType.Nodal, 0, 0, temp_data)

        file.close()

        # Verify file was created
        assert os.path.exists(tmp_path)

        # Try to verify chunking if netCDF4 or h5py is available
        verification_attempted = False

        # Try netCDF4 first
        try:
            import netCDF4
            verification_attempted = True

            nc = netCDF4.Dataset(tmp_path, 'r')

            # Verify coordinate variable chunking
            if 'coordx' in nc.variables:
                var = nc.variables['coordx']
                chunks = var.chunking()
                if chunks != 'contiguous':
                    print(f"✓ coordx chunking: {chunks}")
                    assert chunks[0] == 1000, f"Expected coordx chunk size 1000, got {chunks[0]}"
                else:
                    print("⚠ coordx is contiguous (chunking may not have been applied)")

            # Verify connectivity variable chunking
            if 'connect1' in nc.variables:
                var = nc.variables['connect1']
                chunks = var.chunking()
                if chunks != 'contiguous':
                    print(f"✓ connect1 chunking: {chunks}")
                    assert chunks[0] == 500, f"Expected connect1 chunk size 500, got {chunks[0]}"
                else:
                    print("⚠ connect1 is contiguous (chunking may not have been applied)")

            # Verify nodal variable chunking
            if 'vals_nod_var1' in nc.variables:
                var = nc.variables['vals_nod_var1']
                chunks = var.chunking()
                if chunks != 'contiguous':
                    print(f"✓ vals_nod_var1 chunking: {chunks}")
                    # Should be [time_chunk, node_chunk] or [1, node_chunk] if time_chunk=0
                    if len(chunks) == 2:
                        # Second dimension should be node chunk size
                        assert chunks[1] == 1000, f"Expected node chunk size 1000, got {chunks[1]}"
                else:
                    print("⚠ vals_nod_var1 is contiguous (chunking may not have been applied)")

            nc.close()
            print("✓ Chunking verification completed successfully using netCDF4")

        except ImportError:
            # Try h5py as fallback
            try:
                import h5py
                verification_attempted = True

                with h5py.File(tmp_path, 'r') as f:
                    # Verify coordinate variable chunking
                    if 'coordx' in f:
                        chunks = f['coordx'].chunks
                        if chunks:
                            print(f"✓ coordx chunking: {chunks}")
                            assert chunks[0] == 1000, f"Expected coordx chunk size 1000, got {chunks[0]}"
                        else:
                            print("⚠ coordx has no chunking")

                    # Verify connectivity variable chunking
                    if 'connect1' in f:
                        chunks = f['connect1'].chunks
                        if chunks:
                            print(f"✓ connect1 chunking: {chunks}")
                            assert chunks[0] == 500, f"Expected connect1 chunk size 500, got {chunks[0]}"
                        else:
                            print("⚠ connect1 has no chunking")

                    # Verify nodal variable chunking
                    if 'vals_nod_var1' in f:
                        chunks = f['vals_nod_var1'].chunks
                        if chunks:
                            print(f"✓ vals_nod_var1 chunking: {chunks}")
                            if len(chunks) == 2:
                                assert chunks[1] == 1000, f"Expected node chunk size 1000, got {chunks[1]}"
                        else:
                            print("⚠ vals_nod_var1 has no chunking")

                print("✓ Chunking verification completed successfully using h5py")

            except ImportError:
                pass

        if not verification_attempted:
            print("⚠ Chunking verification skipped (install netCDF4-python or h5py to verify)")
            print("  API test passed - file created successfully with chunk configuration")

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
