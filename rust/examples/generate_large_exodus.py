#!/usr/bin/env python3
"""
Generate a large Exodus file for transformation performance testing.

This script creates an Exodus file with:
- ~75k nodes
- 3D shell elements (quads)
- 43k time steps
- 9 scalar nodal variables
- 1 tensor element variable (stress with 6 components)

Target file size: ~97GB
"""

import sys
import argparse
import time
import numpy as np
from exodus import (
    ExodusWriter, CreateOptions, CreateMode, InitParams, Block, EntityType,
    PerformanceConfig
)


def generate_quad_mesh(nx, ny):
    """
    Generate a structured quad mesh.

    Args:
        nx: Number of nodes in X direction
        ny: Number of nodes in Y direction

    Returns:
        Tuple of (x_coords, y_coords, z_coords, connectivity)
    """
    print(f"Generating {nx}x{ny} = {nx*ny} node structured quad mesh...")

    # Generate node coordinates (unit square)
    x = np.linspace(0.0, 1.0, nx)
    y = np.linspace(0.0, 1.0, ny)
    xx, yy = np.meshgrid(x, y)

    x_coords = xx.flatten()
    y_coords = yy.flatten()
    z_coords = np.zeros_like(x_coords)

    # Generate quad connectivity
    num_elems = (nx - 1) * (ny - 1)
    connectivity = []

    for j in range(ny - 1):
        for i in range(nx - 1):
            # Node IDs (1-based)
            n1 = j * nx + i + 1
            n2 = n1 + 1
            n3 = n2 + nx
            n4 = n1 + nx
            connectivity.extend([n1, n2, n3, n4])

    print(f"  Created {len(x_coords)} nodes and {num_elems} quad elements")

    return x_coords, y_coords, z_coords, connectivity, num_elems


