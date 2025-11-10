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

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="QA Test", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Add QA record (NOT YET IMPLEMENTED)
        qa = QaRecord(
            code_name="TestCode",
            code_version="1.0",
            date="2025-01-15",
            time="12:00:00"
        )
        writer.put_qa_records([qa])
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        qa_records = reader.get_qa_records()
        assert len(qa_records) == 1

        qa_read = qa_records[0]
        assert qa_read.code_name == "TestCode"
        assert qa_read.code_version == "1.0"
        assert qa_read.date == "2025-01-15"
        assert qa_read.time == "12:00:00"
        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_multiple_qa_records():
    """Test multiple QA records"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Multi QA", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Add multiple QA records (NOT YET IMPLEMENTED)
        qa_records = [
            QaRecord("Code1", "1.0", "2025-01-15", "12:00:00"),
            QaRecord("Code2", "2.0", "2025-01-16", "13:00:00"),
            QaRecord("Code3", "3.0", "2025-01-17", "14:00:00"),
        ]
        writer.put_qa_records(qa_records)
        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)
        qa_records_read = reader.get_qa_records()
        assert len(qa_records_read) == 3

        assert qa_records_read[0].code_name == "Code1"
        assert qa_records_read[1].code_name == "Code2"
        assert qa_records_read[2].code_name == "Code3"

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


def test_info_records():
    """Test info records"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

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
        writer.put_info_records(info_lines)
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


def test_combined_metadata():
    """Test combining info records"""
    with tempfile.NamedTemporaryFile(suffix=".exo", delete=False) as tmp:
        tmp_path = tmp.name

    # Delete the empty file so ExodusWriter can create it
    os.unlink(tmp_path)

    try:
        writer = ExodusWriter.create(tmp_path)
        params = InitParams(title="Combined Metadata", num_dim=2, num_nodes=4)
        writer.put_init_params(params)

        # Add info records
        writer.put_info_records([
            "Test mesh with metadata",
            "Multiple info lines"
        ])

        # QA records would go here (not yet implemented)
        # qa = QaRecord("App", "1.0", "2025-01-15", "12:00:00")
        # writer.put_qa_records([qa])

        writer.close()

        # Read back
        reader = ExodusReader.open(tmp_path)

        info = reader.get_info_records()
        assert len(info) == 2
        assert info[0] == "Test mesh with metadata"

        # QA records not yet available for reading
        # qa_records = reader.get_qa_records()

        reader.close()

    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
