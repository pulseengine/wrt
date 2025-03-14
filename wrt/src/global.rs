use crate::error::{Error, Result};
use crate::types::*;
use crate::values::Value;
use crate::{format, Vec};

/// Represents a WebAssembly global instance
#[derive(Debug)]
pub struct Global {
    /// Global type
    global_type: GlobalType,
    /// Global value
    value: Value,
}

impl Global {
    /// Creates a new global instance
    pub fn new(global_type: GlobalType, value: Value) -> Result<Self> {
        // Check that the value matches the global type
        if !value.matches_type(&global_type.content_type) {
            return Err(Error::Execution(format!(
                "Value type {:?} does not match global type {:?}",
                value.type_(),
                global_type.content_type
            )));
        }

        Ok(Self { global_type, value })
    }

    /// Returns the global type
    pub fn type_(&self) -> &GlobalType {
        &self.global_type
    }

    /// Gets the global value
    pub fn get(&self) -> Value {
        self.value.clone()
    }

    /// Sets the global value
    pub fn set(&mut self, value: Value) -> Result<()> {
        // Check mutability
        if !self.global_type.mutable {
            return Err(Error::Execution("Cannot set immutable global".into()));
        }

        // Check value type
        if !value.matches_type(&self.global_type.content_type) {
            return Err(Error::Execution(format!(
                "Value type {:?} does not match global type {:?}",
                value.type_(),
                self.global_type.content_type
            )));
        }

        self.value = value;
        Ok(())
    }
}

/// Represents a collection of global instances
#[derive(Debug)]
pub struct Globals {
    /// Global instances
    globals: Vec<Global>,
}

impl Default for Globals {
    fn default() -> Self {
        Self::new()
    }
}

impl Globals {
    /// Creates a new empty globals collection
    pub fn new() -> Self {
        Self {
            globals: Vec::new(),
        }
    }

    /// Adds a new global instance
    pub fn add(&mut self, global: Global) -> u32 {
        let idx = self.globals.len() as u32;
        self.globals.push(global);
        idx
    }

    /// Gets a global instance by index
    pub fn get(&self, idx: u32) -> Result<&Global> {
        self.globals
            .get(idx as usize)
            .ok_or_else(|| Error::Execution(format!("Global index {} out of bounds", idx)))
    }

    /// Gets a mutable reference to a global instance by index
    pub fn get_mut(&mut self, idx: u32) -> Result<&mut Global> {
        self.globals
            .get_mut(idx as usize)
            .ok_or_else(|| Error::Execution(format!("Global index {} out of bounds", idx)))
    }

    /// Returns the number of global instances
    pub fn len(&self) -> u32 {
        self.globals.len() as u32
    }

    /// Returns whether the globals collection is empty
    pub fn is_empty(&self) -> bool {
        self.globals.is_empty()
    }
}
