use crate::behavior;
use crate::error::{Error, Result};
use crate::values::Value as ValuesValue;

#[derive(Debug, Clone)]
pub struct Label {
    pub arity: usize,
    pub pc: usize,
    pub continuation: usize,
}

impl From<behavior::Label> for Label {
    fn from(label: behavior::Label) -> Self {
        Self {
            arity: label.arity,
            pc: label.pc,
            continuation: label.continuation,
        }
    }
}

/// Trait for stack operations
pub trait Stack: behavior::StackBehavior + std::fmt::Debug {
    fn push_label(&mut self, label: Label) -> Result<()>;
    fn pop_label(&mut self) -> Result<Label>;
    fn get_label(&self, idx: usize) -> Result<&Label>;
    fn get_label_mut(&mut self, idx: usize) -> Result<&mut Label>;
    fn labels_len(&self) -> usize;
}

impl Stack for Vec<ValuesValue> {
    fn push_label(&mut self, _label: Label) -> Result<()> {
        Ok(())
    }

    fn pop_label(&mut self) -> Result<Label> {
        Err(Error::InvalidOperation {
            message: "No labels in Vec<Value>".to_string(),
        })
    }

    fn get_label(&self, _idx: usize) -> Result<&Label> {
        Err(Error::InvalidOperation {
            message: "No labels in Vec<Value>".to_string(),
        })
    }

    fn get_label_mut(&mut self, _idx: usize) -> Result<&mut Label> {
        Err(Error::InvalidOperation {
            message: "No labels in Vec<Value>".to_string(),
        })
    }

    fn labels_len(&self) -> usize {
        0
    }
}

impl behavior::StackBehavior for Vec<ValuesValue> {
    fn push(&mut self, value: ValuesValue) -> Result<()> {
        self.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<ValuesValue> {
        self.pop().ok_or(Error::StackUnderflow)
    }

    fn peek(&self) -> Result<&ValuesValue> {
        self.last().ok_or(Error::StackUnderflow)
    }

    fn peek_mut(&mut self) -> Result<&mut ValuesValue> {
        self.last_mut().ok_or(Error::StackUnderflow)
    }

    fn values(&self) -> &[ValuesValue] {
        self
    }

    fn values_mut(&mut self) -> &mut [ValuesValue] {
        self
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn push_label(&mut self, _arity: usize, _pc: usize) {
        // No-op for Vec<ValuesValue>
    }

    fn pop_label(&mut self) -> Result<behavior::Label> {
        Err(Error::InvalidOperation {
            message: "No labels in Vec<Value>".to_string(),
        })
    }

    fn get_label(&self, _idx: usize) -> Option<&behavior::Label> {
        None
    }
}
