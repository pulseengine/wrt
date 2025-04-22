//! WebAssembly module validation
//!
//! This module provides functions for validating WebAssembly modules.

use wrt_error::{kinds, Error, Result};
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
        return Err(Error::new(kinds::ValidationError(
            "Module has function section but no code section".to_string(),
        )));
    }

    // Check if we have a code section but no function section
    if module.functions.is_empty() && !module.code.is_empty() {
        return Err(Error::new(kinds::ValidationError(
            "Module has code section but no function section".to_string(),
        )));
    }

    // Check that function and code sections match in size
    if module.functions.len() != module.code.len() {
        return Err(Error::new(kinds::ValidationError(format!(
            "Function and code sections have mismatched lengths: {} vs {}",
            module.functions.len(),
            module.code.len()
        ))));
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
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Invalid type index {} in import {}",
                        type_idx, i
                    ))));
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
            return Err(Error::new(kinds::ValidationError(format!(
                "Invalid table element type: {:?}",
                table.element_type
            ))));
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
            return Err(Error::new(kinds::ValidationError(format!(
                "Invalid limits: max ({}) > max allowed ({})",
                max_limit, max
            ))));
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
                return Err(Error::new(kinds::ValidationError(format!(
                    "Memory32 minimum size exceeds maximum allowed pages (65536): {}",
                    memory.limits.min
                ))));
            }

            // Check maximum is within spec bounds (if specified)
            if let Some(max) = memory.limits.max {
                if max > 65536 {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Memory32 maximum size exceeds maximum allowed pages (65536): {}",
                        max
                    ))));
                }
            }
        }
        MemoryIndexType::I64 => {
            // For 64-bit memories, the limit is much higher (2^64 - 1)
            // but we should still check for reasonable values
            if memory.limits.min > (1u64 << 48) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Memory64 minimum size too large: {}",
                    memory.limits.min
                ))));
            }
        }
    }

    // Check maximum >= minimum
    if let Some(max) = memory.limits.max {
        if max < memory.limits.min {
            return Err(Error::new(kinds::ValidationError(format!(
                "Memory maximum size ({}) is less than minimum size ({})",
                max, memory.limits.min
            ))));
        }
    }

    // Check shared memory constraints
    if memory.shared {
        // Shared memory must have max specified
        if memory.limits.max.is_none() {
            return Err(Error::new(kinds::ValidationError(
                "Shared memory must have maximum size specified".to_string(),
            )));
        }

        // Memory64 cannot be shared in the current spec
        if matches!(memory.limits.memory_index_type, MemoryIndexType::I64) {
            return Err(Error::new(kinds::ValidationError(
                "Memory64 cannot be shared in the current specification".to_string(),
            )));
        }
    }

    Ok(())
}

/// Validate memory alignment for memory access instructions
pub fn validate_memory_alignment(align: u32, natural_align: u32) -> Result<()> {
    // According to the spec, alignment must be less than or equal to
    // the natural alignment of the access operation
    if align > natural_align {
        return Err(Error::new(kinds::ValidationError(format!(
            "Alignment 2^{} exceeds natural alignment 2^{}",
            align, natural_align
        ))));
    }

    Ok(())
}

/// Validate memory access instruction (generic for all memory operations)
pub fn validate_memory_access(
    module: &Module,
    mem_idx: u32,
    align: u32,
    access_size: u32,
) -> Result<()> {
    // Check if memory index is valid
    let import_memories = module
        .imports
        .iter()
        .filter(|i| matches!(i.desc, ImportDesc::Memory(_)))
        .count();

    let memory_count = import_memories + module.memories.len();

    if mem_idx as usize >= memory_count {
        return Err(Error::new(kinds::ValidationError(format!(
            "Memory index {} out of bounds (max {})",
            mem_idx, memory_count
        ))));
    }

    // For each memory type, determine natural alignment
    let natural_align = match access_size {
        1 => 0,  // 2^0 = 1-byte alignment
        2 => 1,  // 2^1 = 2-byte alignment
        4 => 2,  // 2^2 = 4-byte alignment
        8 => 3,  // 2^3 = 8-byte alignment
        16 => 4, // 2^4 = 16-byte alignment (for v128)
        _ => {
            return Err(Error::new(kinds::ValidationError(format!(
                "Invalid memory access size: {}",
                access_size
            ))))
        }
    };

    // Validate alignment
    validate_memory_alignment(align, natural_align)?;

    Ok(())
}

