"""
Example showing how to read an existing Exodus mesh file.
"""

try:
    from exodus import ExodusReader

    print("Reading an Exodus mesh file...")
    print("=" * 60)

    # First, check if we have a file to read
    # (This example assumes simple_mesh.py was run first)
    mesh_path = "/tmp/simple_quad.exo"

    try:
        with ExodusReader.open(mesh_path) as reader:
            # Get initialization parameters
            params = reader.init_params()

            print(f"\n📄 File: {reader.path()}")
            print(f"   Title: {params.title}")
            print(f"   Dimensions: {params.num_dim}D")
            print(f"\n📊 Mesh Statistics:")
            print(f"   Nodes: {params.num_nodes}")
            print(f"   Elements: {params.num_elems}")
            print(f"   Element Blocks: {params.num_elem_blocks}")
            print(f"   Node Sets: {params.num_node_sets}")
            print(f"   Side Sets: {params.num_side_sets}")

            # Get coordinates
            x, y, z = reader.get_coords()
            print(f"\n🗺️  Coordinates:")
            print(f"   X: {x}")
            print(f"   Y: {y}")
            if z and any(z):
                print(f"   Z: {z}")

            # Get coordinate names if available
            try:
                coord_names = reader.get_coord_names()
                print(f"   Names: {coord_names}")
            except:
                pass

            # Get block IDs
            block_ids = reader.get_block_ids()
            print(f"\n🧱 Element Blocks:")
            for block_id in block_ids:
                block = reader.get_block(block_id)
                print(f"   Block {block_id}:")
                print(f"      Topology: {block.topology}")
                print(f"      Elements: {block.num_entries}")
                print(f"      Nodes/Element: {block.num_nodes_per_entry}")

                # Get connectivity
                conn = reader.get_connectivity(block_id)
                print(f"      Connectivity: {conn}")

                # Get attributes if any
                try:
                    attrs = reader.get_block_attributes(block_id)
                    if attrs:
                        print(f"      Attributes: {attrs}")
                        attr_names = reader.get_block_attribute_names(block_id)
                        print(f"      Attribute Names: {attr_names}")
                except:
                    pass

            # Get QA records if available
            try:
                qa_records = reader.get_qa_records()
                if qa_records:
                    print(f"\n📝 QA Records:")
                    for qa in qa_records:
                        print(f"   {qa.code_name} v{qa.code_version} ({qa.date} {qa.time})")
            except:
                pass

            # Get info records if available
            try:
                info_records = reader.get_info_records()
                if info_records:
                    print(f"\n💬 Info Records:")
                    for info in info_records:
                        print(f"   {info}")
            except:
                pass

        print(f"\n✅ Successfully read mesh from {mesh_path}")

    except FileNotFoundError:
        print(f"\n❌ File not found: {mesh_path}")
        print("\nTo create a test file, run:")
        print("    python examples/simple_mesh.py")
    except Exception as e:
        print(f"\n❌ Error reading mesh: {e}")
        import traceback
        traceback.print_exc()

except ImportError as e:
    print(f"❌ Failed to import exodus module: {e}")
    print("\nTo build and install:")
    print("    cd rust/exodus-py")
    print("    pip install maturin")
    print("    maturin develop")
