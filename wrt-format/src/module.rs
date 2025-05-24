//! WebAssembly module format.
//!
//! This module provides types and utilities for working with WebAssembly
//! modules.

// Import collection types
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

use wrt_error::{codes, Error, ErrorCategory, Result};
use wrt_foundation::{traits::BoundedCapacity, types::GlobalType, RefType, ValueType};
#[cfg(not(any(feature = "alloc", feature = "std")))]
use wrt_foundation::{BoundedString, BoundedVec, MemoryProvider, NoStdProvider};

use crate::{
    section::CustomSection,
    types::{CoreWasmVersion, FormatGlobalType, Limits},
    validation::Validatable,
};
#[cfg(not(any(feature = "alloc", feature = "std")))]
use crate::{
    ModuleCustomSections, ModuleData, ModuleElements, ModuleExports, ModuleFunctions,
    ModuleGlobals, ModuleImports, WasmString, WasmVec,
};

/// WebAssembly function definition - Pure No_std Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone)]
pub struct Function<
    P: wrt_foundation::MemoryProvider + Clone + Default + Eq = wrt_foundation::NoStdProvider<1024>,
> {
    /// Type index referring to function signature
    pub type_idx: u32,
    /// Local variables (types and counts)
    pub locals: crate::WasmVec<ValueType, P>,
    /// Function body (WebAssembly bytecode instructions)
    pub code: crate::WasmVec<u8, P>,
}

/// WebAssembly function definition - With Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone)]
pub struct Function {
    /// Type index referring to function signature
    pub type_idx: u32,
    /// Local variables (types and counts)
    pub locals: Vec<ValueType>,
    /// Function body (WebAssembly bytecode instructions)
    pub code: Vec<u8>,
}

/// WebAssembly memory definition
///
/// A memory instance as defined in the WebAssembly Core Specification.
/// The memory section consists of a vector of memory definitions, each
/// defining a memory with limits, and optional shared flag for threading.
///
/// WebAssembly 1.0 allows at most one memory per module.
/// Memory64 extension allows memories with 64-bit addressing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Memory {
    /// Memory limits (minimum and optional maximum size in pages)
    /// Each page is 64KiB (65536 bytes)
    pub limits: Limits,
    /// Whether this memory is shared between threads
    /// Shared memory must have a maximum size specified
    pub shared: bool,
}

/// WebAssembly table definition
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Table {
    /// Element type
    pub element_type: ValueType,
    /// Table limits
    pub limits: Limits,
}

/// WebAssembly global definition - Pure No_std Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone)]
pub struct Global<
    P: wrt_foundation::MemoryProvider + Clone + Default + Eq = wrt_foundation::NoStdProvider<1024>,
> {
    /// Global type
    pub global_type: FormatGlobalType,
    /// Initialization expression
    pub init: crate::WasmVec<u8, P>,
}

/// WebAssembly global definition - With Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone)]
pub struct Global {
    /// Global type
    pub global_type: FormatGlobalType,
    /// Initialization expression
    pub init: Vec<u8>,
}

/// WebAssembly data segment types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataMode {
    /// Active data segment (explicitly placed into a memory)
    Active,
    /// Passive data segment (used with memory.init)
    Passive,
}

/// WebAssembly data segment - Pure No_std Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone)]
pub struct Data<
    P: wrt_foundation::MemoryProvider + Clone + Default + Eq = wrt_foundation::NoStdProvider<1024>,
> {
    /// Data mode (active or passive)
    pub mode: DataMode,
    /// Memory index (for active data segments)
    pub memory_idx: u32,
    /// Offset expression (for active data segments)
    pub offset: crate::WasmVec<u8, P>,
    /// Initial data
    pub init: crate::WasmVec<u8, P>,
}

/// WebAssembly data segment - With Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone)]
pub struct Data {
    /// Data mode (active or passive)
    pub mode: DataMode,
    /// Memory index (for active data segments)
    pub memory_idx: u32,
    /// Offset expression (for active data segments)
    pub offset: Vec<u8>,
    /// Initial data
    pub init: Vec<u8>,
}

/// Represents the initialization items for an element segment - Pure No_std
/// Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone)]
pub enum ElementInit<
    P: wrt_foundation::MemoryProvider + Clone + Default + Eq = wrt_foundation::NoStdProvider<1024>,
> {
    /// A vector of function indices (for funcref element type when expressions
    /// are not used).
    FuncIndices(crate::WasmVec<u32, P>),
    /// A vector of initialization expressions (for externref, or funcref with
    /// expressions). Each expression is a raw byte vector, representing a
    /// const expr.
    Expressions(crate::WasmVec<crate::WasmVec<u8, P>, P>),
}

