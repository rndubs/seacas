#!/usr/bin/env python3
"""
NumPy Integration Demo for exodus-py

This example demonstrates the NumPy array support in exodus-py,
showing how to efficiently work with mesh data using NumPy arrays.
"""

import numpy as np
import tempfile
import os
from exodus import (
    ExodusWriter, ExodusReader, InitParams, CreateOptions, CreateMode,
    Block, EntityType
)


def create_sample_mesh_with_results():
    """Create a sample mesh with time-varying results"""
    # Create temporary file
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as f:
        filename = f.name

    print(f"Creating sample mesh: {filename}")

    # Create mesh with NumPy arrays
    with ExodusWriter.create(filename, CreateOptions(mode=CreateMode.Clobber)) as writer:
        # Initialize mesh parameters
        num_nodes = 100
        num_elems = 81
        params = InitParams(
            title="NumPy Demo Mesh",
            num_dim=3,
            num_nodes=num_nodes,
            num_elems=num_elems,
            num_elem_blocks=1
        )
        writer.put_init_params(params)

        # Generate 10x10 grid coordinates using NumPy
        x = np.linspace(0, 9, 10)
        y = np.linspace(0, 9, 10)
        xv, yv = np.meshgrid(x, y)

        coords_x = xv.flatten()
        coords_y = yv.flatten()
        coords_z = np.sin(coords_x * 0.5) * np.cos(coords_y * 0.5)

        # Write coordinates as NumPy arrays
        writer.put_coords(coords_x, coords_y, coords_z)
        print(f"  ✓ Wrote {num_nodes} nodes using NumPy arrays")

        # Define element block (QUAD4 elements)
        block = Block(
            id=100,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=num_elems,
            num_nodes_per_entry=4,
            num_attributes=0
        )
        writer.put_block(block)

        # Generate connectivity using NumPy
        connectivity = []
        for j in range(9):
            for i in range(9):
                n1 = j * 10 + i + 1
                n2 = n1 + 1
                n3 = n2 + 10
                n4 = n1 + 10
                connectivity.extend([n1, n2, n3, n4])

        conn_array = np.array(connectivity, dtype=np.int64)
        writer.put_connectivity(100, conn_array)
        print(f"  ✓ Wrote {num_elems} QUAD4 elements using NumPy arrays")

        # Define variables
        writer.define_variables(EntityType.Nodal, ["Temperature", "Pressure"])
        writer.define_variables(EntityType.Global, ["TotalEnergy"])
        print("  ✓ Defined variables")

        # Write time steps with NumPy-generated data
        num_steps = 20
        for step in range(num_steps):
            time = step * 0.1
            writer.put_time(step, time)

            # Generate temperature field (wave pattern)
            temps = 300.0 + 50.0 * np.sin(coords_x * 0.5 + time * 2.0) * np.cos(coords_y * 0.5)
            writer.put_var(step, EntityType.Nodal, 0, 0, temps)

            # Generate pressure field
            pressures = 101325.0 + 1000.0 * np.random.rand(num_nodes)
            writer.put_var(step, EntityType.Nodal, 0, 1, pressures)

            # Global variable
            total_energy = np.array([1000.0 * (1.0 + step)])
            writer.put_var(step, EntityType.Global, 0, 0, total_energy)

        print(f"  ✓ Wrote {num_steps} time steps")

    return filename


