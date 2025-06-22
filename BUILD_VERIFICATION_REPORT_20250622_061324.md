# Build Matrix Verification Report
Date: 2025-06-22 06:13:24

## Configuration: WRT no_std + alloc
- Package: wrt
- Features: alloc
- ASIL Level: Core

❌ Build: FAILED
❌ Tests: FAILED
⚠️ Architectural issues detected

## Configuration: WRT ASIL-D (no_std + alloc)
- Package: wrt
- Features: alloc, safety-asil-d
- ASIL Level: ASIL-D

❌ Build: FAILED
❌ Tests: FAILED
✅ ASIL Check: No unsafe code

## Configuration: WRT ASIL-C (no_std + alloc)
- Package: wrt
- Features: alloc, safety-asil-c
- ASIL Level: ASIL-C

❌ Build: FAILED
❌ Tests: FAILED
✅ ASIL Check: No unsafe code

## Configuration: WRT ASIL-B (no_std + alloc)
- Package: wrt
- Features: alloc, safety-asil-b
- ASIL Level: ASIL-B

❌ Build: FAILED
❌ Tests: FAILED
⚠️ Architectural issues detected

## Configuration: WRT Development (std)
- Package: wrt
- Features: std
- ASIL Level: Development

❌ Build: FAILED
❌ Tests: FAILED
⚠️ Architectural issues detected

## Configuration: WRT Development with Optimization
- Package: wrt
- Features: std, optimize
- ASIL Level: Development

❌ Build: FAILED
❌ Tests: FAILED
⚠️ Architectural issues detected

## Configuration: WRT Server
- Package: wrt
- Features: std, optimize, platform
- ASIL Level: Server

❌ Build: FAILED
❌ Tests: FAILED
⚠️ Architectural issues detected

## Configuration: WRTD ASIL-D Runtime
- Package: wrtd
- Features: safety-asil-d, wrt-execution, enable-panic-handler
- ASIL Level: ASIL-D

❌ Build: FAILED
❌ Tests: FAILED
✅ ASIL Check: No unsafe code

## Configuration: WRTD ASIL-C Runtime
- Package: wrtd
- Features: safety-asil-c, wrt-execution, enable-panic-handler
- ASIL Level: ASIL-C

❌ Build: FAILED
❌ Tests: FAILED
✅ ASIL Check: No unsafe code

## Configuration: WRTD ASIL-B Runtime
- Package: wrtd
- Features: safety-asil-b, wrt-execution, asil-b-panic
- ASIL Level: ASIL-B

❌ Build: FAILED
❌ Tests: FAILED

## Configuration: WRTD Development Runtime
- Package: wrtd
- Features: std, wrt-execution, dev-panic
- ASIL Level: Development

❌ Build: FAILED
❌ Tests: FAILED
⚠️ Architectural issues detected

## Configuration: WRTD Server Runtime
- Package: wrtd
- Features: std, wrt-execution
- ASIL Level: Server

❌ Build: FAILED
❌ Tests: FAILED
⚠️ Architectural issues detected

## Configuration: Component Model Core
- Package: wrt-component
- Features: no_std, alloc, component-model-core
- ASIL Level: Component

❌ Build: FAILED
❌ Tests: FAILED
⚠️ Architectural issues detected

## Configuration: Component Model Full
- Package: wrt-component
- Features: std, component-model-all
- ASIL Level: Component

❌ Build: FAILED
❌ Tests: FAILED
⚠️ Architectural issues detected

## Kani Formal Verification
❌ Kani: FAILED

# Summary

❌ **Some configurations failed**
