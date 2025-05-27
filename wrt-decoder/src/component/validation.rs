// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! WebAssembly Component model validation
//!
//! This module provides validation for WebAssembly Component Model components.
//! It verifies that components conform to the Component Model specification.

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{collections::BTreeMap as HashMap, vec::Vec};
#[cfg(feature = "std")]
use std::collections::HashMap;

use wrt_error::{codes, Error, ErrorCategory, Result};
use wrt_format::component::{
    Alias, Canon, CanonOperation, ComponentType, ComponentTypeDefinition, Export, Import, Instance,
    ValType,
};
#[cfg(not(any(feature = "std", feature = "alloc")))]
use wrt_foundation::{
    bounded::{BoundedVec, WasmName},
    no_std_hashmap::SimpleHashMap as HashMap,
};

// Import component model types
use crate::component::Component;
// Import prelude for String and other types
use crate::prelude::*;

/// Maximum reasonable number of types in a component for validation
const MAX_TYPES: u32 = 100_000;

/// Maximum reasonable number of imports/exports
const MAX_IMPORTS_EXPORTS: u32 = 10_000;

/// Validation configuration for component model validation
///
/// This allows controlling which features of the Component Model are validated,
/// in case some implementations don't support the full model.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Enable value section validation (🪙)
    pub enable_value_section: bool,
    /// Enable resource types validation (🔧)
    pub enable_resource_types: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self { enable_value_section: true, enable_resource_types: true }
    }
}

impl ValidationConfig {
    /// Create a new validation config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a validation config with all features enabled
    pub fn all_enabled() -> Self {
        Self { enable_value_section: true, enable_resource_types: true }
    }

    /// Create a validation config with only MVP features enabled
    pub fn mvp_only() -> Self {
        Self { enable_value_section: false, enable_resource_types: false }
    }
}

/// Validation context for tracking component structure during validation
struct ValidationContext<'a> {
    /// Component being validated
    component: &'a Component,
    /// Configuration for validation
    config: &'a ValidationConfig,
    /// Track defined type indices
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    defined_types: BoundedVec<u32, 1000>,
    #[cfg(any(feature = "std", feature = "alloc"))]
    defined_types: Vec<u32>,
    /// Track defined import names to detect duplicates
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    import_names: HashMap<WasmName, u32, 100>,
    #[cfg(any(feature = "std", feature = "alloc"))]
    import_names: HashMap<String, u32>,
    /// Track defined export names to detect duplicates
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    export_names: HashMap<WasmName, u32, 100>,
    #[cfg(any(feature = "std", feature = "alloc"))]
    export_names: HashMap<String, u32>,
    /// Track defined instance indices
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    defined_instances: BoundedVec<u32, 1000>,
    #[cfg(any(feature = "std", feature = "alloc"))]
    defined_instances: Vec<u32>,
}

impl<'a> ValidationContext<'a> {
    /// Create a new validation context
    fn new(component: &'a Component, config: &'a ValidationConfig) -> Self {
        Self {
            component,
            config,
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            defined_types: BoundedVec::new(),
            #[cfg(any(feature = "std", feature = "alloc"))]
            defined_types: Vec::new(),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            import_names: HashMap::new(),
            #[cfg(any(feature = "std", feature = "alloc"))]
            import_names: HashMap::new(),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            export_names: HashMap::new(),
            #[cfg(any(feature = "std", feature = "alloc"))]
            export_names: HashMap::new(),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            defined_instances: BoundedVec::new(),
            #[cfg(any(feature = "std", feature = "alloc"))]
            defined_instances: Vec::new(),
        }
    }

