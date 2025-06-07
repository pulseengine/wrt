// Module implementation for runtime execution
//
// This module provides the core runtime implementation of WebAssembly modules
// used by the runtime execution engine.

// Binary std/no_std choice - use our own memory management
#[cfg(feature = "std")]
extern crate alloc;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportKind {
    /// Function export
    #[default]
    Function,
    /// Table export
    Table,
    /// Memory export
    Memory,
    /// Global export
    Global,
}

/// Represents an export in a WebAssembly module
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
    pub fn new(name: String, kind: ExportKind, index: u32) -> Result<Self> {
        let bounded_name = wrt_foundation::bounded::BoundedString::from_str_truncate(
            name.as_str()?,
            wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
        )?;
        Ok(Self { name: bounded_name, kind, index })
    }
}

impl wrt_foundation::traits::Checksummable for Export {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        self.name.update_checksum(checksum);
        checksum.update_slice(&(self.kind.clone() as u8).to_le_bytes());
        checksum.update_slice(&self.index.to_le_bytes());
    }
}

impl wrt_foundation::traits::ToBytes for Export {
    fn serialized_size(&self) -> usize {
        self.name.serialized_size() + 1 + 4 // name + kind (1 byte) + index (4 bytes)
    }

    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        provider: &P,
    ) -> wrt_foundation::Result<()> {
        self.name.to_bytes_with_provider(writer, provider)?;
        writer.write_all(&(self.kind.clone() as u8).to_le_bytes())?;
        writer.write_all(&self.index.to_le_bytes())
    }
}

impl wrt_foundation::traits::FromBytes for Export {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let name = wrt_foundation::bounded::BoundedString::from_bytes_with_provider(reader, provider)?;
        
        let mut kind_bytes = [0u8; 1];
        reader.read_exact(&mut kind_bytes)?;
        let kind = match kind_bytes[0] {
            0 => ExportKind::Function,
            1 => ExportKind::Table,
            2 => ExportKind::Memory,
            3 => ExportKind::Global,
            _ => ExportKind::Function, // Default fallback
        };
        
        let mut index_bytes = [0u8; 4];
        reader.read_exact(&mut index_bytes)?;
        let index = u32::from_le_bytes(index_bytes);
        
        Ok(Self { name, kind, index })
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
    pub fn new(module: String, name: String, ty: ExternType<wrt_foundation::safe_memory::NoStdProvider<1024>>) -> Result<Self> {
        let bounded_module = wrt_foundation::bounded::BoundedString::from_str_truncate(
            module.as_str()?,
            wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
        )?;
        let bounded_name = wrt_foundation::bounded::BoundedString::from_str_truncate(
            name.as_str()?,
            wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
        )?;
        Ok(Self { module: bounded_module, name: bounded_name, ty })
    }
}

impl Default for Import {
    fn default() -> Self {
        Self {
            module: wrt_foundation::bounded::BoundedString::from_str_truncate("", wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
            name: wrt_foundation::bounded::BoundedString::from_str_truncate("", wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
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
            locals: wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
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
        writer.write_all(&self.type_idx.to_le_bytes())
    }
}

impl wrt_foundation::traits::FromBytes for Function {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 4];
        reader.read_exact(&mut bytes)?;
        let type_idx = u32::from_le_bytes(bytes);
        Ok(Self {
            type_idx,
            locals: wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap(),
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
    Table(TableWrapper),
    /// A memory with the specified index
    Memory(MemoryWrapper),
    /// A global with the specified index
    Global(GlobalWrapper),
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
        let mode_byte = match &self.mode {
            WrtElementMode::Active { .. } => 0u8,
            WrtElementMode::Passive => 1u8,
            WrtElementMode::Declarative => 2u8,
        };
        checksum.update_slice(&mode_byte.to_le_bytes());
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
        let mode_byte = match &self.mode {
            WrtElementMode::Active { .. } => 0u8,
            WrtElementMode::Passive => 1u8,
            WrtElementMode::Declarative => 2u8,
        };
        writer.write_all(&mode_byte.to_le_bytes())?;
        writer.write_all(&self.table_idx.unwrap_or(0).to_le_bytes())
    }
}

impl wrt_foundation::traits::FromBytes for Element {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 1];
        reader.read_exact(&mut bytes)?;
        let mode = match bytes[0] {
            0 => WrtElementMode::Active { table_index: 0, offset: 0 },
            1 => WrtElementMode::Passive,
            _ => WrtElementMode::Declarative,
        };
        
        let mut idx_bytes = [0u8; 4];
        reader.read_exact(&mut idx_bytes)?;
        let table_idx = Some(u32::from_le_bytes(idx_bytes));
        
        Ok(Self {
            mode,
            table_idx,
            offset_expr: None,
            element_type: WrtRefType::Funcref,
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
        let mode_byte = match &self.mode {
            WrtDataMode::Active { .. } => 0u8,
            WrtDataMode::Passive => 1u8,
        };
        checksum.update_slice(&mode_byte.to_le_bytes());
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
        let mode_byte = match &self.mode {
            WrtDataMode::Active { .. } => 0u8,
            WrtDataMode::Passive => 1u8,
        };
        writer.write_all(&mode_byte.to_le_bytes())?;
        writer.write_all(&self.memory_idx.unwrap_or(0).to_le_bytes())?;
        writer.write_all(&(self.init.len() as u32).to_le_bytes())
    }
}

impl wrt_foundation::traits::FromBytes for Data {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 1];
        reader.read_exact(&mut bytes)?;
        let mode = match bytes[0] {
            0 => WrtDataMode::Active { memory_index: 0, offset: 0 },
            _ => WrtDataMode::Passive,
        };
        
        let mut idx_bytes = [0u8; 4];
        reader.read_exact(&mut idx_bytes)?;
        let memory_idx = Some(u32::from_le_bytes(idx_bytes));
        
        reader.read_exact(&mut idx_bytes)?;
        let _len = u32::from_le_bytes(idx_bytes);
        
        Ok(Self {
            mode,
            memory_idx,
            offset_expr: None,
            init: wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default())?,
        })
    }
}

