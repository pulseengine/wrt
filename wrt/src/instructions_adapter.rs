// Copyright 2024 The WRT Project Authors

//! Instructions adapter for wrt
//!
//! This adapter bridges between `wrt` execution context and `wrt-instructions` execution context.
//! It provides the necessary adapters to use pure implementations from `wrt-instructions`
//! in the `wrt` runtime.

use crate::{
    behavior::{ControlFlow, FrameBehavior, StackBehavior},
    error::{kinds, Error, Result},
    stackless::StacklessEngine,
    values::Value,
};

use wrt_instructions::{execution::PureExecutionContext, instruction_traits::PureInstruction};

/// Execution context adapter for pure instructions
pub struct WrtExecutionContextAdapter<'a> {
    stack: &'a mut dyn StackBehavior,
    frame: &'a mut dyn FrameBehavior,
    engine: &'a mut StacklessEngine,
}

impl<'a> WrtExecutionContextAdapter<'a> {
    /// Create a new execution context adapter
    pub fn new(
        stack: &'a mut dyn StackBehavior,
        frame: &'a mut dyn FrameBehavior,
        engine: &'a mut StacklessEngine,
    ) -> Self {
        Self {
            stack,
            frame,
            engine,
        }
    }
}

impl<'a> PureExecutionContext for WrtExecutionContextAdapter<'a> {
    fn push_value(&mut self, value: Value) -> wrt_instructions::Result<()> {
        self.stack
            .push(value)
            .map_err(|e| wrt_instructions::Error::from(e))
    }

    fn pop_value(&mut self) -> wrt_instructions::Result<Value> {
        self.stack
            .pop()
            .map_err(|e| wrt_instructions::Error::from(e))
    }

    fn pop_value_expected(
        &mut self,
        expected_type: wrt_instructions::ValueType,
    ) -> wrt_instructions::Result<Value> {
        let value = self.pop_value()?;
        if value.value_type() != expected_type {
            return Err(wrt_instructions::Error::new(
                wrt_error::kinds::TypeMismatch(format!(
                    "Expected {:?}, got {:?}",
                    expected_type,
                    value.value_type()
                )),
            ));
        }
        Ok(value)
    }
}

/// Execute a pure instruction using the wrt runtime
pub fn execute_pure_instruction<'a, T>(
    instruction: &T,
    stack: &'a mut dyn StackBehavior,
    frame: &'a mut dyn FrameBehavior,
    engine: &'a mut StacklessEngine,
) -> Result<ControlFlow, Error>
where
    T: PureInstruction<WrtExecutionContextAdapter<'a>, wrt_error::Error>,
{
    // Create an adapter for the execution context
    let mut context_adapter = WrtExecutionContextAdapter::new(stack, frame, engine);

    // Execute the pure instruction
    instruction
        .execute(&mut context_adapter)
        .map_err(|e| Error::from(e))?;

    // Return continue by default, as pure instructions
    // do not handle control flow directly
    Ok(ControlFlow::Continue)
}

// Additional adapter traits can be added here as needed:

// Memory adapter for pure memory instructions
// Table adapter for pure table instructions
// etc.
