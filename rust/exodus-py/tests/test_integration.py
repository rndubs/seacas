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
)


def test_complete_workflow():
    """Test complete workflow: create, write, append, read"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

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

        # Add element block
        writer.put_elem_block(1, "QUAD4", 1, 4, 0)
        writer.put_elem_connectivity(1, [1, 2, 3, 4])

        # Add node set
        writer.put_node_set(1, [1, 2])

        # Add metadata
        writer.put_qa(1, "TestCode", "1.0", "2025-01-15", "12:00:00")
        writer.put_info(1, "Test mesh")

        # Define variables
        writer.put_global_var_names(["Energy"])
        writer.put_nodal_var_names(["Temperature"])

        writer.close()

        # Step 2: Append time step data
        appender = ExodusAppender.open(tmp_path)
        appender.put_time(1, 0.0)
        appender.put_global_vars(1, [100.0])
        appender.put_nodal_var(1, 1, [20.0, 21.0, 22.0, 23.0])
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
        block = reader.get_elem_block(1)
        assert block.entity_type == "QUAD4"
        conn = reader.get_elem_connectivity(1)
        assert conn == [1, 2, 3, 4]

        # Verify node set
        node_set = reader.get_node_set(1)
        assert node_set == [1, 2]

        # Verify metadata
        qa_records = reader.get_qa_records()
        assert len(qa_records) == 1
        assert qa_records[0].code_name == "TestCode"

        info_records = reader.get_info_records()
        assert len(info_records) == 1
        assert info_records[0] == "Test mesh"

        # Verify variables
        global_vars = reader.get_global_vars(1)
        assert global_vars == pytest.approx([100.0], abs=1e-6)

        nodal_vars = reader.get_nodal_var(1, 1)
        assert nodal_vars == pytest.approx([20.0, 21.0, 22.0, 23.0], abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_builder_and_reader_integration():
    """Test integration between builder API and reader"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

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
            .qa_record("Builder", "1.0", "2025-01-15", "12:00:00")
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
        block_ids = reader.get_elem_block_ids()
        assert 1 in block_ids
        assert 2 in block_ids

        conn1 = reader.get_elem_connectivity(1)
        conn2 = reader.get_elem_connectivity(2)
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

    try:
        # Create file
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Multi Time", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        writer.put_coords([0.0, 1.0, 1.0, 0.0], [0.0, 0.0, 1.0, 1.0], [])
        writer.put_global_var_names(["Energy", "Momentum"])
        writer.put_nodal_var_names(["Temperature"])
        writer.close()

        # Write multiple time steps
        appender = ExodusAppender.open(tmp_path)
        for step in range(1, 11):
            time_val = float(step) * 0.1
            appender.put_time(step, time_val)
            appender.put_global_vars(step, [float(step), float(step * 2)])
            appender.put_nodal_var(
                step, 1, [float(step + i) for i in range(4)]
            )
        appender.close()

        # Read and verify
        reader = ExodusReader.open(tmp_path)

        assert reader.num_time_steps() == 10

        times = reader.get_times()
        assert len(times) == 10
        assert times[0] == pytest.approx(0.1, abs=1e-6)
        assert times[9] == pytest.approx(1.0, abs=1e-6)

        # Verify first time step
        global_vars = reader.get_global_vars(1)
        assert global_vars == pytest.approx([1.0, 2.0], abs=1e-6)

        # Verify last time step
        global_vars = reader.get_global_vars(10)
        assert global_vars == pytest.approx([10.0, 20.0], abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_complex_mesh_with_sets():
    """Test complex mesh with multiple blocks and sets"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

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

        # Define blocks
        writer.put_elem_block(1, "QUAD4", 2, 4, 0)
        writer.put_elem_block(2, "TRI3", 2, 3, 0)

        # Define sets
        writer.put_node_set(1, [1, 2, 3])
        writer.put_node_set(2, [10, 11, 12])
        writer.put_side_set(1, [1, 2], [1, 1])
        writer.put_elem_set(1, [1, 3])

        # Name everything
        writer.put_elem_block_name(1, "QuadRegion")
        writer.put_elem_block_name(2, "TriRegion")
        writer.put_node_set_name(1, "LeftBoundary")
        writer.put_node_set_name(2, "RightBoundary")
        writer.put_side_set_name(1, "TopBoundary")
        writer.put_elem_set_name(1, "MaterialA")

        writer.close()

        # Read and verify
        reader = ExodusReader.open(tmp_path)

        # Verify blocks
        block1 = reader.get_elem_block(1)
        block2 = reader.get_elem_block(2)
        assert block1.num_entries == 2
        assert block2.num_entries == 2

        # Verify block names
        assert reader.get_elem_block_name(1) == "QuadRegion"
        assert reader.get_elem_block_name(2) == "TriRegion"

        # Verify sets
        assert len(reader.get_node_set(1)) == 3
        assert len(reader.get_node_set(2)) == 3

        # Verify set names
        assert reader.get_node_set_name(1) == "LeftBoundary"
        assert reader.get_side_set_name(1) == "TopBoundary"
        assert reader.get_elem_set_name(1) == "MaterialA"

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
