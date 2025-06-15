# WRT Feature Flag Migration Guide

This guide helps you migrate to the new standardized feature flags.

## Feature Mappings

| Old Feature | New Feature | Notes |
|-------------|-------------|-------|
| `safety` | `safety-asil-b` | ASIL-B safety level |
| `safety-critical` | `safety-asil-c` | ASIL-C safety level |
| `linux` | `platform-linux` | Platform-specific feature |
| `qnx` | `platform-qnx` | Platform-specific feature |
| `vxworks` | `platform-vxworks` | Platform-specific feature |

## Removed Features

The following features have been removed:
- `disable-panic-handler` (implied by `no_std`)
- `custom-panic-handler` (implied by `no_std`)

## New Safety Levels

All crates now support these safety levels:
- `safety-asil-b` - Basic safety features (ASIL-B)
- `safety-asil-c` - Critical safety features (ASIL-C)  
- `safety-asil-d` - Highest safety level (ASIL-D)

## KANI Support

The following crates now have KANI formal verification support:
- wrt-foundation, wrt-component, wrt-sync
- wrt-runtime, wrt-platform, wrt-instructions
- wrt-decoder, wrt-host, wrt-debug
