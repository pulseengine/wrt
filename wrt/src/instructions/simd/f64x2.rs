//! SIMD f64x2 operations
//!
//! This module contains implementations for WebAssembly SIMD instructions
//! that operate on 128-bit vectors as two 64-bit floating point lanes.

use crate::behavior::{FrameBehavior, StackBehavior};
use crate::prelude::TypesValue as Value;
use crate::StacklessEngine;
use wrt_error::{kinds, Error, Result};

use super::{pop_v128, push_v128, V128};

/// Helper to get an f64 value from a lane in a v128
#[inline]
const fn get_f64_lane(v: &V128, lane: usize) -> f64 {
    let offset = lane * 8;
    f64::from_le_bytes([
        v[offset],
        v[offset + 1],
        v[offset + 2],
        v[offset + 3],
        v[offset + 4],
        v[offset + 5],
        v[offset + 6],
        v[offset + 7],
    ])
}

/// Helper to set an f64 value to a lane in a v128
#[inline]
fn set_f64_lane(v: &mut V128, lane: usize, value: f64) {
    let offset = lane * 8;
    let bytes = value.to_le_bytes();
    for i in 0..8 {
        v[offset + i] = bytes[i];
    }
}

/// Replicate an f64 value to all lanes of a v128
pub fn f64x2_splat(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    // Pop the f64 value from the stack
    let value = match stack.pop()? {
        Value::F64(v) => v,
        _ => {
            return Err(Error::new(crate::error::kinds::InvalidTypeError(
                "Expected f64 for f64x2.splat".into(),
            )))
        }
    };

    // Create the result v128 with the f64 value replicated to both lanes
    let mut result = [0u8; 16];
    let bytes = value.to_le_bytes();

    // Replicate the 8 bytes to each of the 2 lanes
    for lane in 0..2 {
        let offset = lane * 8;
        for i in 0..8 {
            result[offset + i] = bytes[i];
        }
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Extract a lane as an f64 from a v128
pub fn f64x2_extract_lane(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
    lane: u32,
) -> Result<(), Error> {
    // Ensure lane index is in range
    if lane >= 2 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane
        ))));
    }

    // Pop the v128 value from the stack
    let v128 = pop_v128(stack)?;

    // Extract the f64 value from the specified lane
    let value = get_f64_lane(&v128, lane as usize);

    // Push the extracted f64 value to the stack
    stack.push(Value::F64(value))?;
    Ok(())
}

/// Replace a lane in a v128 with an f64 value
pub fn f64x2_replace_lane(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
    lane: u32,
) -> Result<(), Error> {
    // Ensure lane index is in range
    if lane >= 2 {
        return Err(Error::new(kinds::ValidationError(
            "Invalid lane index: 2".to_string(),
        )));
    }

    // Pop the replacement f64 value from the stack
    let replacement = match stack.pop()? {
        Value::F64(v) => v,
        _ => {
            return Err(Error::new(crate::error::kinds::InvalidTypeError(
                "Expected f64 for replacement value".into(),
            )))
        }
    };

    // Pop the v128 value from the stack
    let mut v128 = pop_v128(stack)?;

    // Replace the specified lane with the f64 value
    set_f64_lane(&mut v128, lane as usize, replacement);

    // Push the modified v128 value back to the stack
    push_v128(stack, v128)?;
    Ok(())
}

/// Add corresponding lanes of two v128 values
pub fn f64x2_add(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Add corresponding f64 lanes
    let mut result = [0u8; 16];

    for i in 0..2 {
        let a_val = get_f64_lane(&a, i);
        let b_val = get_f64_lane(&b, i);
        let sum = a_val + b_val;
        set_f64_lane(&mut result, i, sum);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Subtract corresponding lanes of two v128 values
pub fn f64x2_sub(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Subtract corresponding f64 lanes
    let mut result = [0u8; 16];

    for i in 0..2 {
        let a_val = get_f64_lane(&a, i);
        let b_val = get_f64_lane(&b, i);
        let diff = a_val - b_val;
        set_f64_lane(&mut result, i, diff);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Multiply corresponding lanes of two v128 values
pub fn f64x2_mul(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Multiply corresponding f64 lanes
    let mut result = [0u8; 16];

    for i in 0..2 {
        let a_val = get_f64_lane(&a, i);
        let b_val = get_f64_lane(&b, i);
        let product = a_val * b_val;
        set_f64_lane(&mut result, i, product);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Divide corresponding lanes of two v128 values
pub fn f64x2_div(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Divide corresponding f64 lanes
    let mut result = [0u8; 16];

    for i in 0..2 {
        let a_val = get_f64_lane(&a, i);
        let b_val = get_f64_lane(&b, i);
        let quotient = a_val / b_val;
        set_f64_lane(&mut result, i, quotient);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Negate each lane of a v128 value
pub fn f64x2_neg(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    // Pop the v128 value from the stack
    let v128 = pop_v128(stack)?;

    // Negate each f64 lane
    let mut result = [0u8; 16];

    for i in 0..2 {
        let val = get_f64_lane(&v128, i);
        let neg = -val;
        set_f64_lane(&mut result, i, neg);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

// Add stubs for missing functions

pub fn f64x2_min(
    _stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement f64x2_min")
}

pub fn f64x2_max(
    _stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement f64x2_max")
}

pub fn f64x2_pmin(
    _stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement f64x2_pmin")
}

pub fn f64x2_pmax(
    _stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement f64x2_pmax")
}

pub fn f64x2_convert_low_i32x4_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement f64x2_convert_low_i32x4_s")
}

pub fn f64x2_convert_low_i32x4_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement f64x2_convert_low_i32x4_u")
}

pub fn f64x2_promote_low_f32x4(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement f64x2_promote_low_f32x4")
}
