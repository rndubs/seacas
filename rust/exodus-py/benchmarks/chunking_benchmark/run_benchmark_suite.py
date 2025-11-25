#!/usr/bin/env python3
"""
Benchmark suite driver for HDF5 chunking strategy evaluation.

This script runs transform_mesh (Python or Rust) as a subprocess with various
performance configurations to evaluate the impact of different HDF5 chunking strategies.

The benchmark uses a "one-at-a-time" experimental design:
1. Run baseline with default settings
2. Vary one parameter at a time while holding others at defaults
3. This allows understanding individual parameter effects

Each run is executed as a fresh subprocess so the HDF5 library configuration
can be reset for each test (HDF5 cache settings are set via environment
variables before library initialization).

Usage:
    # Run Python benchmark
    python run_benchmark_suite.py --input-mesh /path/to/input.exo \\
        --output-dir /p/lustre1/whitmore/chunking_benchmark/results --backend python

    # Run Rust benchmark
    python run_benchmark_suite.py --input-mesh /path/to/input.exo \\
        --output-dir /p/lustre1/whitmore/chunking_benchmark/results --backend rust

    # Run both Python and Rust benchmarks for comparison
    python run_benchmark_suite.py --input-mesh /path/to/input.exo \\
        --output-dir /p/lustre1/whitmore/chunking_benchmark/results --backend both

    # Quick test with reduced parameter ranges
    python run_benchmark_suite.py --input-mesh input.exo --output-dir results --quick
"""

import argparse
import datetime
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional


# Default baseline configuration (tuned for RZHound with 256GB RAM)
BASELINE_CONFIG = {
    "cache_mb": 256,
    "node_chunk_size": 25000,
    "element_chunk_size": 25000,
    "time_chunk_size": 100,
    "preemption": 0.75,
}

# Parameter sweep ranges for RZHound (256GB RAM per node)
PARAM_RANGES = {
    "cache_mb": [64, 128, 256, 512, 1024, 2048],
    "node_chunk_size": [10000, 25000, 50000, 75000, 100000],
    "element_chunk_size": [10000, 25000, 50000, 75000],
    "time_chunk_size": [10, 50, 100, 250, 500],
    "preemption": [0.0, 0.25, 0.5, 0.75, 1.0],
}

# Quick test ranges (reduced for faster testing)
QUICK_PARAM_RANGES = {
    "cache_mb": [64, 256, 1024],
    "node_chunk_size": [10000, 50000],
    "element_chunk_size": [10000, 50000],
    "time_chunk_size": [10, 100],
    "preemption": [0.0, 0.75],
}


@dataclass
class BenchmarkRun:
    """Configuration and results for a single benchmark run."""
    run_id: int
    param_varied: str  # Which parameter was varied (or "baseline")
    config: dict
    result_file: str
    output_mesh: str
    success: bool
    backend: str = "python"  # "python" or "rust"
    error_message: Optional[str] = None
    duration_seconds: Optional[float] = None


def find_rust_binary(script_dir: str) -> Optional[str]:
    """
    Find the Rust transform_mesh binary.

    Returns the path to the binary if found, None otherwise.
    """
    # Look for the binary in the exodus-rs target directory
    exodus_rs_dir = Path(script_dir).parent.parent.parent / "exodus-rs"

    # Check release build first
    release_binary = exodus_rs_dir / "target" / "release" / "transform_mesh"
    if release_binary.exists():
        return str(release_binary)

    # Check debug build
    debug_binary = exodus_rs_dir / "target" / "debug" / "transform_mesh"
    if debug_binary.exists():
        return str(debug_binary)

    return None


def generate_run_configs(param_ranges: dict, baseline: dict) -> List[dict]:
    """
    Generate run configurations using one-at-a-time design.

    Returns list of (param_name, config_dict) tuples.
    """
    configs = []

    # First, add baseline
    configs.append(("baseline", baseline.copy()))

    # Then vary each parameter
    for param_name, values in param_ranges.items():
        for value in values:
            if value == baseline[param_name]:
                continue  # Skip baseline value (already included)

            config = baseline.copy()
            config[param_name] = value
            configs.append((param_name, config))

    return configs


