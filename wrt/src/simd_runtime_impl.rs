//! Complete SIMD runtime implementation for WebAssembly v128 operations
//!
//! This module provides a comprehensive implementation of all WebAssembly SIMD instructions,
//! mapping them to the appropriate SIMD provider methods with proper error handling.

use wrt_error::{Error, ErrorCategory, Result};
use wrt_foundation::values::{Value, V128, FloatBits32, FloatBits64};
use wrt_instructions::simd_ops::SimdOp;
use wrt_platform::simd::SimdProvider;

/// Execute a SIMD operation using the provided SIMD provider
pub fn execute_simd_operation(
    op: SimdOp,
    inputs: &[Value],
    provider: &dyn SimdProvider,
) -> Result<Value> {
    // Helper macros for common patterns
    macro_rules! unary_op {
        ($inputs:expr, $provider:expr, $method:ident) => {{
            if $inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&$inputs[0])?;
            let result = $provider.$method(&a);
            Ok(Value::V128(V128::new(result)))
        }};
    }

    macro_rules! binary_op {
        ($inputs:expr, $provider:expr, $method:ident) => {{
            if $inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&$inputs[0])?;
            let b = extract_v128_bytes(&$inputs[1])?;
            let result = $provider.$method(&a, &b);
            Ok(Value::V128(V128::new(result)))
        }};
    }

    macro_rules! ternary_op {
        ($inputs:expr, $provider:expr, $method:ident) => {{
            if $inputs.len() != 3 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Operation requires exactly 3 inputs",
                ));
            }
            let a = extract_v128_bytes(&$inputs[0])?;
            let b = extract_v128_bytes(&$inputs[1])?;
            let c = extract_v128_bytes(&$inputs[2])?;
            let result = $provider.$method(&a, &b, &c);
            Ok(Value::V128(V128::new(result)))
        }};
    }

    macro_rules! splat_i32 {
        ($inputs:expr, $provider:expr, $method:ident) => {{
            if $inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Splat operation requires exactly 1 input",
                ));
            }
            let value = $inputs[0].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Splat value must be i32",
                )
            })?;
            let result = $provider.$method(value);
            Ok(Value::V128(V128::new(result)))
        }};
    }

    macro_rules! splat_i64 {
        ($inputs:expr, $provider:expr, $method:ident) => {{
            if $inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Splat operation requires exactly 1 input",
                ));
            }
            let value = $inputs[0].as_i64().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Splat value must be i64",
                )
            })?;
            let result = $provider.$method(value);
            Ok(Value::V128(V128::new(result)))
        }};
    }

    macro_rules! splat_f32 {
        ($inputs:expr, $provider:expr, $method:ident) => {{
            if $inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Splat operation requires exactly 1 input",
                ));
            }
            let value = $inputs[0].as_f32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Splat value must be f32",
                )
            })?;
            let result = $provider.$method(value);
            Ok(Value::V128(V128::new(result)))
        }};
    }

    macro_rules! splat_f64 {
        ($inputs:expr, $provider:expr, $method:ident) => {{
            if $inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Splat operation requires exactly 1 input",
                ));
            }
            let value = $inputs[0].as_f64().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Splat value must be f64",
                )
            })?;
            let result = $provider.$method(value);
            Ok(Value::V128(V128::new(result)))
        }};
    }

    match op {
        // --- Arithmetic Operations ---
        // i8x16 operations
        SimdOp::I8x16Add => binary_op!(inputs, provider, v128_i8x16_add),
        SimdOp::I8x16Sub => binary_op!(inputs, provider, v128_i8x16_sub),
        SimdOp::I8x16Neg => unary_op!(inputs, provider, v128_i8x16_neg),
        SimdOp::I8x16Abs => unary_op!(inputs, provider, v128_i8x16_abs),
        SimdOp::I8x16MinS => binary_op!(inputs, provider, v128_i8x16_min_s),
        SimdOp::I8x16MinU => binary_op!(inputs, provider, v128_i8x16_min_u),
        SimdOp::I8x16MaxS => binary_op!(inputs, provider, v128_i8x16_max_s),
        SimdOp::I8x16MaxU => binary_op!(inputs, provider, v128_i8x16_max_u),
        SimdOp::I8x16AvgrU => binary_op!(inputs, provider, v128_i8x16_avgr_u),

        // i16x8 operations
        SimdOp::I16x8Add => binary_op!(inputs, provider, v128_i16x8_add),
        SimdOp::I16x8Sub => binary_op!(inputs, provider, v128_i16x8_sub),
        SimdOp::I16x8Mul => binary_op!(inputs, provider, v128_i16x8_mul),
        SimdOp::I16x8Neg => unary_op!(inputs, provider, v128_i16x8_neg),
        SimdOp::I16x8Abs => unary_op!(inputs, provider, v128_i16x8_abs),
        SimdOp::I16x8MinS => binary_op!(inputs, provider, v128_i16x8_min_s),
        SimdOp::I16x8MinU => binary_op!(inputs, provider, v128_i16x8_min_u),
        SimdOp::I16x8MaxS => binary_op!(inputs, provider, v128_i16x8_max_s),
        SimdOp::I16x8MaxU => binary_op!(inputs, provider, v128_i16x8_max_u),
        SimdOp::I16x8AvgrU => binary_op!(inputs, provider, v128_i16x8_avgr_u),

        // i32x4 operations
        SimdOp::I32x4Add => binary_op!(inputs, provider, v128_i32x4_add),
        SimdOp::I32x4Sub => binary_op!(inputs, provider, v128_i32x4_sub),
        SimdOp::I32x4Mul => binary_op!(inputs, provider, v128_i32x4_mul),
        SimdOp::I32x4Neg => unary_op!(inputs, provider, v128_i32x4_neg),
        SimdOp::I32x4Abs => unary_op!(inputs, provider, v128_i32x4_abs),
        SimdOp::I32x4MinS => binary_op!(inputs, provider, v128_i32x4_min_s),
        SimdOp::I32x4MinU => binary_op!(inputs, provider, v128_i32x4_min_u),
        SimdOp::I32x4MaxS => binary_op!(inputs, provider, v128_i32x4_max_s),
        SimdOp::I32x4MaxU => binary_op!(inputs, provider, v128_i32x4_max_u),

        // i64x2 operations
        SimdOp::I64x2Add => binary_op!(inputs, provider, v128_i64x2_add),
        SimdOp::I64x2Sub => binary_op!(inputs, provider, v128_i64x2_sub),
        SimdOp::I64x2Mul => binary_op!(inputs, provider, v128_i64x2_mul),
        SimdOp::I64x2Neg => unary_op!(inputs, provider, v128_i64x2_neg),
        SimdOp::I64x2Abs => unary_op!(inputs, provider, v128_i64x2_abs),

        // f32x4 operations
        SimdOp::F32x4Add => binary_op!(inputs, provider, v128_f32x4_add),
        SimdOp::F32x4Sub => binary_op!(inputs, provider, v128_f32x4_sub),
        SimdOp::F32x4Mul => binary_op!(inputs, provider, v128_f32x4_mul),
        SimdOp::F32x4Div => binary_op!(inputs, provider, v128_f32x4_div),
        SimdOp::F32x4Neg => unary_op!(inputs, provider, v128_f32x4_neg),
        SimdOp::F32x4Abs => unary_op!(inputs, provider, v128_f32x4_abs),
        SimdOp::F32x4Min => binary_op!(inputs, provider, v128_f32x4_min),
        SimdOp::F32x4Max => binary_op!(inputs, provider, v128_f32x4_max),
        SimdOp::F32x4PMin => binary_op!(inputs, provider, v128_f32x4_pmin),
        SimdOp::F32x4PMax => binary_op!(inputs, provider, v128_f32x4_pmax),
        SimdOp::F32x4Sqrt => unary_op!(inputs, provider, v128_f32x4_sqrt),
        SimdOp::F32x4Ceil => unary_op!(inputs, provider, v128_f32x4_ceil),
        SimdOp::F32x4Floor => unary_op!(inputs, provider, v128_f32x4_floor),
        SimdOp::F32x4Trunc => unary_op!(inputs, provider, v128_f32x4_trunc),
        SimdOp::F32x4Nearest => unary_op!(inputs, provider, v128_f32x4_nearest),

        // f64x2 operations
        SimdOp::F64x2Add => binary_op!(inputs, provider, v128_f64x2_add),
        SimdOp::F64x2Sub => binary_op!(inputs, provider, v128_f64x2_sub),
        SimdOp::F64x2Mul => binary_op!(inputs, provider, v128_f64x2_mul),
        SimdOp::F64x2Div => binary_op!(inputs, provider, v128_f64x2_div),
        SimdOp::F64x2Neg => unary_op!(inputs, provider, v128_f64x2_neg),
        SimdOp::F64x2Abs => unary_op!(inputs, provider, v128_f64x2_abs),
        SimdOp::F64x2Min => binary_op!(inputs, provider, v128_f64x2_min),
        SimdOp::F64x2Max => binary_op!(inputs, provider, v128_f64x2_max),
        SimdOp::F64x2PMin => binary_op!(inputs, provider, v128_f64x2_pmin),
        SimdOp::F64x2PMax => binary_op!(inputs, provider, v128_f64x2_pmax),
        SimdOp::F64x2Sqrt => unary_op!(inputs, provider, v128_f64x2_sqrt),
        SimdOp::F64x2Ceil => unary_op!(inputs, provider, v128_f64x2_ceil),
        SimdOp::F64x2Floor => unary_op!(inputs, provider, v128_f64x2_floor),
        SimdOp::F64x2Trunc => unary_op!(inputs, provider, v128_f64x2_trunc),
        SimdOp::F64x2Nearest => unary_op!(inputs, provider, v128_f64x2_nearest),

        // --- Comparison Operations ---
        // i8x16 comparisons
        SimdOp::I8x16Eq => binary_op!(inputs, provider, v128_i8x16_eq),
        SimdOp::I8x16Ne => binary_op!(inputs, provider, v128_i8x16_ne),
        SimdOp::I8x16LtS => binary_op!(inputs, provider, v128_i8x16_lt_s),
        SimdOp::I8x16LtU => binary_op!(inputs, provider, v128_i8x16_lt_u),
        SimdOp::I8x16GtS => binary_op!(inputs, provider, v128_i8x16_gt_s),
        SimdOp::I8x16GtU => binary_op!(inputs, provider, v128_i8x16_gt_u),
        SimdOp::I8x16LeS => binary_op!(inputs, provider, v128_i8x16_le_s),
        SimdOp::I8x16LeU => binary_op!(inputs, provider, v128_i8x16_le_u),
        SimdOp::I8x16GeS => binary_op!(inputs, provider, v128_i8x16_ge_s),
        SimdOp::I8x16GeU => binary_op!(inputs, provider, v128_i8x16_ge_u),

        // i16x8 comparisons
        SimdOp::I16x8Eq => binary_op!(inputs, provider, v128_i16x8_eq),
        SimdOp::I16x8Ne => binary_op!(inputs, provider, v128_i16x8_ne),
        SimdOp::I16x8LtS => binary_op!(inputs, provider, v128_i16x8_lt_s),
        SimdOp::I16x8LtU => binary_op!(inputs, provider, v128_i16x8_lt_u),
        SimdOp::I16x8GtS => binary_op!(inputs, provider, v128_i16x8_gt_s),
        SimdOp::I16x8GtU => binary_op!(inputs, provider, v128_i16x8_gt_u),
        SimdOp::I16x8LeS => binary_op!(inputs, provider, v128_i16x8_le_s),
        SimdOp::I16x8LeU => binary_op!(inputs, provider, v128_i16x8_le_u),
        SimdOp::I16x8GeS => binary_op!(inputs, provider, v128_i16x8_ge_s),
        SimdOp::I16x8GeU => binary_op!(inputs, provider, v128_i16x8_ge_u),

        // i32x4 comparisons
        SimdOp::I32x4Eq => binary_op!(inputs, provider, v128_i32x4_eq),
        SimdOp::I32x4Ne => binary_op!(inputs, provider, v128_i32x4_ne),
        SimdOp::I32x4LtS => binary_op!(inputs, provider, v128_i32x4_lt_s),
        SimdOp::I32x4LtU => binary_op!(inputs, provider, v128_i32x4_lt_u),
        SimdOp::I32x4GtS => binary_op!(inputs, provider, v128_i32x4_gt_s),
        SimdOp::I32x4GtU => binary_op!(inputs, provider, v128_i32x4_gt_u),
        SimdOp::I32x4LeS => binary_op!(inputs, provider, v128_i32x4_le_s),
        SimdOp::I32x4LeU => binary_op!(inputs, provider, v128_i32x4_le_u),
        SimdOp::I32x4GeS => binary_op!(inputs, provider, v128_i32x4_ge_s),
        SimdOp::I32x4GeU => binary_op!(inputs, provider, v128_i32x4_ge_u),

        // i64x2 comparisons
        SimdOp::I64x2Eq => binary_op!(inputs, provider, v128_i64x2_eq),
        SimdOp::I64x2Ne => binary_op!(inputs, provider, v128_i64x2_ne),
        SimdOp::I64x2LtS => binary_op!(inputs, provider, v128_i64x2_lt_s),
        SimdOp::I64x2GtS => binary_op!(inputs, provider, v128_i64x2_gt_s),
        SimdOp::I64x2LeS => binary_op!(inputs, provider, v128_i64x2_le_s),
        SimdOp::I64x2GeS => binary_op!(inputs, provider, v128_i64x2_ge_s),

        // f32x4 comparisons
        SimdOp::F32x4Eq => binary_op!(inputs, provider, v128_f32x4_eq),
        SimdOp::F32x4Ne => binary_op!(inputs, provider, v128_f32x4_ne),
        SimdOp::F32x4Lt => binary_op!(inputs, provider, v128_f32x4_lt),
        SimdOp::F32x4Gt => binary_op!(inputs, provider, v128_f32x4_gt),
        SimdOp::F32x4Le => binary_op!(inputs, provider, v128_f32x4_le),
        SimdOp::F32x4Ge => binary_op!(inputs, provider, v128_f32x4_ge),

        // f64x2 comparisons
        SimdOp::F64x2Eq => binary_op!(inputs, provider, v128_f64x2_eq),
        SimdOp::F64x2Ne => binary_op!(inputs, provider, v128_f64x2_ne),
        SimdOp::F64x2Lt => binary_op!(inputs, provider, v128_f64x2_lt),
        SimdOp::F64x2Gt => binary_op!(inputs, provider, v128_f64x2_gt),
        SimdOp::F64x2Le => binary_op!(inputs, provider, v128_f64x2_le),
        SimdOp::F64x2Ge => binary_op!(inputs, provider, v128_f64x2_ge),

        // --- Bitwise Operations ---
        SimdOp::V128Not => unary_op!(inputs, provider, v128_not),
        SimdOp::V128And => binary_op!(inputs, provider, v128_and),
        SimdOp::V128AndNot => binary_op!(inputs, provider, v128_andnot),
        SimdOp::V128Or => binary_op!(inputs, provider, v128_or),
        SimdOp::V128Xor => binary_op!(inputs, provider, v128_xor),
        SimdOp::V128Bitselect => ternary_op!(inputs, provider, v128_bitselect),

        // --- Test Operations ---
        SimdOp::V128AnyTrue => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "any_true operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_any_true(&a);
            Ok(Value::I32(if result { 1 } else { 0 }))
        }

        SimdOp::I8x16AllTrue => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "all_true operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_i8x16_all_true(&a);
            Ok(Value::I32(if result { 1 } else { 0 }))
        }

        SimdOp::I16x8AllTrue => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "all_true operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_i16x8_all_true(&a);
            Ok(Value::I32(if result { 1 } else { 0 }))
        }

        SimdOp::I32x4AllTrue => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "all_true operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_i32x4_all_true(&a);
            Ok(Value::I32(if result { 1 } else { 0 }))
        }

        SimdOp::I64x2AllTrue => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "all_true operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_i64x2_all_true(&a);
            Ok(Value::I32(if result { 1 } else { 0 }))
        }

        // --- Lane Access Operations ---
        SimdOp::I8x16ExtractLaneS { lane } => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Extract lane operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_i8x16_extract_lane_s(&a, *lane);
            Ok(Value::I32(result as i32))
        }

        SimdOp::I8x16ExtractLaneU { lane } => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Extract lane operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_i8x16_extract_lane_u(&a, *lane);
            Ok(Value::I32(result as i32))
        }

        SimdOp::I8x16ReplaceLane { lane } => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Replace lane operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let value = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Lane value must be i32",
                )
            })?;
            let result = provider.v128_i8x16_replace_lane(&a, *lane, value);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I16x8ExtractLaneS { lane } => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Extract lane operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_i16x8_extract_lane_s(&a, *lane);
            Ok(Value::I32(result as i32))
        }

        SimdOp::I16x8ExtractLaneU { lane } => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Extract lane operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_i16x8_extract_lane_u(&a, *lane);
            Ok(Value::I32(result as i32))
        }

        SimdOp::I16x8ReplaceLane { lane } => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Replace lane operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let value = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Lane value must be i32",
                )
            })?;
            let result = provider.v128_i16x8_replace_lane(&a, *lane, value);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I32x4ExtractLane { lane } => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Extract lane operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_i32x4_extract_lane(&a, *lane);
            Ok(Value::I32(result as i32))
        }

        SimdOp::I32x4ReplaceLane { lane } => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Replace lane operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let value = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Lane value must be i32",
                )
            })?;
            let result = provider.v128_i32x4_replace_lane(&a, *lane, value);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I64x2ExtractLane { lane } => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Extract lane operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_i64x2_extract_lane(&a, *lane);
            Ok(Value::I64(result))
        }

        SimdOp::I64x2ReplaceLane { lane } => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Replace lane operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let value = inputs[1].as_i64().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Lane value must be i64",
                )
            })?;
            let result = provider.v128_i64x2_replace_lane(&a, *lane, value);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::F32x4ExtractLane { lane } => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Extract lane operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_f32x4_extract_lane(&a, *lane);
            Ok(Value::F32(FloatBits32::from_float(result)))
        }

        SimdOp::F32x4ReplaceLane { lane } => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Replace lane operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let value = inputs[1].as_f32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Lane value must be f32",
                )
            })?;
            let result = provider.v128_f32x4_replace_lane(&a, *lane, value);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::F64x2ExtractLane { lane } => {
            if inputs.len() != 1 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Extract lane operation requires exactly 1 input",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let result = provider.v128_f64x2_extract_lane(&a, *lane);
            Ok(Value::F64(FloatBits64::from_float(result)))
        }

        SimdOp::F64x2ReplaceLane { lane } => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Replace lane operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let value = inputs[1].as_f64().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Lane value must be f64",
                )
            })?;
            let result = provider.v128_f64x2_replace_lane(&a, *lane, value);
            Ok(Value::V128(V128::new(result)))
        }

        // --- Splat Operations ---
        SimdOp::I8x16Splat => splat_i32!(inputs, provider, v128_i8x16_splat),
        SimdOp::I16x8Splat => splat_i32!(inputs, provider, v128_i16x8_splat),
        SimdOp::I32x4Splat => splat_i32!(inputs, provider, v128_i32x4_splat),
        SimdOp::I64x2Splat => splat_i64!(inputs, provider, v128_i64x2_splat),
        SimdOp::F32x4Splat => splat_f32!(inputs, provider, v128_f32x4_splat),
        SimdOp::F64x2Splat => splat_f64!(inputs, provider, v128_f64x2_splat),

        // --- Shift Operations ---
        SimdOp::I8x16Shl => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i8x16_shl(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I8x16ShrS => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i8x16_shr_s(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I8x16ShrU => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i8x16_shr_u(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I16x8Shl => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i16x8_shl(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I16x8ShrS => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i16x8_shr_s(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I16x8ShrU => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i16x8_shr_u(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I32x4Shl => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i32x4_shl(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I32x4ShrS => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i32x4_shr_s(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I32x4ShrU => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i32x4_shr_u(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I64x2Shl => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i64x2_shl(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I64x2ShrS => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i64x2_shr_s(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        SimdOp::I64x2ShrU => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shift operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let shift = inputs[1].as_u32().ok_or_else(|| {
                Error::new(
                    ErrorCategory::Type,
                    wrt_error::codes::TYPE_MISMATCH,
                    "Shift amount must be i32",
                )
            })? as u8;
            let result = provider.v128_i64x2_shr_u(&a, shift);
            Ok(Value::V128(V128::new(result)))
        }

        // --- Conversion Operations ---
        SimdOp::I32x4TruncSatF32x4S => unary_op!(inputs, provider, v128_i32x4_trunc_sat_f32x4_s),
        SimdOp::I32x4TruncSatF32x4U => unary_op!(inputs, provider, v128_i32x4_trunc_sat_f32x4_u),
        SimdOp::F32x4ConvertI32x4S => unary_op!(inputs, provider, v128_f32x4_convert_i32x4_s),
        SimdOp::F32x4ConvertI32x4U => unary_op!(inputs, provider, v128_f32x4_convert_i32x4_u),
        SimdOp::I32x4TruncSatF64x2SZero => unary_op!(inputs, provider, v128_i32x4_trunc_sat_f64x2_s_zero),
        SimdOp::I32x4TruncSatF64x2UZero => unary_op!(inputs, provider, v128_i32x4_trunc_sat_f64x2_u_zero),
        SimdOp::F64x2ConvertLowI32x4S => unary_op!(inputs, provider, v128_f64x2_convert_low_i32x4_s),
        SimdOp::F64x2ConvertLowI32x4U => unary_op!(inputs, provider, v128_f64x2_convert_low_i32x4_u),
        SimdOp::F32x4DemoteF64x2Zero => unary_op!(inputs, provider, v128_f32x4_demote_f64x2_zero),
        SimdOp::F64x2PromoteLowF32x4 => unary_op!(inputs, provider, v128_f64x2_promote_low_f32x4),

        // --- Extended/Narrow Operations ---
        SimdOp::I16x8ExtendLowI8x16S => unary_op!(inputs, provider, v128_i16x8_extend_low_i8x16_s),
        SimdOp::I16x8ExtendHighI8x16S => unary_op!(inputs, provider, v128_i16x8_extend_high_i8x16_s),
        SimdOp::I16x8ExtendLowI8x16U => unary_op!(inputs, provider, v128_i16x8_extend_low_i8x16_u),
        SimdOp::I16x8ExtendHighI8x16U => unary_op!(inputs, provider, v128_i16x8_extend_high_i8x16_u),
        SimdOp::I32x4ExtendLowI16x8S => unary_op!(inputs, provider, v128_i32x4_extend_low_i16x8_s),
        SimdOp::I32x4ExtendHighI16x8S => unary_op!(inputs, provider, v128_i32x4_extend_high_i16x8_s),
        SimdOp::I32x4ExtendLowI16x8U => unary_op!(inputs, provider, v128_i32x4_extend_low_i16x8_u),
        SimdOp::I32x4ExtendHighI16x8U => unary_op!(inputs, provider, v128_i32x4_extend_high_i16x8_u),
        SimdOp::I64x2ExtendLowI32x4S => unary_op!(inputs, provider, v128_i64x2_extend_low_i32x4_s),
        SimdOp::I64x2ExtendHighI32x4S => unary_op!(inputs, provider, v128_i64x2_extend_high_i32x4_s),
        SimdOp::I64x2ExtendLowI32x4U => unary_op!(inputs, provider, v128_i64x2_extend_low_i32x4_u),
        SimdOp::I64x2ExtendHighI32x4U => unary_op!(inputs, provider, v128_i64x2_extend_high_i32x4_u),

        SimdOp::I8x16NarrowI16x8S => binary_op!(inputs, provider, v128_i8x16_narrow_i16x8_s),
        SimdOp::I8x16NarrowI16x8U => binary_op!(inputs, provider, v128_i8x16_narrow_i16x8_u),
        SimdOp::I16x8NarrowI32x4S => binary_op!(inputs, provider, v128_i16x8_narrow_i32x4_s),
        SimdOp::I16x8NarrowI32x4U => binary_op!(inputs, provider, v128_i16x8_narrow_i32x4_u),

        // --- Advanced Operations ---
        SimdOp::V128Swizzle => binary_op!(inputs, provider, v128_swizzle),
        SimdOp::V128Shuffle { lanes } => {
            if inputs.len() != 2 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    wrt_error::codes::INVALID_OPERAND_COUNT,
                    "Shuffle operation requires exactly 2 inputs",
                ));
            }
            let a = extract_v128_bytes(&inputs[0])?;
            let b = extract_v128_bytes(&inputs[1])?;
            let result = provider.v128_shuffle(&a, &b, lanes);
            Ok(Value::V128(V128::new(result)))
        }

        // --- Saturating Arithmetic ---
        SimdOp::I8x16AddSatS => binary_op!(inputs, provider, v128_i8x16_add_sat_s),
        SimdOp::I8x16AddSatU => binary_op!(inputs, provider, v128_i8x16_add_sat_u),
        SimdOp::I8x16SubSatS => binary_op!(inputs, provider, v128_i8x16_sub_sat_s),
        SimdOp::I8x16SubSatU => binary_op!(inputs, provider, v128_i8x16_sub_sat_u),
        SimdOp::I16x8AddSatS => binary_op!(inputs, provider, v128_i16x8_add_sat_s),
        SimdOp::I16x8AddSatU => binary_op!(inputs, provider, v128_i16x8_add_sat_u),
        SimdOp::I16x8SubSatS => binary_op!(inputs, provider, v128_i16x8_sub_sat_s),
        SimdOp::I16x8SubSatU => binary_op!(inputs, provider, v128_i16x8_sub_sat_u),

        // --- Dot Product Operations ---
        SimdOp::I32x4DotI16x8S => binary_op!(inputs, provider, v128_i32x4_dot_i16x8_s),

        // --- Extended Multiplication ---
        SimdOp::I16x8ExtMulLowI8x16S => binary_op!(inputs, provider, v128_i16x8_extmul_low_i8x16_s),
        SimdOp::I16x8ExtMulHighI8x16S => binary_op!(inputs, provider, v128_i16x8_extmul_high_i8x16_s),
        SimdOp::I16x8ExtMulLowI8x16U => binary_op!(inputs, provider, v128_i16x8_extmul_low_i8x16_u),
        SimdOp::I16x8ExtMulHighI8x16U => binary_op!(inputs, provider, v128_i16x8_extmul_high_i8x16_u),
        SimdOp::I32x4ExtMulLowI16x8S => binary_op!(inputs, provider, v128_i32x4_extmul_low_i16x8_s),
        SimdOp::I32x4ExtMulHighI16x8S => binary_op!(inputs, provider, v128_i32x4_extmul_high_i16x8_s),
        SimdOp::I32x4ExtMulLowI16x8U => binary_op!(inputs, provider, v128_i32x4_extmul_low_i16x8_u),
        SimdOp::I32x4ExtMulHighI16x8U => binary_op!(inputs, provider, v128_i32x4_extmul_high_i16x8_u),
        SimdOp::I64x2ExtMulLowI32x4S => binary_op!(inputs, provider, v128_i64x2_extmul_low_i32x4_s),
        SimdOp::I64x2ExtMulHighI32x4S => binary_op!(inputs, provider, v128_i64x2_extmul_high_i32x4_s),
        SimdOp::I64x2ExtMulLowI32x4U => binary_op!(inputs, provider, v128_i64x2_extmul_low_i32x4_u),
        SimdOp::I64x2ExtMulHighI32x4U => binary_op!(inputs, provider, v128_i64x2_extmul_high_i32x4_u),

        // Memory operations are handled separately in the memory module
        SimdOp::V128Load { .. } |
        SimdOp::V128Load8x8S { .. } |
        SimdOp::V128Load8x8U { .. } |
        SimdOp::V128Load16x4S { .. } |
        SimdOp::V128Load16x4U { .. } |
        SimdOp::V128Load32x2S { .. } |
        SimdOp::V128Load32x2U { .. } |
        SimdOp::V128Load8Splat { .. } |
        SimdOp::V128Load16Splat { .. } |
        SimdOp::V128Load32Splat { .. } |
        SimdOp::V128Load64Splat { .. } |
        SimdOp::V128Store { .. } => {
            Err(Error::new(
                ErrorCategory::Validation,
                wrt_error::codes::UNSUPPORTED_OPERATION,
                "Memory SIMD operations should be handled by memory module",
            ))
        }

        // For any remaining unimplemented operations
        _ => Err(Error::new(
            ErrorCategory::Validation,
            wrt_error::codes::UNSUPPORTED_OPERATION,
            format!("SIMD operation {:?} not yet implemented", op),
        )),
    }
}

/// Extract v128 bytes from a Value
fn extract_v128_bytes(value: &Value) -> Result<[u8; 16]> {
    match value {
        Value::V128(v128) => Ok(v128.bytes),
        _ => Err(Error::new(
            ErrorCategory::Type,
            wrt_error::codes::TYPE_MISMATCH,
            format!("Expected v128 value, got {:?}", value.value_type()),
        )),
    }
}