//! WebAssembly comparison instructions
//!
//! This module contains implementations for all WebAssembly comparison instructions,
//! including equality, inequality, and ordering operations for numeric types.

use crate::{
    behavior::{FrameBehavior, StackBehavior},
    error::{kinds, Error, Result},
    values::Value,
    StacklessEngine,
};

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
/// Pushes 1 if equal, 0 otherwise.
pub fn i32_eq(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a == b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

/// Execute an i32 inequality instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if not equal, 0 otherwise.
pub fn i32_ne(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a != b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

/// Execute an i32 signed less than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is less than second value (signed), 0 otherwise.
pub fn i32_lt_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

/// Execute an i32 unsigned less than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is less than second value (unsigned), 0 otherwise.
pub fn i32_lt_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

/// Execute an i32 signed greater than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value (signed), 0 otherwise.
pub fn i32_gt_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

/// Execute an i32 unsigned greater than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value (unsigned), 0 otherwise.
pub fn i32_gt_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

/// Execute an i32 signed less than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value (signed), 0 otherwise.
pub fn i32_le_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

/// Execute an i32 unsigned less than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value (unsigned), 0 otherwise.
pub fn i32_le_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

/// Execute an i32 signed greater than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value (signed), 0 otherwise.
pub fn i32_ge_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

/// Execute an i32 unsigned greater than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value (unsigned), 0 otherwise.
pub fn i32_ge_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
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
/// Pushes 1 if equal, 0 otherwise.
pub fn i64_eq(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a == b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an i64 inequality instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if not equal, 0 otherwise.
pub fn i64_ne(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a != b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an i64 signed less than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is less than second value (signed), 0 otherwise.
pub fn i64_lt_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an i64 unsigned less than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is less than second value (unsigned), 0 otherwise.
pub fn i64_lt_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let a = a as u64;
            let b = b as u64;
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an i64 signed greater than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value (signed), 0 otherwise.
pub fn i64_gt_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an i64 unsigned greater than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value (unsigned), 0 otherwise.
pub fn i64_gt_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let a = a as u64;
            let b = b as u64;
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an i64 signed less than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value (signed), 0 otherwise.
pub fn i64_le_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an i64 unsigned less than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value (unsigned), 0 otherwise.
pub fn i64_le_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let a = a as u64;
            let b = b as u64;
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an i64 signed greater than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value (signed), 0 otherwise.
pub fn i64_ge_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an i64 unsigned greater than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value (unsigned), 0 otherwise.
pub fn i64_ge_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let a = a as u64;
            let b = b as u64;
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an f32 equality instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if equal, 0 otherwise.
/// Note: This follows IEEE 754 equality semantics, where `NaN` != `NaN`.
pub fn f32_eq(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a == b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

/// Execute an f32 inequality instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if not equal, 0 otherwise.
/// Note: This follows IEEE 754 inequality semantics, where `NaN` != `NaN`.
pub fn f32_ne(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a != b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

/// Execute an f32 less than instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if first value is less than second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f32_lt(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

/// Execute an f32 greater than instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f32_gt(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

/// Execute an f32 less than or equal instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f32_le(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

/// Execute an f32 greater than or equal instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f32_ge(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

/// Execute an f64 equality instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if equal, 0 otherwise.
/// Note: This follows IEEE 754 equality semantics, where `NaN` != `NaN`.
pub fn f64_eq(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a == b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}

/// Execute an f64 inequality instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if not equal, 0 otherwise.
/// Note: This follows IEEE 754 inequality semantics, where `NaN` != `NaN`.
pub fn f64_ne(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a != b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}

/// Execute an f64 less than instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if first value is less than second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f64_lt(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}

/// Execute an f64 greater than instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f64_gt(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}

/// Execute an f64 less than or equal instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f64_le(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}

/// Execute an f64 greater than or equal instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f64_ge(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}
