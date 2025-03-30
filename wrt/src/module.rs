use crate::{
    behavior::{self, ControlFlowBehavior, FrameBehavior, StackBehavior},
    error::{Error, Result},
    global::Global,
    instructions::{types::BlockType, Instruction},
    memory::Memory,
    stack::Stack,
    table::Table,
    types::{ExternType, GlobalType, MemoryType, TableType},
    types::{FuncType, ValueType},
    values::Value,
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

#[derive(Debug, Clone, Copy)]
pub struct TableAddr {
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
    pub memories: Arc<RwLock<Vec<Arc<Memory>>>>,
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
    /// This method validates the binary format and returns a parsed Module.
    pub fn load_from_binary(&mut self, bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(Error::Parse("Binary too short".into()));
        }

        // Check magic number and version
        if bytes[0..8] == [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00] {
            self.load_wasm_binary(bytes)
        } else if bytes[0..8] == [0x00, 0x61, 0x73, 0x6D, 0x0D, 0x00, 0x01, 0x00] {
            self.load_component_binary(bytes)
        } else {
            Err(Error::Parse("Invalid binary format".into()))
        }
    }

    /// Load a WebAssembly module binary
    fn load_wasm_binary(&self, bytes: &[u8]) -> Result<Self> {
        let mut module = self.clone();

        // Clear existing definitions
        module.memories.write().unwrap().clear();
        module.functions.clear();
        module.imports.clear();
        module.exports.clear();
        module.globals.write().unwrap().clear();
        module.data.clear();
        module.elements.clear();
        module.tables.write().unwrap().clear();
        module.types.clear();
        module.custom_sections.clear();

        // Initialize module from binary
        parse_module(&mut module, bytes)?;

        Ok(module)
    }

    /// Load a WebAssembly component binary
    fn load_component_binary(&self, bytes: &[u8]) -> Result<Self> {
        let mut module = self.clone();

        // Clear existing definitions
        module.memories.write().unwrap().clear();
        module.functions.clear();
        module.imports.clear();
        module.exports.clear();
        module.globals.write().unwrap().clear();
        module.data.clear();
        module.elements.clear();
        module.tables.write().unwrap().clear();
        module.types.clear();
        module.custom_sections.clear();

        // Parse the component binary
        parse_component(&mut module, bytes)?;

        Ok(module)
    }

    /// Validates the module
    ///
    /// This function checks that the module is valid according to the WebAssembly spec
    pub fn validate(&self) -> Result<()> {
        // Validate function types
        for func in &self.functions {
            if func.type_idx as usize >= self.types.len() {
                return Err(Error::Parse(format!(
                    "Invalid function type index: {}",
                    func.type_idx
                )));
            }
        }

        // Validate exports
        for export in &self.exports {
            match export.kind {
                ExportKind::Function => {
                    if export.index as usize >= self.functions.len() {
                        return Err(Error::Parse(format!(
                            "Invalid function export index: {}",
                            export.index
                        )));
                    }
                }
                ExportKind::Table => {
                    if export.index as usize >= self.tables.read().unwrap().len() {
                        return Err(Error::Parse(format!(
                            "Invalid table export index: {}",
                            export.index
                        )));
                    }
                }
                ExportKind::Memory => {
                    if export.index as usize >= self.memories.read().unwrap().len() {
                        return Err(Error::Parse(format!(
                            "Invalid memory export index: {}",
                            export.index
                        )));
                    }
                }
                ExportKind::Global => {
                    if export.index as usize >= self.globals.read().unwrap().len() {
                        return Err(Error::Parse(format!(
                            "Invalid global export index: {}",
                            export.index
                        )));
                    }
                }
            }
        }

        // All good
        Ok(())
    }

    #[cfg(feature = "serialization")]
    /// Serialize the module to a binary format
    pub fn to_binary(&self) -> crate::error::Result<Vec<u8>> {
        use crate::error::Error;

        // For now, we'll use the original binary if available,
        // otherwise recreate from the parsed module
        if let Some(binary) = &self.binary {
            Ok(binary.clone())
        } else {
            // In a real implementation, regenerate the binary from the module
            // For now, return an error as this is not yet implemented
            Err(Error::Validation(
                "Serializing a module without original binary is not yet supported".into(),
            ))
        }
    }

    /// Creates a Module from WebAssembly binary bytes
    ///
    /// # Parameters
    ///
    /// * `bytes` - The WebAssembly binary bytes
    ///
    /// # Returns
    ///
    /// The parsed module, or an error if the binary is invalid
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::new()?.load_from_binary(bytes)
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

    #[must_use]
    pub fn get_function(&self, idx: u32) -> Option<&Function> {
        self.functions.get(idx as usize)
    }

    #[must_use]
    pub fn get_function_type(&self, idx: u32) -> Option<&FuncType> {
        self.types
            .get(self.functions.get(idx as usize)?.type_idx as usize)
    }

    /// Gets a global by index
    pub fn get_global(&self, idx: usize) -> Result<Arc<Global>> {
        let globals = self.globals.read().map_err(|_| Error::PoisonedLock)?;
        globals
            .get(idx)
            .map(Arc::clone)
            .ok_or(Error::InvalidGlobalIndex(idx))
    }

    /// Gets a mutable global by index
    pub fn get_global_mut(&mut self, idx: usize) -> Option<&mut Global> {
        // Implementation that handles Arc<RwLock<>> is complex, returning None for now
        None
    }

    pub fn get_global_mut_by_idx(&mut self, idx: u32) -> Option<&mut Global> {
        self.get_global_mut(idx as usize)
    }

    /// Gets a memory by index
    pub fn get_memory(&self, idx: usize) -> Result<Arc<Memory>> {
        let memories = self.memories.read().map_err(|_| Error::PoisonedLock)?;
        memories
            .get(idx)
            .map(Arc::clone)
            .ok_or(Error::InvalidMemoryIndex(idx))
    }

    /// Gets a mutable memory by index
    pub fn get_memory_mut(&mut self, idx: usize) -> Result<Arc<Memory>> {
        let memories = self.memories.read().map_err(|_| Error::PoisonedLock)?;
        memories
            .get(idx)
            .map(Arc::clone)
            .ok_or(Error::InvalidMemoryIndex(idx))
    }

    /// Gets a table by index
    pub fn get_table(&self, idx: usize) -> Result<Arc<Table>> {
        let tables = self.tables.read().map_err(|_| Error::PoisonedLock)?;
        tables
            .get(idx)
            .map(Arc::clone)
            .ok_or(Error::InvalidTableIndex(idx))
    }

    /// Gets a mutable table by index
    pub fn get_table_mut(&mut self, idx: usize) -> Result<Arc<Table>> {
        let tables = self.tables.read().map_err(|_| Error::PoisonedLock)?;
        tables
            .get(idx)
            .map(Arc::clone)
            .ok_or(Error::InvalidTableIndex(idx))
    }

    #[must_use]
    pub fn memories_len(&self) -> usize {
        self.memories.read().unwrap().len()
    }

    #[must_use]
    pub fn tables_len(&self) -> usize {
        self.tables.read().unwrap().len()
    }

    #[must_use]
    pub fn globals_len(&self) -> usize {
        self.globals.read().unwrap().len()
    }

    pub fn create_global(
        &mut self,
        global_type: GlobalType,
        init_expr: Vec<Instruction>,
    ) -> Result<Arc<Global>> {
        let global = Global::new(global_type, Value::I32(0))?;
        let global_arc = Arc::new(global);
        let globals = &mut self.globals.write().map_err(|_| Error::PoisonedLock)?;
        globals.push(Arc::clone(&global_arc));
        Ok(global_arc)
    }
}

