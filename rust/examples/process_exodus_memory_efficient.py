#!/usr/bin/env python3
"""
Memory-efficient Exodus file processing script.

This script demonstrates how to process large Exodus files (100GB+) with limited memory
by processing data sequentially and explicitly managing memory.

Operations performed:
1. Read and transform mesh coordinates (e.g., coordinate system transformation)
2. Read, scale, and write time-history field values for all time steps
3. Minimize memory usage by processing one time step at a time

Memory usage: ~3-4x the size of a single time step (not the entire file)
"""

import sys
import gc
from pathlib import Path

try:
    # Import exodus-py (Python bindings for exodus-rs)
    import exodus
except ImportError:
    print("ERROR: exodus-py not installed. Install with: pip install exodus-py", file=sys.stderr)
    print("Or build locally: cd rust/exodus-py && maturin develop --release", file=sys.stderr)
    sys.exit(1)


def transform_coordinates(x, y, z, num_dim):
    """
    Apply coordinate transformation to mesh coordinates.

    Example: Rotate, translate, or scale the mesh.
    This example applies a simple scaling and translation.

    Args:
        x, y, z: Coordinate arrays (lists)
        num_dim: Number of spatial dimensions (1, 2, or 3)

    Returns:
        Transformed (x, y, z) tuple
    """
    print("  Applying coordinate transformation...")

    # Example transformation: scale by 2.0 and translate
    scale_factor = 2.0
    translation = [1.0, 2.0, 3.0]

    x_new = [scale_factor * xi + translation[0] for xi in x]

    if num_dim >= 2:
        y_new = [scale_factor * yi + translation[1] for yi in y]
    else:
        y_new = y

    if num_dim >= 3:
        z_new = [scale_factor * zi + translation[2] for zi in z]
    else:
        z_new = z

    return x_new, y_new, z_new


def scale_field_values(values, scale_factor):
    """
    Apply scaling to field values.

    Args:
        values: List of field values
        scale_factor: Multiplicative scale factor

    Returns:
        Scaled values list
    """
    return [v * scale_factor for v in values]


