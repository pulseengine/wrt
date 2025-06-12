//! Bounded Infrastructure for Debug
//!
//! This module provides bounded alternatives for debug collections
//! to ensure static memory allocation throughout the debug system.

#![cfg_attr(not(feature = "std"), no_std)]

use wrt_foundation::{
    bounded::{BoundedVec, BoundedString},
    no_std_hashmap::BoundedHashMap,
    budget_provider::BudgetProvider,
    budget_aware_provider::{BudgetAwareProviderFactory, CrateId},
    WrtResult,
};

/// Budget-aware memory provider for debug (32KB)
pub type DebugProvider = BudgetProvider<32768>;

/// Maximum stack trace depth
pub const MAX_STACK_TRACE_DEPTH: usize = 256;

/// Maximum number of source files
pub const MAX_SOURCE_FILES: usize = 512;

/// Maximum source file path length
pub const MAX_FILE_PATH_LEN: usize = 512;

/// Maximum function name length
pub const MAX_FUNCTION_NAME_LEN: usize = 256;

/// Maximum number of breakpoints
pub const MAX_BREAKPOINTS: usize = 1024;

/// Maximum number of watch expressions
pub const MAX_WATCH_EXPRESSIONS: usize = 256;

/// Maximum number of local variables per frame
pub const MAX_LOCALS_PER_FRAME: usize = 256;

/// Maximum number of type definitions for debugging
pub const MAX_DEBUG_TYPE_DEFINITIONS: usize = 2048;

/// Maximum source mapping entries
pub const MAX_SOURCE_MAP_ENTRIES: usize = 4096;

/// Maximum diagnostic messages
pub const MAX_DIAGNOSTIC_MESSAGES: usize = 128;

/// Maximum diagnostic message length
pub const MAX_DIAGNOSTIC_MESSAGE_LEN: usize = 1024;

/// Maximum symbol table entries
pub const MAX_SYMBOL_TABLE_ENTRIES: usize = 8192;

/// Maximum debug string length
pub const MAX_DEBUG_STRING_LEN: usize = 512;

/// Bounded vector for stack frames
pub type BoundedStackTraceVec<T> = BoundedVec<T, MAX_STACK_TRACE_DEPTH, DebugProvider>;

/// Bounded vector for source files
pub type BoundedSourceFileVec<T> = BoundedVec<T, MAX_SOURCE_FILES, DebugProvider>;

/// Bounded string for file paths
pub type BoundedFilePath = BoundedString<MAX_FILE_PATH_LEN, DebugProvider>;

/// Bounded string for function names
pub type BoundedFunctionName = BoundedString<MAX_FUNCTION_NAME_LEN, DebugProvider>;

/// Bounded vector for breakpoints
pub type BoundedBreakpointVec<T> = BoundedVec<T, MAX_BREAKPOINTS, DebugProvider>;

/// Bounded vector for watch expressions
pub type BoundedWatchVec<T> = BoundedVec<T, MAX_WATCH_EXPRESSIONS, DebugProvider>;

/// Bounded vector for local variables
pub type BoundedLocalsDebugVec<T> = BoundedVec<T, MAX_LOCALS_PER_FRAME, DebugProvider>;

/// Bounded map for type definitions
pub type BoundedDebugTypeMap<V> = BoundedHashMap<
    u32, // Type ID
    V,
    MAX_DEBUG_TYPE_DEFINITIONS,
    DebugProvider
>;

/// Bounded map for source mapping
pub type BoundedSourceMap<V> = BoundedHashMap<
    u32, // Binary offset
    V,
    MAX_SOURCE_MAP_ENTRIES,
    DebugProvider
>;

/// Bounded vector for diagnostic messages
pub type BoundedDiagnosticVec<T> = BoundedVec<T, MAX_DIAGNOSTIC_MESSAGES, DebugProvider>;

/// Bounded string for diagnostic messages
pub type BoundedDiagnosticMessage = BoundedString<MAX_DIAGNOSTIC_MESSAGE_LEN, DebugProvider>;

/// Bounded map for symbol table
pub type BoundedSymbolMap<V> = BoundedHashMap<
    BoundedFunctionName,
    V,
    MAX_SYMBOL_TABLE_ENTRIES,
    DebugProvider
>;

/// Bounded string for debug output
pub type BoundedDebugString = BoundedString<MAX_DEBUG_STRING_LEN, DebugProvider>;

/// Create a new bounded stack trace vector
pub fn new_stack_trace_vec<T>() -> WrtResult<BoundedStackTraceVec<T>> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedVec::new(provider)
}

/// Create a new bounded source file vector
pub fn new_source_file_vec<T>() -> WrtResult<BoundedSourceFileVec<T>> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedVec::new(provider)
}

/// Create a new bounded file path
pub fn new_file_path() -> WrtResult<BoundedFilePath> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    Ok(BoundedString::new(provider))
}

/// Create a bounded file path from str
pub fn bounded_file_path_from_str(s: &str) -> WrtResult<BoundedFilePath> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedString::from_str(s, provider)
}

/// Create a new bounded function name
pub fn new_function_name() -> WrtResult<BoundedFunctionName> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    Ok(BoundedString::new(provider))
}

/// Create a bounded function name from str
pub fn bounded_function_name_from_str(s: &str) -> WrtResult<BoundedFunctionName> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedString::from_str(s, provider)
}

/// Create a new bounded breakpoint vector
pub fn new_breakpoint_vec<T>() -> WrtResult<BoundedBreakpointVec<T>> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedVec::new(provider)
}

/// Create a new bounded watch vector
pub fn new_watch_vec<T>() -> WrtResult<BoundedWatchVec<T>> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedVec::new(provider)
}

/// Create a new bounded locals debug vector
pub fn new_locals_debug_vec<T>() -> WrtResult<BoundedLocalsDebugVec<T>> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedVec::new(provider)
}

/// Create a new bounded debug type map
pub fn new_debug_type_map<V>() -> WrtResult<BoundedDebugTypeMap<V>> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedHashMap::new(provider)
}

/// Create a new bounded source map
pub fn new_source_map<V>() -> WrtResult<BoundedSourceMap<V>> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedHashMap::new(provider)
}

/// Create a new bounded diagnostic vector
pub fn new_diagnostic_vec<T>() -> WrtResult<BoundedDiagnosticVec<T>> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedVec::new(provider)
}

/// Create a new bounded diagnostic message
pub fn new_diagnostic_message() -> WrtResult<BoundedDiagnosticMessage> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    Ok(BoundedString::new(provider))
}

/// Create a bounded diagnostic message from str
pub fn bounded_diagnostic_from_str(s: &str) -> WrtResult<BoundedDiagnosticMessage> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedString::from_str(s, provider)
}

/// Create a new bounded symbol map
pub fn new_symbol_map<V>() -> WrtResult<BoundedSymbolMap<V>> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedHashMap::new(provider)
}

/// Create a new bounded debug string
pub fn new_debug_string() -> WrtResult<BoundedDebugString> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    Ok(BoundedString::new(provider))
}

/// Create a bounded debug string from str
pub fn bounded_debug_string_from_str(s: &str) -> WrtResult<BoundedDebugString> {
    let provider = DebugProvider::new(CrateId::Debug)?;
    BoundedString::from_str(s, provider)
}