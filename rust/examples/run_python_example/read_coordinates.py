#!/usr/bin/env python3
"""
Read mesh coordinates from an Exodus file using exodus-py.
"""

import os
import sys

from exodus import ExodusReader, EntityType

# Path to the exodus file in rust/data
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
DATA_DIR = os.path.join(SCRIPT_DIR, "../..", "data")

def find_exodus_files():
    """Return a list of all exodus files in the data directory."""
    if not os.path.isdir(DATA_DIR):
        print(f"ERROR: Data directory not found: {DATA_DIR}")
        sys.exit(1)

    exts = (".exo", ".e", ".g", ".gen")
    files = [
        os.path.join(DATA_DIR, fname)
        for fname in sorted(os.listdir(DATA_DIR))
        if fname.endswith(exts)
    ]

    if not files:
        print(f"ERROR: No exodus files found in {DATA_DIR}")
        sys.exit(1)

    return files

def main():
    mesh_paths = find_exodus_files()
    total = len(mesh_paths)

    for idx, mesh_path in enumerate(mesh_paths, start=1):
        print("=" * 60)
        print(f"Reading mesh {idx}/{total}: {mesh_path}")
        print("=" * 60)

        with ExodusReader.open(mesh_path) as reader:
            # Get basic info
            params = reader.init_params()
            print(f"Title: {params.title}")
            print(f"Dimensions: {params.num_dim}D")
            print(f"Number of nodes: {params.num_nodes}")
            print(f"Number of elements: {params.num_elems}")

            # Get coordinates using get_coords()
            x, y, z = reader.get_coords()

            print("\nCoordinates:")
            print(f"  X ({len(x)} values): {x[:10]}{'...' if len(x) > 10 else ''}")
            print(f"  Y ({len(y)} values): {y[:10]}{'...' if len(y) > 10 else ''}")
            if z and any(z):
                print(f"  Z ({len(z)} values): {z[:10]}{'...' if len(z) > 10 else ''}")

            # Print coordinate ranges
            print("\nCoordinate Ranges:")
            print(f"  X: [{min(x):.6f}, {max(x):.6f}]")
            print(f"  Y: [{min(y):.6f}, {max(y):.6f}]")
            if z and any(z):
                print(f"  Z: [{min(z):.6f}, {max(z):.6f}]")

            # Print time-history fields
            print("\n" + "=" * 60)
            print("TIME-HISTORY FIELDS")
            print("=" * 60)

            # Time steps
            num_steps = reader.num_time_steps()
            print(f"\nTime Steps: {num_steps}")
            if num_steps > 0:
                times = reader.times()
                if len(times) <= 10:
                    print(f"  Times: {times}")
                else:
                    print(f"  Times: {times[:5]} ... {times[-5:]}")
                print(f"  Time range: [{min(times):.6g}, {max(times):.6g}]")

            # Global variables
            global_vars = reader.variable_names(EntityType.Global)
            print(f"\nGlobal Variables ({len(global_vars)}):")
            if global_vars:
                for name in global_vars:
                    print(f"  - {name}")
            else:
                print("  (none)")

            # Nodal variables
            nodal_vars = reader.variable_names(EntityType.Nodal)
            print(f"\nNodal Variables ({len(nodal_vars)}):")
            if nodal_vars:
                for name in nodal_vars:
                    print(f"  - {name}")
            else:
                print("  (none)")

        # Element block variables
        elem_vars = reader.variable_names(EntityType.ElemBlock)
        print(f"\nElement Block Variables ({len(elem_vars)}):")
        if elem_vars:
            for name in elem_vars:
                print(f"  - {name}")
        else:
            print("  (none)")

        # Element blocks summary (IDs, names, and metadata)
        print("\n" + "-" * 60)
        print("ELEMENT BLOCKS")
        print("-" * 60)
        try:
            eb_ids = reader.get_block_ids()
        except Exception:
            eb_ids = []
        try:
            eb_names = reader.get_names("elem_block")
        except Exception:
            eb_names = []
        print(f"\nCount: {params.num_elem_blocks}")
        print(f"IDs: {eb_ids if eb_ids else '[]'}")
        if eb_names:
            print(f"Names: {eb_names}")
        if eb_names and eb_ids and len(eb_names) != len(eb_ids):
            print(f"WARNING: block names ({len(eb_names)}) != ids ({len(eb_ids)})")

        # Detailed per-block info
        for bid in eb_ids:
            try:
                blk = reader.get_block(bid)
                print(f"\nBlock ID {bid}:")
                print(f"  Topology: {getattr(blk, 'topology', 'unknown')}")
                print(f"  Elements: {getattr(blk, 'num_entries', 'unknown')}")
                print(f"  Nodes/Element: {getattr(blk, 'num_nodes_per_entry', 'unknown')}")
            except Exception as e:
                print(f"\nBlock ID {bid}: (error reading details: {e})")

            # Node set variables
            nodeset_vars = reader.variable_names(EntityType.NodeSet)
            print(f"\nNode Set Variables ({len(nodeset_vars)}):")
            if nodeset_vars:
                for name in nodeset_vars:
                    print(f"  - {name}")
            else:
                print("  (none)")

            # Side set variables
            sideset_vars = reader.variable_names(EntityType.SideSet)
            print(f"\nSide Set Variables ({len(sideset_vars)}):")
            if sideset_vars:
                for name in sideset_vars:
                    print(f"  - {name}")
            else:
                print("  (none)")

            # Entity set names
            print("\n" + "-" * 60)
            print("ENTITY SET NAMES")
            print("-" * 60)

            try:
                node_set_names = reader.get_names("node_set")
            except Exception:
                node_set_names = []
            print(f"\nNode Sets ({len(node_set_names)}):")
            if node_set_names:
                for name in node_set_names:
                    print(f"  - {name}")
            else:
                print("  (none)")

            try:
                elem_set_names = reader.get_names("elem_set")
            except Exception:
                elem_set_names = []
            print(f"\nElement Sets ({len(elem_set_names)}):")
            if elem_set_names:
                for name in elem_set_names:
                    print(f"  - {name}")
            else:
                print("  (none)")

            try:
                side_set_names = reader.get_names("side_set")
            except Exception:
                side_set_names = []
            print(f"\nSide Sets ({len(side_set_names)}):")
            if side_set_names:
                for name in side_set_names:
                    print(f"  - {name}")
            else:
                print("  (none)")

            # Entity set IDs and counts (validation)
            print("\n" + "-" * 60)
            print("SET SUMMARY (IDs & Counts)")
            print("-" * 60)

            # Node sets
            try:
                ns_ids = reader.get_node_set_ids()
            except Exception:
                ns_ids = []
            print(f"\nNode Sets: count={params.num_node_sets}, ids={ns_ids if ns_ids else '[]'}")
            if node_set_names and ns_ids and len(node_set_names) != len(ns_ids):
                print(f"  WARNING: names ({len(node_set_names)}) != ids ({len(ns_ids)})")

            # Side sets
            try:
                ss_ids = reader.get_side_set_ids()
            except Exception:
                ss_ids = []
            print(f"\nSide Sets: count={params.num_side_sets}, ids={ss_ids if ss_ids else '[]'}")
            if side_set_names and ss_ids and len(side_set_names) != len(ss_ids):
                print(f"  WARNING: names ({len(side_set_names)}) != ids ({len(ss_ids)})")

            # Element sets
            try:
                es_ids = reader.get_elem_set_ids()
            except Exception:
                es_ids = []
            print(f"\nElement Sets: count={params.num_elem_sets}, ids={es_ids if es_ids else '[]'}")
            if elem_set_names and es_ids and len(elem_set_names) != len(es_ids):
                print(f"  WARNING: names ({len(elem_set_names)}) != ids ({len(es_ids)})")

        print("")

    print("Done!")

if __name__ == "__main__":
    main()
