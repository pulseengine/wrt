use crate::{
    behavior::{self, ControlFlowBehavior, FrameBehavior, StackBehavior},
    error::{kinds, Error, Result},
    global::Global,
    instructions::{types::BlockType, Instruction},
    memory::{DefaultMemory, MemoryBehavior},
    table::Table,
    types::{ExternType, GlobalType, MemoryType, TableType},
    types::{FuncType, ValueType},
    values::Value,
    String, Vec,
};

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
    pub imports: Vec<Import>,
    /// Function definitions
    pub functions: Vec<Function>,
    /// Table definitions
    pub tables: Arc<RwLock<Vec<Arc<Table>>>>,
    /// Memory definitions
    pub memories: Arc<RwLock<Vec<Arc<DefaultMemory>>>>,
    /// Global variable definitions
    pub globals: Arc<RwLock<Vec<Arc<Global>>>>,
    /// Element segments for tables
    pub elements: Vec<Element>,
    /// Data segments for memories
    pub data: Vec<Data>,
    /// Start function index
    pub start: Option<u32>,
    /// Custom sections
    pub custom_sections: Vec<CustomSection>,
    /// Exports (functions, tables, memories, and globals)
    pub exports: Vec<OtherExport>,
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
            imports: Vec::new(),
            functions: Vec::new(),
            tables: Arc::new(RwLock::new(Vec::new())),
            memories: Arc::new(RwLock::new(Vec::new())),
            globals: Arc::new(RwLock::new(Vec::new())),
            elements: Vec::new(),
            data: Vec::new(),
            start: None,
            exports: Vec::new(),
            custom_sections: Vec::new(),
            name: None,
            binary: None,
            table_addrs: Vec::new(),
            locals: Vec::new(),
            label_arity: 0,
        })
    }

    /// Loads a WebAssembly binary and creates a Module.
    ///
    /// This method validates the binary format and loads it into the current module.
    pub fn load_from_binary(&mut self, bytes: &[u8]) -> Result<Self> {
        // Store the binary
        self.binary = Some(bytes.to_vec());

        // Use wrt-decoder to parse the module
        let decoder_module = wrt_decoder::decode(bytes)?;

        // Create a new module
        let mut module = Self::new()?;

        // Copy over the binary
        module.binary = Some(bytes.to_vec());

        // Convert the decoder module to our runtime module
        // This is where we'd map from the decoder's representation to our runtime representation

        // Set basic properties
        module.name = decoder_module.name.clone();
        module.custom_sections = decoder_module.custom_sections.clone();
        module.start = decoder_module.start;

        // Convert types
        for ty in &decoder_module.types {
            // Convert each type and add to our module
            let runtime_type = convert_func_type(ty)?;
            module.types.push(runtime_type);
        }

        // Convert imports
        for import in &decoder_module.imports {
            // Convert each import and add to our module
            let runtime_import = convert_import(import)?;
            module.imports.push(runtime_import);
        }

        // Convert functions, tables, memories, globals, etc.
        // ... (conversion code here)

        // Validate the resulting module
        module.validate()?;

        Ok(module)
    }

    /// Validates the module
    ///
    /// This function checks that the module is valid according to the WebAssembly spec
    pub fn validate(&self) -> Result<()> {
        // Validate function types
        for func in &self.functions {
            if func.type_idx as usize >= self.types.len() {
                return Err(Error::new(kinds::ExecutionError(format!(
                    "Invalid function type index: {}",
                    func.type_idx
                ))));
            }
        }

        // Validate exports
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
                    if export.index as usize >= self.tables.read().unwrap().len() {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid table export index: {}",
                            export.index
                        ))));
                    }
                }
                ExportKind::Memory => {
                    if export.index as usize >= self.memories.read().unwrap().len() {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid memory export index: {}",
                            export.index
                        ))));
                    }
                }
                ExportKind::Global => {
                    if export.index as usize >= self.globals.read().unwrap().len() {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid global export index: {}",
                            export.index
                        ))));
                    }
                }
            }
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
            imports: Vec::new(),
            functions: Vec::new(),
            tables: Arc::new(RwLock::new(Vec::new())),
            memories: Arc::new(RwLock::new(Vec::new())),
            globals: Arc::new(RwLock::new(Vec::new())),
            exports: Vec::new(),
            elements: Vec::new(),
            data: Vec::new(),
            custom_sections: Vec::new(),
            binary: None,
            start: None,
            name: None,
            table_addrs: Vec::new(),
            locals: Vec::new(),
            label_arity: 0,
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
        self.exports.iter().find(|e| e.name == name)
    }

    /// Adds a function export
    pub fn add_function_export(&mut self, name: String, index: u32) {
        self.exports.push(OtherExport {
            name,
            kind: ExportKind::Function,
            index,
        });
    }

    /// Adds a table export
    pub fn add_table_export(&mut self, name: String, index: u32) {
        self.exports.push(OtherExport {
            name,
            kind: ExportKind::Table,
            index,
        });
    }

    /// Adds a memory export
    pub fn add_memory_export(&mut self, name: String, index: u32) {
        self.exports.push(OtherExport {
            name,
            kind: ExportKind::Memory,
            index,
        });
    }

    /// Adds a global export
    pub fn add_global_export(&mut self, name: String, index: u32) {
        self.exports.push(OtherExport {
            name,
            kind: ExportKind::Global,
            index,
        });
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
        let globals = self.globals.read().map_err(|_| {
            Error::new(kinds::PoisonedLockError(
                "Globals RwLock poisoned".to_string(),
            ))
        })?;
        globals
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::new(kinds::InvalidGlobalIndexError(idx as u32)))
    }

    /// Gets a memory by index
    pub fn get_memory(&self, idx: usize) -> Result<Arc<DefaultMemory>> {
        let memories = self.memories.read().map_err(|_| {
            Error::new(kinds::PoisonedLockError(
                "Memories RwLock poisoned".to_string(),
            ))
        })?;
        memories
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::new(kinds::InvalidMemoryIndexError(idx as u32)))
    }

    /// Returns the number of memory definitions in the module.
    pub fn memories_len(&self) -> usize {
        self.memories.read().map_or(0, |memories| memories.len())
    }

    /// Returns the number of table definitions in the module.
    pub fn tables_len(&self) -> usize {
        self.tables.read().map_or(0, |tables| tables.len())
    }

    /// Returns the number of global definitions in the module.
    pub fn globals_len(&self) -> usize {
        self.globals.read().map_or(0, |globals| globals.len())
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
        self.globals
            .write()
            .map_err(|_| {
                Error::new(kinds::PoisonedLockError(
                    "Globals RwLock poisoned".to_string(),
                ))
            })?
            .push(global.clone());
        Ok(global)
    }

    pub fn from_reader<R: std::io::Read>(_reader: R) -> Result<Self> {
        Err(Error::new(kinds::ExecutionError(
            "Reading from reader not implemented".into(),
        )))
    }
}

