#!/bin/bash
#
# Build Rust compatibility test programs
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPAT_DIR="$(dirname "$SCRIPT_DIR")"

echo "===================================="
echo "  Building Rust Test Programs"
echo "===================================="
echo

# Build rust-to-c writer
echo "Building rust-to-c writer..."
cd "$COMPAT_DIR/rust-to-c"
cargo build --release --features netcdf4
echo "✓ Built: target/release/exodus-rust-to-c-tests"
echo

# Build c-to-rust verifier
echo "Building c-to-rust verifier..."
cd "$COMPAT_DIR/c-to-rust"
cargo build --release --features netcdf4
echo "✓ Built: target/release/exodus-c-to-rust-verifier"
echo

echo "===================================="
echo "  Rust builds complete!"
echo "===================================="
