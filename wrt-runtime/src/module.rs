// Module implementation for runtime execution
//
// This module provides the core runtime implementation of WebAssembly modules
// used by the runtime execution engine.

use wrt_foundation::{
    types::{
        CustomSection as WrtCustomSection, DataMode as WrtDataMode,
        ElementMode as WrtElementMode,
        ExportDesc as WrtExportDesc, FuncType as WrtFuncType,
        GlobalType as WrtGlobalType, ImportDesc as WrtImportDesc,
        Limits as WrtLimits, LocalEntry as WrtLocalEntry, MemoryType as WrtMemoryType,
        RefType as WrtRefType, TableType as WrtTableType, ValueType as WrtValueType,
    },
    values::Value as WrtValue,
};
use wrt_format::{
    DataSegment as WrtDataSegment,
    ElementSegment as WrtElementSegment,
};

use crate::{global::Global, memory::Memory, prelude::*, table::Table};

/// A WebAssembly expression (sequence of instructions)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WrtExpr {
    pub instructions: wrt_foundation::bounded::BoundedVec<u8, 4096, wrt_foundation::safe_memory::NoStdProvider<1024>>, // Simplified to byte sequence for now
}

/// Represents a WebAssembly export kind
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
pub struct Export {
    /// Export name
    pub name: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Export kind
    pub kind: ExportKind,
    /// Export index
    pub index: u32,
}

impl Export {
    /// Creates a new export
    pub fn new(name: String, kind: ExportKind, index: u32) -> Self {
        Self { name, kind, index }
    }
}

/// Represents an import in a WebAssembly module
#[derive(Debug, Clone)]
pub struct Import {
    /// Module name
    pub module: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Import name
    pub name: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Import type
    pub ty: ExternType<wrt_foundation::safe_memory::NoStdProvider<1024>>,
}

impl Import {
    /// Creates a new import
    pub fn new(module: String, name: String, ty: ExternType<wrt_foundation::safe_memory::NoStdProvider<1024>>) -> Self {
        Self { module, name, ty }
    }
}

impl Default for Import {
    fn default() -> Self {
        Self {
            module: wrt_foundation::bounded::BoundedString::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            name: wrt_foundation::bounded::BoundedString::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            ty: ExternType::default(),
        }
    }
}

impl PartialEq for Import {
    fn eq(&self, other: &Self) -> bool {
        self.module == other.module && self.name == other.name
    }
}

impl Eq for Import {}

impl wrt_foundation::traits::Checksummable for Import {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        self.module.update_checksum(checksum);
        self.name.update_checksum(checksum);
    }
}

impl wrt_foundation::traits::ToBytes for Import {
    fn serialized_size(&self) -> usize {
        self.module.serialized_size() + self.name.serialized_size() + 4 // simplified
    }

    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        provider: &P,
    ) -> wrt_foundation::Result<()> {
        self.module.to_bytes_with_provider(writer, provider)?;
        self.name.to_bytes_with_provider(writer, provider)
    }
}

impl wrt_foundation::traits::FromBytes for Import {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let module = wrt_foundation::bounded::BoundedString::from_bytes_with_provider(reader, provider)?;
        let name = wrt_foundation::bounded::BoundedString::from_bytes_with_provider(reader, provider)?;
        Ok(Self {
            module,
            name,
            ty: ExternType::default(), // simplified
        })
    }
}

/// Represents a WebAssembly function in the runtime
#[derive(Debug, Clone)]
pub struct Function {
    /// The type index of the function (referring to Module.types)
    pub type_idx: u32,
    /// The parsed local variable declarations
    pub locals: wrt_foundation::bounded::BoundedVec<WrtLocalEntry, 64, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// The parsed instructions that make up the function body
    pub body: WrtExpr,
}

impl Default for Function {
    fn default() -> Self {
        Self {
            type_idx: 0,
            locals: wrt_foundation::bounded::BoundedVec::new_with_provider(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            body: WrtExpr::default(),
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.type_idx == other.type_idx
    }
}

impl Eq for Function {}

impl wrt_foundation::traits::Checksummable for Function {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        checksum.update_slice(&self.type_idx.to_le_bytes());
    }
}

impl wrt_foundation::traits::ToBytes for Function {
    fn serialized_size(&self) -> usize {
        8 // simplified
    }

    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        writer.write_bytes(&self.type_idx.to_le_bytes())
    }
}

