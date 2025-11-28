#!/usr/bin/env python3
"""
Parameter Generator for HDF5 Chunking Benchmark Suite

This script generates configurations for the Merlin workflow using a
"one-at-a-time" experimental design:
1. Run baseline with default settings
2. Vary one parameter at a time while holding others at defaults

This design allows understanding individual parameter effects while
keeping the number of configurations manageable.

Standalone CLI Usage:
    python pgen_configs.py --outfile samples.csv
    python pgen_configs.py --outfile samples.csv --quick  # Reduced parameter set
    python pgen_configs.py --outfile samples.csv --full   # Full factorial (many configs!)

Merlin pgen Usage:
    merlin run workflow.yaml --pgen pgen_configs.py
    merlin run workflow.yaml --pgen pgen_configs.py --pargs "quick:true"
    merlin run workflow.yaml --pgen pgen_configs.py --pargs "full:true"
    merlin run workflow.yaml --pgen pgen_configs.py --pargs "quick:true" --pargs "full:true"
"""

import argparse
import csv
import sys
from typing import List, Dict, Tuple


# Default baseline configuration (tuned for RZHound with 256GB RAM)
BASELINE_CONFIG = {
    "CACHE_MB": 64,
    "NODE_CHUNK_SIZE": 100_000,
    "ELEMENT_CHUNK_SIZE": 50_000,
    "TIME_CHUNK_SIZE": 10,
    "PREEMPTION": 1.0,
}

# Full parameter sweep ranges for RZHound (256GB RAM per node)
# Cache-aware ranges targeting L2 cache (2 MiB) and L3 cache (105 MiB)
# At 10 timesteps: 50k elements = ~2 MiB (L2-sized chunk)
# Trimmed to focus on cache-friendly sizes: 432 configs (full factorial)
PARAM_RANGES = {
    "CACHE_MB": [1, 32, 64, 96, 128, 160],                           # Stay under L3 (105 MiB)
    "NODE_CHUNK_SIZE": [10_000, 50_000, 75_000, 100_000, 200_000],           # 2-4 MiB at 10 timesteps
    "ELEMENT_CHUNK_SIZE": [25_000, 37_500, 50_000, 75_000, 100_000], # 1-3 MiB at 10 timesteps
    "TIME_CHUNK_SIZE": [1, 5, 10, 20],                        # Avoid extremes
    "PREEMPTION": [0.0, 0.5, 1.0],                          # Test extremes + middle
}

# Quick test ranges (reduced for faster testing)
QUICK_PARAM_RANGES = {
    "CACHE_MB": [64, 1024],
    "NODE_CHUNK_SIZE": [10000, 50000],
    "ELEMENT_CHUNK_SIZE": [10000, 50000],
    "TIME_CHUNK_SIZE": [10, 100],
    "PREEMPTION": [0.0, 0.75],
}


def generate_one_at_a_time(param_ranges: Dict, baseline: Dict) -> List[Dict]:
    """
    Generate configurations using one-at-a-time design.

    This varies each parameter individually while keeping others at baseline values.
    Returns list of configuration dictionaries.
    """
    configs = []

    # First, add baseline configuration
    configs.append(baseline.copy())

    # Then vary each parameter one at a time
    for param_name, values in param_ranges.items():
        for value in values:
            # Skip if this is the baseline value (already included)
            if value == baseline[param_name]:
                continue

            # Create config with this parameter varied
            config = baseline.copy()
            config[param_name] = value
            configs.append(config)

    return configs


def generate_full_factorial(param_ranges: Dict, baseline: Dict = None) -> List[Dict]:
    """
    Generate all combinations of parameters (full factorial design).

    If baseline is provided, ensures all one-at-a-time variations are included
    by augmenting the parameter ranges with baseline values before generating
    the factorial design.

    WARNING: This can generate many configurations!
    For the full parameter ranges: 6 * 5 * 4 * 5 * 5 = 3000 configurations
    """
    import itertools

    # Augment parameter ranges to include baseline values
    # This ensures one-at-a-time slices will be in the factorial design
    augmented_ranges = {}
    if baseline:
        for param_name, values in param_ranges.items():
            baseline_val = baseline.get(param_name)
            # Add baseline value to range if not already present
            if baseline_val is not None and baseline_val not in values:
                augmented_ranges[param_name] = sorted(values + [baseline_val])
            else:
                augmented_ranges[param_name] = values
    else:
        augmented_ranges = param_ranges

    # Get all parameter names and their values
    param_names = list(augmented_ranges.keys())
    param_values = [augmented_ranges[name] for name in param_names]

    configs = []
    for combo in itertools.product(*param_values):
        config = dict(zip(param_names, combo))
        configs.append(config)

    return configs


