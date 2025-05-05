//! WebAssembly module validation.
//!
//! This module provides functionality for validating WebAssembly modules
//! according to the WebAssembly specification.

use crate::module::Module;
use crate::prelude::*;
// Use the proper imports from wrt_format instead of local sections
use wrt_error::{codes, kinds, Error, ErrorCategory, Result};
use wrt_format::module::{DataMode, ExportKind, Global, ImportDesc, Memory, Table};
use wrt_format::types::{FuncType, Limits};

/// Validation configuration options
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum allowed function count
    pub max_function_count: usize,
    /// Maximum allowed import count
    pub max_import_count: usize,
    /// Maximum allowed export count
    pub max_export_count: usize,
    /// Maximum allowed memory size (in pages)
    pub max_memory_size: u32,
    /// Maximum allowed table size
    pub max_table_size: u32,
    /// Whether to verify function bodies
    pub verify_function_bodies: bool,
    /// Whether to verify memory limits
    pub verify_memory_limits: bool,
    /// Whether to verify table limits
    pub verify_table_limits: bool,
    /// Whether to perform strict validation (true) or relaxed validation (false)
    pub strict: bool,
    /// Maximum number of locals in a function
    pub max_locals: u32,
    /// Maximum number of globals
    pub max_globals: u32,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_function_count: 10000,
            max_import_count: 1000,
            max_export_count: 1000,
            max_memory_size: 65536, // 4GB
            max_table_size: 10000000,
            verify_function_bodies: true,
            verify_memory_limits: true,
            verify_table_limits: true,
            strict: true,
            max_locals: 50000,
            max_globals: 1000,
        }
    }
}

impl ValidationConfig {
    /// Create a new validation configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a validation configuration with relaxed settings
    pub fn relaxed() -> Self {
        Self {
            strict: false,
            verify_function_bodies: false,
            verify_memory_limits: false,
            verify_table_limits: false,
            ..Self::default()
        }
    }
}

/// Basic validation of a WebAssembly module
pub fn validate_module(module: &Module) -> Result<()> {
    validate_module_with_config(module, &ValidationConfig::default())
}

/// Validate a WebAssembly module with custom configuration
pub fn validate_module_with_config(module: &Module, config: &ValidationConfig) -> Result<()> {
    // Check for unique export names
    let mut export_names = Vec::new();
    for export in &module.exports {
        if export_names.contains(&export.name) {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!("Duplicate export name: {}", export.name),
            ));
        }
        export_names.push(export.name.clone());
    }

    // Skip further validation if we're in relaxed mode
    if !config.strict {
        return Ok(());
    }

    // Apply limits based on configuration
    if module.functions.len() > config.max_function_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Module has too many functions: {} (max: {})",
                module.functions.len(),
                config.max_function_count
            ),
        ));
    }

    if module.imports.len() > config.max_import_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Module has too many imports: {} (max: {})",
                module.imports.len(),
                config.max_import_count
            ),
        ));
    }

    if module.exports.len() > config.max_export_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Module has too many exports: {} (max: {})",
                module.exports.len(),
                config.max_export_count
            ),
        ));
    }

    // Validate basic structure, which we always do
    validate_basic_structure(module)?;
    validate_types(module)?;
    validate_imports(module)?;
    validate_functions(module)?;
    validate_tables(module)?;
    validate_memories(module)?;
    validate_globals(module)?;
    validate_exports(module)?;
    validate_start(module)?;
    validate_elements(module)?;
    validate_data(module)?;

    // Validate code if configured to do so
    if config.verify_function_bodies {
        validate_code(module)?;
    }

    Ok(())
}

