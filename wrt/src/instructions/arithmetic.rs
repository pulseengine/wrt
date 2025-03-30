//! WebAssembly arithmetic instructions
//!
//! This module contains implementations for all WebAssembly arithmetic instructions,
//! including addition, subtraction, multiplication, division, and more.

use crate::{
    behavior::FrameBehavior,
    error::{Error, Result},
    stack::Stack,
    values::Value,
};

/// Execute an i32 addition instruction
///
/// Pops two i32 values from the stack, adds them, and pushes the result.
pub fn i32_add(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    println!("DEBUG: i32_add - Starting execution");
    println!("DEBUG: i32_add - Stack before: {:?}", stack.values());

    // Check for stack underflow
    if stack.len() < 2 {
        println!(
            "DEBUG: i32_add - Error: Stack underflow. Stack has {} values, need at least 2",
            stack.len()
        );
        return Err(Error::StackUnderflow);
    }

    // Pop the values
    let b = stack.pop()?;
    let a = stack.pop()?;

    println!("DEBUG: i32_add - Popped values: a={a:?}, b={b:?}");

    // Perform the addition
    if let (Value::I32(a), Value::I32(b)) = (a, b) {
        let result = a.wrapping_add(b);
        println!("DEBUG: i32_add - Result: {a} + {b} = {result}");

        // Push the result back to the stack
        stack.push(Value::I32(result))?;
        println!("DEBUG: i32_add - Stack after: {:?}", stack.values());

        Ok(())
    } else {
        println!("DEBUG: i32_add - Error: Type mismatch. Expected I32 values");
        Err(Error::InvalidType(
            "Type mismatch in i32_add: Expected I32 values".to_string(),
        ))
    }
}

