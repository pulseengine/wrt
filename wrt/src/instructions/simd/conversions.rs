//! SIMD conversion operations
//!
//! This module contains implementations for WebAssembly SIMD instructions
//! that perform type conversions between different numeric formats.

use crate::{
    behavior::FrameBehavior,
    error::{Error, Result},
    stack::Stack,
    values::Value,
};

use super::common::{pop_v128, push_v128, V128};

/// Truncate each lane in f32x4 to i32x4 with signed saturation
pub fn i32x4_trunc_sat_f32x4_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    // Pop the v128 value containing 4 f32 values
    let v128 = pop_v128(stack)?;
    
    // Convert each f32 lane to i32 with signed saturation
    let mut result = [0u8; 16];
    
    for i in 0..4 {
        let offset = i * 4;
        let f32_val = f32::from_le_bytes([
            v128[offset],
            v128[offset + 1],
            v128[offset + 2],
            v128[offset + 3],
        ]);
        
        // Convert f32 to i32 with saturation
        let i32_val = if f32_val.is_nan() {
            0 // NaN converts to 0
        } else if f32_val <= i32::MIN as f32 {
            i32::MIN // Saturate to minimum i32 value
        } else if f32_val >= i32::MAX as f32 {
            i32::MAX // Saturate to maximum i32 value
        } else {
            f32_val as i32 // Normal conversion
        };
        
        // Store the i32 value as bytes in the result
        let bytes = i32_val.to_le_bytes();
        result[offset] = bytes[0];
        result[offset + 1] = bytes[1];
        result[offset + 2] = bytes[2];
        result[offset + 3] = bytes[3];
    }
    
    // Push the result v128 to the stack
    push_v128(stack, result)
}

/// Truncate each lane in f32x4 to i32x4 with unsigned saturation
pub fn i32x4_trunc_sat_f32x4_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    // Pop the v128 value containing 4 f32 values
    let v128 = pop_v128(stack)?;
    
    // Convert each f32 lane to u32 with unsigned saturation, then store as i32
    let mut result = [0u8; 16];
    
    for i in 0..4 {
        let offset = i * 4;
        let f32_val = f32::from_le_bytes([
            v128[offset],
            v128[offset + 1],
            v128[offset + 2],
            v128[offset + 3],
        ]);
        
        // Convert f32 to u32 with saturation
        let u32_val = if f32_val.is_nan() || f32_val <= 0.0 {
            0 // NaN or negative converts to 0
        } else if f32_val >= u32::MAX as f32 {
            u32::MAX // Saturate to maximum u32 value
        } else {
            f32_val as u32 // Normal conversion
        };
        
        // Store the u32 value as bytes in the result (reinterpreted as i32)
        let bytes = u32_val.to_le_bytes();
        result[offset] = bytes[0];
        result[offset + 1] = bytes[1];
        result[offset + 2] = bytes[2];
        result[offset + 3] = bytes[3];
    }
    
    // Push the result v128 to the stack
    push_v128(stack, result)
}

/// Convert each lane in i32x4 to f32x4 with signed interpretation
pub fn f32x4_convert_i32x4_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    // Pop the v128 value containing 4 i32 values
    let v128 = pop_v128(stack)?;
    
    // Convert each i32 lane to f32
    let mut result = [0u8; 16];
    
    for i in 0..4 {
        let offset = i * 4;
        let i32_val = i32::from_le_bytes([
            v128[offset],
            v128[offset + 1],
            v128[offset + 2],
            v128[offset + 3],
        ]);
        
        // Convert i32 to f32
        let f32_val = i32_val as f32;
        
        // Store the f32 value as bytes in the result
        let bytes = f32_val.to_le_bytes();
        result[offset] = bytes[0];
        result[offset + 1] = bytes[1];
        result[offset + 2] = bytes[2];
        result[offset + 3] = bytes[3];
    }
    
    // Push the result v128 to the stack
    push_v128(stack, result)
}

/// Convert each lane in i32x4 to f32x4 with unsigned interpretation
pub fn f32x4_convert_i32x4_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    // Pop the v128 value containing 4 i32 values (interpreted as u32)
    let v128 = pop_v128(stack)?;
    
    // Convert each u32 lane to f32
    let mut result = [0u8; 16];
    
    for i in 0..4 {
        let offset = i * 4;
        let u32_val = u32::from_le_bytes([
            v128[offset],
            v128[offset + 1],
            v128[offset + 2],
            v128[offset + 3],
        ]);
        
        // Convert u32 to f32
        let f32_val = u32_val as f32;
        
        // Store the f32 value as bytes in the result
        let bytes = f32_val.to_le_bytes();
        result[offset] = bytes[0];
        result[offset + 1] = bytes[1];
        result[offset + 2] = bytes[2];
        result[offset + 3] = bytes[3];
    }
    
    // Push the result v128 to the stack
    push_v128(stack, result)
} 