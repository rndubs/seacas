"""
Tests for attribute operations
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
    AttributeType,
    AttributeData,
    Block,
)


def test_attribute_data_integer():
    """Test creating integer attribute data"""
    attr = AttributeData.integer([1, 2, 3, 4])
    values = attr.as_integer()
    assert values == [1, 2, 3, 4]

    # Test type checking
    with pytest.raises(Exception):
        attr.as_double()

    with pytest.raises(Exception):
        attr.as_char()


def test_attribute_data_double():
    """Test creating double attribute data"""
    attr = AttributeData.double([1.5, 2.5, 3.5])
    values = attr.as_double()
    assert values == pytest.approx([1.5, 2.5, 3.5], abs=1e-6)

    # Test type checking
    with pytest.raises(Exception):
        attr.as_integer()

    with pytest.raises(Exception):
        attr.as_char()


def test_attribute_data_char():
    """Test creating character/string attribute data"""
    attr = AttributeData.char("test string")
    value = attr.as_char()
    assert value == "test string"

    # Test type checking
    with pytest.raises(Exception):
        attr.as_integer()

    with pytest.raises(Exception):
        attr.as_double()


def test_put_and_get_integer_attribute():
    """Test writing and reading integer attributes"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        # Create file with element block
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Attribute Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        # Define element block
        block = Block(
            id=100,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Write integer attribute
        attr_data = AttributeData.integer([42])
        writer.put_attribute(
            EntityType.ElemBlock,
            100,
            "material_id",
            AttributeType.Integer,
            attr_data,
        )
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)

        # Get attribute names
        attr_names = reader.get_attribute_names(EntityType.ElemBlock, 100)
        assert "material_id" in attr_names

        # Get attribute data
        attr_read = reader.get_attribute(EntityType.ElemBlock, 100, "material_id")
        values = attr_read.as_integer()
        assert values == [42]

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_put_and_get_double_attribute():
    """Test writing and reading double attributes"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Double Attr Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Write double attribute
        attr_data = AttributeData.double([3.14159])
        writer.put_attribute(
            EntityType.ElemBlock,
            1,
            "density",
            AttributeType.Double,
            attr_data,
        )
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        attr_read = reader.get_attribute(EntityType.ElemBlock, 1, "density")
        values = attr_read.as_double()
        assert values == pytest.approx([3.14159], abs=1e-6)
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_put_and_get_char_attribute():
    """Test writing and reading character attributes"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Char Attr Test",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Write character attribute
        attr_data = AttributeData.char("steel")
        writer.put_attribute(
            EntityType.ElemBlock,
            1,
            "material_name",
            AttributeType.Char,
            attr_data,
        )
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        attr_read = reader.get_attribute(EntityType.ElemBlock, 1, "material_name")
        value = attr_read.as_char()
        assert value == "steel"
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_attributes():
    """Test multiple attributes on same entity"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(
            title="Multi Attr",
            num_dim=2,
            num_nodes=4,
            num_elems=1,
            num_elem_blocks=1,
        )
        writer.put_init_params(params)

        block = Block(
            id=1,
            entity_type=EntityType.ElemBlock,
            topology="QUAD4",
            num_entries=1,
            num_nodes_per_entry=4,
            num_attributes=0,
        )
        writer.put_block(block)

        # Write multiple attributes
        writer.put_attribute(
            EntityType.ElemBlock,
            1,
            "material_id",
            AttributeType.Integer,
            AttributeData.integer([1]),
        )
        writer.put_attribute(
            EntityType.ElemBlock,
            1,
            "density",
            AttributeType.Double,
            AttributeData.double([7.85]),
        )
        writer.put_attribute(
            EntityType.ElemBlock,
            1,
            "material_name",
            AttributeType.Char,
            AttributeData.char("steel"),
        )
        writer.close()

        # Read back all attributes
        reader = ExodusReader.open(tmp_path)

        attr_names = reader.get_attribute_names(EntityType.ElemBlock, 1)
        assert len(attr_names) == 3
        assert "material_id" in attr_names
        assert "density" in attr_names
        assert "material_name" in attr_names

        # Verify each attribute
        mat_id = reader.get_attribute(EntityType.ElemBlock, 1, "material_id")
        assert mat_id.as_integer() == [1]

        density = reader.get_attribute(EntityType.ElemBlock, 1, "density")
        assert density.as_double() == pytest.approx([7.85], abs=1e-6)

        mat_name = reader.get_attribute(EntityType.ElemBlock, 1, "material_name")
        assert mat_name.as_char() == "steel"

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
