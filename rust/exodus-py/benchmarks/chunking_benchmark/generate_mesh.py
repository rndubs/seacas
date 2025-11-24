#!/usr/bin/env python3
"""
Generate a large (~100GB) Exodus mesh file for benchmarking HDF5 chunking strategies.

This script creates a surface mesh with:
- ~75,000 nodes (QUAD4 surface elements)
- ~18,500 timesteps (to achieve ~100GB file size)
- 9 scalar field variables per timestep

Usage:
    python generate_mesh.py --output /p/lustre1/whitmore/chunking_benchmark/test_mesh.exo
    python generate_mesh.py --output mesh.exo --num-nodes 75000 --num-timesteps 18500
"""

import argparse
import math
import os
import sys
import time
from pathlib import Path

import numpy as np
from tqdm import tqdm

# Add the python directory to the path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "python"))

from exodus import (
    Block,
    CreateMode,
    CreateOptions,
    EntityType,
    ExodusWriter,
    InitParams,
    PerformanceConfig,
)

# Field variable names (9 total)
FIELD_VARIABLES = [
    "temperature",
    "pressure",
    "velocity_x",
    "velocity_y",
    "velocity_z",
    "stress",
    "strain",
    "displacement",
    "heat_flux",
]


def calculate_file_size_gb(num_nodes: int, num_timesteps: int, num_variables: int) -> float:
    """Estimate the file size in GB."""
    # Coordinates: num_nodes * 3 * 8 bytes
    coords_size = num_nodes * 3 * 8

    # Connectivity: ~num_nodes faces * 4 nodes per face * 8 bytes
    num_faces = num_nodes  # Approximately 1:1 ratio for surface mesh
    conn_size = num_faces * 4 * 8

    # Variables: num_timesteps * num_variables * num_nodes * 8 bytes
    var_size = num_timesteps * num_variables * num_nodes * 8

    # Time values: num_timesteps * 8 bytes
    time_size = num_timesteps * 8

    total_bytes = coords_size + conn_size + var_size + time_size
    return total_bytes / (1024 ** 3)


def create_surface_mesh_coords(num_nodes: int) -> tuple:
    """
    Create coordinates for a surface mesh (approximately spherical surface).

    Returns (x, y, z) coordinate arrays.
    """
    # Calculate grid dimensions for approximately num_nodes points
    # Using spherical parameterization
    n_theta = int(math.sqrt(num_nodes / 2))
    n_phi = 2 * n_theta
    actual_nodes = n_theta * n_phi

    print(f"  Creating surface mesh: {n_theta} x {n_phi} = {actual_nodes} nodes")

    # Spherical coordinates
    theta = np.linspace(0, np.pi, n_theta)
    phi = np.linspace(0, 2 * np.pi, n_phi, endpoint=False)

    theta_grid, phi_grid = np.meshgrid(theta, phi, indexing='ij')

    # Convert to Cartesian (sphere of radius 10)
    radius = 10.0
    x = (radius * np.sin(theta_grid) * np.cos(phi_grid)).flatten()
    y = (radius * np.sin(theta_grid) * np.sin(phi_grid)).flatten()
    z = (radius * np.cos(theta_grid)).flatten()

    return x.astype(np.float64), y.astype(np.float64), z.astype(np.float64), n_theta, n_phi


def create_quad_connectivity(n_theta: int, n_phi: int) -> tuple:
    """
    Create QUAD4 connectivity for the surface mesh.

    Returns (connectivity array, num_elements).
    """
    # Each grid cell becomes a QUAD4 element
    num_elements = (n_theta - 1) * n_phi

    connectivity = []
    for i in range(n_theta - 1):
        for j in range(n_phi):
            # Node indices (1-based for Exodus)
            n1 = i * n_phi + j + 1
            n2 = i * n_phi + (j + 1) % n_phi + 1
            n3 = (i + 1) * n_phi + (j + 1) % n_phi + 1
            n4 = (i + 1) * n_phi + j + 1
            connectivity.extend([n1, n2, n3, n4])

    return np.array(connectivity, dtype=np.int64), num_elements


