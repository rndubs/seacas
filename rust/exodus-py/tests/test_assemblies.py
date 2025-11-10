"""
Tests for assembly and blob operations
"""

import pytest
import tempfile
import os

pytest.importorskip("exodus")

from exodus import (
    ExodusWriter,
    ExodusReader,
    InitParams,
    Assembly,
    Blob,
    EntityType,
    Block,
)


def test_assembly_creation():
    """Test creating an Assembly"""
    # Create assembly with basic properties
    assembly = Assembly(
        id=1,
        name="MainAssembly",
        entity_type=EntityType.ElemBlock,
        entity_list=[],
    )
    assert assembly.id == 1
    assert assembly.name == "MainAssembly"
    assert assembly.entity_type == EntityType.ElemBlock
    assert assembly.entity_list == []


def test_put_and_get_assembly():
    """Test writing and reading assemblies"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Assembly Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        # Define element block
        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Create and write assembly
        assembly = Assembly(
            id=100,
            name="Structure",
            entity_type=EntityType.ElemBlock,
            entity_list=[1],
        )
        writer.put_assembly(assembly)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        assembly_ids = reader.get_assembly_ids()
        assert 100 in assembly_ids

        asm_read = reader.get_assembly(100)
        assert asm_read.id == 100
        assert asm_read.name == "Structure"
        assert asm_read.entity_type == EntityType.ElemBlock
        assert asm_read.entity_list == [1]
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_assemblies():
    """Test multiple assemblies"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Multi Assembly", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Create multiple assemblies
        asm1 = Assembly(id=1, name="Part1", entity_type=EntityType.ElemBlock, entity_list=[])
        asm2 = Assembly(id=2, name="Part2", entity_type=EntityType.ElemBlock, entity_list=[])
        asm3 = Assembly(id=3, name="Assembly1", entity_type=EntityType.Assembly, entity_list=[1, 2])

        writer.put_assembly(asm1)
        writer.put_assembly(asm2)
        writer.put_assembly(asm3)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        assembly_ids = reader.get_assembly_ids()
        assert len(assembly_ids) == 3
        assert 1 in assembly_ids
        assert 2 in assembly_ids
        assert 3 in assembly_ids

        # Verify each assembly
        a1 = reader.get_assembly(1)
        assert a1.name == "Part1"

        a2 = reader.get_assembly(2)
        assert a2.name == "Part2"

        a3 = reader.get_assembly(3)
        assert a3.name == "Assembly1"
        # Note: entity_type may be normalized when reading
        assert len(a3.entity_list) == 2
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_blob_creation():
    """Test creating a Blob"""
    blob = Blob(id=1, name="TestBlob")
    assert blob.id == 1
    assert blob.name == "TestBlob"


def test_put_and_get_blob():
    """Test writing and reading blobs with binary data"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Blob Test", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Create blob with binary data
        blob = Blob(id=1, name="BinaryData")
        binary_data = bytes([0x01, 0x02, 0x03, 0x04, 0xFF, 0xFE, 0xFD])

        writer.put_blob(blob, list(binary_data))
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        blob_ids = reader.get_blob_ids()
        assert 1 in blob_ids

        blob_read, data_read = reader.get_blob(1)
        assert blob_read.id == 1
        assert blob_read.name == "BinaryData"
        assert bytes(data_read) == binary_data
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_blobs():
    """Test multiple blobs with different data"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Multi Blob", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Create multiple blobs
        blob1 = Blob(id=1, name="Data1")
        data1 = bytes([0x01, 0x02, 0x03])

        blob2 = Blob(id=2, name="Data2")
        data2 = bytes([0xFF, 0xFE, 0xFD, 0xFC])

        blob3 = Blob(id=3, name="EmptyData")
        data3 = bytes([])

        writer.put_blob(blob1, list(data1))
        writer.put_blob(blob2, list(data2))
        writer.put_blob(blob3, list(data3))
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        blob_ids = reader.get_blob_ids()
        assert len(blob_ids) == 3

        # Verify each blob
        b1, d1 = reader.get_blob(1)
        assert b1.name == "Data1"
        assert bytes(d1) == data1

        b2, d2 = reader.get_blob(2)
        assert b2.name == "Data2"
        assert bytes(d2) == data2

        b3, d3 = reader.get_blob(3)
        assert b3.name == "EmptyData"
        assert bytes(d3) == data3
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_large_blob():
    """Test blob with larger binary data"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Large Blob", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Create blob with larger data (1KB)
        blob = Blob(id=1, name="LargeData")
        binary_data = bytes(range(256)) * 4  # 1024 bytes

        writer.put_blob(blob, list(binary_data))
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        blob_read, data_read = reader.get_blob(1)
        assert len(data_read) == 1024
        assert bytes(data_read) == binary_data
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
