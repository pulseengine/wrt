// WRT - wrt-decoder
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! WebAssembly Component Model section definitions
//!
//! This module provides type definitions for WebAssembly Component Model
//! sections and common structures used in component binary parsing.

use wrt_foundation::bounded::{BoundedString, MAX_WASM_NAME_LENGTH};

/// Represents a Component export for no_alloc decoding
///
/// A simplified version of the wrt-foundation component::Export for
/// use in memory-constrained environments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentExport {
    /// Export name
    pub name: BoundedString<MAX_WASM_NAME_LENGTH>,
    /// Export type index
    pub type_index: u32,
    /// Export kind
    pub kind: u8,
}

/// Represents a Component import for no_alloc decoding
///
/// A simplified version of the wrt-foundation component::Import for
/// use in memory-constrained environments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentImport {
    /// Import name
    pub name: BoundedString<MAX_WASM_NAME_LENGTH>,
    /// Import type index
    pub type_index: u32,
}

/// Represents a Component section
///
/// This provides a common structure for all component sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSection {
    /// Section ID
    pub id: u8,
    /// Section size
    pub size: u32,
    /// Section payload offset
    pub offset: usize,
}

/// Simplified Component type for no_alloc decoding
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentType {
    /// The type form byte
    pub form: u8,
}

/// Component value types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentValueType {
    /// Primitive types
    Primitive = 0,
    /// Composite types
    Composite = 1,
    /// Resource types
    Resource = 2,
}

impl From<u8> for ComponentValueType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Primitive,
            1 => Self::Composite,
            2 => Self::Resource,
            _ => Self::Primitive, // Default to primitive for unknown values
        }
    }
}

/// Component instance for no_alloc decoding
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInstance {
    /// Instance type
    pub type_index: u32,
}
