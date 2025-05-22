# Incremental Migration Plan for wrt-foundation to wrt-foundation

Given the complexity of the codebase and number of errors encountered during the initial migration attempt, we need to take a more incremental approach to ensure a smooth transition.

## Phase 1: Transition Package Setup (Completed)

- [x] Create migration documentation
- [x] Create `wrt-foundation` with updated package metadata
- [x] Create `wrt-foundation-transition` package for backwards compatibility
- [x] Update workspace configuration in root `Cargo.toml`

## Phase 2: Prepare Source Migration (Next Steps)

1. Copy one module at a time from `wrt-foundation` to `wrt-foundation` and fix any errors
   - Start with core modules that have minimal dependencies:
     - [ ] prelude.rs (fix imports)
     - [ ] bounded.rs
     - [ ] traits.rs
     - [ ] types.rs
     - [ ] values.rs
     - [ ] verification.rs
   - Then move to more complex modules:
     - [ ] safe_memory.rs
     - [ ] component modules
     - [ ] other modules

2. Fix cross-module references and imports:
   - [ ] Address issues with `MAX_WASM_NAME_LENGTH` and other cfg-gated constants
   - [ ] Fix duplicate imports and references
   - [ ] Ensure feature flags work correctly

## Phase 3: Fix API Consistency

- [ ] Update references in dependent crates one at a time:
  - [ ] wrt-error
  - [ ] wrt-sync
  - [ ] wrt-format
  - [ ] wrt-decoder
  - [ ] wrt-runtime
  - [ ] wrt-component
  - [ ] wrt-host
  - [ ] wrt-intercept

- [ ] Fix import statements from `wrt_foundation` to `wrt_foundation`

## Phase 4: Testing and Verification

- [ ] Ensure all crates compile successfully
- [ ] Run the test suite for each crate
- [ ] Run integration tests
- [ ] Validate feature combinations

## Phase 5: Final Implementation

- [ ] Remove `wrt-foundation` crate completely
- [ ] Keep only `wrt-foundation` and `wrt-foundation-transition` (for backward compatibility)
- [ ] Update documentation

## Recommendations

1. **Module-by-Module Approach**: Rather than trying to migrate everything at once, focus on one module at a time, starting with those with the fewest dependencies.

2. **Fix Core Modules First**: The prelude, bounded collections, and basic types should be prioritized as they are used extensively.

3. **Incremental Testing**: After each module is migrated, compile the codebase to catch errors early.

4. **Feature Flag Consistency**: Pay special attention to feature-gated code, ensuring that features are defined consistently across all crates.

5. **Update One Dependent Crate at a Time**: After core modules are working, update dependent crates one by one, starting with the lowest-level ones.

## Immediate Action Items

1. Fix the `prelude.rs` in `wrt-foundation` to ensure it correctly handles imports for both std and no_std environments
2. Address the cfg-gated constants like `MAX_WASM_NAME_LENGTH`
3. Fix duplicate imports and references
4. Update dependent crates to use `wrt-foundation` instead of `wrt-foundation`

This incremental approach will help manage the complexity of the migration and ensure a stable transition to the new naming.