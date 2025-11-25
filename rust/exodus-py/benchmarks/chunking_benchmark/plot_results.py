#!/usr/bin/env python3
"""
Generate plots from HDF5 chunking benchmark results.

This script creates various visualizations:
1. Total runtime vs each performance parameter
2. Breakdown of read/transform/write times
3. Memory usage comparison
4. Heatmaps for parameter interactions (if applicable)

Usage:
    python plot_results.py --results benchmark_results.json --output-dir plots/
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np

# Style configuration
plt.style.use('seaborn-v0_8-whitegrid')
COLORS = {
    'read': '#3498db',      # Blue
    'transform': '#2ecc71', # Green
    'write': '#e74c3c',     # Red
    'total': '#9b59b6',     # Purple
    'memory': '#f39c12',    # Orange
    'baseline': '#95a5a6',  # Gray
    'python': '#3776ab',    # Python blue
    'rust': '#dea584',      # Rust orange
}

PARAM_LABELS = {
    'cache_mb': 'Cache Size (MB)',
    'node_chunk_size': 'Node Chunk Size',
    'element_chunk_size': 'Element Chunk Size',
    'time_chunk_size': 'Time Chunk Size',
    'preemption': 'Preemption Value',
}


def load_results(results_file: str) -> dict:
    """Load benchmark results from JSON file."""
    with open(results_file, 'r') as f:
        return json.load(f)


def get_param_data(results: dict, param_name: str, backend: Optional[str] = None, include_timeouts: bool = False) -> tuple:
    """
    Extract data for a specific parameter.

    Args:
        results: Benchmark results dict
        param_name: Parameter name to extract
        backend: Filter by backend ("python", "rust", or None for all)
        include_timeouts: If True, include timed-out runs

    Returns (values, total_times, read_times, transform_times, write_times, memory, timed_out)
    """
    baseline = results.get("baseline", {})
    runs = results.get("runs", [])

    # Filter by backend if specified
    if backend:
        runs = [r for r in runs if r.get("backend", "python") == backend]

    # Get successful runs
    successful_runs = [r for r in runs if r.get("success") and "timing_results" in r]

    # Get timed-out runs if requested
    timed_out_runs = []
    if include_timeouts:
        timed_out_runs = [r for r in runs if r.get("status") == "timeout"]

    all_runs = successful_runs + timed_out_runs

    if not all_runs:
        return [], [], [], [], [], [], []

    # Filter to runs where all OTHER parameters are at baseline values
    # This handles factorial design data where all parameters vary
    filtered_runs = []
    for run in all_runs:
        config = run["config"]
        # Check if all parameters except param_name match baseline
        matches_baseline = True
        for key, baseline_value in baseline.items():
            if key != param_name and config.get(key) != baseline_value:
                matches_baseline = False
                break
        if matches_baseline:
            filtered_runs.append(run)

    # If filtering by baseline yields no data, use all runs
    # (useful for factorial designs where we still want to see trends)
    runs_to_use = filtered_runs if filtered_runs else all_runs

    # Group runs by parameter value
    param_groups = {}
    for run in runs_to_use:
        param_value = run["config"].get(param_name)
        if param_value is not None:
            if param_value not in param_groups:
                param_groups[param_value] = []
            param_groups[param_value].append(run)

    if not param_groups:
        return [], [], [], [], [], [], []

    # For each parameter value, get the best run (minimum total time)
    values = []
    total_times = []
    read_times = []
    transform_times = []
    write_times = []
    memory = []
    timed_out = []

    for param_value in sorted(param_groups.keys()):
        group_runs = param_groups[param_value]
        # Find run with minimum total time (or elapsed time for timeouts)
        best_run = min(group_runs, key=lambda x: x.get("timing_results", {}).get("time_total", x.get("elapsed_seconds", float('inf'))))

        values.append(param_value)
        is_timeout = best_run.get("status") == "timeout"
        timed_out.append(is_timeout)

        if is_timeout:
            total_times.append(best_run.get("elapsed_seconds", best_run.get("timeout_seconds", 600)))
            read_times.append(0)
            transform_times.append(0)
            write_times.append(0)
            memory.append(0)
        else:
            total_times.append(best_run["timing_results"]["time_total"])
            read_times.append(best_run["timing_results"]["time_total_read"])
            transform_times.append(best_run["timing_results"]["time_total_transform"])
            write_times.append(best_run["timing_results"]["time_total_write"])
            memory.append(best_run["timing_results"]["peak_memory_mb"])

    return values, total_times, read_times, transform_times, write_times, memory, timed_out


def get_backends(results: dict) -> List[str]:
    """Get list of backends present in results."""
    backends = results.get("metadata", {}).get("backends", [])
    if not backends:
        # Fall back to checking runs
        backends = list(set(r.get("backend", "python") for r in results.get("runs", [])))
    return backends if backends else ["python"]


def get_timeout_threshold(results: dict) -> float:
    """Get the timeout threshold used in the benchmark."""
    runs = results.get("runs", [])
    for run in runs:
        if run.get("timeout_seconds"):
            return run["timeout_seconds"]
    return 600  # Default timeout


def plot_total_runtime_by_param(results: dict, param_name: str, output_dir: str):
    """Create a line plot of total runtime vs parameter value."""
    values, total_times, _, _, _, _, _ = get_param_data(results, param_name)

    if not values:
        print(f"  No data for {param_name}, skipping...")
        return

    fig, ax = plt.subplots(figsize=(10, 6))

    ax.plot(values, total_times, 'o-', color=COLORS['total'], linewidth=2, markersize=8)

    # Mark baseline
    baseline_value = results.get("baseline", {}).get(param_name)
    if baseline_value in values:
        idx = values.index(baseline_value)
        ax.axvline(x=baseline_value, color=COLORS['baseline'], linestyle='--',
                   alpha=0.7, label=f'Baseline ({baseline_value})')

    ax.set_xlabel(PARAM_LABELS.get(param_name, param_name), fontsize=12)
    ax.set_ylabel('Total Runtime (seconds)', fontsize=12)
    ax.set_title(f'Total Runtime vs {PARAM_LABELS.get(param_name, param_name)}', fontsize=14)

    # Add min/max annotations
    min_idx = np.argmin(total_times)
    max_idx = np.argmax(total_times)
    ax.annotate(f'Min: {total_times[min_idx]:.1f}s',
                xy=(values[min_idx], total_times[min_idx]),
                xytext=(10, 10), textcoords='offset points',
                fontsize=10, color='green')

    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    output_file = os.path.join(output_dir, f'runtime_vs_{param_name}.png')
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"  Saved: {output_file}")


def plot_time_breakdown_by_param(results: dict, param_name: str, output_dir: str):
    """Create a stacked bar chart showing read/transform/write breakdown."""
    values, _, read_times, transform_times, write_times, _, _ = get_param_data(results, param_name)

    if not values:
        print(f"  No data for {param_name}, skipping...")
        return

    fig, ax = plt.subplots(figsize=(12, 6))

    x = np.arange(len(values))
    width = 0.6

    # Create stacked bars
    bars_read = ax.bar(x, read_times, width, label='Read', color=COLORS['read'])
    bars_transform = ax.bar(x, transform_times, width, bottom=read_times,
                           label='Transform', color=COLORS['transform'])
    bars_write = ax.bar(x, write_times, width,
                        bottom=[r + t for r, t in zip(read_times, transform_times)],
                        label='Write', color=COLORS['write'])

    ax.set_xlabel(PARAM_LABELS.get(param_name, param_name), fontsize=12)
    ax.set_ylabel('Time (seconds)', fontsize=12)
    ax.set_title(f'Time Breakdown by {PARAM_LABELS.get(param_name, param_name)}', fontsize=14)
    ax.set_xticks(x)
    ax.set_xticklabels([str(v) for v in values], rotation=45 if len(values) > 5 else 0)
    ax.legend()
    ax.grid(True, alpha=0.3, axis='y')

    plt.tight_layout()
    output_file = os.path.join(output_dir, f'breakdown_vs_{param_name}.png')
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"  Saved: {output_file}")


def plot_memory_by_param(results: dict, param_name: str, output_dir: str):
    """Create a bar chart of peak memory usage vs parameter value."""
    values, _, _, _, _, memory, _ = get_param_data(results, param_name)

    if not values:
        print(f"  No data for {param_name}, skipping...")
        return

    fig, ax = plt.subplots(figsize=(10, 6))

    x = np.arange(len(values))
    bars = ax.bar(x, memory, color=COLORS['memory'], alpha=0.8)

    # Mark baseline
    baseline_value = results.get("baseline", {}).get(param_name)
    if baseline_value in values:
        idx = values.index(baseline_value)
        bars[idx].set_edgecolor('black')
        bars[idx].set_linewidth(2)

    ax.set_xlabel(PARAM_LABELS.get(param_name, param_name), fontsize=12)
    ax.set_ylabel('Peak Memory (MB)', fontsize=12)
    ax.set_title(f'Peak Memory Usage vs {PARAM_LABELS.get(param_name, param_name)}', fontsize=14)
    ax.set_xticks(x)
    ax.set_xticklabels([str(v) for v in values], rotation=45 if len(values) > 5 else 0)
    ax.grid(True, alpha=0.3, axis='y')

    plt.tight_layout()
    output_file = os.path.join(output_dir, f'memory_vs_{param_name}.png')
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"  Saved: {output_file}")


def plot_all_params_comparison(results: dict, output_dir: str):
    """Create a summary plot comparing all parameters."""
    fig, axes = plt.subplots(2, 3, figsize=(16, 10))
    axes = axes.flatten()

    param_names = list(PARAM_LABELS.keys())
    timeout_threshold = get_timeout_threshold(results)

    for idx, param_name in enumerate(param_names):
        ax = axes[idx]
        values, total_times, _, _, _, _, timed_out = get_param_data(results, param_name, include_timeouts=True)

        if not values:
            ax.text(0.5, 0.5, 'No data', ha='center', va='center')
            ax.set_title(PARAM_LABELS.get(param_name, param_name))
            continue

        # Separate successful and timed-out points
        success_vals = [v for v, to in zip(values, timed_out) if not to]
        success_times = [t for t, to in zip(total_times, timed_out) if not to]
        timeout_vals = [v for v, to in zip(values, timed_out) if to]
        timeout_times = [t for t, to in zip(total_times, timed_out) if to]

        # Plot successful runs
        if success_vals:
            ax.plot(success_vals, success_times, 'o-', color=COLORS['total'], linewidth=2, markersize=6, label='Completed')
            # Highlight min among successful runs
            min_idx = np.argmin(success_times)
            ax.scatter([success_vals[min_idx]], [success_times[min_idx]], color='green', s=100, zorder=5)

        # Plot timed-out runs
        if timeout_vals:
            ax.scatter(timeout_vals, timeout_times, marker='x', color='red', s=100, linewidth=2, zorder=5, label='Timeout')

        # Add timeout threshold line
        ax.axhline(y=timeout_threshold, color='red', linestyle='--', alpha=0.5, linewidth=1, label=f'Timeout ({timeout_threshold}s)')

        ax.set_xlabel(PARAM_LABELS.get(param_name, param_name), fontsize=10)
        ax.set_ylabel('Runtime (s)', fontsize=10)
        ax.set_title(PARAM_LABELS.get(param_name, param_name), fontsize=11)
        ax.grid(True, alpha=0.3)
        ax.legend(fontsize=8, loc='best')

    # Hide unused subplot
    if len(param_names) < len(axes):
        axes[-1].axis('off')

    fig.suptitle('Total Runtime vs All Performance Parameters', fontsize=14, y=1.02)
    plt.tight_layout()

    output_file = os.path.join(output_dir, 'all_params_comparison.png')
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"  Saved: {output_file}")


def plot_speedup_comparison(results: dict, output_dir: str):
    """Create a bar chart comparing speedup vs baseline for best value of each param."""
    analysis = results.get("analysis", {}).get("by_parameter", {})

    if not analysis:
        print("  No analysis data available, skipping speedup comparison...")
        return

    fig, ax = plt.subplots(figsize=(12, 6))

    params = []
    speedups = []
    best_values = []

    for param_name, data in analysis.items():
        if data.get("values"):
            best_val = max(data["values"], key=lambda x: x.get("speedup_vs_baseline", 0))
            params.append(PARAM_LABELS.get(param_name, param_name))
            speedups.append(best_val.get("speedup_vs_baseline", 1.0))
            best_values.append(f"{best_val['value']}")

    if not params:
        print("  No speedup data available, skipping...")
        return

    x = np.arange(len(params))
    colors_list = [COLORS['total'] if s > 1 else COLORS['baseline'] for s in speedups]
    bars = ax.bar(x, speedups, color=colors_list, alpha=0.8)

    # Add value labels
    for i, (bar, val) in enumerate(zip(bars, best_values)):
        ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 0.02,
                f'{speedups[i]:.2f}x\n({val})', ha='center', va='bottom', fontsize=9)

    ax.axhline(y=1.0, color=COLORS['baseline'], linestyle='--', label='Baseline')
    ax.set_xlabel('Parameter', fontsize=12)
    ax.set_ylabel('Speedup vs Baseline', fontsize=12)
    ax.set_title('Best Speedup for Each Performance Parameter', fontsize=14)
    ax.set_xticks(x)
    ax.set_xticklabels(params, rotation=30, ha='right')
    ax.legend()
    ax.grid(True, alpha=0.3, axis='y')

    plt.tight_layout()
    output_file = os.path.join(output_dir, 'speedup_comparison.png')
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"  Saved: {output_file}")


def plot_time_breakdown_summary(results: dict, output_dir: str):
    """Create a pie chart showing time breakdown for baseline vs best config."""
    runs = results.get("runs", [])
    analysis = results.get("analysis", {})

    # Get baseline
    baseline_runs = [r for r in runs if r.get("param_varied") == "baseline" and r.get("success")]
    if not baseline_runs:
        print("  No baseline data, skipping breakdown summary...")
        return

    fig, axes = plt.subplots(1, 2, figsize=(14, 6))

    # Baseline breakdown
    br = baseline_runs[0]["timing_results"]
    baseline_times = [br["time_total_read"], br["time_total_transform"], br["time_total_write"]]
    labels = ['Read', 'Transform', 'Write']
    colors = [COLORS['read'], COLORS['transform'], COLORS['write']]

    axes[0].pie(baseline_times, labels=labels, colors=colors, autopct='%1.1f%%',
                startangle=90, explode=(0.02, 0.02, 0.02))
    axes[0].set_title(f'Baseline Configuration\nTotal: {br["time_total"]:.1f}s', fontsize=12)

    # Best configuration breakdown (if available)
    best_overall = analysis.get("best_overall")
    if best_overall:
        best_times = [best_overall["read_time"], best_overall["transform_time"], best_overall["write_time"]]
        axes[1].pie(best_times, labels=labels, colors=colors, autopct='%1.1f%%',
                    startangle=90, explode=(0.02, 0.02, 0.02))
        axes[1].set_title(f'Best Configuration\nTotal: {best_overall["total_time"]:.1f}s', fontsize=12)
    else:
        axes[1].text(0.5, 0.5, 'No best config data', ha='center', va='center')
        axes[1].set_title('Best Configuration')

    fig.suptitle('Time Breakdown: Baseline vs Best Configuration', fontsize=14)
    plt.tight_layout()

    output_file = os.path.join(output_dir, 'time_breakdown_summary.png')
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"  Saved: {output_file}")


def plot_runtime_vs_memory(results: dict, output_dir: str):
    """Create a scatter plot of runtime vs memory for all runs."""
    runs = results.get("runs", [])
    backends = get_backends(results)

    total_times = []
    memory_values = []
    labels = []
    colors = []

    for run in runs:
        if run.get("success") and "timing_results" in run:
            total_times.append(run["timing_results"]["time_total"])
            memory_values.append(run["timing_results"]["peak_memory_mb"])

            backend = run.get("backend", "python")
            param = run.get("param_varied", "unknown")
            if len(backends) > 1:
                # Color by backend when comparing
                colors.append(COLORS.get(backend, COLORS['total']))
                labels.append(backend)
            else:
                if param == "baseline":
                    labels.append("Baseline")
                    colors.append(COLORS['baseline'])
                else:
                    labels.append(param)
                    colors.append(COLORS['total'])

    if not total_times:
        print("  No data for runtime vs memory plot, skipping...")
        return

    fig, ax = plt.subplots(figsize=(10, 8))

    scatter = ax.scatter(total_times, memory_values, c=colors, s=100, alpha=0.7)

    # Add legend for backends if comparing
    if len(backends) > 1:
        handles = [mpatches.Patch(color=COLORS.get(b, COLORS['total']), label=b.capitalize())
                   for b in backends]
        ax.legend(handles=handles)

    ax.set_xlabel('Total Runtime (seconds)', fontsize=12)
    ax.set_ylabel('Peak Memory (MB)', fontsize=12)
    ax.set_title('Runtime vs Memory Trade-off', fontsize=14)
    ax.grid(True, alpha=0.3)

    # Add Pareto frontier note
    ax.text(0.02, 0.98, 'Lower-left is better', transform=ax.transAxes,
            fontsize=10, va='top', style='italic', color='gray')

    plt.tight_layout()
    output_file = os.path.join(output_dir, 'runtime_vs_memory.png')
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"  Saved: {output_file}")


def plot_python_vs_rust_comparison(results: dict, output_dir: str):
    """Create comparison plots for Python vs Rust performance."""
    backends = get_backends(results)
    if len(backends) < 2 or "python" not in backends or "rust" not in backends:
        print("  Skipping Python vs Rust comparison (need both backends)")
        return

    analysis = results.get("analysis", {})
    comparison = analysis.get("comparison", {})

    # Plot 1: Bar chart comparing overall best times
    fig, axes = plt.subplots(1, 2, figsize=(14, 6))

    # Left: Overall time comparison
    ax = axes[0]
    python_best = analysis.get("by_backend", {}).get("python", {}).get("best_overall", {})
    rust_best = analysis.get("by_backend", {}).get("rust", {}).get("best_overall", {})

    if python_best and rust_best:
        categories = ['Total', 'Read', 'Transform', 'Write']
        python_times = [
            python_best.get('total_time', 0),
            python_best.get('read_time', 0),
            python_best.get('transform_time', 0),
            python_best.get('write_time', 0),
        ]
        rust_times = [
            rust_best.get('total_time', 0),
            rust_best.get('read_time', 0),
            rust_best.get('transform_time', 0),
            rust_best.get('write_time', 0),
        ]

        x = np.arange(len(categories))
        width = 0.35

        bars1 = ax.bar(x - width/2, python_times, width, label='Python', color=COLORS['python'])
        bars2 = ax.bar(x + width/2, rust_times, width, label='Rust', color=COLORS['rust'])

        ax.set_xlabel('Phase', fontsize=12)
        ax.set_ylabel('Time (seconds)', fontsize=12)
        ax.set_title('Python vs Rust: Best Configuration Times', fontsize=14)
        ax.set_xticks(x)
        ax.set_xticklabels(categories)
        ax.legend()
        ax.grid(True, alpha=0.3, axis='y')

        # Add value labels
        for bars in [bars1, bars2]:
            for bar in bars:
                height = bar.get_height()
                ax.annotate(f'{height:.1f}s',
                           xy=(bar.get_x() + bar.get_width() / 2, height),
                           xytext=(0, 3), textcoords="offset points",
                           ha='center', va='bottom', fontsize=9)

    # Right: Speedup by phase
    ax = axes[1]
    if comparison:
        phases = ['Overall', 'Read', 'Transform', 'Write']
        speedups = [
            comparison.get('rust_speedup', 1.0),
            comparison.get('read_speedup', 1.0),
            comparison.get('transform_speedup', 1.0),
            comparison.get('write_speedup', 1.0),
        ]

        x = np.arange(len(phases))
        colors_list = [COLORS['rust'] if s > 1 else COLORS['python'] for s in speedups]
        bars = ax.bar(x, speedups, color=colors_list, alpha=0.8)

        ax.axhline(y=1.0, color=COLORS['baseline'], linestyle='--', linewidth=2)
        ax.set_xlabel('Phase', fontsize=12)
        ax.set_ylabel('Rust Speedup (x times faster)', fontsize=12)
        ax.set_title('Rust Speedup vs Python by Phase', fontsize=14)
        ax.set_xticks(x)
        ax.set_xticklabels(phases)
        ax.grid(True, alpha=0.3, axis='y')

        # Add value labels
        for bar, speedup in zip(bars, speedups):
            height = bar.get_height()
            label = f'{speedup:.2f}x'
            if speedup < 1:
                label = f'{1/speedup:.2f}x slower'
            ax.annotate(label,
                       xy=(bar.get_x() + bar.get_width() / 2, height),
                       xytext=(0, 3), textcoords="offset points",
                       ha='center', va='bottom', fontsize=10, fontweight='bold')

    plt.tight_layout()
    output_file = os.path.join(output_dir, 'python_vs_rust_comparison.png')
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"  Saved: {output_file}")


def plot_backend_comparison_by_param(results: dict, param_name: str, output_dir: str):
    """Create a comparison plot of Python vs Rust for a specific parameter."""
    backends = get_backends(results)
    if len(backends) < 2:
        return  # Skip if only one backend

    fig, ax = plt.subplots(figsize=(10, 6))

    for backend in backends:
        values, total_times, _, _, _, _, _ = get_param_data(results, param_name, backend=backend)
        if values:
            color = COLORS.get(backend, COLORS['total'])
            ax.plot(values, total_times, 'o-', color=color, linewidth=2,
                   markersize=8, label=backend.capitalize())

    if not ax.lines:
        plt.close()
        return

    ax.set_xlabel(PARAM_LABELS.get(param_name, param_name), fontsize=12)
    ax.set_ylabel('Total Runtime (seconds)', fontsize=12)
    ax.set_title(f'Python vs Rust: {PARAM_LABELS.get(param_name, param_name)}', fontsize=14)
    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    output_file = os.path.join(output_dir, f'backend_comparison_{param_name}.png')
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    plt.close()
    print(f"  Saved: {output_file}")


def generate_summary_report(results: dict, output_dir: str):
    """Generate a text summary report."""
    analysis = results.get("analysis", {})
    metadata = results.get("metadata", {})
    backends = get_backends(results)

    report_lines = [
        "=" * 70,
        "HDF5 CHUNKING BENCHMARK SUMMARY REPORT",
        "=" * 70,
        "",
        f"Generated: {metadata.get('timestamp', 'Unknown')}",
        f"Total runs: {metadata.get('total_runs', 0)}",
        f"Successful: {metadata.get('successful_runs', 0)}",
        f"Failed: {metadata.get('failed_runs', 0)}",
        f"Backends: {', '.join(backends)}",
        f"Total duration: {metadata.get('total_duration_seconds', 0) / 60:.1f} minutes",
        "",
        "-" * 70,
        "BASELINE CONFIGURATION",
        "-" * 70,
    ]

    baseline = results.get("baseline", {})
    for param, value in baseline.items():
        report_lines.append(f"  {PARAM_LABELS.get(param, param)}: {value}")

    # Per-backend results
    by_backend = analysis.get("by_backend", {})
    for backend in backends:
        backend_data = by_backend.get(backend, {})
        best = backend_data.get("best_overall", {})

        report_lines.extend([
            "",
            "-" * 70,
            f"BEST CONFIGURATION [{backend.upper()}]",
            "-" * 70,
        ])

        if best:
            report_lines.append(f"  Total Time: {best.get('total_time', 0):.2f}s")
            report_lines.append(f"  Read Time: {best.get('read_time', 0):.2f}s")
            report_lines.append(f"  Transform Time: {best.get('transform_time', 0):.2f}s")
            report_lines.append(f"  Write Time: {best.get('write_time', 0):.2f}s")
            report_lines.append(f"  Peak Memory: {best.get('peak_memory_mb', 0):.2f} MB")
            report_lines.append("")
            report_lines.append("  Configuration:")
            for param, value in best.get("config", {}).items():
                report_lines.append(f"    {PARAM_LABELS.get(param, param)}: {value}")
        else:
            report_lines.append("  No data available")

        # Parameter analysis for this backend
        param_analysis = backend_data.get("by_parameter", {})
        if param_analysis:
            report_lines.extend([
                "",
                f"  Parameter Analysis [{backend}]:",
            ])
            for param_name, param_data in param_analysis.items():
                if param_data.get('best_value') is not None:
                    report_lines.append(f"    {PARAM_LABELS.get(param_name, param_name)}:")
                    report_lines.append(f"      Best value: {param_data.get('best_value')}")
                    report_lines.append(f"      Best time: {param_data.get('best_time', 0):.2f}s")
                    improvement = param_data.get('improvement_vs_baseline', 0)
                    if improvement:
                        report_lines.append(f"      Improvement: {improvement:.1f}%")

    # Backend comparison if both are present
    comparison = analysis.get("comparison", {})
    if comparison:
        report_lines.extend([
            "",
            "-" * 70,
            "PYTHON vs RUST COMPARISON",
            "-" * 70,
            f"  Python best time: {comparison.get('python_best_time', 0):.2f}s",
            f"  Rust best time:   {comparison.get('rust_best_time', 0):.2f}s",
            f"  Rust speedup:     {comparison.get('rust_speedup', 1.0):.2f}x",
            "",
            "  Speedup by phase:",
            f"    Read:      {comparison.get('read_speedup', 1.0):.2f}x",
            f"    Transform: {comparison.get('transform_speedup', 1.0):.2f}x",
            f"    Write:     {comparison.get('write_speedup', 1.0):.2f}x",
        ])

    report_lines.extend([
        "",
        "-" * 70,
        "RECOMMENDATIONS",
        "-" * 70,
    ])

    recommendations = analysis.get("recommendations", [])
    if recommendations:
        for rec in recommendations:
            report_lines.append(f"  - {rec}")
    else:
        report_lines.append("  No specific recommendations (baseline may be optimal)")

    report_lines.extend([
        "",
        "=" * 70,
    ])

    report_text = "\n".join(report_lines)

    # Save report
    report_file = os.path.join(output_dir, "benchmark_report.txt")
    with open(report_file, 'w') as f:
        f.write(report_text)
    print(f"  Saved: {report_file}")

    # Also print to console
    print("\n" + report_text)


def plot_interactive_all_params(results: dict, output_dir: str):
    """
    Create an interactive HTML plot with all parameters showing all runs (including timeouts).
    Hover over points to see full configuration details.
    """
    try:
        import plotly.graph_objects as go
        from plotly.subplots import make_subplots
    except ImportError:
        print("  Warning: plotly not installed. Install with: pip install plotly")
        print("  Falling back to static matplotlib plot...")
        plot_interactive_all_params_matplotlib(results, output_dir)
        return

    param_names = list(PARAM_LABELS.keys())
    timeout_threshold = get_timeout_threshold(results)

    # Store all run data
    runs = results.get("runs", [])
    all_runs = [r for r in runs if r.get("success") or r.get("status") == "timeout"]

    # Create subplots (2 rows, 3 columns)
    fig = make_subplots(
        rows=2, cols=3,
        subplot_titles=[PARAM_LABELS.get(p, p) for p in param_names],
        vertical_spacing=0.12,
        horizontal_spacing=0.08
    )

    # Add traces for each parameter
    for idx, param_name in enumerate(param_names):
        row = idx // 3 + 1
        col = idx % 3 + 1

        # Get all runs and separate by status
        success_runs = []
        timeout_runs = []

        for run in all_runs:
            param_val = run["config"].get(param_name)
            if param_val is not None:
                is_timeout = run.get("status") == "timeout"
                time_val = run.get("elapsed_seconds", 0) if is_timeout else run.get("timing_results", {}).get("time_total", 0)

                # Build hover text
                config = run["config"]
                hover_text = f"<b>Runtime: {time_val:.1f}s</b>"
                if is_timeout:
                    hover_text += " (TIMEOUT)"
                hover_text += "<br><br><b>Configuration:</b><br>"
                for key, value in config.items():
                    label = PARAM_LABELS.get(key, key)
                    hover_text += f"  {label}: {value}<br>"

                # Add timing breakdown if available
                if not is_timeout and 'timing_results' in run:
                    tr = run['timing_results']
                    hover_text += f"<br><b>Breakdown:</b><br>"
                    hover_text += f"  Read: {tr.get('time_total_read', 0):.1f}s<br>"
                    hover_text += f"  Transform: {tr.get('time_total_transform', 0):.1f}s<br>"
                    hover_text += f"  Write: {tr.get('time_total_write', 0):.1f}s<br>"
                    hover_text += f"  Memory: {tr.get('peak_memory_mb', 0):.0f} MB"

                run_data = {
                    'param_value': param_val,
                    'time': time_val,
                    'hover_text': hover_text
                }

                if is_timeout:
                    timeout_runs.append(run_data)
                else:
                    success_runs.append(run_data)

        # Add successful runs scatter
        if success_runs:
            x_vals = [r['param_value'] for r in success_runs]
            y_vals = [r['time'] for r in success_runs]
            hover_texts = [r['hover_text'] for r in success_runs]

            fig.add_trace(
                go.Scatter(
                    x=x_vals,
                    y=y_vals,
                    mode='markers',
                    name='Completed',
                    marker=dict(size=8, color='#9b59b6', opacity=0.7),
                    hovertext=hover_texts,
                    hoverinfo='text',
                    showlegend=(idx == 0)  # Only show legend for first subplot
                ),
                row=row, col=col
            )

        # Add timed-out runs scatter
        if timeout_runs:
            x_vals = [r['param_value'] for r in timeout_runs]
            y_vals = [r['time'] for r in timeout_runs]
            hover_texts = [r['hover_text'] for r in timeout_runs]

            fig.add_trace(
                go.Scatter(
                    x=x_vals,
                    y=y_vals,
                    mode='markers',
                    name='Timeout',
                    marker=dict(size=12, color='red', symbol='x', line=dict(width=2)),
                    hovertext=hover_texts,
                    hoverinfo='text',
                    showlegend=(idx == 0)
                ),
                row=row, col=col
            )

        # Add timeout threshold line
        if success_runs or timeout_runs:
            all_x = [r['param_value'] for r in success_runs + timeout_runs]
            x_range = [min(all_x), max(all_x)]

            fig.add_trace(
                go.Scatter(
                    x=x_range,
                    y=[timeout_threshold, timeout_threshold],
                    mode='lines',
                    name=f'Timeout ({timeout_threshold}s)',
                    line=dict(color='red', dash='dash', width=2),
                    hoverinfo='skip',
                    showlegend=(idx == 0)
                ),
                row=row, col=col
            )

        # Update axes labels
        fig.update_xaxes(title_text=PARAM_LABELS.get(param_name, param_name), row=row, col=col)
        fig.update_yaxes(title_text='Runtime (s)', row=row, col=col)

    # Update layout
    fig.update_layout(
        title_text='<b>Interactive Parameter Explorer</b><br><sub>Hover over points for configuration details</sub>',
        title_x=0.5,
        height=900,
        width=1600,
        showlegend=True,
        legend=dict(
            orientation="h",
            yanchor="bottom",
            y=1.02,
            xanchor="right",
            x=1
        ),
        hovermode='closest'
    )

    # Save as HTML
    output_file = os.path.join(output_dir, 'interactive_all_params.html')
    fig.write_html(output_file)
    print(f"  Saved: {output_file}")
    print(f"  Open in browser to interact: file://{os.path.abspath(output_file)}")


def plot_interactive_all_params_matplotlib(results: dict, output_dir: str):
    """
    Fallback: Create a static matplotlib plot when plotly is not available.
    """
    param_names = list(PARAM_LABELS.keys())
    timeout_threshold = get_timeout_threshold(results)

    fig, axes = plt.subplots(2, 3, figsize=(18, 12))
    axes = axes.flatten()

    # Store all run data
    runs = results.get("runs", [])
    all_runs = [r for r in runs if r.get("success") or r.get("status") == "timeout"]

    for idx, param_name in enumerate(param_names):
        ax = axes[idx]

        # Get all runs and separate by status
        param_runs = []
        for run in all_runs:
            param_val = run["config"].get(param_name)
            if param_val is not None:
                is_timeout = run.get("status") == "timeout"
                time_val = run.get("elapsed_seconds", 0) if is_timeout else run.get("timing_results", {}).get("time_total", 0)
                param_runs.append({
                    'param_value': param_val,
                    'time': time_val,
                    'is_timeout': is_timeout
                })

        if not param_runs:
            ax.text(0.5, 0.5, 'No data', ha='center', va='center')
            ax.set_title(PARAM_LABELS.get(param_name, param_name))
            continue

        # Separate successful and timed-out
        success_runs = [r for r in param_runs if not r['is_timeout']]
        timeout_runs = [r for r in param_runs if r['is_timeout']]

        # Plot successful runs
        if success_runs:
            x_vals = [r['param_value'] for r in success_runs]
            y_vals = [r['time'] for r in success_runs]
            ax.scatter(x_vals, y_vals, c=COLORS['total'], s=50, alpha=0.7,
                      marker='o', label='Completed')

        # Plot timed-out runs
        if timeout_runs:
            x_vals = [r['param_value'] for r in timeout_runs]
            y_vals = [r['time'] for r in timeout_runs]
            ax.scatter(x_vals, y_vals, c='red', s=100, alpha=0.7,
                      marker='x', linewidth=2, label='Timeout')

        # Add timeout threshold line
        ax.axhline(y=timeout_threshold, color='red', linestyle='--', alpha=0.5,
                  linewidth=1.5, label=f'Timeout ({timeout_threshold}s)')

        ax.set_xlabel(PARAM_LABELS.get(param_name, param_name), fontsize=11)
        ax.set_ylabel('Runtime (s)', fontsize=11)
        ax.set_title(PARAM_LABELS.get(param_name, param_name), fontsize=12, fontweight='bold')
        ax.grid(True, alpha=0.3)
        ax.legend(fontsize=9, loc='best')

    # Hide unused subplot
    if len(param_names) < len(axes):
        axes[-1].axis('off')

    fig.suptitle('All Parameters (Static - install plotly for interactivity)',
                fontsize=15, fontweight='bold', y=0.995)
    plt.tight_layout()

    output_file = os.path.join(output_dir, 'interactive_all_params.png')
    plt.savefig(output_file, dpi=150, bbox_inches='tight')
    print(f"  Saved: {output_file}")
    plt.close()


def main():
    parser = argparse.ArgumentParser(
        description="Generate plots from HDF5 chunking benchmark results"
    )
    parser.add_argument(
        "--results", "-r",
        type=str,
        required=True,
        help="Path to benchmark_results.json file"
    )
    parser.add_argument(
        "--output-dir", "-o",
        type=str,
        default="plots",
        help="Output directory for plots (default: plots)"
    )

    args = parser.parse_args()

    # Validate input
    if not os.path.exists(args.results):
        print(f"ERROR: Results file not found: {args.results}")
        return 1

    # Create output directory
    os.makedirs(args.output_dir, exist_ok=True)

    # Load results
    print(f"Loading results from: {args.results}")
    results = load_results(args.results)

    print(f"\nGenerating plots in: {args.output_dir}")
    print("-" * 50)

    # Check for multiple backends
    backends = get_backends(results)
    print(f"Backends found: {', '.join(backends)}")

    # Generate per-parameter plots
    param_names = list(PARAM_LABELS.keys())
    for param_name in param_names:
        print(f"\nParameter: {param_name}")
        plot_total_runtime_by_param(results, param_name, args.output_dir)
        plot_time_breakdown_by_param(results, param_name, args.output_dir)
        plot_memory_by_param(results, param_name, args.output_dir)
        # Backend comparison for this parameter (if multiple backends)
        if len(backends) > 1:
            plot_backend_comparison_by_param(results, param_name, args.output_dir)

    # Generate summary plots
    print("\nSummary plots:")
    plot_all_params_comparison(results, args.output_dir)
    plot_speedup_comparison(results, args.output_dir)
    plot_time_breakdown_summary(results, args.output_dir)
    plot_runtime_vs_memory(results, args.output_dir)

    # Generate interactive plot with all runs
    print("\nInteractive plot:")
    plot_interactive_all_params(results, args.output_dir)

    # Generate Python vs Rust comparison plots (if both backends present)
    if len(backends) > 1:
        print("\nBackend comparison plots:")
        plot_python_vs_rust_comparison(results, args.output_dir)

    # Generate text report
    print("\nGenerating summary report:")
    generate_summary_report(results, args.output_dir)

    print(f"\nAll plots saved to: {args.output_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
