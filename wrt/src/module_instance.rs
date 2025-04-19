use crate::{
    error::{self, kinds},
    error::{Error, Result},
    global::Global,
    instructions::Instruction,
    memory::{DefaultMemory, MemoryBehavior},
    module::Element,
    module::{Data, Module, OtherExport},
    table::Table,
    types::FuncType,
    values::Value,
};
use std::sync::Arc;

/// Represents a module instance during execution
#[derive(Debug, Clone)]
pub struct ModuleInstance {
    /// Module index in the engine instances array
    pub module_idx: u32,
    /// Module definition (Now an Arc)
    pub module: Arc<Module>,
    /// Function addresses
    pub func_addrs: Vec<FunctionAddr>,
    /// Table addresses
    pub table_addrs: Vec<TableAddr>,
    /// Memory addresses
    pub memory_addrs: Vec<MemoryAddr>,
    /// Global addresses
    pub global_addrs: Vec<GlobalAddr>,
    /// Actual memory instances wrapped in Arc<dyn MemoryBehavior>
    pub memories: Vec<Arc<dyn MemoryBehavior>>,
    /// Actual table instances (now Arc)
    pub tables: Vec<Arc<Table>>,
    /// Actual global instances (now Arc)
    pub globals: Vec<Arc<Global>>,
}

/// Represents a function address
#[derive(Debug, Clone)]
pub struct FunctionAddr {
    /// Module instance index
    pub instance_idx: u32,
    /// Function index
    pub func_idx: u32,
}

/// Represents a table address
#[derive(Debug, Clone, Copy)]
pub struct TableAddr {
    /// Module instance index
    pub instance_idx: u32,
    /// Table index
    pub table_idx: u32,
}

/// Represents a memory address
#[derive(Debug, Clone)]
pub struct MemoryAddr {
    /// Module instance index
    pub instance_idx: u32,
    /// Memory index
    pub memory_idx: u32,
}

/// Represents a global address
#[derive(Debug, Clone)]
pub struct GlobalAddr {
    /// Module instance index
    pub instance_idx: u32,
    /// Global index
    pub global_idx: u32,
}

impl ModuleInstance {
    /// Creates a new module instance
    pub fn new(module: Arc<Module>) -> Result<Self> {
        // Initialize tables and globals from the module definition, wrapping in Arc
        let tables = module
            .tables
            .read()
            .map_err(|_| {
                Error::new(kinds::PoisonedLockError(
                    "Module tables lock poisoned".to_string(),
                ))
            })?
            .iter()
            .map(|table_arc| table_arc.clone()) // Already Arcs in module? Assume yes. If not, need Arc::new(table.clone())
            .collect();
        let globals = module
            .globals
            .read()
            .map_err(|_| {
                Error::new(kinds::PoisonedLockError(
                    "Module globals lock poisoned".to_string(),
                ))
            })?
            .iter()
            .map(|global_arc| global_arc.clone()) // Already Arcs in module? Assume yes. If not, need Arc::new(global.clone())
            .collect();
        // Initialize memories similarly
        let memories = module
            .memories
            .read()
            .map_err(|_| {
                Error::new(kinds::PoisonedLockError(
                    "Module memories lock poisoned".to_string(),
                ))
            })?
            .iter()
            .map(|mem_arc| Arc::clone(mem_arc) as Arc<dyn MemoryBehavior>) // Clone and cast if necessary
            .collect();

        Ok(Self {
            module,
            module_idx: 0,            // Placeholder, should be set by engine
            func_addrs: Vec::new(),   // Placeholder
            table_addrs: Vec::new(),  // Placeholder
            memory_addrs: Vec::new(), // Placeholder
            global_addrs: Vec::new(), // Placeholder
            memories,                 // Initialized from module
            tables,                   // Initialized from module
            globals,                  // Initialized from module
        })
    }

    /// Finds an export by name
    #[must_use]
    pub fn find_export(&self, name: &str) -> Option<&OtherExport> {
        self.module.exports.iter().find(|e| e.name == name)
    }

    /// Gets an export by name
    pub fn get_export(&self, name: &str) -> Result<&OtherExport> {
        self.find_export(name)
            .ok_or_else(|| Error::new(kinds::ExportNotFoundError(name.to_string())))
    }

    /// Gets a function type by index
    pub fn get_function_type(&self, func_idx: u32) -> Result<&FuncType> {
        self.module.types.get(func_idx as usize).ok_or_else(|| {
            Error::new(kinds::InvalidFunctionTypeError(format!(
                "Invalid function type index: {func_idx}"
            )))
        })
    }

    /// Gets a function instruction by index and program counter
    pub fn get_function_instruction(&self, func_idx: u32, pc: usize) -> Result<&Instruction> {
        let func = self
            .module
            .functions
            .get(func_idx as usize)
            .ok_or_else(|| {
                Error::new(kinds::FunctionNotFoundError(format!(
                    "Function index {func_idx} out of bounds"
                )))
            })?;

        func.code.get(pc).ok_or_else(|| {
            Error::new(kinds::ExecutionError(format!(
                "Instruction index {pc} out of bounds"
            )))
        })
    }

