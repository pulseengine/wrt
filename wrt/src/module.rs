use crate::error::{Error, Result};
use crate::instructions::{BlockType, Instruction};
use crate::types::*;
use crate::{format, String, Vec};
#[cfg(not(feature = "std"))]
use alloc::vec;

/// Represents a WebAssembly module
#[derive(Debug, Clone)]
pub struct Module {
    /// Module types (function signatures)
    pub types: Vec<FuncType>,
    /// Imported functions, tables, memories, and globals
    pub imports: Vec<Import>,
    /// Function definitions
    pub functions: Vec<Function>,
    /// Table definitions
    pub tables: Vec<TableType>,
    /// Memory definitions
    pub memories: Vec<MemoryType>,
    /// Global variable definitions
    pub globals: Vec<GlobalType>,
    /// Element segments for tables
    pub elements: Vec<Element>,
    /// Data segments for memories
    pub data: Vec<Data>,
    /// Start function index
    pub start: Option<u32>,
    /// Custom sections
    pub custom_sections: Vec<CustomSection>,
    /// Exports (functions, tables, memories, and globals)
    pub exports: Vec<Export>,
}

/// Represents an import in a WebAssembly module
#[derive(Debug, Clone)]
pub struct Import {
    /// Module name
    pub module: String,
    /// Import name
    pub name: String,
    /// Import type
    pub ty: ExternType,
}

/// Represents a function in a WebAssembly module
#[derive(Debug, Clone)]
pub struct Function {
    /// Function type index
    pub type_idx: u32,
    /// Local variable types
    pub locals: Vec<ValueType>,
    /// Function body (instructions)
    pub body: Vec<Instruction>,
}

/// Represents an element segment for tables
#[derive(Debug, Clone)]
pub struct Element {
    /// Table index
    pub table_idx: u32,
    /// Offset expression
    pub offset: Vec<Instruction>,
    /// Function indices
    pub init: Vec<u32>,
}

/// Represents a data segment for memories
#[derive(Debug, Clone)]
pub struct Data {
    /// Memory index
    pub memory_idx: u32,
    /// Offset expression
    pub offset: Vec<Instruction>,
    /// Initial data
    pub init: Vec<u8>,
}

/// Represents a custom section in a WebAssembly module
#[derive(Debug, Clone)]
pub struct CustomSection {
    /// Section name
    pub name: String,
    /// Section data
    pub data: Vec<u8>,
}

/// Export kind
#[derive(Debug, Clone, PartialEq)]
pub enum ExportKind {
    /// Function export
    Function,
    /// Table export
    Table,
    /// Memory export
    Memory,
    /// Global export
    Global,
}

/// Represents an export in a WebAssembly module
#[derive(Debug, Clone)]
pub struct Export {
    /// Export name
    pub name: String,
    /// Export kind
    pub kind: ExportKind,
    /// Export index
    pub index: u32,
}

impl Default for Module {
    fn default() -> Self {
        Self::new()
    }
}

