//! SIMD instruction implementations

// Add submodule declarations and re-exports
mod f32x4;
pub use f32x4::*;
mod f64x2;
pub use f64x2::*;
mod i8x16;
pub use i8x16::*;
mod i16x8;
pub use i16x8::*;
mod i32x4;
pub use i32x4::*;
mod i64x2;
pub use i64x2::*;

use crate::execution::ExecutionContext;
use crate::format;
use crate::{
    behavior::{ControlFlow, InstructionExecutor},
    error::{kinds, Error, Result},
    values::v128,
};
use crate::{
    behavior::{ControlFlowBehavior, FrameBehavior, StackBehavior},
    values::Value,
    StacklessEngine, // Import
};
#[cfg(not(feature = "std"))]
use alloc::vec;
use std::borrow::BorrowMut; // Added import
use std::ops::Neg; // Added import for Neg trait
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Sub};
use wasmparser::MemArg; // Removed ValType import

// Define V128 type alias and helper functions
pub type V128 = [u8; 16];

#[inline]
fn pop_v128(stack: &mut dyn StackBehavior) -> Result<V128> {
    match stack.pop()? {
        Value::V128(v) => Ok(v),
        other => Err(Error::new(kinds::ValidationError(format!(
            "Expected V128 on stack, found {}",
            other.value_type()
        )))),
    }
}

#[inline]
fn push_v128(stack: &mut dyn StackBehavior, val: V128) -> Result<()> {
    stack.push(Value::V128(val))
}

pub fn v128_const(
    stack: &mut dyn StackBehavior,
    value: [u8; 16], // Changed to directly accept bytes
) -> Result<()> {
    stack.push(Value::V128(value))?; // V128 takes [u8; 16]
    Ok(())
}

pub fn i8x16_shuffle(
    _frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    indices: &[u8; 16],
) -> Result<()> {
    let b = stack.pop()?.as_v128()?;
    let a = stack.pop()?.as_v128()?;
    let mut result = [0u8; 16];
    for i in 0..16 {
        let lane_idx = indices[i];
        result[i] = if lane_idx < 16 {
            a[lane_idx as usize]
        } else if lane_idx < 32 {
            b[(lane_idx - 16) as usize]
        } else {
            // Indices >= 32 are invalid according to spec, result is implementation defined.
            // Let's return 0 for simplicity/determinism.
            0
        };
    }
    stack.push(Value::V128(result))
}

macro_rules! i8x16_extract_lane {
    ($name:ident, $signedness:ident) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
            lane_idx: u8,
        ) -> Result<()> {
            let v = stack.pop()?.as_v128()?;
            let val = v[lane_idx as usize];
            let result = if stringify!($signedness) == "signed" {
                (val as i8) as i32
            } else {
                val as i32
            };
            stack.push(Value::I32(result))
        }
    };
}

i8x16_extract_lane!(i8x16_extract_lane_s, signed);
i8x16_extract_lane!(i8x16_extract_lane_u, unsigned);

pub fn i8x16_replace_lane(stack: &mut dyn StackBehavior, lane_idx: u8) -> Result<()> {
    let val_to_insert = stack.pop_i32()? as i8;
    let mut v = stack.pop_v128()?;
    // Check if lane_idx is valid (0-15)
    if lane_idx >= 16 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane_idx
        ))));
    }
    v[lane_idx as usize] = val_to_insert.to_le_bytes()[0];
    stack.push(Value::V128(V128::from(v)))?;
    Ok(())
}

macro_rules! i16x8_extract_lane {
    ($name:ident, $signedness:ident) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
            lane_idx: u8,
        ) -> Result<()> {
            let v = stack.pop()?.as_v128()?;
            let start = lane_idx as usize * 2;
            let bytes: [u8; 2] = v[start..start + 2].try_into().unwrap();
            let val = if stringify!($signedness) == "signed" {
                i16::from_le_bytes(bytes) as i32
            } else {
                u16::from_le_bytes(bytes) as i32
            };
            stack.push(Value::I32(val))
        }
    };
}

i16x8_extract_lane!(i16x8_extract_lane_s, signed);
i16x8_extract_lane!(i16x8_extract_lane_u, unsigned);

