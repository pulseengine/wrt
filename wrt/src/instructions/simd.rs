//! SIMD (Single Instruction Multiple Data) instructions
//!
//! This module contains implementations for WebAssembly SIMD (vector) instructions,
//! which operate on 128-bit vectors containing different types of packed data.

use crate::Vec;
use crate::{
    error::{Error, Result},
    format,
    memory::Memory,
    values::Value,
};

/// Load a 128-bit value from memory into a v128 value
///
/// # Arguments
///
/// * `stack` - The operand stack
/// * `memory` - The memory instance to load from
/// * `offset` - The static offset to add to the memory address
/// * `align` - The expected alignment (not used, but included for consistency)
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the operation succeeded, or an Error
pub fn v128_load(values: &mut Vec<Value>, memory: &Memory, offset: u32, align: u8) -> Result<()> {
    // We handle the addr popping and all memory access in execution.rs
    // This function is a placeholder for now
    Ok(())
}

/// Store a 128-bit value from a v128 value into memory
///
/// # Arguments
///
/// * `stack` - The operand stack
/// * `memory` - The memory instance to store to
/// * `offset` - The static offset to add to the memory address
/// * `align` - The expected alignment (not used, but included for consistency)
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the operation succeeded, or an Error
pub fn v128_store(
    values: &mut Vec<Value>,
    memory: &mut Memory,
    offset: u32,
    align: u8,
) -> Result<()> {
    // We handle the addr and value popping and all memory access in execution.rs
    // This function is a placeholder for now
    Ok(())
}

/// Create a v128 constant value with the given bytes
///
/// # Arguments
///
/// * `stack` - The operand stack
/// * `bytes` - The 16 bytes to use for the v128 constant
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the operation succeeded, or an Error
pub fn v128_const(values: &mut Vec<Value>, bytes: [u8; 16]) -> Result<()> {
    // Convert bytes to u128 and push as V128 value
    let v128_val = u128::from_le_bytes(bytes);
    values.push(Value::V128(v128_val));
    Ok(())
}

/// Implements the i8x16.splat operation, which creates a vector with all lanes equal to a single i8 value
///
/// # Arguments
///
/// * `stack` - The operand stack
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the operation succeeded, or an Error
pub fn i8x16_splat(values: &mut Vec<Value>) -> Result<()> {
    let value = values.pop().ok_or(Error::StackUnderflow)?;
    let Value::I32(x) = value else {
        return Err(Error::Execution("Expected i32 for i8x16.splat".into()));
    };

    // Use only the bottom 8 bits
    let byte = (x & 0xFF) as u8;

    // Replicate the byte 16 times
    let bytes = [byte; 16];

    // Convert to u128
    let v128_value = u128::from_le_bytes(bytes);

    // Push result
    values.push(Value::V128(v128_value));
    Ok(())
}

/// Implements the i16x8.splat operation, which creates a vector with all lanes equal to a single i16 value
///
/// # Arguments
///
/// * `stack` - The operand stack
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the operation succeeded, or an Error
pub fn i16x8_splat(values: &mut Vec<Value>) -> Result<()> {
    let value = values.pop().ok_or(Error::StackUnderflow)?;
    let Value::I32(x) = value else {
        return Err(Error::Execution("Expected i32 for i16x8.splat".into()));
    };

    // Use only the bottom 16 bits
    let short = (x & 0xFFFF) as u16;

    // Create array of 8 u16s
    let shorts = [short; 8];

    // Convert to bytes
    let mut bytes = [0u8; 16];
    for i in 0..8 {
        let short_bytes = shorts[i].to_le_bytes();
        bytes[i * 2] = short_bytes[0];
        bytes[i * 2 + 1] = short_bytes[1];
    }

    // Convert to u128
    let v128_value = u128::from_le_bytes(bytes);

    // Push result
    values.push(Value::V128(v128_value));
    Ok(())
}

/// Implements the i32x4.splat operation, which creates a vector with all lanes equal to a single i32 value
///
/// # Arguments
///
/// * `stack` - The operand stack
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the operation succeeded, or an Error
pub fn i32x4_splat(values: &mut Vec<Value>) -> Result<()> {
    let value = values.pop().ok_or(Error::StackUnderflow)?;
    let Value::I32(x) = value else {
        return Err(Error::Execution("Expected i32 for i32x4.splat".into()));
    };

    // Create array of 4 i32s
    let ints = [x; 4];

    // Convert to bytes
    let mut bytes = [0u8; 16];
    for i in 0..4 {
        let int_bytes = ints[i].to_le_bytes();
        bytes[i * 4..i * 4 + 4].copy_from_slice(&int_bytes);
    }

    // Convert to u128
    let v128_value = u128::from_le_bytes(bytes);

    // Push result
    values.push(Value::V128(v128_value));
    Ok(())
}

