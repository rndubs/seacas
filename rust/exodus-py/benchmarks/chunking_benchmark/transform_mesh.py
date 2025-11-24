#!/usr/bin/env python3
"""
Transform an Exodus mesh file with performance configuration options.

This script performs the following transformations:
1. Scale the mesh by a factor of 2.0
2. Rotate the mesh 90 degrees about the Z axis
3. Scale the "pressure" field variable by 10.0

The script measures and reports timing for:
- Reading the mesh
- Transforming coordinates
- Transforming field variables
- Writing the output mesh

Performance options are specified via CLI arguments and control HDF5 chunking behavior.

Usage:
    python transform_mesh.py --input input.exo --output output.exo \\
        --cache-mb 256 --node-chunk-size 25000 --element-chunk-size 25000 \\
        --time-chunk-size 100 --preemption 0.75
"""

import argparse
import json
import math
import os
import sys
import time
import traceback
import tracemalloc
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Optional

import numpy as np
from scipy.spatial.transform import Rotation
from tqdm import tqdm

# Add the python directory to the path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "python"))

from exodus import (
    Block,
    CreateMode,
    CreateOptions,
    EntityType,
    ExodusReader,
    ExodusWriter,
    InitParams,
    PerformanceConfig,
)


@dataclass
class PerformanceParams:
    """Performance configuration parameters."""
    cache_mb: int
    node_chunk_size: int
    element_chunk_size: int
    time_chunk_size: int
    preemption: float

    def to_dict(self) -> dict:
        return asdict(self)

    def summary(self) -> str:
        return (
            f"cache={self.cache_mb}MB, "
            f"node_chunk={self.node_chunk_size}, "
            f"elem_chunk={self.element_chunk_size}, "
            f"time_chunk={self.time_chunk_size}, "
            f"preemption={self.preemption}"
        )


@dataclass
class TimingResult:
    """Timing results for a single benchmark run."""
    # Performance configuration
    perf_params: PerformanceParams

    # Input/output info
    input_file: str
    output_file: str
    input_size_gb: float
    output_size_gb: float

    # Mesh info
    num_nodes: int
    num_elements: int
    num_timesteps: int
    num_variables: int

    # Timing breakdown (seconds)
    time_read_init: float
    time_read_coords: float
    time_read_connectivity: float
    time_read_variables: float
    time_transform_coords: float
    time_transform_variables: float
    time_write_init: float
    time_write_coords: float
    time_write_connectivity: float
    time_write_variables: float
    time_close: float

    # Memory info
    peak_memory_mb: float

    @property
    def time_total_read(self) -> float:
        return (
            self.time_read_init +
            self.time_read_coords +
            self.time_read_connectivity +
            self.time_read_variables
        )

    @property
    def time_total_transform(self) -> float:
        return self.time_transform_coords + self.time_transform_variables

    @property
    def time_total_write(self) -> float:
        return (
            self.time_write_init +
            self.time_write_coords +
            self.time_write_connectivity +
            self.time_write_variables +
            self.time_close
        )

    @property
    def time_total(self) -> float:
        return self.time_total_read + self.time_total_transform + self.time_total_write

    def to_dict(self) -> dict:
        d = {
            "perf_params": self.perf_params.to_dict(),
            "input_file": self.input_file,
            "output_file": self.output_file,
            "input_size_gb": self.input_size_gb,
            "output_size_gb": self.output_size_gb,
            "num_nodes": self.num_nodes,
            "num_elements": self.num_elements,
            "num_timesteps": self.num_timesteps,
            "num_variables": self.num_variables,
            "time_read_init": self.time_read_init,
            "time_read_coords": self.time_read_coords,
            "time_read_connectivity": self.time_read_connectivity,
            "time_read_variables": self.time_read_variables,
            "time_transform_coords": self.time_transform_coords,
            "time_transform_variables": self.time_transform_variables,
            "time_write_init": self.time_write_init,
            "time_write_coords": self.time_write_coords,
            "time_write_connectivity": self.time_write_connectivity,
            "time_write_variables": self.time_write_variables,
            "time_close": self.time_close,
            "time_total_read": self.time_total_read,
            "time_total_transform": self.time_total_transform,
            "time_total_write": self.time_total_write,
            "time_total": self.time_total,
            "peak_memory_mb": self.peak_memory_mb,
        }
        return d


