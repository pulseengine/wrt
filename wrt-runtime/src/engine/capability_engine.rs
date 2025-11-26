//! Capability-aware WebAssembly execution engine
//!
//! This module implements an execution engine that uses memory capabilities
//! to enforce safety constraints based on the selected preset (QM, ASIL-A,
//! ASIL-B).

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::sync::Arc;
use core::sync::atomic::{
    AtomicU32,
    Ordering,
};
#[cfg(feature = "std")]
use alloc::sync::Arc;

// Import decoder function
use wrt_decoder::decoder::decode_module;
// Import execution configuration from wrt-foundation where it belongs
use wrt_foundation::execution::{
    extract_resource_limits_from_binary,
    ASILExecutionConfig,
    ASILExecutionMode,
};
use wrt_foundation::{
    bounded_collections::BoundedMap,
    budget_aware_provider::CrateId,
    capabilities::{
        MemoryCapabilityContext,
        MemoryOperation,
    },
    direct_map::DirectMap,
    safe_managed_alloc,
    safe_memory::NoStdProvider,
    traits::{
        ReadStream,
        WriteStream,
    },
    values::Value,
};
use wrt_host::{
    BoundedHostIntegrationManager,
    CallbackRegistry,
    HostBuilder,
    HostIntegrationLimits,
};

use crate::{
    bounded_runtime_infra::BaseRuntimeProvider,
    module::Module,
    module_instance::ModuleInstance,
    prelude::*,
    stackless::StacklessEngine,
};

/// Handle for a loaded module
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ModuleHandle(u32);

impl ModuleHandle {
    /// Create a new unique module handle
    pub fn new() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl wrt_foundation::traits::Checksummable for ModuleHandle {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        for byte in self.0.to_le_bytes() {
            checksum.update(byte);
        }
    }
}

impl wrt_foundation::traits::ToBytes for ModuleHandle {
    fn serialized_size(&self) -> usize {
        4
    }

    fn to_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut WriteStream<'a>,
        _provider: &PStream,
    ) -> Result<()> {
        writer.write_u32_le(self.0)
    }
}

impl wrt_foundation::traits::FromBytes for ModuleHandle {
    fn from_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        reader: &mut ReadStream<'a>,
        _provider: &PStream,
    ) -> Result<Self> {
        let value = reader.read_u32_le()?;
        Ok(Self(value))
    }
}

/// Handle for a module instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct InstanceHandle(u32);

impl InstanceHandle {
    /// Create from an instance index
    pub fn from_index(idx: usize) -> Self {
        Self(idx as u32)
    }

    /// Get the instance index
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl wrt_foundation::traits::Checksummable for InstanceHandle {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        for byte in self.0.to_le_bytes() {
            checksum.update(byte);
        }
    }
}

impl wrt_foundation::traits::ToBytes for InstanceHandle {
    fn serialized_size(&self) -> usize {
        4
    }

    fn to_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut WriteStream<'a>,
        _provider: &PStream,
    ) -> Result<()> {
        writer.write_u32_le(self.0)
    }
}

impl wrt_foundation::traits::FromBytes for InstanceHandle {
    fn from_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        reader: &mut ReadStream<'a>,
        _provider: &PStream,
    ) -> Result<Self> {
        let value = reader.read_u32_le()?;
        Ok(Self(value))
    }
}

/// Engine preset determining capability configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnginePreset {
    /// Quality Management - Dynamic allocation, flexible limits
    QM,
    /// ASIL-A - Bounded collections, sampling verification
    AsilA,
    /// ASIL-B - Static allocation, continuous verification
    AsilB,
    /// ASIL-C - Enhanced verification, strict memory bounds
    AsilC,
    /// ASIL-D - Deterministic execution, compile-time verification
    AsilD,
}

/// Trait for capability-aware execution engines
pub trait CapabilityEngine: Send + Sync {
    /// Get the capability context for this engine
    fn capability_context(&self) -> &MemoryCapabilityContext;

    /// Load a module with capability verification
    fn load_module(&mut self, binary: &[u8]) -> Result<ModuleHandle>;

    /// Instantiate a module with capability-gated resources
    fn instantiate(&mut self, module: ModuleHandle) -> Result<InstanceHandle>;

    /// Link an import from one module to an export from an instantiated instance
    /// This must be called after the providing instance is instantiated but before
    /// the importing module is instantiated
    fn link_import(
        &mut self,
        module: ModuleHandle,
        import_module: &str,
        import_name: &str,
        provider_instance: InstanceHandle,
        export_name: &str,
    ) -> Result<()>;

