//! Type aliases for no_std compatibility in wrt-runtime

use crate::prelude::*;
use wrt_foundation::{BoundedVec, BoundedMap, NoStdProvider};

// Runtime execution limits
pub const MAX_STACK_DEPTH: usize = 1024;
pub const MAX_CALL_STACK: usize = 512;
pub const MAX_VALUE_STACK: usize = 65536;
pub const MAX_LOCALS: usize = 50000; // WebAssembly spec limit
pub const MAX_GLOBALS: usize = 1024;
pub const MAX_FUNCTIONS: usize = 1024;
pub const MAX_IMPORTS: usize = 512;
pub const MAX_EXPORTS: usize = 512;
pub const MAX_TABLES: usize = 64;
pub const MAX_MEMORIES: usize = 64;
pub const MAX_ELEMENTS: usize = 512;
pub const MAX_DATA: usize = 512;

// Memory management
pub const MAX_MEMORY_PAGES: usize = 65536; // 4GB limit
pub const MAX_TABLE_ENTRIES: usize = 1048576; // 1M entries
pub const MAX_STRING_LENGTH: usize = 256;

// Module instance limits
pub const MAX_MODULE_INSTANCES: usize = 256;
pub const MAX_FUNCTION_BODIES: usize = 1024;
pub const MAX_BRANCH_TABLE_TARGETS: usize = 1024;

// CFI and instrumentation
pub const MAX_CFI_CHECKS: usize = 1024;
pub const MAX_INSTRUMENTATION_POINTS: usize = 2048;

// Runtime state vectors
#[cfg(feature = "alloc")]
pub type ValueStackVec = Vec<wrt_foundation::Value>;
#[cfg(not(feature = "alloc"))]
pub type ValueStackVec = BoundedVec<wrt_foundation::Value, MAX_VALUE_STACK, NoStdProvider<{ MAX_VALUE_STACK * 16 }>>;

#[cfg(feature = "alloc")]
pub type CallStackVec = Vec<crate::execution::CallFrame>;
#[cfg(not(feature = "alloc"))]
pub type CallStackVec = BoundedVec<crate::execution::CallFrame, MAX_CALL_STACK, NoStdProvider<{ MAX_CALL_STACK * 128 }>>;

#[cfg(feature = "alloc")]
pub type LocalsVec = Vec<wrt_foundation::Value>;
#[cfg(not(feature = "alloc"))]
pub type LocalsVec = BoundedVec<wrt_foundation::Value, MAX_LOCALS, NoStdProvider<{ MAX_LOCALS * 16 }>>;

#[cfg(feature = "alloc")]
pub type GlobalsVec = Vec<crate::global::Global>;
#[cfg(not(feature = "alloc"))]
pub type GlobalsVec = BoundedVec<crate::global::Global, MAX_GLOBALS, NoStdProvider<{ MAX_GLOBALS * 64 }>>;

#[cfg(feature = "alloc")]
pub type FunctionsVec = Vec<crate::func::Function>;
#[cfg(not(feature = "alloc"))]
pub type FunctionsVec = BoundedVec<crate::func::Function, MAX_FUNCTIONS, NoStdProvider<{ MAX_FUNCTIONS * 256 }>>;

#[cfg(feature = "alloc")]
pub type ImportsVec<T> = Vec<T>;
#[cfg(not(feature = "alloc"))]
pub type ImportsVec<T> = BoundedVec<T, MAX_IMPORTS, NoStdProvider<{ MAX_IMPORTS * 128 }>>;

#[cfg(feature = "alloc")]
pub type ExportsVec<T> = Vec<T>;
#[cfg(not(feature = "alloc"))]
pub type ExportsVec<T> = BoundedVec<T, MAX_EXPORTS, NoStdProvider<{ MAX_EXPORTS * 64 }>>;

#[cfg(feature = "alloc")]
pub type TablesVec = Vec<crate::table::Table>;
#[cfg(not(feature = "alloc"))]
pub type TablesVec = BoundedVec<crate::table::Table, MAX_TABLES, NoStdProvider<{ MAX_TABLES * 1024 }>>;

#[cfg(feature = "alloc")]
pub type MemoriesVec = Vec<crate::memory::Memory>;
#[cfg(not(feature = "alloc"))]
pub type MemoriesVec = BoundedVec<crate::memory::Memory, MAX_MEMORIES, NoStdProvider<{ MAX_MEMORIES * 1024 }>>;

#[cfg(feature = "alloc")]
pub type ElementsVec = Vec<wrt_foundation::types::ElementSegment>;
#[cfg(not(feature = "alloc"))]
pub type ElementsVec = BoundedVec<wrt_foundation::types::ElementSegment, MAX_ELEMENTS, NoStdProvider<{ MAX_ELEMENTS * 256 }>>;

#[cfg(feature = "alloc")]
pub type DataVec = Vec<wrt_foundation::types::DataSegment>;
#[cfg(not(feature = "alloc"))]
pub type DataVec = BoundedVec<wrt_foundation::types::DataSegment, MAX_DATA, NoStdProvider<{ MAX_DATA * 256 }>>;

