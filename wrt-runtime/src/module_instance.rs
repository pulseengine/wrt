//! Module instance implementation for WebAssembly runtime
//!
//! This module provides the implementation of a WebAssembly module instance,
//! which represents a runtime instance of a WebAssembly module with its own
//! memory, tables, globals, and functions.

// alloc is imported in lib.rs with proper feature gates

#[cfg(feature = "debug-full")]
use wrt_debug::FunctionInfo;
#[cfg(feature = "debug")]
use wrt_debug::{
    DwarfDebugInfo,
    LineInfo,
};
use wrt_foundation::{
    budget_aware_provider::CrateId,
    safe_managed_alloc,
    traits::{
        BoundedCapacity,
        Checksummable,
        FromBytes,
        ReadStream,
        ToBytes,
        WriteStream,
    },
    verification::Checksum,
};
use wrt_instructions::reference_ops::ReferenceOperations;

// Type alias for FuncType to make signatures more readable - uses unified RuntimeProvider
use crate::bounded_runtime_infra::{
    create_runtime_provider,
    BoundedGlobalVec,
    BoundedImportExportName,
    BoundedImportMap,
    BoundedMemoryVec,
    BoundedTableVec,
    RuntimeProvider,
};
use crate::{
    global::Global,
    memory::Memory,
    module::{
        GlobalWrapper,
        MemoryWrapper,
        Module,
        TableWrapper,
    },
    prelude::{
        Debug,
        Error,
        ErrorCategory,
        FuncType,
        Result,
    },
    table::Table,
};
type WrtFuncType = wrt_foundation::types::FuncType;

// Platform sync primitives - use prelude imports for consistency
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::format;
// Import format! macro for string formatting
#[cfg(feature = "std")]
use std::format;
#[cfg(feature = "std")]
use std::sync::{
    Arc,
    Mutex,
};

#[cfg(not(feature = "std"))]
use crate::prelude::{
    Arc,
    Mutex,
};

/// Represents a runtime instance of a WebAssembly module
#[cfg_attr(not(feature = "debug"), derive(Debug))]
pub struct ModuleInstance {
    /// The module this instance was instantiated from
    module:      Arc<Module>,
    /// The instance's memory (using safety-critical wrapper types)
    memories:    Arc<Mutex<BoundedMemoryVec<MemoryWrapper>>>,
    /// The instance's tables (using safety-critical wrapper types)
    tables:      Arc<Mutex<BoundedTableVec<TableWrapper>>>,
    /// The instance's globals (using safety-critical wrapper types)
    globals:     Arc<Mutex<BoundedGlobalVec<GlobalWrapper>>>,
    /// Instance ID for debugging
    instance_id: usize,
    /// Imported instance indices to resolve imports
    imports:     BoundedImportMap<BoundedImportMap<(usize, usize)>>,
    /// Debug information (optional)
    #[cfg(feature = "debug")]
    debug_info:  Option<DwarfDebugInfo<'static>>,
}

// Manual Debug implementation when debug feature is enabled
#[cfg(feature = "debug")]
impl Debug for ModuleInstance {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModuleInstance")
            .field("module", &self.module)
            .field("instance_id", &self.instance_id)
            .field("debug_info", &self.debug_info.is_some())
            .finish()
    }
}

impl ModuleInstance {
    /// Create a new module instance from a module (accepts Arc to avoid deep clones)
    pub fn new(module: Arc<Module>, instance_id: usize) -> Result<Self> {
        // Create a single shared provider to avoid stack overflow from multiple
        // provider allocations
        let shared_provider = create_runtime_provider()?;

        // Allocate memory for memories collection
        let memories_vec = wrt_foundation::bounded::BoundedVec::new(shared_provider.clone())?;

        // Allocate memory for tables collection
        let tables_vec = wrt_foundation::bounded::BoundedVec::new(shared_provider.clone())?;

        // Allocate memory for globals collection
        let globals_vec = wrt_foundation::bounded::BoundedVec::new(shared_provider.clone())?;

        Ok(Self {
            module,
            memories: Arc::new(Mutex::new(memories_vec)),
            tables: Arc::new(Mutex::new(tables_vec)),
            globals: Arc::new(Mutex::new(globals_vec)),
            instance_id,
            imports: Default::default(),
            #[cfg(feature = "debug")]
            debug_info: None,
        })
    }

