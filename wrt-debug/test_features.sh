#!/bin/bash
# Test all feature combinations for wrt-debug

echo "Testing wrt-debug feature combinations..."
echo "========================================"

# Function to test a feature combination
test_feature() {
    local features="$1"
    local desc="$2"
    echo -n "Testing $desc... "
    
    if cargo check --no-default-features --features "$features" 2>/dev/null; then
        echo "✅ OK"
        return 0
    else
        echo "❌ FAILED"
        # Show first few errors
        echo "  Errors:"
        cargo check --no-default-features --features "$features" 2>&1 | grep -E "error\[E[0-9]+\]" | head -5 | sed 's/^/    /'
        return 1
    fi
}

# Test individual features
echo -e "\n1. Individual Features:"
test_feature "" "no features"
test_feature "line-info" "line-info only"
test_feature "abbrev" "abbrev only"
test_feature "debug-info" "debug-info only"
test_feature "function-info" "function-info only"
test_feature "runtime-inspection" "runtime-inspection only"
test_feature "runtime-variables" "runtime-variables only"
test_feature "runtime-memory" "runtime-memory only"
test_feature "runtime-control" "runtime-control only"
test_feature "runtime-breakpoints" "runtime-breakpoints only"
test_feature "runtime-stepping" "runtime-stepping only"

# Test feature groups
echo -e "\n2. Feature Groups:"
test_feature "static-debug" "static-debug (all static features)"
test_feature "runtime-debug" "runtime-debug (all runtime features)"
test_feature "minimal" "minimal"
test_feature "production" "production"
test_feature "development" "development"
test_feature "full-debug" "full-debug (everything)"

# Test common combinations
echo -e "\n3. Common Combinations:"
test_feature "line-info,function-info" "line-info + function-info"
test_feature "static-debug,runtime-inspection" "static + inspection"
test_feature "static-debug,runtime-variables" "static + variables"
test_feature "static-debug,runtime-breakpoints" "static + breakpoints"

# Test dependencies work correctly
echo -e "\n4. Dependency Chains:"
test_feature "function-info" "function-info (should enable debug-info,abbrev)"
test_feature "runtime-variables" "runtime-variables (should enable runtime-inspection)"
test_feature "runtime-stepping" "runtime-stepping (should enable runtime-control)"

echo -e "\n========================================"
echo "Feature testing complete!"