pub fn i16x8_replace_lane(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    lane_idx: u8,
) -> Result<()> {
    let val = stack.pop()?.as_i32().ok_or_else(|| {
        Error::new(kinds::ValidationError(
            "Expected i32 for lane replacement value".to_string(),
        ))
    })?;
    let mut v = stack.pop()?.as_v128()?;
    if lane_idx >= 8 {
        // Indices >= 8 are invalid according to spec, result is implementation defined.
        // Let's return an error for simplicity/determinism.
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane_idx
        ))));
    }
    let start = lane_idx as usize * 2;
    v[start..start + 2].copy_from_slice(&(val as i16).to_le_bytes());
    stack.push(Value::V128(v))
}

pub fn i32x4_extract_lane(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    lane_idx: u8,
) -> Result<()> {
    let v = stack.pop()?.as_v128()?;
    let start = lane_idx as usize * 4;
    let bytes: [u8; 4] = v[start..start + 4].try_into().unwrap();
    let val = i32::from_le_bytes(bytes);
    stack.push(Value::I32(val))
}

pub fn i32x4_replace_lane(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    lane_idx: u8,
) -> Result<()> {
    let val = stack.pop()?.as_i32().ok_or_else(|| {
        Error::new(kinds::ValidationError(
            "Expected i32 for lane replacement value".to_string(),
        ))
    })?;
    let mut v = stack.pop()?.as_v128()?;
    if lane_idx >= 4 {
        // Indices >= 4 are invalid according to spec, result is implementation defined.
        // Let's return an error for simplicity/determinism.
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane_idx
        ))));
    }
    let start = lane_idx as usize * 4;
    v[start..start + 4].copy_from_slice(&val.to_le_bytes());
    stack.push(Value::V128(v))
}

pub fn i64x2_extract_lane(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    lane_idx: u8,
) -> Result<()> {
    let v = stack.pop()?.as_v128()?;
    let start = lane_idx as usize * 8;
    let bytes: [u8; 8] = v[start..start + 8].try_into().unwrap();
    let val = i64::from_le_bytes(bytes);
    stack.push(Value::I64(val))
}

pub fn i64x2_replace_lane(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    lane_idx: u8,
) -> Result<()> {
    let val = stack.pop()?.as_i64().ok_or_else(|| {
        Error::new(kinds::ValidationError(
            "Expected i64 for lane replacement value".to_string(),
        ))
    })?;
    let mut v = stack.pop()?.as_v128()?;
    if lane_idx >= 2 {
        // Indices >= 2 are invalid according to spec, result is implementation defined.
        // Let's return an error for simplicity/determinism.
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane_idx
        ))));
    }
    let start = lane_idx as usize * 8;
    v[start..start + 8].copy_from_slice(&val.to_le_bytes());
    stack.push(Value::V128(v))
}

pub fn f32x4_extract_lane(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    lane_idx: u8,
) -> Result<()> {
    let v = stack.pop()?.as_v128()?;
    let start = lane_idx as usize * 4;
    let bytes: [u8; 4] = v[start..start + 4].try_into().unwrap();
    let val = f32::from_le_bytes(bytes);
    stack.push(Value::F32(val))
}

pub fn f32x4_replace_lane(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    lane_idx: u8,
) -> Result<()> {
    let val = stack.pop()?.as_f32().ok_or_else(|| {
        Error::new(kinds::ValidationError(
            "Expected f32 for lane replacement value".to_string(),
        ))
    })?;
    let mut v = stack.pop()?.as_v128()?;
    if lane_idx >= 4 {
        // Indices >= 4 are invalid according to spec, result is implementation defined.
        // Let's return an error for simplicity/determinism.
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane_idx
        ))));
    }
    let start = lane_idx as usize * 4;
    v[start..start + 4].copy_from_slice(&val.to_le_bytes());
    stack.push(Value::V128(v))
}

pub fn f64x2_extract_lane(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    lane_idx: u8,
) -> Result<()> {
    let v = stack.pop()?.as_v128()?;
    let start = lane_idx as usize * 8;
    let bytes: [u8; 8] = v[start..start + 8].try_into().unwrap();
    let val = f64::from_le_bytes(bytes);
    stack.push(Value::F64(val))
}

pub fn f64x2_replace_lane(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    lane_idx: u8,
) -> Result<()> {
    let val = stack.pop()?.as_f64().ok_or_else(|| {
        Error::new(kinds::ValidationError(
            "Expected f64 for lane replacement value".to_string(),
        ))
    })?;
    let mut v = stack.pop()?.as_v128()?;
    if lane_idx >= 2 {
        // Indices >= 2 are invalid according to spec, result is implementation defined.
        // Let's return an error for simplicity/determinism.
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane_idx
        ))));
    }
    let start = lane_idx as usize * 8;
    v[start..start + 8].copy_from_slice(&val.to_le_bytes());
    stack.push(Value::V128(v))
}

