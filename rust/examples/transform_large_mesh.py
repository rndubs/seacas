#!/usr/bin/env python3
"""
Transform large Exodus meshes with performance monitoring.

This script demonstrates applying transformations (rotation, translation, scaling)
to large Exodus files with comprehensive performance tuning options.

Features:
- Rotate mesh and stress tensors by 90 degrees around Z axis
- Scale mesh by 5x
- Scale scalar values by 10x
- Performance monitoring for different cache/chunk configurations
- CLI arguments for tuning HDF5 cache and chunking
"""

import sys
import argparse
import time
import math
import numpy as np
from exodus import (
    ExodusReader, ExodusWriter, ExodusAppender, CreateOptions, CreateMode,
    EntityType, InitParams, PerformanceConfig
)


class TimingData:
    """Track performance timing data"""

    def __init__(self):
        self.read_metadata = 0.0
        self.copy_mesh = 0.0
        self.transform_coords = 0.0
        self.transform_variables = 0.0
        self.write_output = 0.0
        self.total = 0.0

    def print_summary(self):
        """Print timing summary"""
        print("\n" + "=" * 70)
        print("PERFORMANCE SUMMARY")
        print("=" * 70)
        print(f"{'Read metadata:':<30} {self.read_metadata:>12.2f} seconds")
        print(f"{'Copy mesh:':<30} {self.copy_mesh:>12.2f} seconds")
        print(f"{'Transform coordinates:':<30} {self.transform_coords:>12.2f} seconds")
        print(f"{'Transform variables:':<30} {self.transform_variables:>12.2f} seconds")
        print(f"{'Write output:':<30} {self.write_output:>12.2f} seconds")
        print("-" * 70)
        print(f"{'TOTAL:':<30} {self.total:>12.2f} seconds")
        print("=" * 70)


def rotation_matrix_z(angle_rad):
    """Create a 3x3 rotation matrix for rotation around Z axis."""
    c = math.cos(angle_rad)
    s = math.sin(angle_rad)
    return [c, -s, 0.0, s, c, 0.0, 0.0, 0.0, 1.0]


def apply_rotation_to_vector(matrix, vec):
    """Apply a 3x3 rotation matrix to a 3D vector."""
    return [
        matrix[0] * vec[0] + matrix[1] * vec[1] + matrix[2] * vec[2],
        matrix[3] * vec[0] + matrix[4] * vec[1] + matrix[5] * vec[2],
        matrix[6] * vec[0] + matrix[7] * vec[1] + matrix[8] * vec[2],
    ]


def rotate_symmetric_tensor(rotation, tensor):
    """
    Apply a 3x3 rotation matrix to a symmetric tensor.

    For a symmetric tensor T and rotation matrix R, computes: R * T * R^T

    Args:
        rotation: The 3x3 rotation matrix (flat array of 9 values)
        tensor: The symmetric tensor in Voigt notation [T11, T22, T33, T12, T23, T13]

    Returns:
        The rotated tensor in Voigt notation
    """
    # Convert Voigt notation to full 3x3 matrix
    t = [
        tensor[0], tensor[3], tensor[5],
        tensor[3], tensor[1], tensor[4],
        tensor[5], tensor[4], tensor[2],
    ]

    # Compute R * T
    rt = [0.0] * 9
    for i in range(3):
        for j in range(3):
            s = 0.0
            for k in range(3):
                s += rotation[i * 3 + k] * t[k * 3 + j]
            rt[i * 3 + j] = s

    # Compute (R * T) * R^T
    result_matrix = [0.0] * 9
    for i in range(3):
        for j in range(3):
            s = 0.0
            for k in range(3):
                # R^T[j][k] = R[k][j]
                s += rt[i * 3 + k] * rotation[j * 3 + k]
            result_matrix[i * 3 + j] = s

    # Convert back to Voigt notation
    return [
        result_matrix[0],  # T11
        result_matrix[4],  # T22
        result_matrix[8],  # T33
        result_matrix[1],  # T12
        result_matrix[5],  # T23
        result_matrix[2],  # T13
    ]


def find_stress_tensor_indices(var_names):
    """Find indices of stress tensor components in variable names."""
    try:
        xx = next(i for i, n in enumerate(var_names) if "stress_xx" in n)
        yy = next(i for i, n in enumerate(var_names) if "stress_yy" in n)
        zz = next(i for i, n in enumerate(var_names) if "stress_zz" in n)
        xy = next(i for i, n in enumerate(var_names) if "stress_xy" in n)
        yz = next(i for i, n in enumerate(var_names) if "stress_yz" in n)
        xz = next(i for i, n in enumerate(var_names) if "stress_xz" in n)
        return (xx, yy, zz, xy, yz, xz)
    except StopIteration:
        return None


