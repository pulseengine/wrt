//! WebAssembly module representation
//!
//! This module provides a high-level representation of a WebAssembly module,
//! including all its sections, types, and functions.
//!
//! It serves as the bridge between the binary format (handled by wrt-format)
//! and the runtime representation (using wrt-foundation).

use wrt_error::{codes, Error, ErrorCategory, Result};
use wrt_format::binary::{WASM_MAGIC, WASM_VERSION};
use wrt_foundation::{
    safe_memory::SafeSlice,
    // Add MemoryProvider and SafeMemoryHandler for the new signature
    safe_memory::{MemoryProvider, SafeMemoryHandler},
    types::{
        Code as WrtCode,
        CustomSection as WrtCustomSection,
        DataMode as TypesDataMode,
        DataSegment,
        ElementMode as TypesElementMode,
        ElementSegment,
        Export,
        ExportDesc as TypesExportDesc,
        Expr as WrtExpr,
        FuncType,
        GlobalType,
        Import,
        ImportDesc as TypesImportDesc,
        LocalEntry as WrtLocalEntry,
        MemoryType,
        // Import the canonical Module, Code, Expr, LocalEntry from wrt_foundation
        Module as WrtModule,
        TableType,
        TypeIdx,   // Added TypeIdx for funcs field
        ValueType, // For LocalEntry
    },
    values::Value,
};

use crate::{instructions, prelude::*, types::*, Parser}; // Import instructions module

// Import DataMode directly to avoid reimport issues
// pub use wrt_format::module::DataMode as FormatDataMode; // This might be
// unused after refactor.

/// Module struct representing a parsed WebAssembly module.
/// This struct will now mirror wrt_foundation::types::Module's relevant fields
/// for the output. The internal parsing function `parse_module` will construct
/// an instance of `WrtModule`. The struct defined here is effectively a
/// placeholder for the type `WrtModule`.
// Instead of redefining Module here, the functions that return `Module` will return `WrtModule`.
// The local `struct Module` will be removed.

// Functions like `decode_module` will now return `Result<WrtModule>`
// Functions like `encode_module` will take `&WrtModule`

// Default impl for WrtModule might be better in wrt-foundation or not needed if constructed by
// parser. impl Default for WrtModule { // This should be for WrtModule if needed
// fn default() -> Self {
// Self::new() // WrtModule would need a ::new()
// }
// }
//
// Methods previously on `crate::module::Module` might need to be adapted if they operate
// on fields that have changed structure (e.g., accessing function code).
// For now, focus on the parsing logic in `parse_module_internal_logic` (renamed from
// `parse_module`).

/// Type alias for Module - now uses wrt_foundation's Module type
pub type Module = WrtModule;

/// Decode a WebAssembly module from binary format
///
/// # Notes
///
/// This function requires either the `std` or `alloc` feature to be enabled.
/// In pure no_std environments without alloc, this function will return an
/// error.
// Add MemoryProvider generic and handler argument
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn decode_module<P: MemoryProvider>(
    bytes: &[u8],
    _handler: &mut SafeMemoryHandler<P>, /* Handler is currently unused, for future BoundedVec
                                          * population */
) -> Result<WrtModule> {
    // TODO: When WrtModule uses BoundedVec, pass the handler to
    // parse_module_internal_logic and use it to construct WrtModule's fields.
    // For now, the internal logic still uses Vec, so this function implicitly
    // requires 'alloc'.
    let parser = Parser::new(Some(bytes), false);
    // The internal parse_module_internal_logic now returns WrtModule
    // It will also need the handler in the future.
    let (module, _remaining_bytes) = parse_module_internal_logic(parser)?;
    Ok(module)
}