def run_single_benchmark(
    run_id: int,
    param_varied: str,
    config: dict,
    input_mesh: str,
    output_dir: str,
    script_dir: str,
    backend: str = "python",
    rust_binary: Optional[str] = None,
    verbose: bool = True,
) -> BenchmarkRun:
    """
    Run a single benchmark as a subprocess.

    Args:
        backend: "python" or "rust"
        rust_binary: Path to the Rust binary (required if backend="rust")

    Returns BenchmarkRun with results.
    """
    # Create unique output filenames with backend suffix
    config_str = f"c{config['cache_mb']}_n{config['node_chunk_size']}_e{config['element_chunk_size']}_t{config['time_chunk_size']}_p{int(config['preemption']*100)}"
    output_mesh = os.path.join(output_dir, f"run_{run_id:03d}_{backend}_{config_str}.exo")
    result_file = os.path.join(output_dir, f"run_{run_id:03d}_{backend}_{config_str}.json")

    if verbose:
        print(f"\n{'=' * 70}")
        print(f"Run {run_id} [{backend.upper()}]: {param_varied}")
        print(f"  cache_mb={config['cache_mb']}, node_chunk={config['node_chunk_size']}, "
              f"elem_chunk={config['element_chunk_size']}, time_chunk={config['time_chunk_size']}, "
              f"preemption={config['preemption']}")
        print(f"{'=' * 70}")

    # Build command based on backend
    if backend == "rust":
        if not rust_binary:
            return BenchmarkRun(
                run_id=run_id,
                param_varied=param_varied,
                config=config,
                result_file=result_file,
                output_mesh=output_mesh,
                success=False,
                backend=backend,
                error_message="Rust binary not found",
            )
        cmd = [
            rust_binary,
            "--input", input_mesh,
            "--output", output_mesh,
            "--cache-mb", str(config["cache_mb"]),
            "--node-chunk-size", str(config["node_chunk_size"]),
            "--element-chunk-size", str(config["element_chunk_size"]),
            "--time-chunk-size", str(config["time_chunk_size"]),
            "--preemption", str(config["preemption"]),
            "--output-json", result_file,
        ]
        if not verbose:
            cmd.append("--quiet")
    else:  # python
        transform_script = os.path.join(script_dir, "transform_mesh.py")
        cmd = [
            sys.executable,
            transform_script,
            "--input", input_mesh,
            "--output", output_mesh,
            "--cache-mb", str(config["cache_mb"]),
            "--node-chunk-size", str(config["node_chunk_size"]),
            "--element-chunk-size", str(config["element_chunk_size"]),
            "--time-chunk-size", str(config["time_chunk_size"]),
            "--preemption", str(config["preemption"]),
            "--output-json", result_file,
        ]
        if not verbose:
            cmd.append("--quiet")

    # Run subprocess
    start_time = time.perf_counter()
    try:
        result = subprocess.run(
            cmd,
            capture_output=not verbose,
            text=True,
            timeout=7200,  # 2 hour timeout per run
        )

        duration = time.perf_counter() - start_time

        if result.returncode == 0:
            return BenchmarkRun(
                run_id=run_id,
                param_varied=param_varied,
                config=config,
                result_file=result_file,
                output_mesh=output_mesh,
                success=True,
                backend=backend,
                duration_seconds=duration,
            )
        else:
            error_msg = result.stderr if result.stderr else f"Exit code: {result.returncode}"
            return BenchmarkRun(
                run_id=run_id,
                param_varied=param_varied,
                config=config,
                result_file=result_file,
                output_mesh=output_mesh,
                success=False,
                backend=backend,
                error_message=error_msg,
                duration_seconds=duration,
            )

    except subprocess.TimeoutExpired:
        return BenchmarkRun(
            run_id=run_id,
            param_varied=param_varied,
            config=config,
            result_file=result_file,
            output_mesh=output_mesh,
            success=False,
            backend=backend,
            error_message="Timeout (>2 hours)",
            duration_seconds=7200,
        )
    except Exception as e:
        return BenchmarkRun(
            run_id=run_id,
            param_varied=param_varied,
            config=config,
            result_file=result_file,
            output_mesh=output_mesh,
            success=False,
            backend=backend,
            error_message=str(e),
        )