def main():
    parser = argparse.ArgumentParser(
        description="Transform large Exodus meshes with performance monitoring"
    )
    parser.add_argument(
        "--input", "-i",
        required=True,
        help="Input Exodus file path"
    )
    parser.add_argument(
        "--output", "-o",
        required=True,
        help="Output Exodus file path"
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
    parser.add_argument(
        "--preemption",
        type=float,
        default=0.75,
        help="Cache preemption policy (0.0-1.0, default: 0.75)"
    )
    parser.add_argument(
        "--rotation",
        type=float,
        default=90.0,
        help="Rotation angle in degrees (default: 90)"
    )
    parser.add_argument(
        "--mesh-scale",
        type=float,
        default=5.0,
        help="Mesh scale factor (default: 5.0)"
    )
    parser.add_argument(
        "--scalar-scale",
        type=float,
        default=10.0,
        help="Scalar variable scale factor (default: 10.0)"
    )

    args = parser.parse_args()
    timing = TimingData()

    start_total = time.time()

    print("=" * 70)
    print("TRANSFORM LARGE EXODUS MESH")
    print("=" * 70)
    print(f"Input:  {args.input}")
    print(f"Output: {args.output}")
    print(f"\nTransformations:")
    print(f"  - Rotate {args.rotation} degrees around Z axis")
    print(f"  - Scale mesh by {args.mesh_scale}")
    print(f"  - Scale scalar variables by {args.scalar_scale}")
    print(f"\nPerformance settings:")
    print(f"  - Cache size: {args.cache_mb} MB")
    print(f"  - Node chunk size: {args.node_chunk_size}")
    print(f"  - Element chunk size: {args.element_chunk_size}")
    print(f"  - Time chunk size: {args.time_chunk_size}")
    print(f"  - Cache preemption: {args.preemption}")
    print()

    # Step 1: Read metadata from input file
    print("Step 1: Reading metadata from input file...")
    start = time.time()

    input_file = ExodusReader.open(args.input)
    params = input_file.init_params()
    x_coords, y_coords, z_coords = input_file.get_coords()
    coord_names = input_file.get_coord_names()
    block_ids = input_file.get_block_ids()
    nodal_var_names = input_file.variable_names(EntityType.Nodal)
    elem_var_names = input_file.variable_names(EntityType.ElemBlock)
    num_time_steps = input_file.num_time_steps()

    timing.read_metadata = time.time() - start

    print(f"  Nodes: {params.num_nodes}")
    print(f"  Elements: {params.num_elems}")
    print(f"  Element blocks: {params.num_elem_blocks}")
    print(f"  Nodal variables: {len(nodal_var_names)}")
    print(f"  Element variables: {len(elem_var_names)}")
    print(f"  Time steps: {num_time_steps}")
    print(f"  Duration: {timing.read_metadata:.2f}s")

    # Step 2: Create output file with performance config
    print(f"\nStep 2: Creating output file...")
    start = time.time()

    perf = PerformanceConfig.auto() \
        .with_cache_mb(args.cache_mb) \
        .with_node_chunk_size(args.node_chunk_size) \
        .with_element_chunk_size(args.element_chunk_size) \
        .with_time_chunk_size(args.time_chunk_size) \
        .with_preemption(args.preemption)

    options = CreateOptions(mode=CreateMode.Clobber, performance=perf)
    output_file = ExodusWriter.create(args.output, options)

    # Initialize with same parameters
    output_file.put_init_params(params)
    output_file.put_coord_names(coord_names)

    # Copy element blocks
    for block_id in block_ids:
        block = input_file.get_block(block_id)
        output_file.put_block(block)

        connectivity = input_file.get_connectivity(block_id)
        output_file.put_connectivity(block_id, connectivity)

    # Define variables
    if nodal_var_names:
        output_file.define_variables(EntityType.Nodal, nodal_var_names)
    if elem_var_names:
        output_file.define_variables(EntityType.ElemBlock, elem_var_names)

    timing.copy_mesh = time.time() - start
    print(f"  Duration: {timing.copy_mesh:.2f}s")

    # Step 3: Transform coordinates
    print(f"\nStep 3: Transforming coordinates...")
    start = time.time()

    rotation_matrix = rotation_matrix_z(math.radians(args.rotation))

    # Apply rotation to coordinates
    num_nodes = len(x_coords)
    new_x = []
    new_y = []
    new_z = []

    for i in range(num_nodes):
        point = [x_coords[i], y_coords[i], z_coords[i]]
        rotated = apply_rotation_to_vector(rotation_matrix, point)
        new_x.append(rotated[0] * args.mesh_scale)
        new_y.append(rotated[1] * args.mesh_scale)
        new_z.append(rotated[2] * args.mesh_scale)

    output_file.put_coords(new_x, new_y, new_z)

    timing.transform_coords = time.time() - start
    print(f"  Duration: {timing.transform_coords:.2f}s")

    # Step 4: Transform and write variables
    print(f"\nStep 4: Transforming and writing variables...")
    print(f"  Processing {num_time_steps} time steps...")
    start = time.time()

    stress_indices = find_stress_tensor_indices(elem_var_names)

    for step in range(num_time_steps):
        # Write time value
        time_value = input_file.time(step)
        output_file.put_time(step, time_value)

        # Transform nodal variables (scale scalars)
        for var_idx in range(len(nodal_var_names)):
            values = input_file.var(step, EntityType.Nodal, 0, var_idx)
            scaled_values = [v * args.scalar_scale for v in values]
            output_file.put_var(step, EntityType.Nodal, 0, var_idx, scaled_values)

        # Transform element variables
        for block_id in block_ids:
            block = input_file.get_block(block_id)
            num_elems = block.num_entries

            if stress_indices is not None:
                xx, yy, zz, xy, yz, xz = stress_indices

                # Read all stress components
                stress_xx = input_file.var(step, EntityType.ElemBlock, block_id, xx)
                stress_yy = input_file.var(step, EntityType.ElemBlock, block_id, yy)
                stress_zz = input_file.var(step, EntityType.ElemBlock, block_id, zz)
                stress_xy = input_file.var(step, EntityType.ElemBlock, block_id, xy)
                stress_yz = input_file.var(step, EntityType.ElemBlock, block_id, yz)
                stress_xz = input_file.var(step, EntityType.ElemBlock, block_id, xz)

                # Transform each element's stress tensor
                rotated_xx = []
                rotated_yy = []
                rotated_zz = []
                rotated_xy = []
                rotated_yz = []
                rotated_xz = []

                for elem_idx in range(num_elems):
                    tensor = [
                        stress_xx[elem_idx],
                        stress_yy[elem_idx],
                        stress_zz[elem_idx],
                        stress_xy[elem_idx],
                        stress_yz[elem_idx],
                        stress_xz[elem_idx],
                    ]
                    rotated = rotate_symmetric_tensor(rotation_matrix, tensor)
                    rotated_xx.append(rotated[0])
                    rotated_yy.append(rotated[1])
                    rotated_zz.append(rotated[2])
                    rotated_xy.append(rotated[3])
                    rotated_yz.append(rotated[4])
                    rotated_xz.append(rotated[5])

                # Write rotated stress components
                output_file.put_var(step, EntityType.ElemBlock, block_id, xx, rotated_xx)
                output_file.put_var(step, EntityType.ElemBlock, block_id, yy, rotated_yy)
                output_file.put_var(step, EntityType.ElemBlock, block_id, zz, rotated_zz)
                output_file.put_var(step, EntityType.ElemBlock, block_id, xy, rotated_xy)
                output_file.put_var(step, EntityType.ElemBlock, block_id, yz, rotated_yz)
                output_file.put_var(step, EntityType.ElemBlock, block_id, xz, rotated_xz)
            else:
                # No stress tensor, just copy variables
                for var_idx in range(len(elem_var_names)):
                    values = input_file.var(step, EntityType.ElemBlock, block_id, var_idx)
                    output_file.put_var(step, EntityType.ElemBlock, block_id, var_idx, values)

        if (step + 1) % 1000 == 0:
            elapsed = time.time() - start
            progress = (step + 1) / num_time_steps * 100
            steps_per_sec = (step + 1) / elapsed
            eta_sec = (num_time_steps - step - 1) / steps_per_sec if steps_per_sec > 0 else 0
            print(f"  Step {step+1}/{num_time_steps} ({progress:.1f}%) - "
                  f"{steps_per_sec:.1f} steps/sec - ETA: {eta_sec/60:.1f} min")

    timing.transform_variables = time.time() - start
    print(f"  Duration: {timing.transform_variables:.2f}s")

    # Close files
    print(f"\nStep 5: Finalizing output file...")
    start = time.time()
    output_file.close()
    input_file.close()
    timing.write_output = time.time() - start
    print(f"  Duration: {timing.write_output:.2f}s")

    timing.total = time.time() - start_total
    timing.print_summary()

    # Calculate throughput
    import os
    file_size_gb = os.path.getsize(args.input) / (1024**3)
    throughput = file_size_gb / timing.total

    print(f"\nThroughput: {throughput:.2f} GB/s")
    print(f"\nOutput file: {args.output}")


if __name__ == "__main__":
    main()
