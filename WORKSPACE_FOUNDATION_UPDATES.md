# Workspace Dependencies Update Plan for wrt-foundation

This document outlines the necessary changes to update the workspace dependencies after renaming `wrt-foundation` to `wrt-foundation`.

## Workspace-Level Changes

### 1. Update Cargo.toml Workspace Members

In the root `Cargo.toml`, update the members list:

```toml
[workspace]
members = [
    "wrt",
    "wrtd",
    "xtask",
    "example",
    "wrt-sync",
    "wrt-error",
    "wrt-format",
    "wrt-foundation", # Changed from wrt-foundation
    "wrt-foundation",      # Add back as transition package
    "wrt-decoder",
    # ... other members
]
```

### 2. Update Workspace Dependencies

In the root `Cargo.toml`, update the workspace dependencies:

```toml
[workspace.dependencies]
# ... existing dependencies
wrt-foundation = { path = "wrt-foundation", version = "0.2.0", default-features = false }
wrt-foundation = { path = "wrt-foundation", version = "0.2.0", default-features = false } # Points to transition package
# ... other dependencies
```

## Per-Crate Dependencies

The following crates need their dependencies updated to use `wrt-foundation` instead of `wrt-foundation`:

1. `wrt/Cargo.toml`
2. `wrt-component/Cargo.toml`
3. `wrt-decoder/Cargo.toml`
4. `wrt-format/Cargo.toml`
5. `wrt-host/Cargo.toml`
6. `wrt-instructions/Cargo.toml`
7. `wrt-intercept/Cargo.toml`
8. `wrt-logging/Cargo.toml`
9. `wrt-platform/Cargo.toml`
10. `wrt-runtime/Cargo.toml`
11. `wrtd/Cargo.toml`

For each crate, the change is:

```diff
[dependencies]
- wrt-foundation = { workspace = true }
+ wrt-foundation = { workspace = true }
```

## Code Changes

In each affected crate, update the imports:

```diff
- use wrt_foundation::{...};
+ use wrt_foundation::{...};
```

## Transition Package Creation

Create a minimal `wrt-foundation` crate that re-exports everything from `wrt-foundation`:

1. Create the directory structure:
   ```
   wrt-foundation/
   ├── Cargo.toml
   ├── README.md
   └── src/
       └── lib.rs
   ```

2. In `wrt-foundation/Cargo.toml`:
   ```toml
   [package]
   name = "wrt-foundation"
   version.workspace = true
   edition.workspace = true
   description = "DEPRECATED: Transition package for wrt-foundation. Use wrt-foundation instead."
   readme = "README.md"
   license.workspace = true
   repository.workspace = true
   keywords = ["wasm", "webassembly", "deprecated"]
   categories = ["wasm"]
   deprecated = true

   [dependencies]
   wrt-foundation = { workspace = true }
   ```

3. In `wrt-foundation/src/lib.rs`:
   ```rust
   //! # DEPRECATED
   //! 
   //! This crate has been renamed to `wrt-foundation`.
   //! 
   //! Please update your dependencies to use `wrt-foundation` instead.
   //! 
   //! This is a compatibility shim that re-exports all items from `wrt-foundation`.
   
   #![deprecated(
       since = "0.2.0",
       note = "This crate has been renamed to wrt-foundation. Please update your dependencies."
   )]
   
   pub use wrt_foundation::*;
   ```

## Testing Plan

1. After making all changes, run the full test suite:
   ```
   cargo test --workspace
   ```

2. Verify that all crates compile with the new dependencies:
   ```
   cargo check --workspace
   ```

3. Run a specific integration test that covers multiple crates:
   ```
   cargo test -p wrt -- integration_test
   ```

## Rollback Plan

If issues are encountered:

1. Revert all changes
2. Re-evaluate the migration strategy 
3. Consider more incremental approaches if needed