/// Represents the initialization items for an element segment - With Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone)]
pub enum ElementInit {
    /// A vector of function indices (for funcref element type when expressions
    /// are not used).
    FuncIndices(Vec<u32>),
    /// A vector of initialization expressions (for externref, or funcref with
    /// expressions). Each expression is a raw byte vector, representing a
    /// const expr.
    Expressions(Vec<Vec<u8>>),
}

/// Mode for an element segment, determining how it's initialized - Pure No_std
/// Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone)]
pub enum ElementMode<P: wrt_foundation::MemoryProvider = wrt_foundation::NoStdProvider<1024>> {
    /// Active segment: associated with a table and an offset.
    Active {
        /// Index of the table to initialize.
        table_index: u32,
        /// Offset expression (raw bytes of a const expr).
        offset_expr: crate::WasmVec<u8, P>,
    },
    /// Passive segment: elements are not actively placed in a table at
    /// instantiation.
    Passive,
    /// Declared segment: elements are declared but not available at runtime
    /// until explicitly instantiated. Useful for some linking scenarios.
    Declared,
}

/// Mode for an element segment, determining how it's initialized - With
/// Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone)]
pub enum ElementMode {
    /// Active segment: associated with a table and an offset.
    Active {
        /// Index of the table to initialize.
        table_index: u32,
        /// Offset expression (raw bytes of a const expr).
        offset_expr: Vec<u8>,
    },
    /// Passive segment: elements are not actively placed in a table at
    /// instantiation.
    Passive,
    /// Declared segment: elements are declared but not available at runtime
    /// until explicitly instantiated. Useful for some linking scenarios.
    Declared,
}

/// WebAssembly element segment (Wasm 2.0 compatible) - Pure No_std Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone)]
pub struct Element<
    P: wrt_foundation::MemoryProvider + Clone + Default + Eq = wrt_foundation::NoStdProvider<1024>,
> {
    /// The type of elements in this segment (funcref or externref).
    pub element_type: RefType,
    /// Initialization items for the segment.
    pub init: ElementInit<P>,
    /// The mode of the element segment.
    pub mode: ElementMode<P>,
}

/// WebAssembly element segment (Wasm 2.0 compatible) - With Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone)]
pub struct Element {
    /// The type of elements in this segment (funcref or externref).
    pub element_type: RefType,
    /// Initialization items for the segment.
    pub init: ElementInit,
    /// The mode of the element segment.
    pub mode: ElementMode,
}

/// WebAssembly export - Pure No_std Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone)]
pub struct Export<
    P: wrt_foundation::MemoryProvider + Clone + Default + Eq = wrt_foundation::NoStdProvider<1024>,
> {
    /// Export name (visible external name)
    pub name: crate::WasmString<P>,
    /// Export kind (what type of item is being exported)
    pub kind: ExportKind,
    /// Export index (index into the corresponding space)
    pub index: u32,
}

/// WebAssembly export - With Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone)]
pub struct Export {
    /// Export name (visible external name)
    pub name: String,
    /// Export kind (what type of item is being exported)
    pub kind: ExportKind,
    /// Export index (index into the corresponding space)
    pub index: u32,
}

/// WebAssembly export kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    /// Function export
    Function,
    /// Table export
    Table,
    /// Memory export
    Memory,
    /// Global export
    Global,
    /// Tag export
    Tag,
}

/// WebAssembly import - Pure No_std Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone)]
pub struct Import<
    P: wrt_foundation::MemoryProvider + Clone + Default + Eq = wrt_foundation::NoStdProvider<1024>,
> {
    /// Module name (where to import from)
    pub module: crate::WasmString<P>,
    /// Import name (specific item name)
    pub name: crate::WasmString<P>,
    /// Import description (what type of item)
    pub desc: ImportDesc<P>,
}

/// WebAssembly import - With Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone)]
pub struct Import {
    /// Module name (where to import from)
    pub module: String,
    /// Import name (specific item name)
    pub name: String,
    /// Import description (what type of item)
    pub desc: ImportDesc,
}

/// WebAssembly import description - Pure No_std Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone)]
pub enum ImportDesc<P: wrt_foundation::MemoryProvider = wrt_foundation::NoStdProvider<1024>> {
    /// Function import (type index)
    Function(u32),
    /// Table import
    Table(Table),
    /// Memory import
    Memory(Memory),
    /// Global import
    Global(FormatGlobalType),
    /// Tag import (type index)
    Tag(u32),
}

/// WebAssembly import description - With Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone)]
pub enum ImportDesc {
    /// Function import (type index)
    Function(u32),
    /// Table import
    Table(Table),
    /// Memory import
    Memory(Memory),
    /// Global import
    Global(FormatGlobalType),
    /// Tag import (type index)
    Tag(u32),
}

