#!/bin/bash

# Script to run all fuzzing targets for wrt-component

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
DURATION=60  # seconds per target
WORKERS=4
RUNS=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--duration)
            DURATION="$2"
            shift 2
            ;;
        -w|--workers)
            WORKERS="$2"
            shift 2
            ;;
        -r|--runs)
            RUNS="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  -d, --duration SECONDS   Duration to run each fuzzer (default: 60)"
            echo "  -w, --workers COUNT      Number of workers to use (default: 4)"
            echo "  -r, --runs COUNT         Number of runs (overrides duration)"
            echo "  -h, --help              Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check if cargo-fuzz is installed
if ! cargo +nightly fuzz --help &> /dev/null; then
    echo -e "${RED}Error: cargo-fuzz is not installed or nightly toolchain is not available${NC}"
    echo "Install with: cargo install cargo-fuzz"
    echo "Install nightly: rustup install nightly"
    exit 1
fi

# Change to fuzz directory
cd "$(dirname "$0")/fuzz" || exit 1

# List of fuzzing targets
TARGETS=(
    "fuzz_wit_parser"
    "fuzz_component_parser"
    "fuzz_canonical_options"
    "fuzz_type_bounds"
)

echo -e "${GREEN}Starting fuzzing run${NC}"
echo "Configuration:"
echo "  Duration per target: ${DURATION}s"
echo "  Workers: ${WORKERS}"
if [ -n "$RUNS" ]; then
    echo "  Runs: ${RUNS}"
fi
echo ""

# Run each fuzzing target
for target in "${TARGETS[@]}"; do
    echo -e "${YELLOW}Running fuzzer: $target${NC}"
    
    # Build the fuzzing command
    FUZZ_CMD="cargo +nightly fuzz run $target -- -workers=$WORKERS"
    
    if [ -n "$RUNS" ]; then
        FUZZ_CMD="$FUZZ_CMD -runs=$RUNS"
    else
        FUZZ_CMD="$FUZZ_CMD -max_total_time=$DURATION"
    fi
    
    # Run the fuzzer
    if $FUZZ_CMD; then
        echo -e "${GREEN}✓ $target completed successfully${NC}"
    else
        echo -e "${RED}✗ $target failed or found crashes${NC}"
        
        # Check for crashes
        CRASH_DIR="artifacts/$target"
        if [ -d "$CRASH_DIR" ] && [ "$(ls -A "$CRASH_DIR" 2>/dev/null)" ]; then
            echo -e "${RED}Crashes found in $CRASH_DIR:${NC}"
            ls -la "$CRASH_DIR"
        fi
    fi
    
    echo ""
done

echo -e "${GREEN}Fuzzing run complete!${NC}"

# Summary of any crashes found
echo -e "${YELLOW}Checking for crashes...${NC}"
CRASHES_FOUND=false
for target in "${TARGETS[@]}"; do
    CRASH_DIR="artifacts/$target"
    if [ -d "$CRASH_DIR" ] && [ "$(ls -A "$CRASH_DIR" 2>/dev/null)" ]; then
        CRASHES_FOUND=true
        echo -e "${RED}Crashes in $target:${NC}"
        ls -1 "$CRASH_DIR" | head -5
        COUNT=$(ls -1 "$CRASH_DIR" | wc -l)
        if [ "$COUNT" -gt 5 ]; then
            echo "  ... and $((COUNT - 5)) more"
        fi
    fi
done

if [ "$CRASHES_FOUND" = false ]; then
    echo -e "${GREEN}No crashes found!${NC}"
fi