/// Convert a FuncType from wrt-decoder to wrt
fn convert_func_type(decoder_type: &wrt_decoder::sections::FuncType) -> Result<FuncType> {
    // Map parameter and result types
    let params = decoder_type
        .params
        .iter()
        .map(convert_value_type)
        .collect::<Result<Vec<ValueType>>>()?;

    let results = decoder_type
        .results
        .iter()
        .map(convert_value_type)
        .collect::<Result<Vec<ValueType>>>()?;

    Ok(FuncType { params, results })
}

/// Convert a ValueType from wrt-decoder to wrt
fn convert_value_type(decoder_type: &wrt_decoder::sections::ValueType) -> Result<ValueType> {
    use crate::types::ValueType as RuntimeType;
    use wrt_decoder::sections::ValueType as DecoderType;

    match decoder_type {
        DecoderType::I32 => Ok(RuntimeType::I32),
        DecoderType::I64 => Ok(RuntimeType::I64),
        DecoderType::F32 => Ok(RuntimeType::F32),
        DecoderType::F64 => Ok(RuntimeType::F64),
        DecoderType::FuncRef => Ok(RuntimeType::FuncRef),
        DecoderType::ExternRef => Ok(RuntimeType::ExternRef),
        // Add other type conversions as needed
        _ => Err(Error::new(kinds::ParseError(
            "Unsupported value type".to_string(),
        ))),
    }
}

/// Convert an Import from wrt-decoder to wrt
fn convert_import(decoder_import: &wrt_decoder::sections::Import) -> Result<Import> {
    use wrt_decoder::sections::ImportDesc as DecoderDesc;

    let desc = match &decoder_import.desc {
        DecoderDesc::Func(type_idx) => ExternType::Func(*type_idx),
        DecoderDesc::Table(table_type) => {
            // Convert table type
            let limits = crate::types::Limits {
                min: table_type.limits.min,
                max: table_type.limits.max,
            };

            let element_type = convert_element_type(&table_type.element_type)?;

            let table_type = crate::types::TableType {
                limits,
                element_type,
            };

            ExternType::Table(table_type)
        }
        DecoderDesc::Memory(memory_type) => {
            // Convert memory type
            let limits = crate::types::Limits {
                min: memory_type.limits.min,
                max: memory_type.limits.max,
            };

            let memory_type = crate::types::MemoryType {
                limits,
                shared: memory_type.shared,
            };

            ExternType::Memory(memory_type)
        }
        DecoderDesc::Global(global_type) => {
            // Convert global type
            let value_type = convert_value_type(&global_type.value_type)?;

            let global_type = crate::types::GlobalType {
                value_type,
                mutable: global_type.mutable,
            };

            ExternType::Global(global_type)
        }
    };

    Ok(Import {
        module: decoder_import.module.clone(),
        name: decoder_import.name.clone(),
        ty: desc,
    })
}

/// Convert an element type from wrt-decoder to wrt
fn convert_element_type(
    decoder_type: &wrt_decoder::sections::ElementType,
) -> Result<crate::types::ValueType> {
    use crate::types::ValueType as RuntimeType;
    use wrt_decoder::sections::ElementType as DecoderType;

    match decoder_type {
        DecoderType::FuncRef => Ok(RuntimeType::FuncRef),
        // Add other element type conversions as needed
        _ => Err(Error::new(kinds::ParseError(
            "Unsupported element type".to_string(),
        ))),
    }
}

/// Serialize the module to a binary format
#[cfg(feature = "serialization")]
pub fn to_binary(&self) -> crate::error::Result<Vec<u8>> {
    // For now, we'll use the original binary if available,
    // otherwise recreate from the parsed module
    if let Some(binary) = &self.binary {
        Ok(binary.clone())
    } else {
        // In a real implementation, convert our runtime module to a decoder module,
        // then use wrt-decoder to encode it
        Err(Error::Validation(
            "Serializing a module without original binary is not yet supported".into(),
        ))
    }
}
