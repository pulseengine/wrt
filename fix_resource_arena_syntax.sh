#!/bin/bash

# Fix syntax errors in resource_arena_no_std.rs

echo "Fixing syntax errors in resource_arena_no_std.rs..."

# Fix missing closing parentheses
sed -i '' 's/let table = Mutex::new(ResourceTable::new().unwrap();/let table = Mutex::new(ResourceTable::new().unwrap());/g' wrt-component/src/resources/resource_arena_no_std.rs

# Fix unmatched assert! parentheses
sed -i '' 's/assert!(arena.has_resource(ResourceId(handle)).unwrap();/assert!(arena.has_resource(ResourceId(handle)).unwrap());/g' wrt-component/src/resources/resource_arena_no_std.rs

# Fix malformed string literals
sed -i '' 's/"test-arenaMissing messageMissing messageMissing message"/"test-arena"/g' wrt-component/src/resources/resource_arena_no_std.rs

# Fix missing closing parentheses in assert
sed -i '' 's/assert!(result.is_err();/assert!(result.is_err());/g' wrt-component/src/resources/resource_arena_no_std.rs

echo "Syntax errors fixed!"