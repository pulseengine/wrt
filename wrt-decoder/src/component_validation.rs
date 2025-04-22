//! WebAssembly Component Model validation.
//!
//! This module provides validation functions for WebAssembly Component Model
//! components, ensuring they follow the Component Model specification.

use wrt_error::{kinds, Error, Result};
use wrt_format::component::{
    Component, ComponentTypeDefinition, CoreInstance, CoreInstanceExpr, CoreSort, CoreType,
    CoreTypeDefinition, Export, ExternType, Import, Instance, InstanceExpr, ResourceRepresentation,
    Sort, ValType,
};

// Use our prelude instead of conditional imports
use crate::prelude::*;
use std::collections::HashMap;

/// Check if a string is a valid semantic version (major.minor.patch)
fn is_valid_semver(version: &str) -> bool {
    // Basic semver validation (major.minor.patch)
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    // Check that all parts are valid numbers
    for part in parts {
        if part.parse::<u32>().is_err() {
            return false;
        }
    }

    true
}

/// Check if a string is a valid integrity hash
fn is_valid_integrity(integrity: &str) -> bool {
    // Basic integrity validation
    // Format should be algo-value, e.g., "sha384-AB123..."
    if let Some(dash_pos) = integrity.find('-') {
        let algo = &integrity[0..dash_pos];
        let value = &integrity[dash_pos + 1..];

        // Check that algorithm is one of the valid ones
        match algo {
            "sha256" | "sha384" | "sha512" => {
                // Check that value is a valid hex string
                // This is a very basic check, real implementation would be more thorough
                !value.is_empty() && value.chars().all(|c| c.is_ascii_hexdigit())
            }
            _ => false,
        }
    } else {
        false
    }
}

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
    // Validation context containing indices for different sections
    let mut ctx = ValidationContext::new();

    // Validate core modules
    for (idx, _module) in component.modules.iter().enumerate() {
        ctx.add_module(idx as u32);
        // In a full implementation, we would validate each core module
        // using the standard WebAssembly validation
    }

    // Validate core types
    for (idx, core_type) in component.core_types.iter().enumerate() {
        ctx.add_core_type(idx as u32);
        validate_core_type(core_type, &ctx)?;
    }

    // Validate core instances
    for (idx, core_instance) in component.core_instances.iter().enumerate() {
        ctx.add_core_instance(idx as u32);
        validate_core_instance(core_instance, &ctx)?;
    }

    // Validate component types - including resource types
    for (idx, comp_type) in component.types.iter().enumerate() {
        ctx.add_component_type(idx as u32);

        // Check resource types are enabled
        if !config.enable_resource_types {
            if let ComponentTypeDefinition::Resource { .. } = &comp_type.definition {
                return Err(Error::new(kinds::ValidationError(
                    "Resource types are not enabled in the current configuration".to_string(),
                )));
            }
        }

        validate_component_type(comp_type, &mut ctx)?;
    }

    // Validate nested components
    for (idx, nested_component) in component.components.iter().enumerate() {
        ctx.add_component(idx as u32);
        validate_component_with_config(nested_component, config)?;
    }

    // Validate component instances
    for (idx, instance) in component.instances.iter().enumerate() {
        ctx.add_instance(idx as u32);
        validate_instance(instance, &ctx)?;
    }

    // Validate imports
    for import in &component.imports {
        validate_import(import, &ctx)?;
    }

    // Track values from imports
    for (idx, import) in component.imports.iter().enumerate() {
        if let wrt_format::component::ExternType::Value(_) = &import.ty {
            ctx.add_value(idx as u32);
            ctx.mark_value_unconsumed(idx as u32);
        }
    }

    // Track values from instances
    let import_count = component.imports.len() as u32;
    for (idx, instance) in component.instances.iter().enumerate() {
        if let wrt_format::component::InstanceExpr::Instantiate {
            component_idx: _,
            args: _,
        } = &instance.instance_expr
        {
            // In a full implementation, we would determine the number of values exported by the instance
            // For now, we assume each instance adds one value to the value index space
            let value_idx = import_count + idx as u32;
            ctx.add_value(value_idx);
            ctx.mark_value_unconsumed(value_idx);
        }
    }

    // Validate canonical operations
    for canon in &component.canonicals {
        validate_canon(canon, &ctx)?;
    }

    // Validate value definitions if enabled
    if !component.values.is_empty() && !config.enable_value_section {
        return Err(Error::new(kinds::ValidationError(
            "Value section is not enabled in the current configuration".to_string(),
        )));
    }

    validate_values(component, &mut ctx)?;

    // Validate exports (and mark consumed values)
    for export in &component.exports {
        validate_export(export, &ctx)?;

        if export.sort == Sort::Value {
            ctx.mark_value_consumed(export.idx);
        }
    }

    // Validate start function if present
    if let Some(start) = &component.start {
        validate_start(component, &mut ctx)?;

        // Mark argument values as consumed
        for arg_idx in &start.args {
            ctx.mark_value_consumed(*arg_idx);
        }

        // Add result values to the value index space
        for result_idx in 0..start.results {
            let value_idx = ctx.values.len() as u32;
            ctx.add_value(value_idx);
            ctx.mark_value_unconsumed(value_idx);
        }
    }

    // Validate resources if enabled
    if config.enable_resource_types {
        validate_resources(component, &mut ctx)?;
    }

    // Check that all values have been consumed
    ctx.validate_all_values_consumed()?;

    Ok(())
}

// Update the original validate_component function to use the default config
pub fn validate_component(component: &Component) -> Result<()> {
    validate_component_with_config(component, &ValidationConfig::default())
}

/// Validation context for tracking indices and dependencies.
struct ValidationContext {
    /// Core module indices
    modules: Vec<u32>,
    /// Core type indices
    core_types: Vec<u32>,
    /// Core instance indices
    core_instances: Vec<u32>,
    /// Component type indices
    component_types: Vec<u32>,
    /// Component indices
    components: Vec<u32>,
    /// Instance indices
    instances: Vec<u32>,
    /// Function indices
    funcs: Vec<u32>,
    /// Value indices
    values: Vec<u32>,
    /// Value consumption tracking
    value_consumed: Vec<bool>,
    /// Resource types (for validating resource operations)
    resource_types: Vec<u32>,
    /// Resource ownership tracking - maps resource handle indices to owner state
    #[allow(dead_code)]
    resource_ownership: HashMap<u32, ResourceOwnerState>,
    /// Resource borrowing tracking - maps resource handle indices to borrow state
    #[allow(dead_code)]
    resource_borrowing: HashMap<u32, ResourceBorrowState>,
}

/// Tracks the ownership state of a resource
#[allow(dead_code)]
enum ResourceOwnerState {
    /// Resource is owned
    Owned,
    /// Resource ownership has been transferred
    Transferred,
    /// Resource has been destroyed
    Destroyed,
}

/// Tracks the borrowing state of a resource
struct ResourceBorrowState {
    /// Number of active borrows
    #[allow(dead_code)]
    borrow_count: u32,
    /// Scope where the borrows must be resolved (function instance index)
    #[allow(dead_code)]
    borrow_scope: u32,
}

