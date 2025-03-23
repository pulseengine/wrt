//! WebAssembly comparison instructions
//!
//! This module contains implementations for all WebAssembly comparison instructions,
//! including equality, inequality, and ordering operations for numeric types.

use crate::error::Error;
use crate::values::Value;
use crate::Vec;

/// Execute an i32 equality with zero instruction
///
/// Pops an i32 value from the stack and compares it with zero.
/// Pushes 1 if equal to zero, 0 otherwise.
pub fn i32_eqz(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let Value::I32(v) = val {
        if v == 0 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i32 value".into()))
    }
}

/// Execute an i32 equality instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if equal, 0 otherwise.
pub fn i32_eq(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I32(v1), Value::I32(v2)) = (val1, val2) {
        if v1 == v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i32 values".into()))
    }
}

/// Execute an i32 inequality instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if not equal, 0 otherwise.
pub fn i32_ne(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I32(v1), Value::I32(v2)) = (val1, val2) {
        if v1 == v2 {
            stack.push(Value::I32(0));
        } else {
            stack.push(Value::I32(1));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i32 values".into()))
    }
}

/// Execute an i32 signed less than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is less than second value (signed), 0 otherwise.
pub fn i32_lt_s(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I32(v1), Value::I32(v2)) = (val1, val2) {
        if v1 < v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i32 values".into()))
    }
}

/// Execute an i32 unsigned less than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is less than second value (unsigned), 0 otherwise.
pub fn i32_lt_u(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I32(v1), Value::I32(v2)) = (val1, val2) {
        if (v1 as u32) < (v2 as u32) {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i32 values".into()))
    }
}

/// Execute an i32 signed greater than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value (signed), 0 otherwise.
pub fn i32_gt_s(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I32(v1), Value::I32(v2)) = (val1, val2) {
        if v1 > v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i32 values".into()))
    }
}

/// Execute an i32 unsigned greater than instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value (unsigned), 0 otherwise.
pub fn i32_gt_u(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I32(v1), Value::I32(v2)) = (val1, val2) {
        if (v1 as u32) > (v2 as u32) {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i32 values".into()))
    }
}

/// Execute an i32 signed less than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value (signed), 0 otherwise.
pub fn i32_le_s(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I32(v1), Value::I32(v2)) = (val1, val2) {
        if v1 <= v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i32 values".into()))
    }
}

/// Execute an i32 unsigned less than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value (unsigned), 0 otherwise.
pub fn i32_le_u(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I32(v1), Value::I32(v2)) = (val1, val2) {
        if (v1 as u32) <= (v2 as u32) {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i32 values".into()))
    }
}

/// Execute an i32 signed greater than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value (signed), 0 otherwise.
pub fn i32_ge_s(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I32(v1), Value::I32(v2)) = (val1, val2) {
        if v1 >= v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i32 values".into()))
    }
}

/// Execute an i32 unsigned greater than or equal instruction
///
/// Pops two i32 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value (unsigned), 0 otherwise.
pub fn i32_ge_u(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I32(v1), Value::I32(v2)) = (val1, val2) {
        if (v1 as u32) >= (v2 as u32) {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i32 values".into()))
    }
}

/// Execute an i64 equality with zero instruction
///
/// Pops an i64 value from the stack and compares it with zero.
/// Pushes 1 if equal to zero, 0 otherwise.
pub fn i64_eqz(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let Value::I64(v) = val {
        if v == 0 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i64 value".into()))
    }
}

/// Execute an i64 equality instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if equal, 0 otherwise.
pub fn i64_eq(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I64(v1), Value::I64(v2)) = (val1, val2) {
        if v1 == v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i64 values".into()))
    }
}

/// Execute an i64 inequality instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if not equal, 0 otherwise.
pub fn i64_ne(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I64(v1), Value::I64(v2)) = (val1, val2) {
        if v1 == v2 {
            stack.push(Value::I32(0));
        } else {
            stack.push(Value::I32(1));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i64 values".into()))
    }
}

/// Execute an i64 signed less than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is less than second value (signed), 0 otherwise.
pub fn i64_lt_s(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I64(v1), Value::I64(v2)) = (val1, val2) {
        if v1 < v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i64 values".into()))
    }
}

/// Execute an i64 unsigned less than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is less than second value (unsigned), 0 otherwise.
pub fn i64_lt_u(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I64(v1), Value::I64(v2)) = (val1, val2) {
        if (v1 as u64) < (v2 as u64) {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i64 values".into()))
    }
}

/// Execute an i64 signed greater than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value (signed), 0 otherwise.
pub fn i64_gt_s(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I64(v1), Value::I64(v2)) = (val1, val2) {
        if v1 > v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i64 values".into()))
    }
}

