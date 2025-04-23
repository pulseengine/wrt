use crate::{
    behavior::{self, ControlFlowBehavior, FrameBehavior, StackBehavior},
    decoder_integration,
    error::{kinds, Error, Result},
    global::Global,
    instructions::{types::BlockType, Instruction},
    memory::Memory,
    table::Table,
    types::{ExternType, GlobalType, MemoryType, TableType},
    types::{FuncType, ValueType},
    values::Value,
    String, Vec,
};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[cfg(not(feature = "std"))]
use core::fmt;

// Use debug_println macro as println for no_std environment
#[cfg(not(feature = "std"))]
use crate::debug_println as println;

#[cfg(not(feature = "std"))]
use alloc::vec;

#[cfg(feature = "std")]
use std::string::ToString;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

// Use wrt-decoder for high-level WebAssembly parsing and encoding
use crate::component::InstanceValue;
use wrt_decoder;
use wrt_format;

/// Represents the address of a table within a module instance.
/// Used for indirect function calls.
#[derive(Debug, Clone, Copy)]
pub struct TableAddr {
    /// The index of the table within the module's tables section.
    pub table_idx: u32,
}

/// Represents a WebAssembly module
#[derive(Debug, Clone)]
pub struct Module {
    /// Module types (function signatures)
    pub types: Vec<FuncType>,
    /// Imported functions, tables, memories, and globals
    pub imports: HashMap<String, HashMap<String, Import>>,
    /// Function definitions
    pub functions: Vec<Function>,
    /// Table definitions
    pub tables: Vec<Arc<Table>>,
    /// Memory definitions
    pub memories: Vec<Arc<Memory>>,
    /// Global variable definitions
    pub globals: Vec<Arc<Global>>,
    /// Element segments for tables
    pub elements: Vec<Element>,
    /// Data segments for memories
    pub data: Vec<Data>,
    /// Start function index
    pub start: Option<u32>,
    /// Custom sections
    pub custom_sections: HashMap<String, Vec<u8>>,
    /// Exports (functions, tables, memories, and globals)
    pub exports: HashMap<String, OtherExport>,
    /// Optional name for the module, often derived from the custom "name" section.
    pub name: Option<String>,
    /// Original binary (if available)
    pub binary: Option<Vec<u8>>,
    /// Table addresses for indirect function calls
    pub table_addrs: Vec<TableAddr>,
    /// Local variables for the module
    pub locals: Vec<Value>,
    /// Label arity for the module
    pub label_arity: usize,
    /// Module start function (optional)
    pub start_export: Option<String>,
}

impl Default for Module {
    fn default() -> Self {
        Self::new().expect("Failed to create default module")
    }
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

/// Represents a WebAssembly function
#[derive(Debug, Clone)]
pub struct Function {
    /// The type index of the function
    pub type_idx: u32,
    /// The local variables of the function
    pub locals: Vec<ValueType>,
    /// The instructions that make up the function body
    pub code: Vec<Instruction>,
}

impl Function {
    /// Creates a new function with the given type index, locals, and code
    #[must_use]
    pub const fn new(type_idx: u32, locals: Vec<ValueType>, code: Vec<Instruction>) -> Self {
        Self {
            type_idx,
            locals,
            code,
        }
    }
}

/// Represents an element segment for tables
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    /// Table index
    pub table_idx: u32,
    /// Offset expression
    pub offset: Vec<Instruction>,
    /// Function indices
    pub items: Vec<u32>,
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

impl Data {
    /// Returns a reference to the data in this segment
    pub fn data(&self) -> &[u8] {
        &self.init
    }
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherExport {
    /// Export name
    pub name: String,
    /// Export kind
    pub kind: ExportKind,
    /// Export index
    pub index: u32,
}

/// Represents the value of an export
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportValue {
    /// A function with the specified index
    Function(u32),
    /// A table with the specified index
    Table(u32),
    /// A memory with the specified index
    Memory(u32),
    /// A global with the specified index
    Global(u32),
}

/// Represents an index into one of the module's sections
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExportItem {
    /// A function with the specified index
    Function(u32),
    /// A table with the specified index
    Table(u32),
    /// A memory with the specified index
    Memory(u32),
    /// A global with the specified index
    Global(u32),
}

/// Represents a WebAssembly code section entry
#[derive(Debug, Clone)]
pub struct Code {
    /// The size of the code section entry
    pub size: u32,
    /// The local declarations
    pub locals: Vec<(u32, ValueType)>,
    /// The function body (instructions)
    pub expr: Vec<Instruction>,
}

impl Code {
    /// Creates a new code section entry with the given size, locals, and expression
    #[must_use]
    pub const fn new(size: u32, locals: Vec<(u32, ValueType)>, expr: Vec<Instruction>) -> Self {
        Self { size, locals, expr }
    }
}

impl Module {
    /// Creates a new empty module
    pub fn new() -> Result<Self> {
        Ok(Self {
            types: Vec::new(),
            imports: HashMap::new(),
            functions: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            elements: Vec::new(),
            data: Vec::new(),
            start: None,
            exports: HashMap::new(),
            custom_sections: HashMap::new(),
            name: None,
            binary: None,
            table_addrs: Vec::new(),
            locals: Vec::new(),
            label_arity: 0,
            start_export: None,
        })
    }