impl ValidationContext {
    /// Create a new empty validation context.
    fn new() -> Self {
        Self {
            modules: Vec::new(),
            core_types: Vec::new(),
            core_instances: Vec::new(),
            component_types: Vec::new(),
            components: Vec::new(),
            instances: Vec::new(),
            funcs: Vec::new(),
            values: Vec::new(),
            value_consumed: Vec::new(),
            resource_types: Vec::new(),
            resource_ownership: HashMap::new(),
            resource_borrowing: HashMap::new(),
        }
    }

    /// Add a core module index to the context.
    fn add_module(&mut self, idx: u32) {
        self.modules.push(idx);
    }

    /// Add a core type index to the context.
    fn add_core_type(&mut self, idx: u32) {
        self.core_types.push(idx);
    }

    /// Add a core instance index to the context.
    fn add_core_instance(&mut self, idx: u32) {
        self.core_instances.push(idx);
    }

    /// Add a component type index to the context.
    fn add_component_type(&mut self, idx: u32) {
        self.component_types.push(idx);
    }

    /// Add a component index to the context.
    fn add_component(&mut self, idx: u32) {
        self.components.push(idx);
    }

    /// Add an instance index to the context.
    fn add_instance(&mut self, idx: u32) {
        self.instances.push(idx);
    }

    /// Add a function index to the context.
    #[allow(dead_code)]
    fn add_func(&mut self, idx: u32) {
        self.funcs.push(idx);
    }

    /// Add a value index to the context.
    fn add_value(&mut self, idx: u32) {
        self.values.push(idx);
        // Ensure the value_consumed vector is large enough
        if self.value_consumed.len() <= idx as usize {
            self.value_consumed.resize(idx as usize + 1, false);
        }
    }

    /// Mark a value as consumed
    fn mark_value_consumed(&mut self, idx: u32) {
        if idx as usize >= self.value_consumed.len() {
            return; // Ignore invalid indices, validation will fail elsewhere
        }
        self.value_consumed[idx as usize] = true;
    }

    /// Mark a value as unconsumed (initial state)
    fn mark_value_unconsumed(&mut self, idx: u32) {
        if idx as usize >= self.value_consumed.len() {
            return; // Ignore invalid indices, validation will fail elsewhere
        }
        self.value_consumed[idx as usize] = false;
    }

    /// Check if a value has been consumed
    fn is_value_consumed(&self, idx: u32) -> bool {
        if idx as usize >= self.value_consumed.len() {
            return false; // Ignore invalid indices, validation will fail elsewhere
        }
        self.value_consumed[idx as usize]
    }

    /// Validate that all values have been consumed exactly once
    fn validate_all_values_consumed(&self) -> Result<()> {
        let mut unconsumed = Vec::new();

        for (idx, consumed) in self.value_consumed.iter().enumerate() {
            if !consumed {
                unconsumed.push(idx as u32);
            }
        }

        if !unconsumed.is_empty() {
            return Err(Error::new(kinds::ValidationError(format!(
                "Values not consumed: {:?}",
                unconsumed
            ))));
        }

        Ok(())
    }

    /// Check if a module index is valid.
    fn is_valid_module(&self, idx: u32) -> bool {
        self.modules.contains(&idx)
    }

    /// Check if a core type index is valid.
    fn is_valid_core_type(&self, idx: u32) -> bool {
        self.core_types.contains(&idx)
    }

    /// Check if a core instance index is valid.
    fn is_valid_core_instance(&self, idx: u32) -> bool {
        self.core_instances.contains(&idx)
    }

    /// Check if a component type index is valid.
    fn is_valid_component_type(&self, idx: u32) -> bool {
        self.component_types.contains(&idx)
    }

    /// Check if a component index is valid.
    fn is_valid_component(&self, idx: u32) -> bool {
        self.components.contains(&idx)
    }

    /// Check if an instance index is valid.
    fn is_valid_instance(&self, idx: u32) -> bool {
        self.instances.contains(&idx)
    }

    /// Check if a function index is valid.
    fn is_valid_func(&self, idx: u32) -> bool {
        self.funcs.contains(&idx)
    }

    /// Check if a value index is valid.
    fn is_valid_value(&self, idx: u32) -> bool {
        self.values.contains(&idx)
    }

    /// Add a resource type index to the context
    #[allow(dead_code)]
    fn add_resource_type(&mut self, idx: u32) {
        self.resource_types.push(idx);
    }

    /// Check if a resource type index is valid
    fn is_valid_resource_type(&self, idx: u32) -> bool {
        self.resource_types.contains(&idx)
    }

    /// Track a resource being created
    #[allow(dead_code)]
    fn track_resource_created(&mut self, resource_idx: u32) {
        self.resource_ownership
            .insert(resource_idx, ResourceOwnerState::Owned);
    }

    /// Track a resource ownership transfer
    fn track_resource_transferred(&mut self, resource_idx: u32) -> Result<()> {
        match self.resource_ownership.get_mut(&resource_idx) {
            Some(owner_state) => match owner_state {
                ResourceOwnerState::Owned => {
                    *owner_state = ResourceOwnerState::Transferred;
                    Ok(())
                }
                ResourceOwnerState::Transferred => Err(Error::new(kinds::ValidationError(
                    format!("Resource {} has already been transferred", resource_idx),
                ))),
                ResourceOwnerState::Destroyed => Err(Error::new(kinds::ValidationError(format!(
                    "Resource {} has been destroyed and cannot be transferred",
                    resource_idx
                )))),
            },
            None => Err(Error::new(kinds::ValidationError(format!(
                "Resource {} does not exist",
                resource_idx
            )))),
        }
    }

    /// Track a resource being destroyed
    fn track_resource_destroyed(&mut self, resource_idx: u32) -> Result<()> {
        match self.resource_ownership.get_mut(&resource_idx) {
            Some(owner_state) => match owner_state {
                ResourceOwnerState::Owned => {
                    *owner_state = ResourceOwnerState::Destroyed;
                    Ok(())
                }
                ResourceOwnerState::Transferred => {
                    Err(Error::new(kinds::ValidationError(format!(
                        "Resource {} has been transferred and cannot be destroyed",
                        resource_idx
                    ))))
                }
                ResourceOwnerState::Destroyed => Err(Error::new(kinds::ValidationError(format!(
                    "Resource {} has already been destroyed",
                    resource_idx
                )))),
            },
            None => Err(Error::new(kinds::ValidationError(format!(
                "Resource {} does not exist",
                resource_idx
            )))),
        }
    }

