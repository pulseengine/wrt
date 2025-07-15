#!/bin/bash

# Run WAST tests excluding problematic files
echo "Running WebAssembly Test Suite (excluding names.wast)..."
echo "=================================================="

total_files=0
total_passed=0
total_assertions=0

# Get all .wast files except names.wast
for file in external/testsuite/*.wast; do
    basename=$(basename "$file")
    
    # Skip names.wast (issue #94)
    if [ "$basename" = "names.wast" ]; then
        continue
    fi
    
    # Run the test and capture output
    output=$(cargo-wrt testsuite --run-wast --wast-filter "$basename" 2>&1)
    
    # Extract statistics
    files=$(echo "$output" | grep "Files processed:" | awk '{print $3}')
    assertions=$(echo "$output" | grep "Assertions passed:" | awk '{print $3}')
    
    if [ -n "$files" ] && [ -n "$assertions" ]; then
        echo "✓ $basename: $files files, $assertions assertions"
        total_files=$((total_files + files))
        total_assertions=$((total_assertions + assertions))
        total_passed=$((total_passed + 1))
    else
        echo "✗ $basename: Failed to parse or execute"
    fi
done

echo "=================================================="
echo "Summary:"
echo "  WAST files tested: $total_passed"
echo "  Total test files: $total_files"
echo "  Total assertions: $total_assertions"