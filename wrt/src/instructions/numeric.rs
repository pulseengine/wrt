use crate::{
    behavior::{FrameBehavior, StackBehavior},
    error::{self, kinds, Error, Result},
    values::Value,
    StacklessEngine,
};

/// Push a 32-bit integer constant onto the stack
pub fn i32_const(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    value: i32,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    _stack.push(Value::I32(value))?;
    Ok(())
}

/// Push a 64-bit integer constant onto the stack
pub fn i64_const(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    value: i64,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    _stack.push(Value::I64(value))?;
    Ok(())
}

/// Push a 32-bit float constant onto the stack
pub fn f32_const(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    value: f32,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    _stack.push(Value::F32(value))?;
    Ok(())
}

pub fn f64_const(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    value: f64,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    _stack.push(Value::F64(value))?;
    Ok(())
}

pub fn i32_clz(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::I32(a) => {
            _stack.push(Value::I32(a.leading_zeros() as i32))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_ctz(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::I32(a) => {
            _stack.push(Value::I32(a.trailing_zeros() as i32))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_popcnt(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::I32(a) => {
            _stack.push(Value::I32(a.count_ones() as i32))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_add(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;

    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(a.wrapping_add(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_sub(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(a.wrapping_sub(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_mul(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(a.wrapping_mul(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_div_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            if b == 0 {
                return Err(Error::new(kinds::Trap(
                    "integer division by zero".to_string(),
                )));
            }
            if a == i32::MIN && b == -1 {
                return Err(Error::new(kinds::Trap("integer overflow".to_string())));
            }
            _stack.push(Value::I32(a.wrapping_div(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_div_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            if b == 0 {
                return Err(Error::new(kinds::Trap(
                    "integer division by zero".to_string(),
                )));
            }
            let a = a as u32;
            let b = b as u32;
            _stack.push(Value::I32((a / b) as i32))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_rem_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            if b == 0 {
                return Err(Error::new(kinds::Trap(
                    "integer division by zero".to_string(),
                )));
            }
            _stack.push(Value::I32(a.wrapping_rem(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_rem_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            if b == 0 {
                return Err(Error::new(kinds::Trap(
                    "integer division by zero".to_string(),
                )));
            }
            let a = a as u32;
            let b = b as u32;
            _stack.push(Value::I32((a % b) as i32))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_and(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(a & b))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_or(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(a | b))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_xor(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(a ^ b))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_shl(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(a.wrapping_shl(b as u32)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_shr_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(a.wrapping_shr(b as u32)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_shr_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            _stack.push(Value::I32((a.wrapping_shr(b as u32)) as i32))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_rotl(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let b = b as u32;
            let b = b % 32;
            _stack.push(Value::I32(a.rotate_left(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_rotr(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let b = b as u32;
            let b = b % 32;
            _stack.push(Value::I32(a.rotate_right(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_eqz(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::I32(a) => {
            _stack.push(Value::I32(i32::from(a == 0)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_eq(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(i32::from(a == b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_ne(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(i32::from(a != b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_lt_s(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_lt_u(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            _stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_gt_s(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_gt_u(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            _stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_le_s(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_le_u(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            _stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_ge_s(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            _stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i32_ge_u(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            _stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i32".to_string())),
    }
}

pub fn i64_add(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            _stack.push(Value::I64(a.wrapping_add(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

pub fn i64_sub(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            _stack.push(Value::I64(a.wrapping_sub(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

pub fn i64_mul(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            _stack.push(Value::I64(a.wrapping_mul(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

pub fn i64_div_s(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            if b == 0 {
                return Err(Error::new(kinds::Trap(
                    "integer division by zero".to_string(),
                )));
            }
            if a == i64::MIN && b == -1 {
                _stack.push(Value::I64(i64::MIN))?;
            } else {
                _stack.push(Value::I64(a / b))?;
            }
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

pub fn i64_div_u(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            if b == 0 {
                return Err(Error::new(kinds::Trap(
                    "integer division by zero".to_string(),
                )));
            }
            let ua = a as u64;
            let ub = b as u64;
            _stack.push(Value::I64((ua / ub) as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_rem_s(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            if b == 0 {
                return Err(Error::new(kinds::Trap(
                    "integer division by zero".to_string(),
                )));
            }
            _stack.push(Value::I64(a % b))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_rem_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            if b == 0 {
                return Err(Error::new(kinds::Trap(
                    "integer division by zero".to_string(),
                )));
            }
            let ua = a as u64;
            let ub = b as u64;
            _stack.push(Value::I64((ua % ub) as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_and(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            _stack.push(Value::I64(a & b))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_or(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            _stack.push(Value::I64(a | b))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_xor(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            _stack.push(Value::I64(a ^ b))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_shl(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let shift = (b & 0x3F) as u32;
            _stack.push(Value::I64(a << shift))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_shr_s(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let shift = (b & 0x3F) as u32;
            _stack.push(Value::I64(a >> shift))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_shr_u(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let shift = (b & 0x3F) as u32;
            let ua = a as u64;
            _stack.push(Value::I64((ua >> shift) as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_rotl(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let shift = (b & 0x3F) as u32;
            let ua = a as u64;
            let rotated = ua.rotate_left(shift);
            _stack.push(Value::I64(rotated as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_rotr(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let shift = (b & 0x3F) as u32;
            let ua = a as u64;
            let rotated = ua.rotate_right(shift);
            _stack.push(Value::I64(rotated as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_clz(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::I64(a) => {
            _stack.push(Value::I64(a.leading_zeros() as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_ctz(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::I64(a) => {
            _stack.push(Value::I64(a.trailing_zeros() as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i64_popcnt(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::I64(a) => {
            _stack.push(Value::I64(a.count_ones() as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn f32_sub(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::F32(a - b))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_mul(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::F32(a * b))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_div(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::F32(a / b))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_min(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::F32(a.min(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_max(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::F32(a.max(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_abs(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F32(a) => {
            _stack.push(Value::F32(a.abs()))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_neg(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F32(a) => {
            _stack.push(Value::F32(-a))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_copysign(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::F32(a.copysign(b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_ceil(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F32(a) => {
            _stack.push(Value::F32(a.ceil()))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_floor(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F32(a) => {
            _stack.push(Value::F32(a.floor()))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_trunc(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F32(a) => {
            _stack.push(Value::F32(a.trunc()))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_nearest(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;

    if let Value::F32(f) = a {
        if f.is_nan() || f.is_infinite() || f == 0.0 {
            _stack.push(a)?;
            return Ok(());
        }

        let fractional = f.abs() - f.abs().floor();
        const EPSILON: f32 = 1e-10;

        let result = if (fractional - 0.5).abs() < EPSILON {
            let floor = f.floor();
            let sign = if f < 0.0 { -1.0 } else { 1.0 };
            let abs_floor = f.abs().floor();

            if (abs_floor as i32) % 2 == 0 {
                sign * abs_floor
            } else {
                sign * (abs_floor + 1.0)
            }
        } else {
            f.round()
        };

        _stack.push(Value::F32(result))?;
    } else {
        return Err(Error::new(kinds::ExecutionError(
            "Expected f32 value".into(),
        )));
    }

    Ok(())
}

pub fn f32_sqrt(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F32(a) => {
            _stack.push(Value::F32(a.sqrt()))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_eq(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::I32(i32::from(a == b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_ne(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::I32(i32::from(a != b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_lt(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_gt(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_le(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f32_ge(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

pub fn f64_add(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::F64(a + b))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}

pub fn f64_sub(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::F64(a - b))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}

pub fn f64_mul(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::F64(a * b))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}

pub fn f64_div(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::F64(a / b))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_min(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::F64(a.min(b)))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_max(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::F64(a.max(b)))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_abs(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F64(a) => {
            _stack.push(Value::F64(a.abs()))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_neg(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F64(a) => {
            _stack.push(Value::F64(-a))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_copysign(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::F64(a.copysign(b)))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_ceil(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F64(a) => {
            _stack.push(Value::F64(a.ceil()))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_floor(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F64(a) => {
            _stack.push(Value::F64(a.floor()))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}

pub fn f64_trunc(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F64(a) => {
            _stack.push(Value::F64(a.trunc()))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}

pub fn f64_nearest(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;

    if let Value::F64(f) = a {
        if f.is_nan() || f.is_infinite() || f == 0.0 {
            _stack.push(a)?;
            return Ok(());
        }

        let fractional = f.abs() - f.abs().floor();
        const EPSILON: f64 = 1e-15;

        let result = if (fractional - 0.5).abs() < EPSILON {
            let floor = f.floor();
            let sign = if f < 0.0 { -1.0 } else { 1.0 };
            let abs_floor = f.abs().floor();

            if (abs_floor as i64) % 2 == 0 {
                sign * abs_floor
            } else {
                sign * (abs_floor + 1.0)
            }
        } else {
            f.round()
        };

        _stack.push(Value::F64(result))?;
    } else {
        return Err(Error::new(kinds::ExecutionError(
            "Expected f64 value".into(),
        )));
    }

    Ok(())
}

pub fn f64_sqrt(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F64(a) => {
            _stack.push(Value::F64(a.sqrt()))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f64".to_string())),
    }
}

pub fn f64_eq(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::I32(i32::from(a == b)))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_ne(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::I32(i32::from(a != b)))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_lt(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_gt(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_le(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn f64_ge(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            _stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn i64_extend_i32_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::I32(a) => {
            _stack.push(Value::I64(i64::from(a)))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i32".to_string(),
        ))),
    }
}

pub fn i64_extend_i32_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::I32(a) => {
            _stack.push(Value::I64(i64::from(a)))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i32".to_string(),
        ))),
    }
}

pub fn i64_trunc_f32_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F32(a) => {
            _stack.push(Value::I64(a as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f32".to_string(),
        ))),
    }
}

pub fn i64_trunc_f32_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F32(a) => {
            _stack.push(Value::I64(a as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f32".to_string(),
        ))),
    }
}

pub fn i64_trunc_f64_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F64(a) => {
            _stack.push(Value::I64(a as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn i64_trunc_f64_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::F64(a) => {
            _stack.push(Value::I64(a as i64))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected f64".to_string(),
        ))),
    }
}

pub fn i32_trunc_sat_f32_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i32_trunc_sat_f32_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i32_trunc_sat_f64_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i32_trunc_sat_f64_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i64_trunc_sat_f32_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i64_trunc_sat_f32_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i64_trunc_sat_f64_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i64_trunc_sat_f64_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i32_trunc_f32_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i32_trunc_f32_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i32_trunc_f64_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i32_trunc_f64_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    unimplemented!()
}

pub fn i32_extend8_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    todo!("i32_extend8_s")
}

pub fn i32_extend16_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    todo!("i32_extend16_s")
}

pub fn i64_extend8_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    todo!("i64_extend8_s")
}

pub fn i64_extend16_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    todo!("i64_extend16_s")
}

pub fn i64_extend32_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    todo!("i64_extend32_s")
}

pub fn i64_eqz(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let a = _stack.pop()?;
    match a {
        Value::I64(a) => {
            _stack.push(Value::I32(if a == 0 { 1 } else { 0 }))?;
            Ok(())
        }
        _ => Err(Error::new(kinds::InvalidTypeError(
            "Expected i64".to_string(),
        ))),
    }
}

pub fn i32_wrap_i64(_stack: &mut dyn StackBehavior, _frame: &mut dyn FrameBehavior) -> Result<()> {
    let val = _stack.pop_i64()?;
    _stack.push(Value::I32(val as i32))?;
    Ok(())
}

pub fn f32_convert_i32_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop_i32()?;
    _stack.push(Value::F32(val as f32))?;
    Ok(())
}

pub fn f32_convert_i32_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop_i32()? as u32;
    _stack.push(Value::F32(val as f32))?;
    Ok(())
}

pub fn f32_convert_i64_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop_i64()?;
    _stack.push(Value::F32(val as f32))?;
    Ok(())
}

pub fn f32_convert_i64_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop_i64()? as u64;
    _stack.push(Value::F32(val as f32))?;
    Ok(())
}

pub fn f32_demote_f64(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop()?;
    let f_val = val.as_f64().ok_or_else(|| {
        Error::new(kinds::InvalidTypeError(format!(
            "Expected F64, found {:?}",
            val.value_type()
        )))
    })?;
    _stack.push(Value::F32(f_val as f32))?;
    Ok(())
}

pub fn f64_convert_i32_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop_i32()?;
    _stack.push(Value::F64(val as f64))?;
    Ok(())
}

pub fn f64_convert_i32_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop_i32()? as u32;
    _stack.push(Value::F64(val as f64))?;
    Ok(())
}

pub fn f64_convert_i64_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop_i64()?;
    _stack.push(Value::F64(val as f64))?;
    Ok(())
}

pub fn f64_convert_i64_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop_i64()? as u64;
    _stack.push(Value::F64(val as f64))?;
    Ok(())
}

pub fn f64_promote_f32(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop()?;
    let f_val = val.as_f32().ok_or_else(|| {
        Error::new(kinds::InvalidTypeError(format!(
            "Expected F32, found {:?}",
            val.value_type()
        )))
    })?;
    _stack.push(Value::F64(f_val as f64))?;
    Ok(())
}

/// Implements the reinterpret i32 as f32 operation
pub fn i32_reinterpret_f32(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop()?;
    if let Value::I32(int_val) = val {
        _stack.push(Value::F32(f32::from_bits(int_val as u32)))
    } else {
        Err(Error::invalid_type("Expected i32".to_string()))
    }
}

/// Implements the reinterpret i64 as f64 operation
pub fn i64_reinterpret_f64(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop()?;
    if let Value::I64(int_val) = val {
        _stack.push(Value::F64(f64::from_bits(int_val as u64)))
    } else {
        Err(Error::invalid_type("Expected i64".to_string()))
    }
}

pub fn f32_reinterpret_i32(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop()?;
    if let Value::F32(float_val) = val {
        _stack.push(Value::I32(float_val.to_bits() as i32))
    } else {
        Err(Error::invalid_type("Expected f32".to_string()))
    }
}

pub fn f64_reinterpret_i64(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<()> {
    let val = _stack.pop()?;
    if let Value::F64(float_val) = val {
        _stack.push(Value::I64(float_val.to_bits() as i64))
    } else {
        Err(Error::invalid_type("Expected f64".to_string()))
    }
}

/// Execute an i64 greater than or equal to, signed version
pub fn i64_ge_s(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            _stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an i64 greater than or equal to, unsigned version
pub fn i64_ge_u(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let a = a as u64;
            let b = b as u64;
            _stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected i64".to_string())),
    }
}

/// Execute an f32 add instruction
///
/// Pops two f32 values from the stack, adds them, and pushes the result.
pub fn f32_add(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
) -> Result<()> {
    let b = _stack.pop()?;
    let a = _stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            _stack.push(Value::F32(a + b))?;
            Ok(())
        }
        _ => Err(Error::invalid_type("Expected f32".to_string())),
    }
}

#[cfg(feature = "std")]
pub fn _verify_value(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<ControlFlow, Error> {
    // ... existing code ...
}

#[cfg(feature = "std")]
pub fn _verify_i32(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<ControlFlow, Error> {
    // ... existing code ...
}

#[cfg(feature = "std")]
pub fn _verify_i64(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<ControlFlow, Error> {
    // ... existing code ...
}

#[cfg(feature = "std")]
pub fn _verify_f32(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<ControlFlow, Error> {
    // ... existing code ...
}

#[cfg(feature = "std")]
pub fn _verify_f64(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<ControlFlow, Error> {
    // ... existing code ...
}

#[cfg(feature = "std")]
pub fn _verify_v128(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<ControlFlow, Error> {
    // ... existing code ...
}

#[cfg(feature = "std")]
pub fn _verify_func_ref(
    _stack: &mut dyn StackBehavior,
    _frame: &mut dyn FrameBehavior,
) -> Result<ControlFlow, Error> {
    // ... existing code ...
}

#[cfg(feature = "std")]
pub fn _check_i32_nan(_stack: &mut dyn StackBehavior) -> Result<ControlFlow, Error> {
    // ... existing code ...
}

#[cfg(feature = "std")]
pub fn _check_i64_nan(_stack: &mut dyn StackBehavior) -> Result<ControlFlow, Error> {
    // ... existing code ...
}

#[cfg(feature = "std")]
pub fn _check_f32_nan(_stack: &mut dyn StackBehavior) -> Result<ControlFlow, Error> {
    // ... existing code ...
}

#[cfg(feature = "std")]
pub fn _check_f64_nan(_stack: &mut dyn StackBehavior) -> Result<ControlFlow, Error> {
    // ... existing code ...
}