impl wrt_foundation::traits::FromBytes for Function {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 4];
        reader.read_bytes(&mut bytes)?;
        let type_idx = u32::from_le_bytes(bytes);
        Ok(Self {
            type_idx,
            locals: wrt_foundation::bounded::BoundedVec::new_with_provider(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            body: WrtExpr::default(),
        })
    }
}

/// Represents the value of an export
#[derive(Debug, Clone)]
pub enum ExportItem {
    /// A function with the specified index
    Function(u32),
    /// A table with the specified index
    Table(Arc<Table>),
    /// A memory with the specified index
    Memory(Arc<Memory>),
    /// A global with the specified index
    Global(Arc<Global>),
}

/// Represents an element segment for tables in the runtime
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Element {
    pub mode: WrtElementMode,
    pub table_idx: Option<u32>,
    pub offset_expr: Option<WrtExpr>,
    pub element_type: WrtRefType,
    pub items: wrt_foundation::bounded::BoundedVec<u32, 1024, wrt_foundation::safe_memory::NoStdProvider<1024>>,
}

impl wrt_foundation::traits::Checksummable for Element {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        checksum.update_slice(&(self.mode as u8).to_le_bytes());
        if let Some(table_idx) = self.table_idx {
            checksum.update_slice(&table_idx.to_le_bytes());
        }
    }
}

impl wrt_foundation::traits::ToBytes for Element {
    fn serialized_size(&self) -> usize {
        16 // simplified
    }

    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        writer.write_bytes(&(self.mode as u8).to_le_bytes())?;
        writer.write_bytes(&self.table_idx.unwrap_or(0).to_le_bytes())
    }
}

impl wrt_foundation::traits::FromBytes for Element {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 1];
        reader.read_bytes(&mut bytes)?;
        let mode = match bytes[0] {
            0 => WrtElementMode::Active,
            1 => WrtElementMode::Passive,
            _ => WrtElementMode::Declarative,
        };
        
        let mut idx_bytes = [0u8; 4];
        reader.read_bytes(&mut idx_bytes)?;
        let table_idx = Some(u32::from_le_bytes(idx_bytes));
        
        Ok(Self {
            mode,
            table_idx,
            offset_expr: None,
            element_type: WrtRefType::FuncRef,
            items: wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
        })
    }
}

/// Represents a data segment for memories in the runtime
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Data {
    pub mode: WrtDataMode,
    pub memory_idx: Option<u32>,
    pub offset_expr: Option<WrtExpr>,
    pub init: wrt_foundation::bounded::BoundedVec<u8, 4096, wrt_foundation::safe_memory::NoStdProvider<1024>>,
}

impl wrt_foundation::traits::Checksummable for Data {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        checksum.update_slice(&(self.mode as u8).to_le_bytes());
        if let Some(memory_idx) = self.memory_idx {
            checksum.update_slice(&memory_idx.to_le_bytes());
        }
        checksum.update_slice(&(self.init.len() as u32).to_le_bytes());
    }
}

impl wrt_foundation::traits::ToBytes for Data {
    fn serialized_size(&self) -> usize {
        16 + self.init.len() // simplified
    }

    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        writer.write_bytes(&(self.mode as u8).to_le_bytes())?;
        writer.write_bytes(&self.memory_idx.unwrap_or(0).to_le_bytes())?;
        writer.write_bytes(&(self.init.len() as u32).to_le_bytes())
    }
}

impl wrt_foundation::traits::FromBytes for Data {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 1];
        reader.read_bytes(&mut bytes)?;
        let mode = match bytes[0] {
            0 => WrtDataMode::Active,
            _ => WrtDataMode::Passive,
        };
        
        let mut idx_bytes = [0u8; 4];
        reader.read_bytes(&mut idx_bytes)?;
        let memory_idx = Some(u32::from_le_bytes(idx_bytes));
        
        reader.read_bytes(&mut idx_bytes)?;
        let _len = u32::from_le_bytes(idx_bytes);
        
        Ok(Self {
            mode,
            memory_idx,
            offset_expr: None,
            init: wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
        })
    }
}

impl Data {
    /// Returns a reference to the data in this segment
    pub fn data(&self) -> &[u8] {
        &self.init
    }
}

