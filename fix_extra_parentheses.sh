#!/bin/bash

# Fix extra closing parentheses in error calls in async_canonical.rs

file="wrt-component/src/async_/async_canonical.rs"

echo "Fixing extra parentheses in $file"

# Fix pattern: Error::error_method("message"))) -> Error::error_method("message"))
perl -i -pe 's/Error::[a-zA-Z_]+\("([^"]+)"\)\)\)/Error::$1("$2"))/g' "$file"

# Fix pattern: .map_err(|_| Error::error_method("message")))) -> .map_err(|_| Error::error_method("message")))
perl -i -pe 's/Error::[a-zA-Z_]+\("([^"]+)"\)\)\)\)/Error::$1("$2")))/g' "$file"

# Fix pattern: Error::error_method("message"))) -> Error::error_method("message"))
perl -i -pe 's/wrt_error::Error::[a-zA-Z_]+\("([^"]+)"\)\)\)/wrt_error::Error::$1("$2"))/g' "$file"

# Fix return statements with extra parentheses
perl -i -pe 's/return Err\(wrt_error::Error::[a-zA-Z_]+\("([^"]+)"\)\)\);/return Err(wrt_error::Error::$1("$2"));/g' "$file"

# Fix specific patterns we found
perl -i -pe 's/Error::runtime_execution_error\("Invalid handle"\)\)/Error::runtime_execution_error("Invalid handle")/g' "$file"
perl -i -pe 's/Error::runtime_execution_error\("Stream is closed"\)\)/Error::runtime_execution_error("Stream is closed")/g' "$file"
perl -i -pe 's/Error::resource_exhausted\("Buffer full"\)\)/Error::resource_exhausted("Buffer full")/g' "$file"
perl -i -pe 's/Error::type_mismatch_error\("Value type mismatch"\)\)/Error::type_mismatch_error("Value type mismatch")/g' "$file"
perl -i -pe 's/Error::type_mismatch_error\("Value type mismatch for future"\)\)/Error::type_mismatch_error("Value type mismatch for future")/g' "$file"

echo "Fixed extra parentheses in $file"