//! Resource management test utilities for the WebAssembly Runtime.
//!
//! This crate provides test utilities for resource management features.

#![warn(clippy::missing_panics_doc)]

pub mod buffer_pool;
pub mod memory_manager;
pub mod memory_strategy;
pub mod resource_manager;

pub use buffer_pool::BufferPool;
pub use memory_manager::{MemoryManager, ComponentValue};
pub use memory_strategy::{MemoryStrategy, ResourceOperation, ResourceStrategy};
pub use resource_manager::{ResourceId, ResourceManager, HostResource}; 