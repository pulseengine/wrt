//! WebAssembly format handling for WRT.
//!
//! This crate provides utilities for working with WebAssembly binary formats,
//! including serialization and deserialization of WebAssembly modules and state.

#![cfg_attr(not(feature = "std"), no_std)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(not(feature = "std"))]
extern crate alloc;

// Import std/alloc collections based on feature flag
#[cfg(feature = "std")]
pub use std::{
    boxed::Box,
    collections::{HashMap, HashSet},
    format,
    string::String,
    vec::Vec,
};

#[cfg(not(feature = "std"))]
pub use alloc::{
    boxed::Box,
    collections::{BTreeMap as HashMap, BTreeSet as HashSet},
    format,
    string::String,
    vec::Vec,
};

// Re-export core types from wrt-types
pub use wrt_types::{safe_memory::SafeSlice, FuncType, ValueType};

pub mod binary;
pub mod component;
pub mod compression;
pub mod module;
pub mod section;
pub mod state;
pub mod types;
pub mod version;

pub use component::Component;
pub use compression::{rle_decode, rle_encode, CompressionType};
pub use module::Module;
pub use section::{CustomSection, Section};
pub use state::{create_state_section, extract_state_section, StateSection};
pub use types::{parse_value_type, value_type_to_byte, BlockType, Limits, MemoryIndexType};
pub use version::{
    ComponentModelFeature, ComponentModelVersion, FeatureStatus, VersionInfo, STATE_VERSION,
};

// Public functions for feature detection
/// Check if a component model feature is available in a binary
pub fn is_feature_available(info: &VersionInfo, feature: ComponentModelFeature) -> bool {
    info.is_feature_available(feature)
}

/// Get the status of a component model feature
pub fn get_feature_status(info: &VersionInfo, feature: ComponentModelFeature) -> FeatureStatus {
    info.get_feature_status(feature)
}

/// Check if a binary uses any experimental features
pub fn uses_experimental_features(binary: &[u8]) -> bool {
    let version_bytes = if binary.len() >= 8 {
        [binary[4], binary[5], binary[6], binary[7]]
    } else {
        return false;
    };

    let mut info = VersionInfo::from_version_bytes(version_bytes);
    info.detect_experimental_features(binary)
}