    /// Execute a function with capability enforcement
    fn execute(
        &mut self,
        instance: InstanceHandle,
        func: &str,
        args: &[Value],
    ) -> Result<Vec<Value>>;
}

/// Maximum number of modules and instances
const MAX_MODULES: usize = 32;
const MAX_INSTANCES: usize = 32;

/// Runtime memory provider for engine internals
use crate::bounded_runtime_infra::{
    RuntimeProvider,
    RUNTIME_MEMORY_SIZE,
};

/// Import link describes how to resolve an import
#[derive(Debug, Clone)]
struct ImportLink {
    provider_instance: InstanceHandle,
    export_name: String,
}

/// Capability-aware WebAssembly execution engine
pub struct CapabilityAwareEngine {
    /// Inner stackless execution engine
    inner:             StacklessEngine,
    /// Capability context for memory operations
    context:           MemoryCapabilityContext,
    /// Engine preset used for resource limit extraction
    preset:            EnginePreset,
    /// Loaded modules indexed by handle (using DirectMap to avoid serialization stack overflow)
    modules:           DirectMap<ModuleHandle, Arc<Module>, MAX_MODULES>,
    /// Module instances indexed by handle (using DirectMap to avoid serialization stack overflow)
    instances:         DirectMap<InstanceHandle, Arc<ModuleInstance>, MAX_INSTANCES>,
    /// Next instance index
    next_instance_idx: usize,
    /// Host function registry for WASI and custom host functions
    host_registry:     Option<CallbackRegistry>,
    /// Bounded host integration manager for safety-critical environments
    host_manager:      Option<BoundedHostIntegrationManager>,
    /// Import links: module_handle -> (module::name -> ImportLink)
    #[cfg(feature = "std")]
    import_links:      std::collections::HashMap<ModuleHandle, std::collections::HashMap<String, ImportLink>>,
    /// Instance handle to instance_idx mapping for cross-instance calls
    #[cfg(feature = "std")]
    handle_to_idx:     std::collections::HashMap<InstanceHandle, usize>,
}

impl CapabilityAwareEngine {
    /// Create an engine with a specific preset
    pub fn with_preset(preset: EnginePreset) -> Result<Self> {
        let context = match preset {
            EnginePreset::QM => super::presets::qm()?,
            EnginePreset::AsilA => super::presets::asil_a()?,
            EnginePreset::AsilB => super::presets::asil_b()?,
            EnginePreset::AsilC => super::presets::asil_c()?,
            EnginePreset::AsilD => super::presets::asil_d()?,
        };

        Self::with_context_and_preset(context, preset)
    }

    /// Create an engine with a specific capability context
    pub fn with_context(context: MemoryCapabilityContext) -> Result<Self> {
        Self::with_context_and_preset(context, EnginePreset::QM)
    }

    /// Create an engine with a specific capability context and preset
    pub fn with_context_and_preset(
        context: MemoryCapabilityContext,
        preset: EnginePreset,
    ) -> Result<Self> {
        // Initialize host integration based on preset
        let (host_registry, host_manager) = Self::create_host_integration(&preset)?;

        // Create DirectMaps for engine internal structures
        // DirectMap doesn't require providers and avoids serialization stack overflow
        let modules = DirectMap::new();
        let instances = DirectMap::new();

        // Create the inner stackless engine
        let mut inner_engine = StacklessEngine::new();

        // Pass the host registry to the inner engine if available
        #[cfg(feature = "std")]
        if let Some(ref registry) = host_registry {
            use std::sync::Arc as StdArc;
            inner_engine.set_host_registry(StdArc::new(registry.clone()));
        }

        Ok(Self {
            inner: inner_engine,
            context,
            preset,
            modules,
            instances,
            next_instance_idx: 0,
            host_registry,
            host_manager,
            #[cfg(feature = "std")]
            import_links: std::collections::HashMap::new(),
            #[cfg(feature = "std")]
            handle_to_idx: std::collections::HashMap::new(),
        })
    }

    /// Set the host function registry for WASI and custom host functions
    ///
    /// This allows updating the registry after engine creation, which is needed
    /// for component model instantiation where the registry is created separately.
    #[cfg(feature = "std")]
    pub fn set_host_registry(&mut self, registry: std::sync::Arc<wrt_host::CallbackRegistry>) {
        self.host_registry = Some((*registry).clone());
        self.inner.set_host_registry(registry);
    }

