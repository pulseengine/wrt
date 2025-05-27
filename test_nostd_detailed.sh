#!/bin/bash

echo "=== Detailed No-Std Compatibility Test ==="
echo ""

# Define test results array
declare -A results

# Test each crate
for crate in wrt-error wrt-math wrt-sync wrt-foundation wrt-format wrt-decoder wrt-instructions wrt-runtime wrt-host wrt-intercept wrt-component wrt-platform wrt-logging wrt; do
    echo "=== Testing $crate ==="
    
    # Test build
    echo -n "  Build (no_std): "
    if cargo build -p $crate --no-default-features 2>/dev/null; then
        echo "✅"
        results["$crate-build"]="pass"
    else
        echo "❌"
        results["$crate-build"]="fail"
        # Show first few errors
        echo "  Errors:"
        cargo build -p $crate --no-default-features 2>&1 | head -20 | sed 's/^/    /'
    fi
    
    # Test with alloc
    echo -n "  Build (no_std + alloc): "
    if cargo build -p $crate --no-default-features --features alloc 2>/dev/null; then
        echo "✅"
        results["$crate-alloc"]="pass"
    else
        echo "❌"
        results["$crate-alloc"]="fail"
    fi
    
    # Test with std
    echo -n "  Build (std): "
    if cargo build -p $crate --features std 2>/dev/null; then
        echo "✅"
        results["$crate-std"]="pass"
    else
        echo "❌"
        results["$crate-std"]="fail"
    fi
    
    echo ""
done

# Summary
echo "=== Summary ==="
echo ""
echo "| Crate | no_std | no_std+alloc | std |"
echo "|-------|--------|--------------|-----|"
for crate in wrt-error wrt-math wrt-sync wrt-foundation wrt-format wrt-decoder wrt-instructions wrt-runtime wrt-host wrt-intercept wrt-component wrt-platform wrt-logging wrt; do
    no_std_result=${results["$crate-build"]:-"untested"}
    alloc_result=${results["$crate-alloc"]:-"untested"}
    std_result=${results["$crate-std"]:-"untested"}
    
    # Convert to symbols
    [[ "$no_std_result" == "pass" ]] && no_std_result="✅" || no_std_result="❌"
    [[ "$alloc_result" == "pass" ]] && alloc_result="✅" || alloc_result="❌"
    [[ "$std_result" == "pass" ]] && std_result="✅" || std_result="❌"
    
    printf "| %-15s | %-6s | %-12s | %-3s |\n" "$crate" "$no_std_result" "$alloc_result" "$std_result"
done