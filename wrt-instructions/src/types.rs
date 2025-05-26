//! Type aliases for no_std compatibility

use crate::prelude::*;

// Collection type aliases that work across all configurations
#[cfg(feature = "alloc")]
pub type InstructionVec<T> = Vec<T>;

#[cfg(not(feature = "alloc"))]
pub type InstructionVec<T> = BoundedVec<T, 256, wrt_foundation::DefaultNoStdProvider>;

// Stack type with reasonable size for WASM
pub const MAX_STACK_SIZE: usize = 1024;

#[cfg(feature = "alloc")]
pub type ValueStack = Vec<Value>;

#[cfg(not(feature = "alloc"))]
pub type ValueStack = BoundedStack<Value, MAX_STACK_SIZE>;

// Table storage
pub const MAX_TABLES: usize = 16;
pub const MAX_TABLE_SIZE: usize = 65536;

#[cfg(feature = "alloc")]
pub type TableVec = Vec<Vec<RefValue>>;

#[cfg(not(feature = "alloc"))]
pub type TableVec = BoundedVec<BoundedVec<RefValue, MAX_TABLE_SIZE, wrt_foundation::DefaultNoStdProvider>, MAX_TABLES, wrt_foundation::DefaultNoStdProvider>;

// Locals and globals storage
pub const MAX_LOCALS: usize = 1024;
pub const MAX_GLOBALS: usize = 1024;

#[cfg(feature = "alloc")]
pub type LocalsVec = Vec<Value>;

#[cfg(not(feature = "alloc"))]
pub type LocalsVec = BoundedVec<Value, MAX_LOCALS, wrt_foundation::DefaultNoStdProvider>;

#[cfg(feature = "alloc")]
pub type GlobalsVec = Vec<Value>;

#[cfg(not(feature = "alloc"))]
pub type GlobalsVec = BoundedVec<Value, MAX_GLOBALS, wrt_foundation::DefaultNoStdProvider>;

// Reference value type (for tables)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RefValue {
    /// Function reference
    FuncRef(Option<u32>),
    /// External reference  
    ExternRef(Option<u32>),
}

// Helper to create vectors in both modes
#[cfg(feature = "alloc")]
#[macro_export]
macro_rules! make_vec {
    () => { Vec::new() };
    ($($elem:expr),*) => { vec![$($elem),*] };
}

#[cfg(not(feature = "alloc"))]
#[macro_export]
macro_rules! make_vec {
    () => { BoundedVec::new() };
    ($($elem:expr),*) => {{
        let mut v = BoundedVec::new();
        $(v.push($elem).unwrap();)*
        v
    }};
}