def get_custom_generator(env, **kwargs):
    """
    Merlin/Maestro pgen entry point.

    This function is called by Merlin when using --pgen flag.
    Arguments are passed via --pargs "key:value" on the command line.

    Args:
        env: StudyEnvironment object (can access spec's env block via env.find())
        **kwargs: Dictionary of arguments passed via --pargs

    Returns:
        ParameterGenerator: Configured parameter generator for the study

    Example:
        merlin run workflow.yaml --pgen pgen_configs.py --pargs "quick:true"
    """
    from maestrowf.datastructures.core import ParameterGenerator

    # Parse pargs (values come in as strings)
    quick = kwargs.get('quick', 'false').lower() == 'true'
    full = kwargs.get('full', 'false').lower() == 'true'

    # Select parameter ranges
    if quick:
        param_ranges = QUICK_PARAM_RANGES
        print("pgen: Using QUICK parameter ranges (reduced set)")
    else:
        param_ranges = PARAM_RANGES
        print("pgen: Using FULL parameter ranges")

    # Generate configurations
    if full:
        print("pgen: Generating FULL FACTORIAL design (all combinations)")
        print("pgen: Including baseline values to ensure one-at-a-time slices are present")
        configs = generate_full_factorial(param_ranges, BASELINE_CONFIG)
    else:
        print("pgen: Generating ONE-AT-A-TIME design (vary one parameter at a time)")
        configs = generate_one_at_a_time(param_ranges, BASELINE_CONFIG)

    print(f"pgen: Baseline configuration: {BASELINE_CONFIG}")
    print(f"pgen: Total configurations: {len(configs)}")

    # Build ParameterGenerator
    # Each parameter gets a list of values, one per configuration
    pgen = ParameterGenerator()

    # Short labels for directory naming (keeps paths reasonable)
    label_map = {
        "CACHE_MB": "c",
        "NODE_CHUNK_SIZE": "n",
        "ELEMENT_CHUNK_SIZE": "e",
        "TIME_CHUNK_SIZE": "t",
        "PREEMPTION": "p",
    }

    # Get parameter names from the first config
    param_names = list(configs[0].keys())

    # For each parameter, collect all values across configurations
    for param_name in param_names:
        values = [str(config[param_name]) for config in configs]
        # label with %% placeholder - gets replaced with actual value
        short_label = label_map.get(param_name, param_name[:3].lower())
        pgen.add_parameter(param_name, values, label=f"{short_label}.%%")

    return pgen


def write_csv(configs: List[Dict], outfile: str):
    """Write configurations to CSV file for Merlin.

    Note: Does NOT write header row because Merlin's column_labels
    defines the columns. Writing a header would cause Merlin to
    treat the header row as sample 0, resulting in literal variable
    names (e.g., "CACHE_MB") being passed instead of values.
    """
    if not configs:
        print("ERROR: No configurations to write", file=sys.stderr)
        return

    # Get column names from first config
    fieldnames = list(configs[0].keys())

    with open(outfile, 'w', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        # DO NOT write header - Merlin uses column_labels instead
        # writer.writeheader()  # Commented out to prevent header being treated as sample 0
        writer.writerows(configs)

    print(f"Generated {len(configs)} configurations")
    print(f"Written to: {outfile}")


def main():
    import os

    parser = argparse.ArgumentParser(
        description="Generate parameter configurations for HDF5 chunking benchmark"
    )
    parser.add_argument(
        "--outfile", "-o",
        type=str,
        default="samples.csv",
        help="Output CSV file path (default: samples.csv)"
    )
    parser.add_argument(
        "--quick",
        action="store_true",
        help="Use reduced parameter ranges for quick testing"
    )
    parser.add_argument(
        "--full",
        action="store_true",
        help="Generate full factorial design (WARNING: many configurations!)"
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="Print configurations to stdout instead of writing file"
    )

    args = parser.parse_args()

    # Check environment variables (set by run.sh) if CLI flags not provided
    quick = args.quick or os.environ.get('PGEN_QUICK', '').lower() == 'true'
    full = args.full or os.environ.get('PGEN_FULL', '').lower() == 'true'

    # Select parameter ranges
    if quick:
        param_ranges = QUICK_PARAM_RANGES
        print("Using QUICK parameter ranges (reduced set)")
    else:
        param_ranges = PARAM_RANGES
        print("Using FULL parameter ranges")

    # Generate configurations
    if full:
        print("Generating FULL FACTORIAL design (all combinations)")
        print("Including baseline values to ensure one-at-a-time slices are present")
        configs = generate_full_factorial(param_ranges, BASELINE_CONFIG)
    else:
        print("Generating ONE-AT-A-TIME design (vary one parameter at a time)")
        configs = generate_one_at_a_time(param_ranges, BASELINE_CONFIG)

    # Print summary
    print(f"\nBaseline configuration: {BASELINE_CONFIG}")
    print(f"Total configurations: {len(configs)}")

    if args.list:
        print("\nConfigurations:")
        for i, config in enumerate(configs):
            print(f"  {i}: {config}")
    else:
        write_csv(configs, args.outfile)

    return 0


if __name__ == "__main__":
    sys.exit(main())
