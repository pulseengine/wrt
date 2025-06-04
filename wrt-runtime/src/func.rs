//! WebAssembly function type implementation
//!
//! This module provides the implementation for WebAssembly function types.

use crate::prelude::*;

// Re-export FuncType from wrt-foundation
pub use wrt_foundation::types::FuncType;

/// Placeholder Function type for runtime functions
#[derive(Debug, Clone)]
pub struct Function {
    /// Function type signature
    pub func_type: FuncType<wrt_foundation::NoStdProvider<1024>>,
    /// Function body (placeholder)
    pub body: wrt_foundation::bounded::BoundedVec<u8, 4096, wrt_foundation::safe_memory::NoStdProvider<1024>>,
}

impl Function {
    /// Create a new function
    pub fn new(func_type: FuncType<wrt_foundation::NoStdProvider<1024>>) -> Self {
        Self {
            func_type,
            body: wrt_foundation::bounded::BoundedVec::new_with_provider(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
        }
    }
}
