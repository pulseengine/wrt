#!/bin/bash

# Comprehensive fix for syntax errors in wrt-component
# This script fixes:
# 1. Malformed string literals
# 2. Unmatched parentheses
# 3. Missing closing parentheses

echo "Fixing syntax errors in wrt-component..."

# Fix malformed string literals with pattern: "Error occurred"...Missing message"
find wrt-component/src -name "*.rs" -exec sed -i '' 's/"Error occurred"[^"]*Missing message[^"]*"/"Error occurred"/g' {} \;

# Fix malformed string literals with pattern: "Error occurred"...Component not found...Missing message"
find wrt-component/src -name "*.rs" -exec sed -i '' 's/"Error occurred"[^"]*Component not found[^"]*Missing message[^"]*"/"Error occurred"/g' {} \;

# Fix malformed string literals with pattern: "Error occurred"...Failed to add resource...Missing message"
find wrt-component/src -name "*.rs" -exec sed -i '' 's/"Error occurred"[^"]*Failed to add resource[^"]*Missing message[^"]*"/"Error occurred"/g' {} \;

# Fix unmatched closing parentheses in function calls
find wrt-component/src -name "*.rs" -exec sed -i '' 's/\.to_string();/.to_string());/g' {} \;

# Fix missing closing parentheses in error calls
find wrt-component/src -name "*.rs" -exec sed -i '' 's/Error::runtime_execution_error("Error occurred");/Error::runtime_execution_error("Error occurred"))/g' {} \;

# Fix extra closing parentheses in error handlers
find wrt-component/src -name "*.rs" -exec sed -i '' 's/                )$/            })?;/g' {} \;

echo "Syntax errors fixed! Running cargo check..."
cargo check --package wrt-component