/// Execute an i32 subtraction instruction
///
/// Pops two i32 values from the stack, subtracts the second from the first, and pushes the result.
pub fn i32_sub(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let result = a.wrapping_sub(b);
            stack.push(Value::I32(result))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 multiplication instruction
///
/// Pops two i32 values from the stack, multiplies them, and pushes the result.
pub fn i32_mul(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let result = a.wrapping_mul(b);
            stack.push(Value::I32(result))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 signed division instruction
///
/// Pops two i32 values from the stack, divides the first by the second (signed),
/// and pushes the result. Returns an error if dividing by zero.
pub fn i32_div_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            if b == 0 {
                return Err(Error::DivisionByZero);
            }
            if a == i32::MIN && b == -1 {
                return Err(Error::IntegerOverflow);
            }
            stack.push(Value::I32(a.wrapping_div(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 unsigned division instruction
///
/// Pops two i32 values from the stack, divides the first by the second (unsigned),
/// and pushes the result. Returns an error if dividing by zero.
pub fn i32_div_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            if b == 0 {
                return Err(Error::DivisionByZero);
            }
            let a = a as u32;
            let b = b as u32;
            stack.push(Value::I32((a / b) as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 signed remainder instruction
///
/// Pops two i32 values from the stack, computes the remainder of the first divided
/// by the second (signed), and pushes the result. Returns an error if dividing by zero.
pub fn i32_rem_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            if b == 0 {
                return Err(Error::DivisionByZero);
            }
            stack.push(Value::I32(a.wrapping_rem(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 unsigned remainder instruction
///
/// Pops two i32 values from the stack, computes the remainder of the first divided
/// by the second (unsigned), and pushes the result. Returns an error if dividing by zero.
pub fn i32_rem_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            if b == 0 {
                return Err(Error::DivisionByZero);
            }
            let a = a as u32;
            let b = b as u32;
            stack.push(Value::I32((a % b) as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 bitwise AND instruction
///
/// Pops two i32 values from the stack, computes their bitwise AND, and pushes the result.
pub fn i32_and(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(a & b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 bitwise OR instruction
///
/// Pops two i32 values from the stack, computes their bitwise OR, and pushes the result.
pub fn i32_or(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(a | b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 bitwise XOR instruction
///
/// Pops two i32 values from the stack, computes their bitwise XOR, and pushes the result.
pub fn i32_xor(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(a ^ b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 shift left instruction
///
/// Pops two i32 values from the stack, shifts the first left by the lower 5 bits
/// of the second value, and pushes the result.
pub fn i32_shl(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let result = a.wrapping_shl(b as u32 % 32);
            stack.push(Value::I32(result))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 signed shift right instruction
///
/// Pops two i32 values from the stack, shifts the first right by the lower 5 bits
/// of the second value (signed, preserving sign bit), and pushes the result.
pub fn i32_shr_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let result = a.wrapping_shr(b as u32 % 32);
            stack.push(Value::I32(result))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 unsigned shift right instruction
///
/// Pops two i32 values from the stack, shifts the first right by the lower 5 bits
/// of the second value (unsigned, filling with zeros), and pushes the result.
pub fn i32_shr_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let result = ((a as u32).wrapping_shr(b as u32 % 32)) as i32;
            stack.push(Value::I32(result))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 rotate left instruction
///
/// Pops two i32 values from the stack, rotates the bits of the first left by the
/// lower 5 bits of the second value, and pushes the result.
pub fn i32_rotl(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let shift = (b as u32) % 32;
            let value = a as u32;
            let result = value.rotate_left(shift);
            stack.push(Value::I32(result as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 rotate right instruction
///
/// Pops two i32 values from the stack, rotates the bits of the first right by the
/// lower 5 bits of the second value, and pushes the result.
pub fn i32_rotr(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let shift = (b as u32) % 32;
            let value = a as u32;
            let result = value.rotate_right(shift);
            stack.push(Value::I32(result as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i64 addition instruction
///
/// Pops two i64 values from the stack, adds them, and pushes the result.
pub fn i64_add(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I64(a.wrapping_add(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 subtraction instruction
///
/// Pops two i64 values from the stack, subtracts the second from the first, and pushes the result.
pub fn i64_sub(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I64(a.wrapping_sub(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 multiplication instruction
///
/// Pops two i64 values from the stack, multiplies them, and pushes the result.
pub fn i64_mul(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I64(a.wrapping_mul(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 signed division instruction
///
/// Pops two i64 values from the stack, divides the first by the second (signed),
/// and pushes the result. Returns an error if dividing by zero.
pub fn i64_div_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            if b == 0 {
                return Err(Error::DivisionByZero);
            }
            if a == i64::MIN && b == -1 {
                return Err(Error::IntegerOverflow);
            }
            stack.push(Value::I64(a.wrapping_div(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 unsigned division instruction
///
/// Pops two i64 values from the stack, divides the first by the second (unsigned),
/// and pushes the result. Returns an error if dividing by zero.
pub fn i64_div_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            if b == 0 {
                return Err(Error::DivisionByZero);
            }
            let a = a as u64;
            let b = b as u64;
            stack.push(Value::I64((a / b) as i64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 signed remainder instruction
///
/// Pops two i64 values from the stack, computes the remainder of the first divided
/// by the second (signed), and pushes the result. Returns an error if dividing by zero.
pub fn i64_rem_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            if b == 0 {
                return Err(Error::DivisionByZero);
            }
            stack.push(Value::I64(a.wrapping_rem(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 unsigned remainder instruction
///
/// Pops two i64 values from the stack, computes the remainder of the first divided
/// by the second (unsigned), and pushes the result. Returns an error if dividing by zero.
pub fn i64_rem_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            if b == 0 {
                return Err(Error::DivisionByZero);
            }
            let a = a as u64;
            let b = b as u64;
            stack.push(Value::I64((a % b) as i64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 bitwise AND instruction
///
/// Pops two i64 values from the stack, computes their bitwise AND, and pushes the result.
pub fn i64_and(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I64(a & b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 bitwise OR instruction
///
/// Pops two i64 values from the stack, computes their bitwise OR, and pushes the result.
pub fn i64_or(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I64(a | b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 bitwise XOR instruction
///
/// Pops two i64 values from the stack, computes their bitwise XOR, and pushes the result.
pub fn i64_xor(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I64(a ^ b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 shift left instruction
///
/// Pops two i64 values from the stack, shifts the first left by the lower 6 bits
/// of the second value, and pushes the result.
pub fn i64_shl(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let result = a.wrapping_shl((b as u64 % 64) as u32);
            stack.push(Value::I64(result))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 signed shift right instruction
///
/// Pops two i64 values from the stack, shifts the first right by the lower 6 bits
/// of the second value (signed, preserving sign bit), and pushes the result.
pub fn i64_shr_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let result = a.wrapping_shr((b as u64 % 64) as u32);
            stack.push(Value::I64(result))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 unsigned shift right instruction
///
/// Pops two i64 values from the stack, shifts the first right by the lower 6 bits
/// of the second value (unsigned, filling with zeros), and pushes the result.
pub fn i64_shr_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let result = ((a as u64).wrapping_shr((b as u64 % 64) as u32)) as i64;
            stack.push(Value::I64(result))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 rotate left instruction
///
/// Pops two i64 values from the stack, rotates the bits of the first left by the
/// lower 6 bits of the second value, and pushes the result.
pub fn i64_rotl(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let shift = (b as u64) % 64;
            let value = a as u64;
            let result = value.rotate_left(shift as u32);
            stack.push(Value::I64(result as i64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an i64 rotate right instruction
///
/// Pops two i64 values from the stack, rotates the bits of the first right by the
/// lower 6 bits of the second value, and pushes the result.
pub fn i64_rotr(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let shift = (b as u64) % 64;
            let value = a as u64;
            let result = value.rotate_right(shift as u32);
            stack.push(Value::I64(result as i64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

/// Execute an f32 ceiling instruction
///
/// Pops an f32 value from the stack, rounds it up to the nearest integer, and pushes the result.
pub fn f32_ceil(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::F32(a.ceil()))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f32 floor instruction
///
/// Pops an f32 value from the stack, rounds it down to the nearest integer, and pushes the result.
pub fn f32_floor(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::F32(a.floor()))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f32 truncate instruction
///
/// Pops an f32 value from the stack, truncates it toward zero to the nearest integer, and pushes the result.
pub fn f32_trunc(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::F32(a.trunc()))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f32 nearest instruction
///
/// Pops an f32 value from the stack, rounds it to the nearest integer, and pushes the result.
pub fn f32_nearest(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::F32(a.round()))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f32 add instruction
///
/// Pops two f32 values from the stack, adds them, and pushes the result.
pub fn f32_add(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::F32(a + b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f32 subtract instruction
///
/// Pops two f32 values from the stack, subtracts the second from the first, and pushes the result.
pub fn f32_sub(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::F32(a - b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f32 multiply instruction
///
/// Pops two f32 values from the stack, multiplies them, and pushes the result.
pub fn f32_mul(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::F32(a * b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f32 divide instruction
///
/// Pops two f32 values from the stack, divides the first by the second, and pushes the result.
pub fn f32_div(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::F32(a / b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f64 addition instruction
///
/// Pops two f64 values from the stack, adds them, and pushes the result.
pub fn f64_add(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::F64(a + b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f64 subtraction instruction
///
/// Pops two f64 values from the stack, subtracts the second from the first, and pushes the result.
pub fn f64_sub(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::F64(a - b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f64 multiplication instruction
///
/// Pops two f64 values from the stack, multiplies them, and pushes the result.
pub fn f64_mul(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::F64(a * b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f64 division instruction
///
/// Pops two f64 values from the stack, divides the first by the second, and pushes the result.
pub fn f64_div(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            if b == 0.0 {
                return Err(Error::DivisionByZero);
            }
            stack.push(Value::F64(a / b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f64 maximum instruction
///
/// Pops two f64 values from the stack, pushes the maximum of the two values.
pub fn f64_max(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::F64(a.max(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f64 minimum instruction
///
/// Pops two f64 values from the stack, pushes the minimum of the two values.
pub fn f64_min(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::F64(a.min(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f64 square root instruction
///
/// Pops an f64 value from the stack, computes its square root, and pushes the result.
pub fn f64_sqrt(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::F64(a.sqrt()))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f64 nearest instruction
///
/// Pops an f64 value from the stack, rounds it to the nearest integer, and pushes the result.
/// Round-to-nearest rounds to the nearest integral value; if two integral values are equally near,
/// rounds to the even value (Banker's rounding).
pub fn f64_nearest(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::F64(a.round()))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f64 truncate instruction
///
/// Pops an f64 value from the stack, truncates it toward zero to the nearest integer, and pushes the result.
/// Truncation removes the fractional part, rounding toward zero.
pub fn f64_trunc(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::F64(a.trunc()))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f64 floor instruction
///
/// Pops an f64 value from the stack, rounds it down to the nearest integer, and pushes the result.
/// Floor rounding always rounds towards negative infinity.
pub fn f64_floor(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::F64(a.floor()))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f64 ceil instruction
///
/// Pops an f64 value from the stack, rounds it up to the nearest integer, and pushes the result.
/// Ceiling rounding always rounds towards positive infinity.
pub fn f64_ceil(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    unimplemented!("Duplicate function - use the original implementation")
}

/// Execute an f64 copysign instruction
///
/// Pops two f64 values from the stack, copies the sign of the second to the first, and pushes the result.
pub fn f64_copysign(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    unimplemented!("Duplicate function - use the original implementation")
}

/// Execute an f64 absolute value instruction
///
/// Pops an f64 value from the stack, computes its absolute value, and pushes the result.
pub fn f64_abs(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::F64(a.abs()))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f64 negate instruction
///
/// Pops an f64 value from the stack, negates it, and pushes the result.
pub fn f64_neg(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::F64(-a))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

/// Execute an f32 maximum instruction
///
/// Pops two f32 values from the stack, computes the maximum, and pushes the result.
pub fn f32_max(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::F32(a.max(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f32 minimum instruction
///
/// Pops two f32 values from the stack, computes the minimum, and pushes the result.
pub fn f32_min(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::F32(a.min(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f32 copysign instruction
///
/// Pops two f32 values from the stack, copies the sign of the second to the first, and pushes the result.
pub fn f32_copysign(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            let result = a.copysign(b);
            stack.push(Value::F32(result))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f32 square root instruction
///
/// Pops an f32 value from the stack, computes its square root, and pushes the result.
pub fn f32_sqrt(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::F32(a.sqrt()))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

/// Execute an f32 negation instruction
///
/// Pops an f32 value from the stack, negates it, and pushes the result.
pub fn f32_neg(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::F32(-a))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}
