//! Instance implementation for the WebAssembly Component Model.
//!
//! This module provides the instance types for component instances.

#[cfg(not(feature = "std"))]
use alloc::{
    string::String,
    vec::Vec,
};
#[cfg(feature = "std")]
use std::{
    string::String,
    vec::Vec,
};

use wrt_format::component::ComponentTypeDefinition;

use crate::export::Export;

// Type alias for compatibility
pub type Instance = InstanceValue;

/// Represents an instance value in a component
#[derive(Debug, Clone)]
pub struct InstanceValue {
    /// The name of the instance
    pub name:    String,
    /// Instance type
    pub ty:      ComponentTypeDefinition,
    /// Instance exports
    pub exports: Vec<Export>,
}

impl InstanceValue {
    /// Creates a new instance value
    pub fn new(name: String, ty: ComponentTypeDefinition, exports: Vec<Export>) -> Self {
        Self { name, ty, exports }
    }

    /// Gets an export by name
    pub fn get_export(&self, name: &str) -> Option<&Export> {
        self.exports.iter().find(|export| export.name == name)
    }

    /// Gets a mutable export by name
    pub fn get_export_mut(&mut self, name: &str) -> Option<&mut Export> {
        self.exports.iter_mut().find(|export| export.name == name)
    }
}