impl Data {
    /// Returns a reference to the data in this segment
    pub fn data(&self) -> &[u8] {
        self.init.as_slice()
    }
}

/// Represents a WebAssembly module in the runtime
#[derive(Debug, Clone)]
pub struct Module {
    /// Module types (function signatures)
    pub types: wrt_foundation::bounded::BoundedVec<WrtFuncType<wrt_foundation::safe_memory::NoStdProvider<1024>>, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Imported functions, tables, memories, and globals
    #[cfg(feature = "std")]
    pub imports: HashMap<String, HashMap<String, Import>>,
    #[cfg(not(feature = "std"))]
    pub imports: HashMap<String, HashMap<String, Import>>,
    /// Function definitions
    pub functions: wrt_foundation::bounded::BoundedVec<Function, 1024, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Table instances
    pub tables: wrt_foundation::bounded::BoundedVec<TableWrapper, 64, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Memory instances
    pub memories: wrt_foundation::bounded::BoundedVec<MemoryWrapper, 64, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Global variable instances
    pub globals: wrt_foundation::bounded::BoundedVec<GlobalWrapper, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Element segments for tables
    pub elements: wrt_foundation::bounded::BoundedVec<Element, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Data segments for memories
    pub data: wrt_foundation::bounded::BoundedVec<Data, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// Start function index
    pub start: Option<u32>,
    /// Custom sections
    #[cfg(feature = "std")]
    pub custom_sections: HashMap<String, wrt_foundation::bounded::BoundedVec<u8, 4096, wrt_foundation::safe_memory::NoStdProvider<1024>>>,
    #[cfg(not(feature = "std"))]
    pub custom_sections: HashMap<String, Vec<u8>>,
    /// Exports (functions, tables, memories, and globals)
    #[cfg(feature = "std")]
    pub exports: HashMap<String, Export>,
    #[cfg(not(feature = "std"))]
    pub exports: HashMap<String, Export>,
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
        let provider = wrt_foundation::safe_memory::NoStdProvider::<1024>::default();
        Ok(Self {
            types: wrt_foundation::bounded::BoundedVec::new(provider.clone())?,
            #[cfg(feature = "std")]
            imports: HashMap::new(),
            #[cfg(not(feature = "std"))]
            imports: HashMap::new(),
            functions: wrt_foundation::bounded::BoundedVec::new(provider.clone())?,
            tables: wrt_foundation::bounded::BoundedVec::new(provider.clone())?,
            memories: wrt_foundation::bounded::BoundedVec::new(provider.clone())?,
            globals: wrt_foundation::bounded::BoundedVec::new(provider.clone())?,
            elements: wrt_foundation::bounded::BoundedVec::new(provider.clone())?,
            data: wrt_foundation::bounded::BoundedVec::new(provider.clone())?,
            start: None,
            #[cfg(feature = "std")]
            custom_sections: HashMap::new(),
            #[cfg(not(feature = "std"))]
            custom_sections: HashMap::new(),
            #[cfg(feature = "std")]
            exports: HashMap::new(),
            #[cfg(not(feature = "std"))]
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

        // TODO: wrt_module doesn't have a name field currently
        // if let Some(name) = &wrt_module.name {
        //     runtime_module.name = Some(name.clone());
        // }
        // Map start function if present
        runtime_module.start = wrt_module.start_func;

        for type_def in &wrt_module.types {
            runtime_module.types.push(type_def.clone());
        }

        for import_def in &wrt_module.imports {
            let extern_ty = match &import_def.desc {
                WrtImportDesc::Function(type_idx) => {
                    let ft = runtime_module
                        .types
                        .get(*type_idx as usize)
                        .map_err(|_| {
                            Error::new(
                                ErrorCategory::Validation,
                                codes::TYPE_MISMATCH,
                                "Imported function type index out of bounds",
                            )
                        })?
                        .clone();
                    ExternType::Func(ft)
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
            let import = crate::module::Import::new(
                import_def.module_name.as_str()?.to_string(),
                import_def.item_name.as_str()?.to_string(),
                extern_ty,
            )?;
            #[cfg(feature = "std")]
            {
                let module_key = import_def.module_name.as_str()?.to_string();
                let name_key = import_def.item_name.as_str()?.to_string();
                runtime_module.imports.entry(module_key).or_default().insert(
                    name_key,
                    import,
                );
            }
            #[cfg(not(feature = "std"))]
            {
                let module_key = wrt_foundation::bounded::BoundedString::from_str_truncate(
                    import_def.module_name.as_str()?,
                    wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
                )?;
                let name_key = wrt_foundation::bounded::BoundedString::from_str_truncate(
                    import_def.item_name.as_str()?,
                    wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
                )?;
                if !runtime_module.imports.contains_key(&module_key) {
                    runtime_module.imports.insert(module_key.clone(), HashMap::new());
                }
                if let Some(module_map) = runtime_module.imports.get_mut(&module_key) {
                    module_map.insert(name_key, import)?;
                }
            }
        }

        // Binary std/no_std choice
        // The actual bodies are filled by wrt_module.code_entries
        // Clear existing functions and prepare for new ones
        for code_entry in &wrt_module.func_bodies {
            // Find the corresponding type_idx from wrt_module.functions.
            // This assumes wrt_module.functions has the type indices for functions defined in
            // this module, and wrt_module.code_entries aligns with this.
            // A direct link or combined struct in wrt_foundation::Module would be better.
            // For now, we assume that the i-th code_entry corresponds to the i-th func type
            // index in wrt_module.functions (after accounting for imported
            // functions). This needs clarification in wrt_foundation::Module structure.
            // Let's assume wrt_module.functions contains type indices for *defined* functions
            // and code_entries matches this.
            let func_idx_in_defined_funcs = runtime_module.functions.len(); // 0-indexed among defined functions
            if func_idx_in_defined_funcs >= wrt_module.functions.len() {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    "Mismatch between code entries and function type declarations",
                ));
            }
            let type_idx = wrt_module.functions.get(func_idx_in_defined_funcs).map_err(|_| Error::new(ErrorCategory::Validation, codes::FUNCTION_NOT_FOUND, "Function index out of bounds"))?;

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
            runtime_module.tables.push(TableWrapper::new(Table::new(table_def.clone())?));
        }

        for memory_def in &wrt_module.memories {
            runtime_module.memories.push(MemoryWrapper::new(Memory::new(memory_def.clone())?));
        }

        for global_def in &wrt_module.globals {
            runtime_module.globals.push(GlobalWrapper::new(Global::new(
                global_def.value_type,
                global_def.mutable,
                global_def.initial_value.clone(),
            )?));
        }

        for export_def in &wrt_module.exports {
            let (kind, index) = match export_def.desc {
                WrtExportDesc::Func(idx) => (ExportKind::Function, idx),
                WrtExportDesc::Table(idx) => (ExportKind::Table, idx),
                WrtExportDesc::Memory(idx) => (ExportKind::Memory, idx),
                WrtExportDesc::Global(idx) => (ExportKind::Global, idx),
                WrtExportDesc::Tag(_) => {
                    return Err(Error::new(
                        ErrorCategory::NotSupported,
                        codes::UNSUPPORTED_OPERATION,
                        "Tag exports not supported",
                    ))
                }
            };
            let export = crate::module::Export::new(export_def.name.as_str().to_string(), kind, index)?;
            #[cfg(feature = "std")]
            {
                let name_key = export_def.name.as_str().to_string();
                runtime_module.exports.insert(name_key, export);
            }
            #[cfg(not(feature = "std"))]
            {
                let name_key = wrt_foundation::bounded::BoundedString::from_str_truncate(
                    export_def.name.as_str(),
                    wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
                )?;
                runtime_module.exports.insert(name_key, export)?;
            }
        }

        for element_def in &wrt_module.elements {
            // This requires significant processing to evaluate offset_expr and items
            // expressions For now, store a simplified version or one that
            // requires instantiation-time evaluation. This is a placeholder and
            // needs robust implementation.
            // TODO: ElementItems type not available yet, using empty items for now
            #[cfg(feature = "std")]
            let items_resolved = vec![];
            #[cfg(all(not(feature = "std"), not(feature = "std")))]
            let items_resolved = wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default())?;
            runtime_module.elements.push(crate::module::Element {
                mode: element_def.mode.clone(),
                table_idx: element_def.table_idx,
                offset_expr: element_def.offset_expr.clone(), /* Store expression for later
                                                               * evaluation */
                element_type: element_def.element_type,
                items: items_resolved, // Store resolved/placeholder items
            });
        }

        for data_def in &wrt_module.data {
            runtime_module.data.push(crate::module::Data {
                mode: data_def.mode.clone(),
                memory_idx: data_def.memory_idx,
                offset_expr: data_def.offset_expr.clone(), // Store expression for later evaluation
                init: data_def.data.clone(),
            });
        }

        for custom_def in &wrt_module.custom_sections {
            #[cfg(feature = "std")]
            {
                let name_key = custom_def.name.as_str().to_string();
                runtime_module.custom_sections.insert(name_key, custom_def.data.clone());
            }
            #[cfg(not(feature = "std"))]
            {
                let name_key = wrt_foundation::bounded::BoundedString::from_str_truncate(
                    custom_def.name.as_str(),
                    wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
                )?;
                runtime_module.custom_sections.insert(name_key, custom_def.data.clone())?;
            }
        }

        Ok(runtime_module)
    }

