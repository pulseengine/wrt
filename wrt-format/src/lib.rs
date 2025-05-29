// WRT - wrt-format
// Module: WebAssembly Binary Format Definitions
// SW-REQ-ID: REQ_021
// SW-REQ-ID: REQ_013
//
// Copyright (c) 2024 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)] // Rule 2

//! WebAssembly format handling for WRT
//!
//! This crate defines and handles the WebAssembly binary format specifications,
//! including type encodings, section layouts, and module structures.
//!
//! It is designed to work in both std and no_std environments when configured
//! with the appropriate feature flags.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std environments with allocation
#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::boxed::Box;
// Import types for internal use
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

// Re-export error types directly from wrt-error
pub use wrt_error::{Error, ErrorCategory};
// Re-export resource types from wrt-foundation
pub use wrt_foundation::resource::ResourceRepresentation;
// Re-export Result type from wrt-foundation
pub use wrt_foundation::Result;
// Re-export core types from wrt-foundation (note: these now have generic parameters)
// BlockType, FuncType, RefType, ValueType now require MemoryProvider parameters
// These will be re-exported as type aliases with default providers

// Collection types are imported privately above and used internally

// Import bounded collections for no_std without alloc
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub use wrt_foundation::{BoundedMap, BoundedSet, BoundedString, BoundedVec};

// Type aliases for pure no_std mode
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type WasmString<P> = BoundedString<MAX_WASM_STRING_SIZE, P>;
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type WasmVec<T, P> = BoundedVec<T, 1024, P>; // General purpose bounded vector
                                                 // Module type aliases for pure no_std mode
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type ModuleFunctions<P> = BoundedVec<crate::module::Function<P>, MAX_MODULE_FUNCTIONS, P>;
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type ModuleImports<P> = BoundedVec<crate::module::Import<P>, MAX_MODULE_IMPORTS, P>;
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type ModuleExports<P> = BoundedVec<crate::module::Export<P>, MAX_MODULE_EXPORTS, P>;
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type ModuleGlobals<P> = BoundedVec<crate::module::Global<P>, MAX_MODULE_GLOBALS, P>;
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type ModuleElements<P> = BoundedVec<crate::module::Element<P>, MAX_MODULE_ELEMENTS, P>;
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type ModuleData<P> = BoundedVec<crate::module::Data<P>, MAX_MODULE_DATA, P>;
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type ModuleCustomSections<P> = BoundedVec<crate::section::CustomSection<P>, 64, P>;

// Type aliases for HashMap
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type HashMap<K, V> = wrt_foundation::BoundedMap<K, V, 256, wrt_foundation::NoStdProvider<1024>>; // Default capacity

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub type HashMap<K, V> = alloc::collections::BTreeMap<K, V>; // Use BTreeMap in no_std+alloc

#[cfg(feature = "std")]
pub type HashMap<K, V> = std::collections::HashMap<K, V>;

// Maximum recursion depth for recursive types to replace Box<T>
pub const MAX_TYPE_RECURSION_DEPTH: usize = 32;

// Type aliases for WebAssembly-specific collections
#[cfg(any(feature = "alloc", feature = "std"))]
pub type WasmString = String;
#[cfg(any(feature = "alloc", feature = "std"))]
pub type WasmVec<T> = Vec<T>;

// In pure no_std mode, we don't provide generic Vec/String aliases
// Individual modules should use the appropriate bounded types directly

// Helper macro for conditional type usage
#[macro_export]
macro_rules! collection_type {
    (Vec<$t:ty>) => {
        #[cfg(any(feature = "alloc", feature = "std"))]
        type VecType = Vec<$t>;
        #[cfg(not(any(feature = "alloc", feature = "std")))]
        type VecType = $crate::WasmVec<$t, $crate::NoStdProvider<1024>>;
    };
    (String) => {
        #[cfg(any(feature = "alloc", feature = "std"))]
        type StringType = String;
        #[cfg(not(any(feature = "alloc", feature = "std")))]
        type StringType = $crate::WasmString<$crate::NoStdProvider<1024>>;
    };
}