/// Hypothetical Finding F5: Represents an entry in the TypeInformation section
/// - Pure No_std Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInformationEntry<
    P: wrt_foundation::MemoryProvider + Clone + Default + Eq = wrt_foundation::NoStdProvider<1024>,
> {
    pub type_index: u32, // Assuming TypeIdx is u32
    pub name: crate::WasmString<P>,
}

/// Hypothetical Finding F5: Represents an entry in the TypeInformation section
/// - With Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInformationEntry {
    pub type_index: u32, // Assuming TypeIdx is u32
    pub name: String,
}

/// Hypothetical Finding F5: Represents the custom TypeInformation section -
/// Pure No_std Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInformationSection<
    P: wrt_foundation::MemoryProvider + Clone + Default + Eq = wrt_foundation::NoStdProvider<1024>,
> {
    pub entries: crate::WasmVec<TypeInformationEntry<P>, P>,
}

/// Hypothetical Finding F5: Represents the custom TypeInformation section -
/// With Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeInformationSection {
    pub entries: Vec<TypeInformationEntry>,
}

/// WebAssembly module - Pure No_std Version
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone)]
pub struct Module<
    P: wrt_foundation::MemoryProvider + Clone + Default + Eq = wrt_foundation::NoStdProvider<1024>,
> {
    /// Function type signatures
    pub types: crate::WasmVec<ValueType, P>,
    /// Function definitions (code)
    pub functions: crate::WasmVec<Function<P>, P>,
    /// Table definitions
    pub tables: crate::WasmVec<Table, P>,
    /// Memory definitions  
    pub memories: crate::WasmVec<Memory, P>,
    /// Global definitions
    pub globals: crate::WasmVec<Global<P>, P>,
    /// Element segments (table initializers)
    pub elements: crate::WasmVec<Element<P>, P>,
    /// Data segments (memory initializers)
    pub data: crate::WasmVec<Data<P>, P>,
    /// Module exports (visible functions/globals/etc)
    pub exports: crate::WasmVec<Export<P>, P>,
    /// Module imports (external dependencies)
    pub imports: crate::WasmVec<Import<P>, P>,
    /// Start function index (entry point)
    pub start: Option<u32>,
    /// Custom sections (metadata)
    pub custom_sections: crate::WasmVec<CustomSection<P>, P>,
    /// Original binary data (for round-trip preservation)
    pub binary: Option<crate::WasmVec<u8, P>>,
    /// WebAssembly core version
    pub core_version: CoreWasmVersion,
    /// Type information section (if present)
    pub type_info_section: Option<TypeInformationSection<P>>,
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> Default for Module<P> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> Module<P> {
    /// Create a new empty module for no_std environments
    pub fn new() -> Self {
        Self {
            types: crate::WasmVec::new(P::default())
                .unwrap_or_else(|_| panic!("Failed to create types vector")),
            functions: crate::WasmVec::new(P::default())
                .unwrap_or_else(|_| panic!("Failed to create functions vector")),
            tables: crate::WasmVec::new(P::default())
                .unwrap_or_else(|_| panic!("Failed to create tables vector")),
            memories: crate::WasmVec::new(P::default())
                .unwrap_or_else(|_| panic!("Failed to create memories vector")),
            globals: crate::WasmVec::new(P::default())
                .unwrap_or_else(|_| panic!("Failed to create globals vector")),
            elements: crate::WasmVec::new(P::default())
                .unwrap_or_else(|_| panic!("Failed to create elements vector")),
            data: crate::WasmVec::new(P::default())
                .unwrap_or_else(|_| panic!("Failed to create data vector")),
            exports: crate::WasmVec::new(P::default())
                .unwrap_or_else(|_| panic!("Failed to create exports vector")),
            imports: crate::WasmVec::new(P::default())
                .unwrap_or_else(|_| panic!("Failed to create imports vector")),
            start: None,
            custom_sections: crate::WasmVec::new(P::default())
                .unwrap_or_else(|_| panic!("Failed to create custom_sections vector")),
            binary: None,
            core_version: CoreWasmVersion::default(),
            type_info_section: None,
        }
    }
}

/// WebAssembly module - With Allocation
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Default)]
pub struct Module {
    /// Function type signatures
    pub types: Vec<ValueType>,
    /// Function definitions (code)
    pub functions: Vec<Function>,
    /// Table definitions
    pub tables: Vec<Table>,
    /// Memory definitions
    pub memories: Vec<Memory>,
    /// Global definitions
    pub globals: Vec<Global>,
    /// Element segments (table initializers)
    pub elements: Vec<Element>,
    /// Data segments (memory initializers)
    pub data: Vec<Data>,
    /// Module exports (visible functions/globals/etc)
    pub exports: Vec<Export>,
    /// Module imports (external dependencies)
    pub imports: Vec<Import>,
    /// Start function index (entry point)
    pub start: Option<u32>,
    /// Custom sections (metadata)
    pub custom_sections: Vec<CustomSection>,
    /// Original binary data (for round-trip preservation)
    pub binary: Option<Vec<u8>>,
    /// WebAssembly core version
    pub core_version: CoreWasmVersion,
    /// Type information section (if present)
    pub type_info_section: Option<TypeInformationSection>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl Default for Module {
    fn default() -> Self {
        Self::new()
    }
}

impl Module {
    /// Create a new empty module
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            functions: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            elements: Vec::new(),
            data: Vec::new(),
            exports: Vec::new(),
            imports: Vec::new(),
            start: None,
            custom_sections: Vec::new(),
            binary: None,
            core_version: CoreWasmVersion::default(),
            type_info_section: None,
        }
    }

    /// Convert a WebAssembly binary to a Module.
    ///
    /// This is a convenience method that wraps Binary::from_bytes +
    /// Module::from_binary
    pub fn from_bytes(_wasm_bytes: &[u8]) -> Result<Self> {
        Err(Error::new(
            ErrorCategory::Validation,
            codes::PARSE_ERROR,
            "Module::from_bytes not yet implemented",
        ))
    }

    /// Convert a Module to a WebAssembly binary.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Err(Error::new(
            ErrorCategory::Validation,
            codes::PARSE_ERROR,
            "Module::to_bytes not yet implemented",
        ))
    }

    /// Find a custom section by name
    pub fn find_custom_section(&self, name: &str) -> Option<&CustomSection> {
        self.custom_sections.iter().find(|section| section.name == name)
    }

    /// Add a custom section
    pub fn add_custom_section(&mut self, section: CustomSection) {
        self.custom_sections.push(section);
    }

    /// Check if this module contains state sections
    pub fn has_state_sections(&self) -> bool {
        crate::state::has_state_sections(&self.custom_sections)
    }
}