    /// Track a resource being borrowed
    fn track_resource_borrowed(&mut self, resource_idx: u32, scope: u32) -> Result<()> {
        match self.resource_ownership.get(&resource_idx) {
            Some(ResourceOwnerState::Owned) => {
                // Increment borrow count for this resource
                let borrow_state =
                    self.resource_borrowing
                        .entry(resource_idx)
                        .or_insert(ResourceBorrowState {
                            borrow_count: 0,
                            borrow_scope: scope,
                        });

                if borrow_state.borrow_scope != scope {
                    // Resources can only be borrowed in one scope at a time
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Resource {} is already borrowed in scope {} but attempted to borrow in scope {}",
                        resource_idx, borrow_state.borrow_scope, scope
                    ))));
                }

                borrow_state.borrow_count += 1;
                Ok(())
            }
            Some(ResourceOwnerState::Transferred) => {
                Err(Error::new(kinds::ValidationError(format!(
                    "Resource {} has been transferred and cannot be borrowed",
                    resource_idx
                ))))
            }
            Some(ResourceOwnerState::Destroyed) => {
                Err(Error::new(kinds::ValidationError(format!(
                    "Resource {} has been destroyed and cannot be borrowed",
                    resource_idx
                ))))
            }
            None => Err(Error::new(kinds::ValidationError(format!(
                "Resource {} does not exist",
                resource_idx
            )))),
        }
    }

    /// Track a resource borrow being released
    fn track_resource_borrow_released(&mut self, resource_idx: u32) -> Result<()> {
        match self.resource_borrowing.get_mut(&resource_idx) {
            Some(borrow_state) => {
                if borrow_state.borrow_count > 0 {
                    borrow_state.borrow_count -= 1;
                    Ok(())
                } else {
                    Err(Error::new(kinds::ValidationError(format!(
                        "Resource {} has no active borrows to release",
                        resource_idx
                    ))))
                }
            }
            None => Err(Error::new(kinds::ValidationError(format!(
                "Resource {} has no borrow state",
                resource_idx
            )))),
        }
    }

    /// Validate that all resource borrows are released in a scope
    fn validate_all_borrows_released(&self, scope: u32) -> Result<()> {
        for (resource_idx, borrow_state) in &self.resource_borrowing {
            if borrow_state.borrow_scope == scope && borrow_state.borrow_count > 0 {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Resource {} has {} unreleased borrows at the end of scope {}",
                    resource_idx, borrow_state.borrow_count, scope
                ))));
            }
        }
        Ok(())
    }
}

/// Check if two types are compatible for import/export matching
fn is_compatible_type(imported_type: &ExternType, exported_type: &ExternType) -> bool {
    use wrt_format::component::ExternType;

    match (imported_type, exported_type) {
        // Functions are compatible if their types exactly match
        (
            ExternType::Function {
                params: params1,
                results: results1,
            },
            ExternType::Function {
                params: params2,
                results: results2,
            },
        ) => {
            // Check parameter counts and types
            if params1.len() != params2.len() || results1.len() != results2.len() {
                return false;
            }

            // Check parameter types (names can be different)
            for (i, (_, t1)) in params1.iter().enumerate() {
                if t1 != &params2[i].1 {
                    return false;
                }
            }

            // Check result types
            for (i, t1) in results1.iter().enumerate() {
                if t1 != &results2[i] {
                    return false;
                }
            }

            true
        }

        // Values are compatible if their types exactly match
        (ExternType::Value(v1), ExternType::Value(v2)) => v1 == v2,

        // Types are compatible if they refer to the same index
        (ExternType::Type(idx1), ExternType::Type(idx2)) => idx1 == idx2,

        // Instances are compatible if their exports are compatible
        (
            ExternType::Instance { exports: exports1 },
            ExternType::Instance { exports: exports2 },
        ) => {
            // For imports, the importing instance must have all exports that it needs
            // but the exporting instance can have more
            if exports1.len() > exports2.len() {
                return false;
            }

            // Check that all exports in the import are present in the export
            for (name1, ty1) in exports1 {
                if let Some((_, ty2)) = exports2.iter().find(|(name2, _)| name1 == name2) {
                    if !is_compatible_type(ty1, ty2) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        }

        // Components are compatible if imports and exports match
        (
            ExternType::Component {
                imports: imports1,
                exports: exports1,
            },
            ExternType::Component {
                imports: imports2,
                exports: exports2,
            },
        ) => {
            // The importing component must have all the exports that it offers
            if exports1.len() < exports2.len() {
                return false;
            }

            // The exporting component must satisfy all the imports needed
            if imports1.len() > imports2.len() {
                return false;
            }

            // Check that imports match
            for (ns1, name1, ty1) in imports1 {
                if let Some((_, _, ty2)) = imports2
                    .iter()
                    .find(|(ns2, name2, _)| ns1 == ns2 && name1 == name2)
                {
                    if !is_compatible_type(ty1, ty2) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Check that exports match
            for (name2, ty2) in exports2 {
                if let Some((_, ty1)) = exports1.iter().find(|(name1, _)| name2 == name1) {
                    if !is_compatible_type(ty1, ty2) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        }

        // Any other combination is not compatible
        _ => false,
    }
}

/// Validate the start function
fn validate_start(component: &Component, ctx: &mut ValidationContext) -> Result<()> {
    if let Some(start) = &component.start {
        // Check function index is valid
        if !ctx.is_valid_func(start.func_idx) {
            return Err(Error::new(kinds::ValidationError(format!(
                "Invalid function index {} in start function",
                start.func_idx
            ))));
        }

        // Check all argument indices are valid
        for (idx, arg_idx) in start.args.iter().enumerate() {
            if !ctx.is_valid_value(*arg_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid value index {} at argument position {} in start function",
                    arg_idx, idx
                ))));
            }
        }

        // Add result values to the value index space
        for _result_idx in 0..start.results {
            // Values created by the start function are tracked elsewhere
        }
    }
    Ok(())
}

/// Validate an export, ensuring it references a valid index and has a valid type.
fn validate_export(export: &Export, ctx: &ValidationContext) -> Result<()> {
    // Validate export name
    validate_export_name(&export.name)?;

    // Validate that the sort and index are valid
    match export.sort {
        Sort::Core(core_sort) => {
            match core_sort {
                CoreSort::Function => {
                    // Validate function index against the core scope
                    if export.idx >= ctx.modules.len() as u32 {
                        return Err(Error::new(kinds::ValidationError(format!(
                            "Invalid core function index {} in export",
                            export.idx
                        ))));
                    }
                }
                CoreSort::Table => {
                    // Validate table index against the core scope
                    if export.idx >= ctx.modules.len() as u32 {
                        return Err(Error::new(kinds::ValidationError(format!(
                            "Invalid core table index {} in export",
                            export.idx
                        ))));
                    }
                }
                CoreSort::Memory => {
                    // Validate memory index against the core scope
                    if export.idx >= ctx.modules.len() as u32 {
                        return Err(Error::new(kinds::ValidationError(format!(
                            "Invalid core memory index {} in export",
                            export.idx
                        ))));
                    }
                }
                CoreSort::Global => {
                    // Validate global index against the core scope
                    if export.idx >= ctx.modules.len() as u32 {
                        return Err(Error::new(kinds::ValidationError(format!(
                            "Invalid core global index {} in export",
                            export.idx
                        ))));
                    }
                }
                CoreSort::Type => {
                    // Validate type index against the core scope
                    if export.idx >= ctx.core_types.len() as u32 {
                        return Err(Error::new(kinds::ValidationError(format!(
                            "Invalid core type index {} in export",
                            export.idx
                        ))));
                    }
                }
                CoreSort::Module => {
                    // Validate module index against the core scope
                    if export.idx >= ctx.modules.len() as u32 {
                        return Err(Error::new(kinds::ValidationError(format!(
                            "Invalid core module index {} in export",
                            export.idx
                        ))));
                    }
                }
                CoreSort::Instance => {
                    // Validate instance index against the core scope
                    if export.idx >= ctx.core_instances.len() as u32 {
                        return Err(Error::new(kinds::ValidationError(format!(
                            "Invalid core instance index {} in export",
                            export.idx
                        ))));
                    }
                }
            }
        }
        Sort::Function => {
            // Validate function index
            if !ctx.is_valid_func(export.idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid function index {} in export",
                    export.idx
                ))));
            }
        }
        Sort::Value => {
            // Validate value index
            if !ctx.is_valid_value(export.idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid value index {} in export",
                    export.idx
                ))));
            }
        }
        Sort::Type => {
            // Validate type index
            if !ctx.is_valid_component_type(export.idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid component type index {} in export",
                    export.idx
                ))));
            }
        }
        Sort::Component => {
            // Validate component index
            if !ctx.is_valid_component(export.idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid component index {} in export",
                    export.idx
                ))));
            }
        }
        Sort::Instance => {
            // Validate instance index
            if !ctx.is_valid_instance(export.idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid instance index {} in export",
                    export.idx
                ))));
            }
        }
    }

    // Validate declared type against the actual item type (if provided)
    if let Some(ref ty) = export.ty {
        // Validate the type itself
        validate_extern_type(ty, ctx)?;

        // In a full implementation, we would also check that:
        // 1. The declared type is compatible with the actual type of the exported item
        // 2. For resource types, that resource ownership is properly tracked
    }

    Ok(())
}

