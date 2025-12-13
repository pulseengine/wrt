//! WAST Value Conversion Utilities
//!
//! This module provides functions to convert between WAST test framework value
//! types and WRT runtime value types, including proper handling of NaN patterns,
//! V128 vectors, and reference types.

#![cfg(feature = "std")]

use anyhow::Result;
use wast::{
    core::{
        NanPattern,
        V128Pattern,
        WastArgCore,
        WastRetCore,
    },
    WastArg,
    WastRet,
};
use wrt_foundation::values::{
    FloatBits32,
    FloatBits64,
    Value,
    V128,
};

/// Convert WAST arguments to WRT values
pub fn convert_wast_args_to_values(args: &[WastArg]) -> Result<Vec<Value>> {
    args.iter().map(convert_wast_arg_to_value).collect()
}

/// Convert a single WAST argument to a WRT value
pub fn convert_wast_arg_to_value(arg: &WastArg) -> Result<Value> {
    match arg {
        WastArg::Core(core_arg) => convert_wast_arg_core_to_value(core_arg),
        _ => Err(anyhow::anyhow!("Unsupported WAST argument type")),
    }
}

/// Convert WAST core argument to WRT value
pub fn convert_wast_arg_core_to_value(arg: &WastArgCore) -> Result<Value> {
    match arg {
        WastArgCore::I32(x) => Ok(Value::I32(*x)),
        WastArgCore::I64(x) => Ok(Value::I64(*x)),
        WastArgCore::F32(x) => Ok(Value::F32(FloatBits32::from_bits(x.bits))),
        WastArgCore::F64(x) => Ok(Value::F64(FloatBits64::from_bits(x.bits))),
        WastArgCore::V128(x) => Ok(Value::V128(V128::new(convert_v128_const_to_bytes(x)?))),
        WastArgCore::RefNull(_) => Ok(Value::Ref(0)), // Use 0 for null reference
        WastArgCore::RefExtern(x) => Ok(Value::Ref(*x as u32)),
        WastArgCore::RefHost(x) => Ok(Value::Ref(*x as u32)),
    }
}

/// Convert WAST expected results to WRT values for comparison
pub fn convert_wast_results_to_values(results: &[WastRet]) -> Result<Vec<Value>> {
    results.iter().map(convert_wast_ret_to_value).collect()
}

/// Convert a single WAST return value to a WRT value
pub fn convert_wast_ret_to_value(ret: &WastRet) -> Result<Value> {
    match ret {
        WastRet::Core(core_ret) => convert_wast_ret_core_to_value(core_ret),
        _ => Err(anyhow::anyhow!("Unsupported WAST return type")),
    }
}

/// Convert WAST core return value to WRT value
pub fn convert_wast_ret_core_to_value(ret: &WastRetCore) -> Result<Value> {
    match ret {
        WastRetCore::I32(x) => Ok(Value::I32(*x)),
        WastRetCore::I64(x) => Ok(Value::I64(*x)),
        WastRetCore::F32(nan_pattern) => match nan_pattern {
            NanPattern::Value(x) => Ok(Value::F32(FloatBits32::from_bits(x.bits))),
            NanPattern::CanonicalNan => Ok(Value::F32(FloatBits32::NAN)),
            NanPattern::ArithmeticNan => Ok(Value::F32(FloatBits32::NAN)),
        },
        WastRetCore::F64(nan_pattern) => match nan_pattern {
            NanPattern::Value(x) => Ok(Value::F64(FloatBits64::from_bits(x.bits))),
            NanPattern::CanonicalNan => Ok(Value::F64(FloatBits64::NAN)),
            NanPattern::ArithmeticNan => Ok(Value::F64(FloatBits64::NAN)),
        },
        WastRetCore::V128(x) => Ok(Value::V128(V128::new(convert_v128_pattern_to_bytes(x)?))),
        WastRetCore::RefNull(_) => Ok(Value::Ref(0)), // Use 0 for null reference
        WastRetCore::RefExtern(x) => Ok(Value::Ref(x.unwrap_or(0) as u32)),
        WastRetCore::RefHost(x) => Ok(Value::Ref(*x as u32)),
        WastRetCore::RefFunc(x) => {
            // Function references need special handling - use default value for now
            Ok(Value::Ref(x.is_some() as u32))
        },
        _ => {
            // Handle other reference types with default values
            Ok(Value::Ref(0))
        },
    }
}

/// Convert V128Const to byte array
fn convert_v128_const_to_bytes(v128: &wast::core::V128Const) -> Result<[u8; 16]> {
    Ok(v128.to_le_bytes())
}

