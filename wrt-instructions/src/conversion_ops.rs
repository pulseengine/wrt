//! Conversion operations for WebAssembly instructions.
//!
//! This module provides pure implementations for WebAssembly conversion instructions,
//! including type conversions between numeric types.

use crate::{instruction_traits::PureInstruction, Error, Result, Value};
use wrt_error::kinds;

// When no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::ToString;

// When std is available
#[cfg(feature = "std")]
use std::string::ToString;

/// Represents a pure conversion operation for WebAssembly.
#[derive(Debug, Clone)]
pub enum ConversionOp {
    // i32 conversions
    /// Convert i64 to i32 (truncate)
    I32WrapI64,
    /// Convert f32 to i32 (signed, truncate)
    I32TruncF32S,
    /// Convert f32 to i32 (unsigned, truncate)
    I32TruncF32U,
    /// Convert f64 to i32 (signed, truncate)
    I32TruncF64S,
    /// Convert f64 to i32 (unsigned, truncate)
    I32TruncF64U,
    /// Convert i32 to f32 (reinterpret bits)
    I32ReinterpretF32,

    // i64 conversions
    /// Sign-extend i32 to i64
    I64ExtendI32S,
    /// Zero-extend i32 to i64
    I64ExtendI32U,
    /// Convert f32 to i64 (signed, truncate)
    I64TruncF32S,
    /// Convert f32 to i64 (unsigned, truncate)
    I64TruncF32U,
    /// Convert f64 to i64 (signed, truncate)
    I64TruncF64S,
    /// Convert f64 to i64 (unsigned, truncate)
    I64TruncF64U,
    /// Convert i64 to f64 (reinterpret bits)
    I64ReinterpretF64,

    // f32 conversions
    /// Convert i32 to f32 (signed)
    F32ConvertI32S,
    /// Convert i32 to f32 (unsigned)
    F32ConvertI32U,
    /// Convert i64 to f32 (signed)
    F32ConvertI64S,
    /// Convert i64 to f32 (unsigned)
    F32ConvertI64U,
    /// Demote f64 to f32
    F32DemoteF64,
    /// Reinterpret i32 bits as f32
    F32ReinterpretI32,

    // f64 conversions
    /// Convert i32 to f64 (signed)
    F64ConvertI32S,
    /// Convert i32 to f64 (unsigned)
    F64ConvertI32U,
    /// Convert i64 to f64 (signed)
    F64ConvertI64S,
    /// Convert i64 to f64 (unsigned)
    F64ConvertI64U,
    /// Promote f32 to f64
    F64PromoteF32,
    /// Reinterpret i64 bits as f64
    F64ReinterpretI64,
}

/// Execution context for conversion operations
pub trait ConversionContext {
    /// Pop a value from the context
    fn pop_conversion_value(&mut self) -> Result<Value>;

    /// Push a value to the context
    fn push_conversion_value(&mut self, value: Value) -> Result<()>;
}

