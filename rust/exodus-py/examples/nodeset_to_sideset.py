#!/usr/bin/env python3
"""
Example: Converting a NodeSet to a SideSet

This example demonstrates how to convert a nodeset (collection of nodes)
to a sideset (collection of element faces) using the exodus library.

The conversion automatically:
1. Identifies element faces where all nodes are in the nodeset
2. Filters for boundary faces only (faces on mesh exterior)
3. Verifies normals point outward from mesh center
4. Checks for consistent normal orientations
"""

from exodus import ExodusReader, ExodusAppender

def example_read_and_convert():
    """
    Example 1: Read a file and convert nodeset to sideset (read-only)

    This reads the mesh, performs the conversion, and returns the sideset
    without modifying the original file.
    """
    print("=" * 60)
    print("Example 1: Read-only conversion")
    print("=" * 60)

    # Open file for reading
    reader = ExodusReader.open("mesh_with_nodesets.exo")

    # Convert nodeset 10 to sideset 100
    sideset = reader.convert_nodeset_to_sideset(
        nodeset_id=10,
        new_sideset_id=100
    )

    print(f"✓ Created sideset {sideset.id}")
    print(f"  - Number of faces: {len(sideset.elements)}")
    print(f"  - Element IDs: {sideset.elements[:10]}...")  # Show first 10
    print(f"  - Side numbers: {sideset.sides[:10]}...")

    reader.close()
    print()


def example_convert_and_write():
    """
    Example 2: Convert nodeset and write to file (modify file)

    This opens a file in append mode, converts the nodeset,
    and writes the new sideset to the file.
    """
    print("=" * 60)
    print("Example 2: Convert and write to file")
    print("=" * 60)

    # Open file for appending (read-write)
    appender = ExodusAppender.append("mesh_with_nodesets.exo")

    # Convert nodeset 10 to sideset 100 and write it
    appender.create_sideset_from_nodeset(
        nodeset_id=10,
        new_sideset_id=100
    )

    print("✓ Converted nodeset 10 to sideset 100 and wrote to file")

    # Close and save
    appender.close()
    print()

    # Verify it was written
    reader = ExodusReader.open("mesh_with_nodesets.exo")
    sideset = reader.get_side_set(100)
    print(f"✓ Verified sideset {sideset.id} was written")
    print(f"  - Number of faces: {len(sideset.elements)}")
    reader.close()
    print()


def example_multiple_conversions():
    """
    Example 3: Convert multiple nodesets

    Demonstrates converting multiple nodesets to sidesets in one session.
    """
    print("=" * 60)
    print("Example 3: Multiple nodeset conversions")
    print("=" * 60)

    appender = ExodusAppender.append("mesh_with_nodesets.exo")

    # Convert several nodesets
    conversions = [
        (10, 100, "Top surface"),
        (20, 200, "Bottom surface"),
        (30, 300, "Left surface"),
        (40, 400, "Right surface"),
    ]

    for nodeset_id, sideset_id, description in conversions:
        try:
            appender.create_sideset_from_nodeset(nodeset_id, sideset_id)
            print(f"✓ Converted nodeset {nodeset_id} → sideset {sideset_id} ({description})")
        except Exception as e:
            print(f"✗ Failed to convert nodeset {nodeset_id}: {e}")

    appender.close()
    print()


def example_auto_increment_id():
    """
    Example 4: Auto-increment sideset IDs (NEW!)

    Demonstrates automatic ID assignment for sidesets.
    The sideset ID is automatically assigned as the max existing ID + 1.
    """
    print("=" * 60)
    print("Example 4: Auto-increment sideset IDs")
    print("=" * 60)

    appender = ExodusAppender.append("mesh_with_nodesets.exo")

    # Convert nodesets with auto-assigned IDs
    id1 = appender.create_sideset_from_nodeset_auto(10)
    print(f"✓ Converted nodeset 10 → sideset {id1} (auto-assigned)")

    id2 = appender.create_sideset_from_nodeset_auto(20)
    print(f"✓ Converted nodeset 20 → sideset {id2} (auto-assigned)")

    appender.close()
    print()