/// Validate export name format
fn validate_export_name(name: &wrt_format::component::ExportName) -> Result<()> {
    // Basic syntax validation
    if name.name.is_empty() {
        return Err(Error::new(kinds::ValidationError(
            "Export name cannot be empty".to_string(),
        )));
    }

    // Validate semver format if present
    if let Some(semver) = &name.semver {
        if !is_valid_semver(semver) {
            return Err(Error::new(kinds::ValidationError(format!(
                "Invalid semver format in export name: {}",
                semver
            ))));
        }
    }

    // Validate integrity hash if present
    if let Some(integrity) = &name.integrity {
        if !is_valid_integrity(integrity) {
            return Err(Error::new(kinds::ValidationError(format!(
                "Invalid integrity hash in export name: {}",
                integrity
            ))));
        }
    }

    // Validate resource naming if applicable
    if name.is_resource {
        // In a full implementation, we would validate that the resource
        // naming follows the conventions in the spec
    }

    Ok(())
}

/// Validate an import, ensuring it has a valid type.
fn validate_import(import: &Import, ctx: &ValidationContext) -> Result<()> {
    // Validate the import name format
    validate_import_name(&import.name)?;

    // Validate the import type
    validate_extern_type(&import.ty, ctx)?;

    // Additional validation for resource imports
    if let ExternType::Value(val_type) = &import.ty {
        if let ValType::Own(type_idx) = val_type {
            // Check it's a valid resource type
            if !ctx.is_valid_resource_type(*type_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Import references invalid resource type: {}",
                    type_idx
                ))));
            }
        } else if let ValType::Borrow(type_idx) = val_type {
            // Check it's a valid resource type
            if !ctx.is_valid_resource_type(*type_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Import references invalid resource type: {}",
                    type_idx
                ))));
            }
        }
    } else if let ExternType::Type(type_idx) = &import.ty {
        // Check if component type exists
        if !ctx.is_valid_component_type(*type_idx) {
            return Err(Error::new(kinds::ValidationError(format!(
                "Import references invalid component type index: {}",
                type_idx
            ))));
        }
    }

    Ok(())
}

/// Validate import name format
fn validate_import_name(name: &wrt_format::component::ImportName) -> Result<()> {
    // Basic syntax validation
    if name.namespace.is_empty() {
        return Err(Error::new(kinds::ValidationError(
            "Import namespace cannot be empty".to_string(),
        )));
    }

    if name.name.is_empty() {
        return Err(Error::new(kinds::ValidationError(
            "Import name cannot be empty".to_string(),
        )));
    }

    // Validate allowed characters in namespace and name
    // In a full implementation, we would check for valid UTF-8 characters
    // and disallow control characters

    Ok(())
}

/// Validate an external type
fn validate_extern_type(
    ty: &wrt_format::component::ExternType,
    ctx: &ValidationContext,
) -> Result<()> {
    use wrt_format::component::ExternType;

    match ty {
        ExternType::Function { params, results } => {
            // Validate function parameters
            for (_name, param_type) in params {
                validate_val_type(param_type, ctx)?;
            }

            // Validate function results
            for result_type in results {
                validate_val_type(result_type, ctx)?;
            }
        }
        ExternType::Value(val_type) => {
            validate_val_type(val_type, ctx)?;
        }
        ExternType::Type(idx) => {
            // Validate that the type index exists
            if !ctx.is_valid_component_type(*idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid component type index {} in extern type",
                    idx
                ))));
            }
        }
        ExternType::Instance { exports } => {
            // Validate instance exports
            for (_name, ty) in exports {
                validate_extern_type(ty, ctx)?;
            }
        }
        ExternType::Component { imports, exports } => {
            // Validate component imports
            for (_namespace, _name, ty) in imports {
                validate_extern_type(ty, ctx)?;
            }

            // Validate component exports
            for (_name, ty) in exports {
                validate_extern_type(ty, ctx)?;
            }
        }
    }

    Ok(())
}