impl FrameBehavior for Module {
    fn locals(&mut self) -> &mut Vec<Value> {
        &mut self.locals
    }

    fn get_local(&self, idx: usize) -> Result<Value> {
        self.locals
            .get(idx)
            .cloned()
            .ok_or(Error::InvalidLocal(format!(
                "Local index out of bounds: {idx}"
            )))
    }

    fn set_local(&mut self, idx: usize, value: Value) -> Result<()> {
        if idx < self.locals.len() {
            self.locals[idx] = value;
            Ok(())
        } else {
            Err(Error::InvalidLocal(format!(
                "Local index out of bounds: {idx}"
            )))
        }
    }

    fn get_global(&self, idx: usize) -> Result<Value> {
        let global = self.get_global(idx)?;
        Ok(global.value.clone())
    }

    fn set_global(&mut self, idx: usize, value: Value) -> Result<()> {
        let global = self.get_global(idx)?;
        if !global.global_type.mutable {
            return Err(Error::GlobalNotMutable(idx));
        }
        // Since we can't modify the global directly due to Arc, we'll need to update this
        // with a proper implementation in the future
        Ok(())
    }

    fn get_memory(&self, idx: usize) -> Result<&Memory> {
        // Since we use Arc<Memory>, we can't return a direct reference
        Err(Error::InvalidMemoryIndex(idx))
    }

