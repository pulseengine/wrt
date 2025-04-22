use crate::{
    behavior::{ControlFlow, FrameBehavior, InstructionExecutor, StackBehavior},
    error::{kinds, Error, Result},
    stackless::StacklessEngine,
    types::*,
    Value,
};

// Import RefType from wrt_types
use wrt_types::types::RefType;

/// Represents a ref.null instruction which creates a null reference of the given reference type
#[derive(Debug, Clone, Copy)]
pub struct RefNull {
    ref_type: RefType,
}

impl RefNull {
    pub fn new(ref_type: RefType) -> Self {
        Self { ref_type }
    }
}

impl InstructionExecutor for RefNull {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow, Error> {
        // Create a null reference of the specified type
        let value = match self.ref_type {
            RefType::Funcref => Value::FuncRef(None),
            RefType::Externref => Value::ExternRef(None),
            _ => {
                return Err(Error::new(kinds::InvalidTypeError(format!(
                    "Invalid reference type for ref.null: {:?}",
                    self.ref_type
                ))))
            }
        };

        // Push the null reference onto the stack
        stack.push(value)?;

        Ok(ControlFlow::Continue)
    }
}

/// Represents a ref.func instruction which creates a reference to a function
#[derive(Debug, Clone, Copy)]
pub struct RefFunc {
    func_idx: u32,
}

impl RefFunc {
    pub fn new(func_idx: u32) -> Self {
        Self { func_idx }
    }
}

impl InstructionExecutor for RefFunc {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow, Error> {
        // Create a function reference with the specified index
        let value = Value::FuncRef(Some(self.func_idx));

        // Push the function reference onto the stack
        stack.push(value)?;

        Ok(ControlFlow::Continue)
    }
}

/// Represents a ref.is_null instruction which checks if a reference is null
#[derive(Debug, Clone, Copy)]
pub struct RefIsNull;

impl InstructionExecutor for RefIsNull {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow, Error> {
        // Pop a reference value from the stack
        let value = stack.pop()?;

        // Check if it's null
        let is_null = match value {
            Value::FuncRef(None) | Value::ExternRef(None) => true,
            Value::FuncRef(Some(_)) | Value::ExternRef(Some(_)) => false,
            _ => {
                return Err(Error::new(kinds::InvalidTypeError(format!(
                    "Expected reference type for ref.is_null, got: {:?}",
                    value
                ))))
            }
        };

        // Push the result as an i32 (0 or 1)
        stack.push(Value::I32(if is_null { 1 } else { 0 }))?;

        Ok(ControlFlow::Continue)
    }
}