    /// Get the module associated with this instance
    #[must_use]
    pub fn module(&self) -> &Arc<Module> {
        &self.module
    }

    /// Get a memory from this instance
    pub fn memory(&self, idx: u32) -> Result<MemoryWrapper> {
        #[cfg(feature = "std")]
        let memories = self
            .memories
            .lock()
            .map_err(|_| Error::runtime_error("Failed to lock memories"))?;

        #[cfg(not(feature = "std"))]
        let memories = self.memories.lock();

        let memory = memories
            .get(idx as usize)
            .map_err(|_| Error::runtime_execution_error("Memory index out of bounds"))?;
        Ok(memory.clone())
    }

    /// Get a table from this instance
    pub fn table(&self, idx: u32) -> Result<TableWrapper> {
        #[cfg(feature = "std")]
        let tables =
            self.tables.lock().map_err(|_| Error::runtime_error("Failed to lock tables"))?;

        #[cfg(not(feature = "std"))]
        let tables = self.tables.lock();

        let table = tables
            .get(idx as usize)
            .map_err(|_| Error::resource_table_not_found("Runtime operation error"))?;
        Ok(table.clone())
    }

    /// Get a global from this instance
    pub fn global(&self, idx: u32) -> Result<GlobalWrapper> {
        #[cfg(feature = "std")]
        let globals = self
            .globals
            .lock()
            .map_err(|_| Error::runtime_error("Failed to lock globals"))?;

        #[cfg(not(feature = "std"))]
        let globals = self.globals.lock();

        let global = globals
            .get(idx as usize)
            .map_err(|_| Error::resource_global_not_found("Runtime operation error"))?;
        Ok(global.clone())
    }

    /// Get the function type for a function
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn function_type(&self, idx: u32) -> Result<crate::prelude::CoreFuncType> {
        #[cfg(feature = "std")]
        let function = self
            .module
            .functions
            .get(idx as usize)
            .ok_or_else(|| Error::runtime_function_not_found("Function index not found"))?;
        #[cfg(not(feature = "std"))]
        let function = self
            .module
            .functions
            .get(idx as usize)
            .map_err(|_| Error::runtime_function_not_found("Function index not found"))?;

        // In std mode, types is Vec so get() returns Option<&T>
        #[cfg(feature = "std")]
        let ty = self
            .module
            .types
            .get(function.type_idx as usize)
            .ok_or_else(|| Error::validation_type_mismatch("Type index not found"))?;

        // In no_std mode, types is BoundedVec so get() returns Result<T>
        #[cfg(not(feature = "std"))]
        let ty = &self
            .module
            .types
            .get(function.type_idx as usize)
            .map_err(|_| Error::validation_type_mismatch("Type index not found"))?;

        // Convert from provider-aware FuncType to clean CoreFuncType
        // Create BoundedVecs manually since FromIterator isn't implemented
        let params_slice = ty.params.as_slice();
        let results_slice = ty.results.as_slice();

        let mut params = wrt_foundation::bounded::BoundedVec::<
            wrt_foundation::ValueType,
            128,
            RuntimeProvider,
        >::new(create_runtime_provider()?)
        .map_err(|_| Error::memory_error("Failed to create params vec"))?;

        let mut results = wrt_foundation::bounded::BoundedVec::<
            wrt_foundation::ValueType,
            128,
            RuntimeProvider,
        >::new(create_runtime_provider()?)
        .map_err(|_| Error::memory_error("Failed to create results vec"))?;

        for param in params_slice {
            params
                .push(*param)
                .map_err(|_| Error::capacity_limit_exceeded("Too many params"))?;
        }

        for result in results_slice {
            results
                .push(*result)
                .map_err(|_| Error::capacity_limit_exceeded("Too many results"))?;
        }

        // Use FuncType::new() instead of struct literal
        // Note: BoundedVec's iter() yields ValueType by value, not by reference
        let param_types: Vec<_> = params.iter().collect();
        let result_types: Vec<_> = results.iter().collect();
        crate::prelude::CoreFuncType::new(param_types, result_types)
    }