def transform_coordinates(
    coords_x: np.ndarray,
    coords_y: np.ndarray,
    coords_z: np.ndarray,
    scale_factor: float = 2.0,
    rotation_degrees: float = 90.0,
) -> tuple:
    """
    Transform mesh coordinates:
    1. Scale by scale_factor
    2. Rotate about Z axis by rotation_degrees
    """
    # Stack coordinates for transformation
    coords = np.column_stack([coords_x, coords_y, coords_z])

    # Scale
    coords = coords * scale_factor

    # Rotate 90 degrees about Z axis
    rotation = Rotation.from_euler('z', rotation_degrees, degrees=True)
    coords = rotation.apply(coords)

    # Return contiguous arrays (slicing creates non-contiguous views)
    return (
        np.ascontiguousarray(coords[:, 0]),
        np.ascontiguousarray(coords[:, 1]),
        np.ascontiguousarray(coords[:, 2]),
    )


def transform_pressure(data: np.ndarray, scale_factor: float = 10.0) -> np.ndarray:
    """Scale pressure values by the given factor."""
    return data * scale_factor


def run_transform(
    input_path: str,
    output_path: str,
    perf_params: PerformanceParams,
    verbose: bool = True,
) -> TimingResult:
    """
    Run the full read -> transform -> write pipeline with timing.
    """
    # Start memory tracking
    tracemalloc.start()

    timing = {}

    if verbose:
        print(f"\n{'=' * 70}")
        print(f"Transform Mesh Benchmark")
        print(f"{'=' * 70}")
        print(f"Input:  {input_path}")
        print(f"Output: {output_path}")
        print(f"Config: {perf_params.summary()}")
        print(f"{'=' * 70}")

    # Set HDF5 environment variables BEFORE opening any files
    # This must be done before the HDF5 library is initialized
    os.environ['HDF5_CHUNK_CACHE_NBYTES'] = str(perf_params.cache_mb * 1024 * 1024)
    os.environ['HDF5_CHUNK_CACHE_W0'] = str(perf_params.preemption)

    input_size_gb = os.path.getsize(input_path) / (1024 ** 3)

    # =========================================================================
    # PHASE 1: READ
    # =========================================================================
    if verbose:
        print(f"\n[PHASE 1] Reading input file ({input_size_gb:.2f} GB)...")

    # Read initialization
    start = time.perf_counter()
    reader = ExodusReader.open(input_path)
    params = reader.init_params()
    timing["read_init"] = time.perf_counter() - start

    num_nodes = params.num_nodes
    num_elements = params.num_elems
    num_timesteps = reader.num_time_steps()
    variable_names = reader.variable_names(EntityType.Nodal)
    num_variables = len(variable_names)
    block_ids = reader.get_block_ids()

    if verbose:
        print(f"  Nodes: {num_nodes:,}")
        print(f"  Elements: {num_elements:,}")
        print(f"  Timesteps: {num_timesteps:,}")
        print(f"  Variables: {variable_names}")

    # Read coordinates
    if verbose:
        print(f"  Reading coordinates...")
    start = time.perf_counter()
    coords = reader.get_coords()
    coords_x = coords[:, 0].copy()
    coords_y = coords[:, 1].copy()
    coords_z = coords[:, 2].copy()
    timing["read_coords"] = time.perf_counter() - start

    # Read connectivity
    if verbose:
        print(f"  Reading connectivity...")
    start = time.perf_counter()
    blocks_info = []
    connectivity_data = {}
    for block_id in block_ids:
        block = reader.get_block(block_id)
        conn = reader.get_connectivity(block_id)
        blocks_info.append(block)
        connectivity_data[block_id] = conn
    timing["read_connectivity"] = time.perf_counter() - start

    # Read all variable data
    if verbose:
        print(f"  Reading {num_timesteps:,} timesteps x {num_variables} variables...")
    start = time.perf_counter()

    # Store time values
    time_values = reader.times()

    # Read all variable data into memory
    # Structure: {var_idx: {step: data_array}}
    all_var_data = {}
    total_reads = num_variables * num_timesteps
    pbar = tqdm(total=total_reads, desc="  Reading variables", unit="var",
                ncols=80, mininterval=1.0, disable=not verbose)
    for var_idx in range(num_variables):
        all_var_data[var_idx] = {}
        for step in range(num_timesteps):
            data = reader.var(step, EntityType.Nodal, 0, var_idx)
            all_var_data[var_idx][step] = data.copy()
            pbar.update(1)
    pbar.close()

    timing["read_variables"] = time.perf_counter() - start

    reader.close()

    if verbose:
        read_total = sum([timing["read_init"], timing["read_coords"],
                        timing["read_connectivity"], timing["read_variables"]])
        print(f"  Read complete: {read_total:.2f}s")

    # =========================================================================
    # PHASE 2: TRANSFORM
    # =========================================================================
    if verbose:
        print(f"\n[PHASE 2] Transforming mesh...")

    # Transform coordinates
    if verbose:
        print(f"  Scaling mesh by 2.0 and rotating 90 degrees about Z...")
    start = time.perf_counter()
    new_coords_x, new_coords_y, new_coords_z = transform_coordinates(
        coords_x, coords_y, coords_z,
        scale_factor=2.0,
        rotation_degrees=90.0
    )
    timing["transform_coords"] = time.perf_counter() - start

    # Transform pressure variable
    pressure_idx = None
    for idx, name in enumerate(variable_names):
        if name.lower() == "pressure":
            pressure_idx = idx
            break

    if verbose:
        print(f"  Scaling 'pressure' field by 10.0...")
    start = time.perf_counter()
    if pressure_idx is not None:
        for step in tqdm(range(num_timesteps), desc="  Transforming pressure", unit="step",
                         ncols=80, mininterval=1.0, disable=not verbose):
            all_var_data[pressure_idx][step] = transform_pressure(
                all_var_data[pressure_idx][step], scale_factor=10.0
            )
    timing["transform_variables"] = time.perf_counter() - start

    if verbose:
        transform_total = timing["transform_coords"] + timing["transform_variables"]
        print(f"  Transform complete: {transform_total:.2f}s")

    # =========================================================================
    # PHASE 3: WRITE
    # =========================================================================
    if verbose:
        print(f"\n[PHASE 3] Writing output file...")

    # Configure performance for writing
    perf = PerformanceConfig.aggressive() \
        .with_cache_mb(perf_params.cache_mb) \
        .with_node_chunk_size(perf_params.node_chunk_size) \
        .with_element_chunk_size(perf_params.element_chunk_size) \
        .with_time_chunk_size(perf_params.time_chunk_size) \
        .with_preemption(perf_params.preemption)

    opts = CreateOptions(mode=CreateMode.Clobber, performance=perf)

    # Initialize output file
    start = time.perf_counter()
    writer = ExodusWriter.create(output_path, opts)

    out_params = InitParams(
        title=params.title + " (Transformed)",
        num_dim=params.num_dim,
        num_nodes=num_nodes,
        num_elems=num_elements,
        num_elem_blocks=params.num_elem_blocks,
        num_node_sets=params.num_node_sets,
        num_side_sets=params.num_side_sets,
    )
    writer.put_init_params(out_params)
    timing["write_init"] = time.perf_counter() - start

    # Write coordinates
    if verbose:
        print(f"  Writing coordinates...")
    start = time.perf_counter()
    writer.put_coords(new_coords_x, new_coords_y, new_coords_z)
    timing["write_coords"] = time.perf_counter() - start

    # Write connectivity
    if verbose:
        print(f"  Writing connectivity...")
    start = time.perf_counter()
    for block in blocks_info:
        writer.put_block(block)
        writer.put_connectivity(block.id, connectivity_data[block.id])
    timing["write_connectivity"] = time.perf_counter() - start

    # Define and write variables
    if verbose:
        print(f"  Writing {num_timesteps:,} timesteps x {num_variables} variables...")
    start = time.perf_counter()
    writer.define_variables(EntityType.Nodal, list(variable_names))

    for step in tqdm(range(num_timesteps), desc="  Writing variables", unit="step",
                     ncols=80, mininterval=1.0, disable=not verbose):
        writer.put_time(step, time_values[step])
        for var_idx in range(num_variables):
            writer.put_var(step, EntityType.Nodal, 0, var_idx, all_var_data[var_idx][step])

    timing["write_variables"] = time.perf_counter() - start

    # Close file
    if verbose:
        print(f"  Closing output file...")
    start = time.perf_counter()
    writer.close()
    timing["close"] = time.perf_counter() - start

    # Get memory stats
    current, peak = tracemalloc.get_traced_memory()
    tracemalloc.stop()
    peak_memory_mb = peak / (1024 * 1024)

    output_size_gb = os.path.getsize(output_path) / (1024 ** 3)

    if verbose:
        write_total = sum([timing["write_init"], timing["write_coords"],
                         timing["write_connectivity"], timing["write_variables"],
                         timing["close"]])
        print(f"  Write complete: {write_total:.2f}s")

    # Build result
    result = TimingResult(
        perf_params=perf_params,
        input_file=input_path,
        output_file=output_path,
        input_size_gb=input_size_gb,
        output_size_gb=output_size_gb,
        num_nodes=num_nodes,
        num_elements=num_elements,
        num_timesteps=num_timesteps,
        num_variables=num_variables,
        time_read_init=timing["read_init"],
        time_read_coords=timing["read_coords"],
        time_read_connectivity=timing["read_connectivity"],
        time_read_variables=timing["read_variables"],
        time_transform_coords=timing["transform_coords"],
        time_transform_variables=timing["transform_variables"],
        time_write_init=timing["write_init"],
        time_write_coords=timing["write_coords"],
        time_write_connectivity=timing["write_connectivity"],
        time_write_variables=timing["write_variables"],
        time_close=timing["close"],
        peak_memory_mb=peak_memory_mb,
    )

    if verbose:
        print(f"\n{'=' * 70}")
        print(f"SUMMARY")
        print(f"{'=' * 70}")
        print(f"  Total Read Time:      {result.time_total_read:.2f}s")
        print(f"  Total Transform Time: {result.time_total_transform:.2f}s")
        print(f"  Total Write Time:     {result.time_total_write:.2f}s")
        print(f"  Total Time:           {result.time_total:.2f}s")
        print(f"  Peak Memory:          {peak_memory_mb:.2f} MB")
        print(f"  Output Size:          {output_size_gb:.2f} GB")
        throughput = (input_size_gb + output_size_gb) / result.time_total
        print(f"  Throughput:           {throughput * 1024:.2f} MB/s")
        print(f"{'=' * 70}")

    return result


