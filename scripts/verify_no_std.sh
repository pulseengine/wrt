#!/bin/bash

# Verify all crates in the WRT ecosystem for no_std compatibility
# Tests std, no_std with alloc, and no_std without alloc configurations

set -e

# Define color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Define configurations
CONFIGS=("std" "alloc" "")

# Define crates to test
CRATES=(
  "wrt-math"
  "wrt-sync"
  "wrt-error"
  "wrt-foundation"
  "wrt-format"
  "wrt-decoder"
  "wrt-instructions"
  "wrt-runtime"
  "wrt-host"
  "wrt-intercept"
  "wrt-component"
  "wrt-platform"
  "wrt-logging"
  "wrt"
)

# Function to report success/failure
report() {
  local result=$1
  local operation=$2
  local crate=$3
  local config=$4
  
  if [ $result -eq 0 ]; then
    echo -e "${GREEN}✓ $operation successful for $crate with $config${NC}"
  else
    echo -e "${RED}✗ $operation failed for $crate with $config${NC}"
    if [ "$CONTINUE_ON_ERROR" != "true" ]; then
      exit 1
    fi
  fi
}

# Function to run tests with specific pattern
run_test_pattern() {
  local crate=$1
  local config=$2
  local pattern=$3
  
  echo -e "${BLUE}Running $pattern tests for $crate with $config...${NC}"
  
  if [ "$config" == "std" ]; then
    cargo test -p "$crate" --features std -- "$pattern" > /dev/null 2>&1
  elif [ "$config" == "" ]; then
    cargo test -p "$crate" --no-default-features -- "$pattern" > /dev/null 2>&1
  else
    cargo test -p "$crate" --no-default-features --features "$config" -- "$pattern" > /dev/null 2>&1
  fi
  
  report $? "Test pattern '$pattern'" "$crate" "$config"
}

# Print header
echo -e "${YELLOW}=== WRT no_std Compatibility Verification ===${NC}"
echo -e "${YELLOW}Testing configurations: std, no_std with alloc, no_std without alloc${NC}"
echo ""

# Process command line arguments
CONTINUE_ON_ERROR=false
VERBOSE=false

for arg in "$@"; do
  case $arg in
    --continue-on-error)
      CONTINUE_ON_ERROR=true
      shift
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    *)
      # Unknown option
      ;;
  esac
done