    /// Convert engine preset to ASIL execution mode
    fn preset_to_asil_mode(&self) -> ASILExecutionMode {
        match self.preset {
            EnginePreset::QM => ASILExecutionMode::QM,
            EnginePreset::AsilA => ASILExecutionMode::AsilA,
            EnginePreset::AsilB => ASILExecutionMode::AsilB,
            EnginePreset::AsilC => ASILExecutionMode::AsilC,
            EnginePreset::AsilD => ASILExecutionMode::AsilD,
        }
    }

    /// Create host integration components based on engine preset
    fn create_host_integration(
        preset: &EnginePreset,
    ) -> Result<(
        Option<CallbackRegistry>,
        Option<BoundedHostIntegrationManager>,
    )> {
        match preset {
            EnginePreset::QM => {
                // QM mode: Full host function support with standard limits
                #[cfg(feature = "std")]
                {
                    let builder = HostBuilder::new()
                        .with_component_name("wrt_qm_component")
                        .with_host_id("wrt_qm_host");
                    let registry = builder.build()?;
                    Ok((Some(registry), None))
                }
                #[cfg(not(feature = "std"))]
                {
                    let limits = HostIntegrationLimits::qnx();
                    let manager = BoundedHostIntegrationManager::new(limits)?;
                    Ok((None, Some(manager)))
                }
            },
            EnginePreset::AsilA => {
                // ASIL-A: Bounded host functions with embedded limits
                let limits = HostIntegrationLimits::embedded();
                let manager = BoundedHostIntegrationManager::new(limits)?;
                Ok((None, Some(manager)))
            },
            EnginePreset::AsilB | EnginePreset::AsilC => {
                // ASIL-B/C: Restricted host functions with strict limits
                let limits = HostIntegrationLimits::embedded();
                let manager = BoundedHostIntegrationManager::new(limits)?;
                Ok((None, Some(manager)))
            },
            EnginePreset::AsilD => {
                // ASIL-D: Minimal or no host functions for maximum safety
                let limits = HostIntegrationLimits::embedded();
                let manager = BoundedHostIntegrationManager::new(limits)?;
                Ok((None, Some(manager)))
            },
        }
    }

    /// Register a custom host function
    pub fn register_host_function<F>(
        &mut self,
        module_name: &str,
        func_name: &str,
        func: F,
    ) -> Result<()>
    where
        F: Fn(&[Value]) -> Result<Vec<Value>> + Send + Sync + Clone + 'static,
    {
        #[cfg(feature = "std")]
        {
            if let Some(ref mut registry) = self.host_registry {
                use wrt_host::CloneableFn;
                use core::any::Any;

                // Wrap the user's function to match the HostFunctionHandler signature
                // HostFunctionHandler expects: Fn(&mut dyn Any, Vec<Value>) -> Result<Vec<Value>>
                let wrapped = move |_engine: &mut dyn Any, args: Vec<Value>| -> Result<Vec<Value>> {
                    func(&args)
                };

                // Create a CloneableFn from the wrapped closure
                let handler = CloneableFn::new_with_args(wrapped);

                // Register with the CallbackRegistry
                registry.register_host_function(module_name, func_name, handler);

                eprintln!("[ENGINE] Registered host function: {}::{}", module_name, func_name);
                Ok(())
            } else {
                Err(Error::not_supported_unsupported_operation(
                    "Host functions not supported in this configuration",
                ))
            }
        }
        #[cfg(not(feature = "std"))]
        {
            if let Some(ref mut manager) = self.host_manager {
                use wrt_host::BoundedHostFunction;
                // TODO: Create BoundedHostFunction and add to manager
                // For now, return success as placeholder
                Ok(())
            } else {
                Err(Error::not_supported_unsupported_operation(
                    "Host functions not supported in this configuration",
                ))
            }
        }
    }

    /// Enable WASI support with the current capability constraints
    pub fn enable_wasi(&mut self) -> Result<()> {
        match self.preset {
            EnginePreset::QM | EnginePreset::AsilA => {
                // WASI is allowed in QM and ASIL-A modes
                self.register_wasi_functions()
            },
            EnginePreset::AsilB | EnginePreset::AsilC => {
                // Limited WASI in ASIL-B/C (e.g., only safe I/O operations)
                self.register_limited_wasi_functions()
            },
            EnginePreset::AsilD => {
                // No WASI in ASIL-D for maximum safety
                Err(Error::not_supported_unsupported_operation(
                    "WASI not supported in ASIL-D mode",
                ))
            },
        }
    }