/// Decode a WebAssembly module from binary format
///
/// # Notes
///
/// This is a no-op implementation for pure no_std environments without alloc.
/// It returns an error indicating that this function requires allocation
/// support.
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub fn decode_module<P: MemoryProvider>(
    _bytes: &[u8],
    _handler: &mut SafeMemoryHandler<P>,
) -> Result<WrtModule> {
    Err(Error::new(
        ErrorCategory::Runtime,
        codes::UNSUPPORTED_OPERATION,
        "decode_module requires 'std' or 'alloc' feature to be enabled",
    ))
}

/// Decode a WebAssembly module from binary format and store the original binary
///
/// # Notes
///
/// This function requires either the `std` or `alloc` feature to be enabled.
/// In pure no_std environments without alloc, this function will return an
/// error.
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn decode_module_with_binary<P: MemoryProvider>(
    binary: &[u8],
    handler: &mut SafeMemoryHandler<P>,
) -> Result<WrtModule> {
    // This function would need to handle how `binary: Option<SafeSlice<'static>>`
    // is populated if that field is desired on `WrtModule`. `WrtModule` as
    // defined in `wrt-foundation` does not have it. For now, let's assume
    // `WrtModule` is as defined in `wrt-foundation`. If `SafeSlice` needs to be
    // part of it, `wrt-foundation::Module` must be extended. This function
    // might be simplified to just call decode_module for now.
    decode_module(binary, handler)
}

/// Decode a WebAssembly module from binary format and store the original binary
///
/// # Notes
///
/// This is a no-op implementation for pure no_std environments without alloc.
/// It returns an error indicating that this function requires allocation
/// support.
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub fn decode_module_with_binary<P: MemoryProvider>(
    _binary: &[u8],
    _handler: &mut SafeMemoryHandler<P>,
) -> Result<WrtModule> {
    Err(Error::new(
        ErrorCategory::Runtime,
        codes::UNSUPPORTED_OPERATION,
        "decode_module_with_binary requires 'std' or 'alloc' feature to be enabled",
    ))
}

/// Encode a custom section to binary format
///
/// # Arguments
///
/// * `result` - Binary vector to append to
/// * `section` - Custom section to encode
///
/// # Returns
///
/// * `Result<()>` - Success or error
// This function uses Vec<u8> internally, so it's tied to 'alloc'.
// It's called by encode_module, which will be feature-gated.
fn encode_custom_section(result: &mut Vec<u8>, section: &WrtCustomSection) -> Result<()> {
    // Write section ID
    result.push(wrt_format::binary::CUSTOM_SECTION_ID);

    // Write section size placeholder (will be filled in later)
    let size_offset = result.len();
    result.extend_from_slice(&[0, 0, 0, 0]); // Placeholder for section size

    // Write name length and name
    write_string(result, &section.name)?;

    // Write section data
    result.extend_from_slice(&section.data);

    // Go back and write the section size
    let section_size = result.len() - size_offset - 4;
    let size_bytes = section_size.to_le_bytes();
    result[size_offset..size_offset + 4].copy_from_slice(&size_bytes);

    Ok(())
}

/// Encode a WebAssembly module to binary format
///
/// # Arguments
///
/// * `module` - Module to encode
///
/// # Returns
///
/// * `Result<Vec<u8>>` - Binary representation of the module
// This function returns Vec<u8> and uses Vec internally, so gate with 'alloc'.
#[cfg(feature = "alloc")]
pub fn encode_module(module: &WrtModule) -> Result<Vec<u8>> {
    // This would ideally use SafeMemory types, but for the final binary output
    // we need a Vec<u8> that can be returned as the serialized representation
    let mut result = Vec::new();

    // Write module header
    result.extend_from_slice(&WASM_MAGIC);
    result.extend_from_slice(&WASM_VERSION);

    // For a complete implementation, each section would be encoded here

    // Encode custom sections
    for section in &module.custom_sections {
        encode_custom_section(&mut result, section)?;
    }

    Ok(result)
}