/// Validate a value type
fn validate_val_type(
    val_type: &wrt_format::component::ValType,
    ctx: &ValidationContext,
) -> Result<()> {
    match val_type {
        ValType::Bool
        | ValType::S8
        | ValType::U8
        | ValType::S16
        | ValType::U16
        | ValType::S32
        | ValType::U32
        | ValType::S64
        | ValType::U64
        | ValType::F32
        | ValType::F64
        | ValType::Char
        | ValType::String => {
            // Primitive types are always valid
            Ok(())
        }
        ValType::Ref(type_idx) => {
            // Type reference must be a valid component type index
            if !ctx.is_valid_component_type(*type_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid type reference: {}",
                    type_idx
                ))));
            }
            Ok(())
        }
        ValType::Record(fields) => {
            // Validate each field type
            if fields.is_empty() {
                return Err(Error::new(kinds::ValidationError(
                    "Record type must have at least one field".to_string(),
                )));
            }

            // Check for duplicate field names
            let mut field_names = HashSet::new();
            for (name, field_type) in fields {
                if !field_names.insert(name) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Duplicate field name in record: {}",
                        name
                    ))));
                }

                // Recursively validate field type
                validate_val_type(field_type, ctx)?;
            }
            Ok(())
        }
        ValType::Variant(cases) => {
            // Variant must have at least one case
            if cases.is_empty() {
                return Err(Error::new(kinds::ValidationError(
                    "Variant type must have at least one case".to_string(),
                )));
            }

            // Check for duplicate case names
            let mut case_names = HashSet::new();
            for (name, case_type) in cases {
                if !case_names.insert(name) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Duplicate case name in variant: {}",
                        name
                    ))));
                }

                // Recursively validate case type if present
                if let Some(case_type) = case_type {
                    validate_val_type(case_type, ctx)?;
                }
            }
            Ok(())
        }
        ValType::List(elem_type) => {
            // Recursively validate element type
            validate_val_type(elem_type, ctx)
        }
        ValType::Tuple(types) => {
            // Tuple must have at least one type
            if types.is_empty() {
                return Err(Error::new(kinds::ValidationError(
                    "Tuple type must have at least one element".to_string(),
                )));
            }

            // Recursively validate each type in the tuple
            for ty in types {
                validate_val_type(ty, ctx)?;
            }
            Ok(())
        }
        ValType::Flags(names) => {
            // Flags must have at least one name
            if names.is_empty() {
                return Err(Error::new(kinds::ValidationError(
                    "Flags type must have at least one flag".to_string(),
                )));
            }

            // Check for duplicate flag names
            let mut flag_names = HashSet::new();
            for name in names {
                if !flag_names.insert(name) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Duplicate flag name: {}",
                        name
                    ))));
                }
            }
            Ok(())
        }
        ValType::Enum(names) => {
            // Enum must have at least one name
            if names.is_empty() {
                return Err(Error::new(kinds::ValidationError(
                    "Enum type must have at least one variant".to_string(),
                )));
            }

            // Check for duplicate enum variant names
            let mut variant_names = HashSet::new();
            for name in names {
                if !variant_names.insert(name) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Duplicate enum variant name: {}",
                        name
                    ))));
                }
            }
            Ok(())
        }
        ValType::Option(inner_type) => {
            // Recursively validate the inner type
            validate_val_type(inner_type, ctx)
        }
        ValType::Result(ok_type) => {
            // Recursively validate the ok type
            validate_val_type(ok_type, ctx)
        }
        ValType::ResultErr(err_type) => {
            // Recursively validate the error type
            validate_val_type(err_type, ctx)
        }
        ValType::ResultBoth(ok_type, err_type) => {
            // Recursively validate both ok and error types
            validate_val_type(ok_type, ctx)?;
            validate_val_type(err_type, ctx)
        }
        ValType::Own(type_idx) => {
            // Type reference must be a valid resource type index
            if !ctx.is_valid_resource_type(*type_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid resource type reference in Own: {}",
                    type_idx
                ))));
            }
            Ok(())
        }
        ValType::Borrow(type_idx) => {
            // Type reference must be a valid resource type index
            if !ctx.is_valid_resource_type(*type_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid resource type reference in Borrow: {}",
                    type_idx
                ))));
            }
            Ok(())
        }
    }
}

/// Validate a core type, ensuring it is well-formed.
fn validate_core_type(core_type: &CoreType, _ctx: &ValidationContext) -> Result<()> {
    match &core_type.definition {
        CoreTypeDefinition::Function { params, results } => {
            // WebAssembly spec limits function signature to 1000 params and 1000 results
            if params.len() > 1000 {
                return Err(Error::new(kinds::ValidationError(
                    "Core function type has too many parameters".to_string(),
                )));
            }

            if results.len() > 1000 {
                return Err(Error::new(kinds::ValidationError(
                    "Core function type has too many results".to_string(),
                )));
            }
        }
        CoreTypeDefinition::Module { imports, exports } => {
            // Validate module imports
            for (module, name, ty) in imports {
                // Validate import name format
                if module.is_empty() {
                    return Err(Error::new(kinds::ValidationError(
                        "Core module import module name cannot be empty".to_string(),
                    )));
                }

                if name.is_empty() {
                    return Err(Error::new(kinds::ValidationError(
                        "Core module import item name cannot be empty".to_string(),
                    )));
                }

                // Validate the type is a valid core type
                match ty {
                    wrt_format::component::CoreExternType::Function {
                        params: _,
                        results: _,
                    }
                    | wrt_format::component::CoreExternType::Global {
                        value_type: _,
                        mutable: _,
                    }
                    | wrt_format::component::CoreExternType::Memory {
                        min: _,
                        max: _,
                        shared: _,
                    }
                    | wrt_format::component::CoreExternType::Table {
                        element_type: _,
                        min: _,
                        max: _,
                    } => {
                        // These are valid core types
                    }
                }
            }

            // Check for duplicate import names
            let mut import_names = HashSet::new();
            for (module, name, _) in imports {
                let full_name = format!("{}.{}", module, name);
                if !import_names.insert(full_name) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Duplicate import name in core module type: {}.{}",
                        module, name
                    ))));
                }
            }

            // Validate module exports
            for (name, ty) in exports {
                // Validate export name format
                if name.is_empty() {
                    return Err(Error::new(kinds::ValidationError(
                        "Core module export name cannot be empty".to_string(),
                    )));
                }

                // Validate the type is a valid core type
                match ty {
                    wrt_format::component::CoreExternType::Function {
                        params: _,
                        results: _,
                    }
                    | wrt_format::component::CoreExternType::Global {
                        value_type: _,
                        mutable: _,
                    }
                    | wrt_format::component::CoreExternType::Memory {
                        min: _,
                        max: _,
                        shared: _,
                    }
                    | wrt_format::component::CoreExternType::Table {
                        element_type: _,
                        min: _,
                        max: _,
                    } => {
                        // These are valid core types
                    }
                }
            }

            // Check for duplicate export names
            let mut export_names = HashSet::new();
            for (name, _) in exports {
                if !export_names.insert(name) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Duplicate export name in core module type: {}",
                        name
                    ))));
                }
            }
        }
    }

    Ok(())
}