def generate_field_data(
    num_nodes: int,
    timestep: int,
    var_name: str,
    coords_x: np.ndarray,
    coords_y: np.ndarray,
    coords_z: np.ndarray,
) -> np.ndarray:
    """Generate synthetic field data for a given variable and timestep."""
    t = timestep * 0.001  # Time value

    if var_name == "temperature":
        # Temperature: wave pattern that varies with time
        data = 300.0 + 100.0 * np.sin(coords_x * 0.3 + t) * np.cos(coords_y * 0.3)
    elif var_name == "pressure":
        # Pressure: gradient with time oscillation
        data = 101325.0 + 5000.0 * (coords_z / 10.0) + 1000.0 * np.sin(t * 2.0)
    elif var_name == "velocity_x":
        data = 5.0 * np.sin(coords_y * 0.2 + t)
    elif var_name == "velocity_y":
        data = 5.0 * np.cos(coords_x * 0.2 + t)
    elif var_name == "velocity_z":
        data = 2.0 * np.sin(coords_z * 0.2 + t * 0.5)
    elif var_name == "stress":
        data = 1e6 + 1e5 * np.sin(coords_x * 0.1) * np.sin(coords_y * 0.1)
    elif var_name == "strain":
        data = 0.001 + 0.0005 * np.cos(coords_z * 0.2 + t)
    elif var_name == "displacement":
        data = 0.1 * np.sin(coords_x * 0.1 + t) * np.sin(coords_y * 0.1)
    elif var_name == "heat_flux":
        data = 1000.0 + 500.0 * np.exp(-((coords_x ** 2 + coords_y ** 2) / 100.0))
    else:
        data = np.random.rand(num_nodes)

    return data.astype(np.float64)