/// Create a parse error with the given message
///
/// # Arguments
///
/// * `message` - Error message
///
/// # Returns
///
/// * `Error` - Parse error
pub fn parse_error(message: &str) -> Error {
    // TODO: If this needs to work without alloc, ensure Error::new doesn't rely on
    // formatted strings or use a version that takes pre-formatted parts.
    Error::new(ErrorCategory::Parse, codes::PARSE_ERROR, message)
}

/// Create a parse error with the given message and context
///
/// # Arguments
///
/// * `message` - Error message
/// * `context` - Additional context
///
/// # Returns
///
/// * `Error` - Parse error with context
// format! requires alloc. Conditionally compile or use a non-allocating alternative.
#[cfg(feature = "alloc")]
pub fn parse_error_with_context(message: &str, context: &str) -> Error {
    Error::new(ErrorCategory::Parse, codes::PARSE_ERROR, format!("{}: {}", message, context))
}

#[cfg(not(feature = "alloc"))]
pub fn parse_error_with_context(message: &str, _context: &str) -> Error {
    // Basic error if no alloc for formatting. Context is lost.
    Error::new(ErrorCategory::Parse, codes::PARSE_ERROR, message)
}

/// Create a parse error with the given message and position
///
/// # Arguments
///
/// * `message` - Error message
/// * `position` - Position in the binary
///
/// # Returns
///
/// * `Error` - Parse error with position
// format! requires alloc. Conditionally compile or use a non-allocating alternative.
#[cfg(feature = "alloc")]
pub fn parse_error_with_position(message: &str, position: usize) -> Error {
    Error::new(
        ErrorCategory::Parse,
        codes::PARSE_ERROR,
        format!("{} at position {}", message, position),
    )
}

#[cfg(not(feature = "alloc"))]
pub fn parse_error_with_position(message: &str, _position: usize) -> Error {
    // Basic error if no alloc for formatting. Position is lost.
    Error::new(ErrorCategory::Parse, codes::PARSE_ERROR, message)
}

/// Create a runtime error with the given message
///
/// # Arguments
///
/// * `message` - Error message
///
/// # Returns
///
/// * `Error` - Runtime error
pub fn runtime_error(message: &str) -> Error {
    Error::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, message)
}

/// Create a runtime error with the given message and context
///
/// # Arguments
///
/// * `message` - Error message
/// * `context` - Additional context
///
/// # Returns
///
/// * `Error` - Runtime error with context
// format! requires alloc. Conditionally compile or use a non-allocating alternative.
#[cfg(feature = "alloc")]
pub fn runtime_error_with_context(message: &str, context: &str) -> Error {
    Error::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, format!("{}: {}", message, context))
}

#[cfg(not(feature = "alloc"))]
pub fn runtime_error_with_context(message: &str, _context: &str) -> Error {
    Error::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, message)
}

/// Wrapper for custom sections with additional functionality
// This struct uses String and Vec<u8>, so it requires 'alloc'.
// If decode_module needs to work without 'alloc', this needs to be refactored
// or this wrapper is only used when 'alloc' is available.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub struct CustomSectionWrapper {
    /// Name of the custom section
    pub name: String,
    /// Data of the custom section
    pub data: Vec<u8>,
}

