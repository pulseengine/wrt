//! WebAssembly arithmetic instructions
//!
//! This module contains implementations for all WebAssembly arithmetic instructions,
//! including addition, subtraction, multiplication, division, and more.
//!
//! This module integrates with the pure implementations in `wrt-instructions/arithmetic_ops.rs`,
//! providing the runtime-specific context needed for execution.

use crate::{
    behavior::{ControlFlow, ControlFlowBehavior, FrameBehavior, NullBehavior, StackBehavior},
    error::{kinds, Error, Result},
    memory::PAGE_SIZE,
    prelude::TypesValue as Value,
    stackless::StacklessEngine,
};

// Import the pure implementations from wrt-instructions
use wrt_instructions::arithmetic_ops::{ArithmeticContext, ArithmeticOp};

/// Runtime adapter that bridges the pure arithmetic operations with the stackless engine
struct RuntimeArithmeticContext<'a> {
    stack: &'a mut dyn StackBehavior,
}

impl<'a> ArithmeticContext for RuntimeArithmeticContext<'a> {
    fn pop_arithmetic_value(&mut self) -> wrt_instructions::Result<Value> {
        self.stack
            .pop()
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn push_arithmetic_value(&mut self, value: Value) -> wrt_instructions::Result<()> {
        self.stack
            .push(value)
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }
}

/// Execute an i32 addition instruction
///
/// Pops two i32 values from the stack, adds them, and pushes the result.
pub fn i32_add<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    // Create the runtime context and delegate to the pure implementation
    let mut context = RuntimeArithmeticContext { stack };
    ArithmeticOp::I32Add
        .execute(&mut context)
        .map_err(|e| Error::new(e.kind()))
}

/// Execute an i32 subtraction instruction
///
/// Pops two i32 values from the stack, subtracts the second from the first, and pushes the result.
pub fn i32_sub<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    // Create the runtime context and delegate to the pure implementation
    let mut context = RuntimeArithmeticContext { stack };
    ArithmeticOp::I32Sub
        .execute(&mut context)
        .map_err(|e| Error::new(e.kind()))
}

/// Execute an i32 multiplication instruction
///
/// Pops two i32 values from the stack, multiplies them, and pushes the result.
pub fn i32_mul<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    // Create the runtime context and delegate to the pure implementation
    let mut context = RuntimeArithmeticContext { stack };
    ArithmeticOp::I32Mul
        .execute(&mut context)
        .map_err(|e| Error::new(e.kind()))
}

/// Execute an i32 signed division instruction
///
/// Pops two i32 values from the stack, divides the first by the second (signed),
/// and pushes the result. Returns an error if dividing by zero.
pub fn i32_div_s<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    // Create the runtime context and delegate to the pure implementation
    let mut context = RuntimeArithmeticContext { stack };
    ArithmeticOp::I32DivS
        .execute(&mut context)
        .map_err(|e| Error::new(e.kind()))
}

/// Execute an i32 unsigned division instruction
///
/// Pops two i32 values from the stack, divides the first by the second (unsigned),
/// and pushes the result. Returns an error if dividing by zero.
pub fn i32_div_u<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    // Create the runtime context and delegate to the pure implementation
    let mut context = RuntimeArithmeticContext { stack };
    ArithmeticOp::I32DivU
        .execute(&mut context)
        .map_err(|e| Error::new(e.kind()))
}

/// Execute an i32 signed remainder instruction
///
/// Pops two i32 values from the stack, computes the remainder of the first divided
/// by the second (signed), and pushes the result. Returns an error if dividing by zero.
pub fn i32_rem_s<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    // Create the runtime context and delegate to the pure implementation
    let mut context = RuntimeArithmeticContext { stack };
    ArithmeticOp::I32RemS
        .execute(&mut context)
        .map_err(|e| Error::new(e.kind()))
}

