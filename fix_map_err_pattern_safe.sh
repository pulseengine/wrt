#!/bin/bash

# Fix the systematic map_err pattern error across all Rust files
# This version creates backups before making changes

BACKUP_DIR="wrt-component-backup-$(date +%Y%m%d_%H%M%S)"

echo "Creating backup directory: $BACKUP_DIR"
mkdir -p "$BACKUP_DIR"

# First, create a full backup of wrt-component
echo "Backing up wrt-component directory..."
cp -r wrt-component "$BACKUP_DIR/"

echo "Fixing map_err pattern errors in wrt-component..."

# Count total files to process
total_files=$(find wrt-component -name "*.rs" -type f | wc -l)
echo "Found $total_files Rust files to check"

fixed_count=0

# Find all Rust files and fix the pattern
find wrt-component -name "*.rs" -type f | while read -r file; do
    # Create a temporary file
    temp_file="${file}.tmp"
    
    # First check if the file contains the problematic pattern
    if grep -q "map_err.*{" "$file" && grep -A 3 "map_err.*{" "$file" | grep -q "^\s*)" ; then
        echo "Processing: $file"
        
        # Use sed to fix the specific pattern
        # This is more conservative than perl - it only fixes the exact pattern we know is broken
        sed -e '/\.map_err(|_| {/{
            N
            N
            s/\(.*map_err(|_| {\)\n\(\s*\)\(.*\)\n\s*)/\1\n\2\3/
        }' "$file" > "$temp_file"
        
        # Check if the file was actually modified
        if ! cmp -s "$file" "$temp_file"; then
            # Create individual backup
            cp "$file" "${file}.backup"
            mv "$temp_file" "$file"
            echo "  ✓ Fixed: $file"
            ((fixed_count++))
        else
            rm "$temp_file"
        fi
    fi
done

echo ""
echo "Pattern fix complete!"
echo "Files fixed: $fixed_count"
echo "Full backup saved in: $BACKUP_DIR"
echo ""
echo "To restore from backup if needed:"
echo "  rm -rf wrt-component && cp -r $BACKUP_DIR/wrt-component ."