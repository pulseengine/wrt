#!/bin/bash

set -e

# Define colors for output
YELLOW='\033[1;33m'
CYAN='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Define the crates to test
CRATES=("wrt-decoder" "wrt-runtime" "wrt-component" "wrt-logging")

# Function to test a crate with a specific configuration
test_crate() {
    crate_name=$1
    config=$2
    features=""

    if [ "$config" = "std" ]; then
        features="std"
    elif [ "$config" = "alloc" ]; then
        features="alloc"
    fi

    echo -e "${CYAN}--- Configuration: $config ---${NC}"
    echo -e "${CYAN}Building $crate_name with $config...${NC}"

    if [ -z "$features" ]; then
        cargo build -p $crate_name --no-default-features
    else
        cargo build -p $crate_name --no-default-features --features $features
    fi

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Build successful for $crate_name with $config${NC}"
    else
        echo -e "${RED}✗ Build failed for $crate_name with $config${NC}"
        return 1
    fi

    echo -e "${CYAN}Testing $crate_name with $config...${NC}"

    if [ -z "$features" ]; then
        cargo test -p $crate_name --no-default-features --lib --doc
    else
        cargo test -p $crate_name --no-default-features --features $features --lib --doc
    fi

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Test successful for $crate_name with $config${NC}"
    else
        echo -e "${RED}✗ Test failed for $crate_name with $config${NC}"
        return 1
    fi

    return 0
}

echo -e "${YELLOW}=== WRT no_std Compatibility Verification ===${NC}"
echo -e "${YELLOW}Testing configurations: std, no_std with alloc, no_std without alloc${NC}"

# Test each crate
for crate in "${CRATES[@]}"; do
    echo -e "\n${YELLOW}=== Verifying $crate ===${NC}"
    
    # Test with std
    test_crate $crate "std"
    
    # Test with alloc
    test_crate $crate "alloc"
    
    # Test with pure no_std
    test_crate $crate ""
done

echo -e "\n${GREEN}✓ All tests completed successfully${NC}"