// Instruction vectors
#[cfg(feature = "alloc")]
// Instructions module is temporarily disabled in wrt-decoder
// pub type InstructionVec = Vec<wrt_decoder::instructions::Instruction>;
pub type InstructionVec = Vec<wrt_instructions::Instruction>;
#[cfg(not(feature = "alloc"))]
pub type InstructionVec = BoundedVec<wrt_instructions::Instruction, 65536, NoStdProvider<{ 65536 * 8 }>>;

#[cfg(feature = "alloc")]
pub type BranchTargetsVec = Vec<u32>;
#[cfg(not(feature = "alloc"))]
pub type BranchTargetsVec = BoundedVec<u32, MAX_BRANCH_TABLE_TARGETS, NoStdProvider<{ MAX_BRANCH_TABLE_TARGETS * 4 }>>;

// Module instance vectors
#[cfg(feature = "alloc")]
pub type ModuleInstanceVec = Vec<crate::module_instance::ModuleInstance>;
#[cfg(not(feature = "alloc"))]
pub type ModuleInstanceVec = BoundedVec<crate::module_instance::ModuleInstance, MAX_MODULE_INSTANCES, NoStdProvider<{ MAX_MODULE_INSTANCES * 1024 }>>;

#[cfg(feature = "alloc")]
pub type FunctionBodiesVec = Vec<Vec<u8>>;
#[cfg(not(feature = "alloc"))]
pub type FunctionBodiesVec = BoundedVec<BoundedVec<u8, 65536, NoStdProvider<65536>>, MAX_FUNCTION_BODIES, NoStdProvider<{ MAX_FUNCTION_BODIES * 65536 }>>;

// Memory and table data
#[cfg(feature = "alloc")]
pub type MemoryDataVec = Vec<u8>;
#[cfg(not(feature = "alloc"))]
pub type MemoryDataVec = BoundedVec<u8, { 64 * 1024 * 1024 }, NoStdProvider<{ 64 * 1024 * 1024 }>>; // 64MB max

#[cfg(feature = "alloc")]
pub type TableDataVec = Vec<Option<wrt_foundation::RefValue>>;
#[cfg(not(feature = "alloc"))]
pub type TableDataVec = BoundedVec<Option<wrt_foundation::RefValue>, MAX_TABLE_ENTRIES, NoStdProvider<{ MAX_TABLE_ENTRIES * 32 }>>;

// String type for runtime
#[cfg(feature = "alloc")]
pub type RuntimeString = String;
#[cfg(not(feature = "alloc"))]
pub type RuntimeString = wrt_foundation::BoundedString<MAX_STRING_LENGTH, NoStdProvider<MAX_STRING_LENGTH>>;

// Maps for runtime state
#[cfg(feature = "alloc")]
pub type FunctionMap = HashMap<u32, crate::func::Function>;
#[cfg(not(feature = "alloc"))]
pub type FunctionMap = BoundedMap<u32, crate::func::Function, MAX_FUNCTIONS, NoStdProvider<{ MAX_FUNCTIONS * 256 }>>;

#[cfg(feature = "alloc")]
pub type GlobalMap = HashMap<u32, crate::global::Global>;
#[cfg(not(feature = "alloc"))]
pub type GlobalMap = BoundedMap<u32, crate::global::Global, MAX_GLOBALS, NoStdProvider<{ MAX_GLOBALS * 64 }>>;

#[cfg(feature = "alloc")]
pub type MemoryMap = HashMap<u32, crate::memory::Memory>;
#[cfg(not(feature = "alloc"))]
pub type MemoryMap = BoundedMap<u32, crate::memory::Memory, MAX_MEMORIES, NoStdProvider<{ MAX_MEMORIES * 1024 }>>;

#[cfg(feature = "alloc")]
pub type TableMap = HashMap<u32, crate::table::Table>;
#[cfg(not(feature = "alloc"))]
pub type TableMap = BoundedMap<u32, crate::table::Table, MAX_TABLES, NoStdProvider<{ MAX_TABLES * 1024 }>>;

// CFI and instrumentation types
#[cfg(feature = "alloc")]
pub type CfiCheckVec = Vec<crate::cfi_engine::CfiCheck>;
#[cfg(not(feature = "alloc"))]
pub type CfiCheckVec = BoundedVec<crate::cfi_engine::CfiCheck, MAX_CFI_CHECKS, NoStdProvider<{ MAX_CFI_CHECKS * 64 }>>;

#[cfg(feature = "alloc")]
pub type InstrumentationVec = Vec<crate::execution::InstrumentationPoint>;
#[cfg(not(feature = "alloc"))]
pub type InstrumentationVec = BoundedVec<crate::execution::InstrumentationPoint, MAX_INSTRUMENTATION_POINTS, NoStdProvider<{ MAX_INSTRUMENTATION_POINTS * 32 }>>;

// Generic byte vector for raw data
#[cfg(feature = "alloc")]
pub type ByteVec = Vec<u8>;
#[cfg(not(feature = "alloc"))]
pub type ByteVec = BoundedVec<u8, 65536, NoStdProvider<65536>>;

// Error collection for batch operations
#[cfg(feature = "alloc")]
pub type ErrorVec = Vec<wrt_error::Error>;
#[cfg(not(feature = "alloc"))]
pub type ErrorVec = BoundedVec<wrt_error::Error, 256, NoStdProvider<{ 256 * 256 }>>;