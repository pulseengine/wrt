//! WebAssembly format handling for WRT.
//!
//! This crate provides utilities for working with WebAssembly binary formats,
//! including serialization and deserialization of WebAssembly modules and state.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::missing_panics_doc)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Import std/alloc collections based on feature flag
#[cfg(feature = "std")]
pub use std::{
    boxed::Box,
    collections::{HashMap, HashSet},
    fmt, format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::{
    boxed::Box,
    collections::{BTreeMap as HashMap, BTreeSet as HashSet},
    fmt, format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

// Re-export core types from wrt-types
pub use wrt_types::{safe_memory::SafeSlice, BlockType, FuncType, RefType, ValueType};
// Re-export error types from wrt-types error_convert module
pub use wrt_types::error_convert::{Error, ErrorCategory};
// Re-export Result type from wrt-types
pub use wrt_types::Result;
// Re-export resource types from wrt-types
pub use wrt_types::resource::ResourceRepresentation;

/// WebAssembly binary format parsing and access
pub mod binary;
pub mod canonical;
/// WebAssembly component model format
pub mod component;
/// Conversion utilities for component model types
pub mod component_conversion;
/// Compression utilities for WebAssembly modules
pub mod compression;
/// Error utilities for working with wrt-error types
pub mod error;
/// WebAssembly module format
pub mod module;
pub mod section;
pub mod state;
pub mod types;
/// Validation utilities
pub mod validation;
pub mod verify;
pub mod version;

pub use component::Component;
pub use compression::{rle_decode, rle_encode, CompressionType};
pub use error::{parse_error, runtime_error, type_error, validation_error, IntoError};
pub use module::Module;
pub use section::{CustomSection, Section};
pub use state::{create_state_section, extract_state_section, StateSection};
pub use types::{parse_value_type, value_type_to_byte, FormatBlockType, Limits, MemoryIndexType};
pub use validation::Validatable;
pub use version::{
    ComponentModelFeature, ComponentModelVersion, FeatureStatus, VersionInfo, STATE_VERSION,
};
// Re-export binary constants
pub use binary::{
    read_leb128_u32, read_string, write_leb128_u32, write_string, COMPONENT_CORE_SORT_FUNC,
    COMPONENT_CORE_SORT_GLOBAL, COMPONENT_CORE_SORT_INSTANCE, COMPONENT_CORE_SORT_MEMORY,
    COMPONENT_CORE_SORT_MODULE, COMPONENT_CORE_SORT_TABLE, COMPONENT_CORE_SORT_TYPE,
    COMPONENT_MAGIC, COMPONENT_SORT_COMPONENT, COMPONENT_SORT_CORE, COMPONENT_SORT_FUNC,
    COMPONENT_SORT_INSTANCE, COMPONENT_SORT_TYPE, COMPONENT_SORT_VALUE, COMPONENT_VERSION,
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