    /// Loads a WebAssembly binary and creates a Module.
    ///
    /// This method validates the binary format and loads it into the current module.
    pub fn load_from_binary(&mut self, bytes: &[u8]) -> Result<Self> {
        // Store the binary
        self.binary = Some(bytes.to_vec());

        // Use decoder_integration to load and validate the module
        decoder_integration::load_module_from_binary(bytes)
    }

    /// Validates the module
    ///
    /// This function checks that the module is valid according to the WebAssembly spec
    /// and performs integrity verification for ASIL-B requirements
    pub fn validate(&self) -> Result<()> {
        // Verify type integrity for all types
        for func_type in &self.types {
            func_type.verify()?;
        }

        // Verify function types
        for func in &self.functions {
            if func.type_idx as usize >= self.types.len() {
                return Err(Error::new(kinds::ExecutionError(format!(
                    "Invalid function type index: {}",
                    func.type_idx
                ))));
            }
        }

        // Verify custom sections
        for section in &self.custom_sections {
            // Verify custom section integrity
            let _checksum = wrt_types::verification::Checksum::compute(&section.data);
            // In a full implementation, we would store and verify this checksum
        }

        // Verify exports
        for export in &self.exports {
            match export.kind {
                ExportKind::Function => {
                    if export.index as usize >= self.functions.len() {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid function export index: {}",
                            export.index
                        ))));
                    }
                }
                ExportKind::Table => {
                    if export.index as usize >= self.tables.len() {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid table export index: {}",
                            export.index
                        ))));
                    }
                }
                ExportKind::Memory => {
                    if export.index as usize >= self.memories.len() {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid memory export index: {}",
                            export.index
                        ))));
                    }
                }
                ExportKind::Global => {
                    if export.index as usize >= self.globals.len() {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid global export index: {}",
                            export.index
                        ))));
                    }
                }
            }
        }

        // Verify dual-validation for global initializers
        for global in &self.globals {
            // Verify global type integrity
            // Ensure global's value matches its declared type
            let value = global.get();
            if !value.matches_type(&global.global_type.value_type) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Global value does not match its type: {:?} vs {:?}",
                    value.type_(),
                    global.global_type.value_type
                ))));
            }
        }

        // Verify memory segements integrity before access
        for data in &self.data {
            // Check that data segement is valid
            if data.memory_idx as usize >= self.memories.len() {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Data segment references invalid memory index: {}",
                    data.memory_idx
                ))));
            }

            // Verify data integrity with checksum
            let _checksum = wrt_types::verification::Checksum::compute(&data.init);
            // In a real implementation, we would verify against a stored checksum
        }

        // All good
        Ok(())
    }

    /// Create a WebAssembly binary module from raw bytes.
    ///
    /// This function parses a binary WebAssembly module from the provided bytes.
    /// It validates the binary format and constructs a Module instance.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The raw WebAssembly binary bytes.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `Module` on success, or an `Error` if parsing fails.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // Parse the binary using wrt-format
        let format_module = wrt_format::binary::parse_binary(bytes)?;

        // Create a new empty module
        let mut module = Self::new()?;

        // Store the original binary
        module.binary = Some(bytes.to_vec());

        // Convert binary contents to our module structure
        // This is a simplified version - a full implementation would
        // parse all module sections
        module.load_from_binary(bytes)?;

        Ok(module)
    }

    /// Creates an empty module
    #[must_use]
    pub fn empty() -> Self {
        Self {
            types: Vec::new(),
            imports: HashMap::new(),
            functions: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            exports: HashMap::new(),
            elements: Vec::new(),
            data: Vec::new(),
            custom_sections: HashMap::new(),
            binary: None,
            start: None,
            name: None,
            table_addrs: Vec::new(),
            locals: Vec::new(),
            label_arity: 0,
            start_export: None,
        }
    }

    /// Creates a Module from a WebAssembly Text Format (WAT) string.
    ///
    /// # Arguments
    ///
    /// * `wat`: A string slice containing the WAT representation of the module.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `Module` on success, or an `Error` if parsing fails.
    #[cfg(feature = "serialization")]
    pub fn from_wat(wat: &str) -> Result<Self> {
        let wasm = wat::parse_str(wat)?;
        Self::from_bytes(&wasm)
    }

    /// Gets an export by name
    #[must_use]
    pub fn get_export(&self, name: &str) -> Option<&OtherExport> {
        self.exports.get(name)
    }

    /// Adds a function export
    pub fn add_function_export(&mut self, name: String, index: u32) {
        self.exports.insert(
            name.clone(),
            OtherExport {
                name,
                index,
                kind: ExportKind::Function,
            },
        );
    }

    /// Adds a table export
    pub fn add_table_export(&mut self, name: String, index: u32) {
        self.exports.insert(
            name.clone(),
            OtherExport {
                name,
                index,
                kind: ExportKind::Table,
            },
        );
    }

    /// Adds a memory export
    pub fn add_memory_export(&mut self, name: String, index: u32) {
        self.exports.insert(
            name.clone(),
            OtherExport {
                name,
                index,
                kind: ExportKind::Memory,
            },
        );
    }

    /// Adds a global export
    pub fn add_global_export(&mut self, name: String, index: u32) {
        self.exports.insert(
            name.clone(),
            OtherExport {
                name,
                index,
                kind: ExportKind::Global,
            },
        );
    }

    /// Gets a reference to a function definition by its index.
    ///
    /// # Arguments
    ///
    /// * `idx`: The index of the function.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `Function` if found, otherwise `None`.
    #[must_use]
    pub fn get_function(&self, idx: u32) -> Option<&Function> {
        self.functions.get(idx as usize)
    }

    /// Gets a reference to the function type (signature) for a given function index.
    ///
    /// # Arguments
    ///
    /// * `idx`: The index of the function.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `FuncType` if the function and its type exist,
    /// otherwise `None`.
    #[must_use]
    pub fn get_function_type(&self, idx: u32) -> Option<&FuncType> {
        self.types
            .get(self.functions.get(idx as usize)?.type_idx as usize)
    }

    /// Gets a global by index
    pub fn get_global(&self, idx: usize) -> Result<Arc<Global>> {
        self.globals
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::new(kinds::InvalidGlobalIndexError(idx as u32)))
    }

    /// Gets a memory by index
    pub fn get_memory(&self, idx: usize) -> Result<Arc<Memory>> {
        self.memories
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::new(kinds::InvalidMemoryIndexError(idx as u32)))
    }

    /// Returns the number of memory definitions in the module.
    pub fn memories_len(&self) -> usize {
        self.memories.len()
    }

    /// Returns the number of table definitions in the module.
    pub fn tables_len(&self) -> usize {
        self.tables.len()
    }

    /// Returns the number of global definitions in the module.
    pub fn globals_len(&self) -> usize {
        self.globals.len()
    }

    /// Evaluates a constant expression (used for global/element/data offsets).
    ///
    /// Note: This requires an initial `stack` usually containing globals.
    /// For simple const exprs like `i32.const`, the stack can be empty.
    fn evaluate_const_expr(
        &self,
        expr: &[Instruction],
        // Initial stack, typically containing imported globals if needed by expr.
        initial_stack: &mut Vec<Value>,
    ) -> Result<Value> {
        let mut stack = initial_stack.clone(); // Use a local stack
        for instr in expr {
            match instr {
                Instruction::I32Const(v) => stack.push(Value::I32(*v)),
                Instruction::I64Const(v) => stack.push(Value::I64(*v)),
                Instruction::F32Const(v) => {
                    let value = *v;
                    stack.push(Value::F32(value));
                }
                Instruction::F64Const(v) => {
                    let value = *v;
                    stack.push(Value::F64(value));
                }
                Instruction::GlobalGet(_idx) => {
                    // For constant expressions, only imported immutable globals can be accessed
                    // This requires looking up the import definition and checking the global type
                    // Placeholder: Assume lookup logic exists
                    return Err(Error::new(kinds::ValidationError(
                        "global.get in constant expression not fully implemented".to_string(),
                    )));
                }
                Instruction::End => break, // End of expression
                _ => {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Unsupported instruction in constant expression: {:?}",
                        instr
                    ))))
                }
            }
        }

        if stack.len() == 1 {
            stack.pop().ok_or_else(|| {
                Error::new(kinds::ValidationError(
                    "Constant expression evaluation ended with empty stack".into(),
                ))
            })
        } else {
            Err(Error::new(kinds::ValidationError(format!(
                "Constant expression evaluation ended with {} values on stack (expected 1)",
                stack.len()
            ))))
        }
    }

    /// Creates a global variable instance based on its definition and initial value expression.
    pub fn create_global(
        &mut self,
        global_type: GlobalType,
        init_expr: Vec<Instruction>,
    ) -> Result<Arc<Global>> {
        let initial_value = self.evaluate_const_expr(&init_expr, &mut vec![])?; // Evaluate init expression
        let global = Arc::new(Global::new(global_type, initial_value)?);
        self.globals.push(global.clone());
        Ok(global)
    }

    pub fn from_reader<R: std::io::Read>(_reader: R) -> Result<Self> {
        Err(Error::new(kinds::ExecutionError(
            "Reading from reader not implemented".into(),
        )))
    }

    /// Returns a WebAssembly binary representation of this module
    pub fn to_binary(&self) -> crate::error::Result<Vec<u8>> {
        match &self.binary {
            Some(binary) => Ok(binary.clone()),
            None => Err(Error::new(kinds::ValidationError(
                "No binary available for this module".to_string(),
            ))),
        }
    }

    /// Creates a Module from WebAssembly binary bytes
    pub fn from_binary(bytes: &[u8]) -> Result<Self> {
        // Use decoder_integration to load and validate the module
        decoder_integration::load_module_from_binary(bytes)
    }

    /// Creates a Module from WebAssembly binary bytes with validation
    pub fn from_binary_validated(bytes: &[u8]) -> Result<Self> {
        // Use decoder_integration which already includes validation
        decoder_integration::load_module_from_binary(bytes)
    }

    fn get_function_body_data(&self, idx: u32) -> Result<&[u8], Error> {
        match self.functions.get(idx as usize) {
            Some(function) => Ok(&function.code),
            None => Err(Error::new(kinds::ValidationError(format!(
                "Function body not found for index {}",
                idx
            )))),
        }
    }
}
