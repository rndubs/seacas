#!/bin/bash
#
# Build all compatibility test programs (Rust and C)
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo
echo "========================================"
echo "  Building All Compatibility Tests"
echo "========================================"
echo

# Build Rust programs
"$SCRIPT_DIR/build_rust.sh"

echo

# Build C programs
"$SCRIPT_DIR/build_c.sh"

echo
echo "========================================"
echo "  All builds complete!"
echo "========================================"
echo
