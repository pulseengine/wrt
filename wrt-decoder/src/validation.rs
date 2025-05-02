//! WebAssembly module validation
//!
//! This module provides functions for validating WebAssembly modules.

use wrt_error::{kinds, Error, Result, WrtError, codes, ErrorCategory};
use wrt_format::types::ValueType;
use wrt_types::FuncType;
// Use our prelude for common imports
use crate::prelude::*;

use crate::module::Module;
use crate::sections::{Global, ImportDesc, Limits, Memory, MemoryIndexType, Table};
use wrt_format::module::{DataMode, ExportKind};

/// Validate a WebAssembly module
///
/// This checks that the module follows the WebAssembly specification.
pub fn validate_module(module: &Module) -> Result<()> {
    // Validate basic structure
    validate_basic_structure(module)?;

    // Validate types
    validate_types(module)?;

    // Validate imports
    validate_imports(module)?;

    // Validate functions
    validate_functions(module)?;

    // Validate tables
    validate_tables(module)?;

    // Validate memories
    validate_memories(module)?;

    // Validate globals
    validate_globals(module)?;

    // Validate exports
    validate_exports(module)?;

    // Validate start function
    validate_start(module)?;

    // Validate elements
    validate_elements(module)?;

    // Validate data
    validate_data(module)?;

    // Validate code
    validate_code(module)?;

    Ok(())
}

/// Validate the basic structure of a WebAssembly module
fn validate_basic_structure(module: &Module) -> Result<()> {
    // Check if we have a function section but no code section
    if !module.functions.is_empty() && module.code.is_empty() {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(
                "Module has function section but no code section".to_string(),
            )
        ));
    }

    // Check if we have a code section but no function section
    if module.functions.is_empty() && !module.code.is_empty() {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(
                "Module has code section but no function section".to_string(),
            )
        ));
    }

    // Check that function and code sections match in size
    if module.functions.len() != module.code.len() {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(format!(
                "Function and code sections have mismatched lengths: {} vs {}",
                module.functions.len(),
                module.code.len()
            ))
        ));
    }

    Ok(())
}

/// Validate the types section of a WebAssembly module
fn validate_types(module: &Module) -> Result<()> {
    for ty in module.types.iter() {
        // Validate each function type
        validate_func_type(ty)?;
    }
    Ok(())
}

/// Validate a value type
fn validate_value_type(_ty: &ValueType) -> Result<()> {
    // All value types are valid
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
                        kinds::ValidationError(format!(
                            "Invalid type index {} in import {}",
                            type_idx, i
                        ))
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
                ))
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
                ))
            ));
        }
    }

    Ok(())
}

/// Validate a memory type
pub fn validate_memory_type(memory: &Memory) -> Result<()> {
    // Check limits based on memory index type
    match memory.limits.memory_index_type {
        MemoryIndexType::I32 => {
            // For 32-bit memories, enforce the 4GiB (65536 pages) limit
            if memory.limits.min > 65536 {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    kinds::ValidationError(format!(
                        "Memory32 minimum size exceeds maximum allowed pages (65536): {}",
                        memory.limits.min
                    ))
                ));
            }

            // Check maximum is within spec bounds (if specified)
            if let Some(max) = memory.limits.max {
                if max > 65536 {
                    return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::VALIDATION_ERROR,
                        kinds::ValidationError(format!(
                            "Memory32 maximum size exceeds maximum allowed pages (65536): {}",
                            max
                        ))
                    ));
                }
            }
        }
        MemoryIndexType::I64 => {
            // For 64-bit memories, the limit is much higher (2^64 - 1)
            // but we should still check for reasonable values
            if memory.limits.min > (1u64 << 48) {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    kinds::ValidationError(format!(
                        "Memory64 minimum size too large: {}",
                        memory.limits.min
                    ))
                ));
            }
        }
    }

    // Check maximum >= minimum
    if let Some(max) = memory.limits.max {
        if max < memory.limits.min {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                kinds::ValidationError(format!(
                    "Memory maximum size ({}) is less than minimum size ({})",
                    max, memory.limits.min
                ))
            ));
        }
    }

    // Validate shared memory requirements
    if memory.shared {
        if memory.limits.max.is_none() {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                kinds::ValidationError(
                    "Shared memory must have maximum size specified".to_string(),
                )
            ));
        }

        // Memory64 cannot be shared in the current specification
        if memory.limits.memory_index_type == MemoryIndexType::I64 {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                kinds::ValidationError(
                    "Memory64 cannot be shared in the current specification".to_string(),
                )
            ));
        }
    }

    Ok(())
}