/// Represents a WebAssembly module in the runtime
#[derive(Debug, Clone)]
pub struct Module {
    /// Module types (function signatures)
    pub types: wrt_foundation::bounded::BoundedVec<WrtFuncType<wrt_foundation::safe_memory::NoStdProvider<1024>>, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Imported functions, tables, memories, and globals
    pub imports: wrt_format::HashMap<wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>, wrt_format::HashMap<wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>, Import>>,
    /// Function definitions
    pub functions: wrt_foundation::bounded::BoundedVec<Function, 1024, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Table instances
    pub tables: wrt_foundation::bounded::BoundedVec<Arc<Table>, 64, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Memory instances
    pub memories: wrt_foundation::bounded::BoundedVec<Arc<Memory>, 64, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Global variable instances
    pub globals: wrt_foundation::bounded::BoundedVec<Arc<Global>, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Element segments for tables
    pub elements: wrt_foundation::bounded::BoundedVec<Element, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Data segments for memories
    pub data: wrt_foundation::bounded::BoundedVec<Data, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Start function index
    pub start: Option<u32>,
    /// Custom sections
    pub custom_sections: wrt_format::HashMap<wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>, wrt_foundation::bounded::BoundedVec<u8, 4096, wrt_foundation::safe_memory::NoStdProvider<1024>>>,
    /// Exports (functions, tables, memories, and globals)
    pub exports: wrt_format::HashMap<wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>, Export>,
    /// Optional name for the module
    pub name: Option<wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>>,
    /// Original binary (if available)
    pub binary: Option<wrt_foundation::bounded::BoundedVec<u8, 65536, wrt_foundation::safe_memory::NoStdProvider<1024>>>,
    /// Execution validation flag
    pub validated: bool,
}

impl Module {
    /// Creates a new empty module
    pub fn new() -> Result<Self> {
        Ok(Self {
            types: Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            imports: HashMap::new(),
            functions: Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            tables: Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            memories: Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            globals: Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            elements: Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            data: Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            start: None,
            custom_sections: HashMap::new(),
            exports: HashMap::new(),
            name: None,
            binary: None,
            validated: false,
        })
    }

    /// Creates a runtime Module from a wrt_foundation::types::Module.
    /// This is the primary constructor after decoding.
    pub fn from_wrt_module(wrt_module: &wrt_foundation::types::Module<wrt_foundation::safe_memory::NoStdProvider<1024>>) -> Result<Self> {
        let mut runtime_module = Self::new()?;

        if let Some(name) = &wrt_module.name {
            // Assuming Module in wrt_foundation has an optional name
            runtime_module.name = Some(name.clone());
        }
        runtime_module.start = wrt_module.start;

        for type_def in &wrt_module.types {
            runtime_module.types.push(type_def.clone());
        }

        for import_def in &wrt_module.imports {
            let extern_ty = match &import_def.desc {
                WrtImportDesc::Function(type_idx) => {
                    let ft = runtime_module
                        .types
                        .get(*type_idx as usize)
                        .ok_or_else(|| {
                            Error::new(
                                ErrorCategory::Validation,
                                codes::TYPE_MISMATCH,
                                "Imported function type index out of bounds",
                            )
                        })?
                        .clone();
                    ExternType::Function(ft)
                }
                WrtImportDesc::Table(tt) => {
                    ExternType::Table(tt.clone())
                }
                WrtImportDesc::Memory(mt) => {
                    ExternType::Memory(mt.clone())
                }
                WrtImportDesc::Global(gt) => {
                    ExternType::Global(wrt_foundation::types::GlobalType {
                        value_type: gt.value_type,
                        mutable: gt.mutable,
                    })
                }
            };
            runtime_module.imports.entry(import_def.module.clone()).or_default().insert(
                import_def.name.clone(),
                crate::module::Import::new(
                    import_def.module.clone(),
                    import_def.name.clone(),
                    extern_ty,
                ),
            );
        }

        // Pre-allocate functions vector based on type indices in wrt_module.funcs
        // The actual bodies are filled by wrt_module.code_entries
        runtime_module.functions = Vec::with_capacity(wrt_module.code_entries.len());
        for code_entry in &wrt_module.code_entries {
            // Find the corresponding type_idx from wrt_module.funcs.
            // This assumes wrt_module.funcs has the type indices for functions defined in
            // this module, and wrt_module.code_entries aligns with this.
            // A direct link or combined struct in wrt_foundation::Module would be better.
            // For now, we assume that the i-th code_entry corresponds to the i-th func type
            // index in wrt_module.funcs (after accounting for imported
            // functions). This needs clarification in wrt_foundation::Module structure.
            // Let's assume wrt_module.funcs contains type indices for *defined* functions
            // and code_entries matches this.
            let func_idx_in_defined_funcs = runtime_module.functions.len(); // 0-indexed among defined functions
            if func_idx_in_defined_funcs >= wrt_module.funcs.len() {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    "Mismatch between code entries and function type declarations",
                ));
            }
            let type_idx = wrt_module.funcs[func_idx_in_defined_funcs];

