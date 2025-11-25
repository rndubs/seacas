#!/usr/bin/env python3
"""
Collect and aggregate benchmark results from individual JSON files.

This script is used by the Merlin workflow to aggregate results from
all individual benchmark runs into a single benchmark_results.json file
that can be processed by plot_results.py.

Usage:
    python collect_results.py --results-dir /path/to/results --output results.json
"""

import argparse
import datetime
import glob
import json
import os
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional


# Default baseline configuration (must match pgen_configs.py)
BASELINE_CONFIG = {
    "cache_mb": 256,
    "node_chunk_size": 25000,
    "element_chunk_size": 25000,
    "time_chunk_size": 100,
    "preemption": 0.75,
}

# Parameter ranges (must match pgen_configs.py)
PARAM_RANGES = {
    "cache_mb": [64, 128, 256, 512, 1024, 2048],
    "node_chunk_size": [10000, 25000, 50000, 75000, 100000],
    "element_chunk_size": [10000, 25000, 50000, 75000],
    "time_chunk_size": [10, 50, 100, 250, 500],
    "preemption": [0.0, 0.25, 0.5, 0.75, 1.0],
}


def parse_config_from_filename(filename: str) -> Optional[Dict]:
    """
    Extract configuration from filename like result_c256_n25000_e25000_t100_p0.75.json
    """
    basename = os.path.basename(filename)
    # Pattern: result_c{cache}_n{node}_e{elem}_t{time}_p{preemption}.json
    pattern = r'result_c(\d+)_n(\d+)_e(\d+)_t(\d+)_p([\d.]+)\.json'
    match = re.match(pattern, basename)

    if match:
        return {
            "cache_mb": int(match.group(1)),
            "node_chunk_size": int(match.group(2)),
            "element_chunk_size": int(match.group(3)),
            "time_chunk_size": int(match.group(4)),
            "preemption": float(match.group(5)),
        }
    return None


def determine_varied_param(config: Dict, baseline: Dict) -> str:
    """
    Determine which parameter was varied from baseline.
    Returns the parameter name or "baseline" if all match.
    """
    varied_params = []
    for param, baseline_val in baseline.items():
        if config.get(param) != baseline_val:
            varied_params.append(param)

    if len(varied_params) == 0:
        return "baseline"
    elif len(varied_params) == 1:
        return varied_params[0]
    else:
        # Multiple parameters varied - this is a full factorial run
        return "multiple"