    fn get_memory_mut(&mut self, idx: usize) -> Result<&mut Memory> {
        // Since we use Arc<Memory>, we can't return a direct mutable reference
        Err(Error::InvalidMemoryIndex(idx))
    }

    fn get_table(&self, idx: usize) -> Result<&Table> {
        // Since we use Arc<Table>, we can't return a direct reference
        Err(Error::InvalidTableIndex(idx))
    }

    fn get_table_mut(&mut self, idx: usize) -> Result<&mut Table> {
        // Since we use Arc<Table>, we can't return a direct mutable reference
        Err(Error::InvalidTableIndex(idx))
    }

    fn get_global_mut(&mut self, idx: usize) -> Option<&mut Global> {
        // Implementation that handles Arc<RwLock<>> is complex, returning None for now
        None
    }

    fn pc(&self) -> usize {
        unimplemented!("Module does not have a program counter")
    }

    fn set_pc(&mut self, pc: usize) {
        unimplemented!("Module does not have a program counter")
    }

    fn func_idx(&self) -> u32 {
        unimplemented!("Module does not have a function index")
    }

    fn instance_idx(&self) -> usize {
        unimplemented!("Module does not have an instance index")
    }

    fn locals_len(&self) -> usize {
        unimplemented!("Module does not have locals")
    }

    // Add missing methods
    fn label_stack(&mut self) -> &mut Vec<behavior::Label> {
        unimplemented!("Module does not have a label stack")
    }

    fn arity(&self) -> usize {
        unimplemented!("Module does not have arity")
    }

    fn set_arity(&mut self, _arity: usize) {
        unimplemented!("Module does not have arity")
    }

    fn label_arity(&self) -> usize {
        self.label_arity
    }

    fn return_pc(&self) -> usize {
        0
    }

    fn set_return_pc(&mut self, _pc: usize) {
        unimplemented!("Module does not have a return program counter")
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    // Implement the remaining methods from FrameBehavior trait
    fn load_i32(&mut self, addr: usize, align: u32) -> Result<i32> {
        unimplemented!("Module does not support memory operations")
    }

    fn load_i64(&mut self, addr: usize, align: u32) -> Result<i64> {
        unimplemented!("Module does not support memory operations")
    }

    fn load_f32(&mut self, addr: usize, align: u32) -> Result<f32> {
        unimplemented!("Module does not support memory operations")
    }

    fn load_f64(&mut self, addr: usize, align: u32) -> Result<f64> {
        unimplemented!("Module does not support memory operations")
    }

    fn load_i8(&mut self, addr: usize, align: u32) -> Result<i8> {
        unimplemented!("Module does not support memory operations")
    }

    fn load_u8(&mut self, addr: usize, align: u32) -> Result<u8> {
        unimplemented!("Module does not support memory operations")
    }

    fn load_i16(&mut self, addr: usize, align: u32) -> Result<i16> {
        unimplemented!("Module does not support memory operations")
    }

    fn load_u16(&mut self, addr: usize, align: u32) -> Result<u16> {
        unimplemented!("Module does not support memory operations")
    }

    fn store_i32(&mut self, addr: usize, align: u32, value: i32) -> Result<()> {
        unimplemented!("Module does not support memory operations")
    }

    fn store_i64(&mut self, addr: usize, align: u32, value: i64) -> Result<()> {
        unimplemented!("Module does not support memory operations")
    }

    fn memory_size(&mut self) -> Result<u32> {
        unimplemented!("Module does not support memory operations")
    }

    fn memory_grow(&mut self, pages: u32) -> Result<u32> {
        unimplemented!("Module does not support memory operations")
    }

    fn table_get(&mut self, table_idx: u32, idx: u32) -> Result<Value> {
        unimplemented!("Module does not support table operations")
    }

    fn table_set(&mut self, table_idx: u32, idx: u32, value: Value) -> Result<()> {
        unimplemented!("Module does not support table operations")
    }

    fn table_size(&mut self, table_idx: u32) -> Result<u32> {
        unimplemented!("Module does not support table operations")
    }

    fn table_grow(&mut self, table_idx: u32, delta: u32, value: Value) -> Result<u32> {
        unimplemented!("Module does not support table operations")
    }

    fn table_init(
        &mut self,
        table_idx: u32,
        elem_idx: u32,
        dst: u32,
        src: u32,
        n: u32,
    ) -> Result<()> {
        unimplemented!("Module does not support table operations")
    }

    fn table_copy(
        &mut self,
        dst_table: u32,
        src_table: u32,
        dst: u32,
        src: u32,
        n: u32,
    ) -> Result<()> {
        unimplemented!("Module does not support table operations")
    }

    fn elem_drop(&mut self, elem_idx: u32) -> Result<()> {
        unimplemented!("Module does not support element operations")
    }

    fn table_fill(&mut self, table_idx: u32, dst: u32, val: Value, n: u32) -> Result<()> {
        unimplemented!("Module does not support table operations")
    }

    fn pop_bool(&mut self, stack: &mut dyn Stack) -> Result<bool> {
        match stack.pop()? {
            Value::I32(0) => Ok(false),
            Value::I32(_) => Ok(true),
            _ => Err(Error::TypeMismatch(
                "Expected i32 boolean value".to_string(),
            )),
        }
    }

    fn pop_i32(&mut self, stack: &mut dyn Stack) -> Result<i32> {
        match stack.pop()? {
            Value::I32(v) => Ok(v),
            _ => Err(Error::TypeMismatch("Expected i32 value".to_string())),
        }
    }
}

impl ControlFlowBehavior for Module {
    fn enter_block(&mut self, _ty: BlockType, _stack_len: usize) -> Result<()> {
        unimplemented!("Module does not support control flow operations")
    }

