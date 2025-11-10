#!/bin/bash

cd "$(dirname "$0")"
VERIFIER="./target/release/exodus-c-to-rust-verifier"
OUTPUT_DIR="../rust-to-c/output"

echo "Testing all generated Exodus files..."
echo "======================================"
echo

passed=0
failed=0

for file in "$OUTPUT_DIR"/*.exo; do
    filename=$(basename "$file")
    echo -n "Testing $filename... "

    if $VERIFIER "$file" > /dev/null 2>&1; then
        echo -e "\033[32mPASS\033[0m"
        ((passed++))
    else
        echo -e "\033[31mFAIL\033[0m"
        ((failed++))
    fi
done

echo
echo "======================================"
echo "Total: $((passed + failed)) files"
echo -e "\033[32mPassed: $passed\033[0m"
echo -e "\033[31mFailed: $failed\033[0m"
echo "======================================"

if [ $failed -eq 0 ]; then
    exit 0
else
    exit 1
fi
