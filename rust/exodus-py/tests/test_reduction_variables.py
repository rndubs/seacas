"""Tests for reduction variables functionality

Reduction variables store aggregated/summary values for entire objects
(e.g., assemblies, blocks, sets) rather than for individual entities within those objects.
"""

import pytest
from exodus import (
    ExodusWriter,
    ExodusReader,
    EntityType,
    InitParams,
    Block,
    Assembly,
)


def test_assembly_reduction_variables(tmp_path):
    """Test reduction variables on assemblies"""
    file_path = tmp_path / "assembly_reduction.exo"

    # Write
    writer = ExodusWriter.create(str(file_path))

    params = InitParams(
        title="Assembly Reduction Test",
        num_dim=3,
        num_nodes=3,
        num_elems=1,
        num_elem_blocks=1,
        num_assemblies=2,
    )
    writer.put_init_params(params)

    writer.put_coords([0.0, 1.0, 2.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0])

    block = Block(
        id=10,
        entity_type=EntityType.ElemBlock,
        topology="tri",
        num_entries=1,
        num_nodes_per_entry=3,
        num_edges_per_entry=0,
        num_faces_per_entry=0,
        num_attributes=0,
    )
    writer.put_block(block)
    writer.put_connectivity(10, [1, 2, 3])

    assembly1 = Assembly(
        id=100,
        name="Assembly_A",
        entity_type=EntityType.ElemBlock,
        entity_list=[10],
    )
    writer.put_assembly(assembly1)

    assembly2 = Assembly(
        id=200,
        name="Assembly_B",
        entity_type=EntityType.ElemBlock,
        entity_list=[10],
    )
    writer.put_assembly(assembly2)

    # Define reduction variables
    writer.define_reduction_variables(
        EntityType.Assembly,
        ["Momentum_X", "Momentum_Y", "Momentum_Z", "Kinetic_Energy"],
    )

    # Write time steps
    for ts in range(3):
        time_val = (ts + 1) * 0.1
        writer.put_time(ts, time_val)

        # Assembly 100
        values_100 = [
            time_val * 1.0,  # Momentum_X
            time_val * 2.0,  # Momentum_Y
            time_val * 3.0,  # Momentum_Z
            time_val * 10.0,  # Kinetic_Energy
        ]
        writer.put_reduction_vars(ts, EntityType.Assembly, 100, values_100)

        # Assembly 200
        values_200 = [
            time_val * 4.0,  # Momentum_X
            time_val * 5.0,  # Momentum_Y
            time_val * 6.0,  # Momentum_Z
            time_val * 20.0,  # Kinetic_Energy
        ]
        writer.put_reduction_vars(ts, EntityType.Assembly, 200, values_200)

    writer.close()

    # Read
    reader = ExodusReader.open(str(file_path))

    # Check variable names
    names = reader.reduction_variable_names(EntityType.Assembly)
    assert len(names) == 4
    assert names[0] == "Momentum_X"
    assert names[1] == "Momentum_Y"
    assert names[2] == "Momentum_Z"
    assert names[3] == "Kinetic_Energy"

    # Check values for assembly 100 at time step 0
    values = reader.get_reduction_vars(0, EntityType.Assembly, 100)
    assert len(values) == 4
    assert abs(values[0] - 0.1) < 1e-10
    assert abs(values[1] - 0.2) < 1e-10
    assert abs(values[2] - 0.3) < 1e-10
    assert abs(values[3] - 1.0) < 1e-10

    # Check values for assembly 200 at time step 2
    values = reader.get_reduction_vars(2, EntityType.Assembly, 200)
    assert len(values) == 4
    assert abs(values[0] - 1.2) < 1e-10
    assert abs(values[1] - 1.5) < 1e-10
    assert abs(values[2] - 1.8) < 1e-10
    assert abs(values[3] - 6.0) < 1e-10

    reader.close()


