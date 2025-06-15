# WRT Feature Flag Standardization Plan

## Overview
This document outlines the plan to standardize feature flags across all WRT crates for better consistency, safety compliance, and maintainability.

## Current State Analysis

### 1. Binary std/no_std Model
- **Current**: All crates follow binary choice (no `alloc` middle ground)
- **Status**: ✅ Already consistent

### 2. Safety Features
- **Current**: 
  - `optimize` - Performance mode
  - `safety` - ASIL-B level
  - `safety-critical` - ASIL-C level
- **Issues**: Not all crates implement all levels consistently

### 3. KANI Features
- **Current**: 7 packages with KANI support
- **Issues**: 
  - Inconsistent unwind depths
  - Missing from some safety-critical crates

### 4. Platform Features
- **Current**: Various prefixes (`platform-`, `target-`, direct names)
- **Issues**: Inconsistent naming convention

## Standardization Guidelines

### 1. Core Feature Set
Every crate MUST define these features:

```toml
[features]
default = ["std"]

# Memory model (mutually exclusive)
std = []
no_std = []

# Safety levels (additive)
safety-asil-b = []
safety-asil-c = ["safety-asil-b"]
safety-asil-d = ["safety-asil-c"]

# Verification
kani = []
```

### 2. Optional Features
Crates MAY define these features:

```toml
# Platform-specific (use platform- prefix)
platform-linux = []
platform-qnx = []
platform-vxworks = []

# Debug features
debug-trace = []
runtime-checks = []

# Performance
optimize = []
simd = []
```

### 3. Deprecated Features
These features should be removed:

```toml
# Remove these
safety = []           # Use safety-asil-b
safety-critical = []  # Use safety-asil-c
disable-panic-handler = []  # Implied by no_std
custom-panic-handler = []   # Implied by no_std
```

## Implementation Steps

### Phase 1: Core Infrastructure Crates
Update these crates first as they affect all others:

1. **wrt-foundation**
   - Add new safety-asil-* features
   - Update memory allocation features
   - Ensure KANI configuration

2. **wrt-error**
   - Standardize error handling features
   - Add safety level features

3. **wrt-platform**
   - Rename platform features with consistent prefix
   - Add safety level features

### Phase 2: Runtime Crates
Update runtime and execution crates:

4. **wrt-runtime**
   - Add all safety levels
   - Standardize KANI configuration
   - Update platform features

5. **wrt-component**
   - Add missing safety features
   - Ensure KANI support

6. **wrt-instructions**
   - Standardize feature set
   - Add safety levels

### Phase 3: Support Crates
Update remaining crates:

7. **wrt-sync**
   - Already well-structured
   - Just add safety-asil-* aliases

8. **wrt-debug**
   - Consolidate debug features
   - Add safety levels

9. **wrt-host**
   - Add KANI support
   - Standardize features

## Migration Script

A migration script will be created to:
1. Backup all Cargo.toml files
2. Update feature definitions
3. Add deprecation warnings for old features
4. Update feature dependencies

## Validation

After standardization:
1. All crates compile with each safety level
2. Feature combinations are tested
3. Documentation is updated
4. CI/CD pipelines are adjusted

## Timeline

- Phase 1: 1 day (core infrastructure)
- Phase 2: 1 day (runtime crates)
- Phase 3: 1 day (support crates)
- Validation: 1 day

Total: 4 days

## Backwards Compatibility

For one release cycle:
- Old feature names will be aliases to new ones
- Deprecation warnings will be shown
- Migration guide will be provided

## Success Criteria

1. All crates have consistent feature naming
2. Safety levels are clearly defined
3. KANI support is uniform
4. Platform features follow naming convention
5. No compilation errors with any valid feature combination