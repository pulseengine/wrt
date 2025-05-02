//! WebAssembly Component Model decoding.
//!
//! This module provides functions for decoding WebAssembly Component Model
//! components from binary format.

pub mod analysis;
mod decode;
mod encode;
pub mod name_section;
mod parse;
pub mod types;
pub mod utils;
pub mod val_type;
pub mod validation;

pub use analysis::{
    analyze_component, analyze_component_extended, extract_embedded_modules, extract_inline_module,
    extract_module_info, is_valid_module, AliasInfo, ComponentSummary, CoreInstanceInfo,
    CoreModuleInfo, ExtendedExportInfo, ExtendedImportInfo, ModuleExportInfo, ModuleImportInfo,
};
pub use decode::decode_component;
pub use encode::encode_component;
pub use name_section::{
    generate_component_name_section, parse_component_name_section, ComponentNameSection, NameMap,
    NameMapEntry, SortIdentifier,
};
pub use types::*;
pub use utils::*;
pub use val_type::encode_val_type;
pub use validation::{validate_component, validate_component_with_config, ValidationConfig};
