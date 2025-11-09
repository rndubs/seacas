# Shared Test Utilities

This directory contains shared utilities and test data for compatibility testing.

## Structure

- `test_data/` - Reference test data definitions
- `schemas/` - Validation schemas for test data
- `compare_utils/` - Cross-language comparison tools

## Planned Contents

### test_data/

JSON files defining reference meshes and expected values:
- `simple_mesh.json` - Simple 2D/3D test meshes
- `complex_mesh.json` - Complex meshes with all features
- `expected_values/` - Expected output values for validation

### schemas/

Data validation schemas:
- JSON schemas for test data format
- Validation rules for compatibility checks

### compare_utils/

Tools for comparing Exodus files:
- `compare.py` - Python script for detailed comparison
- `diff_exodus.sh` - Shell script using ncdump for comparison

## Future Enhancements

- Binary comparison tools
- Automated difference reporting
- Tolerance configuration files
- Reference file generation utilities