def main():
    parser = argparse.ArgumentParser(
        description="Generate large Exodus file for transformation performance testing"
    )
    parser.add_argument(
        "--output", "-o",
        default="large_mesh.exo",
        help="Output file path (default: large_mesh.exo)"
    )
    parser.add_argument(
        "--nodes-x",
        type=int,
        default=274,
        help="Number of nodes in X direction (default: 274, gives ~75k nodes)"
    )
    parser.add_argument(
        "--nodes-y",
        type=int,
        default=274,
        help="Number of nodes in Y direction (default: 274, gives ~75k nodes)"
    )
    parser.add_argument(
        "--timesteps",
        type=int,
        default=43000,
        help="Number of time steps (default: 43000)"
    )
    parser.add_argument(
        "--num-scalar-vars",
        type=int,
        default=9,
        help="Number of scalar nodal variables (default: 9)"
    )
    parser.add_argument(
        "--cache-mb",
        type=int,
        default=128,
        help="HDF5 cache size in MB (default: 128)"
    )
    parser.add_argument(
        "--node-chunk-size",
        type=int,
        default=10000,
        help="Node chunk size for HDF5 (default: 10000)"
    )
    parser.add_argument(
        "--element-chunk-size",
        type=int,
        default=8000,
        help="Element chunk size for HDF5 (default: 8000)"
    )
    parser.add_argument(
        "--time-chunk-size",
        type=int,
        default=10,
        help="Time step chunk size for HDF5 (default: 10)"
    )

    args = parser.parse_args()

    # Calculate dimensions
    num_nodes = args.nodes_x * args.nodes_y
    num_elems_approx = (args.nodes_x - 1) * (args.nodes_y - 1)

    print("=" * 70)
    print("GENERATING LARGE EXODUS FILE")
    print("=" * 70)
    print(f"Output file: {args.output}")
    print(f"Mesh dimensions: {args.nodes_x} x {args.nodes_y} = {num_nodes:,} nodes")
    print(f"Approximate elements: {num_elems_approx:,} quads")
    print(f"Time steps: {args.timesteps:,}")
    print(f"Scalar variables: {args.num_scalar_vars}")
    print(f"Tensor variables: 1 (stress with 6 components)")
    print(f"HDF5 cache: {args.cache_mb} MB")
    print(f"Chunk sizes: nodes={args.node_chunk_size}, elems={args.element_chunk_size}, time={args.time_chunk_size}")
    print()

    # Estimate file size
    nodal_var_size = args.num_scalar_vars * num_nodes * args.timesteps * 8 / (1024**3)
    elem_var_size = 6 * num_elems_approx * args.timesteps * 8 / (1024**3)
    estimated_size = nodal_var_size + elem_var_size

    print(f"Estimated file size:")
    print(f"  Nodal variables: ~{nodal_var_size:.1f} GB")
    print(f"  Element variables: ~{elem_var_size:.1f} GB")
    print(f"  Total: ~{estimated_size:.1f} GB")
    print()

    start_time = time.time()

    # Generate mesh
    x_coords, y_coords, z_coords, connectivity, num_elems = generate_quad_mesh(
        args.nodes_x, args.nodes_y
    )

    print(f"\nCreating Exodus file with performance tuning...")

    # Configure performance settings
    perf = PerformanceConfig.auto() \
        .with_cache_mb(args.cache_mb) \
        .with_node_chunk_size(args.node_chunk_size) \
        .with_element_chunk_size(args.element_chunk_size) \
        .with_time_chunk_size(args.time_chunk_size)

    options = CreateOptions(mode=CreateMode.Clobber, performance=perf)

    # Create file
    with ExodusWriter.create(args.output, options) as writer:
        # Initialize database
        params = InitParams(
            title=f"Large test mesh: {num_nodes} nodes, {args.timesteps} steps",
            num_dim=3,
            num_nodes=num_nodes,
            num_elems=num_elems,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        print(f"Writing coordinates...")
        writer.put_coords(x_coords.tolist(), y_coords.tolist(), z_coords.tolist())
        writer.put_coord_names(["X", "Y", "Z"])

        print(f"Defining element block...")
        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="SHELL4",
            num_entries=num_elems,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        print(f"Writing connectivity...")
        writer.put_connectivity(1, connectivity)

        # Define variables
        print(f"Defining {args.num_scalar_vars} scalar nodal variables...")
        nodal_var_names = [f"scalar_{i+1}" for i in range(args.num_scalar_vars)]
        writer.define_variables(EntityType.Nodal, nodal_var_names)

        print(f"Defining stress tensor components (6 element variables)...")
        elem_var_names = ["stress_xx", "stress_yy", "stress_zz",
                         "stress_xy", "stress_yz", "stress_xz"]
        writer.define_variables(EntityType.ElemBlock, elem_var_names)

        # Write time step data
        print(f"\nWriting {args.timesteps:,} time steps...")
        print("This will take a while... Progress updates every 1000 steps")

        for step in range(args.timesteps):
            time_value = step * 0.01  # 0.01 second increments

            writer.put_time(step, time_value)

            # Write nodal scalar variables (varying with time)
            for var_idx in range(args.num_scalar_vars):
                # Create simple varying data (sine wave based on time and variable index)
                values = np.sin(2 * np.pi * (step / 100.0 + var_idx / 10.0)) * np.ones(num_nodes)
                writer.put_var(step, EntityType.Nodal, 0, var_idx, values.tolist())

            # Write element stress tensor components (varying with time)
            for comp_idx in range(6):
                # Create simple varying stress data
                values = (100.0 + comp_idx * 10.0) * np.cos(2 * np.pi * step / 200.0) * np.ones(num_elems)
                writer.put_var(step, EntityType.ElemBlock, 1, comp_idx, values.tolist())

            if (step + 1) % 1000 == 0:
                elapsed = time.time() - start_time
                progress = (step + 1) / args.timesteps * 100
                steps_per_sec = (step + 1) / elapsed
                eta = (args.timesteps - step - 1) / steps_per_sec if steps_per_sec > 0 else 0
                print(f"  Step {step+1:,}/{args.timesteps:,} ({progress:.1f}%) - "
                      f"{steps_per_sec:.1f} steps/sec - ETA: {eta/60:.1f} min")

    total_time = time.time() - start_time

    print(f"\n" + "=" * 70)
    print(f"FILE GENERATION COMPLETE")
    print("=" * 70)
    print(f"Output file: {args.output}")
    print(f"Total time: {total_time/60:.1f} minutes ({total_time:.1f} seconds)")
    print(f"Average: {args.timesteps/total_time:.2f} timesteps/second")

    # Get actual file size
    import os
    file_size_gb = os.path.getsize(args.output) / (1024**3)
    print(f"Actual file size: {file_size_gb:.2f} GB")
    print()


if __name__ == "__main__":
    main()