// Unary ops
macro_rules! v128_unary_op {
    ($name:ident, $op:ident) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
        ) -> Result<()> {
            let a = stack.pop()?.as_v128()?;
            let mut result = [0u8; 16];
            for i in 0..16 {
                result[i] = a[i].$op();
            }
            stack.push(Value::V128(result))
        }
    };
}

macro_rules! i8x16_unary_op {
    ($name:ident, $op:ident) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
        ) -> Result<()> {
            let a_v = stack.pop()?.as_v128()?;
            let mut result = [0u8; 16];
            for i in 0..16 {
                result[i] = (a_v[i] as i8).$op() as u8;
            }
            stack.push(Value::V128(result))
        }
    };
}

macro_rules! i16x8_unary_op {
    ($name:ident, $op:ident) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
        ) -> Result<()> {
            let a_v = stack.pop()?.as_v128()?;
            let mut result = [0u8; 16];
            for i in 0..8 {
                let start = i * 2;
                let a = i16::from_le_bytes(a_v[start..start + 2].try_into().unwrap());
                let r = a.$op();
                result[start..start + 2].copy_from_slice(&r.to_le_bytes());
            }
            stack.push(Value::V128(result))
        }
    };
}

macro_rules! i32x4_unary_op {
    ($name:ident, $op:ident) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
        ) -> Result<()> {
            let a_v = stack.pop()?.as_v128()?;
            let mut result = [0u8; 16];
            for i in 0..4 {
                let start = i * 4;
                let a = i32::from_le_bytes(a_v[start..start + 4].try_into().unwrap());
                let r = a.$op();
                result[start..start + 4].copy_from_slice(&r.to_le_bytes());
            }
            stack.push(Value::V128(result))
        }
    };
}

macro_rules! i64x2_unary_op {
    ($name:ident, $op:ident) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
        ) -> Result<()> {
            let a_v = stack.pop()?.as_v128()?;
            let mut result = [0u8; 16];
            for i in 0..2 {
                let start = i * 8;
                let a = i64::from_le_bytes(a_v[start..start + 8].try_into().unwrap());
                let r = a.$op();
                result[start..start + 8].copy_from_slice(&r.to_le_bytes());
            }
            stack.push(Value::V128(result))
        }
    };
}

macro_rules! f32x4_unary_op {
    ($name:ident, $op:ident) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
        ) -> Result<()> {
            let a_v = stack.pop()?.as_v128()?;
            let mut result = [0u8; 16];
            for i in 0..4 {
                let start = i * 4;
                let a = f32::from_le_bytes(a_v[start..start + 4].try_into().unwrap());
                let r = a.$op();
                result[start..start + 4].copy_from_slice(&r.to_le_bytes());
            }
            stack.push(Value::V128(result))
        }
    };
}

macro_rules! f64x2_unary_op {
    ($name:ident, $op:ident) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
        ) -> Result<()> {
            let a_v = stack.pop()?.as_v128()?;
            let mut result = [0u8; 16];
            for i in 0..2 {
                let start = i * 8;
                let a = f64::from_le_bytes(a_v[start..start + 8].try_into().unwrap());
                let r = a.$op();
                result[start..start + 8].copy_from_slice(&r.to_le_bytes());
            }
            stack.push(Value::V128(result))
        }
    };
}

pub fn v128_not(
    stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    let a = stack.pop()?.as_v128()?;
    let mut result = [0u8; 16];
    for i in 0..16 {
        result[i] = !a[i];
    }
    stack.push(Value::V128(result))
}

// Population count
pub fn i8x16_popcnt(
    stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    let a_v = stack.pop()?.as_v128()?;
    let mut result = [0u8; 16];
    for i in 0..16 {
        result[i] = a_v[i].count_ones() as u8;
    }
    stack.push(Value::V128(result))
}

// Negation
i8x16_unary_op!(i8x16_neg, neg);
i16x8_unary_op!(i16x8_neg, neg);
i32x4_unary_op!(i32x4_neg, neg);
i64x2_unary_op!(i64x2_neg, neg);

// Absolute value
f32x4_unary_op!(f32x4_abs, abs);
f64x2_unary_op!(f64x2_abs, abs);

