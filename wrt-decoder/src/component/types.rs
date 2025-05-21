// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// Re-export the main component types from wrt-format for convenience
pub use wrt_format::component::{
    Component, ComponentType, CoreExternType, CoreInstance, CoreType, Export, ExternType, Import,
    Instance, Start, ValType,
};

use crate::prelude::*;

/// Trait for component analysis capabilities
pub trait ComponentAnalyzer {
    /// Create a summary of a component's structure
    fn analyze(&self) -> crate::component::analysis::ComponentSummary;

    /// Get embedded modules from a component
    fn get_embedded_modules(&self) -> Vec<Vec<u8>>;

    /// Check if a component has a specific export
    fn has_export(&self, name: &str) -> bool;

    /// Get information about exports
    fn get_export_info(&self) -> Vec<ExportInfo>;

    /// Get information about imports
    fn get_import_info(&self) -> Vec<ImportInfo>;
}

/// Export information
#[derive(Debug, Clone)]
pub struct ExportInfo {
    /// Export name
    pub name: String,
    /// Type of export (function, memory, etc.)
    pub kind: String,
    /// Type information (as string)
    pub type_info: String,
}

/// Import information
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// Import module
    pub module: String,
    /// Import name
    pub name: String,
    /// Type of import (function, memory, etc.)
    pub kind: String,
    /// Type information (as string)
    pub type_info: String,
}

/// Component binary metadata
#[derive(Debug, Clone)]
pub struct ComponentMetadata {
    /// Component name or identifier
    pub name: String,
    /// Component version (if available)
    pub version: Option<String>,
    /// Custom sections contained in the component
    pub custom_sections: Vec<String>,
}

/// Module information within a component
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Module index
    pub idx: u32,
    /// Module size in bytes
    pub size: usize,
    /// Module function count
    pub function_count: usize,
    /// Module memory count
    pub memory_count: usize,
    /// Module table count
    pub table_count: usize,
    /// Module global count
    pub global_count: usize,
}

/// Implementation of ComponentAnalyzer for Component
impl ComponentAnalyzer for Component {
    fn analyze(&self) -> crate::component::analysis::ComponentSummary {
        // Create a basic summary directly from the component
        crate::component::analysis::ComponentSummary {
            name: "".to_string(),
            core_modules_count: self.modules.len() as u32,
            core_instances_count: self.core_instances.len() as u32,
            imports_count: self.imports.len() as u32,
            exports_count: self.exports.len() as u32,
            aliases_count: self.aliases.len() as u32,
            module_info: Vec::new(),
            export_info: Vec::new(),
            import_info: Vec::new(),
        }
    }

    fn get_embedded_modules(&self) -> Vec<Vec<u8>> {
        // This will be implemented in the analysis module
        Vec::new()
    }

    fn has_export(&self, name: &str) -> bool {
        self.exports.iter().any(|export| export.name.name == name)
    }

    fn get_export_info(&self) -> Vec<ExportInfo> {
        // This will be implemented in the analysis module
        Vec::new()
    }

    fn get_import_info(&self) -> Vec<ImportInfo> {
        // This will be implemented in the analysis module
        Vec::new()
    }
}
