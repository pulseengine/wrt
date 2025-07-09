#!/bin/bash

# More aggressive fixes for wrt-component syntax errors

echo "=== Aggressive syntax fixes for wrt-component ==="

# Fix 1: ALL incomplete feature flags
echo "1. Fixing ALL incomplete feature flags..."
find wrt-component -name "*.rs" -type f | while read -r file; do
    # Fix any cfg with incomplete quotes
    if grep -q '#\[cfg(feature = "' "$file"; then
        sed -i.bak2 's/#\[cfg(feature = "\]/#\[cfg(feature = "std"\]/g' "$file"
        sed -i.bak2 's/#\[cfg(not(feature = "\]/#\[cfg(not(feature = "std"\]/g' "$file"
        sed -i.bak2 's/#\[cfg(not(feature = "))\]/#\[cfg(not(feature = "std"))\]/g' "$file"
        sed -i.bak2 's/#\[cfg(feature = ")\]/#\[cfg(feature = "std")\]/g' "$file"
        sed -i.bak2 's/#\[cfg(not(any(feature = "))\]/#\[cfg(not(any(feature = "std")))\]/g' "$file"
    fi
done

# Fix 2: String literals with only closing quote in error messages
echo "2. Fixing string literals with only closing quotes..."
find wrt-component -name "*.rs" -type f | while read -r file; do
    # Fix patterns like: Error::some_error("
    sed -i.bak3 's/Error::\([a-z_]*\)("/Error::\1("Missing error message"/g' "$file"
    
    # Fix patterns like: error(",  
    sed -i.bak3 's/error(",/error("Missing error message",/g' "$file"
    
    # Fix patterns like: "))
    sed -i.bak3 's/"))/Missing message"))/g' "$file"
    
    # Fix patterns ending with just "
    sed -i.bak3 's/\("[[:space:]]*$/\1Missing message"/g' "$file"
done

# Fix 3: Fix specific known patterns
echo "3. Fixing specific error patterns..."
find wrt-component -name "*.rs" -type f | while read -r file; do
    # Fix multi-line error patterns
    perl -i.bak4 -0pe 's/Error::runtime_execution_error\("\s*\)\)/Error::runtime_execution_error("Error occurred")/gs' "$file"
    perl -i.bak4 -0pe 's/Error::([a-z_]+)\("\s*\)\)/Error::\1("Error occurred")/gs' "$file"
    
    # Fix patterns with just closing quote and parentheses
    perl -i.bak4 -0pe 's/"\)\)/Missing message"))/gs' "$file"
    perl -i.bak4 -0pe 's/"\)/Missing message")/gs' "$file"
done

# Fix 4: Clean up any remaining double parentheses
echo "4. Cleaning up double parentheses..."
find wrt-component -name "*.rs" -type f | while read -r file; do
    # Remove extra closing parentheses at end of lines
    sed -i.bak5 's/));\s*$/);/g' "$file"
    sed -i.bak5 's/))\s*$/)/g' "$file"
done

# Fix 5: Final check and report
echo ""
echo "5. Checking for remaining issues..."
remaining_issues=0
find wrt-component -name "*.rs" -type f | while read -r file; do
    # Check for odd quotes
    odd_quotes=$(grep -n '"' "$file" | awk '{line = $0; count = gsub(/"/, "", line); if (count % 2 == 1) print $0}' | wc -l)
    if [ "$odd_quotes" -gt 0 ]; then
        echo "  WARNING: $file still has $odd_quotes lines with odd quotes"
        ((remaining_issues++))
    fi
done

echo ""
echo "=== Aggressive fixes complete ==="
echo "Created multiple backup levels (.bak2, .bak3, .bak4, .bak5)"
echo ""
echo "To clean up all backup files:"
echo "  find wrt-component -name '*.bak*' -delete"