/// Validate the basic structure of a WebAssembly module
fn validate_basic_structure(module: &Module) -> Result<()> {
    // Check if we have a function section but no code section
    if !module.functions.is_empty() && module.code.is_empty() {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError("Module has function section but no code section".to_string()),
        ));
    }

    // Check if we have a code section but no function section
    if module.functions.is_empty() && !module.code.is_empty() {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError("Module has code section but no function section".to_string()),
        ));
    }

    // Check that function and code sections match in size
    if module.functions.len() != module.code.len() {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            #[cfg(feature = "std")]
            kinds::ValidationError(format!(
                "Function and code sections have mismatched lengths: {} vs {}",
                module.functions.len(),
                module.code.len()
            )),
            #[cfg(all(feature = "alloc", not(feature = "std")))]
            kinds::ValidationError(alloc::format!(
                "Function and code sections have mismatched lengths: {} vs {}",
                module.functions.len(),
                module.code.len()
            )),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            kinds::ValidationError(
                "Function and code sections have mismatched lengths".to_string(),
            ),
        ));
    }

    Ok(())
}

/// Validate the types section of a WebAssembly module
fn validate_types(module: &Module) -> Result<()> {
    for (i, func_type) in module.types.iter().enumerate() {
        // Validate function type
        validate_func_type(func_type, i)?;
    }
    Ok(())
}

/// Validate a value type
fn validate_value_type(value_type: &ValueType, context: &str) -> Result<()> {
    // In MVPv1, only i32, i64, f32, and f64 are valid
    match value_type {
        ValueType::I32 | ValueType::I64 | ValueType::F32 | ValueType::F64 => Ok(()),
        ValueType::FuncRef | ValueType::ExternRef => {
            // Reference types are part of later specifications
            Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!("{}: reference types not supported in MVPv1", context),
            ))
        }
    }
}

/// Validate function type
fn validate_func_type(func_type: &FuncType, type_idx: usize) -> Result<()> {
    // Validate parameter types
    for (i, param) in func_type.params.iter().enumerate() {
        let context = format!("parameter {} of type {}", i, type_idx);
        validate_value_type(param, &context)?;
    }

    // Validate result types
    for (i, result) in func_type.results.iter().enumerate() {
        let context = format!("result {} of type {}", i, type_idx);
        validate_value_type(result, &context)?;
    }

    // In MVP, functions can have at most one result
    if func_type.results.len() > 1 {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Function type {} has {} results (max: 1 in MVP)",
                type_idx,
                func_type.results.len()
            ),
        ));
    }

    Ok(())
}

/// Validate the imports section of a WebAssembly module
fn validate_imports(module: &Module) -> Result<()> {
    for (i, import) in module.imports.iter().enumerate() {
        match &import.desc {
            ImportDesc::Function(type_idx) => {
                // Validate type index
                if *type_idx as usize >= module.types.len() {
                    return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::VALIDATION_ERROR,
                        format!("Invalid type index {} in import {}", type_idx, i),
                    ));
                }
            }
            ImportDesc::Table(table) => {
                // Validate table type
                validate_table_type(table)?;
            }
            ImportDesc::Memory(memory) => {
                // Validate memory type
                validate_memory_type(memory)?;
            }
            ImportDesc::Global(global) => {
                // Validate global type
                validate_global_type(global)?;
            }
        }
    }

    Ok(())
}

/// Validate a table type
fn validate_table_type(table: &Table) -> Result<()> {
    // Validate reference type (must be funcref or externref)
    match table.element_type {
        ValueType::FuncRef | ValueType::ExternRef => {}
        _ => {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                kinds::ValidationError(format!(
                    "Invalid table element type: {:?}",
                    table.element_type
                )),
            ));
        }
    }

    // Validate limits
    validate_limits(&table.limits, 0xFFFF_FFFF)
}

/// Validate limits (min/max)
fn validate_limits(limits: &Limits, max: u64) -> Result<()> {
    // If max is specified, it must be >= min
    if let Some(max_limit) = limits.max {
        if max_limit > max {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                kinds::ValidationError(format!(
                    "Invalid limits: max ({}) > max allowed ({})",
                    max_limit, max
                )),
            ));
        }
    }

    Ok(())
}

