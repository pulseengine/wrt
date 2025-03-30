use crate::{
    behavior::{FrameBehavior, StackBehavior},
    error::{Error, Result},
    stack::Stack,
    values::Value,
};

/// Push a 32-bit integer constant onto the stack
pub fn i32_const(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
    value: i32,
) -> Result<()> {
    stack.push(Value::I32(value));
    Ok(())
}

pub fn i64_const(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
    value: i64,
) -> Result<()> {
    stack.push(Value::I64(value));
    Ok(())
}

pub fn f32_const(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
    value: f32,
) -> Result<()> {
    stack.push(Value::F32(value));
    Ok(())
}

pub fn f64_const(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
    value: f64,
) -> Result<()> {
    stack.push(Value::F64(value));
    Ok(())
}

pub fn i32_clz(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::I32(a.leading_zeros() as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_ctz(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::I32(a.trailing_zeros() as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_popcnt(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::I32(a.count_ones() as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_add(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(a.wrapping_add(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_sub(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(a.wrapping_sub(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_mul(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(a.wrapping_mul(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_div_s(
    stack: &mut (impl StackBehavior + ?Sized),
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
                return Err(Error::InvalidOperation {
                    message: "Integer overflow in i32.div_s".to_string(),
                });
            }
            stack.push(Value::I32(a.wrapping_div(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_div_u(
    stack: &mut (impl StackBehavior + ?Sized),
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

pub fn i32_rem_s(
    stack: &mut (impl StackBehavior + ?Sized),
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

pub fn i32_rem_u(
    stack: &mut (impl StackBehavior + ?Sized),
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

pub fn i32_and(
    stack: &mut (impl StackBehavior + ?Sized),
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

pub fn i32_or(
    stack: &mut (impl StackBehavior + ?Sized),
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

pub fn i32_xor(
    stack: &mut (impl StackBehavior + ?Sized),
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

pub fn i32_shl(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(a.wrapping_shl(b as u32)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_shr_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(a.wrapping_shr(b as u32)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_shr_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            stack.push(Value::I32((a.wrapping_shr(b as u32)) as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_rotl(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let b = b as u32;
            let b = b % 32;
            stack.push(Value::I32(a.rotate_left(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_rotr(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let b = b as u32;
            let b = b % 32;
            stack.push(Value::I32(a.rotate_right(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_eqz(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::I32(i32::from(a == 0)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_eq(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a == b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_ne(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a != b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_lt_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_lt_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_gt_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_gt_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_le_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_le_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_ge_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i32_ge_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I32(a), Value::I32(b)) => {
            let a = a as u32;
            let b = b as u32;
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

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
                return Err(Error::InvalidOperation {
                    message: "Integer overflow in i64.div_s".to_string(),
                });
            }
            stack.push(Value::I64(a.wrapping_div(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

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

pub fn i64_shl(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I64(a.wrapping_shl(b as u32)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_shr_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I64(a.wrapping_shr(b as u32)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_shr_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let a = a as u64;
            stack.push(Value::I64((a.wrapping_shr(b as u32)) as i64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_rotl(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let b = b as u32;
            let b = b % 64;
            stack.push(Value::I64(a.rotate_left(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_rotr(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let b = b as u32;
            let b = b % 64;
            stack.push(Value::I64(a.rotate_right(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_clz(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I64(a) => {
            stack.push(Value::I64(i64::from(a.leading_zeros())))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_ctz(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I64(a) => {
            stack.push(Value::I64(i64::from(a.trailing_zeros())))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_popcnt(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I64(a) => {
            stack.push(Value::I64(i64::from(a.count_ones())))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_eqz(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I64(a) => {
            stack.push(Value::I32(i32::from(a == 0)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_eq(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a == b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_ne(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a != b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_lt_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_lt_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let a = a as u64;
            let b = b as u64;
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_gt_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_gt_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let a = a as u64;
            let b = b as u64;
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_le_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_le_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let a = a as u64;
            let b = b as u64;
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_ge_s(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i64_ge_u(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => {
            let a = a as u64;
            let b = b as u64;
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

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

/// Execute f32 absolute value instruction
///
/// Computes the absolute value of the f32 on top of the stack.
pub fn f32_abs(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let value = match stack.pop()? {
        Value::F32(val) => val.abs(),
        _ => return Err(Error::InvalidType("Expected f32 value".to_string())),
    };

    stack.push(Value::F32(value))?;
    Ok(())
}

/// Execute f32 negation instruction
///
/// Negates the f32 value on top of the stack.
pub fn f32_neg(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let value = match stack.pop()? {
        Value::F32(val) => -val,
        _ => return Err(Error::InvalidType("Expected f32 value".to_string())),
    };

    stack.push(Value::F32(value))?;
    Ok(())
}

/// Execute f32 ceiling instruction
///
/// Computes the smallest integer value not less than the input.
pub fn f32_ceil(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let value = match stack.pop()? {
        Value::F32(val) => val.ceil(),
        _ => return Err(Error::InvalidType("Expected f32 value".to_string())),
    };

    stack.push(Value::F32(value))?;
    Ok(())
}

/// Execute f32 floor instruction
///
/// Computes the largest integer value not greater than the input.
pub fn f32_floor(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let value = match stack.pop()? {
        Value::F32(val) => val.floor(),
        _ => return Err(Error::InvalidType("Expected f32 value".to_string())),
    };

    stack.push(Value::F32(value))?;
    Ok(())
}

/// Execute f32 truncate instruction
///
/// Truncates the fractional part of the value.
pub fn f32_trunc(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let value = match stack.pop()? {
        Value::F32(val) => val.trunc(),
        _ => return Err(Error::InvalidType("Expected f32 value".to_string())),
    };

    stack.push(Value::F32(value))?;
    Ok(())
}

/// Execute f32 nearest instruction
///
/// Rounds to the nearest integer, with ties going to the even number.
pub fn f32_nearest(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let value = match stack.pop()? {
        Value::F32(val) => {
            let floor = val.floor();
            let ceil = val.ceil();
            let floor_diff = (val - floor).abs();
            let ceil_diff = (ceil - val).abs();

            if floor_diff < ceil_diff {
                floor
            } else if ceil_diff < floor_diff {
                ceil
            } else {
                // Ties to even
                if floor % 2.0 == 0.0 {
                    floor
                } else {
                    ceil
                }
            }
        }
        _ => return Err(Error::InvalidType("Expected f32 value".to_string())),
    };

    stack.push(Value::F32(value))?;
    Ok(())
}

/// Execute f32 square root instruction
///
/// Computes the square root of the value.
pub fn f32_sqrt(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let value = match stack.pop()? {
        Value::F32(val) => val.sqrt(),
        _ => return Err(Error::InvalidType("Expected f32 value".to_string())),
    };

    stack.push(Value::F32(value))?;
    Ok(())
}

pub fn f32_eq(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a == b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn f32_ne(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a != b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn f32_lt(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn f32_gt(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn f32_le(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn f32_ge(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn f32_copysign(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F32(a), Value::F32(b)) => {
            stack.push(Value::F32(a.copysign(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

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

pub fn f64_div(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::F64(a / b))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

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

pub fn f64_ceil(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::F64(a.ceil()))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

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

pub fn f64_eq(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a == b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn f64_ne(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a != b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn f64_lt(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a < b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn f64_gt(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a > b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn f64_le(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a <= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn f64_ge(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::I32(i32::from(a >= b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn f64_copysign(
    stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let b = stack.pop()?;
    let a = stack.pop()?;
    match (a, b) {
        (Value::F64(a), Value::F64(b)) => {
            stack.push(Value::F64(a.copysign(b)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn i32_wrap_i64(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I64(a) => {
            stack.push(Value::I32(a as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn i32_trunc_f32_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::I32(a as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn i32_trunc_f32_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::I32(a as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn i32_trunc_f64_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::I32(a as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn i32_trunc_f64_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::I32((a as i64) as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn i64_extend_i32_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::I64(i64::from(a)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i64_extend_i32_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::I64(i64::from(a)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn i64_trunc_f32_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::I64(a as i64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn i64_trunc_f32_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::I64(a as i64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn i64_trunc_f64_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::I64(a as i64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn i64_trunc_f64_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::I64(a as i64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn f32_convert_i32_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::F32(a as f32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn f32_convert_i32_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::F32(a as f32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn f32_convert_i64_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I64(a) => {
            stack.push(Value::F32(a as f32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn f32_convert_i64_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I64(a) => {
            stack.push(Value::F32(a as f32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn f32_demote_f64(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::F32(a as f32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn f64_convert_i32_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::F64(f64::from(a)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn f64_convert_i32_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::F64(f64::from(a)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn f64_convert_i64_s(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I64(a) => {
            stack.push(Value::F64(a as f64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn f64_convert_i64_u(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I64(a) => {
            stack.push(Value::F64(a as f64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}

pub fn f64_promote_f32(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::F64(f64::from(a)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn i32_reinterpret_f32(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F32(a) => {
            stack.push(Value::I32(a as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f32".to_string())),
    }
}

pub fn i64_reinterpret_f64(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::F64(a) => {
            stack.push(Value::I64(a as i64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected f64".to_string())),
    }
}

pub fn f32_reinterpret_i32(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I32(a) => {
            stack.push(Value::F32(a as f32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn f64_reinterpret_i64(
    stack: &mut (impl StackBehavior + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let a = stack.pop()?;
    match a {
        Value::I64(a) => {
            stack.push(Value::F64(a as f64))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i64".to_string())),
    }
}