/// Internal parsing logic that consumes a `crate::parser::Parser`.
/// Renamed from `parse_module` to avoid conflict with the public one if struct
/// Module is removed.
// TODO: This function will eventually need to take the SafeMemoryHandler<P>
// and use it to populate BoundedVec fields of WrtModule.
// For now, it still uses Vec internally, so it implicitly requires 'alloc'.
fn parse_module_internal_logic(
    mut parser: crate::parser::Parser<'_>,
) -> Result<(WrtModule, Vec<u8>)> {
    // Initialize collections based on feature flags
    #[cfg(feature = "alloc")]
    let mut mod_types = Vec::new();
    #[cfg(not(feature = "alloc"))]
    let mut mod_types = TypesVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate module types"))?;

    #[cfg(feature = "alloc")]
    let mut mod_imports = Vec::new();
    #[cfg(not(feature = "alloc"))]
    let mut mod_imports = ImportsVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate module imports"))?;

    #[cfg(feature = "alloc")]
    let mut mod_funcs = Vec::new(); // Type indices for functions
    #[cfg(not(feature = "alloc"))]
    let mut mod_funcs = FunctionsVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate module functions"))?;

    #[cfg(feature = "alloc")]
    let mut mod_tables = Vec::new();
    #[cfg(not(feature = "alloc"))]
    let mut mod_tables = TablesVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate module tables"))?;

    #[cfg(feature = "alloc")]
    let mut mod_memories = Vec::new();
    #[cfg(not(feature = "alloc"))]
    let mut mod_memories = MemoriesVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate module memories"))?;

    #[cfg(feature = "alloc")]
    let mut mod_globals = Vec::new();
    #[cfg(not(feature = "alloc"))]
    let mut mod_globals = GlobalsVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate module globals"))?;

    #[cfg(feature = "alloc")]
    let mut mod_exports = Vec::new();
    #[cfg(not(feature = "alloc"))]
    let mut mod_exports = ExportsVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate module exports"))?;

    let mut mod_start = None;

    #[cfg(feature = "alloc")]
    let mut mod_elements = Vec::new();
    #[cfg(not(feature = "alloc"))]
    let mut mod_elements = ElementsVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate module elements"))?;

    #[cfg(feature = "alloc")]
    let mut mod_code_entries = Vec::new(); // Will hold WrtCode
    #[cfg(not(feature = "alloc"))]
    let mut mod_code_entries = FunctionsVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate module code entries"))?;

    #[cfg(feature = "alloc")]
    let mut mod_data_segments = Vec::new();
    #[cfg(not(feature = "alloc"))]
    let mut mod_data_segments = DataVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate module data segments"))?;

    let mut mod_data_count = None;

    #[cfg(feature = "alloc")]
    let mut mod_custom_sections = Vec::new();
    #[cfg(not(feature = "alloc"))]
    let mut mod_custom_sections = CustomSectionsVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate module custom sections"))?;

    #[cfg(feature = "alloc")]
    let mut remaining_bytes = Vec::new();
    #[cfg(not(feature = "alloc"))]
    let mut remaining_bytes = ByteVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate remaining bytes buffer"))?;

    loop {
        match parser.read() {
            Ok(Some(payload)) => {
                match payload {
                    crate::parser::Payload::Version(_version, _bytes) => {}
                    crate::parser::Payload::TypeSection(slice, _size) => {
                        if let Ok(data) = slice.data() {
                            mod_types = crate::sections::parsers::parse_type_section(data)?;
                        } else {
                            return Err(Error::new(
                                ErrorCategory::Parse,
                                codes::DECODE_ERROR,
                                "Failed to get data from TypeSection SafeSlice",
                            ));
                        }
                    }
                    crate::parser::Payload::ImportSection(slice, _size) => {
                        if let Ok(data) = slice.data() {
                            mod_imports = crate::sections::parsers::parse_import_section(data)?;
                        } else {
                            return Err(Error::new(
                                ErrorCategory::Parse,
                                codes::DECODE_ERROR,
                                "Failed to get data from ImportSection SafeSlice",
                            ));
                        }
                    }
                    crate::parser::Payload::FunctionSection(slice, _size) => {
                        if let Ok(data) = slice.data() {
                            // This parser returns Vec<u32> directly which is correct for mod_funcs
                            mod_funcs = crate::sections::parsers::parse_function_section(data)?;
                        } else {
                            return Err(Error::new(
                                ErrorCategory::Parse,
                                codes::DECODE_ERROR,
                                "Failed to get data from FunctionSection SafeSlice",
                            ));
                        }
                    }
                    crate::parser::Payload::TableSection(slice, _size) => {
                        if let Ok(data) = slice.data() {
                            mod_tables = crate::sections::parsers::parse_table_section(data)?;
                        } else {
                            return Err(Error::new(
                                ErrorCategory::Parse,
                                codes::DECODE_ERROR,
                                "Failed to get data from TableSection SafeSlice",
                            ));
                        }
                    }
                    crate::parser::Payload::MemorySection(slice, _size) => {
                        if let Ok(data) = slice.data() {
                            mod_memories = crate::sections::parsers::parse_memory_section(data)?;
                        } else {
                            return Err(Error::new(
                                ErrorCategory::Parse,
                                codes::DECODE_ERROR,
                                "Failed to get data from MemorySection SafeSlice",
                            ));
                        }
                    }
                    crate::parser::Payload::GlobalSection(slice, _size) => {
                        if let Ok(data) = slice.data() {
                            mod_globals = crate::sections::parsers::parse_global_section(data)?;
                        } else {
                            return Err(Error::new(
                                ErrorCategory::Parse,
                                codes::DECODE_ERROR,
                                "Failed to get data from GlobalSection SafeSlice",
                            ));
                        }
                    }
                    crate::parser::Payload::ExportSection(slice, _size) => {
                        if let Ok(data) = slice.data() {
                            mod_exports = crate::sections::parsers::parse_export_section(data)?;
                        } else {
                            return Err(Error::new(
                                ErrorCategory::Parse,
                                codes::DECODE_ERROR,
                                "Failed to get data from ExportSection SafeSlice",
                            ));
                        }
                    }
                    crate::parser::Payload::StartSection(func_idx) => {
                        mod_start = Some(func_idx);
                    }
                    crate::parser::Payload::ElementSection(slice, _size) => {
                        if let Ok(data) = slice.data() {
                            mod_elements = crate::sections::parsers::parse_element_section(data)?;
                        } else {
                            return Err(Error::new(
                                ErrorCategory::Parse,
                                codes::DECODE_ERROR,
                                "Failed to get data from ElementSection SafeSlice",
                            ));
                        }
                    }
                    crate::parser::Payload::CodeSection(slice, _size) => {
                        if let Ok(mut code_section_data) = slice.data() {
                            let (num_functions, mut bytes_read_for_count) =
                                wrt_format::binary::read_leb_u32(code_section_data)?;
                            code_section_data = &code_section_data[bytes_read_for_count..];

                            if num_functions as usize != mod_funcs.len() {
                                return Err(Error::new(
                                    ErrorCategory::Validation,
                                    codes::VALIDATION_ERROR,
                                    format!(
                                        "Code section function count {} mismatches function \
                                         section count {}",
                                        num_functions,
                                        mod_funcs.len()
                                    ),
                                ));
                            }

                            for _ in 0..num_functions {
                                let (func_size, size_len) =
                                    wrt_format::binary::read_leb_u32(code_section_data)?;
                                code_section_data = &code_section_data[size_len..];
                                // bytes_read_for_count += size_len; // This counter is not total
                                // for section, but per-func

                                if code_section_data.len() < func_size as usize {
                                    return Err(Error::new(
                                        ErrorCategory::Parse,
                                        codes::DECODE_UNEXPECTED_EOF,
                                        "EOF in code section entry",
                                    ));
                                }
                                let mut func_data_slice = &code_section_data[..func_size as usize];

                                let (locals, locals_len) =
                                    instructions::parse_locals(func_data_slice)?;
                                func_data_slice = &func_data_slice[locals_len..];

                                let (instructions_vec, _instr_len) =
                                    instructions::parse_instructions(func_data_slice)?;

                                let expr = WrtExpr { instructions: instructions_vec };
                                mod_code_entries.push(WrtCode { locals, body: expr });

                                code_section_data = &code_section_data[func_size as usize..];
                                // bytes_read_for_count += func_size as usize;
                                // // This counter is not total for section, but
                                // per-func
                            }
                        } else {
                            return Err(Error::new(
                                ErrorCategory::Parse,
                                codes::DECODE_ERROR,
                                "Failed to get data from CodeSection SafeSlice",
                            ));
                        }
                    }
                    crate::parser::Payload::DataSection(slice, _size) => {
                        if let Ok(data) = slice.data() {
                            mod_data_segments = crate::sections::parsers::parse_data_section(data)?;
                        } else {
                            return Err(Error::new(
                                ErrorCategory::Parse,
                                codes::DECODE_ERROR,
                                "Failed to get data from DataSection SafeSlice",
                            ));
                        }
                    }
                    crate::parser::Payload::DataCountSection { count } => {
                        mod_data_count = Some(count);
                    }
                    crate::parser::Payload::CustomSection { name, data, size: _ } => {
                        if let Ok(data_bytes) = data.data() {
                            // TODO: When debug section support is added to WrtModule,
                            // check if name starts with ".debug_" and handle specially
                            // For now, store all custom sections as-is
                            mod_custom_sections
                                .push(WrtCustomSection { name, data: data_bytes.to_vec() });
                        } else {
                            return Err(Error::new(
                                ErrorCategory::Parse,
                                codes::DECODE_ERROR,
                                "Failed to get data from CustomSection SafeSlice",
                            ));
                        }
                    }
                    crate::parser::Payload::ComponentSection { .. } => {
                        return Err(Error::new(
                            ErrorCategory::Parse,
                            codes::UNSUPPORTED_FEATURE,
                            "Component sections not supported in core module parsing",
                        ));
                    }
                    crate::parser::Payload::End => {
                        break;
                    }
                }
            }
            Ok(None) => {
                break;
            }
            Err(e) => {
                return Err(
                    e.add_context(codes::DECODE_ERROR, "Failed to read payload from parser")
                );
            }
        }
    }
    // Placeholder for WrtModule construction
    // This assumes WrtModule has a constructor or fields that can be populated from
    // these Vecs. This will likely fail compilation or be incorrect until
    // WrtModule's alloc-free structure is finalized in wrt-foundation and used
    // here.

    // TODO: Replace this placeholder construction with actual BoundedVec population
    // using the 'handler' passed into decode_module -> parse_module_internal_logic.
    // The WrtModule instance should be created using the handler.
    // The following is a temporary measure assuming WrtModule can be created from
    // Vecs, which might not be true if it's already using BoundedVecs.

    let result_module = WrtModule {
        // These fields need to be populated from mod_... Vecs into BoundedVecs
        // This is a conceptual mapping, actual field names/types in WrtModule might differ.
        types: mod_types,   // TODO: Convert to BoundedVec<FuncType, MAX_TYPES, P>
        funcs: mod_funcs,   // TODO: Convert to BoundedVec<TypeIdx, MAX_FUNCS, P>
        tables: mod_tables, // TODO: Convert to BoundedVec<TableType, MAX_TABLES, P>
        memories: mod_memories, // TODO: Convert to BoundedVec<MemoryType, MAX_MEMORIES, P>
        globals: mod_globals, /* TODO: Convert to BoundedVec<Global<P>, MAX_GLOBALS, P> (if Global
                             * is generic) */
        exports: mod_exports, // TODO: Convert to BoundedVec<Export<P>, MAX_EXPORTS, P>
        imports: mod_imports, // TODO: Convert to BoundedVec<Import<P>, MAX_IMPORTS, P>
        elements: mod_elements, /* TODO: Convert to BoundedVec<ElementSegment<P>,
                               * MAX_ELEMENT_SEGMENTS, P> */
        code: mod_code_entries, // TODO: Convert to BoundedVec<Code<P>, MAX_FUNCS, P>
        data: mod_data_segments, /* TODO: Convert to BoundedVec<DataSegment<P>,
                                 * MAX_DATA_SEGMENTS, P> */
        start: mod_start,
        custom_sections: mod_custom_sections, /* TODO: Convert to BoundedVec<CustomSection<P>,
                                               * MAX_CUSTOM_SECTIONS, P> */
        data_count: mod_data_count,
        // Assuming other fields like name, version, etc., are handled or not present in WrtModule
        // from types Add _marker: PhantomData<P> if P is needed by WrtModule itself
    };

    Ok((result_module, remaining_bytes))
}

