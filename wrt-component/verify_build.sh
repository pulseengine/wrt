#!/bin/bash

# Exit on error
set -e

echo "Verifying std build..."
cargo build --verbose

echo "Verifying no_std build..."
cargo build --no-default-features --features no_std --verbose

echo "Verifying component model features..."
cargo build --features component-model-all --verbose

echo "Verifying kani feature..."
cargo build --features kani --verbose

echo "All builds successful!" 