#!/bin/bash

# Fix malformed error messages in a single file
# Usage: ./fix_single_file_errors.sh <file>

file="$1"

if [ -z "$file" ]; then
    echo "Usage: $0 <file>"
    exit 1
fi

if [ ! -f "$file" ]; then
    echo "File not found: $file"
    exit 1
fi

echo "Fixing error messages in $file..."

# Create a backup
cp "$file" "$file.bak"

# Fix the malformed error patterns
# Pattern: "Error occurred"Some message"Missing message"* -> "Some message"
sed -i -E 's/"Error occurred"([^"]+)(Missing message)+"/"\1"/g' "$file"

# Show what was changed
if diff -q "$file.bak" "$file" > /dev/null; then
    echo "No changes needed in $file"
    rm "$file.bak"
else
    echo "Fixed error messages in $file"
    # Optionally show the diff
    # diff "$file.bak" "$file"
    rm "$file.bak"
fi