/// Validate memory alignment
///
/// The alignment must not exceed the natural alignment of the access.
/// For example, a 4-byte access must not have an alignment larger than 2 (2^2 = 4).
pub fn validate_memory_alignment(align: u32, natural_align: u32) -> Result<()> {
    if align > natural_align {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(format!(
                "Alignment 2^{} exceeds natural alignment 2^{}",
                align, natural_align
            ))
        ));
    }
    Ok(())
}

/// Validate a memory access instruction
///
/// This validates:
/// 1. The memory index is valid
/// 2. The alignment is valid for the access size
/// 3. The access size is valid (1, 2, 4, or 8 bytes)
pub fn validate_memory_access(
    module: &Module,
    mem_idx: u32,
    align: u32,
    access_size: u32,
) -> Result<()> {
    // Validate memory index
    let memory_count = module.memories.len() as u32 + module.imported_memories.len() as u32;
    if mem_idx >= memory_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(format!(
                "Memory index {} out of bounds (max {})",
                mem_idx, memory_count
            ))
        ));
    }

    // Validate access size
    match access_size {
        1 => validate_memory_alignment(align, 0)?, // natural align for 1 byte is 2^0
        2 => validate_memory_alignment(align, 1)?, // natural align for 2 bytes is 2^1
        4 => validate_memory_alignment(align, 2)?, // natural align for 4 bytes is 2^2
        8 => validate_memory_alignment(align, 3)?, // natural align for 8 bytes is 2^3
        _ => {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                kinds::ValidationError(format!(
                    "Invalid memory access size: {}",
                    access_size
                ))
            ))
        }
    }

    Ok(())
}

/// Validate all memory types in a module
pub fn validate_memories(module: &Module) -> Result<()> {
    // Count imported memories
    let imported_memories = module.imports.iter()
        .filter(|i| matches!(i.desc, ImportDesc::Memory(_)))
        .count();
    
    // Count defined memories
    let defined_memories = module.memories.len();
    
    // In the MVP, only 1 memory is allowed (imported or defined)
    if imported_memories + defined_memories > 1 {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(format!(
                "Multiple memories are not supported in WebAssembly 1.0: {} defined, {} imported",
                defined_memories, imported_memories
            ))
        ));
    }
    
    // Validate each memory definition
    for memory in &module.memories {
        validate_memory_type(memory)?;
    }
    
    Ok(())
}

/// Validate the globals section of a WebAssembly module
fn validate_globals(module: &Module) -> Result<()> {
    for global in &module.globals {
        // Validate global type
        validate_value_type(&global.global_type.value_type)?;

        // Note: We'd normally validate init expr here, but skipping for simplicity
    }
    Ok(())
}

/// Validate the exports section of a WebAssembly module
fn validate_exports(module: &Module) -> Result<()> {
    // Check for duplicate export names
    let mut export_names = std::collections::HashSet::new();
    
    for export in &module.exports {
        // Check for duplicate export names
        if !export_names.insert(&export.name) {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                kinds::ValidationError(format!(
                    "Duplicate export name: {}",
                    export.name
                ))
            ));
        }
        
        // Validate the export kind
        match export.kind {
            ExportKind::Function => validate_func_idx(module, export.idx, export_names.len() - 1)?,
            ExportKind::Table => validate_table_idx(module, export.idx, export_names.len() - 1)?,
            ExportKind::Memory => validate_memory_idx(module, export.idx, export_names.len() - 1)?,
            ExportKind::Global => validate_global_idx(module, export.idx, export_names.len() - 1)?,
        }
    }
    
    Ok(())
}

/// Validate a function index
fn validate_func_idx(module: &Module, idx: u32, export_idx: usize) -> Result<()> {
    // Count total functions (imported + defined)
    let func_count = module.functions.len() + module.imported_functions.len();
    
    // Check index is within bounds
    if idx as usize >= func_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(format!(
                "Export {} function index {} out of bounds (max {})",
                export_idx, idx, func_count
            ))
        ));
    }
    
    Ok(())
}

/// Validate a table index
fn validate_table_idx(module: &Module, idx: u32, export_idx: usize) -> Result<()> {
    // Count total tables (imported + defined)
    let table_count = module.tables.len() + module.imported_tables.len();
    
    // Check index is within bounds
    if idx as usize >= table_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(format!(
                "Export {} table index {} out of bounds (max {})",
                export_idx, idx, table_count
            ))
        ));
    }
    
    Ok(())
}

