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


def get_param_data(results: dict, param_name: str, backend: Optional[str] = None) -> tuple:
    """
    Extract data for a specific parameter.

    Args:
        results: Benchmark results dict
        param_name: Parameter name to extract
        backend: Filter by backend ("python", "rust", or None for all)

    Returns (values, total_times, read_times, transform_times, write_times, memory)
    """
    baseline = results.get("baseline", {})
    runs = results.get("runs", [])

    # Filter by backend if specified
    if backend:
        runs = [r for r in runs if r.get("backend", "python") == backend]

    values = []
    total_times = []
    read_times = []
    transform_times = []
    write_times = []
    memory = []

    # Get baseline data
    baseline_runs = [r for r in runs if r.get("param_varied") == "baseline" and r.get("success")]
    if baseline_runs:
        br = baseline_runs[0]
        baseline_value = baseline.get(param_name)
        if baseline_value is not None and "timing_results" in br:
            values.append(baseline_value)
            total_times.append(br["timing_results"]["time_total"])
            read_times.append(br["timing_results"]["time_total_read"])
            transform_times.append(br["timing_results"]["time_total_transform"])
            write_times.append(br["timing_results"]["time_total_write"])
            memory.append(br["timing_results"]["peak_memory_mb"])

    # Get varied parameter data
    param_runs = [r for r in runs if r.get("param_varied") == param_name and r.get("success")]
    for run in sorted(param_runs, key=lambda x: x["config"].get(param_name, 0)):
        if "timing_results" in run:
            values.append(run["config"][param_name])
            total_times.append(run["timing_results"]["time_total"])
            read_times.append(run["timing_results"]["time_total_read"])
            transform_times.append(run["timing_results"]["time_total_transform"])
            write_times.append(run["timing_results"]["time_total_write"])
            memory.append(run["timing_results"]["peak_memory_mb"])

    return values, total_times, read_times, transform_times, write_times, memory


def get_backends(results: dict) -> List[str]:
    """Get list of backends present in results."""
    backends = results.get("metadata", {}).get("backends", [])
    if not backends:
        # Fall back to checking runs
        backends = list(set(r.get("backend", "python") for r in results.get("runs", [])))
    return backends if backends else ["python"]


def plot_total_runtime_by_param(results: dict, param_name: str, output_dir: str):
    """Create a line plot of total runtime vs parameter value."""
    values, total_times, _, _, _, _ = get_param_data(results, param_name)

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
    values, _, read_times, transform_times, write_times, _ = get_param_data(results, param_name)

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
    values, _, _, _, _, memory = get_param_data(results, param_name)

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

    for idx, param_name in enumerate(param_names):
        ax = axes[idx]
        values, total_times, _, _, _, _ = get_param_data(results, param_name)

        if not values:
            ax.text(0.5, 0.5, 'No data', ha='center', va='center')
            ax.set_title(PARAM_LABELS.get(param_name, param_name))
            continue

        ax.plot(values, total_times, 'o-', color=COLORS['total'], linewidth=2, markersize=6)

        # Highlight min
        min_idx = np.argmin(total_times)
        ax.scatter([values[min_idx]], [total_times[min_idx]], color='green', s=100, zorder=5)

        ax.set_xlabel(PARAM_LABELS.get(param_name, param_name), fontsize=10)
        ax.set_ylabel('Runtime (s)', fontsize=10)
        ax.set_title(PARAM_LABELS.get(param_name, param_name), fontsize=11)
        ax.grid(True, alpha=0.3)

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
        values, total_times, _, _, _, _ = get_param_data(results, param_name, backend=backend)
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
