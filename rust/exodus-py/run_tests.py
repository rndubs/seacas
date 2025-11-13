#!/usr/bin/env python3
"""
Simple test runner for exomerge tests (no pytest required).
"""

import sys
import os

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))

# Import all test functions
from tests.test_exomerge_implementation import (
    test_import_simple_mesh,
    test_element_block_info,
    test_export_simple_mesh,
    test_manual_model_construction,
    test_element_block_names,
    test_element_dimensions,
    test_timesteps,
    test_metadata_operations,
    test_get_connectivity_auto_error,
    test_element_block_not_found,
    test_roundtrip_preserve_data,
    test_phase4_node_operations,
    test_phase4_merge_nodes,
    test_phase5_side_sets,
    test_phase5_node_sets,
    test_phase6_element_fields,
    test_phase6_node_fields,
    test_phase6_global_variables,
    test_phase6_side_set_fields,
    test_phase6_node_set_fields,
    test_delete_empty_sets,
    test_delete_unused_nodes,
)

def run_test(test_func):
    """Run a single test function and report results."""
    test_name = test_func.__name__
    try:
        test_func()
        print(f"✓ {test_name}")
        return True
    except AssertionError as e:
        print(f"✗ {test_name}: {e}")
        return False
    except Exception as e:
        print(f"✗ {test_name}: {type(e).__name__}: {e}")
        return False

def main():
    """Run all tests."""
    tests = [
        test_import_simple_mesh,
        test_element_block_info,
        test_export_simple_mesh,
        test_manual_model_construction,
        test_element_block_names,
        test_element_dimensions,
        test_timesteps,
        test_metadata_operations,
        test_get_connectivity_auto_error,
        test_element_block_not_found,
        test_roundtrip_preserve_data,
        test_phase4_node_operations,
        test_phase4_merge_nodes,
        test_phase5_side_sets,
        test_phase5_node_sets,
        test_phase6_element_fields,
        test_phase6_node_fields,
        test_phase6_global_variables,
        test_phase6_side_set_fields,
        test_phase6_node_set_fields,
        test_delete_empty_sets,
        test_delete_unused_nodes,
    ]

    print("Running exodus.exomerge tests...")
    print("=" * 60)

    passed = 0
    failed = 0

    for test in tests:
        if run_test(test):
            passed += 1
        else:
            failed += 1

    print("=" * 60)
    print(f"Results: {passed} passed, {failed} failed")

    return 0 if failed == 0 else 1

if __name__ == '__main__':
    sys.exit(main())
