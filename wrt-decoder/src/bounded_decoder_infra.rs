//! Bounded Infrastructure for Decoder
//!
//! This module provides bounded alternatives for decoder collections
//! to ensure static memory allocation throughout the decoder.

#![cfg_attr(not(feature = "std"), no_std)]

use wrt_foundation::{
    bounded::{BoundedVec, BoundedString},
    no_std_hashmap::BoundedHashMap,
    budget_provider::BudgetProvider,
    budget_aware_provider::{BudgetAwareProviderFactory, CrateId},
    WrtResult,
};

/// Budget-aware memory provider for decoder (64KB)
pub type DecoderProvider = BudgetProvider<65536>;

/// Maximum number of sections in a module
pub const MAX_SECTIONS: usize = 32;

/// Maximum number of types in a module
pub const MAX_TYPES: usize = 1024;

/// Maximum number of imports in a module
pub const MAX_IMPORTS: usize = 1024;

/// Maximum number of exports in a module
pub const MAX_EXPORTS: usize = 1024;

/// Maximum number of functions in a module
pub const MAX_FUNCTIONS: usize = 4096;

/// Maximum number of table entries
pub const MAX_TABLE_ENTRIES: usize = 1024;

/// Maximum number of memory definitions
pub const MAX_MEMORIES: usize = 8;

/// Maximum number of globals
pub const MAX_GLOBALS: usize = 1024;

/// Maximum number of elements
pub const MAX_ELEMENTS: usize = 1024;

/// Maximum number of data segments
pub const MAX_DATA_SEGMENTS: usize = 1024;

/// Maximum number of locals per function
pub const MAX_LOCALS_PER_FUNCTION: usize = 256;

/// Maximum length for names (modules, functions, etc.)
pub const MAX_NAME_LENGTH: usize = 256;

/// Maximum custom section data size
pub const MAX_CUSTOM_SECTION_SIZE: usize = 16384; // 16KB

/// Bounded vector for sections
pub type BoundedSectionVec<T> = BoundedVec<T, MAX_SECTIONS, DecoderProvider>;

/// Bounded vector for types
pub type BoundedTypeVec<T> = BoundedVec<T, MAX_TYPES, DecoderProvider>;

/// Bounded vector for imports
pub type BoundedImportVec<T> = BoundedVec<T, MAX_IMPORTS, DecoderProvider>;

/// Bounded vector for exports
pub type BoundedExportVec<T> = BoundedVec<T, MAX_EXPORTS, DecoderProvider>;

/// Bounded vector for functions
pub type BoundedFunctionVec<T> = BoundedVec<T, MAX_FUNCTIONS, DecoderProvider>;

/// Bounded vector for table entries
pub type BoundedTableVec<T> = BoundedVec<T, MAX_TABLE_ENTRIES, DecoderProvider>;

/// Bounded vector for memory definitions
pub type BoundedMemoryVec<T> = BoundedVec<T, MAX_MEMORIES, DecoderProvider>;

/// Bounded vector for globals
pub type BoundedGlobalVec<T> = BoundedVec<T, MAX_GLOBALS, DecoderProvider>;

/// Bounded vector for elements
pub type BoundedElementVec<T> = BoundedVec<T, MAX_ELEMENTS, DecoderProvider>;

/// Bounded vector for data segments
pub type BoundedDataVec<T> = BoundedVec<T, MAX_DATA_SEGMENTS, DecoderProvider>;

/// Bounded vector for locals
pub type BoundedLocalsVec<T> = BoundedVec<T, MAX_LOCALS_PER_FUNCTION, DecoderProvider>;

/// Bounded string for names
pub type BoundedNameString = BoundedString<MAX_NAME_LENGTH, DecoderProvider>;

/// Bounded vector for custom section data
pub type BoundedCustomData = BoundedVec<u8, MAX_CUSTOM_SECTION_SIZE, DecoderProvider>;

/// Bounded map for name resolution (exports, imports, etc.)
pub type BoundedNameMap<V> = BoundedHashMap<
    BoundedNameString,
    V,
    MAX_EXPORTS, // Use MAX_EXPORTS as general limit
    DecoderProvider
>;

/// Create a new bounded section vector
pub fn new_section_vec<T>() -> WrtResult<BoundedSectionVec<T>> {
    let provider = DecoderProvider::new(CrateId::Decoder)?;
    BoundedVec::new(provider)
}

/// Create a new bounded type vector
pub fn new_type_vec<T>() -> WrtResult<BoundedTypeVec<T>> {
    let provider = DecoderProvider::new(CrateId::Decoder)?;
    BoundedVec::new(provider)
}

/// Create a new bounded import vector
pub fn new_import_vec<T>() -> WrtResult<BoundedImportVec<T>> {
    let provider = DecoderProvider::new(CrateId::Decoder)?;
    BoundedVec::new(provider)
}

/// Create a new bounded export vector
pub fn new_export_vec<T>() -> WrtResult<BoundedExportVec<T>> {
    let provider = DecoderProvider::new(CrateId::Decoder)?;
    BoundedVec::new(provider)
}

/// Create a new bounded function vector
pub fn new_function_vec<T>() -> WrtResult<BoundedFunctionVec<T>> {
    let provider = DecoderProvider::new(CrateId::Decoder)?;
    BoundedVec::new(provider)
}

/// Create a new bounded name string
pub fn new_name_string() -> WrtResult<BoundedNameString> {
    let provider = DecoderProvider::new(CrateId::Decoder)?;
    Ok(BoundedString::new(provider))
}

/// Create a bounded name string from a str
pub fn bounded_name_from_str(s: &str) -> WrtResult<BoundedNameString> {
    let provider = DecoderProvider::new(CrateId::Decoder)?;
    BoundedString::from_str(s, provider)
}

/// Create a new bounded name map
pub fn new_name_map<V>() -> WrtResult<BoundedNameMap<V>> {
    let provider = DecoderProvider::new(CrateId::Decoder)?;
    BoundedHashMap::new(provider)
}