def collect_results(runs: List[BenchmarkRun], backends: List[str]) -> dict:
    """Collect and aggregate results from all runs."""
    results = {
        "metadata": {
            "timestamp": datetime.datetime.now().isoformat(),
            "total_runs": len(runs),
            "successful_runs": sum(1 for r in runs if r.success),
            "failed_runs": sum(1 for r in runs if not r.success),
            "backends": backends,
        },
        "baseline": BASELINE_CONFIG,
        "runs": [],
    }

    for run in runs:
        run_data = {
            "run_id": run.run_id,
            "param_varied": run.param_varied,
            "config": run.config,
            "success": run.success,
            "backend": run.backend,
            "duration_seconds": run.duration_seconds,
        }

        if run.success and os.path.exists(run.result_file):
            with open(run.result_file, 'r') as f:
                run_data["timing_results"] = json.load(f)
        elif run.error_message:
            run_data["error"] = run.error_message

        results["runs"].append(run_data)

    return results


def analyze_results(results: dict) -> dict:
    """Analyze results to find optimal configurations."""
    backends = results["metadata"].get("backends", ["python"])
    analysis = {
        "by_backend": {},
        "comparison": {},
        "recommendations": [],
    }

    # Analyze each backend separately
    for backend in backends:
        backend_runs = [r for r in results["runs"] if r["success"] and r.get("backend", "python") == backend]

        if not backend_runs:
            continue

        backend_analysis = {
            "by_parameter": {},
            "best_overall": None,
        }

        # Find best overall configuration for this backend
        best_run = min(backend_runs, key=lambda r: r["timing_results"]["time_total"])
        backend_analysis["best_overall"] = {
            "config": best_run["config"],
            "total_time": best_run["timing_results"]["time_total"],
            "read_time": best_run["timing_results"]["time_total_read"],
            "transform_time": best_run["timing_results"]["time_total_transform"],
            "write_time": best_run["timing_results"]["time_total_write"],
            "peak_memory_mb": best_run["timing_results"]["peak_memory_mb"],
        }

        # Analyze each parameter
        baseline_runs = [r for r in backend_runs if r["param_varied"] == "baseline"]
        baseline_time = baseline_runs[0]["timing_results"]["time_total"] if baseline_runs else None

        for param_name in PARAM_RANGES.keys():
            param_runs = [r for r in backend_runs if r["param_varied"] == param_name]

            if not param_runs or baseline_time is None:
                continue

            param_analysis = {
                "values": [],
                "best_value": None,
                "best_time": None,
                "improvement_vs_baseline": None,
            }

            for run in param_runs:
                value = run["config"][param_name]
                total_time = run["timing_results"]["time_total"]
                param_analysis["values"].append({
                    "value": value,
                    "total_time": total_time,
                    "speedup_vs_baseline": baseline_time / total_time if total_time > 0 else 0,
                })

            # Find best value for this parameter
            if param_analysis["values"]:
                best = min(param_analysis["values"], key=lambda x: x["total_time"])
                param_analysis["best_value"] = best["value"]
                param_analysis["best_time"] = best["total_time"]
                param_analysis["improvement_vs_baseline"] = (baseline_time - best["total_time"]) / baseline_time * 100

            backend_analysis["by_parameter"][param_name] = param_analysis

        analysis["by_backend"][backend] = backend_analysis

    # Generate comparison if both backends are present
    if "python" in analysis["by_backend"] and "rust" in analysis["by_backend"]:
        python_best = analysis["by_backend"]["python"]["best_overall"]
        rust_best = analysis["by_backend"]["rust"]["best_overall"]

        if python_best and rust_best:
            speedup = python_best["total_time"] / rust_best["total_time"] if rust_best["total_time"] > 0 else 0
            analysis["comparison"] = {
                "python_best_time": python_best["total_time"],
                "rust_best_time": rust_best["total_time"],
                "rust_speedup": speedup,
                "read_speedup": python_best["read_time"] / rust_best["read_time"] if rust_best["read_time"] > 0 else 0,
                "transform_speedup": python_best["transform_time"] / rust_best["transform_time"] if rust_best["transform_time"] > 0 else 0,
                "write_speedup": python_best["write_time"] / rust_best["write_time"] if rust_best["write_time"] > 0 else 0,
            }

    # Generate recommendations
    recommendations = []
    for backend, backend_data in analysis["by_backend"].items():
        for param_name, param_data in backend_data.get("by_parameter", {}).items():
            if param_data.get("improvement_vs_baseline") and param_data["improvement_vs_baseline"] > 5:
                recommendations.append(
                    f"[{backend}] Consider {param_name}={param_data['best_value']} "
                    f"({param_data['improvement_vs_baseline']:.1f}% improvement)"
                )

    if "comparison" in analysis and analysis["comparison"].get("rust_speedup"):
        speedup = analysis["comparison"]["rust_speedup"]
        if speedup > 1.1:
            recommendations.append(f"Rust is {speedup:.2f}x faster than Python overall")
        elif speedup < 0.9:
            recommendations.append(f"Python is {1/speedup:.2f}x faster than Rust overall")
        else:
            recommendations.append("Python and Rust have similar performance")

    analysis["recommendations"] = recommendations

    return analysis


