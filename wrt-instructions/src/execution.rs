//! Pure execution context for WebAssembly instructions.
//!
//! This module provides a minimal execution context for pure instruction implementations.
//! It defines interfaces that can be implemented by different execution engines.

use crate::{
    arithmetic_ops::ArithmeticContext, comparison_ops::ComparisonContext, Result, Value, ValueType,
};

// Imports for alloc
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{format, vec::Vec};

// Imports for std
#[cfg(feature = "std")]
use std::{format, vec::Vec};

/// A trait defining a minimal execution context for pure instructions.
///
/// This trait provides the minimal interface required for executing pure instructions.
/// It is designed to be implemented by different execution engines according to their needs.
pub trait PureExecutionContext {
    /// Pushes a value onto the stack.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to push
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the value was pushed successfully
    /// * `Err(Error)` - If an error occurred
    fn push_value(&mut self, value: Value) -> Result<()>;

    /// Pops a value from the stack.
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - The popped value
    /// * `Err(Error)` - If an error occurred (e.g., stack underflow)
    fn pop_value(&mut self) -> Result<Value>;

    /// Pops a value of the expected type from the stack.
    ///
    /// # Arguments
    ///
    /// * `expected_type` - The expected type of the value
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - The popped value
    /// * `Err(Error)` - If an error occurred (e.g., stack underflow or type mismatch)
    fn pop_value_expected(&mut self, expected_type: ValueType) -> Result<Value>;
}

/// A general-purpose execution context for pure instructions
#[derive(Default)]
pub struct ExecutionContext {
    stack: Vec<Value>,
}

impl ExecutionContext {
    /// Creates a new ExecutionContext
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Returns the current stack
    pub fn stack(&self) -> &[Value] {
        &self.stack
    }
}

impl PureExecutionContext for ExecutionContext {
    fn push_value(&mut self, value: Value) -> Result<()> {
        self.stack.push(value);
        Ok(())
    }

    fn pop_value(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .ok_or_else(|| crate::Error::from(wrt_error::kinds::stack_underflow()))
    }

    fn pop_value_expected(&mut self, expected_type: ValueType) -> Result<Value> {
        let value = PureExecutionContext::pop_value(self)?;
        if value.value_type() != expected_type {
            return Err(crate::Error::from(wrt_error::kinds::type_mismatch(
                format!("Expected {expected_type:?}, got {:?}", value.value_type()),
            )));
        }
        Ok(value)
    }
}

impl ArithmeticContext for ExecutionContext {
    fn push_arithmetic_value(&mut self, value: Value) -> Result<()> {
        PureExecutionContext::push_value(self, value)
    }

    fn pop_arithmetic_value(&mut self) -> Result<Value> {
        PureExecutionContext::pop_value(self)
    }
}

impl ComparisonContext for ExecutionContext {
    fn pop_comparison_value(&mut self) -> Result<Value> {
        PureExecutionContext::pop_value(self)
    }

    fn push_comparison_value(&mut self, value: Value) -> Result<()> {
        PureExecutionContext::push_value(self, value)
    }
}

/// A minimal implementation of the PureExecutionContext for testing.
///
/// This implementation is used for testing pure instruction implementations.
#[cfg(test)]
pub struct TestExecutionContext {
    stack: Vec<Value>,
}

#[cfg(test)]
impl Default for TestExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl TestExecutionContext {
    /// Creates a new TestExecutionContext.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Returns the current stack.
    pub fn stack(&self) -> &[Value] {
        &self.stack
    }
}

#[cfg(test)]
impl PureExecutionContext for TestExecutionContext {
    fn push_value(&mut self, value: Value) -> Result<()> {
        self.stack.push(value);
        Ok(())
    }

    fn pop_value(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .ok_or_else(|| crate::Error::from(wrt_error::kinds::stack_underflow()))
    }

    fn pop_value_expected(&mut self, expected_type: ValueType) -> Result<Value> {
        let value = PureExecutionContext::pop_value(self)?;
        if value.value_type() != expected_type {
            return Err(crate::Error::from(wrt_error::kinds::type_mismatch(
                format!("Expected {expected_type:?}, got {:?}", value.value_type()),
            )));
        }
        Ok(value)
    }
}

#[cfg(test)]
impl ArithmeticContext for TestExecutionContext {
    fn push_arithmetic_value(&mut self, value: Value) -> Result<()> {
        PureExecutionContext::push_value(self, value)
    }

    fn pop_arithmetic_value(&mut self) -> Result<Value> {
        PureExecutionContext::pop_value(self)
    }
}

#[cfg(test)]
impl ComparisonContext for TestExecutionContext {
    fn pop_comparison_value(&mut self) -> Result<Value> {
        PureExecutionContext::pop_value(self)
    }

    fn push_comparison_value(&mut self, value: Value) -> Result<()> {
        PureExecutionContext::push_value(self, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context() {
        let mut context = TestExecutionContext::new();

        // Test pushing and popping values
        PureExecutionContext::push_value(&mut context, Value::I32(42)).unwrap();
        assert_eq!(context.stack(), &[Value::I32(42)]);

        let value = PureExecutionContext::pop_value(&mut context).unwrap();
        assert_eq!(value, Value::I32(42));
        assert!(context.stack().is_empty());

        // Test pop with empty stack
        assert!(PureExecutionContext::pop_value(&mut context).is_err());

        // Test pop_value_expected
        PureExecutionContext::push_value(&mut context, Value::I32(42)).unwrap();
        let value = context.pop_value_expected(ValueType::I32).unwrap();
        assert_eq!(value, Value::I32(42));

        // Test pop_value_expected with type mismatch
        PureExecutionContext::push_value(&mut context, Value::I32(42)).unwrap();
        assert!(context.pop_value_expected(ValueType::I64).is_err());
    }
}