            runtime_module.functions.push(Function {
                type_idx,
                locals: code_entry.locals.clone(),
                body: code_entry.body.clone(),
            });
        }

        for table_def in &wrt_module.tables {
            // For now, runtime tables are created empty and populated by element segments
            // or host. This assumes runtime::table::Table::new can take
            // WrtTableType.
            runtime_module.tables.push(Arc::new(Table::new(table_def.clone())?));
        }

        for memory_def in &wrt_module.memories {
            runtime_module.memories.push(Arc::new(Memory::new(memory_def.clone())?));
        }

        for global_def in &wrt_module.globals {
            runtime_module.globals.push(Arc::new(Global::new(
                global_def.value_type,
                global_def.mutable,
                global_def.initial_value.clone(),
            )?));
        }

        for export_def in &wrt_module.exports {
            let kind = match export_def.desc {
                WrtExportDesc::Func(_) => ExportKind::Function,
                WrtExportDesc::Table(_) => ExportKind::Table,
                WrtExportDesc::Memory(_) => ExportKind::Memory,
                WrtExportDesc::Global(_) => ExportKind::Global,
                WrtExportDesc::Tag(_) => {
                    return Err(Error::new(
                        ErrorCategory::NotSupported,
                        codes::UNSUPPORTED_OPERATION,
                        "Tag exports not supported",
                    ))
                }
            };
            runtime_module.exports.insert(
                export_def.name.clone(),
                crate::module::Export::new(export_def.name.clone(), kind, export_def.desc.index()),
            );
        }

        for element_def in &wrt_module.elements {
            // This requires significant processing to evaluate offset_expr and items
            // expressions For now, store a simplified version or one that
            // requires instantiation-time evaluation. This is a placeholder and
            // needs robust implementation.
            // TODO: ElementItems type not available yet, using empty items for now
            #[cfg(any(feature = "std", feature = "alloc"))]
            let items_resolved = vec![];
            #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
            let items_resolved = wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap();
            runtime_module.elements.push(crate::module::Element {
                mode: element_def.mode.clone(),
                table_idx: element_def.table_idx,
                offset_expr: element_def.offset_expr.clone(), /* Store expression for later
                                                               * evaluation */
                element_type: element_def.element_type,
                items: items_resolved, // Store resolved/placeholder items
            });
        }

        for data_def in &wrt_module.data_segments {
            runtime_module.data.push(crate::module::Data {
                mode: data_def.mode.clone(),
                memory_idx: data_def.memory_idx,
                offset_expr: data_def.offset_expr.clone(), // Store expression for later evaluation
                init: data_def.data.clone(),
            });
        }

        for custom_def in &wrt_module.custom_sections {
            runtime_module.custom_sections.insert(custom_def.name.clone(), custom_def.data.clone());
        }

        Ok(runtime_module)
    }

    /// Gets an export by name
    pub fn get_export(&self, name: &str) -> Option<&Export> {
        self.exports.get(name)
    }

    /// Gets a function by index
    pub fn get_function(&self, idx: u32) -> Option<&Function> {
        if idx as usize >= self.functions.len() {
            return None;
        }
        Some(&self.functions[idx as usize])
    }

    /// Gets a function type by index
    pub fn get_function_type(&self, idx: u32) -> Option<&WrtFuncType<wrt_foundation::safe_memory::NoStdProvider<1024>>> {
        if idx as usize >= self.types.len() {
            return None;
        }
        Some(&self.types[idx as usize])
    }

    /// Gets a global by index
    pub fn get_global(&self, idx: usize) -> Result<Arc<Global>> {
        self.globals.get(idx).cloned().ok_or_else(|| {
            Error::new(
                ErrorCategory::Runtime,
                codes::GLOBAL_NOT_FOUND,
                format!("Global at index {} not found", idx),
            )
        })
    }

    /// Gets a memory by index
    pub fn get_memory(&self, idx: usize) -> Result<Arc<Memory>> {
        self.memories.get(idx).cloned().ok_or_else(|| {
            Error::new(
                ErrorCategory::Runtime,
                codes::MEMORY_NOT_FOUND,
                format!("Memory at index {} not found", idx),
            )
        })
    }

    /// Gets a table by index
    pub fn get_table(&self, idx: usize) -> Result<Arc<Table>> {
        self.tables.get(idx).cloned().ok_or_else(|| {
            Error::new(
                ErrorCategory::Runtime,
                codes::TABLE_NOT_FOUND,
                format!("Table at index {} not found", idx),
            )
        })
    }

    /// Adds a function export
    pub fn add_function_export(&mut self, name: String, index: u32) {
        self.exports.insert(name.clone(), Export::new(name, ExportKind::Function, index));
    }

    /// Adds a table export
    pub fn add_table_export(&mut self, name: String, index: u32) {
        self.exports.insert(name.clone(), Export::new(name, ExportKind::Table, index));
    }

    /// Adds a memory export
    pub fn add_memory_export(&mut self, name: String, index: u32) {
        self.exports.insert(name.clone(), Export::new(name, ExportKind::Memory, index));
    }

    /// Adds a global export
    pub fn add_global_export(&mut self, name: String, index: u32) {
        self.exports.insert(name.clone(), Export::new(name, ExportKind::Global, index));
    }

    /// Adds an export to the module from a wrt_format::module::Export
    pub fn add_export(&mut self, format_export: wrt_format::module::Export) -> Result<()> {
        let runtime_export_kind = match format_export.kind {
            wrt_format::module::ExportKind::Function => ExportKind::Function,
            wrt_format::module::ExportKind::Table => ExportKind::Table,
            wrt_format::module::ExportKind::Memory => ExportKind::Memory,
            wrt_format::module::ExportKind::Global => ExportKind::Global,
        };
        let runtime_export = Export {
            name: format_export.name,
            kind: runtime_export_kind,
            index: format_export.index,
        };
        self.exports.insert(runtime_export.name.clone(), runtime_export);
        Ok(())
    }

    /// Set the name of the module
    pub fn set_name(&mut self, name: String) -> Result<()> {
        self.name = Some(name);
        Ok(())
    }

    /// Set the start function index
    pub fn set_start(&mut self, start: u32) -> Result<()> {
        self.start = Some(start);
        Ok(())
    }

    /// Add a function type to the module
    pub fn add_type(&mut self, ty: WrtFuncType<wrt_foundation::safe_memory::NoStdProvider<1024>>) -> Result<()> {
        self.types.push(ty);
        Ok(())
    }

    /// Add a function import to the module
    pub fn add_import_func(
        &mut self,
        module_name: &str,
        item_name: &str,
        type_idx: u32,
    ) -> Result<()> {
        let func_type = self
            .types
            .get(type_idx as usize)
            .ok_or_else(|| {
                Error::new(
                    ErrorCategory::Validation,
                    codes::TYPE_MISMATCH,
                    "Type index out of bounds for import func",
                )
            })?
            .clone();

        let import_struct = crate::module::Import::new(
            module_name.to_string(),
            item_name.to_string(),
            ExternType::Function(func_type),
        );
        self.imports
            .entry(module_name.to_string())
            .or_default()
            .insert(item_name.to_string(), import_struct);
        Ok(())
    }

    /// Adds an imported table to the module
    pub fn add_import_table(
        &mut self,
        module_name: &str,
        item_name: &str,
        table_type: WrtTableType,
    ) -> Result<()> {
        let import_struct = crate::module::Import::new(
            module_name.to_string(),
            item_name.to_string(),
            ExternType::Table(table_type),
        );
        self.imports
            .entry(module_name.to_string())
            .or_default()
            .insert(item_name.to_string(), import_struct);
        Ok(())
    }

    /// Adds an imported memory to the module
    pub fn add_import_memory(
        &mut self,
        module_name: &str,
        item_name: &str,
        memory_type: WrtMemoryType,
    ) -> Result<()> {
        let import_struct = crate::module::Import::new(
            module_name.to_string(),
            item_name.to_string(),
            ExternType::Memory(memory_type),
        );
        self.imports
            .entry(module_name.to_string())
            .or_default()
            .insert(item_name.to_string(), import_struct);
        Ok(())
    }

    /// Adds an imported global to the module
    pub fn add_import_global(
        &mut self,
        module_name: &str,
        item_name: &str,
        format_global: wrt_format::module::Global,
    ) -> Result<()> {
        let component_global_type = wrt_foundation::types::GlobalType {
            value_type: format_global.global_type.value_type,
            mutable: format_global.global_type.mutable,
        };

        let import = Import::new(
            module_name.to_string(),
            item_name.to_string(),
            ExternType::Global(component_global_type),
        );

        self.imports
            .entry(module_name.to_string())
            .or_default()
            .insert(item_name.to_string(), import);
        Ok(())
    }

    /// Add a function to the module
    pub fn add_function_type(&mut self, type_idx: u32) -> Result<()> {
        if type_idx as usize >= self.types.len() {
            return Err(Error::from(kinds::ValidationError(format!(
                "Function type index {} out of bounds (max {})",
                type_idx,
                self.types.len()
            ))));
        }

        let function = Function { type_idx, locals: Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(), body: WrtExpr::default() };

        self.functions.push(function);
        Ok(())
    }

    /// Add a table to the module
    pub fn add_table(&mut self, table_type: WrtTableType) -> Result<()> {
        self.tables.push(Arc::new(Table::new(table_type)?));
        Ok(())
    }

    /// Add a memory to the module
    pub fn add_memory(&mut self, memory_type: WrtMemoryType) -> Result<()> {
        self.memories.push(Arc::new(Memory::new(memory_type)?));
        Ok(())
    }

    /// Add a global to the module
    pub fn add_global(&mut self, global_type: WrtGlobalType, init: WrtValue) -> Result<()> {
        let global = Global::new(global_type.value_type, global_type.mutable, init)?;
        self.globals.push(Arc::new(global));
        Ok(())
    }

    /// Add a function export to the module
    pub fn add_export_func(&mut self, name: &str, index: u32) -> Result<()> {
        if index as usize >= self.functions.len() {
            return Err(Error::validation_error(format!(
                "Export function index {} out of bounds",
                index
            )));
        }

        let export = Export { name: name.to_string(), kind: ExportKind::Function, index };

        self.exports.insert(name.to_string(), export);
        Ok(())
    }

    /// Add a table export to the module
    pub fn add_export_table(&mut self, name: &str, index: u32) -> Result<()> {
        if index as usize >= self.tables.len() {
            return Err(Error::validation_error(format!(
                "Export table index {} out of bounds",
                index
            )));
        }

        let export = Export { name: name.to_string(), kind: ExportKind::Table, index };

        self.exports.insert(name.to_string(), export);
        Ok(())
    }

    /// Add a memory export to the module
    pub fn add_export_memory(&mut self, name: &str, index: u32) -> Result<()> {
        if index as usize >= self.memories.len() {
            return Err(Error::validation_error(format!(
                "Export memory index {} out of bounds",
                index
            )));
        }

        let export = Export { name: name.to_string(), kind: ExportKind::Memory, index };

        self.exports.insert(name.to_string(), export);
        Ok(())
    }

    /// Add a global export to the module
    pub fn add_export_global(&mut self, name: &str, index: u32) -> Result<()> {
        if index as usize >= self.globals.len() {
            return Err(Error::validation_error(format!(
                "Export global index {} out of bounds",
                index
            )));
        }

        let export = Export { name: name.to_string(), kind: ExportKind::Global, index };

        self.exports.insert(name.to_string(), export);
        Ok(())
    }

    /// Add an element segment to the module
    pub fn add_element(&mut self, element: wrt_format::module::Element) -> Result<()> {
        // Convert format element to runtime element
        let items = match &element.init {
            wrt_format::module::ElementInit::Passive => {
                // For passive elements, create empty items list
                wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default())?
            }
            wrt_format::module::ElementInit::Active { func_indices, .. } => {
                // For active elements, copy the function indices
                let mut bounded_items = wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default())?;
                for &idx in func_indices {
                    bounded_items.push(idx)?;
                }
                bounded_items
            }
            wrt_format::module::ElementInit::Declarative => {
                // For declarative elements, create empty items list
                wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default())?
            }
        };
        
        let runtime_element = crate::module::Element {
            mode: WrtElementMode::Active, // Default mode, should be determined from element.init
            table_idx: element.table_idx,
            offset_expr: None, // Would need to convert from element.offset
            element_type: WrtRefType::FuncRef, // Default type
            items,
        };

        self.elements.push(runtime_element);
        Ok(())
    }

    /// Set a function body
    pub fn set_function_body(
        &mut self,
        func_idx: u32,
        type_idx: u32,
        locals: Vec<WrtLocalEntry>,
        body: WrtExpr,
    ) -> Result<()> {
        if func_idx as usize > self.functions.len() {
            // Allow appending
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::FUNCTION_NOT_FOUND,
                "Function index out of bounds for set_function_body",
            ));
        }
        let func_entry = Function { type_idx, locals, body };
        if func_idx as usize == self.functions.len() {
            self.functions.push(func_entry);
        } else {
            self.functions[func_idx as usize] = func_entry;
        }
        Ok(())
    }

    /// Add a data segment to the module
    pub fn add_data(&mut self, data: wrt_format::module::Data) -> Result<()> {
        // Convert format data to runtime data
        let mut init_4096 = wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default())?;
        
        // Copy data from the format's init (1024 capacity) to runtime's init (4096 capacity)
        for byte in data.init.iter() {
            init_4096.push(*byte)?;
        }
        
        let runtime_data = crate::module::Data {
            mode: WrtDataMode::Active, // Default mode
            memory_idx: data.memory_idx,
            offset_expr: None, // Would need to convert from data.offset
            init: init_4096,
        };

        self.data.push(runtime_data);
        Ok(())
    }

    /// Add a custom section to the module
    pub fn add_custom_section(&mut self, name: &str, data: Vec<u8>) -> Result<()> {
        self.custom_sections.insert(name.to_string(), data);
        Ok(())
    }

    /// Set the binary representation of the module
    pub fn set_binary(&mut self, binary: Vec<u8>) -> Result<()> {
        self.binary = Some(binary);
        Ok(())
    }

    /// Validate the module
    pub fn validate(&self) -> Result<()> {
        // TODO: Implement comprehensive validation of the runtime module structure.
        // - Check type indices are valid.
        // - Check function indices in start/exports/elements are valid.
        // - Check table/memory/global indices.
        // - Validate instruction sequences in function bodies (optional, decoder should
        //   do most of this).
        Ok(())
    }

    /// Add an import runtime global to the module
    pub fn add_import_runtime_global(
        &mut self,
        module_name: &str,
        item_name: &str,
        global_type: WrtGlobalType,
    ) -> Result<()> {
        let component_global_type = wrt_foundation::types::GlobalType {
            value_type: global_type.value_type,
            mutable: global_type.mutable,
        };
        let import_struct = crate::module::Import::new(
            module_name.to_string(),
            item_name.to_string(),
            ExternType::Global(component_global_type),
        );
        self.imports
            .entry(module_name.to_string())
            .or_default()
            .insert(item_name.to_string(), import_struct);
        Ok(())
    }

    /// Add a runtime export to the module
    pub fn add_runtime_export(&mut self, name: String, export_desc: WrtExportDesc) -> Result<()> {
        let kind = match export_desc {
            WrtExportDesc::Func(_) => ExportKind::Function,
            WrtExportDesc::Table(_) => ExportKind::Table,
            WrtExportDesc::Memory(_) => ExportKind::Memory,
            WrtExportDesc::Global(_) => ExportKind::Global,
            WrtExportDesc::Tag(_) => {
                return Err(Error::new(
                    ErrorCategory::NotSupported,
                    codes::UNSUPPORTED_OPERATION,
                    "Tag exports not supported",
                ))
            }
        };
        let runtime_export = crate::module::Export::new(name.clone(), kind, export_desc.index());
        self.exports.insert(name, runtime_export);
        Ok(())
    }

    /// Add a runtime element to the module
    pub fn add_runtime_element(&mut self, element_segment: WrtElementSegment) -> Result<()> {
        // TODO: Resolve element_segment.items expressions if they are not direct
        // indices. This is a placeholder and assumes items can be derived or
        // handled during instantiation.
        // TODO: ElementItems type not available yet, using empty items for now
        #[cfg(any(feature = "std", feature = "alloc"))]
        let items_resolved = vec![];
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        let items_resolved = wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap();

        self.elements.push(crate::module::Element {
            mode: element_segment.mode,
            table_idx: element_segment.table_idx,
            offset_expr: element_segment.offset_expr,
            element_type: element_segment.element_type,
            items: items_resolved,
        });
        Ok(())
    }

    /// Add a runtime data segment to the module  
    pub fn add_runtime_data(&mut self, data_segment: WrtDataSegment) -> Result<()> {
        self.data.push(crate::module::Data {
            mode: data_segment.mode,
            memory_idx: data_segment.memory_idx,
            offset_expr: data_segment.offset_expr,
            init: data_segment.data,
        });
        Ok(())
    }

    /// Add a custom section to the module
    pub fn add_custom_section_runtime(&mut self, section: WrtCustomSection<wrt_foundation::safe_memory::NoStdProvider<1024>>) -> Result<()> {
        self.custom_sections.insert(section.name, section.data);
        Ok(())
    }

    /// Set the binary representation of the module (alternative method)
    pub fn set_binary_runtime(&mut self, binary: Vec<u8>) -> Result<()> {
        self.binary = Some(binary);
        Ok(())
    }
}

