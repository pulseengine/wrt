#!/bin/bash

# Fix malformed error messages in wrt-component
# Pattern: "Error occurred"Some message" -> "Some message"

find wrt-component/src -name "*.rs" -type f | while read -r file; do
    echo "Processing $file..."
    # Use sed to fix the pattern
    sed -i.bak -E 's/"Error occurred"([^"]+)(Missing message)+"/"\\1"/g' "$file"
done

# Remove backup files
find wrt-component/src -name "*.rs.bak" -type f -delete

echo "Fixed error messages"