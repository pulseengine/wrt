//! WebAssembly type utilities
//!
//! This module provides utilities for working with WebAssembly types.
//! Most type definitions are re-exported from wrt-format.

// Re-export types from wrt-format
pub use wrt_format::types::{
    parse_value_type, value_type_to_byte, BlockType, FuncType, Limits, ValueType,
};

// Use the block type parsing from wrt-format
pub use wrt_format::binary::parse_block_type;