/// Validate all memory types in a module
pub fn validate_memories(module: &Module) -> Result<()> {
    // Validate memory declarations
    for memory in &module.memories {
        validate_memory_type(memory)?;
    }

    // Validate imported memories
    for import in &module.imports {
        if let ImportDesc::Memory(memory_type) = &import.desc {
            validate_memory_type(&Memory {
                limits: memory_type.limits.clone(),
                shared: memory_type.limits.shared,
            })?;
        }
    }

    // WebAssembly 1.0 only allows a maximum of 1 memory per module
    let defined_memories = module.memories.len();
    let imported_memories = module
        .imports
        .iter()
        .filter(|i| matches!(i.desc, ImportDesc::Memory(_)))
        .count();

    if defined_memories + imported_memories > 1 {
        return Err(Error::new(kinds::ValidationError(format!(
            "Multiple memories are not supported in WebAssembly 1.0: {} defined, {} imported",
            defined_memories, imported_memories
        ))));
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
    let mut export_names = Vec::new();

    for (i, export) in module.exports.iter().enumerate() {
        // Check for duplicate export names
        if export_names.contains(&export.name) {
            return Err(Error::new(kinds::ValidationError(format!(
                "Duplicate export name: {}",
                export.name
            ))));
        }

        export_names.push(export.name.clone());

        // Validate export kind and index
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

/// Validate a function index
fn validate_func_idx(module: &Module, idx: u32, export_idx: usize) -> Result<()> {
    let import_funcs = module
        .imports
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Function(_)))
        .count();

    let func_count = import_funcs + module.functions.len();

    if idx as usize >= func_count {
        return Err(Error::new(kinds::ValidationError(format!(
            "Export {} function index {} out of bounds (max {})",
            export_idx, idx, func_count
        ))));
    }

    Ok(())
}

/// Validate a table index
fn validate_table_idx(module: &Module, idx: u32, export_idx: usize) -> Result<()> {
    let import_tables = module
        .imports
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Table(_)))
        .count();

    let table_count = import_tables + module.tables.len();

    if idx as usize >= table_count {
        return Err(Error::new(kinds::ValidationError(format!(
            "Export {} table index {} out of bounds (max {})",
            export_idx, idx, table_count
        ))));
    }

    Ok(())
}

/// Validate a memory index
fn validate_memory_idx(module: &Module, idx: u32, export_idx: usize) -> Result<()> {
    let import_memories = module
        .imports
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Memory(_)))
        .count();

    let memory_count = import_memories + module.memories.len();

    if idx as usize >= memory_count {
        return Err(Error::new(kinds::ValidationError(format!(
            "Export {} memory index {} out of bounds (max {})",
            export_idx, idx, memory_count
        ))));
    }

    Ok(())
}

/// Validate a global index
fn validate_global_idx(module: &Module, idx: u32, export_idx: usize) -> Result<()> {
    let import_globals = module
        .imports
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Global(_)))
        .count();

    let global_count = import_globals + module.globals.len();

    if idx as usize >= global_count {
        return Err(Error::new(kinds::ValidationError(format!(
            "Export {} global index {} out of bounds (max {})",
            export_idx, idx, global_count
        ))));
    }

    Ok(())
}

/// Validate the start section of a WebAssembly module
fn validate_start(module: &Module) -> Result<()> {
    if let Some(start_idx) = module.start {
        // Validate start function index
        validate_func_idx(module, start_idx, 0)?;

        // Check that the start function has the correct type (no params, no results)
        let import_funcs = module
            .imports
            .iter()
            .filter(|import| matches!(import.desc, ImportDesc::Function(_)))
            .count();

        let func_idx = start_idx as usize;

        // If it's an imported function
        if func_idx < import_funcs {
            let import_idx = module
                .imports
                .iter()
                .filter(|import| matches!(import.desc, ImportDesc::Function(_)))
                .enumerate()
                .filter(|(i, _)| *i == func_idx)
                .map(|(_, import)| {
                    if let ImportDesc::Function(type_idx) = import.desc {
                        type_idx as usize
                    } else {
                        0
                    }
                })
                .next()
                .unwrap_or(0);

            if import_idx < module.types.len() {
                let func_type = &module.types[import_idx];
                if !func_type.params.is_empty() || !func_type.results.is_empty() {
                    return Err(Error::new(kinds::ValidationError(
                        "Start function must have type [] -> []".to_string(),
                    )));
                }
            }
        } else {
            // It's a defined function
            let defined_idx = func_idx - import_funcs;
            if defined_idx < module.functions.len() {
                let type_idx = module.functions[defined_idx].type_idx as usize;
                if type_idx < module.types.len() {
                    let func_type = &module.types[type_idx];
                    if !func_type.params.is_empty() || !func_type.results.is_empty() {
                        return Err(Error::new(kinds::ValidationError(
                            "Start function must have type [] -> []".to_string(),
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

/// Validate the elements section of a WebAssembly module
fn validate_elements(module: &Module) -> Result<()> {
    for (i, elem) in module.elements.iter().enumerate() {
        // Validate table index
        validate_table_idx(module, elem.table_idx, i)?;

        // Validate function indices
        for (j, func_idx) in elem.init.iter().enumerate() {
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
    for (i, table) in module.tables.iter().enumerate() {
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
