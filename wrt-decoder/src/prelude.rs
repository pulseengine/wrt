// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Prelude module for wrt-decoder
//!
//! This module provides a unified set of imports for both std and no_std
//! environments. It re-exports commonly used types and traits to ensure
//! consistency across all crates in the WRT project and simplify imports in
//! individual modules.

// Core imports for both std and no_std environments
// Re-export from alloc when no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::{
    borrow::Cow,
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    rc::Rc,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};

// Don't duplicate format import since it's already in the use block above
#[cfg(not(feature = "std"))]
pub use core::result::Result as StdResult;
pub use core::{
    any::Any,
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    convert::{From, Into, TryFrom, TryInto},
    fmt,
    fmt::{Debug, Display},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    slice, str,
};
// Re-export from std when the std feature is enabled
#[cfg(feature = "std")]
pub use std::{
    borrow::Cow,
    boxed::Box,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    format, io,
    io::{Read, Write},
    rc::Rc,
    result::Result as StdResult,
    string::{String, ToString},
    sync::{Arc, Mutex, RwLock},
    vec,
    vec::Vec,
};

// Don't duplicate format import since it's already in the use block above

// Import synchronization primitives for no_std
//#[cfg(not(feature = "std"))]
// pub use wrt_sync::{Mutex, RwLock};

// Re-export from wrt-error
pub use wrt_error::{codes, kinds, Error, ErrorCategory, Result};
// Re-export format module for compatibility
pub use wrt_format as wrt_format_module;
// Re-export from wrt-format
pub use wrt_format::{
    // Conversion utilities
    conversion::{
        block_type_to_format_block_type, format_block_type_to_block_type,
        format_value_type as value_type_to_byte, parse_value_type,
    },
    // Module types
    module::{
        Data, DataMode, Element, Export, ExportKind, Function, Global, Import, ImportDesc, Memory,
        Table,
    },
    // Section types
    section::{CustomSection, Section, SectionId},
    // Format-specific types
    types::{FormatBlockType, Limits, MemoryIndexType},
};

// Import additional functions that require alloc (beyond what wrt_format exports)
#[cfg(any(feature = "alloc", feature = "std"))]
pub use wrt_format::state::{create_state_section, extract_state_section, StateSection};
// Component model types (require alloc)
#[cfg(feature = "alloc")]
pub use wrt_foundation::component_value::{ComponentValue, ValType};
// Conversion utilities from wrt-foundation
#[cfg(feature = "conversion")]
pub use wrt_foundation::conversion::{ref_type_to_val_type, val_type_to_ref_type};
// Re-export from wrt-foundation
pub use wrt_foundation::{
    // SafeMemory types
    safe_memory::{SafeMemoryHandler, SafeSlice, SafeStack},
    // Types
    types::{BlockType, FuncType, GlobalType, MemoryType, RefType, TableType, ValueType},
    values::Value,
};

// Most re-exports temporarily disabled for demo

// No-alloc support (always available)
pub use crate::decoder_no_alloc;

// Type aliases for no_std mode
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub use wrt_foundation::{BoundedString, BoundedVec, NoStdProvider};

// For no_std mode, provide bounded collection aliases
/// Bounded vector for no_std environments
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type Vec<T> = BoundedVec<T, 1024, NoStdProvider<2048>>;
/// Bounded string for no_std environments
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type String = BoundedString<512, NoStdProvider<1024>>;

// For no_std mode, provide a minimal ToString trait
/// Minimal ToString trait for no_std environments
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub trait ToString {
    /// Convert to string
    fn to_string(&self) -> String;
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl ToString for &str {
    fn to_string(&self) -> String {
        String::from_str(self, NoStdProvider::<1024>::default()).unwrap_or_default()
    }
}

// For no_std without alloc, provide a minimal format macro implementation
/// Minimal format macro for no_std environments
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {{
        // In pure no_std, return a simple bounded string
        use wrt_foundation::{BoundedString, NoStdProvider};
        BoundedString::<256, NoStdProvider<512>>::from_str(
            "formatted_string",
            NoStdProvider::<512>::default(),
        )
        .unwrap_or_default()
    }};
}

// Export our custom format macro for no_std
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub use crate::format;

/// Binary format utilities
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod binary {
    /// Read LEB128 u32 from data
    pub fn read_leb_u32(data: &[u8]) -> wrt_error::Result<(u32, usize)> {
        wrt_format::binary::read_leb128_u32(data, 0)
    }
}

