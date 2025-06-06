//! Unified Type System for WRT Runtime
//!
//! This module establishes a consistent type system across all WRT crates
//! to eliminate the 421+ "mismatched types" errors caused by incompatible
//! bounded collection capacities and memory provider hierarchies.

use wrt_foundation::safe_memory::{NoStdProvider, MemoryProvider};
use wrt_foundation::bounded::{BoundedVec, BoundedString};

// =============================================================================
// UNIFIED CAPACITY CONSTANTS
// =============================================================================
// These replace ad-hoc sizing across all crates

/// Small collections (function locals, basic lists)
pub const SMALL_CAPACITY: usize = 64;

/// Medium collections (instructions, imports, exports)  
pub const MEDIUM_CAPACITY: usize = 1024;

/// Large collections (memory buffers, code bodies)
pub const LARGE_CAPACITY: usize = 65536;

/// String capacities
pub const SMALL_STRING_LEN: usize = 128;
pub const MEDIUM_STRING_LEN: usize = 256; 
pub const LARGE_STRING_LEN: usize = 1024;

/// Memory provider sizes
pub const SMALL_PROVIDER_SIZE: usize = 1024;        // 1KB
pub const MEDIUM_PROVIDER_SIZE: usize = 65536;      // 64KB  
pub const LARGE_PROVIDER_SIZE: usize = 1048576;     // 1MB

// =============================================================================
// STANDARDIZED MEMORY PROVIDERS
// =============================================================================

/// Small memory provider for lightweight operations
pub type SmallProvider = NoStdProvider<SMALL_PROVIDER_SIZE>;

/// Medium memory provider for standard operations
pub type MediumProvider = NoStdProvider<MEDIUM_PROVIDER_SIZE>;

/// Large memory provider for heavy operations
pub type LargeProvider = NoStdProvider<LARGE_PROVIDER_SIZE>;

// =============================================================================
// UNIFIED COLLECTION TYPES
// =============================================================================

/// Small bounded vector for function locals, block depths, etc.
pub type SmallVec<T> = BoundedVec<T, SMALL_CAPACITY, SmallProvider>;

/// Medium bounded vector for instructions, imports, exports
pub type MediumVec<T> = BoundedVec<T, MEDIUM_CAPACITY, MediumProvider>;

/// Large bounded vector for memory buffers, large code bodies
pub type LargeVec<T> = BoundedVec<T, LARGE_CAPACITY, LargeProvider>;

/// Small bounded string for names, identifiers
pub type SmallString = BoundedString<SMALL_STRING_LEN, SmallProvider>;

/// Medium bounded string for descriptions, paths
pub type MediumString = BoundedString<MEDIUM_STRING_LEN, MediumProvider>;

/// Large bounded string for large text data
pub type LargeString = BoundedString<LARGE_STRING_LEN, LargeProvider>;

// =============================================================================
// RUNTIME-SPECIFIC TYPES
// =============================================================================

/// Function locals (small, typically <20 locals per function)
pub type LocalsVec = SmallVec<wrt_foundation::Value>;

/// Value stack (medium, for expression evaluation)
pub type ValueStackVec = MediumVec<wrt_foundation::Value>;

/// Instruction buffer (large, for function bodies)
pub type InstructionVec = LargeVec<wrt_foundation::types::Instruction>;

/// Memory buffer (large, for WebAssembly linear memory)
pub type MemoryBuffer = LargeVec<u8>;

/// Module imports map
pub type ImportsMap = std::collections::HashMap<MediumString, std::collections::HashMap<MediumString, crate::module::Import>>;

/// Module exports map  
pub type ExportsMap = std::collections::HashMap<MediumString, crate::module::Export>;

// =============================================================================
// TYPE CONVERSION UTILITIES
// =============================================================================

/// Convert between different capacity bounded vectors safely
pub fn convert_vec<T, const FROM_CAP: usize, const TO_CAP: usize, P1, P2>(
    from: BoundedVec<T, FROM_CAP, P1>,
    to_provider: P2,
) -> Result<BoundedVec<T, TO_CAP, P2>, wrt_error::Error>
where
    T: Clone,
    P1: MemoryProvider + Default + Clone,
    P2: MemoryProvider + Default + Clone,
{
    let mut result = BoundedVec::new(to_provider)?;
    for item in from.iter() {
        result.push(item.clone())?;
    }
    Ok(result)
}

/// Convert between different capacity bounded strings safely
pub fn convert_string<const FROM_LEN: usize, const TO_LEN: usize, P1, P2>(
    from: BoundedString<FROM_LEN, P1>,
    to_provider: P2,
) -> Result<BoundedString<TO_LEN, P2>, wrt_error::Error>
where
    P1: MemoryProvider + Default + Clone,
    P2: MemoryProvider + Default + Clone,
{
    let str_data = from.as_str()?;
    BoundedString::from_str_truncate(str_data, to_provider)
}

// =============================================================================
// COMPATIBILITY LAYER
// =============================================================================

/// Legacy type aliases for gradual migration
#[deprecated(note = "Use SmallVec instead")]
pub type BoundedVec64<T> = SmallVec<T>;

#[deprecated(note = "Use MediumVec instead")]  
pub type BoundedVec1024<T> = MediumVec<T>;

#[deprecated(note = "Use LargeVec instead")]
pub type BoundedVec65536<T> = LargeVec<T>;