/// Helper function to write a string to a binary vector
fn write_string(result: &mut Vec<u8>, s: &str) -> Result<()> {
    let bytes = s.as_bytes();
    let len = bytes.len();

    // Write the length as LEB128
    let len_bytes = wrt_format::binary::write_leb128_u32(len as u32);
    result.extend_from_slice(&len_bytes);

    // Write the string
    result.extend_from_slice(bytes);

    Ok(())
}

#[cfg(test)]
#[cfg(feature = "alloc")] // Tests might rely on Vec or String, gate them too
mod tests {
    use wrt_foundation::safe_memory::NoStdProvider; // For tests
    use wrt_foundation::safe_memory::SafeMemoryHandler;

    use super::*; // For tests

    #[test]
    fn test_decode_module_valid_header() {
        let bytes = vec![
            // ... existing code ...
        ];
        // Create a dummy handler for the test
        let mut memory_backing = [0u8; 1024]; // Example backing store for NoStdProvider
        let provider = NoStdProvider::new(&mut memory_backing);
        let mut handler = SafeMemoryHandler::new(provider);
        // Pass the handler to decode_module
        let result = decode_module(&bytes, &mut handler);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decode_module_invalid_magic() {
        let bytes = vec![
            // ... existing code ...
        ];
        // Create a dummy handler for the test
        let mut memory_backing = [0u8; 1024];
        let provider = NoStdProvider::new(&mut memory_backing);
        let mut handler = SafeMemoryHandler::new(provider);
        // Pass the handler to decode_module
        let result = decode_module(&bytes, &mut handler);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_module_minimal_valid() {
        // A minimal valid WebAssembly module (just magic and version)
        let bytes = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        let mut memory_backing = [0u8; 1024];
        let provider = NoStdProvider::new(&mut memory_backing);
        let mut handler = SafeMemoryHandler::new(provider);
        let result = decode_module(&bytes, &mut handler);
        assert!(result.is_ok());
        let module = result.unwrap();
        // Basic checks, assuming WrtModule can be default-like for empty sections
        assert!(module.types.is_empty());
        assert!(module.funcs.is_empty());
    }

    #[test]
    fn test_encode_decode_custom_section() {
        // This test inherently requires alloc for WrtCustomSection string and
        // encode_module Vec
        let original_section = WrtCustomSection {
            name: "test_section".to_string(), // Requires alloc
            data: vec![1, 2, 3, 4, 5],        // Requires alloc
        };

        let mut module = WrtModule {
            // Assuming WrtModule can be constructed like this for testing
            types: Vec::new(),
            funcs: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            exports: Vec::new(),
            imports: Vec::new(),
            elements: Vec::new(),
            code: Vec::new(),
            data: Vec::new(),
            start: None,
            custom_sections: vec![original_section.clone()], // Requires alloc
            data_count: None,
        };

        let encoded_bytes = encode_module(&module).expect("Encoding failed");

        let mut memory_backing = [0u8; 1024];
        let provider = NoStdProvider::new(&mut memory_backing);
        let mut handler = SafeMemoryHandler::new(provider);
        let decoded_module = decode_module(&encoded_bytes, &mut handler).expect("Decoding failed");

        assert_eq!(decoded_module.custom_sections.len(), 1);
        assert_eq!(decoded_module.custom_sections[0].name, original_section.name);
        assert_eq!(decoded_module.custom_sections[0].data, original_section.data);
    }

    // TODO: Add more tests for different sections once WrtModule structure is
    // alloc-free and can be populated correctly.
}