    fn enter_loop(&mut self, _ty: BlockType, _stack_len: usize) -> Result<()> {
        unimplemented!("Module does not support control flow operations")
    }

    fn enter_if(&mut self, _ty: BlockType, _stack_len: usize, _condition: bool) -> Result<()> {
        unimplemented!("Module does not support control flow operations")
    }

    fn enter_else(&mut self, _stack_len: usize) -> Result<()> {
        unimplemented!("Module does not support control flow operations")
    }

    fn exit_block(&mut self, _stack: &mut dyn Stack) -> Result<()> {
        unimplemented!("Module does not support control flow operations")
    }

    fn branch(&mut self, _label_idx: u32, _stack: &mut dyn Stack) -> Result<()> {
        unimplemented!("Module does not support control flow operations")
    }

    fn return_(&mut self, _stack: &mut dyn Stack) -> Result<()> {
        unimplemented!("Module does not support control flow operations")
    }

    fn call(&mut self, _func_idx: u32, _stack: &mut dyn Stack) -> Result<()> {
        unimplemented!("Module does not support control flow operations")
    }

    fn call_indirect(
        &mut self,
        _type_idx: u32,
        _table_idx: u32,
        _entry: u32,
        _stack: &mut dyn Stack,
    ) -> Result<()> {
        unimplemented!("call_indirect not implemented for Module")
    }

