#!/bin/bash
#
# Clean up generated files and build artifacts
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPAT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Cleaning compatibility test artifacts..."

# Remove output directory
if [ -d "$COMPAT_DIR/output" ]; then
    echo "  Removing output/"
    rm -rf "$COMPAT_DIR/output"
fi

# Remove C binaries
if [ -f "$COMPAT_DIR/rust-to-c/verify" ]; then
    echo "  Removing rust-to-c/verify"
    rm -f "$COMPAT_DIR/rust-to-c/verify"
fi

if [ -f "$COMPAT_DIR/c-to-rust/writer" ]; then
    echo "  Removing c-to-rust/writer"
    rm -f "$COMPAT_DIR/c-to-rust/writer"
fi

# Optionally remove Rust build artifacts
if [ "$1" = "--all" ]; then
    echo "  Removing Rust build artifacts..."

    if [ -d "$COMPAT_DIR/rust-to-c/target" ]; then
        echo "    Removing rust-to-c/target/"
        rm -rf "$COMPAT_DIR/rust-to-c/target"
    fi

    if [ -d "$COMPAT_DIR/c-to-rust/target" ]; then
        echo "    Removing c-to-rust/target/"
        rm -rf "$COMPAT_DIR/c-to-rust/target"
    fi

    # Remove Cargo.lock files
    rm -f "$COMPAT_DIR/rust-to-c/Cargo.lock"
    rm -f "$COMPAT_DIR/c-to-rust/Cargo.lock"
fi

echo "✓ Clean complete!"
