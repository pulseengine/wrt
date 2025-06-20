//! Simplified WebAssembly module representation
//!
//! This module provides a complete but simplified representation of a WebAssembly
//! module that can be built incrementally during streaming parsing.

use crate::bounded_types::SimpleBoundedVec;
use crate::types::{ValueType, FuncType, GlobalType, MemoryType, TableType};
use wrt_error::Result;

/// A complete WebAssembly module representation
#[derive(Debug, Clone)]
pub struct SimpleModule {
    /// Type section - function signatures
    pub types: SimpleBoundedVec<FuncType, 512>,
    /// Function section - type indices for functions
    pub functions: SimpleBoundedVec<u32, 4096>,
    /// Table section
    pub tables: SimpleBoundedVec<TableType, 128>,
    /// Memory section
    pub memories: SimpleBoundedVec<MemoryType, 128>,
    /// Global section
    pub globals: SimpleBoundedVec<GlobalType, 512>,
    /// Export section
    pub exports: SimpleBoundedVec<Export, 512>,
    /// Import section
    pub imports: SimpleBoundedVec<Import, 512>,
    /// Start function index
    pub start: Option<u32>,
    /// Code section - function bodies
    pub code: SimpleBoundedVec<FunctionBody, 4096>,
    /// Data section
    pub data: SimpleBoundedVec<DataSegment, 512>,
    /// Element section
    pub elements: SimpleBoundedVec<ElementSegment, 512>,
}

impl SimpleModule {
    /// Create a new empty module
    pub fn new() -> Self {
        SimpleModule {
            types: SimpleBoundedVec::new(),
            functions: SimpleBoundedVec::new(),
            tables: SimpleBoundedVec::new(),
            memories: SimpleBoundedVec::new(),
            globals: SimpleBoundedVec::new(),
            exports: SimpleBoundedVec::new(),
            imports: SimpleBoundedVec::new(),
            start: None,
            code: SimpleBoundedVec::new(),
            data: SimpleBoundedVec::new(),
            elements: SimpleBoundedVec::new(),
        }
    }
}

impl Default for SimpleModule {
    fn default() -> Self {
        Self::new()
    }
}

/// Export entry
#[derive(Debug, Clone)]
pub struct Export {
    pub name: SimpleBoundedVec<u8, 256>,
    pub kind: ExportKind,
    pub index: u32,
}

/// Export kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Func,
    Table,
    Memory,
    Global,
}

/// Import entry
#[derive(Debug, Clone)]
pub struct Import {
    pub module: SimpleBoundedVec<u8, 256>,
    pub name: SimpleBoundedVec<u8, 256>,
    pub desc: ImportDesc,
}

/// Import description
#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(u32), // Type index
    Table(TableType),
    Memory(MemoryType),
    Global(GlobalType),
}

/// Function body
#[derive(Debug, Clone)]
pub struct FunctionBody {
    pub locals: SimpleBoundedVec<LocalDecl, 64>,
    pub code: SimpleBoundedVec<u8, 65536>,
}

/// Local variable declaration
#[derive(Debug, Clone)]
pub struct LocalDecl {
    pub count: u32,
    pub value_type: ValueType,
}

/// Data segment
#[derive(Debug, Clone)]
pub struct DataSegment {
    pub memory_index: u32,
    pub offset: SimpleBoundedVec<u8, 256>, // Constant expression
    pub data: SimpleBoundedVec<u8, 65536>,
}

/// Element segment
#[derive(Debug, Clone)]
pub struct ElementSegment {
    pub table_index: u32,
    pub offset: SimpleBoundedVec<u8, 256>, // Constant expression
    pub init: SimpleBoundedVec<u32, 1024>, // Function indices
}