/// Additional exports that are not part of the standard WebAssembly exports
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherExport {
    /// Export name
    pub name: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Export kind
    pub kind: ExportKind,
    /// Export index
    pub index: u32,
}

/// Represents an imported item in a WebAssembly module
#[derive(Debug, Clone)]
pub enum ImportedItem {
    /// An imported function
    Function {
        /// The module name
        module: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
        /// The function name
        name: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
        /// The function type
        ty: FuncType<wrt_foundation::safe_memory::NoStdProvider<1024>>,
    },
    /// An imported table
    Table {
        /// The module name
        module: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
        /// The table name
        name: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
        /// The table type
        ty: WrtTableType,
    },
    /// An imported memory
    Memory {
        /// The module name
        module: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
        /// The memory name
        name: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
        /// The memory type
        ty: WrtMemoryType,
    },
    /// An imported global
    Global {
        /// The module name
        module: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
        /// The global name
        name: wrt_foundation::bounded::BoundedString<128, wrt_foundation::safe_memory::NoStdProvider<1024>>,
        /// The global type
        ty: WrtGlobalType,
    },
}


// Ensure ExternType is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::collections::BTreeMap as HashMap; // For BTreeMap in Module struct
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::sync::Arc; // For Arc<Table/Memory/Global>
#[cfg(feature = "std")]
use std::collections::BTreeMap as HashMap; // Use BTreeMap for consistency
#[cfg(feature = "std")]
use std::sync::Arc; // For Arc<Table/Memory/Global>