    /// Register standard WASI functions
    fn register_wasi_functions(&mut self) -> Result<()> {
        // TODO: Implement WASI function registration
        // This would register functions like:
        // - fd_write, fd_read (file operations)
        // - proc_exit (process exit)
        // - random_get (random number generation)
        // etc.
        Ok(())
    }

    /// Register limited WASI functions for higher safety levels
    fn register_limited_wasi_functions(&mut self) -> Result<()> {
        // TODO: Implement limited WASI function registration
        // This would only register safe, deterministic functions like:
        // - proc_exit (process exit)
        // - limited memory operations
        // But NOT:
        // - File I/O operations
        // - Network operations
        // - Random number generation
        Ok(())
    }
}

impl CapabilityEngine for CapabilityAwareEngine {
    fn capability_context(&self) -> &MemoryCapabilityContext {
        &self.context
    }

    fn load_module(&mut self, binary: &[u8]) -> Result<ModuleHandle> {
        // Verify capability for module allocation
        let operation = MemoryOperation::Allocate { size: binary.len() };
        self.context.verify_operation(CrateId::Runtime, &operation)?;

        // Extract resource limits from binary if available
        let asil_mode = self.preset_to_asil_mode();
        let _resource_config =
            extract_resource_limits_from_binary(binary, asil_mode).unwrap_or(None); // Ignore errors, use defaults if extraction fails

        // TODO: Apply resource limits to execution context
        // This would integrate with the fuel async executor to enforce limits

        // Decode the module using wrt-decoder (Box to avoid stack overflow)
        let decoded = Box::new(decode_module(binary)?);

        // Convert to runtime module (pass by reference, returns Box<Module>)
        let runtime_module = Module::from_wrt_module(&*decoded)?;

        // TODO: Initialize data segments into memory
        // #[cfg(feature = "std")]
        // runtime_module.initialize_data_segments()?;

        // Stack pointer is now initialized early during global creation in from_wrt_module()
        // No need for late initialization here anymore

        // Create and store with unique handle (wrapped in Arc to avoid deep clones)
        let handle = ModuleHandle::new();
        #[cfg(feature = "std")]
        eprintln!("DEBUG load_module: About to Arc::new(*runtime_module)");
        let module_arc = Arc::new(*runtime_module);
        #[cfg(feature = "std")]
        eprintln!("DEBUG load_module: About to insert into modules map");
        self.modules.insert(handle, module_arc)?;
        #[cfg(feature = "std")]
        eprintln!("DEBUG load_module: Insert completed, returning handle");

        Ok(handle)
    }

