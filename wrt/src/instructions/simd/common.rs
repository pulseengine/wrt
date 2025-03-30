//! Common utilities for SIMD operations
//!
//! This module contains shared utilities and functions for SIMD operations.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::{
    error::{Error, Result},
    stack::Stack,
    values::Value,
};

/// A v128 value is represented as 16 bytes (128 bits)
pub type V128 = [u8; 16];

/// Pops a v128 value from the stack
#[inline]
pub fn pop_v128(stack: &mut (impl Stack + ?Sized)) -> Result<V128> {
    match stack.pop()? {
        Value::V128(bytes) => Ok(bytes),
        _ => Err(Error::InvalidType("Expected v128 value".into())),
    }
}

/// Pushes a v128 value onto the stack
#[inline]
pub fn push_v128(stack: &mut (impl Stack + ?Sized), bytes: V128) -> Result<()> {
    stack.push(Value::V128(bytes))
}

/// Load a 128-bit value from memory into a v128 value
pub fn v128_load(stack: &mut (impl Stack + ?Sized), offset: u32, _align: u32) -> Result<()> {
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as u32,
        _ => return Err(Error::InvalidType("Expected i32 for memory address".into())),
    };

    // Calculate the effective address
    let effective_addr = addr.wrapping_add(offset) as usize;

    // This function would normally load 16 bytes from memory
    // But since we don't have access to the memory directly here,
    // we'll return an error that this needs to be implemented in the caller
    Err(Error::Unimplemented(
        "v128_load operations should be handled by the executor".to_string(),
    ))
}

/// Store a 128-bit value from a v128 value to memory
pub fn v128_store(stack: &mut (impl Stack + ?Sized), offset: u32, _align: u32) -> Result<()> {
    let bytes = match stack.pop()? {
        Value::V128(bytes) => bytes,
        _ => return Err(Error::InvalidType("Expected v128 for store".into())),
    };

    let addr = match stack.pop()? {
        Value::I32(addr) => addr as u32,
        _ => return Err(Error::InvalidType("Expected i32 for memory address".into())),
    };

    // Calculate the effective address
    let effective_addr = addr.wrapping_add(offset) as usize;

    // This function would normally store 16 bytes to memory
    // But since we don't have access to the memory directly here,
    // we'll return an error that this needs to be implemented in the caller
    Err(Error::Unimplemented(
        "v128_store operations should be handled by the executor".to_string(),
    ))
}

/// Push a constant v128 value onto the stack
pub fn v128_const(stack: &mut (impl Stack + ?Sized), bytes: V128) -> Result<()> {
    stack.push(Value::V128(bytes))
}

/// Shuffle 16 bytes from two v128 operands into a new v128 value
pub fn i8x16_shuffle(stack: &mut (impl Stack + ?Sized), lanes: &[u8; 16]) -> Result<()> {
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    let mut result = [0u8; 16];
    for i in 0..16 {
        let lane_idx = lanes[i] as usize;
        result[i] = if lane_idx < 16 {
            a[lane_idx]
        } else if lane_idx < 32 {
            b[lane_idx - 16]
        } else {
            0 // Out of range index results in 0
        };
    }

    push_v128(stack, result)
}

/// Swizzle 16 bytes from two v128 operands into a new v128 value
pub fn i8x16_swizzle(stack: &mut (impl Stack + ?Sized)) -> Result<()> {
    let indices = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    let mut result = [0u8; 16];
    for i in 0..16 {
        let idx = indices[i] as usize;
        result[i] = if idx < 16 {
            a[idx]
        } else {
            0 // Out of range index results in 0
        };
    }

    push_v128(stack, result)
}
