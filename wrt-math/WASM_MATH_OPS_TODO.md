# WebAssembly Math Operations TODO

This document tracks the implementation status of WebAssembly 3.0 numeric operations in the wrt-math crate.

## Implementation Status Legend
- ✅ Implemented
- ❌ Not implemented
- 🚧 In progress
- 🔄 Needs refactoring

## Integer Operations (i32/i64)

### Arithmetic
- ✅ `i32_add`, `i64_add`
- ✅ `i32_sub`, `i64_sub`
- ✅ `i32_mul`, `i64_mul`
- ✅ `i32_div_s`, `i32_div_u`, `i64_div_s`, `i64_div_u`
- ✅ `i32_rem_s`, `i32_rem_u`, `i64_rem_s`, `i64_rem_u`
- ✅ `i32_neg`, `i64_neg` - Two's complement negation
- ✅ `i32_abs`, `i64_abs` - Absolute value

### Saturating Arithmetic
- ❌ `i32_add_sat_s`, `i32_add_sat_u`
- ❌ `i64_add_sat_s`, `i64_add_sat_u`
- ❌ `i32_sub_sat_s`, `i32_sub_sat_u`
- ❌ `i64_sub_sat_s`, `i64_sub_sat_u`

### Bitwise
- ✅ `i32_and`, `i64_and`
- ✅ `i32_or`, `i64_or`
- ✅ `i32_xor`, `i64_xor`
- ❌ `i32_not`, `i64_not` - Bitwise NOT (can use xor with -1)
- ❌ `i32_andnot`, `i64_andnot` - AND with NOT of second operand
- ✅ `i32_shl`, `i64_shl`
- ✅ `i32_shr_s`, `i32_shr_u`, `i64_shr_s`, `i64_shr_u`
- ✅ `i32_rotl`, `i32_rotr`, `i64_rotl`, `i64_rotr`
- ❌ `i32_bitselect`, `i64_bitselect` - Bitwise select

### Bit Manipulation
- ✅ `i32_clz`, `i64_clz` - Count leading zeros
- ✅ `i32_ctz`, `i64_ctz` - Count trailing zeros
- ✅ `i32_popcnt`, `i64_popcnt` - Population count

### Comparison (CRITICAL GAP)
- ✅ `i32_eqz`, `i64_eqz` - Equal to zero
- ✅ `i32_eq`, `i64_eq` - Equal
- ✅ `i32_ne`, `i64_ne` - Not equal
- ✅ `i32_lt_s`, `i64_lt_s` - Less than (signed)
- ✅ `i32_lt_u`, `i64_lt_u` - Less than (unsigned)
- ✅ `i32_gt_s`, `i64_gt_s` - Greater than (signed)
- ✅ `i32_gt_u`, `i64_gt_u` - Greater than (unsigned)
- ✅ `i32_le_s`, `i64_le_s` - Less than or equal (signed)
- ✅ `i32_le_u`, `i64_le_u` - Less than or equal (unsigned)
- ✅ `i32_ge_s`, `i64_ge_s` - Greater than or equal (signed)
- ✅ `i32_ge_u`, `i64_ge_u` - Greater than or equal (unsigned)
- ❌ `i32_inez`, `i64_inez` - Not equal to zero (can use eqz + not)

### Sign/Zero Extension
- ✅ `i32_extend8_s` - Sign-extend 8-bit to 32-bit
- ✅ `i32_extend16_s` - Sign-extend 16-bit to 32-bit
- ✅ `i64_extend8_s` - Sign-extend 8-bit to 64-bit
- ✅ `i64_extend16_s` - Sign-extend 16-bit to 64-bit
- ✅ `i64_extend32_s` - Sign-extend 32-bit to 64-bit

### Special Operations
- ❌ `i32_avgr_u`, `i64_avgr_u` - Unsigned average with rounding
- ❌ `i32_q15mulrsat_s`, `i64_q15mulrsat_s` - Q15 saturating multiply

## Floating-Point Operations (f32/f64)

### Arithmetic
- ✅ `f32_add`, `f64_add`
- ✅ `f32_sub`, `f64_sub`
- ✅ `f32_mul`, `f64_mul`
- ✅ `f32_div`, `f64_div`
- ✅ `f32_sqrt`, `f64_sqrt`
- ✅ `f32_neg`, `f64_neg`
- ✅ `f32_abs`, `f64_abs`
- ❌ `f32_fma`, `f64_fma` - Fused multiply-add

### Rounding
- ✅ `f32_ceil`, `f64_ceil`
- ✅ `f32_floor`, `f64_floor`
- ✅ `f32_trunc`, `f64_trunc`
- ✅ `f32_nearest`, `f64_nearest`

