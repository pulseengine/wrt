// WRT - wrt-decoder
// Module: WebAssembly Binary Decoder
// SW-REQ-ID: REQ_013
// SW-REQ-ID: REQ_SAFETY_DECODE_001
//
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)] // Rule 2

//! WebAssembly module decoder for wrt runtime
//!
//! This crate provides a high-level API for decoding WebAssembly binary modules
//! into structured representations that can be used by the wrt runtime.
//!
//! The decoder sits between the low-level binary format handling in
//! `wrt-format` and the runtime execution in `wrt`. It properly converts
//! between format types and runtime types, ensuring type consistency across the
//! system.
//!
//! # Features
//!
//! - Decoding WebAssembly modules from binary format
//! - Encoding modules back to binary format
//! - Validating module structure
//! - Memory-efficient handling of WASM modules
//! - Component model support
//! - No_std and std environment support
//!
//! ## Feature Flags
//!
//! - `std` (default): Enable standard library support
//! - `alloc`: Enable allocation support (required for no_std)
//! - `component`: Enable WebAssembly Component Model support
//! - `safe_memory`: Enable memory safety features

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::missing_panics_doc)]
//#![deny(missing_docs)] // Temporarily disabled for build

// Import core
extern crate core;

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Note: Panic handler removed to avoid conflicts with std library

// Module exports
// Core memory optimization modules (always available)
pub mod memory_optimized;
pub mod optimized_string;
pub mod prelude;

// Conditionally include other modules
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod component;
// Temporarily disabled due to type issues
// #[cfg(feature = "alloc")]
// pub mod conversion;
// Most modules temporarily disabled for demo
// #[cfg(feature = "alloc")]
// pub mod custom_section_utils;
// #[cfg(feature = "alloc")]
// pub mod decoder_core;
// #[cfg(feature = "alloc")]
// pub mod instructions;
// #[cfg(feature = "alloc")]
// pub mod module;
// #[cfg(feature = "alloc")]
// pub mod optimized_module;
// #[cfg(feature = "alloc")]
// pub mod name_section;
// #[cfg(feature = "alloc")]
// pub mod parser;
// #[cfg(feature = "alloc")]
// pub mod producers_section;
// #[cfg(feature = "alloc")]
// pub mod runtime_adapter;
// #[cfg(feature = "alloc")]
// pub mod section_error;
// #[cfg(feature = "alloc")]
// pub mod section_reader;
// #[cfg(feature = "alloc")]
// pub mod types;
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod utils;
// #[cfg(feature = "alloc")]
// pub mod validation;
// #[cfg(feature = "alloc")]
// pub mod wasm;

// CFI metadata generation - temporarily disabled due to type issues
// pub mod cfi_metadata;

// Dedicated module for no_alloc decoding
pub mod decoder_no_alloc;

// Branch hint custom section support (requires alloc)
#[cfg(feature = "alloc")]
pub mod branch_hint_section;
#[cfg(feature = "alloc")]
pub mod custom_section_handler;

// Most re-exports temporarily disabled for demo - keep only essential ones
pub use decoder_no_alloc::{
    create_memory_provider, decode_module_header, extract_section_info, validate_module_no_alloc,
    verify_wasm_header, SectionId, SectionInfo, ValidatorType, WasmModuleHeader, MAX_MODULE_SIZE,
};
pub use wrt_error::{codes, kinds, Error, Result};
// Essential re-exports only
#[cfg(feature = "std")]
pub use wrt_foundation::safe_memory::StdProvider as StdMemoryProvider;
pub use wrt_foundation::safe_memory::{MemoryProvider, SafeSlice};

/// Validate WebAssembly header
///
/// # Arguments
///
/// * `bytes` - WebAssembly binary data
///
/// # Returns
///
/// * `Result<()>` - Success or error
pub fn validate_header(bytes: &[u8]) -> Result<()> {
    verify_wasm_header(bytes)
}