def generate_mesh(
    output_path: str,
    num_nodes: int = 75000,
    num_timesteps: int = 18500,
    cache_mb: int = 256,
    node_chunk_size: int = 25000,
    element_chunk_size: int = 25000,
    time_chunk_size: int = 100,
) -> dict:
    """
    Generate the benchmark mesh file.

    Returns timing information dictionary.
    """
    timing = {}
    start_total = time.perf_counter()

    # Calculate estimated file size
    estimated_size = calculate_file_size_gb(num_nodes, num_timesteps, len(FIELD_VARIABLES))
    print(f"\nMesh Generation Configuration:")
    print(f"  Target nodes: {num_nodes:,}")
    print(f"  Timesteps: {num_timesteps:,}")
    print(f"  Variables: {len(FIELD_VARIABLES)}")
    print(f"  Estimated file size: {estimated_size:.2f} GB")
    print(f"  Output: {output_path}")
    print(f"\nPerformance Settings:")
    print(f"  Cache: {cache_mb} MB")
    print(f"  Node chunk size: {node_chunk_size}")
    print(f"  Element chunk size: {element_chunk_size}")
    print(f"  Time chunk size: {time_chunk_size}")

    # Create coordinates
    print("\n[1/5] Creating mesh coordinates...")
    start = time.perf_counter()
    coords_x, coords_y, coords_z, n_theta, n_phi = create_surface_mesh_coords(num_nodes)
    actual_nodes = len(coords_x)
    timing["create_coords"] = time.perf_counter() - start
    print(f"  Actual nodes: {actual_nodes:,}")
    print(f"  Time: {timing['create_coords']:.2f}s")

    # Create connectivity
    print("\n[2/5] Creating connectivity...")
    start = time.perf_counter()
    connectivity, num_elements = create_quad_connectivity(n_theta, n_phi)
    timing["create_connectivity"] = time.perf_counter() - start
    print(f"  Elements: {num_elements:,}")
    print(f"  Time: {timing['create_connectivity']:.2f}s")

    # Configure performance options
    perf = PerformanceConfig.aggressive() \
        .with_cache_mb(cache_mb) \
        .with_node_chunk_size(node_chunk_size) \
        .with_element_chunk_size(element_chunk_size) \
        .with_time_chunk_size(time_chunk_size)

    opts = CreateOptions(mode=CreateMode.Clobber, performance=perf)

    # Create the Exodus file
    print("\n[3/5] Initializing Exodus file...")
    start = time.perf_counter()
    writer = ExodusWriter.create(output_path, opts)

    params = InitParams(
        title="Chunking Benchmark Mesh (~100GB)",
        num_dim=3,
        num_nodes=actual_nodes,
        num_elems=num_elements,
        num_elem_blocks=1,
        num_node_sets=0,
        num_side_sets=0,
    )
    writer.put_init_params(params)
    timing["init_file"] = time.perf_counter() - start
    print(f"  Time: {timing['init_file']:.2f}s")

    # Write coordinates
    print("\n[4/5] Writing coordinates and connectivity...")
    start = time.perf_counter()
    writer.put_coords(coords_x, coords_y, coords_z)

    # Write element block
    block = Block(
        id=1,
        entity_type=EntityType.ElemBlock,
        topology="QUAD4",
        num_entries=num_elements,
        num_nodes_per_entry=4,
        num_attributes=0,
    )
    writer.put_block(block)
    writer.put_connectivity(1, connectivity)
    timing["write_mesh"] = time.perf_counter() - start
    print(f"  Time: {timing['write_mesh']:.2f}s")

    # Define nodal variables
    writer.define_variables(EntityType.Nodal, FIELD_VARIABLES)

    # Write timesteps with variable data
    print(f"\n[5/5] Writing {num_timesteps:,} timesteps with {len(FIELD_VARIABLES)} variables each...")
    start = time.perf_counter()

    for step in tqdm(range(num_timesteps), desc="  Writing timesteps", unit="step",
                      ncols=80, mininterval=1.0, miniters=100):
        time_val = step * 0.001
        writer.put_time(step, time_val)

        for var_idx, var_name in enumerate(FIELD_VARIABLES):
            data = generate_field_data(
                actual_nodes, step, var_name,
                coords_x, coords_y, coords_z
            )
            writer.put_var(step, EntityType.Nodal, 0, var_idx, data)

    timing["write_timesteps"] = time.perf_counter() - start
    print(f"  Time: {timing['write_timesteps']:.2f}s")

    # Close file
    print("\nClosing file...")
    start = time.perf_counter()
    writer.close()
    timing["close_file"] = time.perf_counter() - start

    timing["total"] = time.perf_counter() - start_total

    # Get actual file size
    actual_size = os.path.getsize(output_path) / (1024 ** 3)

    print(f"\n{'=' * 60}")
    print(f"Mesh Generation Complete!")
    print(f"{'=' * 60}")
    print(f"  File: {output_path}")
    print(f"  Actual size: {actual_size:.2f} GB")
    print(f"  Total time: {timing['total']:.2f}s")
    print(f"  Write throughput: {actual_size / timing['total'] * 1024:.2f} MB/s")

    timing["file_size_gb"] = actual_size
    timing["actual_nodes"] = actual_nodes
    timing["num_elements"] = num_elements

    return timing


def main():
    parser = argparse.ArgumentParser(
        description="Generate a large (~100GB) Exodus mesh for benchmarking"
    )
    parser.add_argument(
        "--output", "-o",
        type=str,
        required=True,
        help="Output file path"
    )
    parser.add_argument(
        "--num-nodes",
        type=int,
        default=75000,
        help="Target number of nodes (default: 75000)"
    )
    parser.add_argument(
        "--num-timesteps",
        type=int,
        default=18500,
        help="Number of timesteps (default: 18500 for ~100GB)"
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

    args = parser.parse_args()

    # Ensure output directory exists
    output_dir = os.path.dirname(args.output)
    if output_dir:
        os.makedirs(output_dir, exist_ok=True)

    timing = generate_mesh(
        output_path=args.output,
        num_nodes=args.num_nodes,
        num_timesteps=args.num_timesteps,
        cache_mb=args.cache_mb,
        node_chunk_size=args.node_chunk_size,
        element_chunk_size=args.element_chunk_size,
        time_chunk_size=args.time_chunk_size,
    )

    return 0


if __name__ == "__main__":
    sys.exit(main())