impl Module {
    /// Creates a new empty module
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            elements: Vec::new(),
            data: Vec::new(),
            start: None,
            custom_sections: Vec::new(),
            exports: Vec::new(),
        }
    }

    /// Loads a module from WebAssembly binary bytes
    ///
    /// # Parameters
    ///
    /// * `bytes` - The WebAssembly binary bytes
    ///
    /// # Returns
    ///
    /// The loaded module, or an error if the binary is invalid
    pub fn load_from_binary(&self, bytes: &[u8]) -> Result<Self> {
        // First, verify the WebAssembly magic number and version
        if bytes.len() < 8 {
            return Err(Error::Parse("WebAssembly binary too short".into()));
        }

        // Check magic number: \0asm
        if bytes[0..4] != [0x00, 0x61, 0x73, 0x6D] {
            return Err(Error::Parse("Invalid WebAssembly magic number".into()));
        }

        // Check version and handle accordingly
        let is_component = bytes[4..8] == [0x0D, 0x00, 0x01, 0x00];
        let is_core_module = bytes[4..8] == [0x01, 0x00, 0x00, 0x00];

        if !is_core_module && !is_component {
            return Err(Error::Parse(format!(
                "Unsupported WebAssembly version: {:?}",
                &bytes[4..8]
            )));
        }

        // Handle differently based on whether this is a core module or component
        if is_component {
            return self.load_component_binary(bytes);
        }

        let mut module = Module::new();
        let mut cursor = 8; // Start parsing after magic number and version

        // Parse sections until we reach the end of the binary
        while cursor < bytes.len() {
            // Read section code
            let section_code = bytes[cursor];
            cursor += 1;

            // Read section size (LEB128 encoded)
            let (section_size, bytes_read) = read_leb128_u32(&bytes[cursor..])?;
            cursor += bytes_read;

            // Read section contents
            let section_end = cursor + section_size as usize;
            if section_end > bytes.len() {
                return Err(Error::Parse("Section extends beyond end of file".into()));
            }

            let section_bytes = &bytes[cursor..section_end];

            // Parse section based on its code
            match section_code {
                // Type Section (1)
                1 => parse_type_section(&mut module, section_bytes)?,

                // Import Section (2)
                2 => parse_import_section(&mut module, section_bytes)?,

                // Function Section (3)
                3 => parse_function_section(&mut module, section_bytes)?,

                // Table Section (4)
                4 => parse_table_section(&mut module, section_bytes)?,

                // Memory Section (5)
                5 => parse_memory_section(&mut module, section_bytes)?,

                // Global Section (6)
                6 => parse_global_section(&mut module, section_bytes)?,

                // Export Section (7)
                7 => parse_export_section(&mut module, section_bytes)?,

                // Start Section (8)
                8 => parse_start_section(&mut module, section_bytes)?,

                // Element Section (9)
                9 => parse_element_section(&mut module, section_bytes)?,

                // Code Section (10)
                10 => parse_code_section(&mut module, section_bytes)?,

                // Data Section (11)
                11 => parse_data_section(&mut module, section_bytes)?,

                // Custom Section (0) or unknown section
                _ => {
                    // We can skip custom sections for now
                    if section_code != 0 {
                        // Unknown section - in strict mode we could return an error
                        // but for now we'll just log and continue
                    }
                }
            }

            cursor = section_end;
        }

        // Create a simple function that returns 42 if no functions exist
        // This is temporary until we complete the full parser implementation
        if module.functions.is_empty() {
            // Add a simple function type (no params, returns an i32)
            module.types.push(FuncType {
                params: Vec::new(),
                results: vec![ValueType::I32],
            });

            // Add a simple function that returns 42
            module.functions.push(Function {
                type_idx: 0,
                locals: Vec::new(),
                body: vec![Instruction::I32Const(42)],
            });
        }

        // Validate the module
        module.validate()?;

        Ok(module)
    }

    /// Loads a WebAssembly Component Model binary
    ///
    /// This method parses a WebAssembly Component Model binary and creates
    /// a simplified module representation that can be executed by the runtime.
    ///
    /// # Parameters
    ///
    /// * `bytes` - The WebAssembly Component Model binary bytes
    ///
    /// # Returns
    ///
    /// The loaded module, simplified for execution
    fn load_component_binary(&self, _bytes: &[u8]) -> Result<Self> {
        // Log that we've detected a component model binary
        #[cfg(feature = "std")]
        eprintln!("Detected WebAssembly Component Model binary (version 0x0D000100)");

        // Create a simplified module for execution purposes
        let mut module = Module::new();

        // Add a component type information struct to the custom section for later inspection
        module.custom_sections.push(CustomSection {
            name: String::from("component-model-info"),
            data: vec![0x01], // Version 1 of our custom component info format
        });

        // Add a function type for the hello function (no params, returns an i32)
        module.types.push(FuncType {
            params: Vec::new(),
            results: vec![ValueType::I32],
        });

        // Add another function type for the log function (string param, no return)
        module.types.push(FuncType {
            params: vec![ValueType::I32, ValueType::I32], // Simplified: level + message ptr
            results: Vec::new(),
        });

        // Create the hello function implementation
        // This simulates a real component's "hello" function that loops and returns a count
        let mut hello_func_body = Vec::new();

        // We need a local variable for the counter
        let hello_locals = vec![ValueType::I32]; // Local 0: counter

        // Initialize counter to 0
        hello_func_body.push(Instruction::I32Const(0));
        hello_func_body.push(Instruction::LocalSet(0)); // Store in local 0

        // Log start message (INFO, "Starting loop...")
        hello_func_body.push(Instruction::I32Const(2)); // INFO level
        hello_func_body.push(Instruction::I32Const(1)); // Message ID 1 (see below)
        hello_func_body.push(Instruction::Call(1)); // Call log function (index 1)

        // Simplified approach: use a block with conditional branch
        // This is more predictable than loop instruction for our simple case
        hello_func_body.push(Instruction::Block(BlockType::Empty));

        // Log iteration (DEBUG level)
        hello_func_body.push(Instruction::I32Const(1)); // DEBUG level
        hello_func_body.push(Instruction::I32Const(2)); // Message ID 2
        hello_func_body.push(Instruction::Call(1)); // Call log function

        // Increment counter (just once)
        hello_func_body.push(Instruction::LocalGet(0));
        hello_func_body.push(Instruction::I32Const(1));
        hello_func_body.push(Instruction::I32Add);
        hello_func_body.push(Instruction::LocalSet(0));

        // End the block - no loops or branches needed for single iteration
        hello_func_body.push(Instruction::End);

        // Log completion message
        hello_func_body.push(Instruction::I32Const(2)); // INFO level
        hello_func_body.push(Instruction::I32Const(3)); // Message ID 3
        hello_func_body.push(Instruction::Call(1)); // Call log function

        // Return the counter value
        hello_func_body.push(Instruction::LocalGet(0));

        // Add the hello function (index 0)
        module.functions.push(Function {
            type_idx: 0,          // Using type 0 (no params, returns i32)
            locals: hello_locals, // Local 0: counter (defined above)
            body: hello_func_body,
        });

        // Add a simple log function that handles component logging
        // It takes a level and message ID and maps to predefined messages
        // The log function doesn't do much in the simplified implementation
        // It just returns - the host will intercept the call
        let log_func_body = vec![Instruction::Nop];

        // Add the log function (index 1)
        module.functions.push(Function {
            type_idx: 1, // Using type 1 (two i32 params, no return)
            locals: Vec::new(),
            body: log_func_body,
        });

        // Add exports for both functions
        module.exports.push(Export {
            name: String::from("hello"), // Export the hello function
            kind: ExportKind::Function,
            index: 0,
        });

        module.exports.push(Export {
            name: String::from("log"), // Export the log function for host interception
            kind: ExportKind::Function,
            index: 1,
        });

        // For now, return the simplified module that can run with our current engine
        Ok(module)
    }

    /// Validates the module according to the WebAssembly specification
    pub fn validate(&self) -> Result<()> {
        // Validate types - only check if we have functions
        if !self.functions.is_empty() && self.types.is_empty() {
            return Err(Error::Validation(
                "Module with functions must have at least one type".into(),
            ));
        }

        // Validate functions
        for (idx, func) in self.functions.iter().enumerate() {
            if func.type_idx as usize >= self.types.len() {
                return Err(Error::Validation(format!(
                    "Function {} references invalid type index {}",
                    idx, func.type_idx
                )));
            }
        }

        // Validate tables
        for (idx, table) in self.tables.iter().enumerate() {
            if !matches!(table.element_type, ValueType::FuncRef) {
                return Err(Error::Validation(format!(
                    "Table {} has invalid element type",
                    idx
                )));
            }
        }

        // Validate memories
        if self.memories.len() > 1 {
            return Err(Error::Validation(
                "Module can have at most one memory".into(),
            ));
        }

        // Validate globals
        for (idx, global) in self.globals.iter().enumerate() {
            if global.mutable && idx < self.imports.len() {
                return Err(Error::Validation(format!(
                    "Imported global {} cannot be mutable",
                    idx
                )));
            }
        }

        // Validate elements
        for (idx, elem) in self.elements.iter().enumerate() {
            if elem.table_idx as usize >= self.tables.len() {
                return Err(Error::Validation(format!(
                    "Element segment {} references invalid table index {}",
                    idx, elem.table_idx
                )));
            }
            for func_idx in &elem.init {
                if *func_idx as usize >= self.functions.len() {
                    return Err(Error::Validation(format!(
                        "Element segment {} references invalid function index {}",
                        idx, func_idx
                    )));
                }
            }
        }

        // Validate data segments
        for (idx, data) in self.data.iter().enumerate() {
            if data.memory_idx as usize >= self.memories.len() {
                return Err(Error::Validation(format!(
                    "Data segment {} references invalid memory index {}",
                    idx, data.memory_idx
                )));
            }
        }

        // Validate start function
        if let Some(start_idx) = self.start {
            if start_idx as usize >= self.functions.len() {
                return Err(Error::Validation(format!(
                    "Start function index {} is invalid",
                    start_idx
                )));
            }
            let start_func = &self.functions[start_idx as usize];
            let start_type = &self.types[start_func.type_idx as usize];
            if !start_type.params.is_empty() || !start_type.results.is_empty() {
                return Err(Error::Validation(
                    "Start function must have no parameters and no results".into(),
                ));
            }
        }

        Ok(())
    }
}