    /// Get the function type for a function (no_std version)
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub fn function_type(&self, idx: u32) -> Result<WrtFuncType> {
        let function = self
            .module
            .functions
            .get(idx as usize)
            .ok_or_else(|| Error::runtime_function_not_found("Function index not found"))?;

        // In std mode, types is Vec so get() returns Option<&T>
        #[cfg(feature = "std")]
        let ty = self
            .module
            .types
            .get(function.type_idx as usize)
            .ok_or_else(|| Error::validation_type_mismatch("Type index not found"))?;

        // In no_std mode, types is BoundedVec so get() returns Result<T>
        #[cfg(not(feature = "std"))]
        let ty = &self
            .module
            .types
            .get(function.type_idx as usize)
            .map_err(|_| Error::validation_type_mismatch("Type index not found"))?;

        Ok(ty.clone())
    }

    /// Add a memory to this instance
    pub fn add_memory(&self, memory: Memory) -> Result<()> {
        #[cfg(feature = "std")]
        let mut memories = self
            .memories
            .lock()
            .map_err(|_| Error::runtime_error("Failed to lock memories"))?;

        #[cfg(not(feature = "std"))]
        let mut memories = self.memories.lock();

        memories
            .push(MemoryWrapper::new(Box::new(memory)))
            .map_err(|_| Error::capacity_limit_exceeded("Memory capacity exceeded"))?;
        Ok(())
    }

    /// Add a table to this instance
    pub fn add_table(&self, table: Table) -> Result<()> {
        #[cfg(feature = "std")]
        let mut tables =
            self.tables.lock().map_err(|_| Error::runtime_error("Failed to lock tables"))?;

        #[cfg(not(feature = "std"))]
        let mut tables = self.tables.lock();

        tables
            .push(TableWrapper::new(table))
            .map_err(|_| Error::capacity_limit_exceeded("Table capacity exceeded"))?;
        Ok(())
    }

    /// Add a global to this instance
    pub fn add_global(&self, global: Global) -> Result<()> {
        #[cfg(feature = "std")]
        let mut globals = self
            .globals
            .lock()
            .map_err(|_| Error::runtime_error("Failed to lock globals"))?;

        #[cfg(not(feature = "std"))]
        let mut globals = self.globals.lock();

        globals
            .push(GlobalWrapper::new(global))
            .map_err(|_| Error::capacity_limit_exceeded("Global capacity exceeded"))?;
        Ok(())
    }

    /// Initialize debug information for this instance
    #[cfg(feature = "debug")]
    pub fn init_debug_info(&mut self, module_bytes: &'static [u8]) -> Result<()> {
        let debug_info = DwarfDebugInfo::new(module_bytes)?;

        // TODO: Extract debug section offsets from the module
        // For now, this is a placeholder that would need module parsing integration

        self.debug_info = Some(debug_info);
        Ok(())
    }

    /// Get line information for a given program counter
    #[cfg(feature = "debug")]
    pub fn get_line_info(&mut self, pc: u32) -> Result<Option<LineInfo>> {
        if let Some(ref mut debug_info) = self.debug_info {
            debug_info
                .find_line_info(pc)
                .map_err(|e| Error::runtime_debug_info_error("Runtime operation error"))
        } else {
            Ok(None)
        }
    }

    /// Get function information for a given program counter
    #[cfg(feature = "debug-full")]
    pub fn get_function_info(&self, pc: u32) -> Option<&FunctionInfo> {
        self.debug_info.as_ref()?.find_function_info(pc)
    }

    /// Check if debug information is available
    #[cfg(feature = "debug")]
    pub fn has_debug_info(&self) -> bool {
        self.debug_info.as_ref().is_some_and(|di| di.has_debug_info())
    }

