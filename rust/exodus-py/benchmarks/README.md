# exodus-py Performance Benchmarks

This directory contains performance benchmarks for comparing NumPy array operations vs Python list operations in exodus-py.

## Overview

The benchmarks measure performance differences across several key operations:

1. **Mesh Data Reading**
   - Coordinate reading (NumPy 2D array vs tuple of lists)
   - Connectivity reading (NumPy 2D array vs flat list)
   - Node set reading

2. **Time-History Variable Access**
   - Full time-series reading for all nodes
   - Single node time-history extraction with statistics

3. **SciPy Transformations** (requires scipy)
   - Rotation using scipy.spatial.transform
   - Scale and translate operations
   - Distance matrix computation

## Running Benchmarks

### Quick Start

```bash
# From the exodus-py directory
cd rust/exodus-py

# Install dependencies first
./install_and_test.sh

# Run benchmarks with default settings (small and medium mesh)
python benchmarks/benchmark_numpy_vs_lists.py

# Run with all mesh sizes
python benchmarks/benchmark_numpy_vs_lists.py --all

# Run with specific sizes
python benchmarks/benchmark_numpy_vs_lists.py --sizes small,medium,large

# Adjust number of runs for more accurate timing
python benchmarks/benchmark_numpy_vs_lists.py --runs 10
```

### Mesh Sizes

| Size   | Nodes    | Elements | Time Steps | Description |
|--------|----------|----------|------------|-------------|
| small  | 1,000    | 900      | 10         | Quick benchmarks |
| medium | 10,000   | 9,000    | 50         | Representative workload |
| large  | 100,000  | 90,000   | 100        | Stress test |

## Expected Results

Based on the NumPy integration design, you should observe:

- **2-10x speedup** for most read operations
- **50-75% memory reduction** compared to Python lists
- **Significant improvements** for time-series data access
- **Near-zero overhead** for scipy transformations (no conversion needed)

### Sample Output

```
BENCHMARK RESULTS - Mesh Size: medium
  Nodes: 10,000, Elements: 9,000, Time Steps: 50, Node Sets: 5
================================================================================

Benchmark                           NumPy Time   List Time    Speedup    Memory Saved
--------------------------------------------------------------------------------
Coordinates Read                    1.25 ms      3.45 ms      2.76x      45.2%
Connectivity Read                   0.89 ms      2.12 ms      2.38x      42.8%
Node Set Read                       0.34 ms      0.42 ms      1.24x      38.5%
Time History Read (Temperature)     45.67 ms     156.23 ms    3.42x      62.1%
Single Node Time History            48.12 ms     189.45 ms    3.94x      71.3%
SciPy Rotation (90deg Z)            0.23 ms      1.45 ms      6.30x      0.0%
Scale + Translate Transform         0.12 ms      8.92 ms      74.33x     0.0%
SciPy Distance Matrix (1000 nodes)  12.34 ms     15.67 ms     1.27x      0.0%
```

## Understanding the Results

### Speedup Factor
- `Speedup = List Time / NumPy Time`
- Higher is better
- Values > 1.0 indicate NumPy is faster

### Memory Savings
- Percentage reduction in memory usage
- Calculated as `(1 - NumPy Memory / List Memory) * 100`
- Higher is better
- Note: Some operations show 0% because the final result size is the same

### Key Insights

1. **Coordinate and Connectivity Reading**: NumPy's 2D array layout is more memory-efficient than separate Python lists

2. **Time-History Access**: Largest gains here because:
   - NumPy returns a contiguous 2D array (time_steps x nodes)
   - Array slicing is O(1) vs O(n) for list reconstruction
   - Statistics (mean, max, min) are vectorized

3. **SciPy Transformations**:
   - NumPy arrays work directly with scipy
   - List version requires conversion to arrays first
   - Scale/translate shows massive speedup due to NumPy broadcasting

## Adding New Benchmarks

To add a new benchmark:

1. Create a function following the pattern:
```python
def benchmark_my_operation(filename: str, num_runs: int = 5) -> BenchmarkResult:
    reader = ExodusReader.open(filename)

    # Benchmark NumPy version
    numpy_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        # ... NumPy operation ...
        numpy_times.append(time.perf_counter() - start)

    # Benchmark list version
    list_times = []
    for _ in range(num_runs):
        gc.collect()
        start = time.perf_counter()
        # ... List operation ...
        list_times.append(time.perf_counter() - start)

    reader.close()

    return BenchmarkResult(
        name="My Operation",
        numpy_time=np.median(numpy_times),
        list_time=np.median(list_times),
        numpy_memory=...,
        list_memory=...,
        data_size=...
    )
```

2. Add the benchmark call in `run_benchmarks()`

## Dependencies

- `exodus-py` (built with NumPy support)
- `numpy`
- `scipy` (optional, for transformation benchmarks)