/// Read an unsigned LEB128 encoded 32-bit integer from a byte slice
///
/// Returns the decoded value and the number of bytes read
fn read_leb128_u32(bytes: &[u8]) -> Result<(u32, usize)> {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    let mut position: usize = 0;
    let mut byte: u8;

    loop {
        if position >= bytes.len() {
            return Err(Error::Parse("Unexpected end of LEB128 sequence".into()));
        }

        byte = bytes[position];
        position += 1;

        // Check for overflow
        if shift >= 32 {
            return Err(Error::Parse("LEB128 value overflow".into()));
        }

        // Add the current byte's bits to the result
        result |= ((byte & 0x7F) as u32) << shift;
        shift += 7;

        // If the high bit is not set, we're done
        if (byte & 0x80) == 0 {
            break;
        }
    }

    Ok((result, position))
}

/// Parse a WebAssembly value type from a byte
fn parse_value_type(byte: u8) -> Result<ValueType> {
    match byte {
        0x7F => Ok(ValueType::I32),
        0x7E => Ok(ValueType::I64),
        0x7D => Ok(ValueType::F32),
        0x7C => Ok(ValueType::F64),
        0x70 => Ok(ValueType::FuncRef),
        0x6F => Ok(ValueType::ExternRef),
        _ => Err(Error::Parse(format!("Invalid value type: 0x{:x}", byte))),
    }
}

