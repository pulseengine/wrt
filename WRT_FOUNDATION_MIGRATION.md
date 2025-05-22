# wrt-types to wrt-foundation Migration Plan

## Overview

This document outlines the process and rationale for renaming the `wrt-types` crate to `wrt-foundation`. The migration aims to better reflect the crate's purpose as a foundational layer providing essential building blocks for the entire WRT ecosystem, rather than just a collection of type definitions.

## Motivation

The current name `wrt-types` does not accurately represent the breadth and depth of the crate's functionality:

1. The crate provides more than just type definitions - it includes memory safety primitives, bounded collections, builder patterns, and other foundational elements of the runtime.
2. The crate acts as a core dependency for most other WRT crates, serving as a foundation layer.
3. The name "types" might suggest to newcomers that it's simply a collection of typedefs, when it actually provides critical safety and abstraction mechanisms.

## Migration Steps

The migration will be executed in the following phases:

### Phase 1: Preparation (Current)

1. Create this migration document
2. Rename the crate directory:
   ```bash
   git mv wrt-types wrt-foundation
   ```
3. Update the crate's `Cargo.toml` to use the new name
4. Update the `README.md` to reflect the new name and expanded purpose description

### Phase 2: Workspace Integration

1. Update the workspace-level `Cargo.toml` to:
   - Change the member from `wrt-types` to `wrt-foundation`
   - Update the workspace dependencies to reference `wrt-foundation` instead of `wrt-types`

2. Create a transition package:
   - Create a minimal `wrt-types` crate that depends on `wrt-foundation` and re-exports its contents
   - Mark the transition package as deprecated in its `Cargo.toml`
   - Add clear documentation about the migration

### Phase 3: Update Dependencies

Update all crates that depend on `wrt-types` to now depend on `wrt-foundation`:

1. Update all internal workspace crates:
   - `wrt`
   - `wrt-component`
   - `wrt-decoder`
   - `wrt-error`
   - `wrt-format`
   - `wrt-host`
   - `wrt-instructions`
   - `wrt-intercept`
   - `wrt-logging`
   - `wrt-math`
   - `wrt-platform`
   - `wrt-runtime`
   - `wrt-sync`
   - `wrt-test-registry`
   - `wrt-verification-tool`
   - `wrtd`

2. Ensure integration tests continue to pass

### Phase 4: Documentation Updates

1. Update documentation references to use the new name
2. Update code examples in the documentation
3. Add a note in relevant crates about the name change

### Phase 5: Final Verification

1. Run the full test suite, ensuring everything passes
2. Verify that all import paths have been updated
3. Check that the transition package correctly re-exports everything

## Backwards Compatibility

To maintain backwards compatibility:

1. The transition package (`wrt-types`) will continue to work for existing code
2. The API will remain unchanged, only the name and organizational structure are affected
3. Documentation will clearly indicate the migration path for consumers

## Timeline

1. Phase 1 (Preparation): Immediate
2. Phase 2 (Workspace Integration): Within 1 week
3. Phase 3 (Update Dependencies): Within 2 weeks
4. Phase 4 (Documentation Updates): Within 3 weeks
5. Phase 5 (Final Verification): Within 4 weeks

## Post-Migration

After the migration is complete:

1. Monitor for any issues reported by users
2. Plan for eventual removal of the transition package in a future major version
3. Update any external documentation or references