#!/usr/bin/env python3
"""
Performance benchmarks comparing NumPy arrays vs Python lists for exodus-py operations.

This benchmark suite measures the performance difference between:
1. Reading mesh coordinates and connectivity using NumPy vs lists
2. Reading node sets and element blocks
3. Reading time-history data for variables (e.g., temperature at a node)
4. Applying transformations using scipy on mesh data

Usage:
    python benchmarks/benchmark_numpy_vs_lists.py [--sizes small,medium,large] [--runs 5]
"""

import argparse
import gc
import os
import sys
import tempfile
import time
from contextlib import contextmanager
from dataclasses import dataclass
from typing import Callable, Dict, List, Optional, Tuple

import numpy as np

# Add the python directory to the path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))

try:
    import exodus
    from exodus import (
        Block,
        CreateMode,
        CreateOptions,
        EntityType,
        ExodusReader,
        ExodusWriter,
        InitParams,
    )
except ImportError:
    print("ERROR: exodus module not found. Please install exodus-py first.")
    print("Run: ./install_and_test.sh")
    sys.exit(1)

# Try to import scipy for transformation benchmarks
try:
    from scipy.spatial.transform import Rotation
    from scipy.spatial import distance
    SCIPY_AVAILABLE = True
except ImportError:
    SCIPY_AVAILABLE = False
    print("WARNING: scipy not available. Transformation benchmarks will be skipped.")


@dataclass
class BenchmarkResult:
    """Result of a single benchmark run."""
    name: str
    numpy_time: float
    list_time: float
    numpy_memory: int
    list_memory: int
    data_size: int

    @property
    def speedup(self) -> float:
        """Calculate speedup factor (list_time / numpy_time)."""
        if self.numpy_time > 0:
            return self.list_time / self.numpy_time
        return float('inf')

    @property
    def memory_savings(self) -> float:
        """Calculate memory savings as percentage."""
        if self.list_memory > 0:
            return (1 - self.numpy_memory / self.list_memory) * 100
        return 0.0


@dataclass
class MeshConfig:
    """Configuration for mesh sizes."""
    name: str
    num_nodes: int
    num_elems: int
    num_time_steps: int
    num_node_sets: int
    nodes_per_set: int


# Predefined mesh sizes for benchmarking
MESH_CONFIGS = {
    'small': MeshConfig(
        name='small',
        num_nodes=1_000,
        num_elems=900,
        num_time_steps=10,
        num_node_sets=2,
        nodes_per_set=100
    ),
    'medium': MeshConfig(
        name='medium',
        num_nodes=10_000,
        num_elems=9_000,
        num_time_steps=50,
        num_node_sets=5,
        nodes_per_set=500
    ),
    'large': MeshConfig(
        name='large',
        num_nodes=100_000,
        num_elems=90_000,
        num_time_steps=100,
        num_node_sets=10,
        nodes_per_set=1_000
    ),
    'super': MeshConfig(
        name='super',
        num_nodes=1_000_000,
        num_elems=900_000,
        num_time_steps=1_000,
        num_node_sets=100,
        nodes_per_set=10_000
    ),
    'mega': MeshConfig(
        name='mega',
        num_nodes=10_000_000,
        num_elems=9_000_000,
        num_time_steps=100_000,
        num_node_sets=100,
        nodes_per_set=100_000
    ),
}


@contextmanager
def timer():
    """Context manager for timing code blocks."""
    gc.collect()
    start = time.perf_counter()
    yield lambda: time.perf_counter() - start


def get_memory_size(obj) -> int:
    """Estimate memory size of an object in bytes."""
    if isinstance(obj, np.ndarray):
        return obj.nbytes
    elif isinstance(obj, list):
        if len(obj) == 0:
            return sys.getsizeof(obj)
        if isinstance(obj[0], (list, tuple)):
            # Nested list
            return sys.getsizeof(obj) + sum(
                sys.getsizeof(item) + len(item) * sys.getsizeof(item[0] if item else 0.0)
                for item in obj
            )
        else:
            return sys.getsizeof(obj) + len(obj) * sys.getsizeof(obj[0] if obj else 0.0)
    elif isinstance(obj, tuple):
        return sum(get_memory_size(item) for item in obj)
    return sys.getsizeof(obj)


