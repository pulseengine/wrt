//! Module for WebAssembly linear memory
//!
//! This module provides memory types and re-exports for WebAssembly memory.
//!
//! # Safety Features
//!
//! The memory implementation includes several safety features:
//!
//! - Checksum verification for data integrity
//! - Bounds checking for all memory operations
//! - Alignment validation
//! - Thread safety guarantees
//! - Memory access tracking
//!
//! # Usage
//!
//! ```no_run
//! use wrt::{Memory, MemoryType};
//! use wrt_types::types::Limits;
//!
//! // Create a memory type with initial 1 page (64KB) and max 2 pages
//! let mem_type = MemoryType {
//!     limits: Limits { min: 1, max: Some(2) },
//! };
//!
//! // Create a new memory instance
//! let mut memory = create_memory(mem_type).unwrap();
//!
//! // Write data to memory
//! memory.write(0, &[1, 2, 3, 4]).unwrap();
//!
//! // Read data from memory
//! let mut buffer = [0; 4];
//! memory.read(0, &mut buffer).unwrap();
//! assert_eq!(buffer, [1, 2, 3, 4]);
//! ```

use wrt_error::Result;
use wrt_types::safe_memory::{MemoryProvider, MemorySafety, SafeSlice, VerificationLevel};

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

/// Create a new memory instance with a specific verification level
///
/// This is a convenience function that creates a memory instance
/// with the given type and verification level.
///
/// # Arguments
///
/// * `mem_type` - The memory type
/// * `level` - The verification level
///
/// # Returns
///
/// A new memory instance
///
/// # Errors
///
/// Returns an error if the memory cannot be created
pub fn create_memory_with_verification(
    mem_type: MemoryType,
    level: VerificationLevel,
) -> Result<Memory> {
    let mut memory = Memory::new(mem_type)?;
    memory.set_verification_level(level);
    Ok(memory)
}

/// Get a safe slice of memory with integrity verification
///
/// This is a convenience function that gets a safe slice of memory
/// with the given offset and length.
///
/// # Arguments
///
/// * `memory` - The memory instance
/// * `offset` - The offset in bytes
/// * `len` - The length in bytes
///
/// # Returns
///
/// A safe slice with integrity verification
///
/// # Errors
///
/// Returns an error if the slice would be invalid
pub fn get_safe_slice(memory: &Memory, offset: usize, len: usize) -> Result<SafeSlice<'_>> {
    memory.borrow_slice(offset, len)
}

/// Verify the integrity of a memory instance
///
/// This is a convenience function that verifies the integrity
/// of a memory instance.
///
/// # Arguments
///
/// * `memory` - The memory instance
///
/// # Errors
///
/// Returns an error if memory corruption is detected
pub fn verify_memory_integrity(memory: &Memory) -> Result<()> {
    memory.verify_integrity()
}

/// Get memory statistics
///
/// This is a convenience function that gets statistics about
/// a memory instance.
///
/// # Arguments
///
/// * `memory` - The memory instance
///
/// # Returns
///
/// Memory statistics
pub fn get_memory_stats(memory: &Memory) -> wrt_types::safe_memory::MemoryStats {
    memory.memory_stats()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_types::types::Limits;

    #[test]
    fn test_create_memory() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let memory = create_memory(mem_type).unwrap();
        assert_eq!(memory.size(), 1);
        assert_eq!(memory.size_in_bytes(), PAGE_SIZE);
    }

    #[test]
    fn test_create_memory_with_name() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let memory = create_memory_with_name(mem_type, "test").unwrap();
        assert_eq!(memory.debug_name(), Some("test"));
    }

    #[test]
    fn test_create_memory_with_verification() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let level = VerificationLevel::High;
        let memory = create_memory_with_verification(mem_type, level).unwrap();
        assert_eq!(memory.verification_level(), level);
    }

    #[test]
    fn test_get_safe_slice() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory = create_memory(mem_type).unwrap();
        let data = [1, 2, 3, 4];
        memory.write(0, &data).unwrap();
        let slice = get_safe_slice(&memory, 0, 4).unwrap();
        assert_eq!(slice.data().unwrap(), &data);
    }

    #[test]
    fn test_verify_memory_integrity() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let memory = create_memory(mem_type).unwrap();
        verify_memory_integrity(&memory).unwrap();
    }

    #[test]
    fn test_get_memory_stats() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory = create_memory(mem_type).unwrap();
        let data = [1, 2, 3, 4];
        memory.write(0, &data).unwrap();
        let stats = get_memory_stats(&memory);
        assert_eq!(stats.total_size, PAGE_SIZE);
        assert_eq!(stats.access_count, 1);
    }
}
