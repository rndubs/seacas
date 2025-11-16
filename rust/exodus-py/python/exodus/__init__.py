"""
exodus-py: Python bindings for the Exodus II file format

This package provides Python bindings to the exodus-rs library,
a pure Rust implementation of the Exodus II finite element data format.
"""

# Get version from the Rust extension module (which reads from Cargo.toml)
# Fall back to package metadata if the extension module isn't available
try:
    from .exodus import __version__
    _exodus_available = True
except (ImportError, ModuleNotFoundError, AttributeError):
    _exodus_available = False
    # Fallback: read from package metadata
    try:
        from importlib.metadata import version
        __version__ = version("exodus-py")
    except Exception:
        __version__ = "0.0.0-unknown"

# Import the rest of the Rust extension module
if _exodus_available:
    from .exodus import *

# Import the exomerge module for high-level mesh manipulation
# This provides API compatibility with the legacy exomerge3.py
from . import exomerge

__all__ = ["__version__", "exomerge", "_exodus_available"]