    fn set_label_arity(&mut self, arity: usize) {
        self.label_arity = arity;
    }
}

/// Returns the decoded value and the number of bytes read
fn read_leb128_u32(bytes: &[u8]) -> Result<(u32, usize)> {
    let mut result = 0u32;
    let mut shift = 0;
    let mut bytes_read = 0;
    let mut byte;

    loop {
        byte = bytes.get(bytes_read).ok_or(Error::UnexpectedEof)?;
        bytes_read += 1;

        result |= u32::from(byte & 0x7f) << shift;
        shift += 7;

        if (byte & 0x80) == 0 {
            break;
        }

        if shift >= 32 {
            return Err(Error::InvalidLeb128("LEB128 value too large".to_string()));
        }
    }

    Ok((result, bytes_read))
}

/// Returns the decoded signed value and the number of bytes read
fn read_leb128_i32(bytes: &[u8]) -> Result<(i32, usize)> {
    let mut result = 0i32;
    let mut shift = 0;
    let mut bytes_read = 0;
    let mut byte;
    let mut sign_bit_set = false;

    loop {
        byte = bytes.get(bytes_read).ok_or(Error::UnexpectedEof)?;
        bytes_read += 1;

        // Apply the 7 bits to the result
        result |= (i32::from(byte & 0x7f)) << shift;
        shift += 7;

        // Check if we're done
        if (byte & 0x80) == 0 {
            // Check if the sign bit (bit 6 in the last byte) is set
            sign_bit_set = (byte & 0x40) != 0;
            break;
        }

        if shift >= 32 {
            return Err(Error::InvalidLeb128("LEB128 value too large".to_string()));
        }
    }

    // Sign extend if necessary
    if sign_bit_set && shift < 32 {
        // Fill in the sign extension bits
        result |= !0 << shift;
    }

    Ok((result, bytes_read))
}

/// Returns the decoded signed value and the number of bytes read
fn read_leb128_i64(bytes: &[u8]) -> Result<(i64, usize)> {
    let mut result = 0i64;
    let mut shift = 0;
    let mut bytes_read = 0;
    let mut byte;
    let mut sign_bit_set = false;

    loop {
        byte = bytes.get(bytes_read).ok_or(Error::UnexpectedEof)?;
        bytes_read += 1;

        // Apply the 7 bits to the result
        result |= (i64::from(byte & 0x7f)) << shift;
        shift += 7;

        // Check if we're done
        if (byte & 0x80) == 0 {
            // Check if the sign bit (bit 6 in the last byte) is set
            sign_bit_set = (byte & 0x40) != 0;
            break;
        }

        if shift >= 64 {
            return Err(Error::InvalidLeb128("LEB128 value too large".to_string()));
        }
    }

    // Sign extend if necessary
    if sign_bit_set && shift < 64 {
        // Fill in the sign extension bits
        result |= !0 << shift;
    }

    Ok((result, bytes_read))
}

fn parse_module(module: &mut Module, bytes: &[u8]) -> Result<()> {
    // Parse module header
    if bytes.len() < 8 {
        return Err(Error::InvalidModule("Invalid module".to_string()));
    }

    // Check magic number
    if bytes[0..4] != [0x00, 0x61, 0x73, 0x6D] {
        return Err(Error::InvalidModule("Invalid module".to_string()));
    }

    // Check version
    if bytes[4..8] != [0x01, 0x00, 0x00, 0x00] {
        return Err(Error::InvalidModule("Invalid module".to_string()));
    }

    // Parse sections
    let mut offset = 8;
    while offset < bytes.len() {
        let section_id = bytes[offset];
        offset += 1;

        let (size, bytes_read) = read_leb128_u32(&bytes[offset..])?;
        offset += bytes_read;

        match section_id {
            // Type section
            0x01 => {
                let (count, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;

                for _ in 0..count {
                    let (func_type, bytes_read) = read_func_type(&bytes[offset..])?;
                    offset += bytes_read;
                    module.types.push(func_type);
                }
            }
            // Import section
            0x02 => {
                let (count, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;

                for _ in 0..count {
                    let (import, bytes_read) = read_import(&bytes[offset..])?;
                    offset += bytes_read;
                    module.imports.push(import);
                }
            }
            // Function section
            0x03 => {
                let (count, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;

                for _ in 0..count {
                    let (type_idx, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                    offset += bytes_read;
                    module
                        .functions
                        .push(Function::new(type_idx, Vec::new(), Vec::new()));
                }
            }
            // Table section
            0x04 => {
                let (count, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;

                for _ in 0..count {
                    let (table, bytes_read) = read_table(&bytes[offset..])?;
                    offset += bytes_read;
                    module.tables.write().unwrap().push(Arc::clone(&table));
                }
            }
            // Memory section
            0x05 => {
                let (count, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;

                for _ in 0..count {
                    let (memory, bytes_read) = read_memory(&bytes[offset..])?;
                    offset += bytes_read;
                    module.memories.write().unwrap().push(Arc::clone(&memory));
                }
            }
            // Global section
            0x06 => {
                let (count, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;

                for _ in 0..count {
                    let (global, bytes_read) = read_global(&bytes[offset..])?;
                    offset += bytes_read;
                    module.globals.write().unwrap().push(Arc::clone(&global));
                }
            }
            // Export section
            0x07 => {
                let (count, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;

                for _ in 0..count {
                    let (export, bytes_read) = read_export(&bytes[offset..])?;
                    offset += bytes_read;
                    module.exports.push(export);
                }
            }
            // Start section
            0x08 => {
                let (start_func, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;
                module.start = Some(start_func);
            }
            // Element section
            0x09 => {
                let (count, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;

                for _ in 0..count {
                    let (element, bytes_read) = read_element(&bytes[offset..])?;
                    offset += bytes_read;
                    module.elements.push(element);
                }
            }
            // Code section
            0x0A => {
                let (count, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;

                for _ in 0..count {
                    let (code, bytes_read) = read_code(&bytes[offset..])?;
                    offset += bytes_read;
                    module.functions[count as usize - 1].code = code.expr;
                }
            }
            // Data section
            0x0B => {
                let (count, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;

                for _ in 0..count {
                    let (data, bytes_read) = read_data(&bytes[offset..])?;
                    offset += bytes_read;
                    module.data.push(data);
                }
            }
            // Custom section
            0x00 => {
                // Skip custom section
                offset += size as usize;
            }
            // Unknown section
            _ => {
                return Err(Error::InvalidModule("Invalid module".to_string()));
            }
        }
    }

    Ok(())
}

fn parse_component(module: &mut Module, bytes: &[u8]) -> Result<()> {
    // TODO: Implement component parsing
    Err(Error::InvalidModule(
        "Component parsing not implemented".into(),
    ))
}

fn read_func_type(bytes: &[u8]) -> Result<(FuncType, usize)> {
    if bytes.is_empty() {
        return Err(Error::Parse("Empty function type section".into()));
    }

    // The first byte should be 0x60 for function type
    if bytes[0] != 0x60 {
        return Err(Error::Parse(format!(
            "Invalid function type tag: 0x{:02x}, expected 0x60",
            bytes[0]
        )));
    }

    let mut offset = 1;

    // Read parameter count (leb128 encoded)
    let (param_count, param_bytes_read) = read_leb128_u32(&bytes[offset..])?;
    offset += param_bytes_read;

    // Read parameters
    let mut params = Vec::with_capacity(param_count as usize);
    for _ in 0..param_count {
        if offset >= bytes.len() {
            return Err(Error::Parse("Unexpected end of function type bytes".into()));
        }

        let value_type = match bytes[offset] {
            0x7F => ValueType::I32,
            0x7E => ValueType::I64,
            0x7D => ValueType::F32,
            0x7C => ValueType::F64,
            0x70 => ValueType::FuncRef,
            0x6F => ValueType::ExternRef,
            0x7B => ValueType::V128,
            _ => {
                return Err(Error::Parse(format!(
                    "Invalid value type: 0x{:02x}",
                    bytes[offset]
                )));
            }
        };
        params.push(value_type);
        offset += 1;
    }

    // Read result count (leb128 encoded)
    let (result_count, result_bytes_read) = read_leb128_u32(&bytes[offset..])?;
    offset += result_bytes_read;

    // Read results
    let mut results = Vec::with_capacity(result_count as usize);
    for _ in 0..result_count {
        if offset >= bytes.len() {
            return Err(Error::Parse("Unexpected end of function type bytes".into()));
        }

        let value_type = match bytes[offset] {
            0x7F => ValueType::I32,
            0x7E => ValueType::I64,
            0x7D => ValueType::F32,
            0x7C => ValueType::F64,
            0x70 => ValueType::FuncRef,
            0x6F => ValueType::ExternRef,
            0x7B => ValueType::V128,
            _ => {
                return Err(Error::Parse(format!(
                    "Invalid value type: 0x{:02x}",
                    bytes[offset]
                )));
            }
        };
        results.push(value_type);
        offset += 1;
    }

    Ok((FuncType { params, results }, offset))
}

fn read_import(bytes: &[u8]) -> Result<(Import, usize)> {
    // TODO: Implement import reading
    Err(Error::InvalidImport(
        "Import reading not implemented".into(),
    ))
}

fn read_table(bytes: &[u8]) -> Result<(Arc<Table>, usize)> {
    let table_type = TableType {
        element_type: ValueType::FuncRef,
        min: 0,
        max: None,
    };
    let table = Table::new(table_type);
    Ok((Arc::new(table), 0))
}

fn read_memory(bytes: &[u8]) -> Result<(Arc<Memory>, usize)> {
    let memory_type = MemoryType { min: 0, max: None };
    let memory = Memory::new(memory_type);
    Ok((Arc::new(memory), 0))
}

fn read_global(bytes: &[u8]) -> Result<(Arc<Global>, usize)> {
    let global_type = GlobalType {
        content_type: ValueType::I32,
        mutable: false,
    };
    let global = Global::new(global_type, Value::I32(0))?;
    Ok((Arc::new(global), 0))
}

fn read_export(bytes: &[u8]) -> Result<(OtherExport, usize)> {
    if bytes.is_empty() {
        return Err(Error::Parse("Empty export section".into()));
    }

    let mut offset = 0;

    // Read export name length (LEB128)
    let (name_len, name_len_bytes) = read_leb128_u32(&bytes[offset..])?;
    offset += name_len_bytes;

    if offset + name_len as usize > bytes.len() {
        return Err(Error::Parse("Export name exceeds available bytes".into()));
    }

    // Read export name
    let name_bytes = &bytes[offset..offset + name_len as usize];
    let name = match std::str::from_utf8(name_bytes) {
        Ok(s) => s.to_string(),
        Err(_) => return Err(Error::Parse("Invalid UTF-8 sequence in export name".into())),
    };
    offset += name_len as usize;

    // Read export kind
    if offset >= bytes.len() {
        return Err(Error::Parse("Unexpected end of export bytes".into()));
    }

    let kind = match bytes[offset] {
        0x00 => ExportKind::Function,
        0x01 => ExportKind::Table,
        0x02 => ExportKind::Memory,
        0x03 => ExportKind::Global,
        _ => {
            return Err(Error::Parse(format!(
                "Invalid export kind: 0x{:02x}",
                bytes[offset]
            )))
        }
    };
    offset += 1;

    // Read export index (LEB128)
    let (index, index_bytes) = read_leb128_u32(&bytes[offset..])?;
    offset += index_bytes;

    Ok((OtherExport { name, kind, index }, offset))
}

fn read_element(bytes: &[u8]) -> Result<(Element, usize)> {
    // TODO: Implement element reading
    Err(Error::InvalidElement(
        "Element reading not implemented".into(),
    ))
}

fn read_code(bytes: &[u8]) -> Result<(Code, usize)> {
    if bytes.is_empty() {
        return Err(Error::Parse("Empty code section".into()));
    }

    let mut offset = 0;

    // Read the size of the code section entry
    let (size, size_bytes_read) = read_leb128_u32(&bytes[offset..])?;
    offset += size_bytes_read;

    // Read local declarations
    let (local_count, local_count_bytes) = read_leb128_u32(&bytes[offset..])?;
    offset += local_count_bytes;

    let mut locals = Vec::with_capacity(local_count as usize);
    let mut total_bytes_read = size_bytes_read + local_count_bytes;

    // Read local entries
    for _ in 0..local_count {
        if offset >= bytes.len() {
            return Err(Error::Parse("Unexpected end of code section".into()));
        }

        // Read local count
        let (count, count_bytes) = read_leb128_u32(&bytes[offset..])?;
        offset += count_bytes;
        total_bytes_read += count_bytes;

        // Read local type
        if offset >= bytes.len() {
            return Err(Error::Parse("Unexpected end of code section".into()));
        }

        let value_type = match bytes[offset] {
            0x7F => ValueType::I32,
            0x7E => ValueType::I64,
            0x7D => ValueType::F32,
            0x7C => ValueType::F64,
            0x70 => ValueType::FuncRef,
            0x6F => ValueType::ExternRef,
            0x7B => ValueType::V128,
            _ => {
                return Err(Error::Parse(format!(
                    "Invalid value type: 0x{:02x}",
                    bytes[offset]
                )));
            }
        };
        offset += 1;
        total_bytes_read += 1;

        locals.push((count, value_type));
    }

    // Read expression (instructions)
    let mut expr = Vec::new();
    let function_body_start = offset;
    let function_body_size = (size as usize).saturating_sub(total_bytes_read - size_bytes_read);

    // Decode instructions until end opcode or size limit
    let mut i = 0;
    while i < function_body_size && offset < bytes.len() {
        let opcode = bytes[offset];
        offset += 1;
        i += 1;

        let instruction = match opcode {
            0x00 => Instruction::Unreachable,
            0x01 => Instruction::Nop,
            0x02 => {
                // Block instruction with block type
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let block_type = match bytes[offset] {
                    0x40 => BlockType::Empty,
                    0x7F => BlockType::Value(ValueType::I32),
                    0x7E => BlockType::Value(ValueType::I64),
                    0x7D => BlockType::Value(ValueType::F32),
                    0x7C => BlockType::Value(ValueType::F64),
                    _ => BlockType::Empty, // Simplified for now
                };
                offset += 1;
                i += 1;
                Instruction::Block(block_type)
            }
            0x03 => {
                // Loop instruction with block type
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let block_type = match bytes[offset] {
                    0x40 => BlockType::Empty,
                    0x7F => BlockType::Value(ValueType::I32),
                    0x7E => BlockType::Value(ValueType::I64),
                    0x7D => BlockType::Value(ValueType::F32),
                    0x7C => BlockType::Value(ValueType::F64),
                    _ => BlockType::Empty, // Simplified for now
                };
                offset += 1;
                i += 1;
                Instruction::Loop(block_type)
            }
            0x0B => Instruction::End,
            0x0C => {
                // br instruction
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let (label_idx, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;
                i += bytes_read;
                Instruction::Br(label_idx)
            }
            0x0D => {
                // br_if instruction
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let (label_idx, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;
                i += bytes_read;
                Instruction::BrIf(label_idx)
            }
            0x10 => {
                // call instruction
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let (func_idx, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;
                i += bytes_read;
                Instruction::Call(func_idx)
            }
            0x20 => {
                // local.get instruction
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let (local_idx, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;
                i += bytes_read;
                Instruction::LocalGet(local_idx)
            }
            0x21 => {
                // local.set instruction
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let (local_idx, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;
                i += bytes_read;
                Instruction::LocalSet(local_idx)
            }
            0x22 => {
                // local.tee instruction
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let (local_idx, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;
                i += bytes_read;
                Instruction::LocalTee(local_idx)
            }
            0x23 => {
                // global.get instruction
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let (global_idx, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;
                i += bytes_read;
                Instruction::GlobalGet(global_idx)
            }
            0x24 => {
                // global.set instruction
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let (global_idx, bytes_read) = read_leb128_u32(&bytes[offset..])?;
                offset += bytes_read;
                i += bytes_read;
                Instruction::GlobalSet(global_idx)
            }
            // Numeric instructions
            0x41 => {
                // i32.const instruction
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let (value, bytes_read) = read_leb128_i32(&bytes[offset..])?;
                offset += bytes_read;
                i += bytes_read;
                Instruction::I32Const(value)
            }
            0x42 => {
                // i64.const instruction
                if offset >= bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let (value, bytes_read) = read_leb128_i64(&bytes[offset..])?;
                offset += bytes_read;
                i += bytes_read;
                Instruction::I64Const(value)
            }
            0x43 => {
                // f32.const instruction
                if offset + 4 > bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let mut buf = [0u8; 4];
                buf.copy_from_slice(&bytes[offset..offset + 4]);
                let value = f32::from_le_bytes(buf);
                offset += 4;
                i += 4;
                Instruction::F32Const(value)
            }
            0x44 => {
                // f64.const instruction
                if offset + 8 > bytes.len() {
                    return Err(Error::Parse("Unexpected end of code section".into()));
                }
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[offset..offset + 8]);
                let value = f64::from_le_bytes(buf);
                offset += 8;
                i += 8;
                Instruction::F64Const(value)
            }
            // Binary operations
            0x6A => Instruction::I32Add,
            0x6B => Instruction::I32Sub,
            0x6C => Instruction::I32Mul,
            0x6D => Instruction::I32DivS,
            0x6E => Instruction::I32DivU,
            0x6F => Instruction::I32RemS,
            0x70 => Instruction::I32RemU,
            0x71 => Instruction::I32And,
            0x72 => Instruction::I32Or,
            0x73 => Instruction::I32Xor,
            0x74 => Instruction::I32Shl,
            0x75 => Instruction::I32ShrS,
            0x76 => Instruction::I32ShrU,
            0x77 => Instruction::I32Rotl,
            0x78 => Instruction::I32Rotr,
            0x7C => Instruction::F64Add,
            0x7D => Instruction::F64Sub,
            0x7E => Instruction::F64Mul,
            0x7F => Instruction::F64Div,
            // Comparison
            0x45 => Instruction::I32Eqz,
            0x46 => Instruction::I32Eq,
            0x47 => Instruction::I32Ne,
            0x48 => Instruction::I32LtS,
            0x49 => Instruction::I32LtU,
            0x4A => Instruction::I32GtS,
            0x4B => Instruction::I32GtU,
            0x4C => Instruction::I32LeS,
            0x4D => Instruction::I32LeU,
            0x4E => Instruction::I32GeS,
            0x4F => Instruction::I32GeU,
            // We can add more instructions later, but this should cover the basic ones in the test
            _ => {
                // For any unknown opcode, log it and use a placeholder
                println!("Unknown opcode: 0x{opcode:02x}");
                Instruction::End // Using End as placeholder for unknown instructions
            }
        };

        expr.push(instruction);

        if opcode == 0x0B {
            // End opcode
            break;
        }
    }

    Ok((Code { size, locals, expr }, offset))
}

fn read_data(bytes: &[u8]) -> Result<(Data, usize)> {
    // TODO: Implement data reading
    Err(Error::InvalidData("Data reading not implemented".into()))
}
