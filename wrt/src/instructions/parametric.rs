//! WebAssembly parametric instructions
//!
//! This module contains implementations for all WebAssembly parametric instructions,
//! including operations for stack manipulation and control flow.

use crate::behavior::ControlFlow;
use crate::{
    behavior::{FrameBehavior, InstructionExecutor, StackBehavior},
    error::{kinds, Error, Result},
    stackless::StacklessEngine,
    types::ValueType,
    values::Value,
};
use log::trace;

/// Execute a drop instruction
///
/// Removes the top value from the stack.
#[derive(Debug)]
pub struct Drop;

impl InstructionExecutor for Drop {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let _ = stack.pop()?;
        Ok(ControlFlow::Continue)
    }
}

/// Execute a select instruction
///
/// Selects one of two values based on a condition.
#[derive(Debug)]
pub struct Select;

impl InstructionExecutor for Select {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let c = stack.pop()?.as_i32().ok_or_else(|| {
            Error::new(kinds::InvalidTypeError(
                "Select condition must be i32".to_string(),
            ))
        })?;
        let val2 = stack.pop()?;
        let val1 = stack.pop()?;

        if c != 0 {
            stack.push(val1)?;
        } else {
            stack.push(val2)?;
        }
        Ok(ControlFlow::Continue)
    }
}

/// Execute a `select_typed` instruction
///
/// Selects one of two values based on a condition, with type checking.
#[derive(Debug, Clone)]
pub struct SelectTyped {
    pub types: Vec<ValueType>,
}

impl SelectTyped {
    pub fn new(types: Vec<ValueType>) -> Self {
        Self { types }
    }
}

impl InstructionExecutor for SelectTyped {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let c = stack.pop()?.as_i32().ok_or_else(|| {
            Error::new(kinds::InvalidTypeError(
                "Select condition must be i32".to_string(),
            ))
        })?;
        let val2 = stack.pop()?;
        let val1 = stack.pop()?;

        if !self.types.is_empty() {
            let expected_type = self.types[0];
            if val1.type_() != expected_type || val2.type_() != expected_type {
                return Err(Error::new(kinds::InvalidTypeError(format!(
                    "Type mismatch for select: expected {:?}, got {:?} and {:?}",
                    expected_type,
                    val1.type_(),
                    val2.type_()
                ))));
            }
        }

        if c != 0 {
            stack.push(val1)?;
        } else {
            stack.push(val2)?;
        }
        Ok(ControlFlow::Continue)
    }
}