def test_element_block_reduction_variables(tmp_path):
    """Test reduction variables on element blocks"""
    file_path = tmp_path / "elem_block_reduction.exo"

    # Write
    writer = ExodusWriter.create(str(file_path))

    params = InitParams(
        title="Element Block Reduction Test",
        num_dim=3,
        num_nodes=3,
        num_elems=6,
        num_elem_blocks=2,
    )
    writer.put_init_params(params)

    writer.put_coords([0.0, 1.0, 2.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0])

    # Block 1: 2 triangles
    block1 = Block(
        id=10,
        entity_type=EntityType.ElemBlock,
        topology="tri",
        num_entries=2,
        num_nodes_per_entry=3,
        num_edges_per_entry=0,
        num_faces_per_entry=0,
        num_attributes=0,
    )
    writer.put_block(block1)
    writer.put_connectivity(10, [1, 2, 3, 2, 3, 1])

    # Block 2: 4 triangles
    block2 = Block(
        id=20,
        entity_type=EntityType.ElemBlock,
        topology="tri",
        num_entries=4,
        num_nodes_per_entry=3,
        num_edges_per_entry=0,
        num_faces_per_entry=0,
        num_attributes=0,
    )
    writer.put_block(block2)
    writer.put_connectivity(20, [1, 2, 3, 2, 3, 1, 1, 2, 3, 2, 3, 1])

    # Define reduction variables
    writer.define_reduction_variables(EntityType.ElemBlock, ["AvgStrain", "MaxStress"])

    # Write time step
    writer.put_time(0, 0.0)
    writer.put_reduction_vars(0, EntityType.ElemBlock, 10, [0.01, 100.0])
    writer.put_reduction_vars(0, EntityType.ElemBlock, 20, [0.02, 200.0])

    writer.close()

    # Read
    reader = ExodusReader.open(str(file_path))

    names = reader.reduction_variable_names(EntityType.ElemBlock)
    assert names == ["AvgStrain", "MaxStress"]

    values_10 = reader.get_reduction_vars(0, EntityType.ElemBlock, 10)
    assert abs(values_10[0] - 0.01) < 1e-10
    assert abs(values_10[1] - 100.0) < 1e-10

    values_20 = reader.get_reduction_vars(0, EntityType.ElemBlock, 20)
    assert abs(values_20[0] - 0.02) < 1e-10
    assert abs(values_20[1] - 200.0) < 1e-10

    reader.close()


def test_node_set_reduction_variables(tmp_path):
    """Test reduction variables on node sets"""
    file_path = tmp_path / "node_set_reduction.exo"

    # Write
    writer = ExodusWriter.create(str(file_path))

    params = InitParams(
        title="Node Set Reduction Test",
        num_dim=3,
        num_nodes=3,
        num_elems=1,
        num_elem_blocks=1,
        num_node_sets=2,
    )
    writer.put_init_params(params)

    writer.put_coords([0.0, 1.0, 2.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0])

    block = Block(
        id=10,
        entity_type=EntityType.ElemBlock,
        topology="tri",
        num_entries=1,
        num_nodes_per_entry=3,
        num_edges_per_entry=0,
        num_faces_per_entry=0,
        num_attributes=0,
    )
    writer.put_block(block)
    writer.put_connectivity(10, [1, 2, 3])

    # Node sets
    writer.put_node_set(100, [1, 2])
    writer.put_node_set(200, [3])

    # Define reduction variables
    writer.define_reduction_variables(EntityType.NodeSet, ["MaxTemp", "AvgPress"])

    writer.put_time(0, 0.0)
    writer.put_reduction_vars(0, EntityType.NodeSet, 100, [300.0, 101.0])
    writer.put_reduction_vars(0, EntityType.NodeSet, 200, [350.0, 102.0])

    writer.close()

    # Read
    reader = ExodusReader.open(str(file_path))

    names = reader.reduction_variable_names(EntityType.NodeSet)
    assert names == ["MaxTemp", "AvgPress"]

    values = reader.get_reduction_vars(0, EntityType.NodeSet, 100)
    assert abs(values[0] - 300.0) < 1e-10
    assert abs(values[1] - 101.0) < 1e-10

    reader.close()


def test_side_set_reduction_variables(tmp_path):
    """Test reduction variables on side sets"""
    file_path = tmp_path / "side_set_reduction.exo"

    # Write
    writer = ExodusWriter.create(str(file_path))

    params = InitParams(
        title="Side Set Reduction Test",
        num_dim=3,
        num_nodes=4,
        num_elems=1,
        num_elem_blocks=1,
        num_side_sets=1,
    )
    writer.put_init_params(params)

    writer.put_coords(
        [0.0, 1.0, 1.0, 0.0],
        [0.0, 0.0, 1.0, 1.0],
        [0.0, 0.0, 0.0, 0.0],
    )

    block = Block(
        id=10,
        entity_type=EntityType.ElemBlock,
        topology="quad",
        num_entries=1,
        num_nodes_per_entry=4,
        num_edges_per_entry=0,
        num_faces_per_entry=0,
        num_attributes=0,
    )
    writer.put_block(block)
    writer.put_connectivity(10, [1, 2, 3, 4])

    # Side set
    writer.put_side_set(300, [1, 1], [1, 2])

    # Define reduction variables
    writer.define_reduction_variables(EntityType.SideSet, ["AvgFlux"])

    writer.put_time(0, 0.0)
    writer.put_reduction_vars(0, EntityType.SideSet, 300, [42.5])

    writer.close()

    # Read
    reader = ExodusReader.open(str(file_path))

    names = reader.reduction_variable_names(EntityType.SideSet)
    assert names == ["AvgFlux"]

    values = reader.get_reduction_vars(0, EntityType.SideSet, 300)
    assert abs(values[0] - 42.5) < 1e-10

    reader.close()


