#!/usr/bin/env python3
"""
Helper script to write benchmark result JSON files for timeout/error cases.

This is used by the Merlin workflow when a benchmark times out or fails,
to create a result JSON that postprocessing can handle robustly.

Usage:
    python write_result_json.py --output result.json --status timeout \
        --timeout-seconds 600 --elapsed-seconds 600 \
        --cache-mb 256 --node-chunk 25000 --elem-chunk 25000 \
        --time-chunk 100 --preemption 0.75 \
        --input-file /path/to/input.exo \
        --error "Benchmark exceeded time limit"
"""

import argparse
import json
import sys


def main():
    parser = argparse.ArgumentParser(
        description="Write benchmark result JSON for timeout/error cases"
    )
    parser.add_argument(
        "--output", "-o",
        type=str,
        required=True,
        help="Output JSON file path"
    )
    parser.add_argument(
        "--status",
        type=str,
        required=True,
        choices=["timeout", "error"],
        help="Result status (timeout or error)"
    )
    parser.add_argument(
        "--timeout-seconds",
        type=int,
        default=None,
        help="Timeout limit in seconds (for timeout status)"
    )
    parser.add_argument(
        "--exit-code",
        type=int,
        default=None,
        help="Exit code (for error status)"
    )
    parser.add_argument(
        "--elapsed-seconds",
        type=int,
        required=True,
        help="Elapsed time in seconds before termination"
    )
    parser.add_argument(
        "--cache-mb",
        type=int,
        required=True,
        help="HDF5 cache size in MB"
    )
    parser.add_argument(
        "--node-chunk",
        type=int,
        required=True,
        help="Node chunk size"
    )
    parser.add_argument(
        "--elem-chunk",
        type=int,
        required=True,
        help="Element chunk size"
    )
    parser.add_argument(
        "--time-chunk",
        type=int,
        required=True,
        help="Time chunk size"
    )
    parser.add_argument(
        "--preemption",
        type=float,
        required=True,
        help="HDF5 cache preemption value"
    )
    parser.add_argument(
        "--input-file",
        type=str,
        required=True,
        help="Input mesh file path"
    )
    parser.add_argument(
        "--error",
        type=str,
        required=True,
        help="Error message describing the failure"
    )

    args = parser.parse_args()

    # Build result dictionary
    result = {
        "status": args.status,
        "elapsed_seconds": args.elapsed_seconds,
        "perf_params": {
            "cache_mb": args.cache_mb,
            "node_chunk_size": args.node_chunk,
            "element_chunk_size": args.elem_chunk,
            "time_chunk_size": args.time_chunk,
            "preemption": args.preemption
        },
        "input_file": args.input_file,
        "error": args.error
    }

    # Add status-specific fields
    if args.status == "timeout" and args.timeout_seconds is not None:
        result["timeout_seconds"] = args.timeout_seconds
    if args.status == "error" and args.exit_code is not None:
        result["exit_code"] = args.exit_code

    # Write JSON file
    with open(args.output, 'w') as f:
        json.dump(result, f, indent=2)

    print(f"Result written to: {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