### Comparison
- ✅ `f32_eq`, `f64_eq`
- ✅ `f32_ne`, `f64_ne`
- ✅ `f32_lt`, `f64_lt`
- ✅ `f32_gt`, `f64_gt`
- ✅ `f32_le`, `f64_le`
- ✅ `f32_ge`, `f64_ge`

### Min/Max
- ✅ `f32_min`, `f64_min`
- ✅ `f32_max`, `f64_max`
- ❌ `f32_pmin`, `f64_pmin` - Pseudo-min (NaN propagating)
- ❌ `f32_pmax`, `f64_pmax` - Pseudo-max (NaN propagating)

### Other
- ✅ `f32_copysign`, `f64_copysign`

## Type Conversion Operations (CRITICAL GAP)

### Integer to Float
- ✅ `f32_convert_i32_s` - Convert signed i32 to f32
- ✅ `f32_convert_i32_u` - Convert unsigned i32 to f32
- ✅ `f32_convert_i64_s` - Convert signed i64 to f32
- ✅ `f32_convert_i64_u` - Convert unsigned i64 to f32
- ✅ `f64_convert_i32_s` - Convert signed i32 to f64
- ✅ `f64_convert_i32_u` - Convert unsigned i32 to f64
- ✅ `f64_convert_i64_s` - Convert signed i64 to f64
- ✅ `f64_convert_i64_u` - Convert unsigned i64 to f64

### Float to Integer
- ✅ `i32_trunc_f32_s`, `i32_trunc_f32_u` - Truncate f32 to i32 (trapping)
- ✅ `i32_trunc_f64_s`, `i32_trunc_f64_u` - Truncate f64 to i32 (trapping)
- ✅ `i64_trunc_f32_s`, `i64_trunc_f32_u` - Truncate f32 to i64 (trapping)
- ✅ `i64_trunc_f64_s`, `i64_trunc_f64_u` - Truncate f64 to i64 (trapping)
- ✅ `i32_trunc_sat_f32_s`, `i32_trunc_sat_f32_u` - Truncate f32 to i32 (saturating)
- ✅ `i32_trunc_sat_f64_s`, `i32_trunc_sat_f64_u` - Truncate f64 to i32 (saturating)
- ✅ `i64_trunc_sat_f32_s`, `i64_trunc_sat_f32_u` - Truncate f32 to i64 (saturating)
- ✅ `i64_trunc_sat_f64_s`, `i64_trunc_sat_f64_u` - Truncate f64 to i64 (saturating)

### Float to Float
- ✅ `f32_demote_f64` - Demote f64 to f32
- ✅ `f64_promote_f32` - Promote f32 to f64

### Reinterpret (bit casting)
- ✅ `i32_reinterpret_f32` - Reinterpret f32 bits as i32
- ✅ `i64_reinterpret_f64` - Reinterpret f64 bits as i64
- ✅ `f32_reinterpret_i32` - Reinterpret i32 bits as f32
- ✅ `f64_reinterpret_i64` - Reinterpret i64 bits as f64

### Integer Width Conversion
- ✅ `i32_wrap_i64` - Wrap i64 to i32 (truncate)
- ✅ `i64_extend_i32_s` - Sign-extend i32 to i64
- ✅ `i64_extend_i32_u` - Zero-extend i32 to i64

## SIMD/Vector Operations (v128)

All v128 operations are missing. This is a massive feature set with hundreds of operations.

### Basic v128
- ❌ `v128.const`
- ❌ `v128.load`, `v128.store`
- ❌ All lane operations (i8x16, i16x8, i32x4, i64x2, f32x4, f64x2)

## Relaxed SIMD Operations

All relaxed SIMD operations from WASM 3.0 are missing.

## Implementation Priority

1. **Critical (Blocking basic functionality)**
   - Integer comparison operations
   - Type conversions (except SIMD)
   - Sign/zero extensions

2. **High (Common operations)**
   - Integer neg/abs
   - Reinterpret operations
   - Float promotion/demotion

3. **Medium (Performance/special cases)**
   - Saturating arithmetic
   - FMA operations
   - Pseudo min/max

4. **Low (Advanced features)**
   - SIMD operations
   - Relaxed SIMD
   - Special operations (avgr, q15mulrsat)

## CPU Acceleration Considerations

### Intrinsics Available
- [ ] Check for LLVM intrinsics mapping
- [ ] x86_64: SSE2/AVX for float ops
- [ ] ARM64: NEON for SIMD
- [ ] RISC-V: Vector extension

### Platform-specific Optimizations
- [ ] Use platform intrinsics where available
- [ ] Fallback to portable implementation
- [ ] Consider moving to wrt-platform for arch-specific code

### Compiler Optimizations
- [ ] Verify LLVM auto-vectorization
- [ ] Check if inline assembly needed
- [ ] Profile hot paths