def collect_results(results_dir: str) -> Dict:
    """
    Collect all result JSON files from the results directory.

    Returns a combined results dictionary compatible with plot_results.py.

    Handles three types of results:
    - Successful: Contains full timing_results with time_total
    - Timeout: Contains status="timeout", elapsed_seconds, timeout_seconds
    - Error: Contains status="error", exit_code, elapsed_seconds
    """
    # Find all result JSON files
    pattern = os.path.join(results_dir, "result_*.json")
    result_files = glob.glob(pattern)

    if not result_files:
        print(f"WARNING: No result files found matching {pattern}")
        return None

    print(f"Found {len(result_files)} result files")

    runs = []
    successful = 0
    failed = 0
    timed_out = 0
    errored = 0

    for idx, filepath in enumerate(sorted(result_files)):
        # Parse config from filename
        config = parse_config_from_filename(filepath)
        if not config:
            print(f"  WARNING: Could not parse config from {filepath}")
            failed += 1
            continue

        # Load the result JSON
        try:
            with open(filepath, 'r') as f:
                result_data = json.load(f)
        except Exception as e:
            print(f"  WARNING: Failed to load {filepath}: {e}")
            runs.append({
                "run_id": idx,
                "param_varied": determine_varied_param(config, BASELINE_CONFIG),
                "config": config,
                "success": False,
                "status": "load_error",
                "backend": "python",
                "error": str(e),
            })
            failed += 1
            continue

        # Determine which parameter was varied
        param_varied = determine_varied_param(config, BASELINE_CONFIG)

        # Check if this is a timeout or error result
        status = result_data.get("status")

        if status == "timeout":
            # Timeout case - benchmark exceeded time limit
            run_data = {
                "run_id": idx,
                "param_varied": param_varied,
                "config": config,
                "success": False,
                "status": "timeout",
                "backend": "python",
                "timeout_seconds": result_data.get("timeout_seconds"),
                "elapsed_seconds": result_data.get("elapsed_seconds"),
                "error": result_data.get("error", "Benchmark timed out"),
            }
            runs.append(run_data)
            timed_out += 1
            print(f"  TIMEOUT: {os.path.basename(filepath)} ({param_varied}) - "
                  f"exceeded {result_data.get('timeout_seconds')}s limit")

        elif status == "error":
            # Error case - benchmark failed with non-zero exit code
            run_data = {
                "run_id": idx,
                "param_varied": param_varied,
                "config": config,
                "success": False,
                "status": "error",
                "backend": "python",
                "exit_code": result_data.get("exit_code"),
                "elapsed_seconds": result_data.get("elapsed_seconds"),
                "error": result_data.get("error", "Benchmark failed"),
            }
            runs.append(run_data)
            errored += 1
            print(f"  ERROR: {os.path.basename(filepath)} ({param_varied}) - "
                  f"exit code {result_data.get('exit_code')}")

        else:
            # Success case - full timing results available
            run_data = {
                "run_id": idx,
                "param_varied": param_varied,
                "config": config,
                "success": True,
                "status": "completed",
                "backend": "python",
                "timing_results": result_data,
            }
            runs.append(run_data)
            successful += 1
            total_time = result_data.get("time_total", 0)
            print(f"  Loaded: {os.path.basename(filepath)} ({param_varied}) - {total_time:.1f}s")

    # Build the combined results structure
    results = {
        "metadata": {
            "timestamp": datetime.datetime.now().isoformat(),
            "total_runs": len(runs),
            "successful_runs": successful,
            "timed_out_runs": timed_out,
            "errored_runs": errored,
            "failed_runs": failed,  # Load errors
            "backends": ["python"],
            "source": "merlin_workflow",
        },
        "baseline": BASELINE_CONFIG,
        "runs": runs,
    }

    # Run analysis
    results["analysis"] = analyze_results(results)

    return results


def analyze_results(results: Dict) -> Dict:
    """
    Analyze results to find optimal configurations.
    This mirrors the analysis from run_benchmark_suite.py.

    Handles timeout/error cases by:
    - Only analyzing successful (completed) runs for timing statistics
    - Tracking which configurations timed out (these are very slow)
    - Including timeout info in recommendations
    """
    analysis = {
        "by_backend": {},
        "comparison": {},
        "recommendations": [],
        "timeout_configurations": [],
        "error_configurations": [],
    }

    backend = "python"
    backend_runs = [r for r in results["runs"] if r["success"]]
    timeout_runs = [r for r in results["runs"] if r.get("status") == "timeout"]
    error_runs = [r for r in results["runs"] if r.get("status") == "error"]

    # Track timeout configurations
    for run in timeout_runs:
        analysis["timeout_configurations"].append({
            "config": run["config"],
            "param_varied": run["param_varied"],
            "timeout_seconds": run.get("timeout_seconds"),
            "elapsed_seconds": run.get("elapsed_seconds"),
        })

    # Track error configurations
    for run in error_runs:
        analysis["error_configurations"].append({
            "config": run["config"],
            "param_varied": run["param_varied"],
            "exit_code": run.get("exit_code"),
            "error": run.get("error"),
        })

    if not backend_runs:
        if timeout_runs:
            analysis["recommendations"].append(
                f"WARNING: All {len(timeout_runs)} benchmark runs timed out. "
                "Consider increasing the timeout limit or using faster configurations."
            )
        return analysis

    backend_analysis = {
        "by_parameter": {},
        "best_overall": None,
    }

    # Find best overall configuration
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
        param_timeouts = [r for r in timeout_runs if r["param_varied"] == param_name]

        param_analysis = {
            "values": [],
            "timed_out_values": [],
            "best_value": None,
            "best_time": None,
            "improvement_vs_baseline": None,
        }

        # Track successful runs
        for run in param_runs:
            value = run["config"][param_name]
            total_time = run["timing_results"]["time_total"]
            param_analysis["values"].append({
                "value": value,
                "total_time": total_time,
                "speedup_vs_baseline": baseline_time / total_time if baseline_time and total_time > 0 else 0,
            })

        # Track timed out values (these are very slow configurations)
        for run in param_timeouts:
            value = run["config"][param_name]
            param_analysis["timed_out_values"].append({
                "value": value,
                "timeout_seconds": run.get("timeout_seconds"),
                "status": "did_not_finish",
            })

        # Find best value for this parameter (only from successful runs)
        if param_analysis["values"]:
            best = min(param_analysis["values"], key=lambda x: x["total_time"])
            param_analysis["best_value"] = best["value"]
            param_analysis["best_time"] = best["total_time"]
            if baseline_time:
                param_analysis["improvement_vs_baseline"] = (
                    (baseline_time - best["total_time"]) / baseline_time * 100
                )

        backend_analysis["by_parameter"][param_name] = param_analysis

    analysis["by_backend"][backend] = backend_analysis

    # Generate recommendations
    recommendations = []

    # Recommendations based on best parameters
    for param_name, param_data in backend_analysis.get("by_parameter", {}).items():
        improvement = param_data.get("improvement_vs_baseline")
        if improvement and improvement > 5:
            recommendations.append(
                f"Consider {param_name}={param_data['best_value']} "
                f"({improvement:.1f}% improvement)"
            )

        # Warn about timed out configurations
        timed_out = param_data.get("timed_out_values", [])
        if timed_out:
            timeout_values = [str(v["value"]) for v in timed_out]
            recommendations.append(
                f"AVOID {param_name} values [{', '.join(timeout_values)}] - "
                f"these configurations exceeded the time limit"
            )

    # Summary of timeout/error runs
    if timeout_runs:
        recommendations.append(
            f"Note: {len(timeout_runs)} configuration(s) timed out and were excluded from analysis"
        )
    if error_runs:
        recommendations.append(
            f"Note: {len(error_runs)} configuration(s) failed with errors"
        )

    analysis["recommendations"] = recommendations

    return analysis