def process_exodus_file(input_path, output_path, field_scale_factor=1.5):
    """
    Process an Exodus file with minimal memory usage.

    Args:
        input_path: Path to input Exodus file
        output_path: Path to output Exodus file
        field_scale_factor: Factor to scale field values by
    """
    print(f"Processing Exodus file: {input_path}")
    print(f"Output file: {output_path}")
    print(f"Field scale factor: {field_scale_factor}")
    print()

    # Step 1: Open input file for reading
    print("[1/6] Opening input file...")
    reader = exodus.ExodusReader(str(input_path))

    # Step 2: Read metadata (minimal memory)
    print("[2/6] Reading metadata...")
    init_params = reader.get_init_params()
    num_nodes = init_params.num_nodes
    num_elem = init_params.num_elem
    num_dim = init_params.num_dim
    num_blocks = init_params.num_elem_blk
    num_time_steps = reader.num_time_steps()

    print(f"  Nodes: {num_nodes:,}")
    print(f"  Elements: {num_elem:,}")
    print(f"  Dimensions: {num_dim}")
    print(f"  Element Blocks: {num_blocks}")
    print(f"  Time Steps: {num_time_steps:,}")

    # Get variable info
    nodal_var_names = reader.variable_names(exodus.EntityType.NODAL)
    num_nodal_vars = len(nodal_var_names)
    print(f"  Nodal Variables: {num_nodal_vars} - {nodal_var_names}")

    # Estimate memory usage per time step
    bytes_per_node = 8  # f64
    mem_per_step_mb = (num_nodes * num_nodal_vars * bytes_per_node) / (1024 * 1024)
    print(f"  Estimated memory per time step: {mem_per_step_mb:.1f} MB")
    print(f"  Peak memory usage: ~{mem_per_step_mb * 4:.1f} MB (4x per step)")
    print()

    # Step 3: Process coordinates (load once, transform, write)
    print("[3/6] Processing coordinates...")
    print("  Reading coordinates from input...")
    x, y, z = reader.get_coords()

    print(f"  Loaded {len(x):,} nodes")
    print("  Transforming coordinates...")
    x_new, y_new, z_new = transform_coordinates(x, y, z, num_dim)

    # Free original coordinates
    del x, y, z
    gc.collect()

    # Step 4: Create output file and write metadata
    print("[4/6] Creating output file...")
    writer = exodus.ExodusWriter(
        str(output_path),
        init_params,
        exodus.CreateMode.CLOBBER,
        exodus.FloatSize.FLOAT64
    )

    # Write transformed coordinates
    print("  Writing transformed coordinates...")
    writer.put_coords(x_new, y_new if num_dim >= 2 else None, z_new if num_dim >= 3 else None)

    # Free transformed coordinates
    del x_new, y_new, z_new
    gc.collect()

    # Copy coordinate names
    coord_names = reader.get_coord_names()
    if coord_names:
        writer.put_coord_names(coord_names)

    # Copy element blocks (connectivity)
    print("  Copying element blocks...")
    for i in range(num_blocks):
        block_info = reader.get_block_info(exodus.EntityType.ELEM_BLOCK, i)
        writer.define_block(
            exodus.EntityType.ELEM_BLOCK,
            block_info.id,
            block_info.elem_type,
            block_info.num_entries,
            block_info.num_nodes_per_entry,
            block_info.num_attrs_per_entry
        )

        # Copy connectivity
        connectivity = reader.get_connectivity(exodus.EntityType.ELEM_BLOCK, block_info.id)
        writer.put_connectivity(exodus.EntityType.ELEM_BLOCK, block_info.id, connectivity)
        del connectivity

    # Define variables in output file
    if num_nodal_vars > 0:
        print("  Defining variables...")
        writer.define_variables(exodus.EntityType.NODAL, nodal_var_names)

    gc.collect()

    # Step 5: Process time steps sequentially (CRITICAL for memory efficiency)
    print(f"[5/6] Processing {num_time_steps:,} time steps...")
    print("  Processing one time step at a time to minimize memory usage...")

    for step in range(num_time_steps):
        if step % 100 == 0 or step == num_time_steps - 1:
            progress = (step + 1) / num_time_steps * 100
            print(f"  Progress: {step + 1:,}/{num_time_steps:,} ({progress:.1f}%)")

        # Read time value
        time_val = reader.time(step)
        writer.put_time(step, time_val)

        # Process each nodal variable
        for var_idx in range(num_nodal_vars):
            # Read variable data for this time step
            # Memory: ~1x allocation (80MB for 10M nodes)
            data = reader.var(step, exodus.EntityType.NODAL, 0, var_idx)

            # Scale the data
            # Memory: ~1x allocation (new list)
            scaled_data = scale_field_values(data, field_scale_factor)

            # Write immediately
            # Memory: PyO3 copy + type conversion = ~2x additional
            writer.put_var(step, exodus.EntityType.NODAL, 0, var_idx, scaled_data)

            # Explicitly free memory (helps Python GC)
            del data
            del scaled_data

        # Force garbage collection every 1000 steps
        if step % 1000 == 0:
            gc.collect()

    print()
    print("[6/6] Finalizing output file...")

    # Close files (Python handles this automatically, but explicit is better)
    del writer
    del reader
    gc.collect()

    print()
    print("✓ Processing complete!")
    print(f"  Output written to: {output_path}")


def main():
    """Main entry point."""
    if len(sys.argv) < 3:
        print("Usage: python process_exodus_memory_efficient.py INPUT.exo OUTPUT.exo [SCALE_FACTOR]")
        print()
        print("Arguments:")
        print("  INPUT.exo       - Input Exodus file path")
        print("  OUTPUT.exo      - Output Exodus file path")
        print("  SCALE_FACTOR    - Optional scale factor for field values (default: 1.5)")
        print()
        print("Example:")
        print("  python process_exodus_memory_efficient.py input.exo output.exo 2.0")
        sys.exit(1)

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    scale_factor = float(sys.argv[3]) if len(sys.argv) > 3 else 1.5

    if not input_path.exists():
        print(f"ERROR: Input file not found: {input_path}", file=sys.stderr)
        sys.exit(1)

    if output_path.exists():
        print(f"WARNING: Output file exists and will be overwritten: {output_path}")
        response = input("Continue? [y/N] ")
        if response.lower() != 'y':
            print("Aborted.")
            sys.exit(0)

    try:
        process_exodus_file(input_path, output_path, scale_factor)
    except Exception as e:
        print(f"\nERROR: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