/// Validate a memory type
pub fn validate_memory_type(memory: &Memory) -> Result<()> {
    // Check limits based on memory index type
    if memory.limits.memory64 {
        // For 64-bit memories, the limit is much higher (2^64 - 1)
        // but we should still check for reasonable values
        if memory.limits.min > (1u64 << 48) {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!("Memory64 minimum size too large: {}", memory.limits.min),
            ));
        }
    } else {
        // For 32-bit memories, enforce the 4GiB (65536 pages) limit
        if memory.limits.min > 65536 {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!(
                    "Memory32 minimum size exceeds maximum allowed pages (65536): {}",
                    memory.limits.min
                ),
            ));
        }

        // Check maximum is within spec bounds (if specified)
        if let Some(max) = memory.limits.max {
            if max > 65536 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    format!(
                        "Memory32 maximum size exceeds maximum allowed pages (65536): {}",
                        max
                    ),
                ));
            }
        }
    }

    // Check that min <= max (if max is specified)
    if let Some(max) = memory.limits.max {
        if memory.limits.min > max {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!(
                    "Memory minimum size ({}) exceeds maximum size ({})",
                    memory.limits.min, max
                ),
            ));
        }
    }

    Ok(())
}

/// Validate memory alignment requirements
pub fn validate_memory_alignment(align: u32, natural_align: u32) -> Result<()> {
    if align > 32 {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!("Alignment must be <= 32, got {}", align),
        ));
    }

    let align_bytes = 1u32 << align;

    if align_bytes > natural_align {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Alignment 2^{} exceeds natural alignment {} for the memory access",
                align, natural_align
            ),
        ));
    }

    Ok(())
}

/// Validate memory access (used for instructions that access memory)
pub fn validate_memory_access(
    module: &Module,
    mem_idx: u32,
    align: u32,
    access_size: u32,
) -> Result<()> {
    // First validate memory index
    validate_memory_idx(module, mem_idx, 0)?;

    // Validate alignment
    validate_memory_alignment(align, access_size)?;

    Ok(())
}

/// Validate the memories section of a WebAssembly module
pub fn validate_memories(module: &Module) -> Result<()> {
    // In MVP, only one memory is allowed
    if module.memories.len() > 1 {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            "Multiple memories are not allowed in MVP".to_string(),
        ));
    }

    // Count imported memories
    let imported_memories = module
        .imports
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Memory(_)))
        .count();

    // Total memory count is defined memories + imported memories
    let total_memories = module.memories.len() + imported_memories;

    if total_memories > 1 {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Too many memories: {} defined + {} imported = {} (max 1)",
                module.memories.len(),
                imported_memories,
                total_memories
            ),
        ));
    }

    // Validate each memory
    for memory in &module.memories {
        validate_memory_type(memory)?;
    }

    Ok(())
}

/// Validate the globals section of a WebAssembly module
fn validate_globals(module: &Module) -> Result<()> {
    for global in &module.globals {
        validate_global_type(global)?;
    }

    Ok(())
}

/// Validate the exports section of a WebAssembly module
fn validate_exports(module: &Module) -> Result<()> {
    #[cfg(feature = "std")]
    let mut export_names = std::collections::HashSet::new();
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    let mut export_names = alloc::collections::BTreeSet::new();

    for (i, export) in module.exports.iter().enumerate() {
        // Check for duplicate export names
        if !export_names.insert(&export.name) {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!("Duplicate export name: {}", export.name),
            ));
        }

        // Validate export based on kind
        match export.kind {
            ExportKind::Function => {
                validate_func_idx(module, export.index, i)?;
            }
            ExportKind::Table => {
                validate_table_idx(module, export.index, i)?;
            }
            ExportKind::Memory => {
                validate_memory_idx(module, export.index, i)?;
            }
            ExportKind::Global => {
                validate_global_idx(module, export.index, i)?;
            }
        }
    }

    Ok(())
}

/// Validate a function index (used for exports)
fn validate_func_idx(module: &Module, idx: u32, _export_idx: usize) -> Result<()> {
    let func_count = module.functions.len() as u32;
    let imported_func_count = module
        .imports
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Function(_)))
        .count() as u32;

    if idx >= func_count + imported_func_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Invalid function index {} (max: {})",
                idx,
                func_count + imported_func_count - 1
            ),
        ));
    }

    Ok(())
}