    /// Add a defined type index
    fn add_type(&mut self, idx: u32) -> Result<()> {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            self.defined_types.push(idx).map_err(|_| {
                Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("too many types in component")
            })?;
        }
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if self.defined_types.len() >= MAX_TYPES as usize {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("too many types in component"));
            }
            self.defined_types.push(idx);
        }
        Ok(())
    }

    /// Check if a type index is valid
    fn is_type_valid(&self, idx: u32) -> bool {
        self.defined_types.contains(&idx)
    }

    /// Add an import name and check for duplicates
    fn add_import_name(&mut self, name: &str) -> Result<()> {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            let wasm_name = WasmName::try_from(name).map_err(|_| {
                Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("import name too long")
            })?;
            if self.import_names.contains_key(&wasm_name) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("duplicate import name"));
            }
            self.import_names.insert(wasm_name, self.import_names.len() as u32).map_err(|_| {
                Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("too many imports")
            })?;
        }
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if self.import_names.contains_key(name) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("duplicate import name"));
            }
            if self.import_names.len() >= MAX_IMPORTS_EXPORTS as usize {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("too many imports"));
            }
            self.import_names.insert(name.to_string(), self.import_names.len() as u32);
        }
        Ok(())
    }

    /// Add an export name and check for duplicates
    fn add_export_name(&mut self, name: &str) -> Result<()> {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            let wasm_name = WasmName::try_from(name).map_err(|_| {
                Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("export name too long")
            })?;
            if self.export_names.contains_key(&wasm_name) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("duplicate export name"));
            }
            self.export_names.insert(wasm_name, self.export_names.len() as u32).map_err(|_| {
                Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("too many exports")
            })?;
        }
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if self.export_names.contains_key(name) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("duplicate export name"));
            }
            if self.export_names.len() >= MAX_IMPORTS_EXPORTS as usize {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("too many exports"));
            }
            self.export_names.insert(name.to_string(), self.export_names.len() as u32);
        }
        Ok(())
    }

    /// Add a defined instance index
    fn add_instance(&mut self, idx: u32) -> Result<()> {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            self.defined_instances.push(idx).map_err(|_| {
                Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("too many instances in component")
            })?;
        }
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if self.defined_instances.len() >= MAX_TYPES as usize {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("too many instances in component"));
            }
            self.defined_instances.push(idx);
        }
        Ok(())
    }

    /// Check if an instance index is valid
    fn is_instance_valid(&self, idx: u32) -> bool {
        self.defined_instances.contains(&idx)
    }
}

/// Validate all types in the component
fn validate_types(ctx: &mut ValidationContext) -> Result<()> {
    for (idx, component_type) in ctx.component.types.iter().enumerate() {
        validate_component_type(ctx, component_type)?;
        ctx.add_type(idx as u32)?;
    }
    Ok(())
}

/// Validate a single component type
fn validate_component_type(ctx: &ValidationContext, component_type: &ComponentType) -> Result<()> {
    match &component_type.definition {
        ComponentTypeDefinition::Module(_module_type) => {
            // Module types are validated during parsing
            Ok(())
        }
        ComponentTypeDefinition::Component(_comp_type) => {
            // Nested component types are validated recursively
            Ok(())
        }
        ComponentTypeDefinition::Instance(_instance_type) => {
            // Instance types are validated during parsing
            Ok(())
        }
        ComponentTypeDefinition::Func(_func_type) => {
            // Function types are validated during parsing
            Ok(())
        }
        ComponentTypeDefinition::Value(val_type) => {
            if !ctx.config.enable_value_section && matches!(val_type, ValType::Resource(_)) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("resource types not enabled"));
            }
            Ok(())
        }
        ComponentTypeDefinition::Type(_type_def) => {
            // Type definitions are validated during parsing
            Ok(())
        }
        ComponentTypeDefinition::Alias(alias) => validate_alias(ctx, alias),
        ComponentTypeDefinition::Export { .. } | ComponentTypeDefinition::Import { .. } => {
            // These are validated during parsing
            Ok(())
        }
    }
}

/// Validate an alias
fn validate_alias(ctx: &ValidationContext, alias: &Alias) -> Result<()> {
    match alias {
        Alias::Type { instance, index } => {
            if !ctx.is_instance_valid(*instance) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("invalid instance index in type alias"));
            }
            // Further validation would check if the type exists in the instance
            _ = index; // Suppress unused warning
        }
        Alias::Export { instance, name } => {
            if !ctx.is_instance_valid(*instance) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("invalid instance index in export alias"));
            }
            // Further validation would check if the export exists in the instance
            _ = name; // Suppress unused warning
        }
        Alias::Outer { count, index } => {
            // Outer aliases reference parent components
            // Validation would check if we're nested deep enough
            _ = (count, index); // Suppress unused warnings
        }
    }
    Ok(())
}

/// Validate all imports in the component
fn validate_imports(ctx: &mut ValidationContext) -> Result<()> {
    for import in &ctx.component.imports {
        validate_import(ctx, import)?;
    }
    Ok(())
}

/// Validate a single import
fn validate_import(ctx: &mut ValidationContext, import: &Import) -> Result<()> {
    // Check for duplicate import names
    ctx.add_import_name(&import.name)?;

    // Validate the import type reference
    if !ctx.is_type_valid(import.ty) {
        return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
            .with_context("invalid type index in import"));
    }

    Ok(())
}