/// Execute an i32 unsigned remainder instruction
///
/// Pops two i32 values from the stack, computes the remainder of the first divided
/// by the second (unsigned), and pushes the result. Returns an error if dividing by zero.
pub fn i32_rem_u<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    // Create the runtime context and delegate to the pure implementation
    let mut context = RuntimeArithmeticContext { stack };
    ArithmeticOp::I32RemU
        .execute(&mut context)
        .map_err(|e| Error::new(e.kind()))
}

/// Execute an i32 bitwise AND instruction
///
/// Pops two i32 values from the stack, computes their bitwise AND, and pushes the result.
pub fn i32_and<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    println!("DEBUG: i32_and - Stack BEFORE pop: {:?}", stack.values());
    let b_val = stack.pop()?;
    let b = b_val.as_i32().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I32 for i32.and operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i32().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I32 for i32.and operand a, found {}",
            a_val.value_type()
        ))
    })?;
    println!("DEBUG: i32_and - Popped a={:?}, b={:?}", a, b);
    stack.push(Value::I32(a & b))?;
    println!("DEBUG: i32_and - Stack BEFORE push: {:?}", stack.values());
    Ok(())
}

/// Execute an i32 bitwise OR instruction
///
/// Pops two i32 values from the stack, computes their bitwise OR, and pushes the result.
pub fn i32_or<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_i32().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I32 for i32.or operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i32().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I32 for i32.or operand a, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I32(a | b))?;
    Ok(())
}

/// Execute an i32 bitwise XOR instruction
///
/// Pops two i32 values from the stack, computes their bitwise XOR, and pushes the result.
pub fn i32_xor<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::invalid_type("Expected I32".to_string()))?;
    let a = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::invalid_type("Expected I32".to_string()))?;
    stack.push(Value::I32(a ^ b))?;
    Ok(())
}

/// Execute an i32 shift left instruction
///
/// Pops two i32 values from the stack, shifts the first left by the lower 5 bits
/// of the second value, and pushes the result.
pub fn i32_shl<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_u32()
        .ok_or_else(|| Error::invalid_type("Expected U32".to_string()))?;
    let a = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::invalid_type("Expected I32".to_string()))?;
    stack.push(Value::I32(a.wrapping_shl(b % 32)))?;
    Ok(())
}

/// Execute an i32 signed shift right instruction
///
/// Pops two i32 values from the stack, shifts the first right by the lower 5 bits
/// of the second value (signed, preserving sign bit), and pushes the result.
pub fn i32_shr_s<TFrame: FrameBehavior + ?Sized>(
    _frame: &mut TFrame,
    stack: &mut dyn StackBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_u32()
        .ok_or_else(|| Error::invalid_type("Expected U32".to_string()))?;
    let a = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::invalid_type("Expected I32".to_string()))?;
    stack.push(Value::I32(a.wrapping_shr(b % 32)))?;
    Ok(())
}

/// Execute an i32 unsigned shift right instruction
///
/// Pops two i32 values from the stack, shifts the first right by the lower 5 bits
/// of the second value (unsigned, filling with zeros), and pushes the result.
pub fn i32_shr_u(
    frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_u32()
        .ok_or_else(|| Error::invalid_type("Expected U32".to_string()))?;
    let a = stack
        .pop()?
        .as_u32()
        .ok_or_else(|| Error::invalid_type("Expected U32".to_string()))?;
    stack.push(Value::I32(a.wrapping_shr(b % 32) as i32))?;
    Ok(())
}

/// Execute an i32 rotate left instruction
///
/// Pops two i32 values from the stack, rotates the bits of the first left by the
/// lower 5 bits of the second value, and pushes the result.
pub fn i32_rotl(
    frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_u32()
        .ok_or_else(|| Error::invalid_type("Expected U32".to_string()))?;
    let a = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::invalid_type("Expected I32".to_string()))?;
    stack.push(Value::I32(a.rotate_left(b % 32)))?;
    Ok(())
}

/// Execute an i32 rotate right instruction
///
/// Pops two i32 values from the stack, rotates the bits of the first right by the
/// lower 5 bits of the second value, and pushes the result.
pub fn i32_rotr(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_u32()
        .ok_or_else(|| Error::invalid_type("Expected U32".to_string()))?;
    let a = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::invalid_type("Expected I32".to_string()))?;
    stack.push(Value::I32(a.rotate_right(b % 32)))?;
    Ok(())
}

