#!/bin/bash
cd /home/user/seacas/rust/compat-tests/rust-to-c
for f in output/*.exo; do
    fname=$(basename "$f")
    echo "=== $fname ==="
    ./verify "$f" 2>&1 | tail -6
    echo ""
done
