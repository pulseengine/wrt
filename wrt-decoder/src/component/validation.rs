//! WebAssembly Component Model validation.
//!
//! This module provides validation functions for WebAssembly Component Model
//! components, ensuring they follow the Component Model specification.

use wrt_error::{codes, Error, ErrorCategory, Result};
use wrt_format::component::Component;

#[cfg(not(feature = "std"))]
use alloc::collections::HashMap;

// Use our prelude instead of conditional imports
use crate::prelude::*;

/// Configuration for component validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Enable value section validation (🪙)
    pub enable_value_section: bool,
    /// Enable resource types validation (🔧)
    pub enable_resource_types: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enable_value_section: true,
            enable_resource_types: true,
        }
    }
}

impl ValidationConfig {
    /// Create a new validation configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a validation configuration with all features enabled
    pub fn all_enabled() -> Self {
        Self {
            enable_value_section: true,
            enable_resource_types: true,
        }
    }

    /// Create a validation configuration with only MVP features enabled
    pub fn mvp_only() -> Self {
        Self {
            enable_value_section: false,
            enable_resource_types: false,
        }
    }
}

/// Validate a WebAssembly Component with custom validation configuration
pub fn validate_component_with_config(
    component: &Component,
    config: &ValidationConfig,
) -> Result<()> {
    // For now, just return Ok as a placeholder implementation
    // The full implementation would validate the component against the config
    Ok(())
}

/// Validate a WebAssembly Component using default configuration
pub fn validate_component(component: &Component) -> Result<()> {
    validate_component_with_config(component, &ValidationConfig::default())
}

/// Create a validation error with the given message
fn validation_error(message: String) -> Error {
    Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR, message)
}