    /// Get a function by index - alias for compatibility with tail_call.rs
    pub fn get_function(&self, idx: usize) -> Result<crate::module::Function> {
        #[cfg(feature = "std")]
        return self.module
            .functions
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::runtime_function_not_found("Function index not found"));
        #[cfg(not(feature = "std"))]
        return self.module
            .functions
            .get(idx)
            .map(|f| f.clone())
            .map_err(|_| Error::runtime_function_not_found("Function index not found"));
    }

    /// Get function type by index - alias for compatibility with tail_call.rs
    pub fn get_function_type(&self, idx: usize) -> Result<WrtFuncType> {
        #[cfg(feature = "std")]
        let function = self
            .module
            .functions
            .get(idx)
            .ok_or_else(|| Error::runtime_function_not_found("Function index not found"))?;
        #[cfg(not(feature = "std"))]
        let function = self
            .module
            .functions
            .get(idx)
            .map_err(|_| Error::runtime_function_not_found("Function index not found"))?;

        // In std mode, types is Vec so get() returns Option<&T>
        #[cfg(feature = "std")]
        return self.module.types.get(function.type_idx as usize)
            .cloned()
            .ok_or_else(|| Error::runtime_error("Function type index out of bounds"));

        // In no_std mode, types is BoundedVec so get() returns Result<T>
        #[cfg(not(feature = "std"))]
        self.module.types.get(function.type_idx as usize)
    }

    /// Get a table by index - alias for compatibility with tail_call.rs
    pub fn get_table(&self, idx: usize) -> Result<TableWrapper> {
        self.table(idx as u32)
    }

    /// Get a type by index - alias for compatibility with tail_call.rs
    pub fn get_type(&self, idx: usize) -> Result<WrtFuncType> {
        // In std mode, types is Vec so get() returns Option<&T>
        #[cfg(feature = "std")]
        return self.module.types.get(idx)
            .cloned()
            .ok_or_else(|| Error::runtime_error("Type index out of bounds"));

        // In no_std mode, types is BoundedVec so get() returns Result<T>
        #[cfg(not(feature = "std"))]
        self.module.types.get(idx)
    }
}

/// Implementation of ReferenceOperations trait for ModuleInstance
impl ReferenceOperations for ModuleInstance {
    fn get_function(&self, function_index: u32) -> Result<Option<u32>> {
        // Check if function exists in module
        if (function_index as usize) < self.module.functions.len() {
            Ok(Some(function_index))
        } else {
            Ok(None)
        }
    }

    fn validate_function_index(&self, function_index: u32) -> Result<()> {
        if (function_index as usize) < self.module.functions.len() {
            Ok(())
        } else {
            Err(Error::runtime_function_not_found(
                "Function index out of bounds",
            ))
        }
    }
}

// Implement the ModuleInstance trait for module_instance - temporarily disabled
// impl crate::stackless::extensions::ModuleInstance for ModuleInstance {
// fn module(&self) -> &Module {
//     &self.module
// }

// fn memory(&self, idx: u32) -> Result<MemoryWrapper> {
//     self.memory(idx)
// }

// fn table(&self, idx: u32) -> Result<TableWrapper> {
//     self.table(idx)
// }

// fn global(&self, idx: u32) -> Result<GlobalWrapper> {
//     self.global(idx)
// }

// fn function_type(&self, idx: u32) -> Result<FuncType> {
//     self.function_type(idx)
// }
// } // End of commented impl block