/// Execute an i64 unsigned greater than instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value (unsigned), 0 otherwise.
pub fn i64_gt_u(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I64(v1), Value::I64(v2)) = (val1, val2) {
        if (v1 as u64) > (v2 as u64) {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i64 values".into()))
    }
}

/// Execute an i64 signed less than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value (signed), 0 otherwise.
pub fn i64_le_s(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I64(v1), Value::I64(v2)) = (val1, val2) {
        if v1 <= v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i64 values".into()))
    }
}

/// Execute an i64 unsigned less than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value (unsigned), 0 otherwise.
pub fn i64_le_u(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I64(v1), Value::I64(v2)) = (val1, val2) {
        if (v1 as u64) <= (v2 as u64) {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i64 values".into()))
    }
}

/// Execute an i64 signed greater than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value (signed), 0 otherwise.
pub fn i64_ge_s(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I64(v1), Value::I64(v2)) = (val1, val2) {
        if v1 >= v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i64 values".into()))
    }
}

/// Execute an i64 unsigned greater than or equal instruction
///
/// Pops two i64 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value (unsigned), 0 otherwise.
pub fn i64_ge_u(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::I64(v1), Value::I64(v2)) = (val1, val2) {
        if (v1 as u64) >= (v2 as u64) {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected i64 values".into()))
    }
}

/// Execute an f32 equality instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if equal, 0 otherwise.
/// Note: This follows IEEE 754 equality semantics, where `NaN` != `NaN`.
pub fn f32_eq(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F32(v1), Value::F32(v2)) = (val1, val2) {
        if v1 == v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f32 values".into()))
    }
}

/// Execute an f32 inequality instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if not equal, 0 otherwise.
/// Note: This follows IEEE 754 inequality semantics, where `NaN` != `NaN`.
pub fn f32_ne(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F32(v1), Value::F32(v2)) = (val1, val2) {
        if v1 == v2 {
            stack.push(Value::I32(0));
        } else {
            stack.push(Value::I32(1));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f32 values".into()))
    }
}

/// Execute an f32 less than instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if first value is less than second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f32_lt(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F32(v1), Value::F32(v2)) = (val1, val2) {
        if v1 < v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f32 values".into()))
    }
}

/// Execute an f32 greater than instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f32_gt(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F32(v1), Value::F32(v2)) = (val1, val2) {
        if v1 > v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f32 values".into()))
    }
}

/// Execute an f32 less than or equal instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f32_le(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F32(v1), Value::F32(v2)) = (val1, val2) {
        if v1 <= v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f32 values".into()))
    }
}

/// Execute an f32 greater than or equal instruction
///
/// Pops two f32 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f32_ge(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F32(v1), Value::F32(v2)) = (val1, val2) {
        if v1 >= v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f32 values".into()))
    }
}

/// Execute an f64 equality instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if equal, 0 otherwise.
/// Note: This follows IEEE 754 equality semantics, where `NaN` != `NaN`.
pub fn f64_eq(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F64(v1), Value::F64(v2)) = (val1, val2) {
        if v1 == v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f64 values".into()))
    }
}

/// Execute an f64 inequality instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if not equal, 0 otherwise.
/// Note: This follows IEEE 754 inequality semantics, where `NaN` != `NaN`.
pub fn f64_ne(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F64(v1), Value::F64(v2)) = (val1, val2) {
        if v1 == v2 {
            stack.push(Value::I32(0));
        } else {
            stack.push(Value::I32(1));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f64 values".into()))
    }
}

/// Execute an f64 less than instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if first value is less than second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f64_lt(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F64(v1), Value::F64(v2)) = (val1, val2) {
        if v1 < v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f64 values".into()))
    }
}

/// Execute an f64 greater than instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if first value is greater than second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f64_gt(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F64(v1), Value::F64(v2)) = (val1, val2) {
        if v1 > v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f64 values".into()))
    }
}

/// Execute an f64 less than or equal instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if first value is less than or equal to second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f64_le(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F64(v1), Value::F64(v2)) = (val1, val2) {
        if v1 <= v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f64 values".into()))
    }
}

/// Execute an f64 greater than or equal instruction
///
/// Pops two f64 values from the stack and compares them.
/// Pushes 1 if first value is greater than or equal to second value, 0 otherwise.
/// Note: This follows IEEE 754 comparison semantics, where `NaN` comparisons return false.
pub fn f64_ge(stack: &mut Vec<Value>) -> std::result::Result<(), Error> {
    let val2 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;
    let val1 = stack
        .pop()
        .ok_or(Error::Execution("Stack underflow".into()))?;

    if let (Value::F64(v1), Value::F64(v2)) = (val1, val2) {
        if v1 >= v2 {
            stack.push(Value::I32(1));
        } else {
            stack.push(Value::I32(0));
        }
        Ok(())
    } else {
        Err(Error::Execution("Expected f64 values".into()))
    }
}