def main():
    parser = argparse.ArgumentParser(
        description="Transform Exodus mesh with HDF5 performance options"
    )
    parser.add_argument(
        "--input", "-i",
        type=str,
        required=True,
        help="Input Exodus file path"
    )
    parser.add_argument(
        "--output", "-o",
        type=str,
        required=True,
        help="Output Exodus file path"
    )
    parser.add_argument(
        "--cache-mb",
        type=int,
        default=256,
        help="HDF5 cache size in MB (default: 256)"
    )
    parser.add_argument(
        "--node-chunk-size",
        type=int,
        default=25000,
        help="Node chunk size (default: 25000)"
    )
    parser.add_argument(
        "--element-chunk-size",
        type=int,
        default=25000,
        help="Element chunk size (default: 25000)"
    )
    parser.add_argument(
        "--time-chunk-size",
        type=int,
        default=100,
        help="Time chunk size (default: 100)"
    )
    parser.add_argument(
        "--preemption",
        type=float,
        default=0.75,
        help="HDF5 cache preemption value 0.0-1.0 (default: 0.75)"
    )
    parser.add_argument(
        "--output-json",
        type=str,
        default=None,
        help="Output JSON file for timing results"
    )
    parser.add_argument(
        "--quiet", "-q",
        action="store_true",
        help="Suppress verbose output"
    )

    args = parser.parse_args()

    # Validate input file
    if not os.path.exists(args.input):
        print(f"ERROR: Input file not found: {args.input}")
        return 1

    # Ensure output directory exists
    output_dir = os.path.dirname(args.output)
    if output_dir:
        os.makedirs(output_dir, exist_ok=True)

    # Build performance params
    perf_params = PerformanceParams(
        cache_mb=args.cache_mb,
        node_chunk_size=args.node_chunk_size,
        element_chunk_size=args.element_chunk_size,
        time_chunk_size=args.time_chunk_size,
        preemption=args.preemption,
    )

    try:
        result = run_transform(
            input_path=args.input,
            output_path=args.output,
            perf_params=perf_params,
            verbose=not args.quiet,
        )

        # Write JSON output if requested
        if args.output_json:
            with open(args.output_json, 'w') as f:
                json.dump(result.to_dict(), f, indent=2)
            if not args.quiet:
                print(f"\nResults written to: {args.output_json}")

        return 0

    except Exception as e:
        print(f"ERROR: {e}")
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