    /// Gets an export by name
    pub fn get_export(&self, name: &str) -> Option<&Export> {
        #[cfg(feature = "std")]
        {
            self.exports.get(name)
        }
        #[cfg(not(feature = "std"))]
        {
            // BoundedHashMap requires exact key type match - search manually
            for (key, value) in self.exports.iter() {
                if key.as_str() == name {
                    return Some(value);
                }
            }
            None
        }
    }

    /// Gets a function by index
    pub fn get_function(&self, idx: u32) -> Option<&Function> {
        if idx as usize >= self.functions.len() {
            return None;
        }
        self.functions.get(idx as usize).ok()
    }

    /// Gets a function type by index
    pub fn get_function_type(&self, idx: u32) -> Option<&WrtFuncType<wrt_foundation::safe_memory::NoStdProvider<1024>>> {
        if idx as usize >= self.types.len() {
            return None;
        }
        self.types.get(idx as usize)
    }

    /// Gets a global by index
    pub fn get_global(&self, idx: usize) -> Result<GlobalWrapper> {
        self.globals.get(idx).map(|global| global.clone()).map_err(|_| {
            Error::new(
                ErrorCategory::Runtime,
                codes::GLOBAL_NOT_FOUND,
                "Runtime operation error",
            )
        })
    }

    /// Gets a memory by index
    pub fn get_memory(&self, idx: usize) -> Result<MemoryWrapper> {
        self.memories.get(idx).map(|memory| memory.clone()).map_err(|_| {
            Error::new(
                ErrorCategory::Runtime,
                codes::MEMORY_NOT_FOUND,
                "Runtime operation error",
            )
        })
    }

