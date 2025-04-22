//! WebAssembly type utilities
//!
//! This module provides utilities for working with WebAssembly types.
//! Type definitions are imported from wrt-types.

// Import types directly from wrt-types
pub use wrt_types::{safe_memory::SafeSlice, FuncType, ValueType};

// Import format-specific types and functions from wrt-format
pub use wrt_format::types::{
    parse_value_type, value_type_to_byte, BlockType, Limits, MemoryIndexType,
};

// Use the block type parsing from wrt-format
pub use wrt_format::binary::parse_block_type;