/// Execute an i64 addition instruction
///
/// Pops two i64 values from the stack, adds them, and pushes the result.
pub fn i64_add(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.add operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.add operand a, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I64(a.wrapping_add(b)))?;
    Ok(())
}

/// Execute an i64 subtraction instruction
///
/// Pops two i64 values from the stack, subtracts the second from the first, and pushes the result.
pub fn i64_sub(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.sub operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.sub operand a, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I64(a.wrapping_sub(b)))?;
    Ok(())
}

/// Execute an i64 multiplication instruction
///
/// Pops two i64 values from the stack, multiplies them, and pushes the result.
pub fn i64_mul(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.mul operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.mul operand a, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I64(a.wrapping_mul(b)))?;
    Ok(())
}

/// Execute an i64 signed division instruction
///
/// Pops two i64 values from the stack, divides the first by the second (signed),
/// and pushes the result. Returns an error if dividing by zero.
pub fn i64_div_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.div_s operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.div_s operand a, found {}",
            a_val.value_type()
        ))
    })?;
    if b == 0 {
        return Err(Error::division_by_zero());
    }
    if a == i64::MIN && b == -1 {
        return Err(Error::integer_overflow());
    }
    stack.push(Value::I64(a.wrapping_div(b)))?;
    Ok(())
}

/// Execute an i64 unsigned division instruction
///
/// Pops two i64 values from the stack, divides the first by the second (unsigned),
/// and pushes the result. Returns an error if dividing by zero.
pub fn i64_div_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_u64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.div_u operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_u64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.div_u operand a, found {}",
            a_val.value_type()
        ))
    })?;
    if b == 0 {
        return Err(Error::division_by_zero());
    }
    stack.push(Value::I64(a.wrapping_div(b) as i64))?;
    Ok(())
}

/// Execute an i64 signed remainder instruction
///
/// Pops two i64 values from the stack, computes the remainder of the first divided
/// by the second (signed), and pushes the result. Returns an error if dividing by zero.
pub fn i64_rem_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.rem_s operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.rem_s operand a, found {}",
            a_val.value_type()
        ))
    })?;
    if b == 0 {
        return Err(Error::division_by_zero());
    }
    stack.push(Value::I64(a.wrapping_rem(b)))?;
    Ok(())
}

/// Execute an i64 unsigned remainder instruction
///
/// Pops two i64 values from the stack, computes the remainder of the first divided
/// by the second (unsigned), and pushes the result. Returns an error if dividing by zero.
pub fn i64_rem_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_u64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.rem_u operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_u64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.rem_u operand a, found {}",
            a_val.value_type()
        ))
    })?;
    if b == 0 {
        return Err(Error::division_by_zero());
    }
    stack.push(Value::I64(a.wrapping_rem(b) as i64))?;
    Ok(())
}

/// Execute an i64 bitwise AND instruction
///
/// Pops two i64 values from the stack, computes their bitwise AND, and pushes the result.
pub fn i64_and(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.and operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.and operand a, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I64(a & b))?;
    Ok(())
}

/// Execute an i64 bitwise OR instruction
///
/// Pops two i64 values from the stack, computes their bitwise OR, and pushes the result.
pub fn i64_or(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.or operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.or operand a, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I64(a | b))?;
    Ok(())
}

/// Execute an i64 bitwise XOR instruction
///
/// Pops two i64 values from the stack, computes their bitwise XOR, and pushes the result.
pub fn i64_xor(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.xor operand b, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.xor operand a, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I64(a ^ b))?;
    Ok(())
}

/// Execute an i64 shift left instruction
///
/// Pops two i64 values from the stack, shifts the first left by the lower 6 bits
/// of the second value, and pushes the result.
pub fn i64_shl(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_u64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.shl shift amount, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.shl value, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I64(a.wrapping_shl((b % 64) as u32)))
}

