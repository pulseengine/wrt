#!/bin/bash

# Comprehensive fix for systematic syntax errors in wrt-component
# Creates backups and fixes multiple patterns

BACKUP_DIR="wrt-component-backup-$(date +%Y%m%d_%H%M%S)"

echo "Creating backup directory: $BACKUP_DIR"
mkdir -p "$BACKUP_DIR"

# Create a full backup
echo "Backing up wrt-component directory..."
cp -r wrt-component "$BACKUP_DIR/"

echo "=== Starting comprehensive fixes ==="

# Fix 1: Incomplete feature flags
echo ""
echo "1. Fixing incomplete feature flags..."
find wrt-component -name "*.rs" -type f | while read -r file; do
    if grep -q '#\[cfg(feature = "\]' "$file"; then
        echo "  Fixing feature flags in: $file"
        sed -i.bak 's/#\[cfg(feature = "\]/#\[cfg(feature = "std"\]/g' "$file"
    fi
done

# Fix 2: Empty string literals in Error::new patterns
echo ""
echo "2. Fixing empty string literals in Error::new..."
find wrt-component -name "*.rs" -type f | while read -r file; do
    # Fix pattern: codes::SOME_CODE,\n                ")),
    if grep -B1 '^\s*"' "$file" | grep -q 'codes::'; then
        echo "  Fixing Error::new strings in: $file"
        # This perl command fixes multi-line patterns
        perl -i.bak -0pe 's/(codes::\w+,)\s*\n(\s*)"(\)\))/\1\n\2"Error message needed"\3/g' "$file"
    fi
done

# Fix 3: Double closing parentheses in map_err
echo ""
echo "3. Fixing double closing parentheses in error handling..."
find wrt-component -name "*.rs" -type f | while read -r file; do
    if grep -q '))\s*$' "$file"; then
        echo "  Checking for error patterns in: $file"
        # Fix the specific pattern we've seen
        perl -i.bak -0pe 's/(\s*\)\s*\n\s*\)\s*\}\))(\?;|\s*$)/)\1\2/g' "$file"
    fi
done

# Fix 4: String literals with only closing quote
echo ""
echo "4. Fixing string literals with only closing quote..."
find wrt-component -name "*.rs" -type f | while read -r file; do
    # Look for patterns like: some_function(")
    if grep -E '\("[)"]' "$file" > /dev/null 2>&1; then
        echo "  Fixing string literals in: $file"
        # Fix common patterns
        sed -i.bak 's/("\)/"test")/g' "$file"
        sed -i.bak 's/(";\)/"test";/g' "$file"
    fi
done

# Fix 5: Check for odd number of quotes per line (catches other issues)
echo ""
echo "5. Finding remaining quote issues..."
find wrt-component -name "*.rs" -type f | while read -r file; do
    odd_quotes=$(grep -n '"' "$file" | awk '{line = $0; count = gsub(/"/, "", line); if (count % 2 == 1) print $0}' | head -5)
    if [ ! -z "$odd_quotes" ]; then
        echo "  WARNING: Odd quotes found in $file:"
        echo "$odd_quotes" | head -3
    fi
done

# Count total backup files created
backup_count=$(find wrt-component -name "*.bak" | wc -l)
echo ""
echo "=== Fix complete ==="
echo "Backup files created: $backup_count"
echo "Full backup saved in: $BACKUP_DIR"
echo ""
echo "To check if fixes worked:"
echo "  cargo check --package wrt-component"
echo ""
echo "To restore from backup if needed:"
echo "  rm -rf wrt-component && cp -r $BACKUP_DIR/wrt-component ."
echo ""
echo "To clean up .bak files after verification:"
echo "  find wrt-component -name '*.bak' -delete"