/// Manual trait implementations for ModuleInstance since fields don't support
/// automatic derivation
/// REMOVED: Default implementation causes stack overflow through Module::empty()
/// Use ModuleInstance::new() with proper initialization instead
/* DISABLED - CAUSES STACK OVERFLOW
impl Default for ModuleInstance {
    fn default() -> Self {
        // Create a default module instance with an empty module
        let default_module = Module::empty();
        // Default implementation must succeed for basic functionality
        // Use minimal memory allocation that should always work
        match Self::new(Arc::new(default_module), 0) {
            Ok(instance) => instance,
            Err(_) => {
                // Create minimal instance using RuntimeProvider for type consistency
                // This maintains controllability while avoiding allocation failures
                use crate::bounded_runtime_infra::create_runtime_provider;
                // Use the factory function - if this fails, we have a fundamental system issue
                let runtime_provider = match create_runtime_provider() {
                    Ok(provider) => provider,
                    Err(_) => {
                        // Last resort: try to create a minimal provider
                        // This should work even in constrained environments
                        match create_runtime_provider() {
                            Ok(provider) => provider,
                            Err(_) => {
                                // System is in unrecoverable state - but we must return something
                                // Create an invalid instance that will fail safely later
                                return Self {
                                    module: Arc::new(Module::empty()),
                                    memories: Arc::new(Mutex::new(Default::default())),
                                    tables: Arc::new(Mutex::new(Default::default())),
                                    globals: Arc::new(Mutex::new(Default::default())),
                                    instance_id: 0,
                                    imports: Default::default(),
                                    #[cfg(feature = "debug")]
                                    debug_info: None,
                                };
                            },
                        }
                    },
                };
                Self {
                    module: Arc::new(Module::empty()),
                    memories: Arc::new(Mutex::new(
                        // Try to create with RuntimeProvider, fallback to empty vector creation
                        wrt_foundation::bounded::BoundedVec::new(runtime_provider.clone())
                            .unwrap_or_else(|_| {
                                // Last resort: try creating another provider
                                let fallback_provider = create_runtime_provider()
                                    .expect("Failed to create fallback runtime provider");
                                wrt_foundation::bounded::BoundedVec::new(fallback_provider)
                                    .expect("Failed to create even minimal memory vector")
                            }),
                    )),
                    tables: Arc::new(Mutex::new(
                        wrt_foundation::bounded::BoundedVec::new(runtime_provider.clone())
                            .unwrap_or_else(|_| {
                                let fallback_provider = create_runtime_provider()
                                    .expect("Failed to create fallback runtime provider");
                                wrt_foundation::bounded::BoundedVec::new(fallback_provider)
                                    .expect("Failed to create even minimal table vector")
                            }),
                    )),
                    globals: Arc::new(Mutex::new(
                        wrt_foundation::bounded::BoundedVec::new(runtime_provider).unwrap_or_else(
                            |_| {
                                let fallback_provider = create_runtime_provider()
                                    .expect("Failed to create fallback runtime provider");
                                wrt_foundation::bounded::BoundedVec::new(fallback_provider)
                                    .expect("Failed to create even minimal global vector")
                            },
                        ),
                    )),
                    instance_id: 0,
                    imports: Default::default(),
                    #[cfg(feature = "debug")]
                    debug_info: None,
                }
            },
        }
    }
}
*/ // End of DISABLED Default impl

impl Clone for ModuleInstance {
    fn clone(&self) -> Self {
        // Create a new instance with the same module and instance ID
        Self::new(Arc::clone(&self.module), self.instance_id)
            .expect("Failed to clone ModuleInstance - memory allocation failed")
    }
}

impl PartialEq for ModuleInstance {
    fn eq(&self, other: &Self) -> bool {
        // Compare based on instance ID and module equality
        self.instance_id == other.instance_id && self.module == other.module
    }
}

impl Eq for ModuleInstance {}

/// Trait implementations for ModuleInstance to support BoundedMap usage
impl Checksummable for ModuleInstance {
    fn update_checksum(&self, checksum: &mut Checksum) {
        // Use instance ID and module checksum for unique identification
        checksum.update_slice(&self.instance_id.to_le_bytes());

        // Include module checksum if the module implements Checksummable
        // For now, use a simplified approach with module validation status
        if let Some(name) = self.module.name.as_ref() {
            if let Ok(name_str) = name.as_str() {
                checksum.update_slice(name_str.as_bytes());
            } else {
                checksum.update_slice(b"invalid_module_name");
            }
        } else {
            checksum.update_slice(b"unnamed_module_instance");
        }

        // Include counts of resources for uniqueness
        #[cfg(feature = "std")]
        let memories_count = self.memories.lock().map_or(0, |m| m.len()) as u32;
        #[cfg(not(feature = "std"))]
        let memories_count = self.memories.lock().len() as u32;

        #[cfg(feature = "std")]
        let tables_count = self.tables.lock().map_or(0, |t| t.len()) as u32;
        #[cfg(not(feature = "std"))]
        let tables_count = self.tables.lock().len() as u32;

        #[cfg(feature = "std")]
        let globals_count = self.globals.lock().map_or(0, |g| g.len()) as u32;
        #[cfg(not(feature = "std"))]
        let globals_count = self.globals.lock().len() as u32;

        checksum.update_slice(&memories_count.to_le_bytes());
        checksum.update_slice(&tables_count.to_le_bytes());
        checksum.update_slice(&globals_count.to_le_bytes());
    }
}

impl ToBytes for ModuleInstance {
    fn serialized_size(&self) -> usize {
        // Simplified size calculation for module instance metadata
        // instance_id (8) + resource counts (12) + module name length estimation (64)
        8 + 12 + 64
    }

