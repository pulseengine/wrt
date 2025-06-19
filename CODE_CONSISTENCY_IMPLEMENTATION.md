# Code Consistency Implementation Plan

## Immediate Action 1: Test File Migration

### Step-by-Step Process for canonical_abi_tests.rs

**File**: `wrt-component/src/canonical_abi_tests.rs` (26KB)
**Target**: `wrt-component/src/canonical_abi/mod.rs`

#### Pre-Migration Checklist
- [ ] Run existing tests: `cargo test -p wrt-component canonical_abi`
- [ ] Note test count and names
- [ ] Check for any special test dependencies

#### Migration Steps

1. **Open the test file**
   ```bash
   # Check current structure
   head -20 wrt-component/src/canonical_abi_tests.rs
   ```

2. **Identify the target module**
   ```bash
   # The canonical_abi directory exists with mod.rs
   ls -la wrt-component/src/canonical_abi/
   ```

3. **Check existing test structure in target**
   ```bash
   # See if mod.rs already has tests
   grep -n "#\[cfg(test)\]" wrt-component/src/canonical_abi/mod.rs
   ```

4. **Prepare the migration**
   ```rust
   // At the end of canonical_abi/mod.rs, add:
   
   #[cfg(test)]
   mod tests {
       use super::*;
       // Test content will go here
   }
   ```

5. **Copy test content**
   - Copy everything from inside `mod tests { }` in canonical_abi_tests.rs
   - Paste into the new test module in mod.rs
   - Adjust imports:
     - Change `use super::super::canonical_abi::*` to `use super::*`
     - Remove redundant imports

6. **Run tests to verify**
   ```bash
   cargo test -p wrt-component canonical_abi
   ```

7. **Clean up**
   ```bash
   # If tests pass, remove the old file
   rm wrt-component/src/canonical_abi_tests.rs
   
   # Update lib.rs if it references the test file
   grep -n "canonical_abi_tests" wrt-component/src/lib.rs
   ```

### Migration Order by Complexity

1. **simple_instantiation_test.rs** (8KB) - Smallest, start here
2. **canonical_abi_tests.rs** (26KB) - Well-organized, clear target
3. **component_instantiation_tests.rs** (25KB) - May need splitting
4. **resource_management_tests.rs** (39KB) - Largest, may need multiple targets

### Common Issues and Solutions

#### Import Path Adjustments
```rust
// Before (in separate test file):
use super::super::module_name::*;
use crate::some_module;

// After (in module's test):
use super::*;
use crate::some_module;
```

#### Test Module Already Exists
If the target module already has a `#[cfg(test)] mod tests`:
1. Add a submodule: `mod migrated_tests { }`
2. Or merge tests carefully
3. Ensure no duplicate test names

#### Circular Dependencies
If tests import from multiple modules:
1. Consider if test belongs elsewhere
2. Use full paths: `crate::other_module::Type`
3. May indicate module organization issue

## Immediate Action 2: Add Module Documentation

### Documentation Addition Process

1. **Generate documentation TODO list**
   ```bash
   # Find all .rs files without module docs
   find . -name "*.rs" -type f -exec sh -c \
     'if ! head -5 "$1" | grep -q "^//!"; then echo "$1"; fi' _ {} \; \
     | grep -v target | grep -v test | sort
   ```

2. **Template for adding docs**
   ```rust
   //! Module name based on file name.
   //!
   //! Brief description of module purpose.
   
   // Existing imports and code follow...
   ```

3. **Priority modules for documentation**
   - Core runtime modules (execution critical)
   - Public API modules
   - Safety-critical modules

### Quick Documentation Script
```bash
#!/bin/bash
# Add basic module documentation to a file

FILE=$1
MODULE_NAME=$(basename "$FILE" .rs)

# Check if already has module doc
if head -5 "$FILE" | grep -q "^//!"; then
    echo "$FILE already has module documentation"
    exit 0
fi

# Create temp file with documentation
cat > temp_doc.txt << EOF
//! ${MODULE_NAME} module.
//!
//! TODO: Add proper documentation for this module.

EOF

# Prepend to file
cat temp_doc.txt "$FILE" > "$FILE.tmp"
mv "$FILE.tmp" "$FILE"
rm temp_doc.txt

echo "Added basic documentation to $FILE"
```

## Immediate Action 3: Remove unwrap() from Production

### Systematic unwrap() Removal

1. **Find all unwrap() calls**
   ```bash
   # Find unwrap() excluding tests
   rg "\.unwrap\(\)" --type rust -g '!test' -g '!tests' \
     | grep -v "#\[cfg(test)\]" -B2 -A2
   ```

2. **Categorize unwrap() usage**
   - Initialization (convert to proper error handling)
   - After explicit checks (add safety comment)
   - In infallible operations (document why)
   - Lazy/temporary code (must fix)

3. **Replacement Patterns**

   **Pattern 1: Initialization**
   ```rust
   // Before:
   let adapter = create_adapter().unwrap();
   
   // After:
   let adapter = create_adapter()
       .map_err(|e| Error::new(ErrorCategory::Initialization, "Failed to create adapter"))?;
   ```

   **Pattern 2: After checks**
   ```rust
   // Before:
   if vec.len() > index {
       let val = vec.get(index).unwrap();
   }
   
   // After:
   if vec.len() > index {
       // SAFETY: We just checked index is within bounds
       let val = vec.get(index).unwrap();
   }
   
   // Even better:
   if let Some(val) = vec.get(index) {
       // use val
   }
   ```

   **Pattern 3: Infallible operations**
   ```rust
   // Before:
   let num = "42".parse::<u32>().unwrap();
   
   // After (if truly constant):
   // SAFETY: Parsing a valid numeric literal cannot fail
   let num = "42".parse::<u32>().unwrap();
   
   // Better:
   const NUM: u32 = 42;
   ```

### Validation Process

After each change:
1. Run tests for affected module
2. Check error propagation makes sense
3. Ensure error messages are helpful
4. Verify no panic paths remain

## Tooling Support

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check for test files in src/
if find . -name "*test*.rs" -path "*/src/*" -not -path "*/target/*" | grep -q .; then
    echo "ERROR: Test files found in src/ directory"
    echo "Move tests to #[cfg(test)] modules"
    exit 1
fi

# Check for unwrap without safety comment
if git diff --cached --name-only | xargs grep -l "\.unwrap()" | \
   xargs grep -B1 "\.unwrap()" | grep -v "SAFETY:" | grep -q "unwrap"; then
    echo "WARNING: unwrap() without SAFETY comment"
fi
```

### CI Integration
```yaml
# In CI workflow
- name: Check code consistency
  run: |
    # No test files in src/
    ! find . -name "*test*.rs" -path "*/src/*" -not -path "*/target/*" | grep -q .
    
    # All modules have documentation
    ! find . -name "*.rs" -exec sh -c \
      'if ! head -5 "$1" | grep -q "^//!"; then exit 1; fi' _ {} \;
```

## Success Metrics

1. **Week 1**: All test files migrated
   - 0 test files in src/ directories
   - All tests still pass
   
2. **Week 2**: Documentation complete
   - 100% of modules have `//!` docs
   - Public items documented
   
3. **Week 3**: unwrap() cleaned up
   - No unwrap() without safety docs
   - Proper error propagation

## Next Steps

1. Run the migration script to get current status
2. Start with smallest test file (simple_instantiation_test.rs)
3. Document as you go - what worked, what didn't
4. Create PR for each major component
5. Update CI to prevent regression