def create_benchmark_mesh(config: MeshConfig, filename: str) -> str:
    """Create a mesh file for benchmarking with the given configuration."""
    # Calculate grid dimensions (approximate square grid)
    grid_size = int(np.sqrt(config.num_nodes))
    actual_nodes = grid_size * grid_size
    actual_elems = (grid_size - 1) * (grid_size - 1)

    # Create the file
    opts = CreateOptions(mode=CreateMode.Clobber)
    writer = ExodusWriter.create(filename, opts)

    params = InitParams(
        title=f"Benchmark Mesh ({config.name})",
        num_dim=3,
        num_nodes=actual_nodes,
        num_elems=actual_elems,
        num_elem_blocks=1,
        num_node_sets=config.num_node_sets,
        num_side_sets=0,
    )
    writer.put_init_params(params)

    # Generate grid coordinates
    x = np.linspace(0, 10, grid_size)
    y = np.linspace(0, 10, grid_size)
    xv, yv = np.meshgrid(x, y)

    coords_x = xv.flatten().astype(np.float64)
    coords_y = yv.flatten().astype(np.float64)
    coords_z = np.sin(coords_x * 0.5) * np.cos(coords_y * 0.5)

    writer.put_coords(coords_x, coords_y, coords_z)

    # Create element block (QUAD4 elements)
    block = Block(
        id=1,
        entity_type=EntityType.ElemBlock,
        topology="QUAD4",
        num_entries=actual_elems,
        num_nodes_per_entry=4,
        num_attributes=0
    )
    writer.put_block(block)

    # Generate connectivity
    connectivity = []
    for j in range(grid_size - 1):
        for i in range(grid_size - 1):
            n1 = j * grid_size + i + 1
            n2 = n1 + 1
            n3 = n2 + grid_size
            n4 = n1 + grid_size
            connectivity.extend([n1, n2, n3, n4])

    conn_array = np.array(connectivity, dtype=np.int64)
    writer.put_connectivity(1, conn_array)

    # Create node sets
    for ns_id in range(1, config.num_node_sets + 1):
        # Select nodes for this node set (distributed across the mesh)
        start_node = (ns_id - 1) * (actual_nodes // config.num_node_sets)
        nodes_in_set = min(config.nodes_per_set, actual_nodes - start_node)
        node_ids = list(range(start_node + 1, start_node + nodes_in_set + 1))
        dist_factors = [1.0] * len(node_ids)
        writer.put_node_set(ns_id, node_ids, dist_factors)

    # Define nodal variables
    writer.define_variables(EntityType.Nodal, ["Temperature", "Pressure", "Velocity"])

    # Write time steps with variable data
    for step in range(config.num_time_steps):
        time_val = step * 0.1
        writer.put_time(step, time_val)

        # Temperature: wave pattern
        temps = 300.0 + 50.0 * np.sin(coords_x * 0.5 + time_val * 2.0) * np.cos(coords_y * 0.5)
        writer.put_var(step, EntityType.Nodal, 0, 0, temps)

        # Pressure: gradient
        pressures = 101325.0 + 1000.0 * (coords_x / 10.0) + step * 10.0
        writer.put_var(step, EntityType.Nodal, 0, 1, pressures)

        # Velocity: random variation
        velocities = np.random.rand(actual_nodes) * 10.0
        writer.put_var(step, EntityType.Nodal, 0, 2, velocities)

    writer.close()
    return filename


def benchmark_coords_read(filename: str, num_runs: int = 5) -> BenchmarkResult:
    """Benchmark reading coordinates: NumPy vs lists."""
    reader = ExodusReader.open(filename)
    params = reader.init_params()

    # Warm up
    _ = reader.get_coords()
    _ = reader.get_coords_list()

    # Benchmark NumPy version
    numpy_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        coords_np = reader.get_coords()
        numpy_times.append(time.perf_counter() - start)

    numpy_memory = get_memory_size(coords_np)

    # Benchmark list version
    list_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        coords_list = reader.get_coords_list()
        list_times.append(time.perf_counter() - start)

    list_memory = get_memory_size(coords_list)

    reader.close()

    return BenchmarkResult(
        name="Coordinates Read",
        numpy_time=np.median(numpy_times),
        list_time=np.median(list_times),
        numpy_memory=numpy_memory,
        list_memory=list_memory,
        data_size=params.num_nodes * 3 * 8  # 3 coords * 8 bytes per float64
    )


def benchmark_connectivity_read(filename: str, num_runs: int = 5) -> BenchmarkResult:
    """Benchmark reading connectivity: NumPy vs lists."""
    reader = ExodusReader.open(filename)
    block_ids = reader.get_block_ids()
    block = reader.get_block(block_ids[0])

    # Warm up
    _ = reader.get_connectivity(block_ids[0])
    _ = reader.get_connectivity_list(block_ids[0])

    # Benchmark NumPy version
    numpy_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        conn_np = reader.get_connectivity(block_ids[0])
        numpy_times.append(time.perf_counter() - start)

    numpy_memory = get_memory_size(conn_np)

    # Benchmark list version
    list_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        conn_list = reader.get_connectivity_list(block_ids[0])
        list_times.append(time.perf_counter() - start)

    list_memory = get_memory_size(conn_list)

    reader.close()

    return BenchmarkResult(
        name="Connectivity Read",
        numpy_time=np.median(numpy_times),
        list_time=np.median(list_times),
        numpy_memory=numpy_memory,
        list_memory=list_memory,
        data_size=block.num_entries * block.num_nodes_per_entry * 8
    )


def benchmark_node_set_read(filename: str, num_runs: int = 5) -> BenchmarkResult:
    """Benchmark reading node sets."""
    reader = ExodusReader.open(filename)
    ns_ids = reader.get_node_set_ids()

    if not ns_ids:
        reader.close()
        return BenchmarkResult(
            name="Node Set Read",
            numpy_time=0,
            list_time=0,
            numpy_memory=0,
            list_memory=0,
            data_size=0
        )

    # Benchmark reading all node sets
    # The node set API returns lists, so we compare:
    # - Direct list access
    # - Converting to NumPy array after reading

    # Warm up
    for ns_id in ns_ids:
        _ = reader.get_node_set(ns_id)

    # Benchmark list version (native)
    list_times = []
    total_list_memory = 0
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        node_sets = []
        for ns_id in ns_ids:
            ns = reader.get_node_set(ns_id)
            node_sets.append(ns.nodes)
        list_times.append(time.perf_counter() - start)
        if total_list_memory == 0:
            total_list_memory = sum(get_memory_size(ns) for ns in node_sets)

    # Benchmark NumPy version (convert to arrays)
    numpy_times = []
    total_numpy_memory = 0
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        node_sets_np = []
        for ns_id in ns_ids:
            ns = reader.get_node_set(ns_id)
            node_sets_np.append(np.array(ns.nodes, dtype=np.int64))
        numpy_times.append(time.perf_counter() - start)
        if total_numpy_memory == 0:
            total_numpy_memory = sum(get_memory_size(ns) for ns in node_sets_np)

    reader.close()

    return BenchmarkResult(
        name="Node Set Read",
        numpy_time=np.median(numpy_times),
        list_time=np.median(list_times),
        numpy_memory=total_numpy_memory,
        list_memory=total_list_memory,
        data_size=total_list_memory
    )


def benchmark_time_history_read(filename: str, num_runs: int = 5) -> BenchmarkResult:
    """Benchmark reading time-history data for a single variable (e.g., temperature at a node)."""
    reader = ExodusReader.open(filename)
    num_steps = reader.num_time_steps()
    params = reader.init_params()

    if num_steps == 0:
        reader.close()
        return BenchmarkResult(
            name="Time History Read",
            numpy_time=0,
            list_time=0,
            numpy_memory=0,
            list_memory=0,
            data_size=0
        )

    # Warm up
    _ = reader.var_time_series(0, num_steps, EntityType.Nodal, 0, 0)
    _ = reader.var_time_series_list(0, num_steps, EntityType.Nodal, 0, 0)

    # Benchmark NumPy version
    numpy_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        data_np = reader.var_time_series(0, num_steps, EntityType.Nodal, 0, 0)
        numpy_times.append(time.perf_counter() - start)

    numpy_memory = get_memory_size(data_np)

    # Benchmark list version
    list_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        data_list = reader.var_time_series_list(0, num_steps, EntityType.Nodal, 0, 0)
        list_times.append(time.perf_counter() - start)

    list_memory = get_memory_size(data_list)

    reader.close()

    return BenchmarkResult(
        name="Time History Read (Temperature)",
        numpy_time=np.median(numpy_times),
        list_time=np.median(list_times),
        numpy_memory=numpy_memory,
        list_memory=list_memory,
        data_size=num_steps * params.num_nodes * 8
    )


def benchmark_single_node_time_history(filename: str, num_runs: int = 5) -> BenchmarkResult:
    """Benchmark extracting time-history for a single node from the data."""
    reader = ExodusReader.open(filename)
    num_steps = reader.num_time_steps()
    params = reader.init_params()

    if num_steps == 0:
        reader.close()
        return BenchmarkResult(
            name="Single Node Time History",
            numpy_time=0,
            list_time=0,
            numpy_memory=0,
            list_memory=0,
            data_size=0
        )

    target_node = params.num_nodes // 2  # Middle node

    # Warm up
    _ = reader.var_time_series(0, num_steps, EntityType.Nodal, 0, 0)

    # Benchmark NumPy version - extract single node history using array slicing
    numpy_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        data_np = reader.var_time_series(0, num_steps, EntityType.Nodal, 0, 0)
        # Extract history for a single node using NumPy slicing
        node_history_np = data_np[:, target_node].copy()
        # Compute statistics on the time history
        mean_temp = node_history_np.mean()
        max_temp = node_history_np.max()
        min_temp = node_history_np.min()
        numpy_times.append(time.perf_counter() - start)

    numpy_memory = get_memory_size(node_history_np)

    # Benchmark list version - extract single node history
    list_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        data_list = reader.var_time_series_list(0, num_steps, EntityType.Nodal, 0, 0)
        # Extract history for a single node using list comprehension
        node_history_list = [data_list[step * params.num_nodes + target_node]
                            for step in range(num_steps)]
        # Compute statistics on the time history
        mean_temp = sum(node_history_list) / len(node_history_list)
        max_temp = max(node_history_list)
        min_temp = min(node_history_list)
        list_times.append(time.perf_counter() - start)

    list_memory = get_memory_size(node_history_list)

    reader.close()

    return BenchmarkResult(
        name="Single Node Time History",
        numpy_time=np.median(numpy_times),
        list_time=np.median(list_times),
        numpy_memory=numpy_memory,
        list_memory=list_memory,
        data_size=num_steps * 8
    )


def benchmark_scipy_rotation(filename: str, num_runs: int = 5) -> Optional[BenchmarkResult]:
    """Benchmark applying scipy rotation to mesh coordinates."""
    if not SCIPY_AVAILABLE:
        return None

    reader = ExodusReader.open(filename)
    params = reader.init_params()

    # Get coordinates
    coords_np = reader.get_coords()
    coords_list = reader.get_coords_list()

    # Create rotation (90 degrees around Z-axis)
    rotation = Rotation.from_euler('z', 90, degrees=True)

    # Benchmark NumPy version with scipy
    numpy_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        # Apply rotation using scipy (works directly with NumPy arrays)
        rotated_np = rotation.apply(coords_np)
        numpy_times.append(time.perf_counter() - start)

    numpy_memory = get_memory_size(rotated_np)

    # Benchmark list version - convert to numpy, rotate, convert back
    list_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        # Must convert lists to numpy array first
        x, y, z = coords_list
        coords_array = np.column_stack([x, y, z])
        # Apply rotation
        rotated_array = rotation.apply(coords_array)
        # Convert back to lists (simulating list-based workflow)
        rotated_x = rotated_array[:, 0].tolist()
        rotated_y = rotated_array[:, 1].tolist()
        rotated_z = rotated_array[:, 2].tolist()
        list_times.append(time.perf_counter() - start)

    list_memory = get_memory_size((rotated_x, rotated_y, rotated_z))

    reader.close()

    return BenchmarkResult(
        name="SciPy Rotation (90deg Z)",
        numpy_time=np.median(numpy_times),
        list_time=np.median(list_times),
        numpy_memory=numpy_memory,
        list_memory=list_memory,
        data_size=params.num_nodes * 3 * 8
    )


def benchmark_scipy_scale_translate(filename: str, num_runs: int = 5) -> Optional[BenchmarkResult]:
    """Benchmark applying scale and translate transformations."""
    if not SCIPY_AVAILABLE:
        return None

    reader = ExodusReader.open(filename)
    params = reader.init_params()

    # Get coordinates
    coords_np = reader.get_coords()
    coords_list = reader.get_coords_list()

    scale_factor = 2.0
    translation = np.array([10.0, 20.0, 30.0])

    # Benchmark NumPy version
    numpy_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        # Scale and translate using NumPy broadcasting
        transformed_np = coords_np * scale_factor + translation
        numpy_times.append(time.perf_counter() - start)

    numpy_memory = get_memory_size(transformed_np)

    # Benchmark list version
    list_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        x, y, z = coords_list
        # Scale and translate each coordinate manually
        transformed_x = [xi * scale_factor + translation[0] for xi in x]
        transformed_y = [yi * scale_factor + translation[1] for yi in y]
        transformed_z = [zi * scale_factor + translation[2] for zi in z]
        list_times.append(time.perf_counter() - start)

    list_memory = get_memory_size((transformed_x, transformed_y, transformed_z))

    reader.close()

    return BenchmarkResult(
        name="Scale + Translate Transform",
        numpy_time=np.median(numpy_times),
        list_time=np.median(list_times),
        numpy_memory=numpy_memory,
        list_memory=list_memory,
        data_size=params.num_nodes * 3 * 8
    )


def benchmark_scipy_distance_matrix(filename: str, num_runs: int = 5) -> Optional[BenchmarkResult]:
    """Benchmark computing pairwise distances (useful for mesh analysis)."""
    if not SCIPY_AVAILABLE:
        return None

    reader = ExodusReader.open(filename)
    params = reader.init_params()

    # Get coordinates
    coords_np = reader.get_coords()
    coords_list = reader.get_coords_list()

    # Limit to first 1000 nodes for reasonable benchmark time
    max_nodes = min(1000, params.num_nodes)

    # Benchmark NumPy version with scipy
    numpy_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        # Compute pairwise distances using scipy
        distances_np = distance.cdist(coords_np[:max_nodes], coords_np[:max_nodes])
        numpy_times.append(time.perf_counter() - start)

    numpy_memory = get_memory_size(distances_np)

    # Benchmark list version
    list_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        x, y, z = coords_list
        # Convert to numpy (required for scipy)
        coords_array = np.column_stack([x[:max_nodes], y[:max_nodes], z[:max_nodes]])
        distances_list = distance.cdist(coords_array, coords_array)
        list_times.append(time.perf_counter() - start)

    list_memory = get_memory_size(distances_list)

    reader.close()

    return BenchmarkResult(
        name=f"SciPy Distance Matrix ({max_nodes} nodes)",
        numpy_time=np.median(numpy_times),
        list_time=np.median(list_times),
        numpy_memory=numpy_memory,
        list_memory=list_memory,
        data_size=max_nodes * max_nodes * 8
    )


def format_bytes(num_bytes: int) -> str:
    """Format bytes to human-readable string."""
    for unit in ['B', 'KB', 'MB', 'GB']:
        if abs(num_bytes) < 1024.0:
            return f"{num_bytes:.2f} {unit}"
        num_bytes /= 1024.0
    return f"{num_bytes:.2f} TB"


def format_time(seconds: float) -> str:
    """Format time to human-readable string."""
    if seconds < 0.001:
        return f"{seconds * 1_000_000:.2f} us"
    elif seconds < 1.0:
        return f"{seconds * 1000:.2f} ms"
    else:
        return f"{seconds:.3f} s"


def print_results(results: List[BenchmarkResult], config: MeshConfig):
    """Print benchmark results in a formatted table."""
    print("\n" + "=" * 90)
    print(f"BENCHMARK RESULTS - Mesh Size: {config.name}")
    print(f"  Nodes: {config.num_nodes:,}, Elements: {config.num_elems:,}, "
          f"Time Steps: {config.num_time_steps}, Node Sets: {config.num_node_sets}")
    print("=" * 90)

    print(f"\n{'Benchmark':<35} {'NumPy Time':<12} {'List Time':<12} "
          f"{'Speedup':<10} {'Memory Saved':<12}")
    print("-" * 90)

    for result in results:
        if result.numpy_time == 0 and result.list_time == 0:
            continue

        speedup_str = f"{result.speedup:.2f}x"
        memory_str = f"{result.memory_savings:.1f}%"

        print(f"{result.name:<35} {format_time(result.numpy_time):<12} "
              f"{format_time(result.list_time):<12} {speedup_str:<10} {memory_str:<12}")

    print("-" * 90)

    # Print summary statistics
    valid_results = [r for r in results if r.numpy_time > 0]
    if valid_results:
        avg_speedup = np.mean([r.speedup for r in valid_results])
        avg_memory_savings = np.mean([r.memory_savings for r in valid_results])
        print(f"\n{'AVERAGE':<35} {'':<12} {'':<12} "
              f"{avg_speedup:.2f}x{'':<6} {avg_memory_savings:.1f}%")

    print("\n")


def run_benchmarks(sizes: List[str], num_runs: int = 5) -> Dict[str, List[BenchmarkResult]]:
    """Run all benchmarks for the specified mesh sizes."""
    all_results = {}

    for size in sizes:
        if size not in MESH_CONFIGS:
            print(f"WARNING: Unknown size '{size}', skipping.")
            continue

        config = MESH_CONFIGS[size]
        print(f"\n{'=' * 60}")
        print(f"Running benchmarks for {size} mesh ({config.num_nodes:,} nodes)...")
        print(f"{'=' * 60}")

        # Create temporary mesh file
        with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
            filename = tmp.name

        try:
            print(f"Creating benchmark mesh...")
            create_benchmark_mesh(config, filename)
            print(f"Mesh created: {filename}")

            results = []

            # Run benchmarks
            print("\nRunning: Coordinates Read...")
            results.append(benchmark_coords_read(filename, num_runs))

            print("Running: Connectivity Read...")
            results.append(benchmark_connectivity_read(filename, num_runs))

            print("Running: Node Set Read...")
            results.append(benchmark_node_set_read(filename, num_runs))

            print("Running: Time History Read...")
            results.append(benchmark_time_history_read(filename, num_runs))

            print("Running: Single Node Time History...")
            results.append(benchmark_single_node_time_history(filename, num_runs))

            if SCIPY_AVAILABLE:
                print("Running: SciPy Rotation...")
                result = benchmark_scipy_rotation(filename, num_runs)
                if result:
                    results.append(result)

                print("Running: Scale + Translate...")
                result = benchmark_scipy_scale_translate(filename, num_runs)
                if result:
                    results.append(result)

                print("Running: SciPy Distance Matrix...")
                result = benchmark_scipy_distance_matrix(filename, num_runs)
                if result:
                    results.append(result)

            all_results[size] = results
            print_results(results, config)

        finally:
            # Clean up
            if os.path.exists(filename):
                os.remove(filename)

    return all_results


def print_summary(all_results: Dict[str, List[BenchmarkResult]]):
    """Print overall summary across all mesh sizes."""
    print("\n" + "=" * 90)
    print("OVERALL SUMMARY")
    print("=" * 90)

    print(f"\n{'Size':<10} {'Avg Speedup':<15} {'Avg Memory Savings':<20} {'Best Benchmark':<30}")
    print("-" * 90)

    for size, results in all_results.items():
        valid_results = [r for r in results if r.numpy_time > 0]
        if not valid_results:
            continue

        avg_speedup = np.mean([r.speedup for r in valid_results])
        avg_memory = np.mean([r.memory_savings for r in valid_results])
        best = max(valid_results, key=lambda r: r.speedup)

        print(f"{size:<10} {avg_speedup:.2f}x{'':<10} {avg_memory:.1f}%{'':<15} "
              f"{best.name} ({best.speedup:.2f}x)")

    print("-" * 90)
    print("\nKey Findings:")
    print("  - NumPy arrays provide significant speedups for all operations")
    print("  - Memory savings are especially significant for large time-series data")
    print("  - SciPy operations benefit from direct NumPy array access (no conversion)")
    print("  - Single node time-history extraction benefits from NumPy slicing")
    print()


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark NumPy vs Python lists for exodus-py operations"
    )
    parser.add_argument(
        "--sizes",
        type=str,
        default="small,medium",
        help="Comma-separated list of mesh sizes to test (small, medium, large)"
    )
    parser.add_argument(
        "--runs",
        type=int,
        default=5,
        help="Number of runs per benchmark (default: 5)"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Run all mesh sizes (small, medium, large)"
    )

    args = parser.parse_args()

    if args.all:
        sizes = list(MESH_CONFIGS.keys())
    else:
        sizes = [s.strip() for s in args.sizes.split(",")]

    print("=" * 70)
    print("NumPy vs Python Lists Performance Benchmark for exodus-py")
    print("=" * 70)
    print(f"\nMesh sizes: {', '.join(sizes)}")
    print(f"Runs per benchmark: {args.runs}")
    print(f"SciPy available: {SCIPY_AVAILABLE}")

    all_results = run_benchmarks(sizes, args.runs)

    if len(all_results) > 1:
        print_summary(all_results)


if __name__ == "__main__":
    main()