// Compile-time capacity constants for bounded collections
pub const MAX_MODULE_TYPES: usize = 256;
pub const MAX_MODULE_FUNCTIONS: usize = 1024;
pub const MAX_MODULE_IMPORTS: usize = 256;
pub const MAX_MODULE_EXPORTS: usize = 256;
pub const MAX_MODULE_GLOBALS: usize = 256;
pub const MAX_MODULE_TABLES: usize = 64;
pub const MAX_MODULE_MEMORIES: usize = 64;
pub const MAX_MODULE_ELEMENTS: usize = 256;
pub const MAX_MODULE_DATA: usize = 256;
pub const MAX_WASM_STRING_SIZE: usize = 256;
pub const MAX_BINARY_SIZE: usize = 1024 * 1024; // 1MB max module size
pub const MAX_LEB128_BUFFER: usize = 10; // Max bytes for LEB128 u64
pub const MAX_INSTRUCTION_OPERANDS: usize = 16;
pub const MAX_STACK_DEPTH: usize = 1024;

// Component model constants
pub const MAX_COMPONENT_INSTANCES: usize = 128;
pub const MAX_COMPONENT_TYPES: usize = 256;
pub const MAX_COMPONENT_IMPORTS: usize = 256;
pub const MAX_COMPONENT_EXPORTS: usize = 256;

// For no_std mode, provide format! macro replacement using static strings
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[macro_export]
macro_rules! format {
    ($lit:literal) => {
        $lit
    };
    ($fmt:literal, $($arg:expr),*) => {
        // In pure no_std mode, we can't format strings dynamically
        // Return a static error message instead
        "formatting not available in no_std mode"
    };
}

/// WebAssembly binary format parsing and access
pub mod binary;
/// WebAssembly canonical format
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod canonical;
/// WebAssembly component model format
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod component;
/// Conversion utilities for component model types
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod component_conversion;
/// Compression utilities for WebAssembly modules
pub mod compression;
/// Conversion utilities for type system standardization
pub mod conversion;
/// Error utilities for working with wrt-error types
pub mod error;
/// WebAssembly module format
pub mod module;
/// Common imports for convenience
pub mod prelude;
/// Resource handle management for Component Model
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod resource_handle;
/// Safe memory operations
pub mod safe_memory;
pub mod section;
pub mod state;
/// Streaming parser for no_std environments
pub mod streaming;
/// Type storage system for Component Model
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod type_store;
pub mod types;
/// Validation utilities
pub mod validation;
/// ValType builder utilities
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod valtype_builder;
pub mod verify;
pub mod version;
// WIT (WebAssembly Interface Types) parser (requires alloc for component model)
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod wit_parser;

// Re-export binary constants (always available)
// Re-export write functions (only with alloc)
#[cfg(any(feature = "alloc", feature = "std"))]
pub use binary::with_alloc::{write_leb128_u32, write_string};
pub use binary::{
    read_leb128_u32, read_string, COMPONENT_CORE_SORT_FUNC, COMPONENT_CORE_SORT_GLOBAL,
    COMPONENT_CORE_SORT_INSTANCE, COMPONENT_CORE_SORT_MEMORY, COMPONENT_CORE_SORT_MODULE,
    COMPONENT_CORE_SORT_TABLE, COMPONENT_CORE_SORT_TYPE, COMPONENT_MAGIC, COMPONENT_SORT_COMPONENT,
    COMPONENT_SORT_CORE, COMPONENT_SORT_FUNC, COMPONENT_SORT_INSTANCE, COMPONENT_SORT_TYPE,
    COMPONENT_SORT_VALUE, COMPONENT_VERSION,
};
// Re-export no_std write functions
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub use binary::{
    write_leb128_u32_bounded, write_leb128_u32_to_slice, write_string_bounded,
    write_string_to_slice,
};
#[cfg(any(feature = "alloc", feature = "std"))]
pub use component::Component;
pub use compression::CompressionType;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use compression::{rle_decode, rle_encode};
// Re-export conversion utilities
pub use conversion::{
    block_type_to_format_block_type, format_block_type_to_block_type, format_limits_to_wrt_limits,
    wrt_limits_to_format_limits,
};
pub use error::{
    parse_error, wrt_runtime_error as runtime_error, wrt_type_error as type_error,
    wrt_validation_error as validation_error,
};
pub use module::Module;
// Re-export safe memory utilities
pub use safe_memory::safe_slice;
pub use section::{CustomSection, Section};
#[cfg(any(feature = "alloc", feature = "std"))]
pub use state::{create_state_section, extract_state_section, is_state_section_name, StateSection};
// Use the conversion module versions for consistency
pub use types::{FormatBlockType, Limits, MemoryIndexType};
pub use validation::Validatable;
pub use version::{
    ComponentModelFeature, ComponentModelVersion, FeatureStatus, VersionInfo, STATE_VERSION,
};
// Re-export WIT parser (requires alloc for component model)
#[cfg(any(feature = "alloc", feature = "std"))]
pub use wit_parser::{
    WitEnum, WitExport, WitFlags, WitFunction, WitImport, WitInterface, WitItem, WitParam,
    WitParseError, WitParser, WitRecord, WitResult, WitType, WitTypeDef, WitVariant, WitWorld,
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

// Deprecated: use conversion utilities instead
#[deprecated(
    since = "0.2.0",
    note = "Use conversion::parse_value_type instead for better type conversion"
)]
pub use types::parse_value_type;
#[deprecated(
    since = "0.2.0",
    note = "Use conversion::value_type_to_byte instead for better type conversion"
)]
pub use types::value_type_to_byte;

