//! WebAssembly Component Model Canonical ABI
//!
//! This module provides implementations of the Canonical ABI for the
//! WebAssembly Component Model, including lifting, lowering, and memory
//! allocation functions.

pub mod canonical;
pub mod canonical_abi;
pub mod canonical_options;
pub mod canonical_realloc;
pub mod post_return;

// Re-export from canonical module (primary implementation)
pub use canonical::*;
// Re-export key types from canonical_abi module for consistent access
pub use canonical_abi::{
    CanonicalMemory,
    ComponentType,
    ComponentValue,
};
pub use canonical_options::*;
pub use canonical_realloc::{
    CanonicalOptionsWithRealloc,
    MemoryLayout,
    ReallocFn,
    ReallocManager,
    StringEncoding,
};
pub use post_return::*;

// Placeholder types for async canonical ABI support
#[derive(Debug, Clone)]
pub struct LiftingContext {
    pub options: canonical_options::CanonicalOptions,
}

impl LiftingContext {
    pub fn new(options: canonical_options::CanonicalOptions) -> Self {
        Self { options }
    }
}

#[derive(Debug, Clone)]
pub struct LoweringContext {
    pub options: canonical_options::CanonicalOptions,
}

impl LoweringContext {
    pub fn new(options: canonical_options::CanonicalOptions) -> Self {
        Self { options }
    }

}
