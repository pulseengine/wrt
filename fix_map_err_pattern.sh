#!/bin/bash

# Fix the systematic map_err pattern error across all Rust files

echo "Fixing map_err pattern errors in wrt-component..."

# Find all Rust files and fix the pattern
find wrt-component -name "*.rs" -type f | while read -r file; do
    # Create a temporary file
    temp_file="${file}.tmp"
    
    # Use perl for multi-line pattern matching and replacement
    perl -0777 -pe 's/\.map_err\(\|_\|\s*\{\s*([^}]+?)\s*\)\s*\}\)/\.map_err(|_| {\n            $1\n        })/gs' "$file" > "$temp_file"
    
    # Check if the file was modified
    if ! cmp -s "$file" "$temp_file"; then
        mv "$temp_file" "$file"
        echo "Fixed: $file"
    else
        rm "$temp_file"
    fi
done

echo "Pattern fix complete."