def example_name_based_conversion():
    """
    Example 5: Name-based conversion (NEW!)

    Demonstrates looking up nodesets by name and copying names to sidesets.
    """
    print("=" * 60)
    print("Example 5: Name-based conversion")
    print("=" * 60)

    appender = ExodusAppender.append("mesh_with_nodesets.exo")

    # Convert nodeset by name (if the nodeset has a name)
    # The name is automatically copied to the sideset
    try:
        sideset_id = appender.create_sideset_from_nodeset_by_name("inlet")
        print(f"✓ Converted nodeset 'inlet' → sideset {sideset_id}")
        print("  (Name was automatically copied to the sideset)")
    except Exception as e:
        print(f"✗ Could not convert by name: {e}")
        print("  (Nodeset may not have a name defined)")

    # Or create a sideset with an explicit name
    sideset_id = appender.create_sideset_from_nodeset_named(10, "outlet")
    print(f"✓ Converted nodeset 10 → sideset {sideset_id} named 'outlet'")

    appender.close()
    print()


def example_with_context_manager():
    """
    Example 6: Using Python context manager ('with' statement)

    Demonstrates the Pythonic way to handle file operations.
    """
    print("=" * 60)
    print("Example 6: Using context manager")
    print("=" * 60)

    # File automatically closes when leaving 'with' block
    with ExodusAppender.append("mesh_with_nodesets.exo") as appender:
        # Use the auto-increment version for convenience
        sideset_id = appender.create_sideset_from_nodeset_auto(10)
        print(f"✓ Converted to sideset {sideset_id} (file will auto-close)")

    print()


def example_inspect_results():
    """
    Example 7: Inspect conversion results in detail

    Shows how to examine the generated sideset in detail.
    """
    print("=" * 60)
    print("Example 7: Inspect conversion results")
    print("=" * 60)

    reader = ExodusReader.open("mesh_with_nodesets.exo")

    # Get original nodeset
    nodeset = reader.get_node_set(10)
    print(f"Original NodeSet {nodeset.id}:")
    print(f"  - Number of nodes: {len(nodeset.nodes)}")
    print(f"  - Node IDs: {nodeset.nodes[:10]}...")
    print()

    # Convert to sideset
    sideset = reader.convert_nodeset_to_sideset(10, 100)
    print(f"Converted SideSet {sideset.id}:")
    print(f"  - Number of faces: {len(sideset.elements)}")
    print()

    # Show element-side pairs
    print("Element-Side pairs (first 10):")
    for i, (elem, side) in enumerate(zip(sideset.elements[:10], sideset.sides[:10])):
        print(f"  {i+1}. Element {elem}, Side {side}")

    reader.close()
    print()


def main():
    """
    Main function demonstrating all examples.

    Note: These examples assume you have a file 'mesh_with_nodesets.exo'
    with appropriate nodesets defined. Adjust filenames and IDs as needed.
    """
    print("\n")
    print("╔" + "=" * 58 + "╗")
    print("║" + " " * 10 + "NodeSet to SideSet Conversion Examples" + " " * 9 + "║")
    print("╚" + "=" * 58 + "╝")
    print("\n")

    try:
        # Run examples (comment out as needed)
        example_read_and_convert()
        example_convert_and_write()
        example_multiple_conversions()
        example_auto_increment_id()        # NEW!
        example_name_based_conversion()    # NEW!
        example_with_context_manager()
        example_inspect_results()

        print("=" * 60)
        print("All examples completed successfully!")
        print("=" * 60)

    except FileNotFoundError:
        print("\n⚠ Error: Example file 'mesh_with_nodesets.exo' not found.")
        print("   Create a mesh file with nodesets to run these examples.")
        print("   Or modify the script to use your own mesh file.")
    except Exception as e:
        print(f"\n✗ Error: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