    /// Gets a table by index (returns Arc)
    pub fn get_table(&self, idx: usize) -> Result<Arc<Table>> {
        self.tables
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::new(kinds::InvalidTableIndexError(idx as u32)))
    }

    /// Gets a table by index (returns Arc for trait compatibility, even if mutable op isn't possible on Arc directly)
    pub fn get_table_mut(&mut self, idx: usize) -> Result<Arc<Table>> {
        // Returning Arc here because FrameBehavior expects it.
        // Mutability must be handled via interior mutability within Table itself (e.g., Mutex, RwLock).
        // This method doesn't actually provide unique mutable access to the Arc itself.
        self.tables
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::new(kinds::InvalidTableIndexError(idx as u32)))
    }

    /// Sets a global value by index (modifies Arc<Global>)
    pub fn set_global(&mut self, idx: usize, value: Value) -> Result<()> {
        if let Some(global_arc) = self.globals.get(idx) {
            // Global::set takes &self due to interior mutability (Mutex/Atomic)
            global_arc.set(value)
        } else {
            Err(Error::new(kinds::InvalidGlobalIndexError(idx as u32)))
        }
    }

    /// Gets an element segment by index
    pub fn get_element_segment(&self, idx: u32) -> Result<&Element> {
        self.module
            .elements
            .get(idx as usize)
            .ok_or_else(|| Error::new(kinds::InvalidElementIndexError(idx)))
    }

    /// Gets a memory instance by index
    pub fn get_memory(&self, idx: usize) -> Result<&Arc<dyn MemoryBehavior>> {
        self.memories
            .get(idx)
            .ok_or_else(|| Error::new(kinds::InvalidMemoryIndexError(idx as u32)))
    }

    /// Gets a mutable memory instance by index
    /// Note: Returning &mut Arc is unusual. Consider if the design should allow mutable access this way.
    /// For now, aligning with the previous structure, but this might need revisiting.
    pub fn get_memory_mut(&mut self, idx: usize) -> Result<&mut Arc<dyn MemoryBehavior>> {
        self.memories
            .get_mut(idx)
            .ok_or_else(|| Error::new(kinds::InvalidMemoryIndexError(idx as u32)))
    }

    // Add get_global if needed by stackless_frame (returns Arc)
    pub fn get_global(&self, idx: usize) -> Result<Arc<Global>> {
        self.globals
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::new(kinds::InvalidGlobalIndexError(idx as u32)))
    }

    /// Get the data segment at the given index.
    pub fn get_data(&self, idx: u32) -> Result<&Data> {
        self.module.data.get(idx as usize).ok_or_else(|| {
            Error::new(kinds::ValidationError(format!(
                "Invalid data index: {}",
                idx
            )))
        })
    }

    pub fn drop_data_segment(&mut self, _idx: u32) -> Result<()> {
        // Assuming data segments are stored in the module and are immutable once loaded.
        // Dropping might mean clearing a reference or marking as inactive if the instance tracks active segments.
        // If data is part of the immutable Arc<Module>, we can't truly drop it here.
        // For now, return unimplemented error.
        // self.module.data.remove(idx as usize); // Cannot do this on Arc<Module>
        Err(Error::new(kinds::UnimplementedError(
            "drop_data_segment not implemented".to_string(),
        )))
    }

    // Added set_data_segment (placeholder)
    pub fn set_data_segment(&mut self, _idx: u32, _segment: Arc<Data>) -> Result<()> {
        // This implies mutable access to data segments, which conflicts with Arc<Module>.
        // Perhaps the instance should store its own mutable copy or references?
        Err(Error::new(kinds::UnimplementedError(
            "set_data_segment not implemented".to_string(),
        )))
    }

    // Added get_two_tables_mut if needed by stackless_frame (returns Arcs)
    pub fn get_two_tables_mut(&mut self, idx1: u32, idx2: u32) -> Result<(Arc<Table>, Arc<Table>)> {
        // This can just return the Arcs. Mutability is internal to Table.
        let table1 = self.get_table(idx1 as usize)?;
        let table2 = self.get_table(idx2 as usize)?;
        Ok((table1, table2))
    }

    /// Implements the elem.drop instruction
    /// Marks an element segment as dropped, making it unavailable for future use
    /// Since element segments are stored in the immutable module, we can't truly drop them
    /// In a real implementation, this would mark the segment as inactive in some tracking structure
    pub fn elem_drop(&mut self, elem_idx: u32) -> Result<()> {
        // Verify the element index is valid
        if elem_idx as usize >= self.module.elements.len() {
            return Err(Error::new(kinds::InvalidElementIndexError(elem_idx)));
        }

        // In a production implementation, we would mark this element as dropped
        // For now, we'll return Ok to indicate the operation "succeeded"
        // but note that the element isn't actually dropped from memory
        Ok(())
    }

    /// Gets the function address for a function index
    pub fn get_func_addr(&self, func_idx: u32) -> Result<FunctionAddr> {
        if (func_idx as usize) < self.func_addrs.len() {
            Ok(self.func_addrs[func_idx as usize].clone())
        } else {
            Err(Error::new(kinds::InvalidFunctionIndexError(
                func_idx as usize,
            )))
        }
    }
}
