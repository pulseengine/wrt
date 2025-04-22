use crate::behavior::{self, Label as BehaviorLabel, StackBehavior};
use wrt_error::{Error, Result, kinds};
use crate::values::Value;
use crate::StacklessEngine;
use std::vec::Vec;

/// Represents a control flow label on the stack (e.g., for blocks, loops, ifs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    /// The number of values the instruction sequence associated with the label is expected to produce.
    pub arity: usize,
    /// The program counter (instruction index) where execution should resume after the block.
    pub pc: usize,
    /// The program counter for the continuation (e.g., the `else` branch of an `if`).
    pub continuation: usize,
    /// The depth of the value stack when this label was pushed (used for stack cleanup on branch).
    pub stack_depth: usize,
    /// Indicates if this label represents a loop (for `br` targeting).
    pub is_loop: bool,
    /// Indicates if this label represents an if block (for `else` handling).
    pub is_if: bool,
}

impl From<BehaviorLabel> for Label {
    fn from(label: BehaviorLabel) -> Self {
        Self {
            arity: label.arity,
            pc: label.pc,
            continuation: label.continuation,
            stack_depth: label.stack_depth,
            is_loop: label.is_loop,
            is_if: label.is_if,
        }
    }
}

impl From<Label> for BehaviorLabel {
    fn from(label: Label) -> Self {
        Self {
            arity: label.arity,
            pc: label.pc,
            continuation: label.continuation,
            stack_depth: label.stack_depth,
            is_loop: label.is_loop,
            is_if: label.is_if,
        }
    }
}

impl StackBehavior for Vec<Value> {
    fn push(&mut self, value: Value) -> Result<(), Error> {
        self.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<Value, Error> {
        self.pop().ok_or_else(|| Error::new(kinds::StackUnderflowError))
    }

    fn peek(&self) -> Result<&Value, Error> {
        self.last().ok_or_else(|| Error::new(kinds::StackUnderflowError))
    }

    fn peek_mut(&mut self) -> Result<&mut Value, Error> {
        self.last_mut().ok_or_else(|| Error::new(kinds::StackUnderflowError))
    }

    fn values(&self) -> &[Value] {
        self.as_slice()
    }

    fn values_mut(&mut self) -> &mut [Value] {
        self.as_mut_slice()
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    fn push_label(&mut self, label: BehaviorLabel) -> Result<(), Error> {
        // Stack doesn't track labels directly - unsupported operation
        Err(Error::new(kinds::NotImplementedError("push_label not supported on raw stack".to_string())))
    }

    fn pop_label(&mut self) -> Result<BehaviorLabel, Error> {
        // Stack doesn't track labels directly - unsupported operation
        Err(Error::new(kinds::NotImplementedError("pop_label not supported on raw stack".to_string())))
    }

    fn get_label(&self, _index: usize) -> Option<&BehaviorLabel> {
        // Stack doesn't track labels directly
        None
    }

    fn push_n(&mut self, values: &[Value]) {
        self.extend_from_slice(values);
    }

    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        if self.len() < n {
            // Since we can't return an error, return empty vec on underflow
            return Vec::new();
        }
        let new_len = self.len() - n;
        let result = self.split_off(new_len);
        result
    }

    fn pop_frame_label(&mut self) -> Result<BehaviorLabel, Error> {
        // Stack doesn't track frame labels directly - unsupported operation
        Err(Error::new(kinds::NotImplementedError("pop_frame_label not supported on raw stack".to_string())))
    }

    fn execute_function_call_direct(
        &mut self,
        _engine: &mut StacklessEngine,
        _caller_instance_idx: u32,
        _func_idx: u32,
        _args: Vec<Value>,
    ) -> Result<Vec<Value>, Error> {
        // Raw stack can't execute functions
        Err(Error::new(kinds::NotImplementedError("Function calls not supported on raw stack".to_string())))
    }
}
