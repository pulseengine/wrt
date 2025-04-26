//! WebAssembly comparison instructions
//!
//! This module contains implementations for all WebAssembly comparison instructions,
//! including equality, inequality, and ordering operations for numeric types.
//!
//! This module integrates with pure implementations from `wrt-instructions/comparison_ops.rs`.

use crate::{
    behavior::{FrameBehavior, StackBehavior},
    error::{kinds, Error, Result},
    values::Value,
    StacklessEngine,
};

use wrt_instructions::comparison_ops::{ComparisonContext, ComparisonOp};

/// Runtime context for comparison operations
///
/// This struct serves as a bridge between pure comparison operations
/// and the stackless engine.
struct RuntimeComparisonContext<'a, S: StackBehavior + ?Sized> {
    stack: &'a mut S,
}

impl<'a, S: StackBehavior + ?Sized> RuntimeComparisonContext<'a, S> {
    /// Create a new runtime comparison context
    fn new(stack: &'a mut S) -> Self {
        Self { stack }
    }
}

impl<'a, S: StackBehavior + ?Sized> ComparisonContext for RuntimeComparisonContext<'a, S> {
    fn pop_comparison_value(&mut self) -> wrt_instructions::Result<wrt_instructions::Value> {
        let value = self.stack.pop()?;
        // Convert from wrt::values::Value to wrt_instructions::Value
        match value {
            Value::I32(v) => Ok(wrt_instructions::Value::I32(v)),
            Value::I64(v) => Ok(wrt_instructions::Value::I64(v)),
            Value::F32(v) => Ok(wrt_instructions::Value::F32(v)),
            Value::F64(v) => Ok(wrt_instructions::Value::F64(v)),
            _ => Err(wrt_instructions::Error::invalid_type(
                "Expected numeric value".to_string(),
            )),
        }
    }

    fn push_comparison_value(
        &mut self,
        value: wrt_instructions::Value,
    ) -> wrt_instructions::Result<()> {
        // Convert from wrt_instructions::Value to wrt::values::Value
        let value = match value {
            wrt_instructions::Value::I32(v) => Value::I32(v),
            wrt_instructions::Value::I64(v) => Value::I64(v),
            wrt_instructions::Value::F32(v) => Value::F32(v),
            wrt_instructions::Value::F64(v) => Value::F64(v),
            _ => {
                return Err(wrt_instructions::Error::invalid_type(
                    "Expected numeric value".to_string(),
                ))
            }
        };
        self.stack.push(value)?;
        Ok(())
    }
}

/// Execute an i32 equality with zero instruction
///
/// Pops an i32 value from the stack and compares it with zero.
/// Pushes 1 if equal to zero, 0 otherwise.
pub fn i32_eqz(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::I32(i32::from(a == 0)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

/// Execute an i32 equality instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes a result of 1 if equal, 0 otherwise.
pub fn i32_eq(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I32Eq
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I32Eq error: {}", e)))
}

/// Execute an i32 inequality instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes a result of 1 if not equal, 0 otherwise.
pub fn i32_ne(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I32Ne
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I32Ne error: {}", e)))
}

/// Execute an i32 signed less than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than second value (signed), 0 otherwise.
pub fn i32_lt_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I32LtS
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I32LtS error: {}", e)))
}

/// Execute an i32 unsigned less than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than second value (unsigned), 0 otherwise.
pub fn i32_lt_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I32LtU
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I32LtU error: {}", e)))
}

/// Execute an i32 signed greater than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than second value (signed), 0 otherwise.
pub fn i32_gt_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I32GtS
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I32GtS error: {}", e)))
}

/// Execute an i32 unsigned greater than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than second value (unsigned), 0 otherwise.
pub fn i32_gt_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I32GtU
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I32GtU error: {}", e)))
}

/// Execute an i32 signed less than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than or equal to second value (signed), 0 otherwise.
pub fn i32_le_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I32LeS
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I32LeS error: {}", e)))
}

/// Execute an i32 unsigned less than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than or equal to second value (unsigned), 0 otherwise.
pub fn i32_le_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I32LeU
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I32LeU error: {}", e)))
}

/// Execute an i32 signed greater than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than or equal to second value (signed), 0 otherwise.
pub fn i32_ge_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I32GeS
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I32GeS error: {}", e)))
}

