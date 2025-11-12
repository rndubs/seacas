"""Tests for performance configuration"""

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


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
