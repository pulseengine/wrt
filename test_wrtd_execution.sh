#!/bin/bash
# Test script for WebAssembly execution using wrtd

echo "=== Testing WebAssembly Execution with wrtd ==="
echo ""

# Build wrtd with wrt-execution feature
echo "1. Building wrtd with actual execution support..."
cargo build --package wrtd --features std,wrt-execution
if [ $? -ne 0 ]; then
    echo "❌ Failed to build wrtd"
    exit 1
fi
echo "✅ Build successful"
echo ""

# Test if wrtd can load the module
echo "2. Testing module loading with test_add.wasm..."
echo "Running: cargo run --package wrtd -- test_add.wasm"
cargo run --package wrtd -- test_add.wasm
if [ $? -ne 0 ]; then
    echo "❌ Failed to run wrtd"
    exit 1
fi
echo ""

# Also test with explicit ASIL-B configuration
echo "3. Testing with ASIL-B configuration..."
echo "Running: cargo run --package wrtd --features asil-b -- test_add.wasm"
cargo run --package wrtd --features asil-b -- test_add.wasm
if [ $? -ne 0 ]; then
    echo "❌ Failed to run wrtd with ASIL-B"
    exit 1
fi

echo ""
echo "✨ All tests completed!"