/// Implements the i64x2.splat operation, which creates a vector with all lanes equal to a single i64 value
///
/// # Arguments
///
/// * `stack` - The operand stack
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the operation succeeded, or an Error
pub fn i64x2_splat(values: &mut Vec<Value>) -> Result<()> {
    let value = values.pop().ok_or(Error::StackUnderflow)?;
    let Value::I64(x) = value else {
        return Err(Error::Execution("Expected i64 for i64x2.splat".into()));
    };

    // Create array of 2 i64s
    let longs = [x; 2];

    // Convert to bytes
    let mut bytes = [0u8; 16];
    for i in 0..2 {
        let long_bytes = longs[i].to_le_bytes();
        bytes[i * 8..i * 8 + 8].copy_from_slice(&long_bytes);
    }

    // Convert to u128
    let v128_value = u128::from_le_bytes(bytes);

    // Push result
    values.push(Value::V128(v128_value));
    Ok(())
}

/// Implements the f32x4.splat operation, which creates a vector with all lanes equal to a single f32 value
///
/// # Arguments
///
/// * `stack` - The operand stack
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the operation succeeded, or an Error
pub fn f32x4_splat(values: &mut Vec<Value>) -> Result<()> {
    let value = values.pop().ok_or(Error::StackUnderflow)?;
    let Value::F32(x) = value else {
        return Err(Error::Execution("Expected f32 for f32x4.splat".into()));
    };

    // Create array of 4 f32s
    let floats = [x; 4];

    // Convert to bytes
    let mut bytes = [0u8; 16];
    for i in 0..4 {
        let float_bytes = floats[i].to_le_bytes();
        bytes[i * 4..i * 4 + 4].copy_from_slice(&float_bytes);
    }

    // Convert to u128
    let v128_value = u128::from_le_bytes(bytes);

    // Push result
    values.push(Value::V128(v128_value));
    Ok(())
}

/// Implements the f64x2.splat operation, which creates a vector with all lanes equal to a single f64 value
///
/// # Arguments
///
/// * `stack` - The operand stack
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the operation succeeded, or an Error
pub fn f64x2_splat(values: &mut Vec<Value>) -> Result<()> {
    let value = values.pop().ok_or(Error::StackUnderflow)?;
    let Value::F64(x) = value else {
        return Err(Error::Execution("Expected f64 for f64x2.splat".into()));
    };

    // Create array of 2 f64s
    let doubles = [x; 2];

    // Convert to bytes
    let mut bytes = [0u8; 16];
    for i in 0..2 {
        let double_bytes = doubles[i].to_le_bytes();
        bytes[i * 8..i * 8 + 8].copy_from_slice(&double_bytes);
    }

    // Convert to u128
    let v128_value = u128::from_le_bytes(bytes);

    // Push result
    values.push(Value::V128(v128_value));
    Ok(())
}

// Lane extraction operations

/// Extract a signed 8-bit integer from a lane of a v128 value
///
/// # Arguments
///
/// * `stack` - The operand stack
/// * `lane_idx` - The lane index to extract (0-15)
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the operation succeeded, or an Error
pub fn i8x16_extract_lane_s(values: &mut Vec<Value>, lane_idx: u8) -> Result<()> {
    if lane_idx >= 16 {
        return Err(Error::Execution(format!(
            "Lane index out of bounds: {}",
            lane_idx
        )));
    }

    let value = values.pop().ok_or(Error::StackUnderflow)?;
    let Value::V128(x) = value else {
        return Err(Error::Execution(
            "Expected v128 for i8x16.extract_lane_s".into(),
        ));
    };

    // Convert to bytes
    let bytes = x.to_le_bytes();

    // Extract the byte at lane_idx
    let byte = bytes[lane_idx as usize];

    // Sign-extend to i32
    let result = (byte as i8) as i32;

    // Push result
    values.push(Value::I32(result));
    Ok(())
}

/// Extract an unsigned 8-bit integer from a lane of a v128 value
///
/// # Arguments
///
/// * `stack` - The operand stack
/// * `lane_idx` - The lane index to extract (0-15)
///
/// # Returns
///
/// * `Result<(), Error>` - Ok if the operation succeeded, or an Error
pub fn i8x16_extract_lane_u(values: &mut Vec<Value>, lane_idx: u8) -> Result<()> {
    if lane_idx >= 16 {
        return Err(Error::Execution(format!(
            "Lane index out of bounds: {}",
            lane_idx
        )));
    }

    let value = values.pop().ok_or(Error::StackUnderflow)?;
    let Value::V128(x) = value else {
        return Err(Error::Execution(
            "Expected v128 for i8x16.extract_lane_u".into(),
        ));
    };

    // Convert to bytes
    let bytes = x.to_le_bytes();

    // Extract the byte at lane_idx
    let byte = bytes[lane_idx as usize];

    // Zero-extend to i32
    let result = byte as i32;

    // Push result
    values.push(Value::I32(result));
    Ok(())
}

// Add more SIMD operations as needed...