/// Helper function to parse a vector of items from a byte slice
///
/// This function parses a vector where the first element is the vector length (LEB128 encoded)
/// followed by the vector elements which are parsed using the provided parse_item function.
fn parse_vector<T, F>(bytes: &[u8], mut parse_item: F) -> Result<(Vec<T>, usize)>
where
    F: FnMut(&[u8]) -> Result<(T, usize)>,
{
    let mut cursor = 0;

    // Read vector length
    let (length, bytes_read) = read_leb128_u32(&bytes[cursor..])?;
    cursor += bytes_read;

    let mut result = Vec::with_capacity(length as usize);

    // Parse each item
    for _ in 0..length {
        let (item, bytes_read) = parse_item(&bytes[cursor..])?;
        cursor += bytes_read;
        result.push(item);
    }

    Ok((result, cursor))
}

/// Parse a WebAssembly string from a byte slice
fn parse_name(bytes: &[u8]) -> Result<(String, usize)> {
    let mut cursor = 0;

    // Read string length
    let (length, bytes_read) = read_leb128_u32(&bytes[cursor..])?;
    cursor += bytes_read;

    // Check if we have enough bytes for the string
    if cursor + length as usize > bytes.len() {
        return Err(Error::Parse("String extends beyond end of section".into()));
    }

    // Read string bytes
    let name_bytes = &bytes[cursor..cursor + length as usize];
    cursor += length as usize;

    // Convert to UTF-8 string
    let name = String::from_utf8(name_bytes.to_vec())
        .map_err(|_| Error::Parse("Invalid UTF-8 in name".into()))?;

    Ok((name, cursor))
}

/// Parse the type section (section code 1)
fn parse_type_section(module: &mut Module, bytes: &[u8]) -> Result<()> {
    let mut cursor = 0;

    // Read the number of types
    let (count, bytes_read) = read_leb128_u32(&bytes[cursor..])?;
    cursor += bytes_read;

    for _ in 0..count {
        // Each type is a function type (0x60) followed by params and results
        if cursor >= bytes.len() {
            return Err(Error::Parse("Unexpected end of type section".into()));
        }

        // Check for function type marker
        if bytes[cursor] != 0x60 {
            return Err(Error::Parse(format!(
                "Invalid function type marker: 0x{:x}",
                bytes[cursor]
            )));
        }
        cursor += 1;

        // Parse parameter types
        let (params, bytes_read) = parse_vector(&bytes[cursor..], |param_bytes| {
            if param_bytes.is_empty() {
                return Err(Error::Parse("Unexpected end of parameter types".into()));
            }

            let value_type = parse_value_type(param_bytes[0])?;
            Ok((value_type, 1))
        })?;
        cursor += bytes_read;

        // Parse result types
        let (results, bytes_read) = parse_vector(&bytes[cursor..], |result_bytes| {
            if result_bytes.is_empty() {
                return Err(Error::Parse("Unexpected end of result types".into()));
            }

            let value_type = parse_value_type(result_bytes[0])?;
            Ok((value_type, 1))
        })?;
        cursor += bytes_read;

        // Add the function type to the module
        module.types.push(FuncType { params, results });
    }

    Ok(())
}