    fn instantiate(&mut self, module_handle: ModuleHandle) -> Result<InstanceHandle> {
        // Get the module (DirectMap returns Option<&Arc<Module>>)
        let module_arc = self
            .modules
            .get(&module_handle)
            .ok_or_else(|| Error::resource_not_found("Module not found"))?;

        // Verify capability for instance allocation
        let operation = MemoryOperation::Allocate {
            size: core::mem::size_of::<ModuleInstance>(),
        };
        self.context.verify_operation(CrateId::Runtime, &operation)?;

        // Create module instance (clone the Arc, not the Module)
        let mut instance = ModuleInstance::new(module_arc.clone(), self.next_instance_idx)?;

        // TODO: Copy globals from module to instance (critical for stack pointer initialization!)
        // instance.populate_globals_from_module()?;

        // TODO: Copy memories from module to instance (critical for memory access!)
        // instance.populate_memories_from_module()?;

        // TODO: Initialize data segments into instance memory (critical for static data!)
        // #[cfg(feature = "std")]
        // instance.initialize_data_segments()?;

        // Apply import links if any exist for this module
        // We need the instance_idx before we can register links, so we'll do it after registration
        let pending_links = self.import_links.get(&module_handle).cloned();

        // Don't clone! Cloning creates a fresh empty instance, losing all our populate work
        let instance_arc = Arc::new(instance);

        // Register with inner engine
        let instance_idx = self.inner.set_current_module(instance_arc.clone())?;
        self.next_instance_idx += 1;

        // Store mapping (wrapped in Arc to avoid deep clones)
        let handle = InstanceHandle::from_index(instance_idx);
        self.instances.insert(handle, instance_arc)?;

        // Store handle -> instance_idx mapping for cross-instance calls
        #[cfg(feature = "std")]
        self.handle_to_idx.insert(handle, instance_idx);

        // Register import links with the inner engine
        #[cfg(feature = "std")]
        if let Some(links) = pending_links {
            eprintln!("[INSTANTIATE] Applying {} import links for instance {}", links.len(), instance_idx);
            for (import_key, link) in links {
                // Parse import_key (format: "module::name" or just "name")
                let (import_module, import_name) = if let Some(pos) = import_key.rfind("::") {
                    (import_key[..pos].to_string(), import_key[pos+2..].to_string())
                } else {
                    (String::new(), import_key.clone())
                };

                // Get the target instance_idx from our mapping
                let target_idx = self.handle_to_idx.get(&link.provider_instance)
                    .copied()
                    .ok_or_else(|| Error::resource_not_found("Provider instance not found"))?;

                eprintln!("[INSTANTIATE]   Linking {}::{} (in instance {}) -> instance {}.{}",
                         import_module, import_name, instance_idx, target_idx, link.export_name);

                self.inner.register_import_link(
                    instance_idx,
                    import_module,
                    import_name,
                    target_idx,
                    link.export_name,
                );
            }
        }

        // Run start function if present
        if let Some(start_idx) = module_arc.start {
            #[cfg(feature = "std")]
            eprintln!("[INSTANTIATE] Module has start function at index {}. Running automatically...", start_idx);
            self.inner.execute(instance_idx, start_idx as usize, vec![])?;
            #[cfg(feature = "std")]
            eprintln!("[INSTANTIATE] Start function completed");
        } else {
            #[cfg(feature = "std")]
            eprintln!("[INSTANTIATE] No start function in module");
        }

        Ok(handle)
    }

    #[cfg(feature = "std")]
    fn link_import(
        &mut self,
        module: ModuleHandle,
        import_module: &str,
        import_name: &str,
        provider_instance: InstanceHandle,
        export_name: &str,
    ) -> Result<()> {
        // Convert handles to instance IDs for StacklessEngine
        // ModuleHandle and InstanceHandle are both wrappers around usize
        let module_id = module.0 as usize;
        let provider_id = provider_instance.0 as usize;

        // Add to inner StacklessEngine's import_links
        // Key: (instance_id, import_module, import_name)
        // Value: (target_instance_id, export_name)
        #[cfg(feature = "std")]
        {
            self.inner.add_import_link(
                module_id,
                import_module.to_string(),
                import_name.to_string(),
                provider_id,
                export_name.to_string()
            );
        }

        // Also store in our own links map for tracking
        let import_key = if import_module.is_empty() {
            import_name.to_string()
        } else {
            format!("{}::{}", import_module, import_name)
        };

        let link = ImportLink {
            provider_instance,
            export_name: export_name.to_string(),
        };

        self.import_links
            .entry(module)
            .or_insert_with(std::collections::HashMap::new)
            .insert(import_key, link);

        #[cfg(feature = "std")]
        eprintln!("[LINK_IMPORT] Linked {}::{} -> instance {:?}.{}",
                 import_module, import_name, provider_instance, export_name);

        Ok(())
    }

    #[cfg(not(feature = "std"))]
    fn link_import(
        &mut self,
        _module: ModuleHandle,
        _import_module: &str,
        _import_name: &str,
        _provider_instance: InstanceHandle,
        _export_name: &str,
    ) -> Result<()> {
        // No-std mode: import linking not yet supported
        Err(Error::runtime_error("Import linking not supported in no_std mode"))
    }