# Run verification for each crate in each configuration
for crate in "${CRATES[@]}"; do
  echo -e "${YELLOW}=== Verifying $crate ===${NC}"
  
  for config in "${CONFIGS[@]}"; do
    echo -e "${BLUE}--- Configuration: $config ---${NC}"
    
    # Build operation
    echo -e "${BLUE}Building $crate with $config...${NC}"
    if [ "$config" == "std" ]; then
      if [ "$VERBOSE" == "true" ]; then
        cargo build -p "$crate" --features std
      else
        cargo build -p "$crate" --features std > /dev/null 2>&1
      fi
      report $? "Build" "$crate" "$config"
    elif [ "$config" == "" ]; then
      if [ "$VERBOSE" == "true" ]; then
        cargo build -p "$crate" --no-default-features
      else
        cargo build -p "$crate" --no-default-features > /dev/null 2>&1
      fi
      report $? "Build" "$crate" "$config"
    else
      if [ "$VERBOSE" == "true" ]; then
        cargo build -p "$crate" --no-default-features --features "$config"
      else
        cargo build -p "$crate" --no-default-features --features "$config" > /dev/null 2>&1
      fi
      report $? "Build" "$crate" "$config"
    fi
    
    # Test operation
    echo -e "${BLUE}Testing $crate with $config...${NC}"
    if [ "$config" == "std" ]; then
      if [ "$VERBOSE" == "true" ]; then
        cargo test -p "$crate" --features std
      else
        cargo test -p "$crate" --features std > /dev/null 2>&1
      fi
      report $? "Test" "$crate" "$config"
    elif [ "$config" == "" ]; then
      if [ "$VERBOSE" == "true" ]; then
        cargo test -p "$crate" --no-default-features
      else
        cargo test -p "$crate" --no-default-features > /dev/null 2>&1
      fi
      report $? "Test" "$crate" "$config"
    else
      if [ "$VERBOSE" == "true" ]; then
        cargo test -p "$crate" --no-default-features --features "$config"
      else
        cargo test -p "$crate" --no-default-features --features "$config" > /dev/null 2>&1
      fi
      report $? "Test" "$crate" "$config"
    fi
    
    # Run specific pattern tests based on crate name
    case $crate in
      "wrt-error")
        run_test_pattern "$crate" "$config" "integration_test"
        run_test_pattern "$crate" "$config" "no_std_compatibility_test"
        ;;
      "wrt-foundation")
        run_test_pattern "$crate" "$config" "bounded_collections_test"
        run_test_pattern "$crate" "$config" "safe_memory_test"
        run_test_pattern "$crate" "$config" "safe_stack_test"
        ;;
      "wrt-runtime")
        run_test_pattern "$crate" "$config" "memory_safety_tests"
        run_test_pattern "$crate" "$config" "no_std_compatibility_test"
        ;;
      "wrt-component"|"wrt-host"|"wrt-intercept"|"wrt-decoder"|"wrt-format"|"wrt-instructions"|"wrt-sync")
        run_test_pattern "$crate" "$config" "no_std_compatibility_test"
        ;;
      "wrt")
        run_test_pattern "$crate" "$config" "no_std_compatibility_test"
        ;;
    esac
    
    echo ""
  done
done

# Run integration tests
echo -e "${YELLOW}=== Running Integration Tests ===${NC}"

for config in "${CONFIGS[@]}"; do
  echo -e "${BLUE}--- Integration tests with $config ---${NC}"
  
  if [ "$config" == "std" ]; then
    echo -e "${BLUE}Running workspace tests with std...${NC}"
    if [ "$VERBOSE" == "true" ]; then
      cargo test --workspace --features std
    else
      cargo test --workspace --features std > /dev/null 2>&1
    fi
    report $? "Workspace integration tests" "workspace" "$config"
  elif [ "$config" == "" ]; then
    echo -e "${BLUE}Running workspace tests with pure no_std...${NC}"
    if [ "$VERBOSE" == "true" ]; then
      cargo test --workspace --no-default-features
    else
      cargo test --workspace --no-default-features > /dev/null 2>&1
    fi
    report $? "Workspace integration tests" "workspace" "$config"
    
    echo -e "${BLUE}Running no_std compatibility tests...${NC}"
    if [ "$VERBOSE" == "true" ]; then
      cargo test --no-default-features -- no_std_compatibility_test
    else
      cargo test --no-default-features -- no_std_compatibility_test > /dev/null 2>&1
    fi
    report $? "No_std compatibility tests" "workspace" "$config"
  else
    echo -e "${BLUE}Running workspace tests with alloc...${NC}"
    if [ "$VERBOSE" == "true" ]; then
      cargo test --workspace --no-default-features --features "$config"
    else
      cargo test --workspace --no-default-features --features "$config" > /dev/null 2>&1
    fi
    report $? "Workspace integration tests" "workspace" "$config"
    
    echo -e "${BLUE}Running no_std compatibility tests...${NC}"
    if [ "$VERBOSE" == "true" ]; then
      cargo test --no-default-features --features "$config" -- no_std_compatibility_test
    else
      cargo test --no-default-features --features "$config" -- no_std_compatibility_test > /dev/null 2>&1
    fi
    report $? "No_std compatibility tests" "workspace" "$config"
  fi
  
  echo ""
done

echo -e "${GREEN}Verification completed!${NC}"
echo -e "${BLUE}For detailed test output, run with --verbose flag${NC}"