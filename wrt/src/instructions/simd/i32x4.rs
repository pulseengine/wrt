//! SIMD i32x4 operations
//!
//! This module contains implementations for WebAssembly SIMD instructions
//! that operate on 128-bit vectors as four 32-bit integer lanes.

#[cfg(feature = "std")]
use std::cmp;

#[cfg(not(feature = "std"))]
use core::cmp;

use crate::behavior::{FrameBehavior, StackBehavior};
use crate::values::{Value, Value::*};
use crate::StacklessEngine;
use wrt_error::{kinds, Error, Result};

use super::get_i16_lane;
use super::{pop_v128, push_v128, V128};

/// Helper to get an i32 value from a lane in a v128
#[inline]
fn get_i32_lane(v: &V128, lane: usize) -> i32 {
    let offset = lane * 4;
    i32::from_le_bytes([v[offset], v[offset + 1], v[offset + 2], v[offset + 3]])
}

/// Helper to get an u32 value from a lane in a v128
#[inline]
fn get_u32_lane(v: &V128, lane: usize) -> u32 {
    let offset = lane * 4;
    u32::from_le_bytes([v[offset], v[offset + 1], v[offset + 2], v[offset + 3]])
}

/// Helper to set an i32 value to a lane in a v128
#[inline]
pub fn set_i32_lane(v: &mut V128, lane: usize, value: i32) {
    let offset = lane * 4;
    let bytes = value.to_le_bytes();
    v[offset] = bytes[0];
    v[offset + 1] = bytes[1];
    v[offset + 2] = bytes[2];
    v[offset + 3] = bytes[3];
}

/// Replicate an i32 value to all lanes of a v128
pub fn i32x4_splat(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    // Pop the i32 value from the stack
    let value = match stack.pop()? {
        Value::I32(v) => v,
        _ => {
            return Err(Error::new(kinds::InvalidTypeError(
                "Expected i32 for i32x4.splat".into(),
            )))
        }
    };

    // Create the result v128 with the i32 value replicated to all 4 lanes
    let mut result = [0u8; 16];
    let bytes = value.to_le_bytes();

    // Replicate the 4 bytes to each of the 4 lanes
    for i in 0..4 {
        let offset = i * 4;
        result[offset] = bytes[0];
        result[offset + 1] = bytes[1];
        result[offset + 2] = bytes[2];
        result[offset + 3] = bytes[3];
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Extract a lane as a signed i32 from a v128
pub fn i32x4_extract_lane(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
    lane: u32,
) -> Result<(), Error> {
    // Ensure lane index is in range
    if lane >= 4 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane
        ))));
    }

    // Pop the v128 value from the stack
    let v128 = pop_v128(stack)?;

    // Extract the i32 value from the specified lane
    let value = get_i32_lane(&v128, lane as usize);

    // Push the extracted i32 value to the stack
    stack.push(Value::I32(value))?;
    Ok(())
}

/// Replace a lane in a v128 with an i32 value
pub fn i32x4_replace_lane(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
    lane: u32,
) -> Result<(), Error> {
    // Ensure lane index is in range
    if lane >= 4 {
        return Err(Error::new(kinds::ValidationError(
            "Invalid lane index: 4".to_string(),
        )));
    }

    // Pop the replacement i32 value from the stack
    let replacement = match stack.pop()? {
        Value::I32(v) => v,
        _ => {
            return Err(Error::new(kinds::InvalidTypeError(
                "Expected i32 for replacement value".into(),
            )))
        }
    };

    // Pop the v128 value from the stack
    let mut v128 = pop_v128(stack)?;

    // Replace the specified lane with the i32 value
    set_i32_lane(&mut v128, lane as usize, replacement);

    // Push the modified v128 value back to the stack
    push_v128(stack, v128)?;
    Ok(())
}

