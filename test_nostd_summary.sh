#!/bin/bash

echo "=== No-Std Compatibility Test Summary ==="
echo ""
echo "Testing each crate with: no_std, no_std+alloc, std"
echo ""

# Test foundational crates first
for crate in wrt-error wrt-math wrt-sync wrt-foundation wrt-platform wrt-intercept wrt-logging; do
    echo -n "$crate: "
    
    # no_std
    if cargo build -p $crate --no-default-features >/dev/null 2>&1; then
        echo -n "✅ "
    else
        echo -n "❌ "
    fi
    
    # no_std + alloc
    if cargo build -p $crate --no-default-features --features alloc >/dev/null 2>&1; then
        echo -n "✅ "
    else
        echo -n "❌ "
    fi
    
    # std
    if cargo build -p $crate --features std >/dev/null 2>&1; then
        echo "✅"
    else
        echo "❌"
    fi
done

echo ""
echo "Testing dependent crates..."
echo ""

# Test dependent crates
for crate in wrt-format wrt-decoder wrt-instructions wrt-host wrt-runtime wrt-component wrt; do
    echo -n "$crate: "
    
    # no_std
    if cargo build -p $crate --no-default-features >/dev/null 2>&1; then
        echo -n "✅ "
    else
        echo -n "❌ "
    fi
    
    # no_std + alloc
    if cargo build -p $crate --no-default-features --features alloc >/dev/null 2>&1; then
        echo -n "✅ "
    else
        echo -n "❌ "
    fi
    
    # std
    if cargo build -p $crate --features std >/dev/null 2>&1; then
        echo "✅"
    else
        echo "❌"
    fi
done