/// Validate a table index (used for exports)
fn validate_table_idx(module: &Module, idx: u32, _export_idx: usize) -> Result<()> {
    let table_count = module.tables.len() as u32;
    let imported_table_count = module
        .imports
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Table(_)))
        .count() as u32;

    if idx >= table_count + imported_table_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Invalid table index {} (max: {})",
                idx,
                table_count + imported_table_count - 1
            ),
        ));
    }

    Ok(())
}

/// Validate a memory index (used for exports)
fn validate_memory_idx(module: &Module, idx: u32, _export_idx: usize) -> Result<()> {
    let memory_count = module.memories.len() as u32;
    let imported_memory_count = module
        .imports
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Memory(_)))
        .count() as u32;

    if idx >= memory_count + imported_memory_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Invalid memory index {} (max: {})",
                idx,
                memory_count + imported_memory_count - 1
            ),
        ));
    }

    Ok(())
}

/// Validate a global index (used for exports)
fn validate_global_idx(module: &Module, idx: u32, _export_idx: usize) -> Result<()> {
    let global_count = module.globals.len() as u32;
    let imported_global_count = module
        .imports
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Global(_)))
        .count() as u32;

    if idx >= global_count + imported_global_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Invalid global index {} (max: {})",
                idx,
                global_count + imported_global_count - 1
            ),
        ));
    }

    Ok(())
}

/// Validate the start function of a WebAssembly module
fn validate_start(module: &Module) -> Result<()> {
    if let Some(start_func) = module.start {
        // Validate function index
        validate_func_idx(module, start_func, 0)?;

        // In MVP, the start function must have type [] -> []
        let _func_count = module.functions.len() as u32;
        let imported_func_count = module
            .imports
            .iter()
            .filter(|import| matches!(import.desc, ImportDesc::Function(_)))
            .count() as u32;

        let mut type_idx = None;

        if start_func < imported_func_count {
            // Get type index from import
            let import_idx = start_func as usize;
            let mut count = 0;
            for import in &module.imports {
                if let ImportDesc::Function(idx) = import.desc {
                    if count == import_idx {
                        type_idx = Some(idx);
                        break;
                    }
                    count += 1;
                }
            }
        } else {
            // Get type index from function section
            let func_idx = (start_func - imported_func_count) as usize;
            if func_idx < module.functions.len() {
                type_idx = Some(module.functions[func_idx]);
            }
        }

        if let Some(type_idx) = type_idx {
            if type_idx as usize >= module.types.len() {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    format!("Invalid type index {} for start function", type_idx),
                ));
            }

            let func_type = &module.types[type_idx as usize];
            if !func_type.params.is_empty() || !func_type.results.is_empty() {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    "Start function must have type [] -> []".to_string(),
                ));
            }
        }
    }

    Ok(())
}

/// Validate the elements section of a WebAssembly module
fn validate_elements(module: &Module) -> Result<()> {
    for (i, elem) in module.elements.iter().enumerate() {
        // Validate table index
        if i == 0 && elem.table_idx > 0 {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "In MVP, elements must refer to table index 0".to_string(),
            ));
        }

        validate_table_idx(module, elem.table_idx, i)?;

        // Validate function indices in the element
        for (j, func_idx_expr) in elem.init.iter().enumerate() {
            // For now we just use a simple approach until we understand the ConstExpr type better
            // In MVP, these would be simple function indices
            // Just skip validation if we can't get a valid function index
            if let Ok(func_idx) = func_idx_expr.to_string().parse::<u32>() {
                validate_func_idx(module, func_idx, j)?;
            }
        }
    }

    Ok(())
}

/// Validate the data section of a WebAssembly module
fn validate_data(module: &Module) -> Result<()> {
    for (i, data) in module.data.iter().enumerate() {
        // In MVP, only memory 0 is valid
        if data.memory_idx > 0 {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "In MVP, data must refer to memory index 0".to_string(),
            ));
        }

        validate_memory_idx(module, data.memory_idx, i)?;

        // For active segments, validate that the init expr is an i32.const
        if matches!(data.mode, DataMode::Active) {
            validate_const_expr(&data.offset, ValueType::I32)?;
        }
    }

    Ok(())
}