// Negation for floats
f32x4_unary_op!(f32x4_neg, neg);
f64x2_unary_op!(f64x2_neg, neg);

// Square root
f32x4_unary_op!(f32x4_sqrt, sqrt);
f64x2_unary_op!(f64x2_sqrt, sqrt);

// Rounding
f32x4_unary_op!(f32x4_ceil, ceil);
f32x4_unary_op!(f32x4_floor, floor);
f32x4_unary_op!(f32x4_trunc, trunc);
// FIXME: Implement nearest_integer
// f32x4_unary_op!(f32x4_nearest, nearest_integer);

f64x2_unary_op!(f64x2_ceil, ceil);
f64x2_unary_op!(f64x2_floor, floor);
f64x2_unary_op!(f64x2_trunc, trunc);
// FIXME: Implement nearest_integer
// f64x2_unary_op!(f64x2_nearest, nearest_integer);

// Bitmask extraction
macro_rules! v128_bitmask {
    ($name:ident, $ty:ty, $lanes:expr) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
        ) -> Result<()> {
            let v = stack.pop()?.as_v128()?;
            let mut result: i32 = 0;
            let lane_bytes = 16 / $lanes;
            for i in 0..$lanes {
                let start = i * lane_bytes;
                let lane_val =
                    <$ty>::from_le_bytes(v[start..start + lane_bytes].try_into().unwrap());
                if lane_val < 0 {
                    // Check sign bit
                    result |= 1 << i;
                }
            }
            stack.push(Value::I32(result))
        }
    };
}

v128_bitmask!(i8x16_bitmask, i8, 16);
v128_bitmask!(i16x8_bitmask, i16, 8);
v128_bitmask!(i32x4_bitmask, i32, 4);
v128_bitmask!(i64x2_bitmask, i64, 2);

// Splat
macro_rules! v128_splat {
    ($name:ident, $value_type:path, $bytes:expr, $accessor:ident) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
        ) -> Result<()> {
            let val = stack.pop()?;
            let val = val.$accessor().ok_or_else(|| {
                Error::new(kinds::InvalidTypeError(format!(
                    "Expected type for {}, found {}",
                    stringify!($name),
                    val.type_()
                )))
            })?;
            let mut splatted = [0u8; 16];
            for chunk in splatted.chunks_exact_mut($bytes) {
                chunk.copy_from_slice(&val.to_le_bytes()[0..$bytes]);
            }
            stack.push(Value::V128(splatted))
        }
    };
}

v128_splat!(i8x16_splat, Value::I32, 1, as_i32);
v128_splat!(i16x8_splat, Value::I32, 2, as_i32);
v128_splat!(i32x4_splat, Value::I32, 4, as_i32);
v128_splat!(i64x2_splat, Value::I64, 8, as_i64);
v128_splat!(f32x4_splat, Value::F32, 4, as_f32);
v128_splat!(f64x2_splat, Value::F64, 8, as_f64);

// Any/All True
pub fn v128_any_true(
    stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    let v = stack.pop()?.as_v128()?;
    let any_true = v.iter().any(|&byte| byte != 0);
    stack.push(Value::I32(any_true as i32))
}

macro_rules! v128_all_true {
    ($name:ident, $lanes:expr) => {
        pub fn $name(
            stack: &mut dyn StackBehavior,
            _frame: &mut dyn FrameBehavior,
            _engine: &StacklessEngine,
        ) -> Result<()> {
            let v = stack.pop()?.as_v128()?;
            let lane_bytes = 16 / $lanes;
            let all_true = (0..$lanes).all(|i| {
                let start = i * lane_bytes;
                let lane_slice = &v[start..start + lane_bytes];
                // Check if the most significant bit is set (for signed comparison)
                // This assumes the input is a result of a comparison where non-zero means true.
                // A more robust check might depend on the specific comparison that produced the mask.
                // For simplicity, let's check if *any* bit is set in the lane, representing true.
                lane_slice.iter().any(|&byte| byte != 0)
            });
            stack.push(Value::I32(all_true as i32))
        }
    };
}

v128_all_true!(i8x16_all_true, 16);
v128_all_true!(i16x8_all_true, 8);
v128_all_true!(i32x4_all_true, 4);
v128_all_true!(i64x2_all_true, 2);

