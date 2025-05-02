#!/bin/bash

set -e

# Define colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "Testing all crates with no_std feature..."

# List of all crates to test
CRATES=(
  "wrt-error"
  "wrt-sync"
  "wrt-types"
  "wrt-format"
  "wrt-decoder"
  "wrt-component"
  "wrt-host"
  "wrt-logging"
  "wrt-runtime"
  "wrt-instructions"
  "wrt-intercept"
  "wrt"
)

for crate in "${CRATES[@]}"; do
  echo "Building $crate with no_std feature..."
  
  if cargo build --no-default-features --features "no_std,alloc" -p "$crate"; then
    echo -e "${GREEN}Success:${NC} $crate builds with no_std"
  else
    echo -e "${RED}Failed:${NC} $crate failed to build with no_std"
    exit 1
  fi
done

echo -e "${GREEN}All crates build successfully with no_std feature!${NC}" 