// For formal verification when the 'kani' feature is enabled
#[cfg(feature = "kani")]
pub mod verification {
    /// Verify LEB128 encoding and decoding
    #[cfg(all(kani, any(feature = "alloc", feature = "std")))]
    #[kani::proof]
    fn verify_leb128_roundtrip() {
        let value: u32 = kani::any();
        // Limit to reasonable values for test
        kani::assume(value <= 0xFFFF);

        let encoded = super::binary::with_alloc::write_leb128_u32(value);
        let (decoded, _) = super::binary::read_leb128_u32(&encoded, 0).unwrap();

        assert_eq!(value, decoded);
    }
}

/// Demonstration of pure no_std WebAssembly format handling
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub mod no_std_demo {
    use wrt_foundation::NoStdProvider;

    use super::*;

    /// Example showing TypeRef system working
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn demo_type_system() -> wrt_error::Result<()> {
        use crate::component::{FormatValType, TypeRegistry};

        // Create a type registry
        let mut registry = TypeRegistry::new();

        // Add a primitive type
        let bool_ref = registry.add_type(FormatValType::Bool)?;

        // Add a list type that references the bool type
        let bool_list_ref = registry.add_type(FormatValType::List(bool_ref))?;

        // Verify we can retrieve the types
        assert!(registry.get_type(bool_ref).is_some());
        assert!(registry.get_type(bool_list_ref).is_some());

        Ok(())
    }

    /// Example showing bounded string working
    pub fn demo_bounded_string() -> wrt_error::Result<()> {
        let mut wasm_str =
            WasmString::<NoStdProvider<1024>>::from_str("hello", NoStdProvider::<1024>::default())
                .map_err(|_| wrt_foundation::bounded::CapacityError)?;
        assert_eq!(wasm_str.as_str().unwrap(), "hello");
        Ok(())
    }

    /// Example showing LEB128 parsing (no allocation)
    pub fn demo_leb128_parsing() -> crate::Result<()> {
        let data = [0x80, 0x01]; // LEB128 encoding of 128
        let (value, consumed) = crate::binary::read_leb128_u32(&data, 0)?;
        assert_eq!(value, 128);
        assert_eq!(consumed, 2);
        Ok(())
    }

    /// Example showing complete no_std WebAssembly parsing workflow
    pub fn demo_no_std_parsing_workflow() -> crate::Result<()> {
        use wrt_foundation::NoStdProvider;

        use crate::streaming::StreamingParser;

        // Create a minimal valid WebAssembly module (static array, no allocation)
        let wasm_data = [
            // Magic bytes: \0asm
            0x00, 0x61, 0x73, 0x6D, // Version: 1.0.0.0
            0x01, 0x00, 0x00, 0x00,
            // Empty module (no sections)
        ];

        // Create streaming parser with bounded memory
        let provider = NoStdProvider::<1024>::default();
        let mut parser = StreamingParser::new(provider)?;

        // Process the WebAssembly data
        let result = parser.process_chunk(&wasm_data)?;

        // Verify parsing completed successfully
        match result {
            crate::streaming::ParseResult::Complete(_) => {
                assert_eq!(parser.bytes_processed(), 8); // 4 magic + 4 version bytes
                Ok(())
            }
            _ => Err(crate::Error::new(
                crate::ErrorCategory::Validation,
                wrt_error::codes::PARSE_ERROR,
                "Parsing did not complete as expected",
            )),
        }
    }

    /// Example showing module creation in pure no_std mode
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn demo_module_creation() -> Result<(), wrt_foundation::bounded::CapacityError> {
        use wrt_foundation::NoStdProvider;

        use crate::module::Module;

        // This demonstrates that the Module type system works in pure no_std
        let provider = NoStdProvider::<1024>::default();
        let _module = Module::<NoStdProvider<1024>>::default();

        // The module can be created and used without any heap allocation
        Ok(())
    }
}