use wrt_error::{codes, Error, ErrorCategory, Result};
use wrt_foundation::component::ExternType; // For error handling

// Trait implementations for Arc<T> wrappers needed by bounded collections

impl Default for Arc<Table> {
    fn default() -> Self {
        use wrt_foundation::types::{Limits, TableType, ValueType};
        let table_type = TableType {
            element_type: ValueType::FuncRef,
            limits: Limits { min: 0, max: Some(1) },
        };
        Arc::new(Table::new(table_type).unwrap())
    }
}


impl wrt_foundation::traits::Checksummable for Arc<Table> {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        // Use table ID or size for checksum
        checksum.update_slice(&0u32.to_le_bytes()); // simplified
    }
}

impl wrt_foundation::traits::ToBytes for Arc<Table> {
    fn serialized_size(&self) -> usize {
        8 // simplified
    }

    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        writer.write_bytes(&0u32.to_le_bytes()) // simplified
    }
}

impl wrt_foundation::traits::FromBytes for Arc<Table> {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 4];
        reader.read_bytes(&mut bytes)?;
        Ok(Self::default()) // simplified
    }
}

// Trait implementations for Arc<Memory>

impl Default for Arc<Memory> {
    fn default() -> Self {
        use wrt_foundation::types::{Limits, MemoryType};
        let memory_type = MemoryType {
            limits: Limits { min: 1, max: Some(1) },
        };
        Arc::new(Memory::new(memory_type).unwrap())
    }
}


