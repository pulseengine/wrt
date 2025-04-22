//! Module for WebAssembly linear memory
//!
//! This module provides memory types and re-exports for WebAssembly memory.

use wrt_error::Result;

// Re-export memory types from wrt-runtime
pub use wrt_runtime::{Memory, MemoryType, PAGE_SIZE};

// Re-export the memory operations from wrt-instructions
#[cfg(feature = "std")]
pub use wrt_instructions::memory_ops::{MemoryLoad, MemoryStore};

/// Maximum number of memory pages allowed by WebAssembly spec
pub const MAX_PAGES: u32 = 65536;

/// Create a new memory instance
///
/// This is a convenience function that creates a memory instance
/// with the given type.
///
/// # Arguments
///
/// * `mem_type` - The memory type
///
/// # Returns
///
/// A new memory instance
///
/// # Errors
///
/// Returns an error if the memory cannot be created
pub fn create_memory(mem_type: MemoryType) -> Result<Memory> {
    Memory::new(mem_type)
}

/// Create a new memory instance with a name
///
/// This is a convenience function that creates a memory instance
/// with the given type and name.
///
/// # Arguments
///
/// * `mem_type` - The memory type
/// * `name` - The debug name for the memory
///
/// # Returns
///
/// A new memory instance
///
/// # Errors
///
/// Returns an error if the memory cannot be created
pub fn create_memory_with_name(mem_type: MemoryType, name: &str) -> Result<Memory> {
    Memory::new_with_name(mem_type, name)
}