/// Add corresponding lanes of two v128 values
pub fn i32x4_add(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Add corresponding i32 lanes
    let mut result = [0u8; 16];

    for i in 0..4 {
        let a_val = get_i32_lane(&a, i);
        let b_val = get_i32_lane(&b, i);
        let sum = a_val.wrapping_add(b_val);
        set_i32_lane(&mut result, i, sum);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Subtract corresponding lanes of two v128 values
pub fn i32x4_sub(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Subtract corresponding i32 lanes
    let mut result = [0u8; 16];

    for i in 0..4 {
        let a_val = get_i32_lane(&a, i);
        let b_val = get_i32_lane(&b, i);
        let diff = a_val.wrapping_sub(b_val);
        set_i32_lane(&mut result, i, diff);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Multiply corresponding lanes of two v128 values
pub fn i32x4_mul(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Multiply corresponding i32 lanes
    let mut result = [0u8; 16];

    for i in 0..4 {
        let a_val = get_i32_lane(&a, i);
        let b_val = get_i32_lane(&b, i);
        let product = a_val.wrapping_mul(b_val);
        set_i32_lane(&mut result, i, product);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Negate each lane of a v128 value
pub fn i32x4_neg(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    // Pop the v128 value from the stack
    let v128 = pop_v128(stack)?;

    // Negate each i32 lane
    let mut result = [0u8; 16];

    for i in 0..4 {
        let val = get_i32_lane(&v128, i);
        let neg = val.wrapping_neg();
        set_i32_lane(&mut result, i, neg);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Compare lanes for equality
pub fn i32x4_eq(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Compare corresponding i32 lanes for equality
    let mut result = [0u8; 16];

    for i in 0..4 {
        let a_val = get_i32_lane(&a, i);
        let b_val = get_i32_lane(&b, i);
        let mask = if a_val == b_val {
            0xFFFFFFFFu32 as i32
        } else {
            0
        };
        set_i32_lane(&mut result, i, mask);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Compare lanes for inequality
pub fn i32x4_ne(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Compare corresponding i32 lanes for inequality
    let mut result = [0u8; 16];

    for i in 0..4 {
        let a_val = get_i32_lane(&a, i);
        let b_val = get_i32_lane(&b, i);
        let mask = if a_val != b_val {
            0xFFFFFFFFu32 as i32
        } else {
            0
        };
        set_i32_lane(&mut result, i, mask);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Compare lanes for less than (signed)
pub fn i32x4_lt_s(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Compare corresponding i32 lanes for less than (signed)
    let mut result = [0u8; 16];

    for i in 0..4 {
        let a_val = get_i32_lane(&a, i);
        let b_val = get_i32_lane(&b, i);
        let mask = if a_val < b_val {
            0xFFFFFFFFu32 as i32
        } else {
            0
        };
        set_i32_lane(&mut result, i, mask);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Compare lanes for less than (unsigned)
pub fn i32x4_lt_u(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Compare corresponding u32 lanes for less than (unsigned)
    let mut result = [0u8; 16];

    for i in 0..4 {
        let a_val = get_u32_lane(&a, i);
        let b_val = get_u32_lane(&b, i);
        let mask = if a_val < b_val {
            0xFFFFFFFFu32 as i32
        } else {
            0
        };
        set_i32_lane(&mut result, i, mask);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Find minimum of corresponding lanes (signed)
pub fn i32x4_min_s(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Find minimum of corresponding i32 lanes (signed)
    let mut result = [0u8; 16];

    for i in 0..4 {
        let a_val = get_i32_lane(&a, i);
        let b_val = get_i32_lane(&b, i);
        let min = cmp::min(a_val, b_val);
        set_i32_lane(&mut result, i, min);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Find minimum of corresponding lanes (unsigned)
pub fn i32x4_min_u(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Find minimum of corresponding u32 lanes (unsigned)
    let mut result = [0u8; 16];

    for i in 0..4 {
        let a_val = get_u32_lane(&a, i);
        let b_val = get_u32_lane(&b, i);
        let min = cmp::min(a_val, b_val);
        let min_bytes = min.to_le_bytes();
        let offset = i * 4;
        result[offset] = min_bytes[0];
        result[offset + 1] = min_bytes[1];
        result[offset + 2] = min_bytes[2];
        result[offset + 3] = min_bytes[3];
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Find maximum of corresponding lanes (signed)
pub fn i32x4_max_s(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Find maximum of corresponding i32 lanes (signed)
    let mut result = [0u8; 16];

    for i in 0..4 {
        let a_val = get_i32_lane(&a, i);
        let b_val = get_i32_lane(&b, i);
        let max = cmp::max(a_val, b_val);
        set_i32_lane(&mut result, i, max);
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Find maximum of corresponding lanes (unsigned)
pub fn i32x4_max_u(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    // Pop two v128 values from the stack
    let b = pop_v128(stack)?;
    let a = pop_v128(stack)?;

    // Find maximum of corresponding u32 lanes (unsigned)
    let mut result = [0u8; 16];

    for i in 0..4 {
        let a_val = get_u32_lane(&a, i);
        let b_val = get_u32_lane(&b, i);
        let max = cmp::max(a_val, b_val);
        let max_bytes = max.to_le_bytes();
        let offset = i * 4;
        result[offset] = max_bytes[0];
        result[offset + 1] = max_bytes[1];
        result[offset + 2] = max_bytes[2];
        result[offset + 3] = max_bytes[3];
    }

    // Push the result v128 to the stack
    push_v128(stack, result)?;
    Ok(())
}

/// Implementation for i32x4.extadd_pairwise_i16x8_s (Opcode: 0xFD 0x7E)
pub fn i32x4_extadd_pairwise_i16x8_s(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    let v = pop_v128(stack)?;
    let mut result = [0u8; 16];

    for i in 0..4 {
        // Calculate pairwise sum for lane i
        // Pair 0: lanes 0, 1 -> result lane 0
        // Pair 1: lanes 2, 3 -> result lane 1
        // Pair 2: lanes 4, 5 -> result lane 2
        // Pair 3: lanes 6, 7 -> result lane 3
        let lane1_idx = i * 2;
        let lane2_idx = i * 2 + 1;

        let val1 = get_i16_lane(&v, lane1_idx) as i32;
        let val2 = get_i16_lane(&v, lane2_idx) as i32;
        let sum = val1.wrapping_add(val2); // Signed addition

        set_i32_lane(&mut result, i, sum);
    }

    push_v128(stack, result)?;
    Ok(())
}

/// Implementation for i32x4.extadd_pairwise_i16x8_u (Opcode: 0xFD 0x7F)
pub fn i32x4_extadd_pairwise_i16x8_u(
    stack: &mut impl StackBehavior,
    _frame: &mut impl FrameBehavior,
) -> Result<(), Error> {
    let v = pop_v128(stack)?;
    let mut result = [0u8; 16];

    for i in 0..4 {
        // Calculate pairwise sum for lane i (unsigned)
        let lane1_idx = i * 2;
        let lane2_idx = i * 2 + 1;

        // Treat i16 lanes as u16, extend to u32, then add
        let val1 = (get_i16_lane(&v, lane1_idx) as u16) as u32;
        let val2 = (get_i16_lane(&v, lane2_idx) as u16) as u32;
        let sum = val1.wrapping_add(val2);

        // Cast the u32 sum to i32 for storage (result is i32x4)
        set_i32_lane(&mut result, i, sum as i32);
    }

    push_v128(stack, result)?;
    Ok(())
}

// Additional i32x4 operations will be implemented here as needed

// Add stubs for missing functions

pub fn i32x4_shr_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement i32x4_shr_s")
}

pub fn i32x4_shr_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement i32x4_shr_u")
}

pub fn i32x4_dot_i16x8_s(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement i32x4_dot_i16x8_s")
}

pub fn i32x4_extmul_low_i16x8_s(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement i32x4_extmul_low_i16x8_s")
}

pub fn i32x4_extmul_high_i16x8_s(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement i32x4_extmul_high_i16x8_s")
}

pub fn i32x4_extmul_low_i16x8_u(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement i32x4_extmul_low_i16x8_u")
}

pub fn i32x4_extmul_high_i16x8_u(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement i32x4_extmul_high_i16x8_u")
}

pub fn i32x4_trunc_sat_f32x4_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement i32x4_trunc_sat_f32x4_s")
}

pub fn i32x4_trunc_sat_f32x4_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement i32x4_trunc_sat_f32x4_u")
}

pub fn i32x4_trunc_sat_f64x2_s_zero(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement i32x4_trunc_sat_f64x2_s_zero")
}

pub fn i32x4_trunc_sat_f64x2_u_zero(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement i32x4_trunc_sat_f64x2_u_zero")
}

pub fn i32x4_extend_low_i16x8_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement i32x4_extend_low_i16x8_s")
}

pub fn i32x4_extend_high_i16x8_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement i32x4_extend_high_i16x8_s")
}

pub fn i32x4_extend_low_i16x8_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement i32x4_extend_low_i16x8_u")
}

pub fn i32x4_extend_high_i16x8_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<(), Error> {
    todo!("Implement i32x4_extend_high_i16x8_u")
}

pub fn i32x4_all_true(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement i32x4_all_true")
}

pub fn i32x4_bitmask(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement i32x4_bitmask")
}

pub fn i32x4_abs(
    stack: &mut impl StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<(), Error> {
    todo!("Implement i32x4_abs")
}