impl wrt_foundation::traits::Checksummable for Arc<Memory> {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        checksum.update_slice(&0u32.to_le_bytes()); // simplified
    }
}

impl wrt_foundation::traits::ToBytes for Arc<Memory> {
    fn serialized_size(&self) -> usize {
        8 // simplified
    }

    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        writer.write_bytes(&0u32.to_le_bytes()) // simplified
    }
}

impl wrt_foundation::traits::FromBytes for Arc<Memory> {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 4];
        reader.read_bytes(&mut bytes)?;
        Ok(Self::default()) // simplified
    }
}

// Trait implementations for Arc<Global>

impl Default for Arc<Global> {
    fn default() -> Self {
        use wrt_foundation::types::{GlobalType, ValueType};
        use wrt_foundation::values::Value;
        let global_type = GlobalType {
            value_type: ValueType::I32,
            mutable: false,
        };
        Arc::new(Global::new(global_type.value_type, global_type.mutable, Value::I32(0)).unwrap())
    }
}


impl wrt_foundation::traits::Checksummable for Arc<Global> {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        checksum.update_slice(&0u32.to_le_bytes()); // simplified
    }
}

impl wrt_foundation::traits::ToBytes for Arc<Global> {
    fn serialized_size(&self) -> usize {
        8 // simplified
    }

    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        writer.write_bytes(&0u32.to_le_bytes()) // simplified
    }
}

impl wrt_foundation::traits::FromBytes for Arc<Global> {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 4];
        reader.read_bytes(&mut bytes)?;
        Ok(Self::default()) // simplified
    }
}

// Ensure local `crate::module::Import` struct is defined
// Ensure local `crate::module::Export` struct is defined
// Ensure local `crate::global::Global`, `crate::table::Table`,
// `crate::memory::Memory` are defined and their `new` methods are compatible.

