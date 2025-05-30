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

// Import synchronization primitives for no_std
//#[cfg(not(feature = "std"))]
// pub use wrt_sync::{Mutex, RwLock};

// Re-export from wrt-error
pub use wrt_error::{codes, kinds, Error, ErrorCategory, Result};
// Re-export format module for compatibility
pub use wrt_format as format;
// Re-export from wrt-format
pub use wrt_format::{
    // Constants (always available)
    binary::{WASM_MAGIC, WASM_VERSION},
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

// Import binary functions that require alloc
#[cfg(any(feature = "alloc", feature = "std"))]
pub use wrt_format::{
    binary::{
        is_valid_wasm_header, parse_block_type, read_f32, read_f64, read_leb128_i32, read_leb128_i64,
        read_leb128_u64, read_name, read_u8, read_vector as parse_vec, validate_utf8, write_leb128_i32,
        write_leb128_i64, write_leb128_u32, write_leb128_u64, write_string, BinaryFormat,
    },
    state::{create_state_section, extract_state_section, StateSection},
};
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

// Re-export from this crate
#[cfg(any(feature = "alloc", feature = "std"))]
pub use crate::{
    // Component model no-alloc support
    component::decode_no_alloc,
    // Module types
    module::Module,
    // Utils
    utils,
};

#[cfg(feature = "alloc")]
pub use crate::decoder_core::validate;

// No-alloc support (always available)
pub use crate::decoder_no_alloc;

// Type aliases for no_std mode
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub use wrt_foundation::{BoundedString, BoundedVec, NoStdProvider};

// For no_std mode, provide bounded collection aliases
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type Vec<T> = BoundedVec<T, 1024, NoStdProvider<2048>>;
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub type String = BoundedString<512, NoStdProvider<1024>>;

// For no_std mode, provide a minimal ToString trait
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub trait ToString {
    fn to_string(&self) -> String;
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl ToString for &str {
    fn to_string(&self) -> String {
        String::from_str(self, NoStdProvider::default()).unwrap_or_default()
    }
}

// Binary function aliases for different feature configurations
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod binary {
    pub use wrt_format::binary::{write_leb128_u32, write_string};
    // Alias for read_leb_u32
    pub fn read_leb_u32(data: &[u8]) -> wrt_error::Result<(u32, usize)> {
        wrt_format::binary::read_leb128_u32(data, 0)
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
pub mod binary {
    use wrt_foundation::{BoundedVec, NoStdProvider};
    
    // Alias for write_leb128_u32 in no_std mode
    pub fn write_leb128_u32(value: u32) -> BoundedVec<u8, 10, NoStdProvider<64>> {
        let mut result = BoundedVec::new(NoStdProvider::default())
            .expect("Failed to create bounded vec for LEB128");
        let mut buffer = [0u8; 10];
        if let Ok(bytes_written) = wrt_format::binary::write_leb128_u32_to_slice(value, &mut buffer) {
            for i in 0..bytes_written {
                let _ = result.push(buffer[i]);
            }
        }
        result
    }
    
    // Alias for write_string in no_std mode  
    pub fn write_string(_s: &str) -> BoundedVec<u8, 256, NoStdProvider<512>> {
        // Simplified no_std implementation
        BoundedVec::new(NoStdProvider::default())
            .expect("Failed to create bounded vec for string")
    }
    
    // Alias for read_leb_u32
    pub fn read_leb_u32(data: &[u8], offset: usize) -> wrt_error::Result<(u32, usize)> {
        wrt_format::binary::read_leb128_u32(data, offset)
    }
}

// Make commonly used binary functions available at top level
pub use wrt_format::binary::{read_leb128_u32, read_string, read_u32};

// For compatibility, add some aliases that the code expects
pub fn read_leb_u32(data: &[u8]) -> wrt_error::Result<(u32, usize)> {
    read_leb128_u32(data, 0)
}

// Feature-gated function aliases (no duplicates since they're already imported above)
#[cfg(any(feature = "alloc", feature = "std"))]
pub use parse_block_type as parse_format_block_type;