impl<T: ConversionContext> PureInstruction<T, Error> for ConversionOp {
    fn execute(&self, context: &mut T) -> Result<(), Error> {
        match self {
            // i32 conversions
            Self::I32WrapI64 => {
                let a = context.pop_conversion_value()?.as_i64().ok_or_else(|| {
                    Error::invalid_type("Expected I64 for i32.wrap_i64 operand".to_string())
                })?;
                context.push_conversion_value(Value::I32(a as i32))
            }
            Self::I32TruncF32S => {
                let a = context.pop_conversion_value()?.as_f32().ok_or_else(|| {
                    Error::invalid_type("Expected F32 for i32.trunc_f32_s operand".to_string())
                })?;

                if a.is_nan() {
                    return Err(Error::from(kinds::conversion_error(
                        "NaN cannot be converted to integer",
                    )));
                }

                if a >= (i32::MAX as f32) + 1.0 || a < (i32::MIN as f32) {
                    return Err(Error::from(kinds::integer_overflow_error()));
                }

                context.push_conversion_value(Value::I32(a as i32))
            }
            Self::I32TruncF32U => {
                let a = context.pop_conversion_value()?.as_f32().ok_or_else(|| {
                    Error::invalid_type("Expected F32 for i32.trunc_f32_u operand".to_string())
                })?;

                if a.is_nan() {
                    return Err(Error::from(kinds::conversion_error(
                        "NaN cannot be converted to integer",
                    )));
                }

                if a >= (u32::MAX as f32) + 1.0 || a < 0.0 {
                    return Err(Error::from(kinds::integer_overflow_error()));
                }

                context.push_conversion_value(Value::I32(a as u32 as i32))
            }
            Self::I32ReinterpretF32 => {
                let a = context.pop_conversion_value()?.as_f32().ok_or_else(|| {
                    Error::invalid_type("Expected F32 for i32.reinterpret_f32 operand".to_string())
                })?;

                let bits = a.to_bits() as i32;
                context.push_conversion_value(Value::I32(bits))
            }

            // i64 conversions
            Self::I64ExtendI32S => {
                let a = context.pop_conversion_value()?.as_i32().ok_or_else(|| {
                    Error::invalid_type("Expected I32 for i64.extend_i32_s operand".to_string())
                })?;
                context.push_conversion_value(Value::I64(a as i64))
            }
            Self::I64ExtendI32U => {
                let a = context.pop_conversion_value()?.as_u32().ok_or_else(|| {
                    Error::invalid_type("Expected I32 for i64.extend_i32_u operand".to_string())
                })?;
                context.push_conversion_value(Value::I64(a as i64))
            }

            // f32 conversions
            Self::F32ConvertI32S => {
                let a = context.pop_conversion_value()?.as_i32().ok_or_else(|| {
                    Error::invalid_type("Expected I32 for f32.convert_i32_s operand".to_string())
                })?;
                context.push_conversion_value(Value::F32(a as f32))
            }
            Self::F32ConvertI32U => {
                let a = context.pop_conversion_value()?.as_u32().ok_or_else(|| {
                    Error::invalid_type("Expected I32 for f32.convert_i32_u operand".to_string())
                })?;
                context.push_conversion_value(Value::F32(a as f32))
            }
            Self::F32ReinterpretI32 => {
                let a = context.pop_conversion_value()?.as_i32().ok_or_else(|| {
                    Error::invalid_type("Expected I32 for f32.reinterpret_i32 operand".to_string())
                })?;

                let float = f32::from_bits(a as u32);
                context.push_conversion_value(Value::F32(float))
            }

            // f64 conversions
            Self::F64ConvertI32S => {
                let a = context.pop_conversion_value()?.as_i32().ok_or_else(|| {
                    Error::invalid_type("Expected I32 for f64.convert_i32_s operand".to_string())
                })?;
                context.push_conversion_value(Value::F64(a as f64))
            }
            Self::F64ConvertI32U => {
                let a = context.pop_conversion_value()?.as_u32().ok_or_else(|| {
                    Error::invalid_type("Expected I32 for f64.convert_i32_u operand".to_string())
                })?;
                context.push_conversion_value(Value::F64(a as f64))
            }
            Self::F64PromoteF32 => {
                let a = context.pop_conversion_value()?.as_f32().ok_or_else(|| {
                    Error::invalid_type("Expected F32 for f64.promote_f32 operand".to_string())
                })?;
                context.push_conversion_value(Value::F64(a as f64))
            }
            Self::I32TruncF64S => {
                let a = context.pop_conversion_value()?.as_f64().ok_or_else(|| {
                    Error::invalid_type("Expected F64 for i32.trunc_f64_s operand".to_string())
                })?;

                if a.is_nan() {
                    return Err(Error::from(kinds::conversion_error(
                        "NaN cannot be converted to integer",
                    )));
                }

                if a >= (i32::MAX as f64) + 1.0 || a < (i32::MIN as f64) {
                    return Err(Error::from(kinds::integer_overflow_error()));
                }

                context.push_conversion_value(Value::I32(a as i32))
            }

            // Return Ok for unimplemented operations (to be completed)
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_error::kinds;

    // Import Vec and collections based on feature flags
    #[cfg(feature = "std")]
    use std::collections::VecDeque;

    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::collections::VecDeque;

    struct MockExecutionContext {
        stack: VecDeque<Value>,
    }

    impl MockExecutionContext {
        fn new() -> Self {
            Self {
                stack: VecDeque::new(),
            }
        }
    }

    impl ConversionContext for MockExecutionContext {
        fn push_conversion_value(&mut self, value: Value) -> Result<()> {
            self.stack.push_back(value);
            Ok(())
        }

        fn pop_conversion_value(&mut self) -> Result<Value> {
            self.stack
                .pop_back()
                .ok_or_else(|| Error::from(kinds::stack_underflow()))
        }
    }

    #[test]
    fn test_i32_wrap_i64() {
        let mut context = MockExecutionContext::new();
        context
            .push_conversion_value(Value::I64(0x1_0000_0000))
            .unwrap();
        ConversionOp::I32WrapI64.execute(&mut context).unwrap();
        assert_eq!(context.pop_conversion_value().unwrap(), Value::I32(0));
    }

    #[test]
    fn test_i32_trunc_f32_s() {
        let mut context = MockExecutionContext::new();
        context.push_conversion_value(Value::F32(-123.45)).unwrap();
        ConversionOp::I32TruncF32S.execute(&mut context).unwrap();
        assert_eq!(context.pop_conversion_value().unwrap(), Value::I32(-123));
    }

    #[test]
    fn test_i32_trunc_f32_u() {
        let mut context = MockExecutionContext::new();
        context.push_conversion_value(Value::F32(123.45)).unwrap();
        ConversionOp::I32TruncF32U.execute(&mut context).unwrap();
        assert_eq!(context.pop_conversion_value().unwrap(), Value::I32(123));
    }

    #[test]
    fn test_i32_trunc_f64_s() {
        let mut context = MockExecutionContext::new();
        context.push_conversion_value(Value::F64(-123.45)).unwrap();
        ConversionOp::I32TruncF64S.execute(&mut context).unwrap();
        assert_eq!(context.pop_conversion_value().unwrap(), Value::I32(-123));
    }

    // More tests can be added as needed
}
