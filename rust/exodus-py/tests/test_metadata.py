"""
Tests for metadata operations (QA records and info records)
"""

import pytest
import tempfile
import os

pytest.importorskip("exodus")

from exodus import (
    ExodusWriter,
    ExodusReader,
    InitParams,
    QaRecord,
)


def test_qa_record_creation():
    """Test creating QA records"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="QA Test", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Add QA record
        writer.put_qa(1, "TestCode", "1.0", "2025-01-15", "12:00:00")
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        qa_records = reader.get_qa_records()
        assert len(qa_records) == 1

        qa = qa_records[0]
        assert qa.code_name == "TestCode"
        assert qa.code_version == "1.0"
        assert qa.date == "2025-01-15"
        assert qa.time == "12:00:00"
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_qa_records():
    """Test multiple QA records"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Multi QA", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Add multiple QA records
        writer.put_qa(1, "Code1", "1.0", "2025-01-15", "12:00:00")
        writer.put_qa(2, "Code2", "2.0", "2025-01-16", "13:00:00")
        writer.put_qa(3, "Code3", "3.0", "2025-01-17", "14:00:00")
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        qa_records = reader.get_qa_records()
        assert len(qa_records) == 3

        assert qa_records[0].code_name == "Code1"
        assert qa_records[1].code_name == "Code2"
        assert qa_records[2].code_name == "Code3"

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_info_records():
    """Test info records"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Info Test", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Add info records
        info_lines = [
            "This is a test mesh",
            "Created for testing purposes",
            "Contains 4 nodes",
        ]
        for i, line in enumerate(info_lines, 1):
            writer.put_info(i, line)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        info_read = reader.get_info_records()
        assert len(info_read) == 3
        assert info_read == info_lines
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_single_info_record():
    """Test single info record"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Single Info", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        writer.put_info(1, "Single line of information")
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        info_read = reader.get_info_records()
        assert len(info_read) == 1
        assert info_read[0] == "Single line of information"
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_qa_and_info_together():
    """Test QA records and info records together"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="QA and Info", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Add both QA and info records
        writer.put_qa(1, "MyCode", "1.0", "2025-01-15", "12:00:00")
        writer.put_info(1, "Test mesh file")
        writer.put_info(2, "With metadata")
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)

        qa_records = reader.get_qa_records()
        assert len(qa_records) == 1
        assert qa_records[0].code_name == "MyCode"

        info_records = reader.get_info_records()
        assert len(info_records) == 2
        assert info_records[0] == "Test mesh file"
        assert info_records[1] == "With metadata"

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_empty_metadata():
    """Test file with no QA or info records"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="No Metadata", num_dim=2, num_nodes=4)
        writer.put_init_params(params)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        qa_records = reader.get_qa_records()
        info_records = reader.get_info_records()

        # Should return empty lists, not error
        assert isinstance(qa_records, list)
        assert isinstance(info_records, list)

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
