//! Stackless WebAssembly execution engine
//!
//! This module implements a stackless version of the WebAssembly execution
//! engine that doesn't rely on the host language's call stack, making it
//! suitable for environments with limited stack space and for no_std contexts.
//!
//! The stackless engine uses a state machine approach to track execution state
//! and allows for pausing and resuming execution at any point.

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;
#[cfg(not(any(feature = "std", feature = "alloc")))]
type String =
    wrt_foundation::bounded::BoundedString<256, wrt_foundation::safe_memory::NoStdProvider<512>>;

pub mod engine;
pub mod extensions;
pub mod frame;

#[cfg(feature = "std")]
pub mod tail_call;

#[cfg(test)]
mod engine_tests;

pub use engine::{
    StacklessCallbackRegistry,
    StacklessEngine,
    StacklessStack,
};

// Re-export ExecutionResult from cfi_engine to avoid conflicts
pub use crate::cfi_engine::ExecutionResult;

// Define types that may be needed by other modules
#[derive(Debug, Clone, PartialEq)]
pub struct YieldInfo {
    pub instruction_pointer: u32,
    pub yield_reason:        String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EngineState {
    pub instruction_pointer: u32,
    pub fuel:                u64,
    pub current_instance_id: Option<usize>,
}

pub type StacklessExecutionState = EngineState;
pub use frame::StacklessFrame;