def main():
    parser = argparse.ArgumentParser(
        description="Collect and aggregate benchmark results from Merlin workflow"
    )
    parser.add_argument(
        "--results-dir", "-r",
        type=str,
        required=True,
        help="Directory containing result JSON files"
    )
    parser.add_argument(
        "--output", "-o",
        type=str,
        required=True,
        help="Output path for combined results JSON"
    )

    args = parser.parse_args()

    # Validate input
    if not os.path.isdir(args.results_dir):
        print(f"ERROR: Results directory not found: {args.results_dir}")
        return 1

    # Collect results
    print(f"Collecting results from: {args.results_dir}")
    results = collect_results(args.results_dir)

    if not results:
        print("ERROR: No results collected")
        return 1

    # Write combined results
    with open(args.output, 'w') as f:
        json.dump(results, f, indent=2)

    print(f"\nCombined results written to: {args.output}")
    print(f"  Total runs: {results['metadata']['total_runs']}")
    print(f"  Successful: {results['metadata']['successful_runs']}")
    print(f"  Timed out: {results['metadata']['timed_out_runs']}")
    print(f"  Errored: {results['metadata']['errored_runs']}")
    print(f"  Load errors: {results['metadata']['failed_runs']}")

    # Print timeout details if any
    timeout_configs = results.get("analysis", {}).get("timeout_configurations", [])
    if timeout_configs:
        print(f"\nTimed out configurations (exceeded time limit, marked as DNF):")
        for tc in timeout_configs:
            config = tc["config"]
            print(f"  - {tc['param_varied']}: cache={config['cache_mb']}MB, "
                  f"node={config['node_chunk_size']}, elem={config['element_chunk_size']}, "
                  f"time={config['time_chunk_size']}, preempt={config['preemption']}")

    # Print error details if any
    error_configs = results.get("analysis", {}).get("error_configurations", [])
    if error_configs:
        print(f"\nErrored configurations:")
        for ec in error_configs:
            config = ec["config"]
            print(f"  - {ec['param_varied']}: exit_code={ec.get('exit_code')} - {ec.get('error', 'Unknown error')}")

    # Print summary
    best = results.get("analysis", {}).get("by_backend", {}).get("python", {}).get("best_overall")
    if best:
        print(f"\nBest configuration:")
        print(f"  Total time: {best['total_time']:.2f}s")
        print(f"  Config: {best['config']}")

    recommendations = results.get("analysis", {}).get("recommendations", [])
    if recommendations:
        print(f"\nRecommendations:")
        for rec in recommendations:
            print(f"  - {rec}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