/// Execute an i64 signed shift right instruction
///
/// Pops two i64 values from the stack, shifts the first right by the lower 6 bits
/// of the second value (signed, preserving sign bit), and pushes the result.
pub fn i64_shr_s(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_u64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.shr_s shift amount, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.shr_s value, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I64(a.wrapping_shr((b % 64) as u32)))
}

/// Execute an i64 unsigned shift right instruction
///
/// Pops two i64 values from the stack, shifts the first right by the lower 6 bits
/// of the second value (unsigned, filling with zeros), and pushes the result.
pub fn i64_shr_u(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_u64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.shr_u shift amount, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_u64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.shr_u value, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I64(a.wrapping_shr((b % 64) as u32) as i64))
}

/// Execute an i64 rotate left instruction
///
/// Pops two i64 values from the stack, rotates the bits of the first left by the
/// lower 6 bits of the second value, and pushes the result.
pub fn i64_rotl(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_u64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.rotl amount, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.rotl value, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I64(a.rotate_left((b % 64) as u32)))
}

/// Execute an i64 rotate right instruction
///
/// Pops two i64 values from the stack, rotates the bits of the first right by the
/// lower 6 bits of the second value, and pushes the result.
pub fn i64_rotr(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b_val = stack.pop()?;
    let b = b_val.as_u64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.rotr amount, found {}",
            b_val.value_type()
        ))
    })?;
    let a_val = stack.pop()?;
    let a = a_val.as_i64().ok_or_else(|| {
        Error::invalid_type(format!(
            "Expected I64 for i64.rotr value, found {}",
            a_val.value_type()
        ))
    })?;
    stack.push(Value::I64(a.rotate_right((b % 64) as u32)))
}

/// Execute an f32 ceiling instruction
///
/// Pops an f32 value from the stack, rounds it up to the nearest integer, and pushes the result.
pub fn f32_ceil(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let a = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    stack.push(Value::F32(a.ceil()))?;
    Ok(())
}

/// Execute an f32 floor instruction
///
/// Pops an f32 value from the stack, rounds it down to the nearest integer, and pushes the result.
pub fn f32_floor(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let a = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    stack.push(Value::F32(a.floor()))?;
    Ok(())
}

/// Execute an f32 truncate instruction
///
/// Pops an f32 value from the stack, truncates it toward zero to the nearest integer, and pushes the result.
pub fn f32_trunc(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let a = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    stack.push(Value::F32(a.trunc()))?;
    Ok(())
}

/// Execute an f32 nearest instruction
///
/// Pops an f32 value from the stack, rounds it to the nearest integer, and pushes the result.
pub fn f32_nearest(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let a = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    // IEEE 754 roundTiesToEven implementation for f32
    let rounded = if a.is_nan() || a.is_infinite() || a == 0.0 {
        a
    } else {
        let floor = a.floor();
        let ceil = a.ceil();
        let diff_floor = (a - floor).abs();
        let diff_ceil = (a - ceil).abs();

        if diff_floor < diff_ceil {
            floor
        } else if diff_ceil < diff_floor {
            ceil
        } else {
            // Tie-breaking: round to nearest even integer
            if floor % 2.0 == 0.0 {
                floor
            } else {
                ceil
            }
        }
    };
    stack.push(Value::F32(rounded))?;
    Ok(())
}

/// Execute an f32 add instruction
///
/// Pops two f32 values from the stack, adds them, and pushes the result.
pub fn f32_add(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    let a = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    stack.push(Value::F32(a + b))?;
    Ok(())
}

/// Execute an f32 subtract instruction
///
/// Pops two f32 values from the stack, subtracts the second from the first, and pushes the result.
pub fn f32_sub(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    let a = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    stack.push(Value::F32(a - b))?;
    Ok(())
}

/// Execute an f32 multiply instruction
///
/// Pops two f32 values from the stack, multiplies them, and pushes the result.
pub fn f32_mul(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    let a = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    stack.push(Value::F32(a * b))?;
    Ok(())
}

