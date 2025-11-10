"""
Tests for variable operations (global, nodal, element variables)
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
    EntityType,
    Block,
    TruthTable,
)


def test_global_variables():
    """Test global variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        # Create file and define global variables
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Global Vars", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Define 2 global variables using modern generic API
        var_names = ["Temperature", "Pressure"]
        writer.define_variables(EntityType.Global, var_names)
        writer.close()

        # Open for appending to write time steps
        appender = ExodusAppender.append(tmp_path)

        # Write time step 1 - for global vars: entity_id=0, var_index=0 for first var
        appender.put_time(1, 0.0)
        appender.put_var(1, EntityType.Global, 0, 0, [100.0])  # Temperature
        appender.put_var(1, EntityType.Global, 0, 1, [200.0])  # Pressure

        # Write time step 2
        appender.put_time(2, 1.0)
        appender.put_var(2, EntityType.Global, 0, 0, [110.0])
        appender.put_var(2, EntityType.Global, 0, 1, [210.0])
        appender.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.variable_names(EntityType.Global)
        assert names == var_names

        # Read time step 1 - read each variable separately
        temp1 = reader.var(1, EntityType.Global, 0, 0)
        press1 = reader.var(1, EntityType.Global, 0, 1)
        assert temp1 == pytest.approx([100.0], abs=1e-6)
        assert press1 == pytest.approx([200.0], abs=1e-6)

        # Read time step 2
        temp2 = reader.var(2, EntityType.Global, 0, 0)
        press2 = reader.var(2, EntityType.Global, 0, 1)
        assert temp2 == pytest.approx([110.0], abs=1e-6)
        assert press2 == pytest.approx([210.0], abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_nodal_variables():
    """Test nodal variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Nodal Vars", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Define nodal variable using modern generic API
        var_names = ["Displacement"]
        writer.define_variables(EntityType.Nodal, var_names)
        writer.close()

        # Write time step data
        appender = ExodusAppender.append(tmp_path)
        appender.put_time(1, 0.0)

        # Write nodal variable for all 4 nodes
        # For nodal vars: entity_id=0, var_index=0 for first variable
        displacements = [0.1, 0.2, 0.3, 0.4]
        appender.put_var(1, EntityType.Nodal, 0, 0, displacements)
        appender.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.variable_names(EntityType.Nodal)
        assert names == var_names

        vals = reader.var(1, EntityType.Nodal, 0, 0)
        assert vals == pytest.approx(displacements, abs=1e-6)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_element_variables():
    """Test element variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Elem Vars",
            num_dim=2,
            num_nodes=4,
            num_elems=2,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        # Define element block using Block object
        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=2,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Define element variable using modern generic API
        var_names = ["Stress"]
        writer.define_variables(EntityType.ElemBlock, var_names)
        writer.close()

        # Write time step data
        appender = ExodusAppender.append(tmp_path)
        appender.put_time(1, 0.0)

        # Write element variable for block 1
        # For element vars: entity_id=block_id, var_index=0 for first variable
        stresses = [100.0, 200.0]
        appender.put_var(1, EntityType.ElemBlock, 1, 0, stresses)
        appender.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.variable_names(EntityType.ElemBlock)
        assert names == var_names

        vals = reader.var(1, EntityType.ElemBlock, 1, 0)
        assert vals == pytest.approx(stresses, abs=1e-6)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_time_steps():
    """Test multiple time steps with variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Multi Time", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        writer.define_variables(EntityType.Global, ["Energy"])
        writer.close()

        # Write multiple time steps
        appender = ExodusAppender.append(tmp_path)
        for i in range(5):
            appender.put_time(i + 1, float(i) * 0.1)
            appender.put_var(i + 1, EntityType.Global, 0, 0, [float(i * 10)])
        appender.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        num_steps = reader.num_time_steps()
        assert num_steps == 5

        times = reader.times()
        assert len(times) == 5
        assert times == pytest.approx([0.0, 0.1, 0.2, 0.3, 0.4], abs=1e-6)

        # Check each time step
        for i in range(5):
            vals = reader.var(i + 1, EntityType.Global, 0, 0)
            assert vals == pytest.approx([float(i * 10)], abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_nodal_variables():
    """Test multiple nodal variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Multi Nodal Vars", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Define multiple nodal variables
        var_names = ["DisplacementX", "DisplacementY", "Temperature"]
        writer.define_variables(EntityType.Nodal, var_names)
        writer.close()

        # Write data
        appender = ExodusAppender.append(tmp_path)
        appender.put_time(1, 0.0)

        # Write each variable separately (var_index 0, 1, 2)
        appender.put_var(1, EntityType.Nodal, 0, 0, [0.1, 0.2, 0.3, 0.4])
        appender.put_var(1, EntityType.Nodal, 0, 1, [0.5, 0.6, 0.7, 0.8])
        appender.put_var(1, EntityType.Nodal, 0, 2, [100.0, 200.0, 300.0, 400.0])
        appender.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.variable_names(EntityType.Nodal)
        assert len(names) == 3
        assert names == var_names

        # Verify each variable
        var1 = reader.var(1, EntityType.Nodal, 0, 0)
        var2 = reader.var(1, EntityType.Nodal, 0, 1)
        var3 = reader.var(1, EntityType.Nodal, 0, 2)

        assert var1 == pytest.approx([0.1, 0.2, 0.3, 0.4], abs=1e-6)
        assert var2 == pytest.approx([0.5, 0.6, 0.7, 0.8], abs=1e-6)
        assert var3 == pytest.approx([100.0, 200.0, 300.0, 400.0], abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_element_variable_truth_table():
    """Test element variable truth table with multiple blocks"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Truth Table",
            num_dim=2,
            num_nodes=8,
            num_elems=2,
            num_elem_blocks=2,
        )
        writer.put_init_params(params)

        # Define two element blocks using Block objects
        block1 = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        block2 = Block(
            id=2,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block1)
        writer.put_block(block2)

        # Define element variables
        writer.define_variables(EntityType.ElemBlock, ["Stress", "Strain"])

        # Create truth table: Stress on block 1, Strain on block 2
        # num_entities=2 blocks, num_variables=2 vars
        truth_table = TruthTable.new(EntityType.ElemBlock, 2, 2)
        # Set which blocks have which variables (0-indexed)
        truth_table.set(0, 0, True)   # Block 1 (idx 0) has Stress (var 0)
        truth_table.set(0, 1, False)  # Block 1 doesn't have Strain
        truth_table.set(1, 0, False)  # Block 2 doesn't have Stress
        truth_table.set(1, 1, True)   # Block 2 (idx 1) has Strain (var 1)

        writer.put_truth_table(EntityType.ElemBlock, truth_table)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        tt_read = reader.truth_table(EntityType.ElemBlock)

        # Verify the truth table entries
        assert tt_read.get(0, 0) == True   # Block 1 has Stress
        assert tt_read.get(0, 1) == False  # Block 1 doesn't have Strain
        assert tt_read.get(1, 0) == False  # Block 2 doesn't have Stress
        assert tt_read.get(1, 1) == True   # Block 2 has Strain

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
