//! WebAssembly function type implementation
//!
//! This module provides the implementation for WebAssembly function types.

use crate::prelude::{Debug, DefaultProvider, FuncType};

/// Placeholder Function type for runtime functions
#[derive(Debug, Clone)]
pub struct Function {
    /// Function type signature
    pub func_type: FuncType,
    /// Function body (placeholder)
    pub body: wrt_foundation::bounded::BoundedVec<u8, 4096, DefaultProvider>,
}

impl Function {
    /// Create a new function
    pub fn new(func_type: FuncType) -> Self {
        Self {
            func_type,
            body: wrt_foundation::bounded::BoundedVec::new(DefaultProvider::default()).unwrap(),
        }
    }
}