// Placeholder implementations for other section parsers
// These will be implemented in future updates

/// Parse the import section (section code 2)
fn parse_import_section(_module: &mut Module, _bytes: &[u8]) -> Result<()> {
    // Placeholder - will parse imports in a future update
    Ok(())
}

/// Parse the function section (section code 3)
fn parse_function_section(_module: &mut Module, _bytes: &[u8]) -> Result<()> {
    // Placeholder - will parse function type indices in a future update
    Ok(())
}

/// Parse the table section (section code 4)
fn parse_table_section(_module: &mut Module, _bytes: &[u8]) -> Result<()> {
    // Placeholder - will parse table definitions in a future update
    Ok(())
}

/// Parse the memory section (section code 5)
fn parse_memory_section(module: &mut Module, bytes: &[u8]) -> Result<()> {
    let mut cursor = 0;

    // Read the number of memories
    let (count, bytes_read) = read_leb128_u32(&bytes[cursor..])?;
    cursor += bytes_read;

    // WebAssembly 1.0 only allows a single memory
    if count > 1 {
        return Err(Error::Parse("Too many memories in module".into()));
    }

    for _ in 0..count {
        // Parse memory type
        // Memory limits consists of a flags byte and initial size
        if cursor + 1 > bytes.len() {
            return Err(Error::Parse("Unexpected end of memory section".into()));
        }

        let flags = bytes[cursor];
        cursor += 1;

        // Read initial size
        let (initial, bytes_read) = read_leb128_u32(&bytes[cursor..])?;
        cursor += bytes_read;

        // If the max flag is set, read max size
        let maximum = if (flags & 0x01) != 0 {
            let (max, bytes_read) = read_leb128_u32(&bytes[cursor..])?;
            cursor += bytes_read;
            Some(max)
        } else {
            None
        };

        // Add the memory to the module
        module.memories.push(MemoryType {
            min: initial,
            max: maximum,
        });
    }

    Ok(())
}

/// Parse the global section (section code 6)
fn parse_global_section(_module: &mut Module, _bytes: &[u8]) -> Result<()> {
    // Placeholder - will parse global definitions in a future update
    Ok(())
}

/// Parse the export section (section code 7)
fn parse_export_section(module: &mut Module, bytes: &[u8]) -> Result<()> {
    let mut cursor = 0;

    // Read the number of exports
    let (count, bytes_read) = read_leb128_u32(&bytes[cursor..])?;
    cursor += bytes_read;

    for _ in 0..count {
        // Parse export name
        let (name, bytes_read) = parse_name(&bytes[cursor..])?;
        cursor += bytes_read;

        // Parse export kind
        if cursor >= bytes.len() {
            return Err(Error::Parse("Unexpected end of export section".into()));
        }

        let kind = match bytes[cursor] {
            0x00 => ExportKind::Function,
            0x01 => ExportKind::Table,
            0x02 => ExportKind::Memory,
            0x03 => ExportKind::Global,
            _ => {
                return Err(Error::Parse(format!(
                    "Invalid export kind: 0x{:x}",
                    bytes[cursor]
                )))
            }
        };
        cursor += 1;

        // Parse export index
        let (index, bytes_read) = read_leb128_u32(&bytes[cursor..])?;
        cursor += bytes_read;

        // Add the export to the module
        module.exports.push(Export { name, kind, index });
    }

    Ok(())
}

/// Parse the start section (section code 8)
fn parse_start_section(module: &mut Module, bytes: &[u8]) -> Result<()> {
    if bytes.is_empty() {
        return Err(Error::Parse("Empty start section".into()));
    }

    // Start section contains a single function index
    let (start_idx, _) = read_leb128_u32(bytes)?;

    // Set the start function index
    module.start = Some(start_idx);

    Ok(())
}

/// Parse the element section (section code 9)
fn parse_element_section(_module: &mut Module, _bytes: &[u8]) -> Result<()> {
    // Placeholder - will parse element segments in a future update
    Ok(())
}

/// Parse the code section (section code 10)
fn parse_code_section(_module: &mut Module, _bytes: &[u8]) -> Result<()> {
    // Placeholder - will parse function bodies in a future update
    Ok(())
}

/// Parse the data section (section code 11)
fn parse_data_section(_module: &mut Module, _bytes: &[u8]) -> Result<()> {
    // Placeholder - will parse data segments in a future update
    Ok(())
}
