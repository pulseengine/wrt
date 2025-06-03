//! Asynchronous Component Model implementation
//!
//! This module contains all async-related functionality for the WebAssembly
//! Component Model, including async runtimes, execution engines, and async
//! canonical ABI implementations.

pub mod async_canonical;
pub mod async_canonical_lifting;
pub mod async_context_builtins;
pub mod async_execution_engine;
pub mod async_resource_cleanup;
pub mod async_runtime;
pub mod async_runtime_bridge;
pub mod async_types;

pub use async_canonical::*;
pub use async_canonical_lifting::*;
pub use async_context_builtins::*;
pub use async_execution_engine::*;
pub use async_resource_cleanup::*;
pub use async_runtime::*;
pub use async_runtime_bridge::*;
pub use async_types::*;