    fn to_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut WriteStream<'a>,
        _provider: &PStream,
    ) -> Result<()> {
        // Write instance ID
        writer.write_all(&self.instance_id.to_le_bytes())?;

        // Write resource counts
        #[cfg(feature = "std")]
        let memories_count = self.memories.lock().map_or(0, |m| m.len()) as u32;
        #[cfg(not(feature = "std"))]
        let memories_count = self.memories.lock().len() as u32;

        #[cfg(feature = "std")]
        let tables_count = self.tables.lock().map_or(0, |t| t.len()) as u32;
        #[cfg(not(feature = "std"))]
        let tables_count = self.tables.lock().len() as u32;

        #[cfg(feature = "std")]
        let globals_count = self.globals.lock().map_or(0, |g| g.len()) as u32;
        #[cfg(not(feature = "std"))]
        let globals_count = self.globals.lock().len() as u32;

        writer.write_all(&memories_count.to_le_bytes())?;
        writer.write_all(&tables_count.to_le_bytes())?;
        writer.write_all(&globals_count.to_le_bytes())?;

        // Write module name (simplified)
        if let Some(name) = self.module.name.as_ref() {
            if let Ok(name_str) = name.as_str() {
                let name_bytes = name_str.as_bytes();
                writer.write_all(&(name_bytes.len() as u32).to_le_bytes())?;
                writer.write_all(name_bytes)?;
            } else {
                // Write zero length for invalid name
                writer.write_all(&0u32.to_le_bytes())?;
            }
        } else {
            // Write zero length for no name
            writer.write_all(&0u32.to_le_bytes())?;
        }

        Ok(())
    }
}

impl FromBytes for ModuleInstance {
    fn from_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        reader: &mut ReadStream<'a>,
        _provider: &PStream,
    ) -> Result<Self> {
        // Read instance ID
        let mut instance_id_bytes = [0u8; 8];
        reader.read_exact(&mut instance_id_bytes)?;
        let instance_id = usize::from_le_bytes(instance_id_bytes);

        // Read resource counts (for validation, but create empty collections)
        let mut counts = [0u8; 12];
        reader.read_exact(&mut counts)?;

        // Read module name length
        let mut name_len_bytes = [0u8; 4];
        reader.read_exact(&mut name_len_bytes)?;
        let name_len = u32::from_le_bytes(name_len_bytes) as usize;

        // Skip reading the name for now (simplified implementation)
        if name_len > 0 {
            #[cfg(any(feature = "std", feature = "alloc"))]
            {
                #[cfg(feature = "std")]
                let mut name_bytes = std::vec![0u8; name_len];
                #[cfg(all(feature = "alloc", not(feature = "std")))]
                let mut name_bytes = alloc::vec![0u8; name_len];
                reader.read_exact(&mut name_bytes)?;
            }
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            {
                // In no_std without alloc, we can't allocate the buffer
                // Just skip the bytes by reading them one by one
                for _ in 0..name_len {
                    let mut byte = [0u8; 1];
                    reader.read_exact(&mut byte)?;
                }
            }
        }

        // Create a default module instance with empty collections using create_runtime_provider
        // This is a simplified implementation - in a real scenario,
        // you'd need to reconstruct the actual module
        let provider = crate::bounded_runtime_infra::create_runtime_provider()?;

        let default_module = Module {
            types: Vec::new(),
            imports: wrt_foundation::bounded_collections::BoundedMap::new(provider.clone())?,
            functions: Vec::new(),
            tables: wrt_foundation::bounded::BoundedVec::new(provider.clone())?,
            memories: Vec::new(),
            globals: wrt_foundation::bounded::BoundedVec::new(provider.clone())?,
            elements: wrt_foundation::bounded::BoundedVec::new(provider.clone())?,
            data: wrt_foundation::bounded::BoundedVec::new(provider.clone())?,
            start: None,
            custom_sections: wrt_foundation::bounded_collections::BoundedMap::new(provider.clone())?,
            exports: wrt_foundation::direct_map::DirectMap::new(),
            name: None,
            binary: None,
            validated: false,
        };

        // Create the instance using the new method
        Self::new(Arc::new(default_module), instance_id).map_err(|_| {
            wrt_error::Error::runtime_error("Failed to create module instance from bytes")
        })
    }
}
