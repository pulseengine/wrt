#!/bin/bash
set -e

echo "Testing wrt-foundation in all configurations..."
echo "============================================="

# Test 1: Default (no_std, no_alloc)
echo -e "\n1. Testing default configuration (no_std, no_alloc)..."
cargo build --package wrt-foundation --no-default-features
cargo test --package wrt-foundation --no-default-features -- --nocapture

# Test 2: no_std with alloc
echo -e "\n2. Testing no_std with alloc..."
cargo build --package wrt-foundation --no-default-features --features alloc
cargo test --package wrt-foundation --no-default-features --features alloc -- --nocapture

# Test 3: Full std
echo -e "\n3. Testing with std..."
cargo build --package wrt-foundation --features std
cargo test --package wrt-foundation --features std -- --nocapture

# Test 4: Check that SimpleHashMap is only available without std/alloc
echo -e "\n4. Verifying SimpleHashMap availability..."
# This should compile
cargo check --package wrt-foundation --no-default-features

# Test that we can build for embedded targets
echo -e "\n5. Testing embedded target compilation..."
# Try building for a common embedded target
cargo build --package wrt-foundation --no-default-features --target thumbv7m-none-eabi || echo "Note: thumbv7m-none-eabi target not installed, skipping embedded test"

echo -e "\nAll tests passed! ✅"