/// Validate a core instance, ensuring it references valid modules and instances.
fn validate_core_instance(core_instance: &CoreInstance, ctx: &ValidationContext) -> Result<()> {
    match &core_instance.instance_expr {
        CoreInstanceExpr::Instantiate { module_idx, args } => {
            // Check that the module index is valid
            if !ctx.is_valid_module(*module_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Core instance references invalid module index: {}",
                    module_idx
                ))));
            }

            // Check that all instance indices in args are valid
            for arg in args {
                if !ctx.is_valid_core_instance(arg.instance_idx) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Core instance arg references invalid instance index: {}",
                        arg.instance_idx
                    ))));
                }

                // Check export name is not empty
                if arg.name.is_empty() {
                    return Err(Error::new(kinds::ValidationError(
                        "Core instance arg export name cannot be empty".to_string(),
                    )));
                }
            }

            // Check for duplicate export references
            let mut export_names = HashSet::new();
            for arg in args {
                let full_ref = format!("{}.{}", arg.instance_idx, arg.name);
                if !export_names.insert(full_ref) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Duplicate export reference in core instance: instance {} export {}",
                        arg.instance_idx, arg.name
                    ))));
                }
            }
        }
        CoreInstanceExpr::InlineExports(exports) => {
            // Validate inline exports
            let mut export_names = HashSet::new();

            for export in exports {
                // Validate export name is not empty
                if export.name.is_empty() {
                    return Err(Error::new(kinds::ValidationError(
                        "Core instance inline export name cannot be empty".to_string(),
                    )));
                }

                // Check for duplicate export names
                if !export_names.insert(&export.name) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Duplicate export name in core instance: {}",
                        export.name
                    ))));
                }

                // Validate sort-specific indices
                match export.sort {
                    wrt_format::component::CoreSort::Function => {
                        // Would check function index if available
                    }
                    wrt_format::component::CoreSort::Table => {
                        // Would check table index if available
                    }
                    wrt_format::component::CoreSort::Memory => {
                        // Would check memory index if available
                    }
                    wrt_format::component::CoreSort::Global => {
                        // Would check global index if available
                    }
                    wrt_format::component::CoreSort::Type => {
                        if !ctx.is_valid_core_type(export.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Core instance export references invalid core type index: {}",
                                export.idx
                            ))));
                        }
                    }
                    wrt_format::component::CoreSort::Module => {
                        if !ctx.is_valid_module(export.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Core instance export references invalid module index: {}",
                                export.idx
                            ))));
                        }
                    }
                    wrt_format::component::CoreSort::Instance => {
                        if !ctx.is_valid_core_instance(export.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Core instance export references invalid core instance index: {}",
                                export.idx
                            ))));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Validate a component type, ensuring it is well-formed
fn validate_component_type(
    comp_type: &wrt_format::component::ComponentType,
    ctx: &mut ValidationContext,
) -> Result<()> {
    use wrt_format::component::ComponentTypeDefinition;

    match &comp_type.definition {
        ComponentTypeDefinition::Component { imports, exports } => {
            // Validate component imports
            for (_namespace, _name, ty) in imports {
                validate_extern_type(ty, ctx)?;
            }

            // Validate component exports
            for (_name, ty) in exports {
                validate_extern_type(ty, ctx)?;
            }
        }
        ComponentTypeDefinition::Instance { exports } => {
            // Validate instance exports
            for (_name, ty) in exports {
                validate_extern_type(ty, ctx)?;
            }
        }
        ComponentTypeDefinition::Function { params, results } => {
            // Validate function parameters
            for (_name, param_type) in params {
                validate_val_type(param_type, ctx)?;
            }

            // Validate function results
            for result_type in results {
                validate_val_type(result_type, ctx)?;
            }
        }
        ComponentTypeDefinition::Value(val_type) => {
            validate_val_type(val_type, ctx)?;
        }
        ComponentTypeDefinition::Resource {
            representation,
            nullable: _,
        } => {
            // Record this as a valid resource type
            let resource_idx = ctx.component_types.len() as u32 - 1;
            ctx.add_resource_type(resource_idx);

            // Validate the resource representation
            validate_resource_representation(representation, ctx)?;
        }
    }

    Ok(())
}

/// Validate a resource representation
fn validate_resource_representation(
    representation: &wrt_format::component::ResourceRepresentation,
    ctx: &ValidationContext,
) -> Result<()> {
    match representation {
        ResourceRepresentation::Handle32 | ResourceRepresentation::Handle64 => {
            // Simple handle representations are always valid
            Ok(())
        }
        ResourceRepresentation::Record(field_names) => {
            // Validate that field names are unique
            let mut seen_fields = HashSet::new();
            for name in field_names {
                if !seen_fields.insert(name) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Duplicate field name '{}' in resource record representation",
                        name
                    ))));
                }
            }

            // Record representation must have at least one field
            if field_names.is_empty() {
                return Err(Error::new(kinds::ValidationError(
                    "Resource record representation must have at least one field".to_string(),
                )));
            }

            Ok(())
        }
        ResourceRepresentation::Aggregate(type_indices) => {
            // Validate that all referenced types exist
            for (i, type_idx) in type_indices.iter().enumerate() {
                if !ctx.is_valid_component_type(*type_idx) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Invalid component type index {} in resource aggregate representation at position {}",
                        type_idx, i
                    ))));
                }

                // In a full implementation, we would validate that each referenced type
                // is a suitable representation type (e.g., not a function type)
            }

            // Aggregate representation must reference at least one type
            if type_indices.is_empty() {
                return Err(Error::new(kinds::ValidationError(
                    "Resource aggregate representation must reference at least one type"
                        .to_string(),
                )));
            }

            Ok(())
        }
    }
}

