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
)


def test_global_variables():
    """Test global variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        # Create file and define global variables
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Global Vars", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Define 2 global variables
        var_names = ["Temperature", "Pressure"]
        writer.put_global_var_names(var_names)
        writer.close()

        # Open for appending to write time steps
        appender = ExodusAppender.open(tmp_path)

        # Write time step 1
        appender.put_time(1, 0.0)
        appender.put_global_vars(1, [100.0, 200.0])

        # Write time step 2
        appender.put_time(2, 1.0)
        appender.put_global_vars(2, [110.0, 210.0])
        appender.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.get_global_var_names()
        assert names == var_names

        # Read time step 1
        vals1 = reader.get_global_vars(1)
        assert vals1 == pytest.approx([100.0, 200.0], abs=1e-6)

        # Read time step 2
        vals2 = reader.get_global_vars(2)
        assert vals2 == pytest.approx([110.0, 210.0], abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_nodal_variables():
    """Test nodal variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Nodal Vars", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Define nodal variable
        var_names = ["Displacement"]
        writer.put_nodal_var_names(var_names)
        writer.close()

        # Write time step data
        appender = ExodusAppender.open(tmp_path)
        appender.put_time(1, 0.0)

        # Write nodal variable for all 4 nodes
        displacements = [0.1, 0.2, 0.3, 0.4]
        appender.put_nodal_var(1, 1, displacements)
        appender.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.get_nodal_var_names()
        assert names == var_names

        vals = reader.get_nodal_var(1, 1)
        assert vals == pytest.approx(displacements, abs=1e-6)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_element_variables():
    """Test element variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

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

        # Define element block
        writer.put_elem_block(1, "QUAD4", 2, 4, 0)

        # Define element variable
        var_names = ["Stress"]
        writer.put_elem_var_names(var_names)
        writer.close()

        # Write time step data
        appender = ExodusAppender.open(tmp_path)
        appender.put_time(1, 0.0)

        # Write element variable for block 1
        stresses = [100.0, 200.0]
        appender.put_elem_var(1, 1, 1, stresses)
        appender.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.get_elem_var_names()
        assert names == var_names

        vals = reader.get_elem_var(1, 1, 1)
        assert vals == pytest.approx(stresses, abs=1e-6)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_time_steps():
    """Test multiple time steps with variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Multi Time", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        writer.put_global_var_names(["Energy"])
        writer.close()

        # Write multiple time steps
        appender = ExodusAppender.open(tmp_path)
        for i in range(5):
            appender.put_time(i + 1, float(i) * 0.1)
            appender.put_global_vars(i + 1, [float(i * 10)])
        appender.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        num_steps = reader.num_time_steps()
        assert num_steps == 5

        times = reader.get_times()
        assert len(times) == 5
        assert times == pytest.approx([0.0, 0.1, 0.2, 0.3, 0.4], abs=1e-6)

        # Check each time step
        for i in range(5):
            vals = reader.get_global_vars(i + 1)
            assert vals == pytest.approx([float(i * 10)], abs=1e-6)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_nodal_variables():
    """Test multiple nodal variables"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Multi Nodal Vars", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Define multiple nodal variables
        var_names = ["DisplacementX", "DisplacementY", "Temperature"]
        writer.put_nodal_var_names(var_names)
        writer.close()

        # Write data
        appender = ExodusAppender.open(tmp_path)
        appender.put_time(1, 0.0)

        appender.put_nodal_var(1, 1, [0.1, 0.2, 0.3, 0.4])
        appender.put_nodal_var(1, 2, [0.5, 0.6, 0.7, 0.8])
        appender.put_nodal_var(1, 3, [100.0, 200.0, 300.0, 400.0])
        appender.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        names = reader.get_nodal_var_names()
        assert len(names) == 3
        assert names == var_names

        # Verify each variable
        var1 = reader.get_nodal_var(1, 1)
        var2 = reader.get_nodal_var(1, 2)
        var3 = reader.get_nodal_var(1, 3)

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

        # Define two element blocks
        writer.put_elem_block(1, "QUAD4", 1, 4, 0)
        writer.put_elem_block(2, "QUAD4", 1, 4, 0)

        # Define element variables
        writer.put_elem_var_names(["Stress", "Strain"])

        # Set truth table: Stress on block 1, Strain on block 2
        # Truth table is 2 variables x 2 blocks = 4 entries
        # [var1_block1, var1_block2, var2_block1, var2_block2]
        truth_table = [1, 0, 0, 1]  # Stress on block 1, Strain on block 2
        writer.put_elem_var_truth_table(truth_table)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        tt_read = reader.get_elem_var_truth_table()
        assert tt_read == truth_table
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
