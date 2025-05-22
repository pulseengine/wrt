#!/bin/bash
# Test script for wrt-logging crate in different configurations

# Make script exit on error
set -e

# Default to tests in the standard configuration
cargo test -p wrt-logging

# Test with no_std + alloc configuration
echo "Testing with no_std + alloc..."
cargo test -p wrt-logging --no-default-features --features="alloc"

# Test with pure no_std configuration (basic compile check)
echo "Testing with pure no_std..."
cargo check -p wrt-logging --no-default-features

echo "All tests completed successfully!"