    /// Gets a table by index
    pub fn get_table(&self, idx: usize) -> Result<TableWrapper> {
        self.tables.get(idx).map(|table| table.clone()).map_err(|_| {
            Error::new(
                ErrorCategory::Runtime,
                codes::TABLE_NOT_FOUND,
                "Runtime operation error",
            )
        })
    }

    /// Adds a function export
    pub fn add_function_export(&mut self, name: String, index: u32) -> Result<()> {
        let export = Export::new(name.clone(), ExportKind::Function, index)?;
        #[cfg(feature = "std")]
        self.exports.insert(name, export);
        #[cfg(not(feature = "std"))]
        {
            let bounded_name = wrt_foundation::bounded::BoundedString::from_str_truncate(
                name.as_str(),
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            self.exports.insert(bounded_name, export)?;
        }
        Ok(())
    }

    /// Adds a table export
    pub fn add_table_export(&mut self, name: String, index: u32) -> Result<()> {
        let export = Export::new(name.clone(), ExportKind::Table, index)?;
        #[cfg(feature = "std")]
        self.exports.insert(name, export);
        #[cfg(not(feature = "std"))]
        {
            let bounded_name = wrt_foundation::bounded::BoundedString::from_str_truncate(
                name.as_str(),
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            self.exports.insert(bounded_name, export)?;
        }
        Ok(())
    }

    /// Adds a memory export
    pub fn add_memory_export(&mut self, name: String, index: u32) -> Result<()> {
        let export = Export::new(name.clone(), ExportKind::Memory, index)?;
        #[cfg(feature = "std")]
        self.exports.insert(name, export);
        #[cfg(not(feature = "std"))]
        {
            let bounded_name = wrt_foundation::bounded::BoundedString::from_str_truncate(
                name.as_str(),
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            self.exports.insert(bounded_name, export)?;
        }
        Ok(())
    }

    /// Adds a global export
    pub fn add_global_export(&mut self, name: String, index: u32) -> Result<()> {
        let export = Export::new(name.clone(), ExportKind::Global, index)?;
        #[cfg(feature = "std")]
        self.exports.insert(name, export);
        #[cfg(not(feature = "std"))]
        {
            let bounded_name = wrt_foundation::bounded::BoundedString::from_str_truncate(
                name.as_str(),
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            self.exports.insert(bounded_name, export)?;
        }
        Ok(())
    }

    /// Adds an export to the module from a wrt_format::module::Export
    pub fn add_export(&mut self, format_export: wrt_format::module::Export) -> Result<()> {
        let runtime_export_kind = match format_export.kind {
            wrt_format::module::ExportKind::Function => ExportKind::Function,
            wrt_format::module::ExportKind::Table => ExportKind::Table,
            wrt_format::module::ExportKind::Memory => ExportKind::Memory,
            wrt_format::module::ExportKind::Global => ExportKind::Global,
        };
        let runtime_export = Export::new(format_export.name, runtime_export_kind, format_export.index)?;
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
            .map_err(|_| {
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
            ExternType::Func(func_type),
        )?;
        #[cfg(feature = "std")]
        {
            self.imports
                .entry(module_name.to_string())
                .or_default()
                .insert(item_name.to_string(), import_struct);
        }
        #[cfg(not(feature = "std"))]
        {
            let bounded_module = wrt_foundation::bounded::BoundedString::from_str_truncate(
                module_name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            let bounded_item = wrt_foundation::bounded::BoundedString::from_str_truncate(
                item_name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            if !self.imports.contains_key(&bounded_module) {
                self.imports.insert(bounded_module.clone(), HashMap::new());
            }
            if let Some(module_map) = self.imports.get_mut(&bounded_module) {
                module_map.insert(bounded_item, import_struct)?;
            }
        }
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
        )?;
        #[cfg(feature = "std")]
        {
            self.imports
                .entry(module_name.to_string())
                .or_default()
                .insert(item_name.to_string(), import_struct);
        }
        #[cfg(not(feature = "std"))]
        {
            let bounded_module = wrt_foundation::bounded::BoundedString::from_str_truncate(
                module_name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            let bounded_item = wrt_foundation::bounded::BoundedString::from_str_truncate(
                item_name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            if !self.imports.contains_key(&bounded_module) {
                self.imports.insert(bounded_module.clone(), HashMap::new());
            }
            if let Some(module_map) = self.imports.get_mut(&bounded_module) {
                module_map.insert(bounded_item, import_struct)?;
            }
        }
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
        )?;
        #[cfg(feature = "std")]
        {
            self.imports
                .entry(module_name.to_string())
                .or_default()
                .insert(item_name.to_string(), import_struct);
        }
        #[cfg(not(feature = "std"))]
        {
            let bounded_module = wrt_foundation::bounded::BoundedString::from_str_truncate(
                module_name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            let bounded_item = wrt_foundation::bounded::BoundedString::from_str_truncate(
                item_name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            if !self.imports.contains_key(&bounded_module) {
                self.imports.insert(bounded_module.clone(), HashMap::new());
            }
            if let Some(module_map) = self.imports.get_mut(&bounded_module) {
                module_map.insert(bounded_item, import_struct)?;
            }
        }
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
        )?;

        self.imports
            .entry(module_name.to_string())
            .or_default()
            .insert(item_name.to_string(), import);
        Ok(())
    }

    /// Add a function to the module
    pub fn add_function_type(&mut self, type_idx: u32) -> Result<()> {
        if type_idx as usize >= self.types.len() {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::TYPE_MISMATCH,
                &format!(
                    "Function type index {} out of bounds (max {})",
                    type_idx,
                    self.types.len()
                ),
            ));
        }

        let function = Function { 
            type_idx, 
            locals: wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default())?, 
            body: WrtExpr::default() 
        };

        self.functions.push(function);
        Ok(())
    }

    /// Add a table to the module
    pub fn add_table(&mut self, table_type: WrtTableType) -> Result<()> {
        self.tables.push(TableWrapper::new(Table::new(table_type)?));
        Ok(())
    }

    /// Add a memory to the module
    pub fn add_memory(&mut self, memory_type: WrtMemoryType) -> Result<()> {
        self.memories.push(MemoryWrapper::new(Memory::new(memory_type)?));
        Ok(())
    }

    /// Add a global to the module
    pub fn add_global(&mut self, global_type: WrtGlobalType, init: WrtValue) -> Result<()> {
        let global = Global::new(global_type.value_type, global_type.mutable, init)?;
        self.globals.push(GlobalWrapper::new(global));
        Ok(())
    }

    /// Add a function export to the module
    pub fn add_export_func(&mut self, name: &str, index: u32) -> Result<()> {
        if index as usize >= self.functions.len() {
            return Err(Error::validation_error(&format!(
                "Export function index {} out of bounds",
                index
            )));
        }

        let export = Export { name: name.to_string(), kind: ExportKind::Function, index };

        #[cfg(feature = "std")]
        self.exports.insert(name.to_string(), export);
        #[cfg(not(feature = "std"))]
        {
            let bounded_name = wrt_foundation::bounded::BoundedString::from_str_truncate(
                name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            self.exports.insert(bounded_name, export)?;
        }
        Ok(())
    }

    /// Add a table export to the module
    pub fn add_export_table(&mut self, name: &str, index: u32) -> Result<()> {
        if index as usize >= self.tables.len() {
            return Err(Error::validation_error(&format!(
                "Export table index {} out of bounds",
                index
            )));
        }

        let export = Export { name: name.to_string(), kind: ExportKind::Table, index };

        #[cfg(feature = "std")]
        self.exports.insert(name.to_string(), export);
        #[cfg(not(feature = "std"))]
        {
            let bounded_name = wrt_foundation::bounded::BoundedString::from_str_truncate(
                name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            self.exports.insert(bounded_name, export)?;
        }
        Ok(())
    }

    /// Add a memory export to the module
    pub fn add_export_memory(&mut self, name: &str, index: u32) -> Result<()> {
        if index as usize >= self.memories.len() {
            return Err(Error::validation_error(&format!(
                "Export memory index {} out of bounds",
                index
            )));
        }

        let export = Export { name: name.to_string(), kind: ExportKind::Memory, index };

        #[cfg(feature = "std")]
        self.exports.insert(name.to_string(), export);
        #[cfg(not(feature = "std"))]
        {
            let bounded_name = wrt_foundation::bounded::BoundedString::from_str_truncate(
                name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            self.exports.insert(bounded_name, export)?;
        }
        Ok(())
    }

    /// Add a global export to the module
    pub fn add_export_global(&mut self, name: &str, index: u32) -> Result<()> {
        if index as usize >= self.globals.len() {
            return Err(Error::validation_error(&format!(
                "Export global index {} out of bounds",
                index
            )));
        }

        let export = Export { name: name.to_string(), kind: ExportKind::Global, index };

        #[cfg(feature = "std")]
        self.exports.insert(name.to_string(), export);
        #[cfg(not(feature = "std"))]
        {
            let bounded_name = wrt_foundation::bounded::BoundedString::from_str_truncate(
                name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            self.exports.insert(bounded_name, export)?;
        }
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
            mode: WrtElementMode::Active { table_index: 0, offset: 0 }, // Default mode, should be determined from element.init
            table_idx: element.table_idx,
            offset_expr: None, // Would need to convert from element.offset
            element_type: WrtRefType::Funcref, // Default type
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
            let _ = self.functions.set(func_idx as usize, func_entry).map_err(|_| Error::new(
                ErrorCategory::Runtime,
                codes::COMPONENT_LIMIT_EXCEEDED,
                "Failed to set function entry"
            ))?;
        }
        Ok(())
    }

    /// Add a data segment to the module
    pub fn add_data(&mut self, data: wrt_format::module::Data) -> Result<()> {
        // Convert format data to runtime data
        let mut init_4096 = wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default())?;
        
        // Copy data from the format's init (1024 capacity) to runtime's init (4096 capacity)
        for byte in data.init.iter() {
            init_4096.push(byte)?;
        }
        
        let runtime_data = crate::module::Data {
            mode: WrtDataMode::Active { memory_index: 0, offset: 0 }, // Default mode
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
        )?;
        #[cfg(feature = "std")]
        {
            self.imports
                .entry(module_name.to_string())
                .or_default()
                .insert(item_name.to_string(), import_struct);
        }
        #[cfg(not(feature = "std"))]
        {
            let bounded_module = wrt_foundation::bounded::BoundedString::from_str_truncate(
                module_name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            let bounded_item = wrt_foundation::bounded::BoundedString::from_str_truncate(
                item_name,
                wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
            )?;
            if !self.imports.contains_key(&bounded_module) {
                self.imports.insert(bounded_module.clone(), HashMap::new());
            }
            if let Some(module_map) = self.imports.get_mut(&bounded_module) {
                module_map.insert(bounded_item, import_struct)?;
            }
        }
        Ok(())
    }

    /// Add a runtime export to the module
    pub fn add_runtime_export(&mut self, name: String, export_desc: WrtExportDesc) -> Result<()> {
        let (kind, index) = match export_desc {
            WrtExportDesc::Func(idx) => (ExportKind::Function, idx),
            WrtExportDesc::Table(idx) => (ExportKind::Table, idx),
            WrtExportDesc::Memory(idx) => (ExportKind::Memory, idx),
            WrtExportDesc::Global(idx) => (ExportKind::Global, idx),
            WrtExportDesc::Tag(_) => {
                return Err(Error::new(
                    ErrorCategory::NotSupported,
                    codes::UNSUPPORTED_OPERATION,
                    "Tag exports not supported",
                ))
            }
        };
        let runtime_export = crate::module::Export::new(name.clone(), kind, index)?;
        self.exports.insert(name, runtime_export);
        Ok(())
    }

    /// Add a runtime element to the module
    pub fn add_runtime_element(&mut self, element_segment: WrtElementSegment) -> Result<()> {
        // TODO: Resolve element_segment.items expressions if they are not direct
        // indices. This is a placeholder and assumes items can be derived or
        // handled during instantiation.
        // TODO: ElementItems type not available yet, using empty items for now
        #[cfg(feature = "std")]
        let items_resolved = vec![];
        #[cfg(all(not(feature = "std"), not(feature = "std")))]
        let items_resolved = wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default())?;

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
        ty: WrtFuncType<wrt_foundation::safe_memory::NoStdProvider<1024>>,
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
#[cfg(feature = "std")]
use std::{collections::HashMap, sync::Arc}; // For std types
#[cfg(not(feature = "std"))]
use crate::prelude::HashMap; // Use HashMap from prelude which handles no_std

use wrt_error::{codes, Error, ErrorCategory, Result};
use wrt_foundation::component::ExternType; // For error handling

// Newtype wrappers to solve orphan rules issue
// These allow us to implement external traits on types containing Arc<T>

/// Wrapper for Arc<Table> to enable trait implementations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableWrapper(pub Arc<Table>);

impl Default for TableWrapper {
    fn default() -> Self {
        use wrt_foundation::types::{Limits, TableType, RefType};
        let table_type = TableType {
            element_type: RefType::Funcref,
            limits: Limits { min: 0, max: Some(1) },
        };
        Self::new(Table::new(table_type).unwrap())
    }
}

impl TableWrapper {
    /// Create a new table wrapper
    pub fn new(table: Table) -> Self {
        Self(Arc::new(table))
    }
    
    /// Get a reference to the inner table
    pub fn inner(&self) -> &Arc<Table> {
        &self.0
    }
    
    /// Unwrap to get the Arc<Table>
    pub fn into_inner(self) -> Arc<Table> {
        self.0
    }
    
    /// Get table size
    pub fn size(&self) -> u32 {
        self.0.size()
    }
    
    /// Get table element
    pub fn get(&self, idx: u32) -> Result<Option<WrtValue>> {
        self.0.get(idx)
    }
    
    /// Set table element (requires mutable access)
    pub fn set(&self, idx: u32, value: Option<WrtValue>) -> Result<()> {
        // Note: This requires unsafe because we can't get mutable access to Arc<Table>
        // For now, we'll return an error
        Err(Error::new(
            ErrorCategory::Runtime,
            crate::codes::TABLE_ACCESS_DENIED,
            "Set operation not supported through TableWrapper".to_string(),
        ))
    }
    
    /// Grow table (requires mutable access)
    pub fn grow(&self, delta: u32, init_value: WrtValue) -> Result<u32> {
        // Note: This requires unsafe because we can't get mutable access to Arc<Table>
        // For now, we'll return an error
        Err(Error::new(
            ErrorCategory::Runtime,
            crate::codes::TABLE_ACCESS_DENIED,
            "Grow operation not supported through TableWrapper".to_string(),
        ))
    }
    
    /// Initialize table (requires mutable access)
    pub fn init(&self, offset: u32, init_data: &[Option<WrtValue>]) -> Result<()> {
        // Note: This requires unsafe because we can't get mutable access to Arc<Table>
        // For now, we'll return an error
        Err(Error::new(
            ErrorCategory::Runtime,
            crate::codes::TABLE_ACCESS_DENIED,
            "Init operation not supported through TableWrapper".to_string(),
        ))
    }
}

/// Wrapper for Arc<Memory> to enable trait implementations  
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryWrapper(pub Arc<Memory>);

impl Default for MemoryWrapper {
    fn default() -> Self {
        use wrt_foundation::types::{Limits, MemoryType};
        let memory_type = MemoryType {
            limits: Limits { min: 1, max: Some(1) },
            shared: false,
        };
        Self::new(Memory::new(memory_type).unwrap())
    }
}

impl MemoryWrapper {
    /// Create a new memory wrapper
    pub fn new(memory: Memory) -> Self {
        Self(Arc::new(memory))
    }
    
    /// Get a reference to the inner memory
    pub fn inner(&self) -> &Arc<Memory> {
        &self.0
    }
    
    /// Unwrap to get the Arc<Memory>
    pub fn into_inner(self) -> Arc<Memory> {
        self.0
    }
    
    /// Get memory size in bytes
    pub fn size_in_bytes(&self) -> usize {
        self.0.size_in_bytes()
    }
    
    /// Get memory size in pages
    pub fn size(&self) -> u32 {
        self.0.size()
    }
    
    /// Get memory size in pages (alias for compatibility)
    pub fn size_pages(&self) -> u32 {
        self.0.size()
    }
    
    /// Get memory size in bytes (alias for compatibility)
    pub fn size_bytes(&self) -> usize {
        self.0.size_in_bytes()
    }
    
    /// Read from memory
    pub fn read(&self, offset: u32, buffer: &mut [u8]) -> Result<()> {
        self.0.read(offset, buffer)
    }
    
    /// Write to memory (requires mutable access to Arc<Memory>)
    pub fn write(&self, offset: u32, buffer: &[u8]) -> Result<()> {
        // Note: This requires unsafe because we can't get mutable access to Arc<Memory>
        // For now, we'll return an error
        Err(Error::new(
            ErrorCategory::Runtime,
            crate::codes::MEMORY_ACCESS_DENIED,
            "Write access not supported through MemoryWrapper".to_string(),
        ))
    }
    
    /// Grow memory (requires mutable access)
    pub fn grow(&self, pages: u32) -> Result<u32> {
        // Note: This requires unsafe because we can't get mutable access to Arc<Memory>
        // For now, we'll return an error
        Err(Error::new(
            ErrorCategory::Runtime,
            crate::codes::MEMORY_ACCESS_DENIED,
            "Grow operation not supported through MemoryWrapper".to_string(),
        ))
    }
    
    /// Fill memory (requires mutable access)
    pub fn fill(&self, offset: u32, len: u32, value: u8) -> Result<()> {
        // Note: This requires unsafe because we can't get mutable access to Arc<Memory>
        // For now, we'll return an error
        Err(Error::new(
            ErrorCategory::Runtime,
            crate::codes::MEMORY_ACCESS_DENIED,
            "Fill operation not supported through MemoryWrapper".to_string(),
        ))
    }
}

/// Wrapper for Arc<Global> to enable trait implementations
#[derive(Debug, Clone, PartialEq, Eq)]  
pub struct GlobalWrapper(pub Arc<Global>);

impl Default for GlobalWrapper {
    fn default() -> Self {
        use wrt_foundation::types::ValueType;
        use wrt_foundation::values::Value;
        Self::new(Global::new(ValueType::I32, false, Value::I32(0)).unwrap())
    }
}

impl GlobalWrapper {
    /// Create a new global wrapper
    pub fn new(global: Global) -> Self {
        Self(Arc::new(global))
    }
    
    /// Get a reference to the inner global
    pub fn inner(&self) -> &Arc<Global> {
        &self.0
    }
    
    /// Unwrap to get the Arc<Global>
    pub fn into_inner(self) -> Arc<Global> {
        self.0
    }
    
    /// Get global value
    pub fn get_value(&self) -> &WrtValue {
        self.0.get()
    }
    
    /// Set global value (requires mutable access)
    pub fn set_value(&self, new_value: &WrtValue) -> Result<()> {
        // Note: This requires unsafe because we can't get mutable access to Arc<Global>
        // For now, we'll return an error
        Err(Error::new(
            ErrorCategory::Runtime,
            crate::codes::GLOBAL_ACCESS_DENIED,
            "Set operation not supported through GlobalWrapper".to_string(),
        ))
    }
    
    /// Get global value type
    pub fn value_type(&self) -> WrtValueType {
        self.0.global_type_descriptor().value_type
    }
    
    /// Check if global is mutable
    pub fn is_mutable(&self) -> bool {
        self.0.global_type_descriptor().mutable
    }
}

// Implement foundation traits for wrapper types
use wrt_foundation::traits::{Checksummable, ToBytes, FromBytes, ReadStream, WriteStream};
use wrt_foundation::verification::Checksum;

// TableWrapper trait implementations
impl Checksummable for TableWrapper {
    fn update_checksum(&self, checksum: &mut Checksum) {
        // Use table size and element type for checksum
        checksum.update_slice(&self.0.size().to_le_bytes());
        checksum.update_slice(&(self.0.ty.element_type as u8).to_le_bytes());
    }
}

impl ToBytes for TableWrapper {
    fn serialized_size(&self) -> usize {
        12 // table type (4) + size (4) + limits (4)
    }
    
    fn to_bytes_with_provider<P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut WriteStream,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        writer.write_all(&self.0.size().to_le_bytes())?;
        writer.write_all(&(self.0.ty.element_type as u8).to_le_bytes())?;
        writer.write_all(&self.0.ty.limits.min.to_le_bytes())?;
        Ok(())
    }
}

impl FromBytes for TableWrapper {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 12];
        reader.read_exact(&mut bytes)?;
        
        // Create a default table (simplified implementation)
        use wrt_foundation::types::{Limits, TableType, RefType};
        let table_type = TableType {
            element_type: RefType::Funcref,
            limits: Limits { min: 0, max: Some(1) },
        };
        
        let table = Table::new(table_type).map_err(|_| {
            wrt_foundation::Error::new(
                wrt_foundation::ErrorCategory::Memory,
                wrt_foundation::codes::INVALID_VALUE,
                "Failed to create table from bytes"
            )
        })?;
        
        Ok(TableWrapper::new(table))
    }
}

// MemoryWrapper trait implementations  
impl Checksummable for MemoryWrapper {
    fn update_checksum(&self, checksum: &mut Checksum) {
        // Use memory size for checksum
        checksum.update_slice(&self.0.size().to_le_bytes());
        checksum.update_slice(&self.0.size_in_bytes().to_le_bytes());
    }
}

impl ToBytes for MemoryWrapper {
    fn serialized_size(&self) -> usize {
        12 // size (4) + limits min (4) + limits max (4)
    }
    
    fn to_bytes_with_provider<P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut WriteStream,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        writer.write_all(&self.0.size().to_le_bytes())?;
        writer.write_all(&self.0.ty.limits.min.to_le_bytes())?;
        let max = self.0.ty.limits.max.unwrap_or(u32::MAX);
        writer.write_all(&max.to_le_bytes())?;
        Ok(())
    }
}

impl FromBytes for MemoryWrapper {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 12];
        reader.read_exact(&mut bytes)?;
        
        // Create a default memory (simplified implementation)
        use wrt_foundation::types::{Limits, MemoryType};
        let memory_type = MemoryType {
            limits: Limits { min: 1, max: Some(1) },
            shared: false,
        };
        
        let memory = Memory::new(memory_type).map_err(|_| {
            wrt_foundation::Error::new(
                wrt_foundation::ErrorCategory::Memory,
                wrt_foundation::codes::INVALID_VALUE,
                "Failed to create memory from bytes"
            )
        })?;
        
        Ok(MemoryWrapper::new(memory))
    }
}

// GlobalWrapper trait implementations
impl Checksummable for GlobalWrapper {
    fn update_checksum(&self, checksum: &mut Checksum) {
        // Use global value type for checksum
        checksum.update_slice(&((*self.0).value_type() as u8).to_le_bytes());
        checksum.update_slice(&((*self.0).is_mutable() as u8).to_le_bytes());
    }
}

impl ToBytes for GlobalWrapper {
    fn serialized_size(&self) -> usize {
        12 // value type (4) + mutable flag (4) + value (4)
    }
    
    fn to_bytes_with_provider<P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut WriteStream,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        writer.write_all(&((*self.0).value_type() as u8).to_le_bytes())?;
        writer.write_all(&((*self.0).is_mutable() as u8).to_le_bytes())?;
        // Simplified value serialization
        writer.write_all(&0u32.to_le_bytes())?;
        Ok(())
    }
}

impl FromBytes for GlobalWrapper {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 12];
        reader.read_exact(&mut bytes)?;
        
        // Create a default global (simplified implementation)
        use wrt_foundation::types::ValueType;
        use wrt_foundation::values::Value;
        
        let global = Global::new(ValueType::I32, false, Value::I32(0)).map_err(|_| {
            wrt_foundation::Error::new(
                wrt_foundation::ErrorCategory::Memory,
                wrt_foundation::codes::INVALID_VALUE,
                "Failed to create global from bytes"
            ) 
        })?;
        
        Ok(GlobalWrapper::new(global))
    }
}

// Arc<Table> trait implementations removed due to orphan rule violations.
// Use TableWrapper instead which implements these traits properly.

// Trait implementations for Arc<Memory>

// Default for Arc<Memory> removed due to orphan rules - use explicit creation instead
/*
*/


// Arc<Memory> trait implementations removed due to orphan rule violations.
// Use MemoryWrapper instead which implements these traits properly.

// Trait implementations for Arc<Global>

// Default for Arc<Global> removed due to orphan rules - use explicit creation instead


// Arc<Global> trait implementations removed due to orphan rule violations.
// Use GlobalWrapper instead which implements these traits properly.

// Ensure local `crate::module::Import` struct is defined
// Ensure local `crate::module::Export` struct is defined
// Ensure local `crate::global::Global`, `crate::table::Table`,
// `crate::memory::Memory` are defined and their `new` methods are compatible.

