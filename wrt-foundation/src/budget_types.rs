//! Simplified Budget-aware Type Aliases
//!
//! This module provides type aliases that use budget-aware memory providers.

use crate::bounded::{BoundedVec, BoundedString};
use crate::bounded_collections::BoundedMap;
use crate::safe_memory::NoStdProvider;

// Runtime crate types with specific memory budgets
pub type RuntimeVec<T, const N: usize> = BoundedVec<T, N, NoStdProvider<131072>>; // 128KB for runtime
pub type RuntimeString<const N: usize> = BoundedString<N, NoStdProvider<131072>>;
pub type RuntimeMap<K, V, const N: usize> = BoundedMap<K, V, N, NoStdProvider<131072>>;

// Foundation crate types  
pub type FoundationVec<T, const N: usize> = BoundedVec<T, N, NoStdProvider<65536>>; // 64KB for foundation
pub type FoundationString<const N: usize> = BoundedString<N, NoStdProvider<65536>>;
pub type FoundationMap<K, V, const N: usize> = BoundedMap<K, V, N, NoStdProvider<65536>>;

// Format crate types
pub type FormatVec<T, const N: usize> = BoundedVec<T, N, NoStdProvider<65536>>; // 64KB for format
pub type FormatString<const N: usize> = BoundedString<N, NoStdProvider<65536>>;
pub type FormatMap<K, V, const N: usize> = BoundedMap<K, V, N, NoStdProvider<65536>>;

// Component crate types
pub type ComponentVec<T, const N: usize> = BoundedVec<T, N, NoStdProvider<32768>>; // 32KB for component
pub type ComponentString<const N: usize> = BoundedString<N, NoStdProvider<32768>>;
pub type ComponentMap<K, V, const N: usize> = BoundedMap<K, V, N, NoStdProvider<32768>>;

// Decoder crate types
pub type DecoderVec<T, const N: usize> = BoundedVec<T, N, NoStdProvider<32768>>; // 32KB for decoder
pub type DecoderString<const N: usize> = BoundedString<N, NoStdProvider<32768>>;

// Instructions crate types
pub type InstructionsVec<T, const N: usize> = BoundedVec<T, N, NoStdProvider<32768>>; // 32KB for instructions
pub type InstructionsString<const N: usize> = BoundedString<N, NoStdProvider<32768>>;