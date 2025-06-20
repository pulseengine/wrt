//! Module builder for constructing WebAssembly modules during parsing
//!
//! This module provides the Module type and associated builder functionality
//! for incrementally constructing a WebAssembly module during streaming parsing.

use wrt_foundation::bounded::BoundedVec;
use wrt_foundation::safe_memory::MemoryProvider;
use wrt_error::Result;
use crate::types::{FuncType, GlobalType, MemoryType, TableType, ValueType};

/// A WebAssembly module
#[derive(Debug, Clone)]
pub struct Module<P: MemoryProvider + Clone + Eq> {
    pub types: BoundedVec<FuncType, 512, P>,
    pub functions: BoundedVec<Function, 4096, P>,
    pub tables: BoundedVec<Table, 128, P>,
    pub memories: BoundedVec<Memory, 128, P>,
    pub globals: BoundedVec<Global<P>, 512, P>,
    pub exports: BoundedVec<Export<P>, 512, P>,
    pub imports: BoundedVec<Import<P>, 512, P>,
    pub elements: BoundedVec<Element<P>, 512, P>,
    pub data: BoundedVec<Data<P>, 512, P>,
    pub start: Option<u32>,
}

impl<P: MemoryProvider + Clone + Eq> Module<P> {
    /// Create a new empty module
    pub fn new(provider: P) -> Result<Self> {
        Ok(Module {
            types: BoundedVec::new(provider.clone())?,
            functions: BoundedVec::new(provider.clone())?,
            tables: BoundedVec::new(provider.clone())?,
            memories: BoundedVec::new(provider.clone())?,
            globals: BoundedVec::new(provider.clone())?,
            exports: BoundedVec::new(provider.clone())?,
            imports: BoundedVec::new(provider.clone())?,
            elements: BoundedVec::new(provider.clone())?,
            data: BoundedVec::new(provider.clone())?,
            start: None,
        })
    }
}

impl<P: MemoryProvider + Clone + Eq> Default for Module<P> 
where 
    P: Default 
{
    fn default() -> Self {
        Self::new(P::default()).expect("Default provider should not fail")
    }
}

/// A WebAssembly function
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Function {
    pub type_idx: u32,
    pub locals: BoundedVec<Local, 64, crate::ParserProvider>,
    pub code: BoundedVec<u8, 65536, crate::ParserProvider>,
}

impl Function {
    /// Create a new function
    pub fn new(type_idx: u32, provider: crate::ParserProvider) -> Result<Self> {
        Ok(Function {
            type_idx,
            locals: BoundedVec::new(provider.clone())?,
            code: BoundedVec::new(provider)?,
        })
    }
}

/// A local variable declaration
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Local {
    pub count: u32,
    pub value_type: ValueType,
}

/// A WebAssembly table
#[derive(Debug, Clone)]
pub struct Table {
    pub table_type: TableType,
}

/// A WebAssembly memory
#[derive(Debug, Clone)]
pub struct Memory {
    pub memory_type: MemoryType,
}

/// A WebAssembly global
#[derive(Debug, Clone)]
pub struct Global<P: MemoryProvider> {
    pub global_type: GlobalType,
    pub init: BoundedVec<u8, 256, P>, // Constant expression
}

impl<P: MemoryProvider> Global<P> {
    /// Create a new global
    pub fn new(global_type: GlobalType, provider: P) -> Result<Self> {
        Ok(Global {
            global_type,
            init: BoundedVec::new(provider)?,
        })
    }
}

/// A WebAssembly export
#[derive(Debug, Clone)]
pub struct Export<P: MemoryProvider> {
    pub name: BoundedVec<u8, 256, P>,
    pub desc: ExportDesc,
}

impl<P: MemoryProvider> Export<P> {
    /// Create a new export
    pub fn new(provider: P) -> Result<Self> {
        Ok(Export {
            name: BoundedVec::new(provider)?,
            desc: ExportDesc::Func(0),
        })
    }
}

/// Export description
#[derive(Debug, Clone)]
pub enum ExportDesc {
    Func(u32),
    Table(u32),
    Memory(u32),
    Global(u32),
}

/// A WebAssembly import
#[derive(Debug, Clone)]
pub struct Import<P: MemoryProvider> {
    pub module: BoundedVec<u8, 256, P>,
    pub name: BoundedVec<u8, 256, P>,
    pub desc: ImportDesc,
}

impl<P: MemoryProvider> Import<P> {
    /// Create a new import
    pub fn new(provider: P) -> Result<Self> {
        Ok(Import {
            module: BoundedVec::new(provider.clone())?,
            name: BoundedVec::new(provider)?,
            desc: ImportDesc::Func(0),
        })
    }
}

/// Import description
#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(u32),
    Table(TableType),
    Memory(MemoryType),
    Global(GlobalType),
}

/// A WebAssembly element segment
#[derive(Debug, Clone)]
pub struct Element<P: MemoryProvider> {
    pub table_idx: Option<u32>,
    pub offset: BoundedVec<u8, 256, P>, // Constant expression
    pub init: BoundedVec<u32, 1024, P>, // Function indices
}

impl<P: MemoryProvider> Element<P> {
    /// Create a new element segment
    pub fn new(provider: P) -> Result<Self> {
        Ok(Element {
            table_idx: None,
            offset: BoundedVec::new(provider.clone())?,
            init: BoundedVec::new(provider)?,
        })
    }
}

/// A WebAssembly data segment
#[derive(Debug, Clone)]
pub struct Data<P: MemoryProvider> {
    pub memory_idx: Option<u32>,
    pub offset: BoundedVec<u8, 256, P>, // Constant expression
    pub init: BoundedVec<u8, 65536, P>, // Data bytes
}

impl<P: MemoryProvider> Data<P> {
    /// Create a new data segment
    pub fn new(provider: P) -> Result<Self> {
        Ok(Data {
            memory_idx: None,
            offset: BoundedVec::new(provider.clone())?,
            init: BoundedVec::new(provider)?,
        })
    }
}