/// Validate a memory index
fn validate_memory_idx(module: &Module, idx: u32, export_idx: usize) -> Result<()> {
    // Count total memories (imported + defined)
    let memory_count = module.memories.len() + module.imported_memories.len();
    
    // Check index is within bounds
    if idx as usize >= memory_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(format!(
                "Export {} memory index {} out of bounds (max {})",
                export_idx, idx, memory_count
            ))
        ));
    }
    
    Ok(())
}

/// Validate a global index
fn validate_global_idx(module: &Module, idx: u32, export_idx: usize) -> Result<()> {
    // Count total globals (imported + defined)
    let global_count = module.globals.len() + module.imported_globals.len();
    
    // Check index is within bounds
    if idx as usize >= global_count {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(format!(
                "Export {} global index {} out of bounds (max {})",
                export_idx, idx, global_count
            ))
        ));
    }
    
    Ok(())
}

/// Validate the start section of a WebAssembly module
fn validate_start(module: &Module) -> Result<()> {
    // If there's no start function, that's valid
    if module.start.is_none() {
        return Ok(());
    }
    
    // Get the start function index
    let start_idx = module.start.unwrap();
    
    // Validate the function index
    let func_count = module.functions.len() + module.imported_functions.len();
    if start_idx as usize >= func_count {
        return Err(validation_error(&format!(
            "Start function index {} out of bounds (max {})",
            start_idx, func_count
        )));
    }
    
    // Get the function type index
    let mut type_idx = 0;
    
    if start_idx as usize < module.imported_functions.len() {
        // It's an imported function, find its type
        let import_function = &module.imported_functions[start_idx as usize];
        type_idx = import_function.type_idx;
    } else {
        // It's a defined function, find its type
        let adjusted_idx = start_idx as usize - module.imported_functions.len();
        if adjusted_idx < module.functions.len() {
            type_idx = module.functions[adjusted_idx].type_idx;
        } else {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                kinds::ValidationError(
                    "Start function must have type [] -> []".to_string(),
                )
            ));
        }
    }
    
    // Check that the type exists
    if type_idx as usize >= module.types.len() {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(
                "Start function must have type [] -> []".to_string(),
            )
        ));
    }
    
    // Check that the function type is [] -> []
    let func_type = &module.types[type_idx as usize];
    if !func_type.params.is_empty() || !func_type.results.is_empty() {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            kinds::ValidationError(
                "Start function must have type [] -> []".to_string(),
            )
        ));
    }
    
    Ok(())
}

/// Validate the elements section of a WebAssembly module
fn validate_elements(module: &Module) -> Result<()> {
    for (i, elem) in module.elements.iter().enumerate() {
        // Validate table index
        validate_table_idx(module, elem.table_idx, i)?;

        // Validate function indices
        for func_idx in elem.init.iter() {
            validate_func_idx(module, *func_idx, i)?;
        }
    }

    Ok(())
}

/// Validate the data section of a WebAssembly module
fn validate_data(module: &Module) -> Result<()> {
    for (i, data) in module.data.iter().enumerate() {
        match data.mode {
            DataMode::Active => {
                // For active segments, validate the memory index
                validate_memory_idx(module, data.memory_idx, i)?;

                // Validate that the offset expression is a valid constant expression
                validate_const_expr(&data.offset, ValueType::I32)?;
            }
            DataMode::Passive => {
                // Passive segments have no additional validation requirements
            }
        }
    }

    Ok(())
}

/// Validate constant expressions in data offset fields
fn validate_const_expr(expr: &[u8], expected_type: ValueType) -> Result<()> {
    if expr.is_empty() {
        return Err(Error::new(kinds::ValidationError(
            "Empty constant expression".to_string(),
        )));
    }

    // Ensure the expression ends with end opcode (0x0B)
    if expr[expr.len() - 1] != 0x0B {
        return Err(Error::new(kinds::ValidationError(
            "Constant expression must end with end opcode (0x0B)".to_string(),
        )));
    }

    // In a real implementation, you would validate the entire constant expression
    // This is a simplified version that just checks for common constant expressions

    // Check for i32.const expressions
    if expr[0] == 0x41 && expr.len() >= 2 {
        // This is an i32.const, which is valid for memory offsets
        if expected_type == ValueType::I32 {
            return Ok(());
        }
    }

    // Check for i64.const expressions
    if expr[0] == 0x42 && expr.len() >= 2 {
        // This is an i64.const, which is valid for Memory64 offsets
        if expected_type == ValueType::I64 {
            return Ok(());
        }
    }

    // Check for global.get expressions
    if expr[0] == 0x23 && expr.len() >= 2 {
        // This is a global.get, which is valid if the global is immutable and of the right type
        // A full implementation would check the global type
        return Ok(());
    }

    Err(Error::new(kinds::ValidationError(format!(
        "Invalid constant expression for expected type {:?}",
        expected_type
    ))))
}