/// Execute an f32 divide instruction
///
/// Pops two f32 values from the stack, divides the first by the second, and pushes the result.
pub fn f32_div(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    let a = stack
        .pop()?
        .as_f32()
        .ok_or_else(|| Error::invalid_type("Expected F32".to_string()))?;
    if b == 0.0 {
        return Err(Error::division_by_zero());
    }
    stack.push(Value::F32(a / b))?;
    Ok(())
}

/// Execute an f64 addition instruction
///
/// Pops two f64 values from the stack, adds them, and pushes the result.
pub fn f64_add(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    let a = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    stack.push(Value::F64(a + b))?;
    Ok(())
}

/// Execute an f64 subtraction instruction
///
/// Pops two f64 values from the stack, subtracts the second from the first, and pushes the result.
pub fn f64_sub(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    let a = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    stack.push(Value::F64(a - b))?;
    Ok(())
}

/// Execute an f64 multiplication instruction
///
/// Pops two f64 values from the stack, multiplies them, and pushes the result.
pub fn f64_mul(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    let a = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    stack.push(Value::F64(a * b))?;
    Ok(())
}

/// Execute an f64 division instruction
///
/// Pops two f64 values from the stack, divides the first by the second, and pushes the result.
pub fn f64_div(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    let a = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    if b == 0.0 {
        return Err(Error::division_by_zero());
    }
    stack.push(Value::F64(a / b))?;
    Ok(())
}

/// Execute an f64 maximum instruction
///
/// Pops two f64 values from the stack, pushes the maximum of the two values.
pub fn f64_max(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    let a = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    stack.push(Value::F64(a.max(b)))?;
    Ok(())
}

/// Execute an f64 minimum instruction
///
/// Pops two f64 values from the stack, pushes the minimum of the two values.
pub fn f64_min(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let b = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    let a = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    stack.push(Value::F64(a.min(b)))?;
    Ok(())
}

/// Execute an f64 square root instruction
///
/// Pops an f64 value from the stack, computes its square root, and pushes the result.
pub fn f64_sqrt(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let a = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    stack.push(Value::F64(a.sqrt()))?;
    Ok(())
}

/// Execute an f64 nearest instruction
///
/// Pops an f64 value from the stack, rounds it to the nearest integer, and pushes the result.
/// Round-to-nearest rounds to the nearest integral value; if two integral values are equally near,
/// rounds to the even value (Banker's rounding).
pub fn f64_nearest(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let a = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    // IEEE 754 roundTiesToEven
    let rounded = if a.is_nan() || a.is_infinite() || a == 0.0 {
        a
    } else {
        let floor = a.floor();
        let ceil = a.ceil();
        let diff_floor = (a - floor).abs();
        let diff_ceil = (a - ceil).abs();

        if diff_floor < diff_ceil {
            floor
        } else if diff_ceil < diff_floor {
            ceil
        } else {
            // Tie-breaking: round to nearest even integer
            if floor % 2.0 == 0.0 {
                floor
            } else {
                ceil
            }
        }
    };
    stack.push(Value::F64(rounded))?;
    Ok(())
}

/// Execute an f64 truncate instruction
///
/// Pops an f64 value from the stack, truncates it toward zero to the nearest integer, and pushes the result.
/// Truncation removes the fractional part, rounding toward zero.
pub fn f64_trunc(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let a = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    stack.push(Value::F64(a.trunc()))?;
    Ok(())
}

/// Execute an f64 floor instruction
///
/// Pops an f64 value from the stack, rounds it down to the nearest integer, and pushes the result.
/// Floor rounding always rounds towards negative infinity.
pub fn f64_floor(
    _frame: &mut (impl FrameBehavior + ?Sized),
    stack: &mut (impl StackBehavior + ?Sized),
    _engine: &StacklessEngine,
) -> Result<()> {
    let a = stack
        .pop()?
        .as_f64()
        .ok_or_else(|| Error::invalid_type("Expected F64".to_string()))?;
    stack.push(Value::F64(a.floor()))?;
    Ok(())
}