impl Validatable for Module {
    fn validate(&self) -> Result<()> {
        // Basic validation checks

        // Check for reasonable number of types
        if self.types.len() > 10000 {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Module has too many types",
            ));
        }

        // Check for reasonable number of functions
        if self.functions.len() > 10000 {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Module has too many functions",
            ));
        }

        // Check for empty exports
        for export in &self.exports {
            if export.name.is_empty() {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    "Export name cannot be empty",
                ));
            }
        }

        // Check for empty imports
        for import in &self.imports {
            if import.module.is_empty() {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    "Import module name cannot be empty",
                ));
            }

            if import.name.is_empty() {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    "Import name cannot be empty",
                ));
            }
        }

        Ok(())
    }
}

// Serialization helpers for Table
impl Table {
    /// Serialize to bytes
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn to_bytes(&self) -> wrt_foundation::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.push(self.element_type.to_binary());
        bytes.extend(self.limits.to_bytes()?);
        Ok(bytes)
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> wrt_foundation::Result<Self> {
        if bytes.len() < 2 {
            return Err(wrt_error::Error::new(
                wrt_error::ErrorCategory::Validation,
                wrt_error::codes::PARSE_ERROR,
                "Insufficient bytes for Table",
            ));
        }

        let element_type = ValueType::from_binary(bytes[0]).ok_or(wrt_error::Error::new(
            wrt_error::ErrorCategory::Validation,
            wrt_error::codes::PARSE_ERROR,
            "Invalid element type",
        ))?;
        let limits = Limits::from_bytes(&bytes[1..])?;

        Ok(Self { element_type, limits })
    }
}

// Implement Checksummable trait for Table
impl wrt_foundation::traits::Checksummable for Table {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        self.element_type.update_checksum(checksum);
        self.limits.update_checksum(checksum);
    }
}

// Serialization helpers for Memory
impl Memory {
    /// Serialize to bytes
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn to_bytes(&self) -> wrt_foundation::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.extend(self.limits.to_bytes()?);
        bytes.push(self.shared as u8);
        Ok(bytes)
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> wrt_foundation::Result<Self> {
        if bytes.len() < 2 {
            return Err(wrt_error::Error::new(
                wrt_error::ErrorCategory::Validation,
                wrt_error::codes::PARSE_ERROR,
                "Insufficient bytes for Memory",
            ));
        }

        let limits = Limits::from_bytes(&bytes[..bytes.len() - 1])?;
        let shared = bytes[bytes.len() - 1] != 0;

        Ok(Self { limits, shared })
    }
}

// Implement Checksummable trait for Memory
impl wrt_foundation::traits::Checksummable for Memory {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        self.limits.update_checksum(checksum);
        checksum.update_slice(&[self.shared as u8]);
    }
}

#[cfg(test)]
mod tests {

    // ... existing test code ...
}
