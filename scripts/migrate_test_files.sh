#!/bin/bash
# Script to help migrate test files from src/ to proper test modules

set -e

echo "Test File Migration Helper"
echo "========================="

# Function to check if a file has a corresponding module
find_module_for_test() {
    local test_file=$1
    local base_name=$(basename "$test_file" _tests.rs)
    base_name=$(echo "$base_name" | sed 's/_test$//')
    
    echo "Looking for module for: $test_file (base: $base_name)"
    
    # Check for exact match
    if [ -f "${base_name}.rs" ]; then
        echo "  Found: ${base_name}.rs"
        return 0
    fi
    
    # Check in subdirectories
    local module_dir=$(dirname "$test_file")
    if [ -f "${module_dir}/${base_name}/mod.rs" ]; then
        echo "  Found: ${module_dir}/${base_name}/mod.rs"
        return 0
    fi
    
    echo "  No direct module match found"
    return 1
}

# Function to analyze a test file
analyze_test_file() {
    local test_file=$1
    echo ""
    echo "Analyzing: $test_file"
    echo "  Size: $(ls -lh "$test_file" | awk '{print $5}')"
    echo "  Lines: $(wc -l < "$test_file")"
    
    # Check what it imports
    echo "  Imports from:"
    grep -E "use (super|crate)::" "$test_file" | head -5 | sed 's/^/    /'
    
    # Count number of test functions
    local test_count=$(grep -c "#\[test\]" "$test_file" || true)
    echo "  Test functions: $test_count"
    
    # Check if it already has #[cfg(test)]
    if grep -q "#\[cfg(test)\]" "$test_file"; then
        echo "  ⚠️  Already has #[cfg(test)] wrapper"
    fi
    
    find_module_for_test "$test_file"
}

# List all test files in src directories
echo "Finding test files in src/ directories..."
echo ""

TEST_FILES=$(find . -name "*test*.rs" -path "*/src/*" -not -path "*/target/*" | sort)

if [ -z "$TEST_FILES" ]; then
    echo "No test files found in src/ directories"
    exit 0
fi

echo "Found test files:"
echo "$TEST_FILES" | sed 's/^/  /'

# Analyze each test file
for test_file in $TEST_FILES; do
    analyze_test_file "$test_file"
done

echo ""
echo "Migration Steps:"
echo "================"
echo ""
echo "1. For each test file:"
echo "   - Check if it already has #[cfg(test)] mod tests wrapper"
echo "   - Find the corresponding module it should be moved to"
echo "   - Copy the test content to the module's #[cfg(test)] section"
echo "   - Update imports (super:: may need adjustment)"
echo "   - Run tests to ensure they still work"
echo "   - Delete the original test file"
echo ""
echo "2. Priority order (by size/complexity):"
echo "$TEST_FILES" | xargs ls -la | sort -k5 -n -r | awk '{print "   - " $9 " (" $5 " bytes)"}'

echo ""
echo "3. Example migration command:"
echo "   # For canonical_abi_tests.rs -> canonical_abi module"
echo "   # 1. Open both files"
echo "   # 2. Copy test module content"  
echo "   # 3. Paste into canonical_abi/mod.rs at the end"
echo "   # 4. Adjust imports if needed"
echo "   # 5. Run: cargo test -p wrt-component"
echo "   # 6. If passes: rm canonical_abi_tests.rs"