/// Validate the code section of a WebAssembly module
fn validate_code(module: &Module) -> Result<()> {
    if module.functions.len() != module.code.len() {
        return Err(Error::new(kinds::ValidationError(format!(
            "Function count ({}) does not match code count ({})",
            module.functions.len(),
            module.code.len()
        ))));
    }

    for i in 0..module.code.len() {
        // Validate max size (currently no actual limit in spec)

        // Validate locals
        for local_pair in &module.code[i].locals {
            // local_pair is (count, type)
            validate_value_type(&local_pair.1)?;
        }
    }

    Ok(())
}

/// Validate the functions section of a WebAssembly module
fn validate_functions(module: &Module) -> Result<()> {
    for (i, func) in module.functions.iter().enumerate() {
        // Validate function type index
        if func.type_idx as usize >= module.types.len() {
            return Err(Error::new(kinds::ValidationError(format!(
                "Function {} type index {} out of bounds (max {})",
                i,
                func.type_idx,
                module.types.len()
            ))));
        }
    }

    Ok(())
}

/// Validate the tables section of a WebAssembly module
fn validate_tables(module: &Module) -> Result<()> {
    for table in module.tables.iter() {
        // Validate table type
        validate_table_type(table)?;
    }

    Ok(())
}

/// Validate a global type
fn validate_global_type(global: &Global) -> Result<()> {
    validate_value_type(&global.global_type.value_type)?;
    Ok(())
}

/// Validate a WebAssembly function type
fn validate_func_type(ty: &FuncType) -> Result<()> {
    // Validate parameter and result types are valid
    for param in &ty.params {
        validate_value_type(param)?;
    }

    for result in &ty.results {
        validate_value_type(result)?;
    }

    Ok(())
}

/// Validate element segment
#[allow(dead_code)]
fn validate_elem(module: &Module) -> Result<()> {
    for elem in &module.elements {
        // Check table index
        if elem.table_idx >= module.tables.len() as u32 {
            return Err(Error::new(kinds::ValidationError(format!(
                "Element table index {} out of bounds",
                elem.table_idx
            ))));
        }

        // Note: We'd normally validate offset expr here, but skipping for simplicity

        // Check function indices
        for func_idx in elem.init.iter() {
            if *func_idx >= module.functions.len() as u32 {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Element function index {} out of bounds",
                    func_idx
                ))));
            }
        }
    }
    Ok(())
}

/// Validate memory copy operation
pub fn validate_memory_copy(module: &Module, dst_memory_idx: u32, src_memory_idx: u32) -> Result<()> {
    // Check if memory indices are valid
    let import_memories = module
        .imports
        .iter()
        .filter(|i| matches!(i.desc, ImportDesc::Memory(_)))
        .count();

    let memory_count = import_memories + module.memories.len();

    if dst_memory_idx as usize >= memory_count {
        return Err(Error::new(kinds::ValidationError(format!(
            "Destination memory index {} out of bounds (max {})",
            dst_memory_idx, memory_count
        ))));
    }

    if src_memory_idx as usize >= memory_count {
        return Err(Error::new(kinds::ValidationError(format!(
            "Source memory index {} out of bounds (max {})",
            src_memory_idx, memory_count
        ))));
    }

    Ok(())
}

/// Validate memory fill operation
pub fn validate_memory_fill(module: &Module, memory_idx: u32) -> Result<()> {
    // Check if memory index is valid
    let import_memories = module
        .imports
        .iter()
        .filter(|i| matches!(i.desc, ImportDesc::Memory(_)))
        .count();

    let memory_count = import_memories + module.memories.len();

    if memory_idx as usize >= memory_count {
        return Err(Error::new(kinds::ValidationError(format!(
            "Memory index {} out of bounds (max {})",
            memory_idx, memory_count
        ))));
    }

    Ok(())
}

pub fn validation_error(message: &str) -> WrtError {
    WrtError::validation_error(message.to_string())
}

pub fn validation_error_with_context(message: &str, context: &str) -> WrtError {
    WrtError::validation_error(format!("{}: {}", message, context))
}

pub fn validation_error_with_type(message: &str, type_name: &str) -> WrtError {
    WrtError::validation_error(format!("{} for type {}", message, type_name))
}