    fn execute(
        &mut self,
        instance_handle: InstanceHandle,
        func_name: &str,
        args: &[Value],
    ) -> Result<Vec<Value>> {
        // Get the instance (DirectMap returns Option<&Arc<ModuleInstance>>)
        let instance = self
            .instances
            .get(&instance_handle)
            .ok_or_else(|| Error::resource_not_found("Instance not found"))?;

        // Check if this is a directly callable host function by name
        // This allows test code to directly invoke WASI functions
        #[cfg(feature = "std")]
        {
            // Try to split func_name into module::function format
            if func_name.contains("::") {
                let parts: Vec<&str> = func_name.split("::").collect();
                if parts.len() >= 2 {
                    let module_str = parts[0];
                    let func_str = parts[1..].join("::");

                    if let Some(ref registry) = self.host_registry {
                        if registry.has_host_function(module_str, &func_str) {
                            eprintln!("[DISPATCH] Direct call to host function: {}::{}", module_str, func_str);

                            let mut dummy_engine = ();
                            let results = registry.call_host_function(
                                &mut dummy_engine as &mut dyn core::any::Any,
                                module_str,
                                &func_str,
                                args.to_vec()
                            )?;

                            eprintln!("[DISPATCH] Host function returned {} values", results.len());
                            return Ok(results);
                        }
                    }
                }
            }

            // Also try wasi: prefix format like "wasi:cli/stdout@0.2.0"
            if func_name.starts_with("wasi:") {
                // Extract just the function name part (last component after #)
                if let Some(hash_pos) = func_name.rfind('#') {
                    let module_part = &func_name[..hash_pos];
                    let func_part = &func_name[hash_pos+1..];

                    if let Some(ref registry) = self.host_registry {
                        if registry.has_host_function(module_part, func_part) {
                            eprintln!("[DISPATCH] WASI host function call: {}#{}", module_part, func_part);

                            let mut dummy_engine = ();
                            let results = registry.call_host_function(
                                &mut dummy_engine as &mut dyn core::any::Any,
                                module_part,
                                func_part,
                                args.to_vec()
                            )?;

                            eprintln!("[DISPATCH] Host function returned {} values", results.len());
                            return Ok(results);
                        }
                    }
                }
            }
        }

        // Not a host function - execute normally
        // Find the function by name using the new function resolution
        let func_idx = instance.module().validate_function_call(func_name)?;

        #[cfg(feature = "std")]
        eprintln!("DEBUG CapabilityEngine::execute: func_idx={}", func_idx);

        // Set current module for execution (instance is already &Arc<ModuleInstance>)
        self.inner.set_current_module(Arc::clone(instance))?;

        #[cfg(feature = "std")]
        eprintln!("DEBUG CapabilityEngine::execute: about to call inner.execute");

        // Execute the function
        let results =
            self.inner.execute(instance_handle.index(), func_idx as usize, args.to_vec())?;

        Ok(results)
    }
}

impl CapabilityAwareEngine {
    /// Get the list of exported functions from an instance
    pub fn get_exported_functions(&self, instance_handle: InstanceHandle) -> Result<Vec<String>> {
        let instance = self
            .instances
            .get(&instance_handle)
            .ok_or_else(|| Error::resource_not_found("Instance not found"))?;

        let mut functions = Vec::new();
        // TODO: BoundedMap doesn't support iteration, so we can't list all exports
        // For now, return an empty list as a placeholder
        // In a real implementation, we'd need an iterator interface on BoundedMap
        Ok(functions)
    }

    /// Check if a function exists in an instance
    pub fn has_function(&self, instance_handle: InstanceHandle, func_name: &str) -> Result<bool> {
        let instance = self
            .instances
            .get(&instance_handle)
            .ok_or_else(|| Error::resource_not_found("Instance not found"))?;

        Ok(instance.module().find_function_by_name(func_name).is_some())
    }

    /// Get function signature by name
    /// Temporarily disabled due to type system complexity
    pub fn get_function_signature(
        &self,
        _instance_handle: InstanceHandle,
        _func_name: &str,
    ) -> Result<Option<wrt_foundation::types::FuncType>> {
        // TODO: Fix type system inconsistency between BaseRuntimeProvider and actual
        // module provider
        Ok(None)
    }

    /// Execute a function with additional capability validation
    pub fn execute_with_validation(
        &mut self,
        instance_handle: InstanceHandle,
        func_name: &str,
        args: &[wrt_foundation::values::Value],
    ) -> Result<Vec<wrt_foundation::values::Value>> {
        // Additional capability-based validation (DirectMap returns Option)
        let instance = self
            .instances
            .get(&instance_handle)
            .ok_or_else(|| Error::resource_not_found("Instance not found"))?;

        // Verify memory capability allows function execution
        // Note: Using read operation as placeholder since Execute variant doesn't exist
        let operation = wrt_foundation::capabilities::MemoryOperation::Read {
            offset: 0,
            len:    64, // Small placeholder size for function validation
        };
        self.context.verify_operation(
            wrt_foundation::budget_aware_provider::CrateId::Runtime,
            &operation,
        )?;

        // Execute the function
        self.execute(instance_handle, func_name, args)
    }
}