// Bitselect
pub fn v128_bitselect(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
) -> Result<()> {
    let c = stack.pop()?.as_v128()?;
    let v2 = stack.pop()?.as_v128()?;
    let v1 = stack.pop()?.as_v128()?;
    let mut result = [0u8; 16];
    for i in 0..16 {
        result[i] = (v1[i] & c[i]) | (v2[i] & !c[i]);
    }
    stack.push(Value::V128(result))
}

pub fn v128_load(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
) -> Result<()> {
    // Pop address, handle potential type mismatch
    let addr_val = stack.pop()?;
    let addr = addr_val.as_i32().ok_or_else(|| {
        Error::new(kinds::InvalidTypeError(format!(
            "Expected I32 address, found {:?}",
            addr_val.value_type()
        )))
    })? as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let bytes = frame.load_v128(effective_addr, memarg.align.into(), engine)?;
    stack.push(Value::V128(bytes))
}

pub fn v128_store(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
) -> Result<()> {
    let val_to_store = stack.pop()?.as_v128()?;
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    frame.store_v128(effective_addr, memarg.align.into(), val_to_store, engine)?;
    Ok(())
}

pub fn v128_load8_lane(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
    lane_idx: &u8,
) -> Result<()> {
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let mut vec_val = stack.pop()?.as_v128()?;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let byte = frame.load_u8(effective_addr, memarg.align.into(), engine)?;
    let lane = *lane_idx as usize;
    if lane >= 16 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane
        ))));
    }
    vec_val[lane] = byte;
    stack.push(Value::V128(V128::from(vec_val)))?;
    Ok(())
}

pub fn v128_load16_lane(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
    lane_idx: &u8,
) -> Result<()> {
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let mut vec_val = stack.pop()?.as_v128()?;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let val16 = frame.load_u16(effective_addr, memarg.align.into(), engine)?;
    let lane = *lane_idx as usize;
    if lane >= 8 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane
        ))));
    }
    vec_val[lane * 2..lane * 2 + 2].copy_from_slice(&val16.to_le_bytes());
    stack.push(Value::V128(V128::from(vec_val)))?;
    Ok(())
}

pub fn v128_load32_lane(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
    lane_idx: &u8,
) -> Result<()> {
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let mut vec_val = stack.pop()?.as_v128()?;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let val32 = frame.load_i32(effective_addr, memarg.align.into(), engine)? as u32;
    let lane = *lane_idx as usize;
    if lane >= 4 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane
        ))));
    }
    vec_val[lane * 4..lane * 4 + 4].copy_from_slice(&val32.to_le_bytes());
    stack.push(Value::V128(V128::from(vec_val)))?;
    Ok(())
}

pub fn v128_load64_lane(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
    lane_idx: &u8,
) -> Result<()> {
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let mut vec_val = stack.pop()?.as_v128()?;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let val64 = frame.load_i64(effective_addr, memarg.align.into(), engine)? as u64;
    let lane = *lane_idx as usize;
    if lane >= 2 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane
        ))));
    }
    vec_val[lane * 8..lane * 8 + 8].copy_from_slice(&val64.to_le_bytes());
    stack.push(Value::V128(V128::from(vec_val)))?;
    Ok(())
}

pub fn v128_store8_lane(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
    lane_idx: &u8,
) -> Result<()> {
    let vec_val = stack.pop()?.as_v128()?;
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let lane = *lane_idx as usize;
    if lane >= 16 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane
        ))));
    }
    let byte_to_store = vec_val[lane];
    frame.store_u8(effective_addr, memarg.align.into(), byte_to_store, engine)?;
    Ok(())
}

pub fn v128_store16_lane(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
    lane_idx: &u8,
) -> Result<()> {
    let vec_val = stack.pop()?.as_v128()?;
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let lane = *lane_idx as usize;
    if lane >= 8 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane
        ))));
    }
    let bytes_to_store: [u8; 2] = vec_val[lane * 2..lane * 2 + 2].try_into().map_err(|_| {
        Error::new(kinds::ValidationError(
            "SIMD lane conversion failed".to_string(),
        ))
    })?;
    frame.store_u16(
        effective_addr,
        memarg.align.into(),
        u16::from_le_bytes(bytes_to_store),
        engine,
    )?;
    Ok(())
}