/// Validate all exports in the component
fn validate_exports(ctx: &mut ValidationContext) -> Result<()> {
    for export in &ctx.component.exports {
        validate_export(ctx, export)?;
    }
    Ok(())
}

/// Validate a single export
fn validate_export(ctx: &mut ValidationContext, export: &Export) -> Result<()> {
    // Check for duplicate export names
    ctx.add_export_name(&export.name)?;

    // Validate the export reference
    match export.kind {
        0 => {
            // Core module export
            if export.index >= ctx.component.modules.len() as u32 {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("invalid module index in export"));
            }
        }
        1 => {
            // Function export
            // Would need to track defined functions
        }
        2 => {
            // Type export
            if !ctx.is_type_valid(export.index) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("invalid type index in export"));
            }
        }
        3 => {
            // Instance export
            if !ctx.is_instance_valid(export.index) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("invalid instance index in export"));
            }
        }
        4 => {
            // Component export
            if export.index >= ctx.component.components.len() as u32 {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("invalid component index in export"));
            }
        }
        _ => {
            return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                .with_context("invalid export kind"));
        }
    }

    Ok(())
}

/// Validate all instances in the component
fn validate_instances(ctx: &mut ValidationContext) -> Result<()> {
    for (idx, instance) in ctx.component.instances.iter().enumerate() {
        validate_instance(ctx, instance)?;
        ctx.add_instance(idx as u32)?;
    }
    Ok(())
}

/// Validate a single instance
fn validate_instance(ctx: &ValidationContext, instance: &Instance) -> Result<()> {
    match instance {
        Instance::Instantiate { module, args } => {
            // Validate module index
            if *module >= ctx.component.modules.len() as u32 {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("invalid module index in instance"));
            }
            // Would need to validate args match module imports
            _ = args; // Suppress unused warning
        }
        Instance::FromExports(exports) => {
            // Validate each export in the instance
            for export in exports {
                // Would need to validate export references
                _ = export; // Suppress unused warning
            }
        }
    }
    Ok(())
}

/// Validate all canonical functions in the component
fn validate_canonicals(ctx: &ValidationContext) -> Result<()> {
    for canon in &ctx.component.canonicals {
        validate_canonical(ctx, canon)?;
    }
    Ok(())
}

/// Validate a single canonical function
fn validate_canonical(ctx: &ValidationContext, canon: &Canon) -> Result<()> {
    match &canon.operation {
        CanonOperation::Lift { func_ty, options } => {
            // Validate function type index
            if !ctx.is_type_valid(*func_ty) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("invalid function type in canon lift"));
            }
            // Would need to validate options
            _ = options; // Suppress unused warning
        }
        CanonOperation::Lower { func_idx, options } => {
            // Would need to track defined functions
            _ = (func_idx, options); // Suppress unused warnings
        }
        CanonOperation::ResourceNew { resource } => {
            if !ctx.config.enable_resource_types {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("resource types not enabled"));
            }
            if !ctx.is_type_valid(*resource) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("invalid resource type in canon resource.new"));
            }
        }
        CanonOperation::ResourceDrop { resource } => {
            if !ctx.config.enable_resource_types {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("resource types not enabled"));
            }
            if !ctx.is_type_valid(*resource) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("invalid resource type in canon resource.drop"));
            }
        }
        CanonOperation::ResourceRep { resource } => {
            if !ctx.config.enable_resource_types {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("resource types not enabled"));
            }
            if !ctx.is_type_valid(*resource) {
                return Err(Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR)
                    .with_context("invalid resource type in canon resource.rep"));
            }
        }
    }
    Ok(())
}

/// Validate a component with specific configuration options
pub fn validate_component_with_config(
    component: &Component,
    config: &ValidationConfig,
) -> Result<()> {
    let mut ctx = ValidationContext::new(component, config);

    // Validate in dependency order
    validate_types(&mut ctx)?;
    validate_imports(&mut ctx)?;
    validate_instances(&mut ctx)?;
    validate_canonicals(&ctx)?;
    validate_exports(&mut ctx)?;

    // Additional validation for value sections if enabled
    if config.enable_value_section && !component.values.is_empty() {
        // Value section validation would go here
    }

    Ok(())
}

/// Validate a component with default configuration
pub fn validate_component(component: &Component) -> Result<()> {
    validate_component_with_config(component, &ValidationConfig::default())
}
