"""
Integration tests combining multiple features
"""

import pytest
import tempfile
import os

pytest.importorskip("exodus")

from exodus import (
    ExodusWriter,
    ExodusReader,
    ExodusAppender,
    InitParams,
    MeshBuilder,
    BlockBuilder,
    EntityType,
    Block,
)


def test_complete_workflow():
    """Test complete workflow: create, write, append, read"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        # Step 1: Create file with mesh
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Complete Workflow",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
            num_node_sets=1,
        )
        writer.put_init_params(params)

        # Add coordinates
        writer.put_coords([0.0, 1.0, 1.0, 0.0], [0.0, 0.0, 1.0, 1.0], [])

        # Add element block using Block object
        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)
        writer.put_connectivity(1, [1, 2, 3, 4])

        # Add node set
        writer.put_node_set(1, [1, 2], None)

        # Add metadata
        writer.put_info_records(["Test mesh"])
        # Note: QA records not yet implemented in exodus-rs

        # Define variables using modern generic API
        writer.define_variables(EntityType.Global, ["Energy"])
        writer.define_variables(EntityType.Nodal, ["Temperature"])

        writer.close()

        # Step 2: Append time step data
        appender = ExodusAppender.append(tmp_path)
        appender.put_time(1, 0.0)
        appender.put_var(1, EntityType.Global, 0, 0, [100.0])
        appender.put_var(1, EntityType.Nodal, 0, 0, [20.0, 21.0, 22.0, 23.0])
        appender.close()

        # Step 3: Read and verify everything
        reader = ExodusReader.open(tmp_path)

        # Verify init params
        params_read = reader.init_params()
        assert params_read.title == "Complete Workflow"
        assert params_read.num_nodes == 4
        assert params_read.num_elems == 1

        # Verify coordinates
        x, y, z = reader.get_coords()
        assert len(x) == 4
        assert x == pytest.approx([0.0, 1.0, 1.0, 0.0], abs=1e-6)

        # Verify element block
        block_read = reader.get_block(1)
        assert block_read.topology == "QUAD4"
        conn = reader.get_connectivity(1)
        assert conn == [1, 2, 3, 4]

        # Verify node set
        node_set = reader.get_node_set(1)
        assert node_set.nodes == [1, 2]

        # Verify info records
        info_records = reader.get_info_records()
        assert len(info_records) == 1
        assert info_records[0] == "Test mesh"

        # Note: QA records not yet implemented for reading
        # qa_records = reader.get_qa_records()
        # assert len(qa_records) == 1

        # Verify variables using modern generic API
        global_var = reader.var(1, EntityType.Global, 0, 0)
        assert global_var == pytest.approx([100.0], abs=1e-6)

        nodal_vars = reader.var(1, EntityType.Nodal, 0, 0)
        assert nodal_vars == pytest.approx([20.0, 21.0, 22.0, 23.0], abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_builder_and_reader_integration():
    """Test integration between builder API and reader"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so MeshBuilder can create it
    os.unlink(tmp_path)

    try:
        # Create with builder API
        (
            MeshBuilder("Builder Integration")
            .dimensions(2)
            .coordinates(
                x=[0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
                y=[0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                z=[],
            )
            .add_block(
                BlockBuilder(1, "QUAD4")
                .connectivity([1, 2, 5, 4])
                .build()
            )
            .add_block(
                BlockBuilder(2, "QUAD4")
                .connectivity([2, 3, 6, 5])
                .build()
            )
            .info("Two quad elements")
            .write(tmp_path)
        )

        # Read with low-level API
        reader = ExodusReader.open(tmp_path)

        params = reader.init_params()
        assert params.num_nodes == 6
        assert params.num_elems == 2
        assert params.num_elem_blocks == 2

        # Verify coordinates
        x, y, z = reader.get_coords()
        assert len(x) == 6
        assert x == pytest.approx([0.0, 1.0, 2.0, 0.0, 1.0, 2.0], abs=1e-6)

        # Verify blocks
        block_ids = reader.get_block_ids()
        assert 1 in block_ids
        assert 2 in block_ids

        conn1 = reader.get_connectivity(1)
        conn2 = reader.get_connectivity(2)
        assert conn1 == [1, 2, 5, 4]
        assert conn2 == [2, 3, 6, 5]

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multi_timestep_workflow():
    """Test workflow with multiple time steps"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        # Create file
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Multi Time", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        writer.put_coords([0.0, 1.0, 1.0, 0.0], [0.0, 0.0, 1.0, 1.0], [])

        # Define variables using modern generic API
        writer.define_variables(EntityType.Global, ["Energy", "Momentum"])
        writer.define_variables(EntityType.Nodal, ["Temperature"])
        writer.close()

        # Write multiple time steps
        appender = ExodusAppender.append(tmp_path)
        for step in range(1, 11):
            time_val = float(step) * 0.1
            appender.put_time(step, time_val)
            # Write global vars separately (var_index 0 and 1)
            appender.put_var(step, EntityType.Global, 0, 0, [float(step)])
            appender.put_var(step, EntityType.Global, 0, 1, [float(step * 2)])
            # Write nodal var
            appender.put_var(
                step, EntityType.Nodal, 0, 0, [float(step + i) for i in range(4)]
            )
        appender.close()

        # Read and verify
        reader = ExodusReader.open(tmp_path)

        assert reader.num_time_steps() == 10

        times = reader.times()
        assert len(times) == 10
        assert times[0] == pytest.approx(0.1, abs=1e-6)
        assert times[9] == pytest.approx(1.0, abs=1e-6)

        # Verify first time step - read each global var separately
        energy1 = reader.var(1, EntityType.Global, 0, 0)
        momentum1 = reader.var(1, EntityType.Global, 0, 1)
        assert energy1 == pytest.approx([1.0], abs=1e-6)
        assert momentum1 == pytest.approx([2.0], abs=1e-6)

        # Verify last time step
        energy10 = reader.var(10, EntityType.Global, 0, 0)
        momentum10 = reader.var(10, EntityType.Global, 0, 1)
        assert energy10 == pytest.approx([10.0], abs=1e-6)
        assert momentum10 == pytest.approx([20.0], abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_complex_mesh_with_sets():
    """Test complex mesh with multiple blocks and sets"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Complex Mesh",
            num_dim=2,
            num_nodes=12,
            num_elems=4,
            num_elem_blocks=2,
            num_node_sets=2,
            num_side_sets=1,
            num_elem_sets=1,
        )
        writer.put_init_params(params)

        # Define blocks using Block objects
        block1 = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=2,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        block2 = Block(
            id=2,
            entity_type=EntityType.ElemBlock,
            topology="TRI3",
            num_entries=2,
            num_nodes_per_entry=3,
            num_attributes=0,
        )
        writer.put_block(block1)
        writer.put_block(block2)

        # Define sets
        writer.put_node_set(1, [1, 2, 3], None)
        writer.put_node_set(2, [10, 11, 12], None)
        writer.put_side_set(1, [1, 2], [1, 1], None)
        writer.put_entity_set(EntityType.ElemSet, 1, [1, 3])

        # Name everything using generic naming API
        writer.put_name("elem_block", 0, "QuadRegion")  # Block ID 1 is at index 0
        writer.put_name("elem_block", 1, "TriRegion")   # Block ID 2 is at index 1
        writer.put_name("node_set", 0, "LeftBoundary")
        writer.put_name("node_set", 1, "RightBoundary")
        writer.put_name("side_set", 0, "TopBoundary")
        writer.put_name("elem_set", 0, "MaterialA")

        writer.close()

        # Read and verify
        reader = ExodusReader.open(tmp_path)

        # Verify blocks
        block1_read = reader.get_block(1)
        block2_read = reader.get_block(2)
        assert block1_read.num_entries == 2
        assert block2_read.num_entries == 2

        # Verify block names
        assert reader.get_name("elem_block", 0) == "QuadRegion"
        assert reader.get_name("elem_block", 1) == "TriRegion"

        # Verify sets
        node_set_1 = reader.get_node_set(1)
        node_set_2 = reader.get_node_set(2)
        assert len(node_set_1.nodes) == 3
        assert len(node_set_2.nodes) == 3

        # Verify set names
        assert reader.get_name("node_set", 0) == "LeftBoundary"
        assert reader.get_name("side_set", 0) == "TopBoundary"
        assert reader.get_name("elem_set", 0) == "MaterialA"

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