pub fn v128_store32_lane(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
    lane_idx: &u8,
) -> Result<()> {
    let vec_val = stack.pop()?.as_v128()?;
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let lane = *lane_idx as usize;
    if lane >= 4 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane
        ))));
    }
    let bytes_to_store: [u8; 4] = vec_val[lane * 4..lane * 4 + 4].try_into().map_err(|_| {
        Error::new(kinds::ValidationError(
            "SIMD lane conversion failed".to_string(),
        ))
    })?;
    frame.store_i32(
        effective_addr,
        memarg.align.into(),
        i32::from_le_bytes(bytes_to_store),
        engine,
    )?;
    Ok(())
}

pub fn v128_store64_lane(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
    lane_idx: &u8,
) -> Result<()> {
    let vec_val = stack.pop()?.as_v128()?;
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let lane = *lane_idx as usize;
    if lane >= 2 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Invalid lane index: {}",
            lane
        ))));
    }
    let bytes_to_store: [u8; 8] = vec_val[lane * 8..lane * 8 + 8].try_into().map_err(|_| {
        Error::new(kinds::ValidationError(
            "SIMD lane conversion failed".to_string(),
        ))
    })?;
    frame.store_i64(
        effective_addr,
        memarg.align.into(),
        i64::from_le_bytes(bytes_to_store),
        engine,
    )?;
    Ok(())
}

pub fn v128_load32_zero(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
) -> Result<()> {
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let loaded_val = frame.load_i32(effective_addr, memarg.align.into(), engine)? as u32;
    let mut bytes = [0u8; 16];
    bytes[0..4].copy_from_slice(&loaded_val.to_le_bytes());
    stack.push(Value::V128(V128::from(bytes)))?;
    Ok(())
}

pub fn v128_load64_zero(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
) -> Result<()> {
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let loaded_val = frame.load_i64(effective_addr, memarg.align.into(), engine)? as u64;
    let mut bytes = [0u8; 16];
    bytes[0..8].copy_from_slice(&loaded_val.to_le_bytes());
    stack.push(Value::V128(V128::from(bytes)))?;
    Ok(())
}

pub fn v128_load8_splat(
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
) -> Result<()> {
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let byte = frame.load_u8(effective_addr, memarg.align.into(), engine)?;
    let bytes = [byte; 16];
    stack.push(Value::V128(V128::from(bytes)))?;
    Ok(())
}

pub fn v128_load8x8_s(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
) -> Result<()> {
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let mut result_bytes = [0u8; 16];
    for i in 0..8 {
        let byte_addr = effective_addr + i;
        let val = frame.load_i8(byte_addr, 0, engine)?;
        let extended_val = (val as i16).to_le_bytes();
        result_bytes[2 * i..2 * i + 2].copy_from_slice(&extended_val);
    }
    stack.push(Value::V128(V128::from(result_bytes)))?;
    Ok(())
}

pub fn v128_load8x8_u(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
) -> Result<()> {
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let mut result_bytes = [0u8; 16];
    for i in 0..8 {
        let byte_addr = effective_addr + i;
        let val = frame.load_u8(byte_addr, 0, engine)?;
        let extended_val = (val as u16).to_le_bytes();
        result_bytes[2 * i..2 * i + 2].copy_from_slice(&extended_val);
    }
    stack.push(Value::V128(V128::from(result_bytes)))?;
    Ok(())
}

pub fn v128_load16x4_s(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
) -> Result<()> {
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let mut result_bytes = [0u8; 16];
    for i in 0..4 {
        let word_addr = effective_addr + i * 2;
        let val = frame.load_i16(word_addr, memarg.align.into(), engine)?;
        let extended_val = (val as i32).to_le_bytes();
        result_bytes[4 * i..4 * i + 4].copy_from_slice(&extended_val);
    }
    stack.push(Value::V128(V128::from(result_bytes)))?;
    Ok(())
}

pub fn v128_load16x4_u(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    engine: &StacklessEngine,
    memarg: &MemArg,
) -> Result<()> {
    let addr = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::ValidationError("Expected i32 address".to_string())))?
        as u32;
    let effective_addr = (addr as u64 + memarg.offset as u64) as usize;
    let mut result_bytes = [0u8; 16];
    for i in 0..4 {
        let word_addr = effective_addr + i * 2;
        let val = frame.load_u16(word_addr, memarg.align.into(), engine)?;
        let extended_val = (val as u32).to_le_bytes();
        result_bytes[4 * i..4 * i + 4].copy_from_slice(&extended_val);
    }
    stack.push(Value::V128(V128::from(result_bytes)))?;
    Ok(())
}