/// Validate constant expression (used in globals, elem, and data segments)
fn validate_const_expr(expr: &[u8], _expected_type: ValueType) -> Result<()> {
    // In the MVP, constant expressions are limited to:
    // - i32.const
    // - i64.const
    // - f32.const
    // - f64.const
    // - global.get of an immutable imported global

    // For now, we just do a basic check that the expression isn't empty
    if expr.is_empty() {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            "Constant expression cannot be empty".to_string(),
        ));
    }

    // TODO: Add more comprehensive validation of constant expressions

    Ok(())
}

/// Validate code section of a WebAssembly module
fn validate_code(module: &Module) -> Result<()> {
    // Validate each function body
    for (i, code) in module.code.iter().enumerate() {
        // For MVP, we do basic validation that the code isn't empty
        if code.body.is_empty() {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!("Function body cannot be empty for function {}", i),
            ));
        }

        // TODO: Add more comprehensive code validation in the future
    }

    Ok(())
}

/// Validate the functions section of a WebAssembly module
fn validate_functions(module: &Module) -> Result<()> {
    for (i, type_idx) in module.functions.iter().enumerate() {
        if *type_idx as usize >= module.types.len() {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!("Invalid type index {} in function {}", type_idx, i),
            ));
        }
    }

    Ok(())
}

/// Validate the tables section of a WebAssembly module
fn validate_tables(module: &Module) -> Result<()> {
    // In MVP, only one table is allowed
    if module.tables.len() > 1 {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            "Multiple tables are not allowed in MVP".to_string(),
        ));
    }

    // Validate each table
    for table in &module.tables {
        validate_table_type(table)?;
    }

    Ok(())
}

/// Validate global type
fn validate_global_type(global: &Global) -> Result<()> {
    validate_value_type(&global.global_type.value_type, "global type")?;
    Ok(())
}

/// Validate memory.copy instruction
pub fn validate_memory_copy(
    module: &Module,
    dst_memory_idx: u32,
    src_memory_idx: u32,
) -> Result<()> {
    // Validate destination memory index
    validate_memory_idx(module, dst_memory_idx, 0)?;

    // Validate source memory index
    validate_memory_idx(module, src_memory_idx, 0)?;

    // In WasmMVP, both indices should be 0 as only one memory is allowed
    if dst_memory_idx != 0 || src_memory_idx != 0 {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Invalid memory indices for memory.copy: dst={}, src={} (only 0 is valid in MVP)",
                dst_memory_idx, src_memory_idx
            ),
        ));
    }

    Ok(())
}

/// Validate memory.fill instruction
pub fn validate_memory_fill(module: &Module, memory_idx: u32) -> Result<()> {
    // Validate memory index
    validate_memory_idx(module, memory_idx, 0)?;

    // In WasmMVP, memory index should be 0
    if memory_idx != 0 {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!(
                "Invalid memory index for memory.fill: {} (only 0 is valid in MVP)",
                memory_idx
            ),
        ));
    }

    Ok(())
}

/// Helper function to create validation errors
pub fn validation_error(message: &str) -> Error {
    Error::new(
        ErrorCategory::Validation,
        codes::VALIDATION_ERROR,
        kinds::ValidationError(message.to_string()),
    )
}

/// Helper function to create validation errors with context
pub fn validation_error_with_context(message: &str, context: &str) -> Error {
    Error::new(
        ErrorCategory::Validation,
        codes::VALIDATION_ERROR,
        kinds::ValidationError(format!("{}: {}", context, message)),
    )
}

/// Helper function to create validation errors with type information
pub fn validation_error_with_type(message: &str, type_name: &str) -> Error {
    Error::new(
        ErrorCategory::Validation,
        codes::VALIDATION_ERROR,
        kinds::ValidationError(format!("{} (type: {})", message, type_name)),
    )
}