def cleanup_output_meshes(runs: List[BenchmarkRun], keep_best: bool = True):
    """Clean up output mesh files to save disk space."""
    for run in runs:
        if os.path.exists(run.output_mesh):
            os.remove(run.output_mesh)
            print(f"Removed: {run.output_mesh}")


def main():
    parser = argparse.ArgumentParser(
        description="Run HDF5 chunking benchmark suite"
    )
    parser.add_argument(
        "--input-mesh", "-i",
        type=str,
        required=True,
        help="Input mesh file path"
    )
    parser.add_argument(
        "--output-dir", "-o",
        type=str,
        required=True,
        help="Output directory for results"
    )
    parser.add_argument(
        "--backend", "-b",
        type=str,
        default="python",
        choices=["python", "rust", "both"],
        help="Backend to benchmark: 'python', 'rust', or 'both' (default: python)"
    )
    parser.add_argument(
        "--rust-binary",
        type=str,
        default=None,
        help="Path to Rust transform_mesh binary (auto-detected if not specified)"
    )
    parser.add_argument(
        "--quick",
        action="store_true",
        help="Use reduced parameter ranges for quick testing"
    )
    parser.add_argument(
        "--cleanup",
        action="store_true",
        help="Remove output mesh files after benchmarking (keep only JSON results)"
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Show verbose output from each run"
    )

    args = parser.parse_args()

    # Validate input
    if not os.path.exists(args.input_mesh):
        print(f"ERROR: Input mesh not found: {args.input_mesh}")
        return 1

    # Create output directory
    os.makedirs(args.output_dir, exist_ok=True)

    script_dir = os.path.dirname(os.path.abspath(__file__))

    # Determine which backends to run
    if args.backend == "both":
        backends = ["python", "rust"]
    else:
        backends = [args.backend]

    # Find Rust binary if needed
    rust_binary = None
    if "rust" in backends:
        rust_binary = args.rust_binary or find_rust_binary(script_dir)
        if not rust_binary:
            print("ERROR: Rust binary not found. Build it with:")
            print("  cd rust/exodus-rs && cargo build --release --features 'netcdf4,cli'")
            if args.backend == "both":
                print("WARNING: Continuing with Python-only benchmark")
                backends = ["python"]
            else:
                return 1
        else:
            print(f"Using Rust binary: {rust_binary}")

    # Select parameter ranges
    param_ranges = QUICK_PARAM_RANGES if args.quick else PARAM_RANGES

    # Generate configurations
    configs = generate_run_configs(param_ranges, BASELINE_CONFIG)
    total_configs = len(configs) * len(backends)

    print(f"\n{'=' * 70}")
    print(f"HDF5 Chunking Benchmark Suite")
    print(f"{'=' * 70}")
    print(f"Input mesh: {args.input_mesh}")
    print(f"Output dir: {args.output_dir}")
    print(f"Backends: {', '.join(backends)}")
    print(f"Configurations per backend: {len(configs)}")
    print(f"Total runs: {total_configs}")
    print(f"Mode: {'Quick' if args.quick else 'Full'}")
    print(f"{'=' * 70}")

    # Run benchmarks
    runs = []
    total_start = time.perf_counter()
    run_id = 0

    for backend in backends:
        print(f"\n{'=' * 70}")
        print(f"Running {backend.upper()} benchmarks...")
        print(f"{'=' * 70}")

        for param_varied, config in configs:
            run = run_single_benchmark(
                run_id=run_id,
                param_varied=param_varied,
                config=config,
                input_mesh=args.input_mesh,
                output_dir=args.output_dir,
                script_dir=script_dir,
                backend=backend,
                rust_binary=rust_binary,
                verbose=args.verbose,
            )
            runs.append(run)

            status = "SUCCESS" if run.success else f"FAILED: {run.error_message}"
            print(f"\n  Run {run_id} [{backend}] completed: {status}")
            if run.duration_seconds:
                print(f"  Duration: {run.duration_seconds:.1f}s")

            run_id += 1

    total_duration = time.perf_counter() - total_start

    # Collect and analyze results
    print(f"\n{'=' * 70}")
    print(f"Collecting results...")
    print(f"{'=' * 70}")

    results = collect_results(runs, backends)
    results["metadata"]["total_duration_seconds"] = total_duration

    analysis = analyze_results(results)
    results["analysis"] = analysis

    # Save results
    results_file = os.path.join(args.output_dir, "benchmark_results.json")
    with open(results_file, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\nResults saved to: {results_file}")

    # Print summary
    print(f"\n{'=' * 70}")
    print(f"BENCHMARK SUMMARY")
    print(f"{'=' * 70}")
    print(f"Total runs: {results['metadata']['total_runs']}")
    print(f"Successful: {results['metadata']['successful_runs']}")
    print(f"Failed: {results['metadata']['failed_runs']}")
    print(f"Backends: {', '.join(backends)}")
    print(f"Total duration: {total_duration / 60:.1f} minutes")

    # Print per-backend summary
    for backend in backends:
        backend_analysis = analysis.get("by_backend", {}).get(backend, {})
        best_overall = backend_analysis.get("best_overall")
        if best_overall:
            print(f"\nBest Configuration [{backend.upper()}]:")
            print(f"  Total Time: {best_overall['total_time']:.2f}s")
            print(f"  Config: {best_overall['config']}")

    # Print comparison if both backends were run
    comparison = analysis.get("comparison", {})
    if comparison:
        print(f"\n{'=' * 70}")
        print(f"PYTHON vs RUST COMPARISON")
        print(f"{'=' * 70}")
        print(f"Python best time: {comparison['python_best_time']:.2f}s")
        print(f"Rust best time:   {comparison['rust_best_time']:.2f}s")
        print(f"Rust speedup:     {comparison['rust_speedup']:.2f}x")
        print(f"  Read speedup:      {comparison['read_speedup']:.2f}x")
        print(f"  Transform speedup: {comparison['transform_speedup']:.2f}x")
        print(f"  Write speedup:     {comparison['write_speedup']:.2f}x")

    if analysis.get("recommendations"):
        print(f"\nRecommendations:")
        for rec in analysis["recommendations"]:
            print(f"  - {rec}")

    # Cleanup if requested
    if args.cleanup:
        print(f"\nCleaning up output mesh files...")
        cleanup_output_meshes(runs)

    return 0


if __name__ == "__main__":
    sys.exit(main())