/// Execute an i32 unsigned greater than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than or equal to second value (unsigned), 0 otherwise.
pub fn i32_ge_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I32GeU
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I32GeU error: {}", e)))
}

/// Execute an i64 equality with zero instruction
///
/// Pops an i64 value from the stack and compares it with zero.
/// Pushes 1 if equal to zero, 0 otherwise.
pub fn i64_eqz(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I64(a) => {
            stack.push(Value::I32(i32::from(a == 0)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an i64 equality instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes a result of 1 if equal, 0 otherwise.
pub fn i64_eq(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I64Eq
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I64Eq error: {}", e)))
}

/// Execute an i64 inequality instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes a result of 1 if not equal, 0 otherwise.
pub fn i64_ne(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I64Ne
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I64Ne error: {}", e)))
}

/// Execute an i64 signed less than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than second value (signed), 0 otherwise.
pub fn i64_lt_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I64LtS
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I64LtS error: {}", e)))
}

/// Execute an i64 unsigned less than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than second value (unsigned), 0 otherwise.
pub fn i64_lt_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I64LtU
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I64LtU error: {}", e)))
}

/// Execute an i64 signed greater than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than second value (signed), 0 otherwise.
pub fn i64_gt_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I64GtS
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I64GtS error: {}", e)))
}

/// Execute an i64 unsigned greater than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than second value (unsigned), 0 otherwise.
pub fn i64_gt_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I64GtU
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I64GtU error: {}", e)))
}

/// Execute an i64 signed less than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than or equal to second value (signed), 0 otherwise.
pub fn i64_le_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I64LeS
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I64LeS error: {}", e)))
}

/// Execute an i64 unsigned less than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than or equal to second value (unsigned), 0 otherwise.
pub fn i64_le_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I64LeU
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I64LeU error: {}", e)))
}

/// Execute an i64 signed greater than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than or equal to second value (signed), 0 otherwise.
pub fn i64_ge_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I64GeS
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I64GeS error: {}", e)))
}

/// Execute an i64 unsigned greater than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than or equal to second value (unsigned), 0 otherwise.
pub fn i64_ge_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::I64GeU
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::I64GeU error: {}", e)))
}

/// Execute an f32 equality instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes a result of 1 if equal, 0 otherwise.
pub fn f32_eq(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F32Eq
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F32Eq error: {}", e)))
}

/// Execute an f32 inequality instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes a result of 1 if not equal, 0 otherwise.
pub fn f32_ne(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F32Ne
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F32Ne error: {}", e)))
}

/// Execute an f32 less than instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than second value, 0 otherwise.
pub fn f32_lt(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F32Lt
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F32Lt error: {}", e)))
}

/// Execute an f32 greater than instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than second value, 0 otherwise.
pub fn f32_gt(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F32Gt
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F32Gt error: {}", e)))
}

/// Execute an f32 less than or equal instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than or equal to second value, 0 otherwise.
pub fn f32_le(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F32Le
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F32Le error: {}", e)))
}

/// Execute an f32 greater than or equal instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than or equal to second value, 0 otherwise.
pub fn f32_ge(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F32Ge
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F32Ge error: {}", e)))
}

/// Execute an f64 equality instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes a result of 1 if equal, 0 otherwise.
pub fn f64_eq(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F64Eq
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F64Eq error: {}", e)))
}

/// Execute an f64 inequality instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes a result of 1 if not equal, 0 otherwise.
pub fn f64_ne(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F64Ne
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F64Ne error: {}", e)))
}

/// Execute an f64 less than instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than second value, 0 otherwise.
pub fn f64_lt(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F64Lt
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F64Lt error: {}", e)))
}

/// Execute an f64 greater than instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than second value, 0 otherwise.
pub fn f64_gt(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F64Gt
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F64Gt error: {}", e)))
}

/// Execute an f64 less than or equal instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes a result of 1 if first value is less than or equal to second value, 0 otherwise.
pub fn f64_le(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F64Le
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F64Le error: {}", e)))
}

/// Execute an f64 greater than or equal instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes a result of 1 if first value is greater than or equal to second value, 0 otherwise.
pub fn f64_ge(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let mut context = RuntimeComparisonContext::new(stack);
    ComparisonOp::F64Ge
        .execute(&mut context)
        .map_err(|e| Error::invalid_type(format!("ComparisonOp::F64Ge error: {}", e)))
}