def test_multiple_timesteps(tmp_path):
    """Test reduction variables across multiple time steps"""
    file_path = tmp_path / "multi_timestep_reduction.exo"

    # Write
    writer = ExodusWriter.create(str(file_path))

    params = InitParams(
        title="Multi-Timestep Reduction Test",
        num_dim=3,
        num_nodes=3,
        num_elems=1,
        num_elem_blocks=1,
        num_assemblies=1,
    )
    writer.put_init_params(params)

    writer.put_coords([0.0, 1.0, 2.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0])

    block = Block(
        id=10,
        entity_type=EntityType.ElemBlock,
        topology="tri",
        num_entries=1,
        num_nodes_per_entry=3,
        num_edges_per_entry=0,
        num_faces_per_entry=0,
        num_attributes=0,
    )
    writer.put_block(block)
    writer.put_connectivity(10, [1, 2, 3])

    assembly = Assembly(
        id=100,
        name="TestAssembly",
        entity_type=EntityType.ElemBlock,
        entity_list=[10],
    )
    writer.put_assembly(assembly)

    writer.define_reduction_variables(EntityType.Assembly, ["Energy", "Power"])

    # Write 10 time steps
    for ts in range(10):
        time = ts * 0.1
        writer.put_time(ts, time)

        energy = 100.0 + ts * 10.0
        power = 50.0 + ts * 5.0
        writer.put_reduction_vars(ts, EntityType.Assembly, 100, [energy, power])

    writer.close()

    # Read and verify
    reader = ExodusReader.open(str(file_path))

    for ts in range(10):
        values = reader.get_reduction_vars(ts, EntityType.Assembly, 100)
        expected_energy = 100.0 + ts * 10.0
        expected_power = 50.0 + ts * 5.0
        assert abs(values[0] - expected_energy) < 1e-10
        assert abs(values[1] - expected_power) < 1e-10

    reader.close()


def test_empty_reduction_variables(tmp_path):
    """Test behavior when no reduction variables are defined"""
    file_path = tmp_path / "empty_reduction.exo"

    # Write
    writer = ExodusWriter.create(str(file_path))

    params = InitParams(
        title="Empty Reduction Test",
        num_dim=3,
        num_nodes=3,
        num_assemblies=1,
    )
    writer.put_init_params(params)

    writer.put_coords([0.0, 1.0, 2.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0])

    assembly = Assembly(
        id=100,
        name="Assembly_A",
        entity_type=EntityType.ElemBlock,
        entity_list=[],
    )
    writer.put_assembly(assembly)

    writer.close()

    # Read
    reader = ExodusReader.open(str(file_path))

    # Should return empty list if no reduction variables defined
    names = reader.reduction_variable_names(EntityType.Assembly)
    assert len(names) == 0

    reader.close()


def test_context_manager_reduction_variables(tmp_path):
    """Test reduction variables with context managers"""
    file_path = tmp_path / "context_reduction.exo"

    # Write using context manager
    with ExodusWriter.create(str(file_path)) as writer:
        params = InitParams(
            title="Context Manager Test",
            num_dim=3,
            num_nodes=3,
            num_elems=1,
            num_elem_blocks=1,
            num_assemblies=1,
        )
        writer.put_init_params(params)

        writer.put_coords([0.0, 1.0, 2.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0])

        block = Block(
            id=10,
            entity_type=EntityType.ElemBlock,
            topology="tri",
            num_entries=1,
            num_nodes_per_entry=3,
            num_edges_per_entry=0,
            num_faces_per_entry=0,
            num_attributes=0,
        )
        writer.put_block(block)
        writer.put_connectivity(10, [1, 2, 3])

        assembly = Assembly(
            id=100,
            name="TestAssembly",
            entity_type=EntityType.ElemBlock,
            entity_list=[10],
        )
        writer.put_assembly(assembly)

        writer.define_reduction_variables(EntityType.Assembly, ["Value"])
        writer.put_time(0, 0.0)
        writer.put_reduction_vars(0, EntityType.Assembly, 100, [123.45])

    # Read using context manager
    with ExodusReader.open(str(file_path)) as reader:
        names = reader.reduction_variable_names(EntityType.Assembly)
        assert names == ["Value"]

        values = reader.get_reduction_vars(0, EntityType.Assembly, 100)
        assert abs(values[0] - 123.45) < 1e-10