/// Validate an instance, ensuring it references valid components and indices.
fn validate_instance(instance: &Instance, ctx: &ValidationContext) -> Result<()> {
    match &instance.instance_expr {
        InstanceExpr::Instantiate {
            component_idx,
            args,
        } => {
            // Check that the component index is valid
            if !ctx.is_valid_component(*component_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Instance references invalid component index: {}",
                    component_idx
                ))));
            }

            // Check that all referenced indices in args are valid
            for arg in args {
                match arg.sort {
                    Sort::Core(core_sort) => {
                        match core_sort {
                            CoreSort::Function => {
                                // In a full implementation, check core function index
                            }
                            CoreSort::Table => {
                                // In a full implementation, check core table index
                            }
                            CoreSort::Memory => {
                                // In a full implementation, check core memory index
                            }
                            CoreSort::Global => {
                                // In a full implementation, check core global index
                            }
                            CoreSort::Type => {
                                if !ctx.is_valid_core_type(arg.idx) {
                                    return Err(Error::new(kinds::ValidationError(format!(
                                        "Instance arg references invalid core type index: {}",
                                        arg.idx
                                    ))));
                                }
                            }
                            CoreSort::Module => {
                                if !ctx.is_valid_module(arg.idx) {
                                    return Err(Error::new(kinds::ValidationError(format!(
                                        "Instance arg references invalid module index: {}",
                                        arg.idx
                                    ))));
                                }
                            }
                            CoreSort::Instance => {
                                if !ctx.is_valid_core_instance(arg.idx) {
                                    return Err(Error::new(kinds::ValidationError(format!(
                                        "Instance arg references invalid core instance index: {}",
                                        arg.idx
                                    ))));
                                }
                            }
                        }
                    }
                    Sort::Function => {
                        if !ctx.is_valid_func(arg.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Instance arg references invalid function index: {}",
                                arg.idx
                            ))));
                        }
                    }
                    Sort::Value => {
                        if !ctx.is_valid_value(arg.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Instance arg references invalid value index: {}",
                                arg.idx
                            ))));
                        }

                        // Mark that this value is consumed
                        // Can't modify the context here due to borrowing rules
                        // ctx.mark_value_consumed(arg.idx);
                    }
                    Sort::Type => {
                        if !ctx.is_valid_component_type(arg.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Instance arg references invalid component type index: {}",
                                arg.idx
                            ))));
                        }
                    }
                    Sort::Component => {
                        if !ctx.is_valid_component(arg.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Instance arg references invalid component index: {}",
                                arg.idx
                            ))));
                        }
                    }
                    Sort::Instance => {
                        if !ctx.is_valid_instance(arg.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Instance arg references invalid instance index: {}",
                                arg.idx
                            ))));
                        }
                    }
                }

                // Check that arg name is not empty
                if arg.name.is_empty() {
                    return Err(Error::new(kinds::ValidationError(
                        "Instance arg name cannot be empty".to_string(),
                    )));
                }
            }

            // Check for duplicate arg names
            let mut arg_names = HashSet::new();
            for arg in args {
                if !arg_names.insert(&arg.name) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Duplicate arg name in instance: {}",
                        arg.name
                    ))));
                }
            }
        }
        InstanceExpr::InlineExports(exports) => {
            // Validate inline exports
            let mut export_names = HashSet::new();

            for export in exports {
                // Validate export name is not empty
                if export.name.is_empty() {
                    return Err(Error::new(kinds::ValidationError(
                        "Instance inline export name cannot be empty".to_string(),
                    )));
                }

                // Check for duplicate export names
                if !export_names.insert(&export.name) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Duplicate export name in instance: {}",
                        export.name
                    ))));
                }

                // Validate based on sort
                match export.sort {
                    Sort::Core(core_sort) => {
                        match core_sort {
                            CoreSort::Function => {
                                // Would check core function index if available
                            }
                            CoreSort::Table => {
                                // Would check core table index if available
                            }
                            CoreSort::Memory => {
                                // Would check core memory index if available
                            }
                            CoreSort::Global => {
                                // Would check core global index if available
                            }
                            CoreSort::Type => {
                                if !ctx.is_valid_core_type(export.idx) {
                                    return Err(Error::new(kinds::ValidationError(format!(
                                        "Instance export references invalid core type index: {}",
                                        export.idx
                                    ))));
                                }
                            }
                            CoreSort::Module => {
                                if !ctx.is_valid_module(export.idx) {
                                    return Err(Error::new(kinds::ValidationError(format!(
                                        "Instance export references invalid module index: {}",
                                        export.idx
                                    ))));
                                }
                            }
                            CoreSort::Instance => {
                                if !ctx.is_valid_core_instance(export.idx) {
                                    return Err(Error::new(kinds::ValidationError(format!(
                                        "Instance export references invalid core instance index: {}",
                                        export.idx
                                    ))));
                                }
                            }
                        }
                    }
                    Sort::Function => {
                        if !ctx.is_valid_func(export.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Instance export references invalid function index: {}",
                                export.idx
                            ))));
                        }
                    }
                    Sort::Value => {
                        if !ctx.is_valid_value(export.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Instance export references invalid value index: {}",
                                export.idx
                            ))));
                        }

                        // Would mark value as consumed here
                        // ctx.mark_value_consumed(export.idx);
                    }
                    Sort::Type => {
                        if !ctx.is_valid_component_type(export.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Instance export references invalid component type index: {}",
                                export.idx
                            ))));
                        }
                    }
                    Sort::Component => {
                        if !ctx.is_valid_component(export.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Instance export references invalid component index: {}",
                                export.idx
                            ))));
                        }
                    }
                    Sort::Instance => {
                        if !ctx.is_valid_instance(export.idx) {
                            return Err(Error::new(kinds::ValidationError(format!(
                                "Instance export references invalid instance index: {}",
                                export.idx
                            ))));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Validate a canonical operation
fn validate_canon(canon: &wrt_format::component::Canon, ctx: &ValidationContext) -> Result<()> {
    use wrt_format::component::{CanonOperation, ResourceOperation};

    match &canon.operation {
        CanonOperation::Lift {
            func_idx,
            type_idx,
            options,
        } => {
            // Validate that the core function index is valid
            if !ctx.is_valid_func(*func_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid core function index {} in lift operation",
                    func_idx
                ))));
            }

            // Validate that the type index is valid
            if !ctx.is_valid_component_type(*type_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid component type index {} in lift operation",
                    type_idx
                ))));
            }

            // Validate any provided options
            validate_lift_options(options, ctx)?;

            // In a full implementation, we would also check that:
            // 1. The core function's type is compatible with the component function type
            // 2. The memory_idx (if provided) refers to a valid memory

            Ok(())
        }
        CanonOperation::Lower { func_idx, options } => {
            // Validate that the component function index is valid
            if !ctx.is_valid_func(*func_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid component function index {} in lower operation",
                    func_idx
                ))));
            }

            // Validate any provided options
            validate_lower_options(options, ctx)?;

            // In a full implementation, we would also check that:
            // 1. The component function's type is compatible with lowering
            // 2. The memory_idx (if provided) refers to a valid memory

            Ok(())
        }
        CanonOperation::Resource(resource_op) => {
            match resource_op {
                ResourceOperation::New(resource_new) => {
                    // Validate that the resource type index is valid
                    if !ctx.is_valid_resource_type(resource_new.type_idx) {
                        return Err(Error::new(kinds::ValidationError(format!(
                            "Invalid resource type index {} in resource new operation",
                            resource_new.type_idx
                        ))));
                    }
                    Ok(())
                }
                ResourceOperation::Drop(resource_drop) => {
                    // Validate that the resource type index is valid
                    if !ctx.is_valid_resource_type(resource_drop.type_idx) {
                        return Err(Error::new(kinds::ValidationError(format!(
                            "Invalid resource type index {} in resource drop operation",
                            resource_drop.type_idx
                        ))));
                    }
                    Ok(())
                }
                ResourceOperation::Rep(resource_rep) => {
                    // Validate that the resource type index is valid
                    if !ctx.is_valid_resource_type(resource_rep.type_idx) {
                        return Err(Error::new(kinds::ValidationError(format!(
                            "Invalid resource type index {} in resource rep operation",
                            resource_rep.type_idx
                        ))));
                    }
                    Ok(())
                }
            }
        }
    }
}

/// Validate lift options
fn validate_lift_options(
    options: &wrt_format::component::LiftOptions,
    ctx: &ValidationContext,
) -> Result<()> {
    // Check memory index if specified
    if let Some(memory_idx) = options.memory_idx {
        // In a full implementation, we would validate the memory index
        // For now, we just check it's not out of bounds
        if memory_idx >= ctx.modules.len() as u32 {
            return Err(Error::new(kinds::ValidationError(format!(
                "Invalid memory index {} in lift options",
                memory_idx
            ))));
        }
    }

    // Check string encoding if specified
    if let Some(string_encoding) = &options.string_encoding {
        // Check that the encoding is valid
        match string_encoding {
            wrt_format::component::StringEncoding::UTF8 => Ok(()),
            wrt_format::component::StringEncoding::UTF16 => Ok(()),
            wrt_format::component::StringEncoding::Latin1 => Ok(()),
            wrt_format::component::StringEncoding::ASCII => Ok(()),
        }
    } else {
        Ok(())
    }
}

