# API Mismatch Fixes Summary

This document summarizes the fixes applied to resolve API mismatches in wrt-runtime.

## Fixed Issues

### 1. Module::default() doesn't exist
**File**: `src/module_builder.rs`
**Fix**: Changed from using `Module::default()` as fallback to panicking with descriptive error message when `Module::new()` fails.

### 2. BoundedVec.len() method doesn't exist
**File**: `src/module_instance.rs`
**Fix**: Added import for `BoundedCapacity` trait which provides the `len()` method for BoundedVec.
Also fixed the `function_type` method to use `as_slice()` method when iterating over BoundedVec contents.

### 3. Wrong Vec API usage - calling .set() on Vec
**File**: `src/stackless/frame.rs`
**Fix**: Added conditional compilation to handle both Vec (std) and BoundedVec (no_std) cases:
- For Vec: Use direct indexing with bounds checking
- For BoundedVec: Use the `.set()` method
- Fixed memory copy operations to handle different buffer types correctly

### 4. ValueStackVec::new() wrong API usage
**File**: `src/stackless/frame.rs`
**Fix**: Used `Vec::new()` for std builds instead of trying to call non-existent constructor with provider.

## Implementation Details

All fixes use conditional compilation (`#[cfg(feature = "std")]`) to ensure correct API usage for both std and no_std environments.

The fixes maintain compatibility with the existing architecture while properly handling the type aliases that switch between Vec and BoundedVec based on features.