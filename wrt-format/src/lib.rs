//! WebAssembly format handling for WRT.
//!
//! This crate provides utilities for working with WebAssembly binary formats,
//! including serialization and deserialization of WebAssembly modules and state.

#![cfg_attr(not(feature = "std"), no_std)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(not(feature = "std"))]
extern crate alloc;

// Import std/alloc collections based on feature flag
#[cfg(feature = "std")]
pub use std::{
    boxed::Box,
    collections::{HashMap, HashSet},
    format,
    string::String,
    vec::Vec,
};

#[cfg(not(feature = "std"))]
pub use alloc::{
    boxed::Box,
    collections::{BTreeMap as HashMap, BTreeSet as HashSet},
    format,
    string::String,
    vec::Vec,
};

pub mod binary;
pub mod component;
pub mod compression;
pub mod module;
pub mod section;
pub mod state;
pub mod types;
pub mod version;

pub use component::Component;
pub use compression::{rle_decode, rle_encode, CompressionType};
pub use module::Module;
pub use section::{CustomSection, Section};
pub use state::{create_state_section, extract_state_section, StateSection};
pub use types::{parse_value_type, value_type_to_byte, BlockType, FuncType, Limits, ValueType};
pub use version::STATE_VERSION;