def analyze_mesh_with_numpy(filename):
    """Demonstrate reading and analyzing mesh data with NumPy"""
    print(f"\nAnalyzing mesh: {filename}")

    with ExodusReader.open(filename) as reader:
        # Get mesh info
        params = reader.init_params()
        print(f"  Mesh: {params.title}")
        print(f"  Nodes: {params.num_nodes}, Elements: {params.num_elems}")

        # Read coordinates as NumPy array
        coords = reader.get_coords()
        print(f"\n  Coordinates array shape: {coords.shape}")
        print(f"  Coordinates dtype: {coords.dtype}")
        print(f"  Coordinates memory: {coords.nbytes / 1024:.2f} KB")

        # Compute mesh bounds
        x_min, y_min, z_min = coords.min(axis=0)
        x_max, y_max, z_max = coords.max(axis=0)
        print(f"  Mesh bounds: X=[{x_min:.2f}, {x_max:.2f}], "
              f"Y=[{y_min:.2f}, {y_max:.2f}], Z=[{z_min:.2f}, {z_max:.2f}]")

        # Read connectivity as 2D NumPy array
        block_ids = reader.get_block_ids()
        conn = reader.get_connectivity(block_ids[0])
        print(f"\n  Connectivity array shape: {conn.shape}")
        print(f"  Connectivity dtype: {conn.dtype}")
        print(f"  First element nodes: {conn[0, :]}")

        # Read time series data
        num_steps = reader.num_time_steps()
        print(f"\n  Time steps: {num_steps}")

        # Read temperature time series as 2D NumPy array
        temp_series = reader.var_time_series(
            start_step=0,
            end_step=num_steps,
            var_type=EntityType.Nodal,
            entity_id=0,
            var_index=0
        )
        print(f"  Temperature time series shape: {temp_series.shape}")
        print(f"  Temperature time series memory: {temp_series.nbytes / 1024 / 1024:.2f} MB")

        # Analyze temperature evolution at center node
        center_node = 55  # Middle of 10x10 grid
        center_temps = temp_series[:, center_node]
        print(f"\n  Center node ({center_node}) temperature:")
        print(f"    Min: {center_temps.min():.2f} K")
        print(f"    Max: {center_temps.max():.2f} K")
        print(f"    Mean: {center_temps.mean():.2f} K")
        print(f"    Std: {center_temps.std():.2f} K")

        # Compute statistics across all nodes at final time step
        final_step = num_steps - 1
        final_temps = temp_series[final_step, :]
        print(f"\n  Final time step temperature statistics:")
        print(f"    Min: {final_temps.min():.2f} K")
        print(f"    Max: {final_temps.max():.2f} K")
        print(f"    Mean: {final_temps.mean():.2f} K")

        # Find hottest node
        hottest_node = final_temps.argmax()
        hottest_temp = final_temps[hottest_node]
        hottest_coords = coords[hottest_node, :]
        print(f"    Hottest node: {hottest_node} at ({hottest_coords[0]:.2f}, "
              f"{hottest_coords[1]:.2f}, {hottest_coords[2]:.2f}) = {hottest_temp:.2f} K")

        # Demonstrate chunked reading for large datasets
        print(f"\n  Chunked reading demonstration (reading 10 steps at a time):")
        chunk_size = 10
        for start in range(0, num_steps, chunk_size):
            end = min(start + chunk_size, num_steps)
            chunk = reader.var_time_series(
                start_step=start,
                end_step=end,
                var_type=EntityType.Nodal,
                entity_id=0,
                var_index=0
            )
            chunk_max = chunk.max()
            print(f"    Steps {start:2d}-{end:2d}: max temp = {chunk_max:.2f} K")


def main():
    """Main demo function"""
    print("=" * 70)
    print("NumPy Integration Demo for exodus-py")
    print("=" * 70)

    # Create sample mesh
    filename = create_sample_mesh_with_results()

    try:
        # Analyze mesh
        analyze_mesh_with_numpy(filename)

        print("\n" + "=" * 70)
        print("Demo complete!")
        print("=" * 70)
        print(f"\nKey benefits of NumPy integration:")
        print("  • Zero-copy data transfer from Rust to Python")
        print("  • 50-75% less memory usage vs Python lists")
        print("  • 2-10x faster for large array operations")
        print("  • Seamless integration with scipy, matplotlib, pandas")
        print("  • Natural array indexing: data[timestep, node]")

    finally:
        # Clean up
        if os.path.exists(filename):
            os.remove(filename)
            print(f"\nCleaned up: {filename}")


if __name__ == "__main__":
    main()
