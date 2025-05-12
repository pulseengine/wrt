//! WebAssembly module validation
//!
//! This module provides functionality for validating WebAssembly modules
//! against the WebAssembly specification.

use wrt_error::Result;

use crate::{
    decoder_core::validate::{validate_module as core_validate_module, ValidationConfig},
    module::Module,
};

/// Validate a WebAssembly module
///
/// This function checks if a WebAssembly module is valid according to the
/// WebAssembly specification. It delegates to the core validation
/// implementation.
///
/// # Arguments
///
/// * `module` - The module to validate
///
/// # Returns
///
/// * `Result<()>` - Ok if the module is valid, Err otherwise
pub fn validate_module(module: &Module) -> Result<()> {
    core_validate_module(module)
}

/// Validate a WebAssembly module with specific validation configuration
///
/// This function checks if a WebAssembly module is valid according to the
/// WebAssembly specification, using the provided validation configuration.
///
/// # Arguments
///
/// * `module` - The module to validate
/// * `config` - Validation configuration options
///
/// # Returns
///
/// * `Result<()>` - Ok if the module is valid, Err otherwise
pub fn validate_module_with_config(module: &Module, config: ValidationConfig) -> Result<()> {
    crate::decoder_core::validate::validate_module_with_config(module, &config)
}

/// Validate a WebAssembly module from binary data
///
/// This function parses and validates a WebAssembly module from binary data.
///
/// # Arguments
///
/// * `binary` - The binary WebAssembly module data
///
/// # Returns
///
/// * `Result<()>` - Ok if the module is valid, Err otherwise
pub fn validate_binary(binary: &[u8]) -> Result<()> {
    let module = crate::module::decode_module_with_binary(binary)?;
    validate_module(&module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::decode_module_with_binary;

    #[test]
    fn test_empty_module_validation() {
        // The simplest valid module is just the header
        let binary = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        let module = decode_module_with_binary(&binary).unwrap();
        assert!(validate_module(&module).is_ok());
    }

    #[test]
    fn test_invalid_module_validation() {
        // Invalid module header
        let binary = vec![0x00, 0x61, 0x73, 0x6D, 0x02, 0x00, 0x00, 0x00]; // Wrong version
        let module_result = decode_module_with_binary(&binary);
        assert!(module_result.is_err());
    }
}
