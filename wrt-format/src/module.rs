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
#[cfg(not(any(feature = "alloc", feature = "std")))]
use wrt_foundation::{BoundedString, BoundedVec, MemoryProvider, NoStdProvider};
use wrt_foundation::{RefType, ValueType};

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
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> Default for Function<P> {
    fn default() -> Self {
        Function { type_idx: 0, locals: crate::WasmVec::new(), code: crate::WasmVec::new() }
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::Checksummable
    for Function<P>
{
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        checksum.update(&self.type_idx.to_le_bytes());
        self.locals.update_checksum(checksum);
        self.code.update_checksum(checksum);
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::ToBytes
    for Function<P>
{
    fn to_bytes_with_provider<PStream>(
        &self,
        stream: &mut wrt_foundation::traits::WriteStream,
        provider: &PStream,
    ) -> Result<(), wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        stream.write(&self.type_idx.to_le_bytes())?;
        self.locals.to_bytes_with_provider(stream, provider)?;
        self.code.to_bytes_with_provider(stream, provider)?;
        Ok(())
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::FromBytes
    for Function<P>
{
    fn from_bytes_with_provider<PStream>(
        stream: &mut wrt_foundation::traits::ReadStream,
        provider: &PStream,
    ) -> Result<Self, wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        let mut idx_bytes = [0u8; 4];
        stream.read(&mut idx_bytes)?;
        let type_idx = u32::from_le_bytes(idx_bytes);
        let locals = crate::WasmVec::from_bytes_with_provider(stream, provider)?;
        let code = crate::WasmVec::from_bytes_with_provider(stream, provider)?;

        Ok(Function { type_idx, locals, code })
    }
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Global<
    P: wrt_foundation::MemoryProvider + Clone + Default + Eq = wrt_foundation::NoStdProvider<1024>,
> {
    /// Global type
    pub global_type: FormatGlobalType,
    /// Initialization expression
    pub init: crate::WasmVec<u8, P>,
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> Default for Global<P> {
    fn default() -> Self {
        Global { global_type: FormatGlobalType::default(), init: crate::WasmVec::new() }
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::Checksummable
    for Global<P>
{
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        self.global_type.update_checksum(checksum);
        self.init.update_checksum(checksum);
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::ToBytes
    for Global<P>
{
    fn to_bytes_with_provider<PStream>(
        &self,
        stream: &mut wrt_foundation::traits::WriteStream,
        provider: &PStream,
    ) -> Result<(), wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        self.global_type.to_bytes_with_provider(stream, provider)?;
        self.init.to_bytes_with_provider(stream, provider)?;
        Ok(())
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::FromBytes
    for Global<P>
{
    fn from_bytes_with_provider<PStream>(
        stream: &mut wrt_foundation::traits::ReadStream,
        provider: &PStream,
    ) -> Result<Self, wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        let global_type = FormatGlobalType::from_bytes_with_provider(stream, provider)?;
        let init = crate::WasmVec::from_bytes_with_provider(stream, provider)?;

        Ok(Global { global_type, init })
    }
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> Default for Export<P> {
    fn default() -> Self {
        Export { name: crate::WasmString::default(), kind: ExportKind::Function, index: 0 }
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::Checksummable
    for Export<P>
{
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        self.name.update_checksum(checksum);
        checksum.update(&[self.kind as u8]);
        checksum.update(&self.index.to_le_bytes());
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::ToBytes
    for Export<P>
{
    fn to_bytes_with_provider<PStream>(
        &self,
        stream: &mut wrt_foundation::traits::WriteStream,
        provider: &PStream,
    ) -> Result<(), wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        self.name.to_bytes_with_provider(stream, provider)?;
        stream.write(&[self.kind as u8])?;
        stream.write(&self.index.to_le_bytes())?;
        Ok(())
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::FromBytes
    for Export<P>
{
    fn from_bytes_with_provider<PStream>(
        stream: &mut wrt_foundation::traits::ReadStream,
        provider: &PStream,
    ) -> Result<Self, wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        let name = crate::WasmString::from_bytes_with_provider(stream, provider)?;
        let mut kind_byte = [0u8; 1];
        stream.read(&mut kind_byte)?;
        let kind = match kind_byte[0] {
            0 => ExportKind::Function,
            1 => ExportKind::Table,
            2 => ExportKind::Memory,
            3 => ExportKind::Global,
            4 => ExportKind::Tag,
            _ => ExportKind::Function, // Default fallback
        };
        let mut idx_bytes = [0u8; 4];
        stream.read(&mut idx_bytes)?;
        let index = u32::from_le_bytes(idx_bytes);

        Ok(Export { name, kind, index })
    }
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportDesc<P: wrt_foundation::MemoryProvider = wrt_foundation::NoStdProvider<1024>> {
    /// Function import (type index)
    Function(u32, core::marker::PhantomData<P>),
    /// Table import
    Table(Table, core::marker::PhantomData<P>),
    /// Memory import
    Memory(Memory, core::marker::PhantomData<P>),
    /// Global import
    Global(FormatGlobalType, core::marker::PhantomData<P>),
    /// Tag import (type index)
    Tag(u32, core::marker::PhantomData<P>),
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> Default for ImportDesc<P> {
    fn default() -> Self {
        ImportDesc::Function(0, core::marker::PhantomData)
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> Default for Import<P> {
    fn default() -> Self {
        Import {
            module: crate::WasmString::default(),
            name: crate::WasmString::default(),
            desc: ImportDesc::default(),
        }
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::Checksummable
    for ImportDesc<P>
{
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        match self {
            ImportDesc::Function(idx, _) => {
                checksum.update(&idx.to_le_bytes());
            }
            ImportDesc::Table(_, _) => {
                checksum.update(&[0x01]);
            }
            ImportDesc::Memory(_, _) => {
                checksum.update(&[0x02]);
            }
            ImportDesc::Global(_, _) => {
                checksum.update(&[0x03]);
            }
            ImportDesc::Tag(idx, _) => {
                checksum.update(&idx.to_le_bytes());
            }
        }
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::Checksummable
    for Import<P>
{
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        self.module.update_checksum(checksum);
        self.name.update_checksum(checksum);
        self.desc.update_checksum(checksum);
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::ToBytes
    for ImportDesc<P>
{
    fn to_bytes_with_provider<PStream>(
        &self,
        stream: &mut wrt_foundation::traits::WriteStream,
        _provider: &PStream,
    ) -> Result<(), wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        match self {
            ImportDesc::Function(idx, _) => {
                stream.write(&[0x00])?; // Function type tag
                stream.write(&idx.to_le_bytes())?;
            }
            ImportDesc::Table(_, _) => {
                stream.write(&[0x01])?; // Table type tag
            }
            ImportDesc::Memory(_, _) => {
                stream.write(&[0x02])?; // Memory type tag
            }
            ImportDesc::Global(_, _) => {
                stream.write(&[0x03])?; // Global type tag
            }
            ImportDesc::Tag(idx, _) => {
                stream.write(&[0x04])?; // Tag type tag
                stream.write(&idx.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::ToBytes
    for Import<P>
{
    fn to_bytes_with_provider<PStream>(
        &self,
        stream: &mut wrt_foundation::traits::WriteStream,
        provider: &PStream,
    ) -> Result<(), wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        self.module.to_bytes_with_provider(stream, provider)?;
        self.name.to_bytes_with_provider(stream, provider)?;
        self.desc.to_bytes_with_provider(stream, provider)?;
        Ok(())
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::FromBytes
    for ImportDesc<P>
{
    fn from_bytes_with_provider<PStream>(
        stream: &mut wrt_foundation::traits::ReadStream,
        _provider: &PStream,
    ) -> Result<Self, wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        let mut tag = [0u8; 1];
        stream.read(&mut tag)?;

        match tag[0] {
            0x00 => {
                // Function
                let mut idx_bytes = [0u8; 4];
                stream.read(&mut idx_bytes)?;
                let idx = u32::from_le_bytes(idx_bytes);
                Ok(ImportDesc::Function(idx, core::marker::PhantomData))
            }
            0x01 => {
                // Table
                Ok(ImportDesc::Table(Table::default(), core::marker::PhantomData))
            }
            0x02 => {
                // Memory
                Ok(ImportDesc::Memory(Memory::default(), core::marker::PhantomData))
            }
            0x03 => {
                // Global
                Ok(ImportDesc::Global(FormatGlobalType::default(), core::marker::PhantomData))
            }
            0x04 => {
                // Tag
                let mut idx_bytes = [0u8; 4];
                stream.read(&mut idx_bytes)?;
                let idx = u32::from_le_bytes(idx_bytes);
                Ok(ImportDesc::Tag(idx, core::marker::PhantomData))
            }
            _ => Err(wrt_error::Error::new(
                wrt_error::ErrorCategory::Validation,
                0x1001,
                "ImportDesc: Unknown type tag",
            )),
        }
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::FromBytes
    for Import<P>
{
    fn from_bytes_with_provider<PStream>(
        stream: &mut wrt_foundation::traits::ReadStream,
        provider: &PStream,
    ) -> Result<Self, wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        let module = crate::WasmString::from_bytes_with_provider(stream, provider)?;
        let name = crate::WasmString::from_bytes_with_provider(stream, provider)?;
        let desc = ImportDesc::from_bytes_with_provider(stream, provider)?;

        Ok(Import { module, name, desc })
    }
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

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> Default for TypeInformationEntry<P> {
    fn default() -> Self {
        TypeInformationEntry { type_index: 0, name: crate::WasmString::default() }
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::Checksummable
    for TypeInformationEntry<P>
{
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        checksum.update(&self.type_index.to_le_bytes());
        self.name.update_checksum(checksum);
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::ToBytes
    for TypeInformationEntry<P>
{
    fn to_bytes_with_provider<PStream>(
        &self,
        stream: &mut wrt_foundation::traits::WriteStream,
        provider: &PStream,
    ) -> Result<(), wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        stream.write(&self.type_index.to_le_bytes())?;
        self.name.to_bytes_with_provider(stream, provider)?;
        Ok(())
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl<P: wrt_foundation::MemoryProvider + Clone + Default + Eq> wrt_foundation::traits::FromBytes
    for TypeInformationEntry<P>
{
    fn from_bytes_with_provider<PStream>(
        stream: &mut wrt_foundation::traits::ReadStream,
        provider: &PStream,
    ) -> Result<Self, wrt_error::Error>
    where
        PStream: wrt_foundation::MemoryProvider,
    {
        let mut idx_bytes = [0u8; 4];
        stream.read(&mut idx_bytes)?;
        let type_index = u32::from_le_bytes(idx_bytes);
        let name = crate::WasmString::from_bytes_with_provider(stream, provider)?;

        Ok(TypeInformationEntry { type_index, name })
    }
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
#[derive(Debug, Clone)]
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
