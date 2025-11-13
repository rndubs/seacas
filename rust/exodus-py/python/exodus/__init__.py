"""
exodus-py: Python bindings for the Exodus II file format

This package provides Python bindings to the exodus-rs library,
a pure Rust implementation of the Exodus II finite element data format.
"""

__version__ = "0.1.0"

# Import the Rust extension module (optional for testing)
try:
    from .exodus import *
    _exodus_available = True
except (ImportError, ModuleNotFoundError):
    _exodus_available = False

# Import the exomerge module for high-level mesh manipulation
# This provides API compatibility with the legacy exomerge3.py
from . import exomerge

__all__ = ["__version__", "exomerge", "_exodus_available"]
