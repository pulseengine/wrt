//! WebAssembly Core Module Decoder
//!
//! Functions for decoding WebAssembly core modules from binary format.

use crate::module::Module;
use crate::parser::Parser;
use wrt_error::Result;

/// Initialize a default parser config
pub fn default_parser_config() -> crate::decoder_core::config::ParserConfig {
    crate::decoder_core::config::ParserConfig::default()
}

/// Initialize a default validation config
pub fn default_validation_config() -> crate::decoder_core::config::ValidationConfig {
    crate::decoder_core::config::ValidationConfig::default()
}

/// Decode a WebAssembly module from binary data
///
/// This is the main entry point for decoding modules from binary data.
/// It handles both the parsing and validation of the module.
///
/// # Arguments
///
/// * `binary` - Binary WebAssembly module data
///
/// # Returns
///
/// * `Result<Module>` - Decoded module or error
pub fn decode_module(binary: &[u8]) -> Result<Module> {
    // Create a parser to process the binary data
    let _parser = Parser::new(Some(binary), false);

    // Parse the module from the binary data
    let module = crate::parser::parse_module(binary)?;

    // Validate the module
    crate::validation::validate_module(&module)?;

    // Return the validated module
    Ok(module)
}

/// Decode a WebAssembly module from binary data without validation
///
/// This function decodes a module without validating it, which can be useful
/// for certain use cases where validation is not required or will be done later.
///
/// # Arguments
///
/// * `binary` - Binary WebAssembly module data
///
/// # Returns
///
/// * `Result<Module>` - Decoded module or error
pub fn decode_module_without_validation(binary: &[u8]) -> Result<Module> {
    // Parse the module from the binary data
    crate::parser::parse_module(binary)
}

/// Decode a WebAssembly component from binary data
///
/// This function decodes a WebAssembly component from binary data.
///
/// # Arguments
///
/// * `binary` - Binary WebAssembly component data
///
/// # Returns
///
/// * `Result<Component>` - Decoded component or error
#[cfg(feature = "component-model-core")]
pub fn decode_component(binary: &[u8]) -> Result<wrt_format::component::Component> {
    // Create a parser
    let _parser = Parser::new(Some(binary), false);

    // Parse the component
    let component = crate::component::decode::decode_component(binary)?;

    // Validate the component
    #[cfg(feature = "component-model-values")]
    crate::component::validation::validate_component(&component)?;

    // Return the component
    Ok(component)
}

// No duplicated re-exports needed