/// Validate lower options
fn validate_lower_options(
    options: &wrt_format::component::LowerOptions,
    ctx: &ValidationContext,
) -> Result<()> {
    // Check memory index if specified
    if let Some(memory_idx) = options.memory_idx {
        // In a full implementation, we would validate the memory index
        // For now, we just check it's not out of bounds
        if memory_idx >= ctx.modules.len() as u32 {
            return Err(Error::new(kinds::ValidationError(format!(
                "Invalid memory index {} in lower options",
                memory_idx
            ))));
        }
    }

    // Check string encoding if specified
    if let Some(string_encoding) = &options.string_encoding {
        // Check that the encoding is valid
        match string_encoding {
            wrt_format::component::StringEncoding::UTF8 => Ok(()),
            wrt_format::component::StringEncoding::UTF16 => Ok(()),
            wrt_format::component::StringEncoding::Latin1 => Ok(()),
            wrt_format::component::StringEncoding::ASCII => Ok(()),
        }
    } else {
        Ok(())
    }
}

fn validate_values(component: &Component, ctx: &mut ValidationContext) -> Result<()> {
    for (idx, value) in component.values.iter().enumerate() {
        // Validate the value type
        validate_val_type(&value.ty, ctx)?;

        // Validate that the encoded data matches the expected type
        validate_encoded_value(&value.data, &value.ty, ctx)?;

        // Add value to the context
        let value_idx = ctx.values.len() as u32;
        ctx.add_value(value_idx);
        ctx.mark_value_unconsumed(value_idx);

        // Handle resource values specially
        if let ValType::Own(resource_idx) = &value.ty {
            if !ctx.is_valid_resource_type(*resource_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Value references invalid resource type index: {}",
                    resource_idx
                ))));
            }

            // Track the resource creation
            ctx.track_resource_created(idx as u32);
        }
    }
    Ok(())
}

/// Validate that encoded data matches the expected type
fn validate_encoded_value(data: &[u8], val_type: &ValType, ctx: &ValidationContext) -> Result<()> {
    use wrt_format::component::ValType;

    match val_type {
        // Primitive types - validate size constraints
        ValType::Bool => {
            if data.len() != 1 || (data[0] != 0 && data[0] != 1) {
                return Err(Error::new(kinds::ValidationError(
                    "Invalid boolean value encoding".to_string(),
                )));
            }
            Ok(())
        }
        ValType::S8 | ValType::U8 => {
            if data.len() != 1 {
                return Err(Error::new(kinds::ValidationError(
                    "Invalid 8-bit integer encoding".to_string(),
                )));
            }
            Ok(())
        }
        ValType::S16 | ValType::U16 => {
            if data.len() != 2 {
                return Err(Error::new(kinds::ValidationError(
                    "Invalid 16-bit integer encoding".to_string(),
                )));
            }
            Ok(())
        }
        ValType::S32 | ValType::U32 => {
            if data.len() != 4 {
                return Err(Error::new(kinds::ValidationError(
                    "Invalid 32-bit integer encoding".to_string(),
                )));
            }
            Ok(())
        }
        ValType::S64 | ValType::U64 => {
            if data.len() != 8 {
                return Err(Error::new(kinds::ValidationError(
                    "Invalid 64-bit integer encoding".to_string(),
                )));
            }
            Ok(())
        }
        ValType::F32 => {
            if data.len() != 4 {
                return Err(Error::new(kinds::ValidationError(
                    "Invalid f32 encoding".to_string(),
                )));
            }
            Ok(())
        }
        ValType::F64 => {
            if data.len() != 8 {
                return Err(Error::new(kinds::ValidationError(
                    "Invalid f64 encoding".to_string(),
                )));
            }
            Ok(())
        }
        ValType::Char => {
            // Validate UTF-8 encoding of a single character
            match std::str::from_utf8(data) {
                Ok(s) => {
                    if s.chars().count() != 1 {
                        return Err(Error::new(kinds::ValidationError(
                            "Char value must encode exactly one Unicode character".to_string(),
                        )));
                    }
                    Ok(())
                }
                Err(_) => Err(Error::new(kinds::ValidationError(
                    "Invalid UTF-8 encoding for char value".to_string(),
                ))),
            }
        }
        ValType::String => {
            // Validate UTF-8 encoding
            match std::str::from_utf8(data) {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::new(kinds::ValidationError(
                    "Invalid UTF-8 encoding for string value".to_string(),
                ))),
            }
        }
        // For complex types, this validation would need access to the full decoded value
        // This is a placeholder that could be extended with more detailed validation
        // based on the actual decoded structures
        _ => {
            // Basic validation succeeded, but detailed validation
            // of composite types requires the full decoder
            Ok(())
        }
    }
}

/// Validate resource usage in a component
fn validate_resources(component: &Component, ctx: &mut ValidationContext) -> Result<()> {
    // For each component type definition, identify resource types
    for (idx, type_def) in component.types.iter().enumerate() {
        match &type_def.definition {
            ComponentTypeDefinition::Resource { representation, .. } => {
                // Add this resource type to the context
                ctx.add_resource_type(idx as u32);

                // Validate resource representation
                validate_resource_representation(representation, ctx)?;
            }
            _ => {}
        }
    }

    // For value types that are resources, check they reference valid resource types
    for value in &component.values {
        if let ValType::Own(resource_idx) = value.ty {
            if !ctx.is_valid_resource_type(resource_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Value with invalid resource type: {}",
                    resource_idx
                ))));
            }
        }
    }

    Ok(())
}

fn validate_import_export_compatibility(
    ctx: &ValidationContext,
    imported_type: &ExternType,
    exported_type: &ExternType,
) -> Result<()> {
    if !is_compatible_type(imported_type, exported_type) {
        return Err(Error::new(kinds::ValidationError(format!(
            "Incompatible import/export types"
        ))));
    }

    // Special validation for resource types
    match (imported_type, exported_type) {
        (ExternType::Value(ValType::Own(i_idx)), ExternType::Value(ValType::Own(e_idx))) => {
            if i_idx != e_idx {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Incompatible resource types: imported {} != exported {}",
                    i_idx, e_idx
                ))));
            }

            // Check that both are valid resource types
            if !ctx.is_valid_resource_type(*i_idx) || !ctx.is_valid_resource_type(*e_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid resource type index in import/export"
                ))));
            }
        }
        (ExternType::Value(ValType::Borrow(i_idx)), ExternType::Value(ValType::Borrow(e_idx))) => {
            if i_idx != e_idx {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Incompatible borrowed resource types: imported {} != exported {}",
                    i_idx, e_idx
                ))));
            }

            // Check that both are valid resource types
            if !ctx.is_valid_resource_type(*i_idx) || !ctx.is_valid_resource_type(*e_idx) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Invalid resource type index in import/export"
                ))));
            }
        }
        _ => {
            // Other types are handled by the compatibility check above
        }
    }

    Ok(())
}
