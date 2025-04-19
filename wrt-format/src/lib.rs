//! WebAssembly format handling for WRT.
//!
//! This crate provides utilities for working with WebAssembly binary formats,
//! including serialization and deserialization of WebAssembly modules and state.

pub mod binary;
pub mod compression;
pub mod module;
pub mod section;
pub mod state;
pub mod types;
pub mod version;

pub use compression::{rle_decode, rle_encode, CompressionType};
pub use module::Module;
pub use section::{CustomSection, Section};
pub use state::{create_state_section, extract_state_section, StateSection};
pub use types::{parse_value_type, value_type_to_byte, BlockType, FuncType, Limits, ValueType};
pub use version::STATE_VERSION;
