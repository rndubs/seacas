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
                timing_results = json.load(f)
        except Exception as e:
            print(f"  WARNING: Failed to load {filepath}: {e}")
            runs.append({
                "run_id": idx,
                "param_varied": determine_varied_param(config, BASELINE_CONFIG),
                "config": config,
                "success": False,
                "backend": "python",
                "error": str(e),
            })
            failed += 1
            continue

        # Determine which parameter was varied
        param_varied = determine_varied_param(config, BASELINE_CONFIG)

        # Build run data
        run_data = {
            "run_id": idx,
            "param_varied": param_varied,
            "config": config,
            "success": True,
            "backend": "python",
            "timing_results": timing_results,
        }

        runs.append(run_data)
        successful += 1
        print(f"  Loaded: {os.path.basename(filepath)} ({param_varied})")

    # Build the combined results structure
    results = {
        "metadata": {
            "timestamp": datetime.datetime.now().isoformat(),
            "total_runs": len(runs),
            "successful_runs": successful,
            "failed_runs": failed,
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
    """
    analysis = {
        "by_backend": {},
        "comparison": {},
        "recommendations": [],
    }

    backend = "python"
    backend_runs = [r for r in results["runs"] if r["success"]]

    if not backend_runs:
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
            param_analysis["improvement_vs_baseline"] = (
                (baseline_time - best["total_time"]) / baseline_time * 100
            )

        backend_analysis["by_parameter"][param_name] = param_analysis

    analysis["by_backend"][backend] = backend_analysis

    # Generate recommendations
    recommendations = []
    for param_name, param_data in backend_analysis.get("by_parameter", {}).items():
        improvement = param_data.get("improvement_vs_baseline")
        if improvement and improvement > 5:
            recommendations.append(
                f"Consider {param_name}={param_data['best_value']} "
                f"({improvement:.1f}% improvement)"
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
    print(f"  Failed: {results['metadata']['failed_runs']}")

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