/// Convert V128Pattern to byte array
fn convert_v128_pattern_to_bytes(pattern: &V128Pattern) -> Result<[u8; 16]> {
    match pattern {
        V128Pattern::I8x16(values) => {
            let mut bytes = [0u8; 16];
            for (i, &val) in values.iter().enumerate() {
                bytes[i] = val as u8;
            }
            Ok(bytes)
        },
        V128Pattern::I16x8(values) => {
            let mut bytes = [0u8; 16];
            for (i, &val) in values.iter().enumerate() {
                let val_bytes = val.to_le_bytes();
                bytes[i * 2] = val_bytes[0];
                bytes[i * 2 + 1] = val_bytes[1];
            }
            Ok(bytes)
        },
        V128Pattern::I32x4(values) => {
            let mut bytes = [0u8; 16];
            for (i, &val) in values.iter().enumerate() {
                let val_bytes = val.to_le_bytes();
                bytes[i * 4..i * 4 + 4].copy_from_slice(&val_bytes);
            }
            Ok(bytes)
        },
        V128Pattern::I64x2(values) => {
            let mut bytes = [0u8; 16];
            for (i, &val) in values.iter().enumerate() {
                let val_bytes = val.to_le_bytes();
                bytes[i * 8..i * 8 + 8].copy_from_slice(&val_bytes);
            }
            Ok(bytes)
        },
        V128Pattern::F32x4(values) => {
            let mut bytes = [0u8; 16];
            for (i, pattern) in values.iter().enumerate() {
                let val = match pattern {
                    NanPattern::Value(x) => f32::from_bits(x.bits),
                    NanPattern::CanonicalNan => f32::NAN,
                    NanPattern::ArithmeticNan => f32::NAN,
                };
                let val_bytes = val.to_le_bytes();
                bytes[i * 4..i * 4 + 4].copy_from_slice(&val_bytes);
            }
            Ok(bytes)
        },
        V128Pattern::F64x2(values) => {
            let mut bytes = [0u8; 16];
            for (i, pattern) in values.iter().enumerate() {
                let val = match pattern {
                    NanPattern::Value(x) => f64::from_bits(x.bits),
                    NanPattern::CanonicalNan => f64::NAN,
                    NanPattern::ArithmeticNan => f64::NAN,
                };
                let val_bytes = val.to_le_bytes();
                bytes[i * 8..i * 8 + 8].copy_from_slice(&val_bytes);
            }
            Ok(bytes)
        },
    }
}

/// Check if runtime error matches expected trap message
pub fn is_expected_trap(error_str: &str, expected_message: &str) -> bool {
    let error_message = error_str.to_lowercase();
    let expected = expected_message.to_lowercase();

    // Common trap patterns
    let trap_patterns = [
        "out of bounds",
        "unreachable",
        "divide by zero",
        "integer overflow",
        "invalid conversion",
        "stack overflow",
        "call indirect",
        "type mismatch",
        "memory access",
        "table access",
    ];

    // Check if error message contains expected pattern
    if error_message.contains(&expected) {
        return true;
    }

    // Check if error message contains any trap pattern that matches expected
    for pattern in &trap_patterns {
        if expected.contains(pattern) && error_message.contains(pattern) {
            return true;
        }
    }

    false
}

/// Compare two values for equality, handling NaN patterns
pub fn values_equal(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::I32(a), Value::I32(b)) => a == b,
        (Value::I64(a), Value::I64(b)) => a == b,
        (Value::F32(a), Value::F32(b)) => {
            // Handle NaN comparison
            let a_val = a.value();
            let b_val = b.value();
            if a_val.is_nan() && b_val.is_nan() {
                true
            } else {
                a == b
            }
        },
        (Value::F64(a), Value::F64(b)) => {
            // Handle NaN comparison
            let a_val = a.value();
            let b_val = b.value();
            if a_val.is_nan() && b_val.is_nan() {
                true
            } else {
                a == b
            }
        },
        (Value::V128(a), Value::V128(b)) => a == b,
        (Value::Ref(a), Value::Ref(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_conversion() {
        let wast_arg = WastArg::Core(WastArgCore::I32(42));
        let wrt_value = convert_wast_arg_to_value(&wast_arg).unwrap();
        assert_eq!(wrt_value, Value::I32(42));
    }

    #[test]
    fn test_values_equal() {
        assert!(values_equal(&Value::I32(42), &Value::I32(42)));
        assert!(!values_equal(&Value::I32(42), &Value::I32(43)));

        // Test NaN handling
        let nan1 = Value::F32(FloatBits32::NAN);
        let nan2 = Value::F32(FloatBits32::NAN);
        assert!(values_equal(&nan1, &nan2));
    }
}
