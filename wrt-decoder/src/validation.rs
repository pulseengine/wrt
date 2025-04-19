//! WebAssembly module validation
//!
//! This module provides functions for validating WebAssembly modules.

use crate::module::Module;
use crate::sections::*;
use wrt_error::{kinds, Error, Result};
use wrt_format::module::{ExportKind, Global, Memory, Table};
use wrt_format::types::ValueType;

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
    // For each function type, check parameter and result types
    for (i, ty) in module.types.iter().enumerate() {
        // Validate parameter and result types are valid
        for param in &ty.params {
            validate_value_type(param)?;
        }

        for result in &ty.results {
            validate_value_type(result)?;
        }
    }

    Ok(())
}

/// Validate a WebAssembly value type
fn validate_value_type(ty: &ValueType) -> Result<()> {
    // All value types defined in the spec are valid
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
fn validate_limits(limits: &Limits, max: u32) -> Result<()> {
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
fn validate_memory_type(memory: &Memory) -> Result<()> {
    // In WebAssembly 1.0, memories can only have at most 65536 pages (4GiB)
    validate_limits(&memory.limits, 65536)
}

/// Validate a global type
fn validate_global_type(global: &Global) -> Result<()> {
    validate_value_type(&global.value_type)
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

/// Validate the memories section of a WebAssembly module
fn validate_memories(module: &Module) -> Result<()> {
    for (i, memory) in module.memories.iter().enumerate() {
        // Validate memory type
        validate_memory_type(memory)?;
    }

    Ok(())
}

/// Validate the globals section of a WebAssembly module
fn validate_globals(module: &Module) -> Result<()> {
    for (i, global) in module.globals.iter().enumerate() {
        // Validate global type
        validate_global_type(global)?;

        // TODO: Validate initialization expression
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

        // TODO: Validate initialization expression

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
        // Validate memory index
        validate_memory_idx(module, data.memory_idx, i)?;

        // TODO: Validate initialization expression
    }

    Ok(())
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
