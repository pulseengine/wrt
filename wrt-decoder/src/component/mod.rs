//! WebAssembly Component Model decoding.
//!
//! This module provides functions for decoding WebAssembly Component Model
//! components from binary format.

pub mod analysis;
mod decode;
mod encode;
mod parse;
pub mod types;
pub mod utils;

pub use analysis::{
    analyze_component, analyze_component_extended, extract_embedded_modules, extract_inline_module,
    extract_module_info, is_valid_module, AliasInfo, ComponentSummary, CoreInstanceInfo,
    CoreModuleInfo, ExtendedExportInfo, ExtendedImportInfo, ModuleExportInfo, ModuleImportInfo,
};
pub use decode::decode_component;
pub use encode::encode_component;
pub use types::*;
pub use utils::*;
