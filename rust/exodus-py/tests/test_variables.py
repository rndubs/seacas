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

        # Write time series data using the same writer (0-based step indexing)
        # Write time step 0 - for global vars: entity_id=0, var_index=0 for first var
        writer.put_time(0, 0.0)
        writer.put_var(0, EntityType.Global, 0, 0, [100.0])  # Temperature
        writer.put_var(0, EntityType.Global, 0, 1, [200.0])  # Pressure

        # Write time step 1
        writer.put_time(1, 1.0)
        writer.put_var(1, EntityType.Global, 0, 0, [110.0])
        writer.put_var(1, EntityType.Global, 0, 1, [210.0])
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.variable_names(EntityType.Global)
        assert names == var_names

        # Read time step 0 - read each variable separately
        temp1 = reader.var(0, EntityType.Global, 0, 0)
        press1 = reader.var(0, EntityType.Global, 0, 1)
        assert temp1 == pytest.approx([100.0], abs=1e-6)
        assert press1 == pytest.approx([200.0], abs=1e-6)

        # Read time step 1
        temp2 = reader.var(1, EntityType.Global, 0, 0)
        press2 = reader.var(1, EntityType.Global, 0, 1)
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

        # Write time step data using the same writer (0-based step indexing)
        writer.put_time(0, 0.0)

        # Write nodal variable for all 4 nodes
        # For nodal vars: entity_id=0, var_index=0 for first variable
        displacements = [0.1, 0.2, 0.3, 0.4]
        writer.put_var(0, EntityType.Nodal, 0, 0, displacements)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.variable_names(EntityType.Nodal)
        assert names == var_names

        vals = reader.var(0, EntityType.Nodal, 0, 0)
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

        # Write time step data using the same writer (0-based step indexing)
        writer.put_time(0, 0.0)

        # Write element variable for block 1
        # For element vars: entity_id=block_id, var_index=0 for first variable
        stresses = [100.0, 200.0]
        writer.put_var(0, EntityType.ElemBlock, 1, 0, stresses)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.variable_names(EntityType.ElemBlock)
        assert names == var_names

        vals = reader.var(0, EntityType.ElemBlock, 1, 0)
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

        # Write multiple time steps using the same writer (0-based indexing)
        for i in range(5):
            writer.put_time(i, float(i) * 0.1)
            writer.put_var(i, EntityType.Global, 0, 0, [float(i * 10)])
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        num_steps = reader.num_time_steps()
        assert num_steps == 5

        times = reader.times()
        assert len(times) == 5
        assert times == pytest.approx([0.0, 0.1, 0.2, 0.3, 0.4], abs=1e-6)

        # Check each time step (0-based indexing)
        for i in range(5):
            vals = reader.var(i, EntityType.Global, 0, 0)
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

        # Write data using the same writer (0-based step indexing)
        writer.put_time(0, 0.0)

        # Write each variable separately (var_index 0, 1, 2)
        writer.put_var(0, EntityType.Nodal, 0, 0, [0.1, 0.2, 0.3, 0.4])
        writer.put_var(0, EntityType.Nodal, 0, 1, [0.5, 0.6, 0.7, 0.8])
        writer.put_var(0, EntityType.Nodal, 0, 2, [100.0, 200.0, 300.0, 400.0])
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.variable_names(EntityType.Nodal)
        assert len(names) == 3
        assert names == var_names

        # Verify each variable
        var1 = reader.var(0, EntityType.Nodal, 0, 0)
        var2 = reader.var(0, EntityType.Nodal, 0, 1)
        var3 = reader.var(0, EntityType.Nodal, 0, 2)

        assert var1 == pytest.approx([0.1, 0.2, 0.3, 0.4], abs=1e-6)
        assert var2 == pytest.approx([0.5, 0.6, 0.7, 0.8], abs=1e-6)
        assert var3 == pytest.approx([100.0, 200.0, 300.0, 400.0], abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_var_time_series_global():
    """Test multi-timestep variable retrieval for global variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Time Series Test", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        writer.define_variables(EntityType.Global, ["Energy", "Momentum"])

        # Write 10 time steps
        for i in range(10):
            writer.put_time(i, float(i) * 0.5)
            writer.put_var(i, EntityType.Global, 0, 0, [float(i * 10)])  # Energy
            writer.put_var(i, EntityType.Global, 0, 1, [float(i * 5)])   # Momentum
        writer.close()

        # Read back using var_time_series
        reader = ExodusReader.open(tmp_path)

        # Test reading all time steps at once
        energy_series = reader.var_time_series(0, 10, EntityType.Global, 0, 0)
        momentum_series = reader.var_time_series(0, 10, EntityType.Global, 0, 1)

        expected_energy = [float(i * 10) for i in range(10)]
        expected_momentum = [float(i * 5) for i in range(10)]

        assert len(energy_series) == 10
        assert len(momentum_series) == 10
        assert energy_series == pytest.approx(expected_energy, abs=1e-6)
        assert momentum_series == pytest.approx(expected_momentum, abs=1e-6)

        # Test reading partial time range
        energy_partial = reader.var_time_series(3, 7, EntityType.Global, 0, 0)
        expected_partial = [float(i * 10) for i in range(3, 7)]
        assert len(energy_partial) == 4
        assert energy_partial == pytest.approx(expected_partial, abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_var_time_series_nodal():
    """Test multi-timestep variable retrieval for nodal variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Nodal Time Series", num_dim=2, num_nodes=3)
        writer.put_init_params(params)

        writer.define_variables(EntityType.Nodal, ["Temperature"])

        # Write 5 time steps with varying nodal values
        for i in range(5):
            writer.put_time(i, float(i))
            # Temperature increases at each node over time
            temps = [100.0 + i * 10, 200.0 + i * 20, 300.0 + i * 30]
            writer.put_var(i, EntityType.Nodal, 0, 0, temps)
        writer.close()

        # Read back using var_time_series
        reader = ExodusReader.open(tmp_path)

        # Read all time steps at once - should get flattened array
        # [step0_node0, step0_node1, step0_node2, step1_node0, step1_node1, step1_node2, ...]
        temp_series = reader.var_time_series(0, 5, EntityType.Nodal, 0, 0)

        # Expected: 5 time steps * 3 nodes = 15 values
        expected = []
        for i in range(5):
            expected.extend([100.0 + i * 10, 200.0 + i * 20, 300.0 + i * 30])

        assert len(temp_series) == 15
        assert temp_series == pytest.approx(expected, abs=1e-6)

        # Test partial range: steps 1-3
        temp_partial = reader.var_time_series(1, 3, EntityType.Nodal, 0, 0)
        expected_partial = []
        for i in range(1, 3):
            expected_partial.extend([100.0 + i * 10, 200.0 + i * 20, 300.0 + i * 30])

        assert len(temp_partial) == 6  # 2 steps * 3 nodes
        assert temp_partial == pytest.approx(expected_partial, abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_var_time_series_element():
    """Test multi-timestep variable retrieval for element variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Element Time Series",
            num_dim=2,
            num_nodes=8,
            num_elems=2,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        block = Block(
            id=100,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=2,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        writer.define_variables(EntityType.ElemBlock, ["Stress", "Strain"])

        # Write 4 time steps
        for i in range(4):
            writer.put_time(i, float(i) * 0.25)
            stresses = [100.0 + i * 10, 200.0 + i * 20]
            strains = [0.01 + i * 0.01, 0.02 + i * 0.02]
            writer.put_var(i, EntityType.ElemBlock, 100, 0, stresses)
            writer.put_var(i, EntityType.ElemBlock, 100, 1, strains)
        writer.close()

        # Read back using var_time_series
        reader = ExodusReader.open(tmp_path)

        # Read stress over all time steps
        stress_series = reader.var_time_series(0, 4, EntityType.ElemBlock, 100, 0)

        expected_stress = []
        for i in range(4):
            expected_stress.extend([100.0 + i * 10, 200.0 + i * 20])

        assert len(stress_series) == 8  # 4 steps * 2 elements
        assert stress_series == pytest.approx(expected_stress, abs=1e-6)

        # Read strain for a subset of time steps
        strain_series = reader.var_time_series(1, 3, EntityType.ElemBlock, 100, 1)

        expected_strain = []
        for i in range(1, 3):
            expected_strain.extend([0.01 + i * 0.01, 0.02 + i * 0.02])

        assert len(strain_series) == 4  # 2 steps * 2 elements
        assert strain_series == pytest.approx(expected_strain, abs=1e-6)

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