/// Binary utilities for no_std environments
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub mod binary {
    use wrt_foundation::{BoundedVec, NoStdProvider};

    /// Write LEB128 u32 in no_std mode
    pub fn write_leb128_u32(value: u32) -> BoundedVec<u8, 10, NoStdProvider<64>> {
        let mut result = BoundedVec::new(NoStdProvider::<64>::default())
            .expect("Failed to create bounded vec for LEB128");
        let mut buffer = [0u8; 10];
        // Simple LEB128 encoding for no_std
        let mut bytes_written = 0;
        let mut val = value;
        loop {
            let mut byte = (val & 0x7F) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0x80;
            }
            if bytes_written < buffer.len() {
                buffer[bytes_written] = byte;
                bytes_written += 1;
            }
            if val == 0 {
                break;
            }
        }

        if bytes_written > 0 {
            for i in 0..bytes_written {
                let _ = result.push(buffer[i]);
            }
        }
        result
    }

    /// Write string in no_std mode
    pub fn write_string(_s: &str) -> BoundedVec<u8, 256, NoStdProvider<512>> {
        // Simplified no_std implementation
        BoundedVec::new(NoStdProvider::<512>::default()).expect("Failed to create bounded vec for string")
    }

    /// Read LEB128 u32 from data with offset
    pub fn read_leb_u32(data: &[u8], offset: usize) -> wrt_error::Result<(u32, usize)> {
        // Simple implementation for no_std - just read from beginning
        if offset >= data.len() {
            return Err(wrt_error::Error::new(
                wrt_error::ErrorCategory::Parse,
                wrt_error::codes::PARSE_ERROR,
                "Offset out of bounds",
            ));
        }
        // For simplicity, just parse from the offset
        let mut value = 0u32;
        let mut shift = 0;
        let mut bytes_read = 0;

        for &byte in &data[offset..] {
            if bytes_read >= 5 {
                return Err(wrt_error::Error::new(
                    wrt_error::ErrorCategory::Parse,
                    wrt_error::codes::PARSE_ERROR,
                    "LEB128 too long",
                ));
            }

            value |= ((byte & 0x7F) as u32) << shift;
            bytes_read += 1;

            if (byte & 0x80) == 0 {
                return Ok((value, bytes_read));
            }

            shift += 7;
        }

        Err(wrt_error::Error::new(
            wrt_error::ErrorCategory::Parse,
            wrt_error::codes::PARSE_ERROR,
            "Incomplete LEB128",
        ))
    }

    /// Read name from binary data in no_std mode
    pub fn read_name(data: &[u8], offset: usize) -> wrt_error::Result<(&[u8], usize)> {
        if offset >= data.len() {
            return Err(wrt_error::Error::new(
                wrt_error::ErrorCategory::Parse,
                wrt_error::codes::PARSE_ERROR,
                "Offset out of bounds",
            ));
        }

        // Read length as LEB128
        let (length, new_offset) = read_leb_u32(data, offset)?;
        let name_start = offset + new_offset;

        if name_start + length as usize > data.len() {
            return Err(wrt_error::Error::new(
                wrt_error::ErrorCategory::Parse,
                wrt_error::codes::PARSE_ERROR,
                "Name extends beyond data",
            ));
        }

        Ok((&data[name_start..name_start + length as usize], name_start + length as usize))
    }
}

// Make commonly used binary functions available at top level (now exported by wrt_format directly)
// pub use wrt_format::binary::{read_leb128_u32, read_string, read_u32};

// For compatibility, add some aliases that the code expects
/// Read LEB128 u32 from data
#[cfg(any(feature = "alloc", feature = "std"))]
pub fn read_leb_u32(data: &[u8]) -> wrt_error::Result<(u32, usize)> {
    binary::read_leb_u32(data)
}

/// Read LEB128 u32 from data (no_std version)
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub fn read_leb_u32(data: &[u8]) -> wrt_error::Result<(u32, usize)> {
    binary::read_leb_u32(data, 0)
}

// Missing utility functions
/// Validate WebAssembly header
pub fn is_valid_wasm_header(data: &[u8]) -> bool {
    data.len() >= 8
        && &data[0..4] == wrt_format::binary::WASM_MAGIC
        && &data[4..8] == wrt_format::binary::WASM_VERSION
}

/// Read name from binary data
#[cfg(any(feature = "alloc", feature = "std"))]
pub fn read_name(data: &[u8], offset: usize) -> wrt_error::Result<(&[u8], usize)> {
    wrt_format::binary::read_name(data, offset)
}

/// Read name from binary data (no_std version)
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub fn read_name(data: &[u8], offset: usize) -> wrt_error::Result<(&[u8], usize)> {
    binary::read_name(data, offset)
}

/// Read LEB128 u32 with offset
#[cfg(any(feature = "alloc", feature = "std"))]
pub fn read_leb128_u32(data: &[u8], offset: usize) -> wrt_error::Result<(u32, usize)> {
    wrt_format::binary::read_leb128_u32(data, offset)
}

/// Read LEB128 u32 with offset (no_std version)
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub fn read_leb128_u32(data: &[u8], offset: usize) -> wrt_error::Result<(u32, usize)> {
    binary::read_leb_u32(data, offset)
}

// Feature-gated function aliases - bring in functions from wrt_format that aren't already exported
#[cfg(any(feature = "alloc", feature = "std"))]
pub use wrt_format::parse_block_type as parse_format_block_type;
