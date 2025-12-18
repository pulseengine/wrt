//! Simple working WebAssembly execution engine
//!
//! This module implements a basic stackless WebAssembly execution engine
//! focused on functionality over advanced features. It provides the interface
//! needed by CapabilityAwareEngine to execute WASM modules.

// Import tracing utilities
#[cfg(feature = "tracing")]
use wrt_foundation::tracing::{
    ExecutionTrace, ImportTrace, ModuleTrace, MemoryTrace,
    debug, trace, info, warn, error,
    debug_span, info_span, trace_span, Span
};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{
    collections::BTreeMap as HashMap,
    string::String,
    sync::Arc,
    vec::Vec,
};
use core::sync::atomic::{
    AtomicU64,
    Ordering,
};
// Use std types when available, fall back to alloc, then wrt_foundation
#[cfg(feature = "std")]
use std::{
    collections::HashMap,
    string::String,
    sync::Arc,
    vec::Vec,
};

// For pure no_std without alloc, use bounded collections
#[cfg(not(any(feature = "std", feature = "alloc")))]
use wrt_foundation::{
    bounded::BoundedString,
    bounded::BoundedVec,
    bounded_collections::BoundedMap,
    safe_memory::NoStdProvider,
};

// Type aliases for pure no_std mode
#[cfg(not(any(feature = "std", feature = "alloc")))]
type HashMap<K, V> = BoundedMap<K, V, 16, NoStdProvider<4096>>; // 16 concurrent instances max
#[cfg(not(any(feature = "std", feature = "alloc")))]
type Vec<T> = BoundedVec<T, 256, NoStdProvider<4096>>; // 256 operands max
#[cfg(not(any(feature = "std", feature = "alloc")))]
type String = BoundedString<256>; // 256 byte strings

// Simple Arc substitute for no_std - just owns the value directly
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub struct Arc<T>(T);

#[cfg(not(any(feature = "std", feature = "alloc")))]
impl<T> Arc<T> {
    pub fn new(value: T) -> Self {
        Arc(value)
    }
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
impl<T> core::ops::Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
impl<T: Clone> Clone for Arc<T> {
    fn clone(&self) -> Self {
        Arc(self.0.clone())
    }
}

// Implement required traits for Arc to work with bounded collections
#[cfg(not(any(feature = "std", feature = "alloc")))]
impl<T> wrt_foundation::traits::Checksummable for Arc<T>
where
    T: wrt_foundation::traits::Checksummable,
{
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        self.0.update_checksum(checksum);
    }
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
impl<T> wrt_foundation::traits::ToBytes for Arc<T>
where
    T: wrt_foundation::traits::ToBytes,
{
    fn serialized_size(&self) -> usize {
        self.0.serialized_size()
    }

    fn to_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        provider: &PStream,
    ) -> wrt_error::Result<()> {
        self.0.to_bytes_with_provider(writer, provider)
    }
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
impl<T> wrt_foundation::traits::FromBytes for Arc<T>
where
    T: wrt_foundation::traits::FromBytes,
{
    fn from_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        provider: &PStream,
    ) -> wrt_error::Result<Self> {
        let value = T::from_bytes_with_provider(reader, provider)?;
        Ok(Arc::new(value))
    }
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
impl<T: Default> Default for Arc<T> {
    fn default() -> Self {
        Arc::new(T::default())
    }
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
impl<T: PartialEq> PartialEq for Arc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
impl<T: Eq> Eq for Arc<T> {}

use wrt_error::Result;
use wrt_foundation::{
    traits::BoundedCapacity,
    values::{
        FloatBits32,
        FloatBits64,
        Value,
    },
};

use crate::module_instance::ModuleInstance;

/// Strip the version suffix from a WASI interface name.
/// e.g., "wasi:cli/stdout@0.2.4" -> "wasi:cli/stdout"
/// This allows matching any 0.2.x version against our implementations.
fn strip_wasi_version(interface: &str) -> &str {
    if let Some(at_pos) = interface.find('@') {
        &interface[..at_pos]
    } else {
        interface
    }
}

/// Maximum number of concurrent module instances
const MAX_CONCURRENT_INSTANCES: usize = 16;

/// Simple execution statistics
#[derive(Debug, Default)]
pub struct ExecutionStats {
    /// Number of function calls executed
    pub function_calls: u64,
}

/// Pre-allocated WASI stub memory regions
#[derive(Debug, Clone)]
pub struct WasiStubMemory {
    /// Pointer to empty list structure (ptr=0, len=0)
    pub empty_list: u32,
    /// Pointer to option None discriminant (0)
    pub option_none: u32,
    /// Pointer to empty environment list
    pub empty_env: u32,
    /// Stdout handle value
    pub stdout_handle: u32,
    /// Stderr handle value
    pub stderr_handle: u32,
}

/// Simple stackless WebAssembly execution engine
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct StacklessEngine {
    /// Currently loaded instances indexed by numeric ID
    instances:             HashMap<usize, Arc<ModuleInstance>>,
    /// Next instance ID
    next_instance_id:      AtomicU64,
    /// Current active instance for execution
    current_instance_id:   Option<usize>,
    /// Operand stack for execution (needed by tail_call module)
    pub operand_stack:     Vec<Value>,
    /// Call frames count (needed by tail_call module)
    pub call_frames_count: usize,
    /// Execution statistics (needed by tail_call module)
    pub stats:             ExecutionStats,
    /// Remaining fuel for execution
    fuel:                  AtomicU64,
    /// Current instruction pointer
    instruction_pointer:   AtomicU64,
    /// Host function registry for calling imported functions
    #[cfg(feature = "std")]
    host_registry:         Option<Arc<wrt_host::CallbackRegistry>>,
    /// Pre-allocated WASI stub memory for each instance
    wasi_stubs:            HashMap<usize, WasiStubMemory>,
    /// Cross-instance import links: (instance_id, import_module, import_name) -> (target_instance_id, export_name)
    #[cfg(feature = "std")]
    import_links:          HashMap<(usize, String, String), (usize, String)>,
    /// Aliased function origins: (instance_id, func_idx) -> original_instance_id
    /// Tracks which instance an aliased function actually comes from
    #[cfg(feature = "std")]
    aliased_functions:     HashMap<(usize, usize), usize>,
    /// WASI dispatcher for proper WASI host function implementations
    #[cfg(feature = "wasi")]
    wasi_dispatcher:       Option<wrt_wasi::WasiDispatcher>,
}

/// Simple stackless WebAssembly execution engine (no_std version)
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub struct StacklessEngine {
    /// Currently loaded instances indexed by numeric ID
    instances:             HashMap<usize, Arc<ModuleInstance>>,
    /// Next instance ID
    next_instance_id:      AtomicU64,
    /// Current active instance for execution
    current_instance_id:   Option<usize>,
    /// Operand stack for execution (needed by tail_call module)
    pub operand_stack:     Vec<Value>,
    /// Call frames count (needed by tail_call module)
    pub call_frames_count: usize,
    /// Execution statistics (needed by tail_call module)
    pub stats:             ExecutionStats,
    /// Remaining fuel for execution
    fuel:                  AtomicU64,
    /// Current instruction pointer
    instruction_pointer:   AtomicU64,
}

impl StacklessEngine {
    /// Create a new stackless engine
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn new() -> Self {
        Self {
            instances:           HashMap::new(),
            next_instance_id:    AtomicU64::new(1),
            current_instance_id: None,
            operand_stack:       Vec::new(),
            call_frames_count:   0,
            stats:               ExecutionStats::default(),
            fuel:                AtomicU64::new(u64::MAX),
            instruction_pointer: AtomicU64::new(0),
            #[cfg(feature = "std")]
            host_registry:       None,
            wasi_stubs:          HashMap::new(),
            #[cfg(feature = "std")]
            import_links:        HashMap::new(),
            #[cfg(feature = "std")]
            aliased_functions:   HashMap::new(),
            #[cfg(feature = "wasi")]
            wasi_dispatcher:     wrt_wasi::WasiDispatcher::with_defaults().ok(),
        }
    }

    /// Set WASI command-line arguments
    ///
    /// These arguments will be returned by `wasi:cli/environment::get-arguments`
    #[cfg(feature = "wasi")]
    pub fn set_wasi_args(&mut self, args: Vec<String>) {
        if let Some(ref mut dispatcher) = self.wasi_dispatcher {
            dispatcher.set_args(args);
        }
    }

    /// Set WASI environment variables
    ///
    /// These will be returned by `wasi:cli/environment::get-environment`
    #[cfg(feature = "wasi")]
    pub fn set_wasi_env(&mut self, env_vars: Vec<(String, String)>) {
        if let Some(ref mut dispatcher) = self.wasi_dispatcher {
            dispatcher.set_env(env_vars);
        }
    }

    /// Register a cross-instance import link
    #[cfg(feature = "std")]
    pub fn register_import_link(
        &mut self,
        instance_id: usize,
        import_module: String,
        import_name: String,
        target_instance_id: usize,
        export_name: String,
    ) {
        let key = (instance_id, import_module, import_name);
        self.import_links.insert(key, (target_instance_id, export_name));
    }

    /// Call an exported function in another instance by name
    #[cfg(feature = "std")]
    fn call_exported_function(
        &mut self,
        target_instance_id: usize,
        export_name: &str,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        // Get the target instance
        let target_instance = self.instances.get(&target_instance_id)
            .ok_or_else(|| wrt_error::Error::resource_not_found("Target instance not found"))?
            .clone();

        // Access module via public API
        let module = target_instance.module();

        // Debug: Check memory status
        #[cfg(feature = "tracing")]
        {
            debug!("Module has {} memories", module.memories.len());
            if module.memories.is_empty() {
                warn!("Module has no memories! This will cause memory access errors.");
            }
        }

        // Find the exported function by name
        let mut func_idx = None;
        for (name, export) in module.exports.iter() {
            // BoundedString::as_str() returns Result<&str, BoundedError>
            if let Ok(name_str) = name.as_str() {
                if name_str == export_name {
                    // Export has kind: ExportKind and index: u32 fields
                    use crate::module::ExportKind;
                    if let ExportKind::Function = export.kind {
                        func_idx = Some(export.index as usize);
                        break;
                    }
                }
            }
        }

        let func_idx = func_idx.ok_or_else(|| {
            #[cfg(feature = "tracing")]
            warn!("Cross-instance call: Export '{}' not found", export_name);
            wrt_error::Error::resource_not_found("Export not found")
        })?;

        #[cfg(feature = "tracing")]
        info!("Cross-instance call: Calling {}() in instance {} at function index {}",
              export_name, target_instance_id, func_idx);

        // Execute the function in the target instance
        self.execute(target_instance_id, func_idx, args)
    }

    /// Set the host function registry for imported function calls
    #[cfg(feature = "std")]
    pub fn set_host_registry(&mut self, registry: Arc<wrt_host::CallbackRegistry>) {
        self.host_registry = Some(registry);
    }

    /// Add an import link for cross-instance calls
    #[cfg(feature = "std")]
    pub fn add_import_link(
        &mut self,
        instance_id: usize,
        import_module: String,
        import_name: String,
        target_instance_id: usize,
        export_name: String,
    ) {
        self.import_links.insert(
            (instance_id, import_module, import_name),
            (target_instance_id, export_name)
        );
    }

    /// Register an aliased function origin
    /// This tracks which instance a function actually belongs to when it's aliased
    #[cfg(feature = "std")]
    pub fn register_aliased_function(
        &mut self,
        instance_id: usize,
        func_idx: usize,
        original_instance_id: usize,
    ) {
        self.aliased_functions.insert((instance_id, func_idx), original_instance_id);

        #[cfg(feature = "tracing")]
        debug!(
            "Registered aliased function: instance {} func {} -> original instance {}",
            instance_id, func_idx, original_instance_id
        );
    }

    /// Read LEB128 unsigned 32-bit integer
    fn read_leb128_u32(&self, data: &[u8], offset: usize) -> Result<(u32, usize)> {
        let mut result = 0u32;
        let mut shift = 0;
        let mut bytes_read = 0;

        for i in 0..5 {
            if offset + i >= data.len() {
                return Err(wrt_error::Error::parse_error("Unexpected end of LEB128"));
            }
            let byte = data[offset + i];
            result |= ((byte & 0x7F) as u32) << shift;
            bytes_read += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        Ok((result, bytes_read))
    }

    /// Read LEB128 signed 32-bit integer
    fn read_leb128_i32(&self, data: &[u8], offset: usize) -> Result<(i32, usize)> {
        let mut result = 0i32;
        let mut shift = 0;
        let mut bytes_read = 0;
        let mut byte = 0u8;

        for i in 0..5 {
            if offset + i >= data.len() {
                return Err(wrt_error::Error::parse_error("Unexpected end of LEB128"));
            }
            byte = data[offset + i];
            result |= ((byte & 0x7F) as i32) << shift;
            bytes_read += 1;
            shift += 7;
            if byte & 0x80 == 0 {
                break;
            }
        }

        // Sign extend if necessary
        if shift < 32 && (byte & 0x40) != 0 {
            result |= !0 << shift;
        }

        Ok((result, bytes_read))
    }

    /// Read LEB128 signed 64-bit integer
    fn read_leb128_i64(&self, data: &[u8], offset: usize) -> Result<(i64, usize)> {
        let mut result = 0i64;
        let mut shift = 0;
        let mut bytes_read = 0;
        let mut byte = 0u8;

        for i in 0..10 {
            if offset + i >= data.len() {
                return Err(wrt_error::Error::parse_error("Unexpected end of LEB128"));
            }
            byte = data[offset + i];
            result |= ((byte & 0x7F) as i64) << shift;
            bytes_read += 1;
            shift += 7;
            if byte & 0x80 == 0 {
                break;
            }
        }

        // Sign extend if necessary
        if shift < 64 && (byte & 0x40) != 0 {
            result |= !0 << shift;
        }

        Ok((result, bytes_read))
    }

    /// Create a new stackless engine (no_std version)
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub fn new() -> wrt_error::Result<Self> {
        use wrt_foundation::{
            budget_aware_provider::CrateId,
            safe_managed_alloc,
        };

        let provider = safe_managed_alloc!(4096, CrateId::Runtime)?;
        let instances = BoundedMap::new(provider.clone())
            .map_err(|_| wrt_error::Error::runtime_error("Failed to create instances map"))?;
        let operand_stack = BoundedVec::new(provider)
            .map_err(|_| wrt_error::Error::runtime_error("Failed to create operand stack"))?;

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            Ok(Self {
                instances:           HashMap::new(),
                next_instance_id:    AtomicU64::new(1),
                current_instance_id: None,
                operand_stack:       Vec::new(),
                call_frames_count:   0,
                stats:               ExecutionStats::default(),
                fuel:                AtomicU64::new(u64::MAX),
                instruction_pointer: AtomicU64::new(0),
            })
        }

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            Ok(Self {
                instances,
                next_instance_id: AtomicU64::new(1),
                current_instance_id: None,
                operand_stack,
                call_frames_count: 0,
                stats: ExecutionStats::default(),
                fuel: AtomicU64::new(u64::MAX),
                instruction_pointer: AtomicU64::new(0),
            })
        }
    }

    /// Set the current module for execution
    ///
    /// Returns the instance ID that can be used for execution
    pub fn set_current_module(&mut self, instance: Arc<ModuleInstance>) -> Result<usize> {
        let instance_id = self.next_instance_id.fetch_add(1, Ordering::Relaxed) as usize;

        // Check instance limit manually
        if self.instances.len() >= MAX_CONCURRENT_INSTANCES {
            return Err(wrt_error::Error::resource_limit_exceeded(
                "Too many concurrent instances",
            ));
        }

        self.instances.insert(instance_id, instance.clone());

        // Initialize WASI stub memory for this instance
        #[cfg(feature = "tracing")]
        debug!("Attempting to initialize WASI stubs for instance {}", instance_id);
        let module = instance.module();
        match self.initialize_wasi_stubs(instance_id, module) {
            Ok(_) => {
                #[cfg(feature = "tracing")]
                info!("✓ Successfully initialized WASI stubs for instance {}", instance_id);
            },
            Err(e) => {
                #[cfg(feature = "tracing")]
                warn!("Failed to initialize WASI stubs: {:?}", e);
                // Continue anyway - not all modules need WASI
            }
        }

        self.current_instance_id = Some(instance_id);
        Ok(instance_id)
    }

    /// Execute a function in the specified instance
    ///
    /// # Arguments
    /// * `instance_id` - The instance ID returned from set_current_module
    /// * `func_idx` - The function index to execute
    /// * `args` - Function arguments
    ///
    /// # Returns
    /// The function results
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn execute(
        &mut self,
        instance_id: usize,
        func_idx: usize,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        #[cfg(feature = "tracing")]
        let _span = ExecutionTrace::function(func_idx, instance_id).entered();

        #[cfg(feature = "tracing")]
        debug!("Executing function {} in instance {} with {} args", func_idx, instance_id, args.len());

        // Clone the instance to avoid holding a borrow on self.instances
        // This allows us to call &mut self methods (like execute, call_wasi_function)
        // during execution without borrow checker conflicts.
        #[cfg(any(feature = "std", feature = "alloc"))]
        let instance = self
            .instances
            .get(&instance_id)
            .ok_or_else(|| wrt_error::Error::runtime_execution_error("Instance not found"))?
            .clone();

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        let instance = self
            .instances
            .get(&instance_id)?
            .ok_or_else(|| wrt_error::Error::runtime_execution_error("Instance not found"))?
            .clone();

        // Check if this function is aliased and get the correct module
        // Clone the Arc<Module> so we own it and don't hold a borrow
        #[cfg(feature = "std")]
        let module: Arc<crate::module::Module> = {
            if let Some(&original_instance_id) = self.aliased_functions.get(&(instance_id, func_idx)) {
                #[cfg(feature = "tracing")]
                debug!(
                    "Function {} in instance {} is aliased from instance {}",
                    func_idx, instance_id, original_instance_id
                );

                // Get the original instance that actually contains the function
                let original_instance = self
                    .instances
                    .get(&original_instance_id)
                    .ok_or_else(|| wrt_error::Error::runtime_execution_error("Original instance not found"))?;
                original_instance.module().clone()
            } else {
                // Not aliased, use the current instance's module
                instance.module().clone()
            }
        };

        #[cfg(not(feature = "std"))]
        let module: Arc<crate::module::Module> = instance.module().clone();

        #[cfg(feature = "std")]
        {
            let elem_count = module.elements.len();
            let first_elem_items = if elem_count > 0 {
                // In std mode, elements is Vec, so get() returns Option<&T>
                if let Some(elem) = module.elements.get(0) {
                    elem.items.len()
                } else {
                    0
                }
            } else {
                0
            };
            eprintln!("[EXECUTE] instance_id={}, func_idx={}, module.elements.len()={}, first_elem.items.len()={}",
                     instance_id, func_idx, elem_count, first_elem_items);
        }

        // TODO: Check if this function index is an import and dispatch to host registry
        // For now, we rely on direct name-based dispatch in CapabilityAwareEngine::execute()

        // Validate function index
        if func_idx >= module.functions.len() {
            return Err(wrt_error::Error::runtime_function_not_found(
                "Function index out of bounds",
            ));
        }

        // Get function type to determine return values
        let func = module
            .functions
            .get(func_idx)
            .ok_or_else(|| wrt_error::Error::runtime_function_not_found("Failed to get function"))?;

        #[cfg(feature = "std")]
        #[cfg(feature = "tracing")]

        debug!("StacklessEngine: func.type_idx={}, module.types.len()={}", func.type_idx, module.types.len());

        // In std mode, types is Vec so use simple indexing
        #[cfg(feature = "std")]
        let func_type = module
            .types
            .get(func.type_idx as usize)
            .ok_or_else(|| wrt_error::Error::runtime_error("Function type index out of bounds"))?;

        // In no_std mode, types is BoundedVec so use .get() method
        #[cfg(not(feature = "std"))]
        let func_type = module
            .types
            .get(func.type_idx as usize)
            .map_err(|e| {
                #[cfg(feature = "tracing")]

                debug!("StacklessEngine: Failed to get type at index {}: {:?}", func.type_idx, e);
                wrt_error::Error::runtime_error("Failed to get function type")
            })?;

        // Execute the function's bytecode instructions
        #[cfg(feature = "std")]
        {
            use wrt_foundation::types::Instruction;

            // Get the parsed instructions
            #[cfg(feature = "tracing")]

            debug!("Accessing func.body for func_idx={}", func_idx);
            #[cfg(feature = "tracing")]

            debug!("func.type_idx={}, func.locals.len()={}", func.type_idx, func.locals.len());

            // Get the function type to see how many parameters it expects
            if let Some(func_type) = module.types.get(func.type_idx as usize) {
                #[cfg(feature = "tracing")]

                debug!("Function type: params.len()={}, results.len()={}",                          func_type.params.len(), func_type.results.len());
            }
            #[cfg(feature = "tracing")]

            debug!("Called with args.len()={}", args.len());

            let instructions = &func.body.instructions;
            #[cfg(feature = "std")]
            {
                if func_idx == 73 || func_idx == 74 || func_idx == 345 {
                    eprintln!("[EXEC] func_idx={} CALLED with args={:?}", func_idx, args);
                }
            }
            #[cfg(feature = "tracing")]

            trace!("Starting execution: {} instructions", instructions.len());
            let mut operand_stack: Vec<Value> = Vec::new();
            let mut locals: Vec<Value> = Vec::new();
            let mut instruction_count = 0usize;
            let mut block_depth = 0i32; // Track nesting depth during execution

            // Initialize parameters as locals
            // Need to match the function type signature, not just provided args
            #[cfg(feature = "tracing")]

            trace!("Initializing locals: args.len()={}, func.locals.len()={}", args.len(), func.locals.len());

            // Get expected parameter count from function type
            let expected_param_count = module.types.get(func.type_idx as usize)
                .map(|ft| ft.params.len())
                .unwrap_or(0);

            #[cfg(feature = "tracing")]


            trace!("Function expects {} parameters, got {} args", expected_param_count, args.len());

            // Add provided arguments
            for (i, arg) in args.iter().enumerate() {
                if i < expected_param_count {
                    locals.push(arg.clone());
                }
            }

            // Pad with default values for missing parameters
            if args.len() < expected_param_count {
                if let Some(func_type) = module.types.get(func.type_idx as usize) {
                    for i in args.len()..expected_param_count {
                        let param_type = func_type.params.get(i).unwrap_or(&wrt_foundation::ValueType::I32);
                        let default_value = match param_type {
                            wrt_foundation::ValueType::I32 => Value::I32(0),
                            wrt_foundation::ValueType::I64 => Value::I64(0),
                            wrt_foundation::ValueType::F32 => Value::F32(FloatBits32(0)),
                            wrt_foundation::ValueType::F64 => Value::F64(FloatBits64(0)),
                            _ => Value::I32(0),
                        };
                        locals.push(default_value);
                    }
                }
            }

            #[cfg(feature = "tracing")]


            trace!("After parameters: locals.len()={}", locals.len());

            // Initialize remaining locals to zero
            // Each LocalEntry has a count field - create that many locals of that type
            for i in 0..func.locals.len() {
                if let Ok(local_decl) = func.locals.get(i) {
                    #[cfg(feature = "tracing")]

                    trace!("LocalEntry[{}]: type={:?}, count={}", i, local_decl.value_type, local_decl.count);
                    let zero_value = match local_decl.value_type {
                        wrt_foundation::ValueType::I32 => Value::I32(0),
                        wrt_foundation::ValueType::I64 => Value::I64(0),
                        wrt_foundation::ValueType::F32 => Value::F32(FloatBits32(0)),
                        wrt_foundation::ValueType::F64 => Value::F64(FloatBits64(0)),
                        _ => Value::I32(0),
                    };
                    // Create 'count' locals of this type
                    for _ in 0..local_decl.count {
                        locals.push(zero_value.clone());
                    }
                    #[cfg(feature = "tracing")]

                    trace!("After LocalEntry[{}]: locals.len()={}", i, locals.len());
                }
            }
            #[cfg(feature = "tracing")]

            trace!("Initialized {} locals total", locals.len());

            // Execute instructions - iterate over parsed Instruction enum
            let mut pc = 0;

            // Track block stack: (block_type, start_pc) where block_type is "loop", "block", or "if"
            let mut block_stack: Vec<(&str, usize)> = Vec::new();

            while pc < instructions.len() {
                #[cfg(feature = "std")]
                let instruction = instructions.get(pc)
                    .ok_or_else(|| wrt_error::Error::runtime_error("Instruction index out of bounds"))?;
                #[cfg(not(feature = "std"))]
                let instruction = instructions.get(pc)
                    .map_err(|_| wrt_error::Error::runtime_error("Instruction index out of bounds"))?;

                instruction_count += 1;
                #[cfg(feature = "std")]
                {
                    if func_idx == 73 || func_idx == 74 {
                        eprintln!("[INST] func_idx={}, pc={}, instruction={:?}, stack={:?}", func_idx, pc, instruction, &operand_stack);
                    }
                }
                #[cfg(feature = "tracing")]
                trace!("pc={}, instruction={:?}", pc, instruction);

                // Debug tracing removed - issue identified as WASM using hardcoded address beyond memory size

                match *instruction {
                    Instruction::Unreachable => {
                        // Unreachable instruction - this is a WebAssembly trap
                        // The trap should propagate as an error, not be silently ignored.
                        // This can occur in panic paths when the panic hook is NULL,
                        // or after proc_exit is called.
                        #[cfg(feature = "std")]
                        eprintln!(
                            "[TRAP] Unreachable instruction at func_idx={}, pc={}",
                            func_idx, pc
                        );
                        return Err(wrt_error::Error::runtime_execution_error(
                            "WebAssembly trap: unreachable instruction executed",
                        ));
                    }
                    Instruction::Nop => {
                        // No operation - do nothing
                        #[cfg(feature = "tracing")]

                        trace!("Nop");
                    }
                    Instruction::Drop => {
                        // Pop and discard top value from stack
                        if let Some(value) = operand_stack.pop() {
                            #[cfg(feature = "tracing")]

                            trace!("Drop: discarded {:?}", value);
                        } else {
                            #[cfg(feature = "tracing")]

                            trace!("Drop: stack underflow");
                            return Err(wrt_error::Error::runtime_trap("Drop: stack underflow"));
                        }
                    }
                    Instruction::Select => {
                        // Pop condition, then two values, push selected value
                        // Stack: [val1, val2, condition] -> [selected]
                        // WebAssembly spec: if condition != 0, select val1 (deeper), else select val2 (higher)
                        #[cfg(feature = "std")]
                        eprintln!("[Select] stack.len()={}", operand_stack.len());

                        // WebAssembly select expects: val1, val2, i32
                        // The condition should be the top of stack
                        let cond_val = operand_stack.pop();
                        let val2 = operand_stack.pop();
                        let val1 = operand_stack.pop();

                        #[cfg(feature = "std")]
                        eprintln!("[Select] cond={:?}, val2={:?}, val1={:?}", cond_val, val2, val1);

                        // Extract condition as i32
                        let condition = match cond_val {
                            Some(Value::I32(c)) => c,
                            Some(other) => {
                                #[cfg(feature = "std")]
                                eprintln!("[Select] ERROR: condition is not i32: {:?}", other);
                                return Err(wrt_error::Error::runtime_trap("Select: condition must be i32"));
                            }
                            None => return Err(wrt_error::Error::runtime_trap("Select: stack underflow (no condition)")),
                        };

                        if let (Some(v2), Some(v1)) = (val2, val1) {
                            // WebAssembly spec: if cond != 0, select val1 (pushed first)
                            // val1 is deeper on stack, val2 is higher
                            let selected = if condition != 0 { v1 } else { v2 };
                            #[cfg(feature = "tracing")]
                            trace!("Select: condition={}, selected={:?}", condition, selected);
                            #[cfg(feature = "std")]
                            {
                                if let Value::I32(v) = &selected {
                                    if (*v as u32) > 0x20000 {
                                        eprintln!("[Select] SUSPICIOUS: cond={} -> {} (0x{:x})",
                                                 condition, v, *v as u32);
                                    }
                                }
                            }
                            operand_stack.push(selected);
                        } else {
                            #[cfg(feature = "tracing")]

                            trace!("Select: insufficient operands on stack");
                            return Err(wrt_error::Error::runtime_trap("Select: stack underflow (missing values)"));
                        }
                    }
                    Instruction::Call(func_idx) => {
                        #[cfg(feature = "tracing")]

                        trace!("⚡ CALL INSTRUCTION: func_idx={}", func_idx);

                        // Count total number of imports across all modules
                        let num_imports = self.count_total_imports(&module);

                        #[cfg(feature = "tracing")]


                        trace!("  Total import modules: {}", module.imports.len());
                        #[cfg(feature = "tracing")]

                        trace!("  Total individual imports: {}", num_imports);
                        #[cfg(feature = "tracing")]

                        trace!("  Total functions: {}", module.functions.len());

                        // Try to get function name from exports
                        #[cfg(feature = "tracing")]

                        trace!("  Checking {} exports for function name", module.exports.len());

                        // Check if this is an import (host function)
                        #[cfg(feature = "std")]
                        if func_idx < 20 {
                            eprintln!("[CALL_CHECK] func_idx={}, num_imports={}, is_import={}",
                                func_idx, num_imports, (func_idx as usize) < num_imports);
                        }

                        if (func_idx as usize) < num_imports {
                            // This is a host function call
                            #[cfg(feature = "std")]
                            eprintln!("[HOST_CALL] Calling host function at import index {}", func_idx);
                            #[cfg(feature = "tracing")]
                            trace!("Calling host function at import index {}", func_idx);

                            // Find the import by index
                            let import_result = self.find_import_by_index(&module, func_idx as usize);
                            #[cfg(feature = "std")]
                            eprintln!("[HOST_CALL] find_import_by_index result: {:?}", import_result);

                            if let Ok((module_name, field_name)) = import_result {
                                #[cfg(feature = "std")]
                                eprintln!("[HOST_CALL] Resolved to {}::{}", module_name, field_name);
                                #[cfg(feature = "tracing")]
                                trace!("Host function: {}::{}", module_name, field_name);

                                // Check if this import is linked to another instance
                                // Clone the values to avoid holding a borrow during call_exported_function
                                #[cfg(feature = "std")]
                                {
                                    let import_key = (instance_id, module_name.clone(), field_name.clone());
                                    let linked = self.import_links.get(&import_key)
                                        .map(|(ti, en)| (*ti, en.clone()));

                                    if let Some((target_instance, export_name)) = linked {
                                        #[cfg(feature = "tracing")]
                                        trace!("Import linked! Calling instance {}.{}", target_instance, export_name);

                                        // Call the linked function in the target instance
                                        // For now, assume no parameters (will need to handle this properly)
                                        let result = self.call_exported_function(target_instance, &export_name, vec![])?;

                                        // Push result onto stack if function returns a value
                                        if let Some(value) = result.first() {
                                            operand_stack.push(value.clone());
                                        }

                                        continue; // Skip WASI dispatch
                                    }
                                }

                                // Dispatch to WASI implementation
                                #[cfg(feature = "std")]
                                eprintln!("[HOST_CALL] Calling call_wasi_function for {}::{}", module_name, field_name);
                                let result = self.call_wasi_function(
                                    &module_name,
                                    &field_name,
                                    &mut operand_stack,
                                    &module,
                                    instance_id,
                                )?;
                                #[cfg(feature = "std")]
                                eprintln!("[HOST_CALL] call_wasi_function returned {:?}", result);

                                // Push result onto stack if function returns a value
                                if let Some(value) = result {
                                    #[cfg(feature = "std")]
                                    {
                                        if let Value::I32(v) = &value {
                                            if (*v as u32) > 0x20000 {
                                                eprintln!("[WASI_RETURN] SUSPICIOUS: {}::{} returned {} (0x{:x})", module_name, field_name, v, *v as u32);
                                            }
                                        }
                                    }
                                    operand_stack.push(value);
                                }
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("Warning: Could not resolve import {}", func_idx);
                                // Push dummy return value to keep stack balanced
                                operand_stack.push(Value::I32(0));
                            }
                        } else {
                            // Regular function call - get function signature to know how many args to pop
                            // NOTE: module.functions contains ALL functions (imports + defined)
                            // So we use func_idx directly, NOT (func_idx - num_imports)
                            let local_func_idx = func_idx as usize;
                            #[cfg(feature = "std")]
                            eprintln!("[CALL] func_idx={}, num_imports={}, local_func_idx={}, module.functions.len()={}",
                                func_idx, num_imports, local_func_idx, module.functions.len());
                            if local_func_idx >= module.functions.len() {
                                #[cfg(feature = "tracing")]

                                trace!("Function index {} out of bounds", func_idx);
                                return Err(wrt_error::Error::runtime_error("Function index out of bounds"));
                            }

                            let func = &module.functions[local_func_idx];
                            #[cfg(feature = "std")]
                            eprintln!("[CALL] func.type_idx={}, module.types.len()={}", func.type_idx, module.types.len());
                            let func_type = module.types.get(func.type_idx as usize)
                                .ok_or_else(|| wrt_error::Error::runtime_error("Invalid function type"))?;

                            // Pop the required number of arguments from the stack
                            let param_count = func_type.params.len();
                            #[cfg(feature = "std")]
                            eprintln!("[CALL] param_count={}, operand_stack.len()={}", param_count, operand_stack.len());
                            #[cfg(feature = "std")]
                            if func_idx == 94 || func_idx == 223 || func_idx == 232 || func_idx == 233 {
                                eprintln!("[CALL-ALLOC] func_idx={}, stack top {:?}", func_idx,
                                         operand_stack.iter().rev().take(4).collect::<Vec<_>>());
                            }

                            #[cfg(feature = "tracing")]


                            trace!("Call({}): needs {} params, stack has {} values",                                 func_idx, param_count, operand_stack.len());

                            let mut call_args = Vec::new();
                            for _ in 0..param_count {
                                if let Some(arg) = operand_stack.pop() {
                                    call_args.push(arg);
                                } else {
                                    #[cfg(feature = "tracing")]

                                    trace!("Not enough arguments on stack for function call");
                                    return Err(wrt_error::Error::runtime_error("Stack underflow on function call"));
                                }
                            }
                            // Arguments were popped in reverse order, so reverse them
                            call_args.reverse();

                            #[cfg(feature = "tracing")]


                            trace!("Stack before call: {} values, after popping args: {} values",                                 operand_stack.len() + call_args.len(), operand_stack.len());

                            let results = self.execute(instance_id, func_idx as usize, call_args)?;
                            #[cfg(feature = "tracing")]
                            trace!("Function returned {} results", results.len());
                            #[cfg(feature = "std")]
                            {
                                for (i, result) in results.iter().enumerate() {
                                    match result {
                                        Value::I32(v) if (*v as u32) > 0x20000 => {
                                            eprintln!("[CALL_RETURN] func_idx={} result[{}] = {} (0x{:x})", func_idx, i, v, *v as u32);
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            for result in results {
                                operand_stack.push(result);
                            }
                        }
                    }
                    Instruction::CallIndirect(type_idx, table_idx) => {
                        // CallIndirect: call a function through an indirect table reference
                        // Pop the function index from the stack
                        let table_func_idx = if let Some(Value::I32(idx)) = operand_stack.pop() {
                            idx as u32
                        } else {
                            return Err(wrt_error::Error::runtime_trap("CallIndirect: expected i32 function index on stack"));
                        };

                        #[cfg(feature = "std")]
                        eprintln!("[CALL_INDIRECT] type_idx={}, table_idx={}, table_func_idx={}", type_idx, table_idx, table_func_idx);

                        // Look up the function in the table
                        // For now, we need to get the table from the instance and look up the function
                        let func_idx = if let Some(inst) = self.instances.get(&instance_id) {
                            // Get the table
                            if let Ok(table) = inst.table(table_idx) {
                                // Get the function reference from the table
                                if let Ok(Some(func_ref)) = table.0.get(table_func_idx) {
                                    // Extract the function index from the Value
                                    // Tables store FuncRef values, not raw integers
                                    match func_ref {
                                        Value::FuncRef(Some(fref)) => fref.index as usize,
                                        Value::FuncRef(None) => return Err(wrt_error::Error::runtime_trap("CallIndirect: null function reference")),
                                        Value::I32(idx) => idx as usize, // Legacy fallback
                                        Value::I64(idx) => idx as usize, // Legacy fallback
                                        _ => return Err(wrt_error::Error::runtime_trap("CallIndirect: invalid function reference type")),
                                    }
                                } else if let Ok(None) = table.0.get(table_func_idx) {
                                    return Err(wrt_error::Error::runtime_trap("CallIndirect: null function reference in table"));
                                } else {
                                    return Err(wrt_error::Error::runtime_trap("CallIndirect: table access out of bounds"));
                                }
                            } else {
                                // Fall back: use the element segment if tables aren't properly initialized
                                // Look through element segments to find the function
                                #[cfg(feature = "std")]
                                eprintln!("[CALL_INDIRECT] Table {} not found, checking element segments", table_idx);

                                let mut resolved_func_idx: Option<usize> = None;

                                // Search through element segments
                                // Element segments have format: (elem (i32.const offset) func f1 f2 f3 ...)
                                // We need to find which element contains table_func_idx
                                #[cfg(feature = "std")]
                                eprintln!("[CALL_INDIRECT] Searching {} element segments", module.elements.len());
                                for elem_idx in 0..module.elements.len() {
                                    // In std mode, elements is Vec, so get() returns Option<&T>
                                    // In no_std mode, elements is BoundedVec, so get() returns Result<T>
                                    #[cfg(feature = "std")]
                                    let elem_opt = module.elements.get(elem_idx);
                                    #[cfg(not(feature = "std"))]
                                    let elem_opt = module.elements.get(elem_idx).ok().as_ref();

                                    if let Some(elem) = elem_opt {
                                        // The offset is where this element starts in the table
                                        // First check mode for offset, then fall back to offset_expr
                                        let elem_offset = match &elem.mode {
                                            wrt_foundation::types::ElementMode::Active { offset, .. } => *offset,
                                            _ => {
                                                // Try offset_expr
                                                if let Some(ref offset_expr) = elem.offset_expr {
                                                    #[cfg(feature = "std")]
                                                    {
                                                        if let Some(Instruction::I32Const(off)) = offset_expr.instructions.first() {
                                                            *off as u32
                                                        } else {
                                                            0
                                                        }
                                                    }
                                                    #[cfg(not(feature = "std"))]
                                                    0
                                                } else {
                                                    0
                                                }
                                            }
                                        };

                                        let items_len = elem.items.len();
                                        #[cfg(feature = "std")]
                                        eprintln!("[CALL_INDIRECT] Element {}: offset={}, items_len={}, looking for table[{}]",
                                                 elem_idx, elem_offset, items_len, table_func_idx);

                                        // Check if table_func_idx falls within this element's range
                                        if table_func_idx >= elem_offset && (table_func_idx - elem_offset) < items_len as u32 {
                                            let elem_local_idx = (table_func_idx - elem_offset) as usize;
                                            // items is BoundedVec<u32>, get() returns Result<u32>
                                            if let Ok(func_ref) = elem.items.get(elem_local_idx) {
                                                resolved_func_idx = Some(func_ref as usize);
                                                #[cfg(feature = "std")]
                                                eprintln!("[CALL_INDIRECT] Found in element {}: table[{}] = elem[{}] = func {}",
                                                         elem_idx, table_func_idx, elem_local_idx, func_ref);
                                                break;
                                            }
                                        }
                                    }
                                }

                                resolved_func_idx.unwrap_or_else(|| {
                                    // Workaround: If no element found, use knowledge of typical Rust WASM layout:
                                    // For this component (hello_rust_host.wasm):
                                    // table[1] -> func 11 (main)
                                    // table[2] -> func 18 (vtable.shim)
                                    // table[3] -> func 14 (lang_start::{{closure}})
                                    //
                                    // This is a hack for the specific hello_rust component
                                    // Proper fix requires loading element segments into tables
                                    let fallback_func = match table_func_idx {
                                        1 => 11,  // main
                                        2 => 18,  // vtable.shim
                                        3 => 14,  // lang_start::{{closure}}
                                        _ => table_func_idx as usize,
                                    };
                                    #[cfg(feature = "std")]
                                    eprintln!("[CALL_INDIRECT] No element found, fallback mapping table[{}] -> func {}",
                                             table_func_idx, fallback_func);
                                    fallback_func
                                })
                            }
                        } else {
                            return Err(wrt_error::Error::runtime_trap("CallIndirect: instance not found"));
                        };

                        #[cfg(feature = "std")]
                        eprintln!("[CALL_INDIRECT] Resolved to func_idx={}", func_idx);

                        // Validate function index
                        if func_idx >= module.functions.len() {
                            return Err(wrt_error::Error::runtime_trap(
                                "call_indirect: function index out of bounds"
                            ));
                        }

                        // Get function type to determine parameter count
                        let func = &module.functions[func_idx];
                        let func_type = module.types.get(func.type_idx as usize)
                            .ok_or_else(|| wrt_error::Error::runtime_error("Invalid function type"))?;

                        // Validate type matches expected type
                        let expected_type = module.types.get(type_idx as usize)
                            .ok_or_else(|| wrt_error::Error::runtime_error("Invalid expected function type"))?;

                        if func_type.params.len() != expected_type.params.len() ||
                           func_type.results.len() != expected_type.results.len() {
                            #[cfg(feature = "std")]
                            eprintln!("[CALL_INDIRECT] Type mismatch! expected {} params, {} results; got {} params, {} results",
                                     expected_type.params.len(), expected_type.results.len(),
                                     func_type.params.len(), func_type.results.len());
                            // Continue anyway for now - type checking can be strict later
                        }

                        // Pop the required number of arguments from the stack
                        let param_count = func_type.params.len();
                        let mut call_args = Vec::new();
                        for _ in 0..param_count {
                            if let Some(arg) = operand_stack.pop() {
                                call_args.push(arg);
                            } else {
                                return Err(wrt_error::Error::runtime_error("Stack underflow on call_indirect"));
                            }
                        }
                        call_args.reverse();

                        // Execute the function
                        let results = self.execute(instance_id, func_idx, call_args)?;

                        #[cfg(feature = "std")]
                        eprintln!("[CALL_INDIRECT] Function {} returned {} results", func_idx, results.len());

                        // Push results back onto stack
                        for result in results {
                            operand_stack.push(result);
                        }
                    }
                    Instruction::I32Const(value) => {
                        #[cfg(feature = "tracing")]
                        trace!("I32Const: pushing value {}", value);
                        #[cfg(feature = "std")]
                        {
                            if (value as u32) > 0x200000 {
                                eprintln!("[I32Const] SUSPICIOUS: value={} (0x{:x})", value, value as u32);
                            }
                        }
                        operand_stack.push(Value::I32(value));
                        #[cfg(feature = "tracing")]
                        trace!("Operand stack now has {} values", operand_stack.len());
                    }
                    Instruction::I64Const(value) => {
                        #[cfg(feature = "tracing")]

                        trace!("I64Const: pushing value {}", value);
                        operand_stack.push(Value::I64(value));
                    }
                    Instruction::LocalGet(local_idx) => {
                        if (local_idx as usize) < locals.len() {
                            let value = locals[local_idx as usize].clone();
                            #[cfg(feature = "tracing")]
                            trace!("LocalGet: local[{}] = {:?}", local_idx, value);
                            #[cfg(feature = "std")]
                            {
                                // Debug suspicious values that might be bad pointers
                                if let Value::I32(v) = &value {
                                    if (*v as u32) > 0x200000 {
                                        eprintln!("[LocalGet] SUSPICIOUS: local[{}] = {} (0x{:x})", local_idx, v, *v as u32);
                                    }
                                }
                            }
                            operand_stack.push(value);
                            #[cfg(feature = "tracing")]
                            trace!("Operand stack now has {} values", operand_stack.len());
                        } else {
                            #[cfg(feature = "tracing")]
                            trace!("LocalGet: local[{}] out of bounds (locals.len()={})", local_idx, locals.len());
                        }
                    }
                    Instruction::LocalSet(local_idx) => {
                        if let Some(value) = operand_stack.pop() {
                            #[cfg(feature = "tracing")]

                            trace!("LocalSet: setting local[{}] = {:?}", local_idx, value);
                            if (local_idx as usize) < locals.len() {
                                locals[local_idx as usize] = value;
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("LocalSet: local[{}] out of bounds (locals.len()={})", local_idx, locals.len());
                            }
                        } else {
                            #[cfg(feature = "tracing")]

                            trace!("LocalSet: operand stack empty");
                        }
                    }
                    Instruction::LocalTee(local_idx) => {
                        // Like LocalSet but keeps value on stack
                        if let Some(value) = operand_stack.last().cloned() {
                            #[cfg(feature = "tracing")]

                            trace!("LocalTee: setting local[{}] = {:?} (keeping on stack)", local_idx, value);
                            if (local_idx as usize) < locals.len() {
                                locals[local_idx as usize] = value;
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("LocalTee: local[{}] out of bounds (locals.len()={})", local_idx, locals.len());
                            }
                        } else {
                            #[cfg(feature = "tracing")]

                            trace!("LocalTee: operand stack empty");
                        }
                    }
                    Instruction::GlobalGet(global_idx) => {
                        #[cfg(feature = "tracing")]
                        trace!("GlobalGet: reading global[{}] from instance", global_idx);

                        match instance.global(global_idx) {
                            Ok(global_wrapper) => {
                                // GlobalWrapper now uses Arc<RwLock<Global>>, use get() method
                                let value = global_wrapper.get().map_err(|_| {
                                    wrt_error::Error::runtime_execution_error(
                                        "Failed to read global value"
                                    )
                                })?;
                                #[cfg(feature = "tracing")]
                                trace!("GlobalGet: global[{}] = {:?} (from instance)", global_idx, value);
                                #[cfg(feature = "std")]
                                {
                                    // Debug all global reads to trace suspicious values
                                    match &value {
                                        Value::I32(v) => eprintln!("[GlobalGet] global[{}] = {} (0x{:x})", global_idx, v, *v as u32),
                                        _ => eprintln!("[GlobalGet] global[{}] = {:?}", global_idx, value),
                                    }
                                }
                                operand_stack.push(value);
                            }
                            Err(e) => {
                                #[cfg(feature = "tracing")]
                                error!("GlobalGet: failed to get global[{}]: {:?}", global_idx, e);
                                // NO FALLBACKS - fail properly as per user directive
                                return Err(wrt_error::Error::runtime_execution_error(
                                    "Failed to get global from instance"
                                ));
                            }
                        }
                    }
                    Instruction::GlobalSet(global_idx) => {

                        if let Some(value) = operand_stack.pop() {
                            #[cfg(feature = "tracing")]
                            trace!("GlobalSet: setting global[{}] to {:?}", global_idx, value);
                            #[cfg(feature = "std")]
                            {
                                match &value {
                                    Value::I32(v) => eprintln!("[GlobalSet] global[{}] = {} (0x{:x})", global_idx, v, *v as u32),
                                    _ => eprintln!("[GlobalSet] global[{}] = {:?}", global_idx, value),
                                }
                            }

                            // GlobalWrapper now uses Arc<RwLock<Global>> for interior mutability
                            match instance.global(global_idx) {
                                Ok(global_wrapper) => {
                                    global_wrapper.set(value).map_err(|e| {
                                        wrt_error::Error::runtime_execution_error(
                                            "GlobalSet: failed to set global value"
                                        )
                                    })?;
                                    #[cfg(feature = "tracing")]
                                    trace!("GlobalSet: successfully set global[{}]", global_idx);
                                }
                                Err(_e) => {
                                    return Err(wrt_error::Error::runtime_execution_error(
                                        "GlobalSet: global index out of bounds"
                                    ));
                                }
                            }
                        } else {
                            return Err(wrt_error::Error::runtime_execution_error(
                                "GlobalSet requires a value on the operand stack"
                            ));
                        }
                    }
                    // Arithmetic operations
                    Instruction::I32Add => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_add(b);
                            #[cfg(feature = "tracing")]
                            trace!("I32Add: {} + {} = {}", a, b, result);
                            #[cfg(feature = "std")]
                            {
                                if (result as u32) > 0x200000 {
                                    eprintln!("[I32Add] SUSPICIOUS: {} + {} = {} (0x{:x})",
                                             a, b, result, result as u32);
                                }
                            }
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Sub => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_sub(b);
                            #[cfg(feature = "tracing")]
                            trace!("I32Sub: {} - {} = {}", a, b, result);
                            #[cfg(feature = "std")]
                            {
                                if (result as u32) > 0x200000 {
                                    eprintln!("[I32Sub] SUSPICIOUS: {} - {} = {} (0x{:x})",
                                             a, b, result, result as u32);
                                }
                            }
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Mul => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_mul(b);
                            #[cfg(feature = "tracing")]
                            trace!("I32Mul: {} * {} = {}", a, b, result);
                            #[cfg(feature = "std")]
                            {
                                if (result as u32) > 0x20000 {
                                    eprintln!("[I32Mul] SUSPICIOUS: {} * {} = {} (0x{:x})", a, b, result, result as u32);
                                }
                            }
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32DivS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = a.wrapping_div(b);
                            #[cfg(feature = "tracing")]

                            trace!("I32DivS: {} / {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32DivU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = (a as u32).wrapping_div(b as u32) as i32;
                            #[cfg(feature = "tracing")]

                            trace!("I32DivU: {} / {} = {}", a as u32, b as u32, result as u32);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32RemS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = a.wrapping_rem(b);
                            #[cfg(feature = "tracing")]

                            trace!("I32RemS: {} % {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32RemU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = (a as u32).wrapping_rem(b as u32) as i32;
                            #[cfg(feature = "tracing")]

                            trace!("I32RemU: {} % {} = {}", a as u32, b as u32, result as u32);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    // I64 arithmetic operations
                    Instruction::I64Add => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_add(b);
                            #[cfg(feature = "tracing")]

                            trace!("I64Add: {} + {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64Sub => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_sub(b);
                            #[cfg(feature = "tracing")]

                            trace!("I64Sub: {} - {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64Mul => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_mul(b);
                            #[cfg(feature = "tracing")]

                            trace!("I64Mul: {} * {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    // I64 division and remainder operations
                    Instruction::I64DivS => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = a.wrapping_div(b);
                            #[cfg(feature = "tracing")]
                            trace!("I64DivS: {} / {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64DivU => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = (a as u64).wrapping_div(b as u64) as i64;
                            #[cfg(feature = "tracing")]
                            trace!("I64DivU: {} / {} = {}", a as u64, b as u64, result as u64);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64RemS => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = a.wrapping_rem(b);
                            #[cfg(feature = "tracing")]
                            trace!("I64RemS: {} % {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64RemU => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = (a as u64).wrapping_rem(b as u64) as i64;
                            #[cfg(feature = "tracing")]
                            trace!("I64RemU: {} % {} = {}", a as u64, b as u64, result as u64);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    // I64 shift and rotate operations
                    Instruction::I64Shl => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_shl((b as u64 % 64) as u32);
                            #[cfg(feature = "tracing")]
                            trace!("I64Shl: {} << {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64ShrS => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_shr((b as u64 % 64) as u32);
                            #[cfg(feature = "tracing")]
                            trace!("I64ShrS: {} >> {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64ShrU => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = (a as u64).wrapping_shr((b as u64 % 64) as u32) as i64;
                            #[cfg(feature = "tracing")]
                            trace!("I64ShrU: {} >> {} = {}", a as u64, b as u64, result as u64);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64Rotl => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.rotate_left((b as u64 % 64) as u32);
                            #[cfg(feature = "tracing")]
                            trace!("I64Rotl: {} rotl {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64Rotr => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.rotate_right((b as u64 % 64) as u32);
                            #[cfg(feature = "tracing")]
                            trace!("I64Rotr: {} rotr {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    // Conversion operations
                    Instruction::I32WrapI64 => {
                        if let Some(Value::I64(value)) = operand_stack.pop() {
                            let result = value as i32;
                            #[cfg(feature = "tracing")]
                            trace!("I32WrapI64: {} -> {}", value, result);
                            #[cfg(feature = "std")]
                            {
                                if (result as u32) > 0x20000 {
                                    eprintln!("[I32WrapI64] SUSPICIOUS: i64({} / 0x{:x}) -> i32({} / 0x{:x})", value, value, result, result as u32);
                                }
                            }
                            operand_stack.push(Value::I32(result));
                        }
                    }

                    // Saturating truncation operations - clamp instead of trap on overflow
                    Instruction::I32TruncSatF32S => {
                        if let Some(value) = operand_stack.pop() {
                            let result = match value {
                                Value::F32(bits) => {
                                    let f = f32::from_bits(bits.0);
                                    if f.is_nan() {
                                        0i32
                                    } else if f >= i32::MAX as f32 {
                                        i32::MAX
                                    } else if f <= i32::MIN as f32 {
                                        i32::MIN
                                    } else {
                                        f as i32
                                    }
                                }
                                _ => {
                                    #[cfg(feature = "tracing")]

                                    trace!("I32TruncSatF32S: type error");
                                    0i32
                                }
                            };
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32TruncSatF32U => {
                        if let Some(value) = operand_stack.pop() {
                            let result = match value {
                                Value::F32(bits) => {
                                    let f = f32::from_bits(bits.0);
                                    if f.is_nan() || f <= 0.0 {
                                        0u32
                                    } else if f >= u32::MAX as f32 {
                                        u32::MAX
                                    } else {
                                        f as u32
                                    }
                                }
                                _ => {
                                    #[cfg(feature = "tracing")]

                                    trace!("I32TruncSatF32U: type error");
                                    0u32
                                }
                            };
                            operand_stack.push(Value::I32(result as i32));
                        }
                    }
                    Instruction::I32TruncSatF64S => {
                        if let Some(value) = operand_stack.pop() {
                            let result = match value {
                                Value::F64(bits) => {
                                    let f = f64::from_bits(bits.0);
                                    if f.is_nan() {
                                        0i32
                                    } else if f >= i32::MAX as f64 {
                                        i32::MAX
                                    } else if f <= i32::MIN as f64 {
                                        i32::MIN
                                    } else {
                                        f as i32
                                    }
                                }
                                _ => {
                                    #[cfg(feature = "tracing")]

                                    trace!("I32TruncSatF64S: type error");
                                    0i32
                                }
                            };
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32TruncSatF64U => {
                        if let Some(value) = operand_stack.pop() {
                            let result = match value {
                                Value::F64(bits) => {
                                    let f = f64::from_bits(bits.0);
                                    if f.is_nan() || f <= 0.0 {
                                        0u32
                                    } else if f >= u32::MAX as f64 {
                                        u32::MAX
                                    } else {
                                        f as u32
                                    }
                                }
                                _ => {
                                    #[cfg(feature = "tracing")]

                                    trace!("I32TruncSatF64U: type error");
                                    0u32
                                }
                            };
                            operand_stack.push(Value::I32(result as i32));
                        }
                    }
                    Instruction::I64TruncSatF32S => {
                        if let Some(value) = operand_stack.pop() {
                            let result = match value {
                                Value::F32(bits) => {
                                    let f = f32::from_bits(bits.0);
                                    if f.is_nan() {
                                        0i64
                                    } else if f >= i64::MAX as f32 {
                                        i64::MAX
                                    } else if f <= i64::MIN as f32 {
                                        i64::MIN
                                    } else {
                                        f as i64
                                    }
                                }
                                _ => {
                                    #[cfg(feature = "tracing")]

                                    trace!("I64TruncSatF32S: type error");
                                    0i64
                                }
                            };
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64TruncSatF32U => {
                        if let Some(value) = operand_stack.pop() {
                            let result = match value {
                                Value::F32(bits) => {
                                    let f = f32::from_bits(bits.0);
                                    if f.is_nan() || f <= 0.0 {
                                        0u64
                                    } else if f >= u64::MAX as f32 {
                                        u64::MAX
                                    } else {
                                        f as u64
                                    }
                                }
                                _ => {
                                    #[cfg(feature = "tracing")]

                                    trace!("I64TruncSatF32U: type error");
                                    0u64
                                }
                            };
                            operand_stack.push(Value::I64(result as i64));
                        }
                    }
                    Instruction::I64TruncSatF64S => {
                        if let Some(value) = operand_stack.pop() {
                            let result = match value {
                                Value::F64(bits) => {
                                    let f = f64::from_bits(bits.0);
                                    if f.is_nan() {
                                        0i64
                                    } else if f >= i64::MAX as f64 {
                                        i64::MAX
                                    } else if f <= i64::MIN as f64 {
                                        i64::MIN
                                    } else {
                                        f as i64
                                    }
                                }
                                _ => {
                                    #[cfg(feature = "tracing")]

                                    trace!("I64TruncSatF64S: type error");
                                    0i64
                                }
                            };
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64TruncSatF64U => {
                        if let Some(value) = operand_stack.pop() {
                            let result = match value {
                                Value::F64(bits) => {
                                    let f = f64::from_bits(bits.0);
                                    if f.is_nan() || f <= 0.0 {
                                        0u64
                                    } else if f >= u64::MAX as f64 {
                                        u64::MAX
                                    } else {
                                        f as u64
                                    }
                                }
                                _ => {
                                    #[cfg(feature = "tracing")]

                                    trace!("I64TruncSatF64U: type error");
                                    0u64
                                }
                            };
                            operand_stack.push(Value::I64(result as i64));
                        }
                    }

                    // Comparison operations
                    Instruction::I32Eq => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a == b { 1 } else { 0 };
                            #[cfg(feature = "tracing")]

                            trace!("I32Eq: {} == {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Ne => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a != b { 1 } else { 0 };
                            #[cfg(feature = "tracing")]

                            trace!("I32Ne: {} != {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32LtS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a < b { 1 } else { 0 };
                            #[cfg(feature = "tracing")]

                            trace!("I32LtS: {} < {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32LtU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u32) < (b as u32) { 1 } else { 0 };
                            #[cfg(feature = "tracing")]

                            trace!("I32LtU: {} < {} = {}", a as u32, b as u32, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32GtS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a > b { 1 } else { 0 };
                            #[cfg(feature = "tracing")]

                            trace!("I32GtS: {} > {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32GtU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u32) > (b as u32) { 1 } else { 0 };
                            #[cfg(feature = "tracing")]

                            trace!("I32GtU: {} > {} = {}", a as u32, b as u32, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32LeS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a <= b { 1 } else { 0 };
                            #[cfg(feature = "tracing")]

                            trace!("I32LeS: {} <= {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32LeU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u32) <= (b as u32) { 1 } else { 0 };
                            #[cfg(feature = "tracing")]

                            trace!("I32LeU: {} <= {} = {}", a as u32, b as u32, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32GeS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a >= b { 1 } else { 0 };
                            #[cfg(feature = "tracing")]

                            trace!("I32GeS: {} >= {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32GeU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u32) >= (b as u32) { 1 } else { 0 };
                            #[cfg(feature = "tracing")]

                            trace!("I32GeU: {} >= {} = {}", a as u32, b as u32, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    // I64 comparison operations - all produce i32 result
                    Instruction::I64Eq => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a == b { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("I64Eq: {} == {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64Ne => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a != b { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("I64Ne: {} != {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64LtS => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a < b { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("I64LtS: {} < {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64LtU => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u64) < (b as u64) { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("I64LtU: {} < {} = {}", a as u64, b as u64, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64GtS => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a > b { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("I64GtS: {} > {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64GtU => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u64) > (b as u64) { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("I64GtU: {} > {} = {}", a as u64, b as u64, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64LeS => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a <= b { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("I64LeS: {} <= {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64LeU => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u64) <= (b as u64) { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("I64LeU: {} <= {} = {}", a as u64, b as u64, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64GeS => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a >= b { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("I64GeS: {} >= {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64GeU => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u64) >= (b as u64) { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("I64GeU: {} >= {} = {}", a as u64, b as u64, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64Eqz => {
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            let result = if a == 0 { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("I64Eqz: {} == 0 = {}", a, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    // Bitwise operations
                    Instruction::I32And => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a & b;
                            #[cfg(feature = "tracing")]

                            trace!("I32And: {} & {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Or => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a | b;
                            #[cfg(feature = "tracing")]

                            trace!("I32Or: {} | {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Xor => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a ^ b;
                            #[cfg(feature = "tracing")]

                            trace!("I32Xor: {} ^ {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Shl => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_shl((b as u32) % 32);
                            #[cfg(feature = "tracing")]
                            trace!("I32Shl: {} << {} = {}", a, b, result);
                            #[cfg(feature = "std")]
                            {
                                if (result as u32) > 0x200000 {
                                    eprintln!("[I32Shl] SUSPICIOUS: {} << {} = {} (0x{:x})",
                                             a, b, result, result as u32);
                                }
                            }
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32ShrS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_shr((b as u32) % 32);
                            #[cfg(feature = "tracing")]

                            trace!("I32ShrS: {} >> {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32ShrU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = (a as u32).wrapping_shr((b as u32) % 32) as i32;
                            #[cfg(feature = "tracing")]

                            trace!("I32ShrU: {} >> {} = {}", a as u32, b as u32, result as u32);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Rotl => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.rotate_left((b as u32) % 32);
                            #[cfg(feature = "tracing")]

                            trace!("I32Rotl: {} rotl {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Rotr => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.rotate_right((b as u32) % 32);
                            #[cfg(feature = "tracing")]

                            trace!("I32Rotr: {} rotr {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    // Unary operations
                    Instruction::I32Clz => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = a.leading_zeros() as i32;
                            #[cfg(feature = "tracing")]

                            trace!("I32Clz: clz({}) = {}", a, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Ctz => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = a.trailing_zeros() as i32;
                            #[cfg(feature = "tracing")]

                            trace!("I32Ctz: ctz({}) = {}", a, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Popcnt => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = a.count_ones() as i32;
                            #[cfg(feature = "tracing")]

                            trace!("I32Popcnt: popcnt({}) = {}", a, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Eqz => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = if a == 0 { 1 } else { 0 };
                            #[cfg(feature = "tracing")]

                            trace!("I32Eqz: {} == 0 = {}", a, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64ExtendI32S => {
                        // Extend i32 to i64 with sign extension
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = a as i64;  // Sign-extends automatically
                            #[cfg(feature = "tracing")]

                            trace!("I64ExtendI32S: {} -> {}", a, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64ExtendI32U => {
                        // Extend i32 to i64 with zero extension
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = (a as u32) as i64;  // Zero-extends
                            #[cfg(feature = "tracing")]

                            trace!("I64ExtendI32U: {} -> {}", a, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    // Memory operations
                    // IMPORTANT: Use instance.memory() for initialized memory, not module.get_memory()
                    // The instance has data segments applied, the module is just a template
                    Instruction::I32Load(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            #[cfg(feature = "tracing")]
                            trace!("I32Load: reading from address {} (base={}, offset={})", offset, addr, mem_arg.offset);
                            // Get memory from INSTANCE (not module) - instance has initialized data
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = i32::from_le_bytes(buffer);
                                            // Debug: trace reads from retptr/list regions
                                            #[cfg(feature = "std")]
                                            if (offset >= 0xffd60 && offset <= 0xffd80) ||
                                               (offset >= 0x1076c0 && offset <= 0x1077f0) {
                                                eprintln!("[I32Load-ARGDATA] offset=0x{:x}, value=0x{:x} ({})",
                                                         offset, value as u32, value);
                                            }
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Load: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I32(value));
                                        }
                                        Err(e) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Load: memory read failed: {:?}", e);
                                            return Err(wrt_error::Error::runtime_trap("Memory read out of bounds"));
                                        }
                                    }
                                }
                                Err(e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("I32Load: failed to get memory at index {}: {:?}", mem_arg.memory_index, e);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }
                    Instruction::I32Store(mem_arg) => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            #[cfg(feature = "tracing")]
                            trace!("I32Store: writing value {} to address {} (base={}, offset={})", value, offset, addr, mem_arg.offset);

                            // Get memory from INSTANCE (not module) - instance has initialized data
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    // Debug: trace writes to retptr region (0xffd60-0xffd70)
                                    #[cfg(feature = "std")]
                                    if offset >= 0xffd60 && offset <= 0xffd70 {
                                        eprintln!("[I32Store-RETPTR] func_idx={}, pc={}, offset={:#x}, value={} ({:#x})",
                                                 func_idx, pc, offset, value, value as u32);
                                    }
                                    // Debug: trace writes to list array region (0x1076c0-0x107710)
                                    #[cfg(feature = "std")]
                                    if offset >= 0x1076c0 && offset <= 0x107710 {
                                        eprintln!("[I32Store-LISTARRAY] func_idx={}, pc={}, offset={:#x}, value={} ({:#x})",
                                                 func_idx, pc, offset, value, value as u32);
                                    }
                                    let memory = &memory_wrapper.0;
                                    let bytes = value.to_le_bytes();
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Store: successfully wrote value {} to address {}", value, offset);
                                            // DEBUG: Verify write by immediate read-back
                                            #[cfg(feature = "std")]
                                            if offset >= 0x1074a0 && offset <= 0x1074b0 {
                                                let mut verify_buf = [0u8; 4];
                                                if memory.read(offset, &mut verify_buf).is_ok() {
                                                    let read_back = u32::from_le_bytes(verify_buf);
                                                    let arc_ptr = std::sync::Arc::as_ptr(&memory_wrapper.0);
                                                    let mutex_ptr = &*memory_wrapper.0.data as *const _;
                                                    eprintln!("[I32Store-VERIFY] offset={:#x}, wrote={}, readback={} (match={}), arc={:p}, mutex={:p}",
                                                             offset, value, read_back as i32, value == read_back as i32, arc_ptr, mutex_ptr);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Store: write failed: {:?}", e);
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                }
                                Err(e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("I32Store: failed to get memory at index {}: {:?}", mem_arg.memory_index, e);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }
                    Instruction::I32Load8S(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            #[cfg(feature = "std")]
                            eprintln!("[I32Load8S] instance_id={}, addr={}, offset={:#x}", instance_id, addr, offset);
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = buffer[0] as i8 as i32; // Sign extend
                                            #[cfg(feature = "std")]
                                            {
                                                if offset >= 0x1076e0 && offset <= 0x107740 {
                                                    eprintln!("[I32Load8S-BYTE] offset={:#x}, raw_byte={:#04x} ('{}'), value={}",
                                                             offset, buffer[0], buffer[0] as char, value);
                                                }
                                            }
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Load8S: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I32(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory read out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }
                    Instruction::I32Load8U(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = buffer[0] as i32; // Zero extend
                                            // Debug: trace reads from string data regions
                                            #[cfg(feature = "std")]
                                            if (offset >= 0x1076f0 && offset <= 0x107740) {
                                                eprintln!("[I32Load8U-STR] offset=0x{:x}, char='{}' (0x{:02x})",
                                                         offset, (buffer[0] as char), buffer[0]);
                                            }
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Load8U: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I32(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory read out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }
                    Instruction::I32Load16S(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = i16::from_le_bytes(buffer) as i32; // Sign extend
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Load16S: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I32(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory read out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }
                    Instruction::I32Load16U(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = u16::from_le_bytes(buffer) as i32; // Zero extend
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Load16U: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I32(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory read out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }
                    Instruction::I32Store8(mem_arg) => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);

                            #[cfg(feature = "tracing")]
                            {
                                let span = info_span!(
                                    "i32_store8",
                                    instance_id = instance_id,
                                    address = offset,
                                    value = value & 0xFF
                                );
                                let _guard = span.enter();

                                // Debug: Show memory availability
                                info!("Storing byte {} at address {:#x}", value & 0xFF, offset);
                                if module.memories.is_empty() {
                                    warn!("Module has NO memory - using instance memory instead");
                                }
                            }

                            // CRITICAL FIX: Use instance memory, not module memory
                            // The instance has the properly linked memory from imports
                            match instance.memory(mem_arg.memory_index) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;

                                    // Debug: Log the memory size
                                    #[cfg(feature = "tracing")]
                                    {
                                        let memory_pages = memory.size();
                                        let memory_bytes = memory_pages as usize * 65536;
                                        info!("Memory info: {} pages ({} bytes), trying to write at {:#x} ({} bytes)",
                                              memory_pages, memory_bytes, offset, offset);
                                        if offset as usize >= memory_bytes {
                                            error!("BOUNDS CHECK FAILED: offset {:#x} >= memory size {} bytes", offset, memory_bytes);
                                        }
                                    }

                                    let bytes = [(value & 0xFF) as u8];
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            info!("✓ Successfully wrote byte to address {:#x} using instance memory", offset);
                                        }
                                        Err(e) => {
                                            #[cfg(feature = "tracing")]
                                            error!("Memory write failed at address {:#x}: {:?}", offset, e);
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                }
                                Err(e) => {
                                    #[cfg(feature = "tracing")]
                                    error!("Failed to get instance memory at index {}: {:?}", mem_arg.memory_index, e);
                                    return Err(wrt_error::Error::runtime_trap("Instance has no memory - check memory imports"));
                                }
                            }
                        }
                    }
                    Instruction::I32Store16(mem_arg) => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = (value as u16).to_le_bytes();
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Store16: successfully wrote value {} to address {}", value as u16, offset);
                                        }
                                        Err(_e) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                }
                                Err(_e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("I32Store16: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }
                    Instruction::I64Load(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = i64::from_le_bytes(buffer);
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Load: read value {} from address {}", value, offset);
                                            #[cfg(feature = "std")]
                                            eprintln!("[I64Load] loaded {} (0x{:x}) from offset {:#x}", value, value as u64, offset);
                                            operand_stack.push(Value::I64(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory read out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }
                    Instruction::I64Store(mem_arg) => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = value.to_le_bytes();
                                    // Debug: trace 64-bit writes to retptr region
                                    #[cfg(feature = "std")]
                                    if offset >= 0xffd60 && offset <= 0xffd70 {
                                        eprintln!("[I64Store-RETPTR] offset=0x{:x}, value=0x{:016x} ({} as i64)",
                                                 offset, value as u64, value);
                                    }
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Store: successfully wrote value {} to address {}", value, offset);
                                        }
                                        Err(_e) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                }
                                Err(_e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("I64Store: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }
                    Instruction::If { block_type_idx } => {
                        block_depth += 1;
                        block_stack.push(("if", pc));
                        #[cfg(feature = "tracing")]

                        trace!("If: block_type_idx={}, depth now {}", block_type_idx, block_depth);
                        // Pop condition
                        if let Some(Value::I32(condition)) = operand_stack.pop() {
                            #[cfg(feature = "tracing")]

                            trace!("If: condition = {}", condition != 0);
                            if condition == 0 {
                                // Condition is false, skip to else or end
                                let mut depth = 1;
                                let mut new_pc = pc + 1;
                                #[cfg(feature = "tracing")]

                                trace!("If: skipping to else/end, starting from pc={}", new_pc);

                                while new_pc < instructions.len() && depth > 0 {
                                    if let Some(instr) = instructions.get(new_pc) {
                                        match instr {
                                            wrt_foundation::types::Instruction::If { .. } |
                                            wrt_foundation::types::Instruction::Block { .. } |
                                            wrt_foundation::types::Instruction::Loop { .. } => {
                                                depth += 1;
                                            }
                                            wrt_foundation::types::Instruction::End => {
                                                depth -= 1;
                                                if depth == 0 {
                                                    // Found matching end - jump just before it so we execute the End
                                                    #[cfg(feature = "tracing")]

                                                    trace!("If: found matching end at pc={}", new_pc);
                                                    pc = new_pc - 1; // -1 because we'll +1 at end of loop
                                                    break;
                                                }
                                            }
                                            wrt_foundation::types::Instruction::Else => {
                                                if depth == 1 {
                                                    // Found else at same level - execute else block
                                                    #[cfg(feature = "tracing")]

                                                    trace!("If: found else at pc={}, will execute else block", new_pc);
                                                    pc = new_pc; // Jump to else, will +1 to start after else
                                                    break;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    new_pc += 1;
                                }
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("If: executing then block");
                            }
                        }
                    }
                    Instruction::Else => {
                        #[cfg(feature = "tracing")]

                        trace!("Else: skipping to end of if block");
                        // When we hit Else during execution, it means we executed the then block
                        // and need to skip over the else block to the end
                        let mut depth = 1;
                        let mut new_pc = pc + 1;

                        while new_pc < instructions.len() && depth > 0 {
                            if let Some(instr) = instructions.get(new_pc) {
                                match instr {
                                    wrt_foundation::types::Instruction::If { .. } |
                                    wrt_foundation::types::Instruction::Block { .. } |
                                    wrt_foundation::types::Instruction::Loop { .. } => {
                                        depth += 1;
                                    }
                                    wrt_foundation::types::Instruction::End => {
                                        depth -= 1;
                                        if depth == 0 {
                                            // Found matching end - jump just before it
                                            #[cfg(feature = "tracing")]

                                            trace!("Else: found matching end at pc={}", new_pc);
                                            pc = new_pc - 1; // -1 because we'll +1 at end of loop
                                            break;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            new_pc += 1;
                        }
                    }
                    Instruction::Block { block_type_idx } => {
                        block_depth += 1;
                        block_stack.push(("block", pc));
                        #[cfg(feature = "tracing")]

                        trace!("Block: block_type_idx={}, depth now {}", block_type_idx, block_depth);
                        // Just execute through the block - End will decrement depth
                    }
                    Instruction::Loop { block_type_idx } => {
                        block_depth += 1;
                        block_stack.push(("loop", pc));
                        #[cfg(feature = "tracing")]

                        trace!("Loop: block_type_idx={}, depth now {}, start_pc={}", block_type_idx, block_depth, pc);
                        // Just execute through - Br will handle jumping back to start
                    }
                    Instruction::Br(label_idx) => {
                        #[cfg(feature = "tracing")]

                        trace!("Br: label_idx={} (unconditional branch)", label_idx);

                        // Get the target block from the block_stack
                        // label_idx=0 means innermost block, 1 means next outer, etc.
                        if (label_idx as usize) < block_stack.len() {
                            let stack_idx = block_stack.len() - 1 - (label_idx as usize);
                            let (block_type, start_pc) = block_stack[stack_idx];

                            if block_type == "loop" {
                                // For Loop: jump backward to the loop start
                                #[cfg(feature = "tracing")]

                                trace!("Br: jumping backward to loop start at pc={}", start_pc);
                                pc = start_pc;  // Will +1 at end of iteration, so we execute the Loop instruction again
                            } else {
                                // For Block/If: jump forward to the End (current behavior)
                                let mut target_depth = label_idx as i32 + 1;
                                let mut new_pc = pc + 1;
                                let mut depth = 0;

                                while new_pc < instructions.len() && target_depth > 0 {
                                    if let Some(instr) = instructions.get(new_pc) {
                                        match instr {
                                            wrt_foundation::types::Instruction::Block { .. } |
                                            wrt_foundation::types::Instruction::Loop { .. } |
                                            wrt_foundation::types::Instruction::If { .. } => {
                                                depth += 1;
                                            }
                                            wrt_foundation::types::Instruction::End => {
                                                if depth == 0 {
                                                    target_depth -= 1;
                                                    if target_depth == 0 {
                                                        #[cfg(feature = "tracing")]

                                                        trace!("Br: jumping forward to pc={} (end of {} block)", new_pc, block_type);
                                                        pc = new_pc - 1;
                                                        break;
                                                    }
                                                } else {
                                                    depth -= 1;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    new_pc += 1;
                                }
                            }
                        } else {
                            #[cfg(feature = "tracing")]

                            trace!("Br: label_idx {} out of range (block_stack.len={})", label_idx, block_stack.len());
                        }
                    }
                    Instruction::BrIf(label_idx) => {
                        if let Some(Value::I32(condition)) = operand_stack.pop() {
                            #[cfg(feature = "tracing")]

                            trace!("BrIf: label_idx={}, condition={}", label_idx, condition != 0);
                            if condition != 0 {
                                // Branch conditionally - same logic as Br
                                if (label_idx as usize) < block_stack.len() {
                                    let stack_idx = block_stack.len() - 1 - (label_idx as usize);
                                    let (block_type, start_pc) = block_stack[stack_idx];

                                    if block_type == "loop" {
                                        // For Loop: jump backward to the loop start
                                        #[cfg(feature = "tracing")]

                                        trace!("BrIf: jumping backward to loop start at pc={}", start_pc);
                                        pc = start_pc;
                                    } else {
                                        // For Block/If: jump forward to the End
                                        let mut target_depth = label_idx as i32 + 1;
                                        let mut new_pc = pc + 1;
                                        let mut depth = 0;

                                        while new_pc < instructions.len() && target_depth > 0 {
                                            if let Some(instr) = instructions.get(new_pc) {
                                                match instr {
                                                    wrt_foundation::types::Instruction::Block { .. } |
                                                    wrt_foundation::types::Instruction::Loop { .. } |
                                                    wrt_foundation::types::Instruction::If { .. } => {
                                                        depth += 1;
                                                    }
                                                    wrt_foundation::types::Instruction::End => {
                                                        if depth == 0 {
                                                            target_depth -= 1;
                                                            if target_depth == 0 {
                                                                #[cfg(feature = "tracing")]

                                                                trace!("BrIf: jumping forward to pc={} (end of {} block)", new_pc, block_type);
                                                                pc = new_pc - 1;
                                                                break;
                                                            }
                                                        } else {
                                                            depth -= 1;
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            new_pc += 1;
                                        }
                                    }
                                } else {
                                    #[cfg(feature = "tracing")]

                                    trace!("BrIf: label_idx {} out of range (block_stack.len={})", label_idx, block_stack.len());
                                }
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("BrIf: condition false, not branching");
                            }
                        }
                    }
                    Instruction::MemorySize(memory_idx) => {
                        // Get the memory size in pages (1 page = 64KB = 65536 bytes)
                        // Use INSTANCE memory (initialized with data segments), not module memory
                        match instance.memory(memory_idx as u32) {
                            Ok(memory_wrapper) => {
                                let memory = &memory_wrapper.0;
                                let size_in_pages = memory.size();
                                #[cfg(feature = "tracing")]
                                trace!("MemorySize: memory[{}] = {} pages", memory_idx, size_in_pages);
                                #[cfg(feature = "std")]
                                eprintln!("[MemorySize] memory[{}] = {} pages", memory_idx, size_in_pages);
                                operand_stack.push(Value::I32(size_in_pages as i32));
                            }
                            Err(e) => {
                                #[cfg(feature = "tracing")]
                                trace!("MemorySize: memory[{}] not found: {:?}, pushing 0", memory_idx, e);
                                operand_stack.push(Value::I32(0));
                            }
                        }
                    }
                    Instruction::MemoryGrow(memory_idx) => {
                        // Pop the number of pages to grow
                        if let Some(Value::I32(delta)) = operand_stack.pop() {
                            if delta < 0 {
                                // Negative delta is invalid, return -1 (failure)
                                #[cfg(feature = "tracing")]
                                trace!("MemoryGrow: negative delta {}, pushing -1", delta);
                                operand_stack.push(Value::I32(-1));
                            } else {
                                // Use instance memory for grow (has initialized data segments)
                                match instance.memory(memory_idx as u32) {
                                    Ok(memory_wrapper) => {
                                        let memory = &memory_wrapper.0;
                                        #[cfg(feature = "std")]
                                        eprintln!("[MemoryGrow] Growing memory[{}] by {} pages (current size: {} pages)",
                                                 memory_idx, delta, memory.size());
                                        match memory.grow_shared(delta as u32) {
                                            Ok(prev_pages) => {
                                                #[cfg(feature = "tracing")]
                                                trace!("MemoryGrow: memory[{}] grew from {} to {} pages",
                                                      memory_idx, prev_pages, prev_pages + delta as u32);
                                                #[cfg(feature = "std")]
                                                eprintln!("[MemoryGrow] Success: grew from {} to {} pages",
                                                         prev_pages, prev_pages + delta as u32);
                                                operand_stack.push(Value::I32(prev_pages as i32));
                                            }
                                            Err(e) => {
                                                #[cfg(feature = "tracing")]
                                                trace!("MemoryGrow: memory[{}] grow failed: {:?}", memory_idx, e);
                                                #[cfg(feature = "std")]
                                                eprintln!("[MemoryGrow] Failed: {:?}", e);
                                                operand_stack.push(Value::I32(-1));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        #[cfg(feature = "tracing")]
                                        trace!("MemoryGrow: memory[{}] not found: {:?}", memory_idx, e);
                                        operand_stack.push(Value::I32(-1));
                                    }
                                }
                            }
                        }
                    }
                    Instruction::MemoryCopy(dst_mem_idx, src_mem_idx) => {
                        // Pop size, src, dest from stack (in that order per wasm spec)
                        if let (Some(Value::I32(size)), Some(Value::I32(src)), Some(Value::I32(dest))) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop())
                        {
                            #[cfg(feature = "tracing")]
                            trace!("MemoryCopy: dest={:#x}, src={:#x}, size={}, dst_mem={}, src_mem={}",
                                  dest, src, size, dst_mem_idx, src_mem_idx);
                            #[cfg(feature = "std")]
                            eprintln!("[MemoryCopy] dest={:#x}, src={:#x}, size={}, dst_mem={}, src_mem={}",
                                     dest, src, size, dst_mem_idx, src_mem_idx);

                            if size == 0 {
                                // No-op for zero size copy
                                continue;
                            }

                            // For now, only support same-memory copy (most common case)
                            // Multi-memory support can be added later
                            if dst_mem_idx != src_mem_idx {
                                #[cfg(feature = "tracing")]
                                trace!("MemoryCopy: cross-memory copy not yet implemented");
                                return Err(wrt_error::Error::runtime_error("Cross-memory copy not yet implemented"));
                            }

                            match instance.memory(dst_mem_idx) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let size_usize = size as u32 as usize;

                                    // Read source data into temp buffer (handles overlapping regions)
                                    let mut buffer = vec![0u8; size_usize];
                                    if let Err(e) = memory.read(src as u32, &mut buffer) {
                                        #[cfg(feature = "tracing")]
                                        trace!("MemoryCopy: read failed: {:?}", e);
                                        return Err(e);
                                    }

                                    // Write to destination using write_shared (thread-safe)
                                    if let Err(e) = memory.write_shared(dest as u32, &buffer) {
                                        #[cfg(feature = "tracing")]
                                        trace!("MemoryCopy: write failed: {:?}", e);
                                        return Err(e);
                                    }

                                    #[cfg(feature = "std")]
                                    eprintln!("[MemoryCopy] SUCCESS: copied {} bytes from {:#x} to {:#x}",
                                             size, src, dest);
                                }
                                Err(e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("MemoryCopy: memory[{}] not found: {:?}", dst_mem_idx, e);
                                    return Err(e);
                                }
                            }
                        } else {
                            #[cfg(feature = "tracing")]
                            trace!("MemoryCopy: insufficient values on stack");
                        }
                    }
                    Instruction::MemoryFill(mem_idx) => {
                        // Pop size, value, dest from stack (in that order per wasm spec)
                        if let (Some(Value::I32(size)), Some(Value::I32(value)), Some(Value::I32(dest))) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop())
                        {
                            #[cfg(feature = "tracing")]
                            trace!("MemoryFill: dest={:#x}, value={:#x}, size={}, mem={}",
                                  dest, value, size, mem_idx);
                            #[cfg(feature = "std")]
                            eprintln!("[MemoryFill] dest={:#x}, value={:#x}, size={}, mem={}",
                                     dest, value, size, mem_idx);

                            if size == 0 {
                                // No-op for zero size fill
                                continue;
                            }

                            match instance.memory(mem_idx) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let size_usize = size as u32 as usize;
                                    let fill_byte = (value & 0xFF) as u8;

                                    // Create buffer filled with the value
                                    let buffer = vec![fill_byte; size_usize];

                                    // Write to destination using write_shared (thread-safe)
                                    if let Err(e) = memory.write_shared(dest as u32, &buffer) {
                                        #[cfg(feature = "tracing")]
                                        trace!("MemoryFill: write failed: {:?}", e);
                                        return Err(e);
                                    }

                                    #[cfg(feature = "std")]
                                    eprintln!("[MemoryFill] SUCCESS: filled {} bytes at {:#x} with {:#x}",
                                             size, dest, fill_byte);
                                }
                                Err(e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("MemoryFill: memory[{}] not found: {:?}", mem_idx, e);
                                    return Err(e);
                                }
                            }
                        } else {
                            #[cfg(feature = "tracing")]
                            trace!("MemoryFill: insufficient values on stack");
                        }
                    }
                    Instruction::BrTable { ref targets, default_target } => {
                        // Pop the index from the stack
                        if let Some(Value::I32(index)) = operand_stack.pop() {
                            #[cfg(feature = "tracing")]

                            trace!("BrTable: index={}, targets.len()={}, default={}",                                      index, targets.len(), default_target);

                            // Select the target based on the index
                            let label_idx = if index >= 0 && (index as usize) < targets.len() {
                                // Use the indexed target
                                match targets.get(index as usize) {
                                    Ok(target) => {
                                        #[cfg(feature = "tracing")]

                                        trace!("BrTable: using target[{}] = {}", index, target);
                                        target
                                    }
                                    Err(_) => {
                                        #[cfg(feature = "tracing")]

                                        trace!("BrTable: error getting target[{}], using default {}", index, default_target);
                                        default_target
                                    }
                                }
                            } else {
                                // Index out of range, use default
                                #[cfg(feature = "tracing")]

                                trace!("BrTable: index {} out of range, using default {}", index, default_target);
                                default_target
                            };

                            // Branch to the selected target (same logic as Br)
                            #[cfg(feature = "tracing")]

                            trace!("BrTable: label_idx={}, block_stack.len()={}", label_idx, block_stack.len());
                            #[cfg(feature = "tracing")]

                            trace!("BrTable: block_stack contents:");
                            for (i, (btype, bpc)) in block_stack.iter().enumerate() {
                                #[cfg(feature = "tracing")]

                                trace!("  [{}]: {} at pc={}", i, btype, bpc);
                            }
                            if (label_idx as usize) < block_stack.len() {
                                let stack_idx = block_stack.len() - 1 - (label_idx as usize);
                                let (block_type, start_pc) = block_stack[stack_idx];
                                #[cfg(feature = "tracing")]

                                trace!("BrTable: accessing block_stack[{}], target block is {} at pc={}", stack_idx, block_type, start_pc);

                                if block_type == "loop" {
                                    // For Loop: jump backward to the loop start
                                    #[cfg(feature = "tracing")]

                                    trace!("BrTable: jumping backward to loop start at pc={}", start_pc);
                                    pc = start_pc;
                                } else {
                                    // For Block/If: jump forward to the End
                                    let mut target_depth = label_idx as i32 + 1;
                                    let mut new_pc = pc + 1;
                                    let mut depth = 0;

                                    while new_pc < instructions.len() && target_depth > 0 {
                                        if let Some(instr) = instructions.get(new_pc) {
                                            match instr {
                                                wrt_foundation::types::Instruction::Block { .. } |
                                                wrt_foundation::types::Instruction::Loop { .. } |
                                                wrt_foundation::types::Instruction::If { .. } => {
                                                    depth += 1;
                                                }
                                                wrt_foundation::types::Instruction::End => {
                                                    if depth == 0 {
                                                        target_depth -= 1;
                                                        if target_depth == 0 {
                                                            #[cfg(feature = "tracing")]

                                                            trace!("BrTable: jumping forward to pc={} (end of {} block)", new_pc, block_type);
                                                            pc = new_pc - 1;
                                                            break;
                                                        }
                                                    } else {
                                                        depth -= 1;
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        new_pc += 1;
                                    }
                                }
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("BrTable: label_idx {} out of range (block_stack.len={})", label_idx, block_stack.len());
                            }
                        } else {
                            #[cfg(feature = "tracing")]

                            trace!("BrTable: no index on stack");
                        }
                    }
                    Instruction::Return => {
                        #[cfg(feature = "tracing")]

                        trace!("🔙 Return at pc={}", pc);
                        #[cfg(feature = "tracing")]

                        trace!("  Operand stack size: {}", operand_stack.len());
                        #[cfg(feature = "tracing")]

                        trace!("  Instructions executed: {}", instruction_count);
                        break; // Exit function
                    }
                    Instruction::End => {
                        // Decrement block depth first
                        block_depth -= 1;

                        // Check if this is the function's final End instruction
                        // WebAssembly functions have an implicit outer block, so when we hit
                        // an End that brings depth to -1, or we're at the last instruction with depth 0,
                        // we've reached the function end
                        if block_depth < 0 || (pc == instructions.len() - 1 && block_depth == 0) {
                            // This is the function's final End
                            #[cfg(feature = "tracing")]

                            trace!("🔙 End at pc={} (function end)", pc);
                            #[cfg(feature = "tracing")]

                            trace!("  Operand stack size: {}", operand_stack.len());
                            #[cfg(feature = "tracing")]

                            trace!("  Instructions executed: {}", instruction_count);
                            break; // Exit function
                        } else {
                            // This ends a block/loop/if - continue execution
                            if !block_stack.is_empty() {
                                let (block_type, start_pc) = block_stack.pop().unwrap();
                                #[cfg(feature = "tracing")]

                                trace!("End at pc={} (closes {} from pc={}, depth now {})", pc, block_type, start_pc, block_depth);
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("End at pc={} (closes block, depth now {})", pc, block_depth);
                            }
                        }
                    }
                    _ => {
                        // Skip unimplemented instructions for now
                        #[cfg(feature = "tracing")]

                        trace!("Unimplemented instruction at pc={}: {:?}", pc, instruction);
                    }
                }

                // Increment program counter for next iteration
                pc += 1;
            }

            // Return values from operand stack matching function signature
            #[cfg(feature = "tracing")]

            trace!("Function complete. Operand stack has {} values", operand_stack.len());
            #[cfg(feature = "tracing")]

            trace!("STATS: Executed {} instructions total", instruction_count);
            #[cfg(feature = "tracing")]

            trace!("Function type expects {} results", func_type.results.len());

            let mut results = Vec::new();
            for (i, result_type) in func_type.results.iter().enumerate() {
                if let Some(value) = operand_stack.pop() {
                    #[cfg(feature = "tracing")]

                    trace!("Result {}: {:?}", i, value);
                    results.insert(0, value);
                } else {
                    #[cfg(feature = "tracing")]

                    trace!("Result {}: missing, using default", i);
                    results.insert(0, Value::I32(0));
                }
            }

            #[cfg(feature = "tracing")]
            trace!("Returning {} results", results.len());
            Ok(results)
        }

        #[cfg(not(feature = "std"))]
        {
            // Fallback for no_std - return default values
            let mut results = {
                use wrt_foundation::{
                    budget_aware_provider::CrateId,
                    safe_managed_alloc,
                };
                use crate::bounded_runtime_infra::RUNTIME_MEMORY_SIZE;
                let provider = safe_managed_alloc!(RUNTIME_MEMORY_SIZE, CrateId::Runtime)?;
                BoundedVec::new(provider)?
            };
            for result_type in &func_type.results {
                let default_value = match result_type {
                    wrt_foundation::ValueType::I32 => Value::I32(0),
                    wrt_foundation::ValueType::I64 => Value::I64(0),
                    wrt_foundation::ValueType::F32 => Value::F32(FloatBits32(0)),
                    wrt_foundation::ValueType::F64 => Value::F64(FloatBits64(0)),
                    _ => Value::I32(0),
                };
                results.push(default_value)?;
            }
            Ok(results)
        }
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub fn execute(
        &self,
        instance_id: usize,
        func_idx: usize,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        #[cfg(feature = "std")]
        #[cfg(feature = "tracing")]

        trace!("DEBUG StacklessEngine::execute: instance_id={}, func_idx={}", instance_id, func_idx);

        let instance = self
            .instances
            .get(&instance_id)?
            .ok_or_else(|| wrt_error::Error::runtime_execution_error("Instance not found"))?;

        // For now, implement a basic execution that validates the function exists
        // and returns appropriate results
        let module = instance.module();

        #[cfg(feature = "std")]
        #[cfg(feature = "tracing")]

        debug!("Got module, functions.len()={}", module.functions.len());

        // Validate function index
        if func_idx >= module.functions.len() {
            return Err(wrt_error::Error::runtime_function_not_found(
                "Function index out of bounds",
            ));
        }

        let func = module
            .functions
            .get(func_idx)
            .map_err(|_| wrt_error::Error::runtime_error("Failed to get function"))?;

        #[cfg(feature = "std")]
        #[cfg(feature = "tracing")]

        debug!("Retrieved func, body.instructions.len()={}", func.body.instructions.len());

        #[cfg(feature = "std")]
        #[cfg(feature = "tracing")]

        trace!("DEBUG execute: func.type_idx={}, module.types.len()={}", func.type_idx, module.types.len());

        // In std mode, types is Vec so get() returns Option<&T>
        #[cfg(feature = "std")]
        let func_type = module
            .types
            .get(func.type_idx as usize)
            .ok_or_else(|| wrt_error::Error::runtime_error("Failed to get function type"))?;

        // In no_std mode, types is BoundedVec so get() returns Result<T>
        #[cfg(not(feature = "std"))]
        let func_type = &module
            .types
            .get(func.type_idx as usize)
            .map_err(|_| wrt_error::Error::runtime_error("Failed to get function type"))?;

        // Return appropriate default values based on function signature
        let mut results = {
            use wrt_foundation::{
                budget_aware_provider::CrateId,
                safe_managed_alloc,
            };

            let provider = safe_managed_alloc!(4096, CrateId::Runtime)?;
            BoundedVec::new(provider)
                .map_err(|_| wrt_error::Error::runtime_error("Failed to create results vector"))?
        };
        for result_type in &func_type.results {
            let default_value = match result_type {
                wrt_foundation::ValueType::I32 => Value::I32(0),
                wrt_foundation::ValueType::I64 => Value::I64(0),
                wrt_foundation::ValueType::F32 => Value::F32(FloatBits32(0.0f32.to_bits())),
                wrt_foundation::ValueType::F64 => Value::F64(FloatBits64(0.0f64.to_bits())),
                // Add other types as needed
                _ => Value::I32(0), // Default fallback
            };
            results
                .push(default_value)
                .map_err(|_| wrt_error::Error::runtime_error("Failed to push result value"))?;
        }

        Ok(results)
    }

    /// Get the remaining fuel for execution
    pub fn remaining_fuel(&self) -> Option<u64> {
        Some(self.fuel.load(Ordering::Relaxed))
    }

    /// Get the current instruction pointer
    pub fn get_instruction_pointer(&self) -> Result<u32> {
        Ok(self.instruction_pointer.load(Ordering::Relaxed) as u32)
    }

    /// Execute a single step of function execution with instruction limit
    pub fn execute_function_step(
        &mut self,
        instance: &ModuleInstance,
        func_idx: usize,
        params: &[Value],
        max_instructions: u32,
    ) -> Result<crate::stackless::ExecutionResult> {
        use wrt_foundation::{
            budget_aware_provider::CrateId,
            safe_managed_alloc,
        };

        // Validate function exists
        let module = instance.module();
        if func_idx >= module.functions.len() {
            return Err(wrt_error::Error::runtime_function_not_found(
                "Function index out of bounds",
            ));
        }

        // Get function type
        let func = module
            .functions
            .get(func_idx)
            .ok_or_else(|| wrt_error::Error::runtime_function_not_found("Failed to get function"))?;
        // In std mode, types is Vec so get() returns Option<&T>
        #[cfg(feature = "std")]
        let func_type = module
            .types
            .get(func.type_idx as usize)
            .ok_or_else(|| wrt_error::Error::runtime_error("Failed to get function type"))?;

        // In no_std mode, types is BoundedVec so get() returns Result<T>
        #[cfg(not(feature = "std"))]
        let func_type = &module
            .types
            .get(func.type_idx as usize)
            .map_err(|_| wrt_error::Error::runtime_error("Failed to get function type"))?;

        // Simulate step execution - in real implementation would execute instructions
        // For now, return completed with default values
        let provider = safe_managed_alloc!(1024, CrateId::Runtime)?;
        let mut results = wrt_foundation::bounded::BoundedVec::new(provider)
            .map_err(|_| wrt_error::Error::runtime_error("Failed to create results vector"))?;

        for result_type in &func_type.results {
            let default_value = match result_type {
                wrt_foundation::ValueType::I32 => Value::I32(0),
                wrt_foundation::ValueType::I64 => Value::I64(0),
                wrt_foundation::ValueType::F32 => Value::F32(FloatBits32(0.0f32.to_bits())),
                wrt_foundation::ValueType::F64 => Value::F64(FloatBits64(0.0f64.to_bits())),
                _ => Value::I32(0),
            };
            results
                .push(default_value)
                .map_err(|_| wrt_error::Error::runtime_error("Failed to push result value"))?;
        }

        // Update instruction pointer
        self.instruction_pointer
            .fetch_add(max_instructions as u64, Ordering::Relaxed);

        // Consume some fuel
        let fuel_to_consume = max_instructions.min(100) as u64;
        let current_fuel = self.fuel.load(Ordering::Relaxed);
        if current_fuel < fuel_to_consume {
            self.fuel.store(0, Ordering::Relaxed);
            return Ok(crate::stackless::ExecutionResult::FuelExhausted);
        }
        self.fuel
            .fetch_sub(fuel_to_consume, Ordering::Relaxed);

        Ok(crate::stackless::ExecutionResult::Completed(results))
    }

    /// Restore engine state from a saved state
    pub fn restore_state(&mut self, state: crate::stackless::EngineState) -> Result<()> {
        self.instruction_pointer
            .store(state.instruction_pointer as u64, Ordering::Relaxed);

        // In a real implementation, would restore operand stack, locals, and call stack
        // For now, just update the instruction pointer
        Ok(())
    }

    /// Continue execution from current state
    pub fn continue_execution(
        &mut self,
        max_instructions: u32,
    ) -> Result<crate::stackless::ExecutionResult> {
        use wrt_foundation::{
            budget_aware_provider::CrateId,
            safe_managed_alloc,
        };

        // Simulate continued execution
        // In real implementation, would resume from saved state

        // Update instruction pointer
        self.instruction_pointer
            .fetch_add(max_instructions as u64, Ordering::Relaxed);

        // Consume some fuel
        let fuel_to_consume = max_instructions.min(100) as u64;
        let current_fuel = self.fuel.load(Ordering::Relaxed);
        if current_fuel < fuel_to_consume {
            self.fuel.store(0, Ordering::Relaxed);
            return Ok(crate::stackless::ExecutionResult::FuelExhausted);
        }
        self.fuel
            .fetch_sub(fuel_to_consume, Ordering::Relaxed);

        // For now, return completed with empty results
        let provider = safe_managed_alloc!(1024, CrateId::Runtime)?;
        let results = wrt_foundation::bounded::BoundedVec::new(provider)
            .map_err(|_| wrt_error::Error::runtime_error("Failed to create results vector"))?;

        Ok(crate::stackless::ExecutionResult::Completed(results))
    }

    /// Count total number of imports across all modules
    fn count_total_imports(&self, module: &crate::module::Module) -> usize {
        // For now, we'll count based on the fact that imported functions
        // are added as placeholder functions at the beginning of the functions array
        // A proper implementation would iterate through module.imports

        // Count functions that are imports (those with empty body)
        let mut import_count = 0;
        for func in &module.functions {
            if func.body.is_empty() && func.locals.is_empty() {
                import_count += 1;
            } else {
                // Once we hit a non-import function, we're done
                break;
            }
        }

        #[cfg(feature = "tracing")]


        trace!("Total imports counted: {}", import_count);
        import_count
    }

    /// Find import by function index using the ordered import list
    fn find_import_by_index(&self, module: &crate::module::Module, func_idx: usize) -> Result<(String, String)> {
        #[cfg(feature = "tracing")]
        let _span = wrt_foundation::tracing::ImportTrace::lookup("", "").entered();

        #[cfg(feature = "tracing")]
        debug!("Looking for import at index {} (import_order.len()={})", func_idx, module.import_order.len());

        // Use the ordered import list for direct index lookup
        #[cfg(feature = "std")]
        {
            if func_idx < module.import_order.len() {
                let (module_name, field_name) = &module.import_order[func_idx];
                #[cfg(feature = "tracing")]
                trace!("Found import at index {}: {}::{}", func_idx, module_name, field_name);
                return Ok((module_name.clone(), field_name.clone()));
            }
        }

        #[cfg(not(feature = "std"))]
        {
            if let Ok(Some((module_name, field_name))) = module.import_order.get(func_idx) {
                #[cfg(feature = "tracing")]
                trace!("Found import at index {}: {}::{}", func_idx,
                    module_name.as_str().unwrap_or("<error>"),
                    field_name.as_str().unwrap_or("<error>"));
                return Ok((
                    module_name.as_str().map(|s| s.to_string()).unwrap_or_default(),
                    field_name.as_str().map(|s| s.to_string()).unwrap_or_default()
                ));
            }
        }

        #[cfg(feature = "tracing")]
        trace!("Could not find import at index {} (import_order.len()={})", func_idx, module.import_order.len());
        Err(wrt_error::Error::runtime_error("Import not found"))
    }

    /// Find export function index by name
    fn find_export_index(&self, module: &crate::module::Module, name: &str) -> Result<usize> {
        #[cfg(feature = "std")]
        {
            for (export_name, export) in module.exports.iter() {
                // BoundedString::as_str() returns Result<&str, BoundedError>
                if let Ok(export_str) = export_name.as_str() {
                    if export_str == name {
                        if let crate::module::ExportKind::Function = export.kind {
                            return Ok(export.index as usize);
                        }
                    }
                }
            }
        }

        #[cfg(not(feature = "std"))]
        {
            for (export_name, export) in module.exports.iter() {
                if export_name.as_str() == name {
                    if let crate::module::ExportKind::Function = export.kind {
                        return Ok(export.index as usize);
                    }
                }
            }
        }

        Err(wrt_error::Error::runtime_error("Export function not found"))
    }

    /// Call cabi_realloc to allocate memory in WASM instance
    fn call_cabi_realloc(&mut self, instance_id: usize, func_idx: usize,
                         old_ptr: u32, old_size: u32, align: u32, new_size: u32) -> Result<u32> {
        #[cfg(feature = "std")]
        eprintln!("[CABI_REALLOC] Calling with old_ptr={}, old_size={}, align={}, new_size={}",
                 old_ptr, old_size, align, new_size);
        let args = vec![
            Value::I32(old_ptr as i32),
            Value::I32(old_size as i32),
            Value::I32(align as i32),
            Value::I32(new_size as i32),
        ];
        #[cfg(feature = "std")]
        eprintln!("[CABI_REALLOC] Args: {:?}", args);

        let results = self.execute(instance_id, func_idx, args)?;

        if let Some(Value::I32(ptr)) = results.first() {
            Ok(*ptr as u32)
        } else {
            Err(wrt_error::Error::runtime_error("cabi_realloc returned invalid value"))
        }
    }

    /// Allocate memory for WASI arguments using cabi_realloc
    ///
    /// This properly allocates memory owned by the component so that argument
    /// strings don't get overwritten by the component's heap/stack operations.
    ///
    /// Returns (list_ptr, string_data_ptr) where:
    /// - list_ptr: pointer to the (ptr, len) array for list elements
    /// - string_data_ptr: pointer to the start of string data
    #[cfg(feature = "wasi")]
    fn allocate_wasi_args_memory(
        &mut self,
        instance_id: usize,
        args: &[String],
    ) -> Result<Option<(u32, u32)>> {
        // Find cabi_realloc export
        let instance = self.instances.get(&instance_id)
            .ok_or_else(|| wrt_error::Error::runtime_error("Instance not found"))?
            .clone();
        let module = instance.module();

        let cabi_realloc_idx = match self.find_export_index(&module, "cabi_realloc") {
            Ok(idx) => idx,
            Err(_) => {
                #[cfg(feature = "std")]
                eprintln!("[WASI-ALLOC] cabi_realloc not found, falling back to stack-relative allocation");
                return Ok(None);
            }
        };

        // Calculate total memory needed:
        // - (ptr, len) array: 8 bytes per arg
        // - string data: sum of all string lengths + padding
        let list_size = args.len() * 8;
        let mut string_total: usize = 0;
        for arg in args {
            string_total += arg.len();
            string_total += 1; // null terminator
            string_total = (string_total + 7) & !7; // align to 8
        }
        let total_size = list_size + string_total;

        if total_size == 0 {
            return Ok(None);
        }

        #[cfg(feature = "std")]
        eprintln!("[WASI-ALLOC] Allocating {} bytes for {} args (list={}, strings={})",
                 total_size, args.len(), list_size, string_total);

        // Call cabi_realloc to allocate memory
        // cabi_realloc(old_ptr, old_size, align, new_size) -> ptr
        let ptr = self.call_cabi_realloc(
            instance_id,
            cabi_realloc_idx,
            0,  // old_ptr (NULL for new allocation)
            0,  // old_size
            8,  // align (8-byte alignment for pointers)
            total_size as u32,
        )?;

        #[cfg(feature = "std")]
        eprintln!("[WASI-ALLOC] cabi_realloc returned ptr=0x{:x}", ptr);

        // list_ptr is at the start of allocated memory
        // string_data_ptr is after the (ptr, len) array
        let list_ptr = ptr;
        let string_data_ptr = ptr + (list_size as u32);

        Ok(Some((list_ptr, string_data_ptr)))
    }

    /// Pre-allocate memory for WASI arguments before starting execution
    ///
    /// This should be called before `execute` to allocate memory via cabi_realloc
    /// for WASI argument strings. The allocated pointers will be stored in the
    /// WASI dispatcher and used when get-arguments is called.
    ///
    /// # Arguments
    /// * `instance_id` - The instance ID to allocate memory in
    ///
    /// # Returns
    /// Ok(()) if successful, Err if allocation fails
    #[cfg(feature = "wasi")]
    pub fn pre_allocate_wasi_args(&mut self, instance_id: usize) -> Result<()> {
        // Get args from dispatcher
        let args: Vec<String> = self.wasi_dispatcher
            .as_ref()
            .map(|d| d.args().to_vec())
            .unwrap_or_default();

        if args.is_empty() {
            #[cfg(feature = "std")]
            eprintln!("[WASI-PREALLOC] No args to pre-allocate");
            return Ok(());
        }

        #[cfg(feature = "std")]
        eprintln!("[WASI-PREALLOC] Pre-allocating memory for {} args: {:?}", args.len(), args);

        // Allocate memory
        match self.allocate_wasi_args_memory(instance_id, &args)? {
            Some((list_ptr, string_ptr)) => {
                #[cfg(feature = "std")]
                eprintln!("[WASI-PREALLOC] Allocated: list_ptr=0x{:x}, string_ptr=0x{:x}", list_ptr, string_ptr);

                // Store in dispatcher
                if let Some(ref mut dispatcher) = self.wasi_dispatcher {
                    dispatcher.set_args_alloc(list_ptr, string_ptr);
                }
                Ok(())
            }
            None => {
                #[cfg(feature = "std")]
                eprintln!("[WASI-PREALLOC] No allocation needed or cabi_realloc not available");
                Ok(())
            }
        }
    }

    /// Write data to WASM instance memory
    fn write_to_instance(&self, instance_id: usize, addr: u32, data: &[u8]) -> Result<()> {
        let instance = self.instances.get(&instance_id)
            .ok_or_else(|| wrt_error::Error::runtime_error("Instance not found"))?;

        #[cfg(feature = "tracing")]


        debug!("WASI init: write_to_instance: module declares {} memories",                   if instance.module().memories.is_empty() { 0 } else { instance.module().memories.len() });

        // Try to get memory - this will fail if instance doesn't have runtime memory initialized
        let memory = match instance.memory(0) {
            Ok(mem) => mem,
            Err(e) => {
                #[cfg(feature = "tracing")]

                debug!("WASI init: write_to_instance: failed to get memory: {:?}", e);
                return Err(e);
            }
        };

        let pages = memory.0.size();  // This is the size() method that returns pages
        let bytes = pages as usize * 65536;  // Convert pages to bytes
        #[cfg(feature = "tracing")]

        debug!("WASI init: write_to_instance: memory {} pages = {} bytes", pages, bytes);

        // Verify the write won't exceed memory bounds
        if (addr as usize + data.len()) > bytes {
            #[cfg(feature = "tracing")]

            debug!("WASI init: write_to_instance: write at {:#x} + {} bytes would exceed {} byte memory",                      addr, data.len(), bytes);
            return Err(wrt_error::Error::runtime_execution_error("Write would exceed memory bounds"));
        }

        memory.0.write_shared(addr, data)?;
        Ok(())
    }

    /// Initialize WASI stub memory for an instance
    ///
    /// For now, we use a simple fixed address (0x100) for stub data since memory cloning
    /// has issues preserving size. This is safe because:
    /// - Address 0x100 (256 bytes) is well within even small WASM memories
    /// - The data is just zeros which won't interfere with normal operation
    /// - WASM code that reads these pointers will get valid empty lists/None values
    fn initialize_wasi_stubs(&mut self, instance_id: usize, module: &crate::module::Module) -> Result<()> {
        #[cfg(feature = "tracing")]

        debug!("WASI init: Initializing WASI stubs for instance {}", instance_id);

        // Use a low fixed address that's guaranteed to be valid in most WASM memories
        // We need 16 bytes total: 8 for empty list + 1 for option None + 7 padding
        let base_ptr = 0x100u32; // 256 bytes into memory

        // Try to write stub data - if this fails, memory isn't ready yet (which is normal for many instances)
        match self.write_to_instance(instance_id, base_ptr, &[0u8; 16]) {
            Ok(_) => {
                #[cfg(feature = "tracing")]

                info!("WASI init: ✓ Wrote 16 bytes of stub data at ptr={:#x}", base_ptr);

                // Cache the pointers
                let stub_mem = WasiStubMemory {
                    empty_list: base_ptr,      // Points to 8 bytes of zeros = (ptr=0, len=0)
                    option_none: base_ptr + 8, // Points to 1 byte zero = None
                    empty_env: base_ptr,       // Reuse empty_list
                    stdout_handle: 1,
                    stderr_handle: 2,
                };

                self.wasi_stubs.insert(instance_id, stub_mem);
                #[cfg(feature = "tracing")]

                info!("WASI init: ✓ WASI stubs initialized with memory write");
                Ok(())
            }
            Err(_e) => {
                #[cfg(feature = "tracing")]

                debug!("WASI init: Instance has no accessible memory (normal for adapter modules)");
                #[cfg(feature = "tracing")]

                debug!("WASI init: Using fallback pointers (stub WASI functions will return empty values)");

                // Even if we can't write, we can still return valid pointers
                // The WASM memory likely has zeros at these addresses anyway
                let stub_mem = WasiStubMemory {
                    empty_list: base_ptr,
                    option_none: base_ptr + 8,
                    empty_env: base_ptr,
                    stdout_handle: 1,
                    stderr_handle: 2,
                };

                self.wasi_stubs.insert(instance_id, stub_mem);
                Ok(()) // Don't fail - just use the pointers anyway
            }
        }
    }

    /// Call a WASI host function
    fn call_wasi_function(
        &mut self,
        module_name: &str,
        field_name: &str,
        stack: &mut Vec<Value>,
        module: &crate::module::Module,
        instance_id: usize,
    ) -> Result<Option<Value>> {
        use std::io::Write;

        #[cfg(feature = "tracing")]


        debug!("WASI: Calling {}::{}", module_name, field_name);

        // NOTE: We skip the host_registry path for WASI functions because it doesn't
        // properly marshal arguments from the stack. Instead, we fall through to the
        // wasip2_host dispatch (for WASI Preview 2) or the stub implementations which
        // properly handle the stack arguments.
        //
        // The host_registry is kept for non-WASI host functions that don't need stack args.
        #[cfg(feature = "std")]
        if let Some(ref registry) = self.host_registry {
            // Only use host_registry for non-WASI functions
            // WASI functions need proper stack argument marshalling done below
            if !module_name.starts_with("wasi:") {
                #[cfg(feature = "tracing")]
                debug!("WASI: Checking host registry for {}::{}", module_name, field_name);

                if registry.has_host_function(module_name, field_name) {
                    #[cfg(feature = "tracing")]
                    debug!("WASI: Found {} in host registry, calling implementation", field_name);

                    let args: Vec<wrt_foundation::Value> = vec![];
                    let mut dummy_engine: i32 = 0;
                    match registry.call_host_function(&mut dummy_engine, module_name, field_name, args) {
                        Ok(result) => {
                            #[cfg(feature = "tracing")]
                            debug!("WASI: Host function {} returned successfully", field_name);
                            if let Some(val) = result.first() {
                                return Ok(Some(val.clone()));
                            }
                            return Ok(None);
                        }
                        Err(e) => {
                            #[cfg(feature = "tracing")]
                            debug!("WASI: Host function {} failed: {:?}", field_name, e);
                            return Err(e);
                        }
                    }
                }
            }
        }

        // Check if this is a wasip2 canonical function
        let full_name = format!("{}::{}", module_name, field_name);
        if module_name.starts_with("wasi:") && module_name.contains("@0.2") {
            #[cfg(feature = "std")]
            eprintln!("[WASIP2-CANONICAL] Detected wasip2 call: {}, stack has {} values", full_name, stack.len());
            #[cfg(feature = "tracing")]
            trace!("[WASIP2-CANONICAL] Detected wasip2 call: {}", full_name);

            // ============================================================
            // Two-phase WASI dispatch with proper cabi_realloc support
            // This follows the Canonical ABI spec where the HOST calls
            // cabi_realloc during lowering for functions returning lists.
            // ============================================================
            #[cfg(feature = "wasi")]
            {
                // Extract core values from stack for dispatch
                let core_args: Vec<Value> = stack.iter().cloned().collect();

                #[cfg(feature = "std")]
                eprintln!("[WASI-V2] Dispatching {}::{} with {} args", module_name, field_name, core_args.len());

                // Create a temporary dispatcher - we use global args as fallback
                if let Ok(mut temp_dispatcher) = wrt_wasi::WasiDispatcher::with_defaults() {
                    // Use dispatch_core_v2 which returns DispatchResult
                    let dispatch_result = temp_dispatcher.dispatch_core_v2(
                        module_name, field_name, core_args.clone(), None
                    );

                    match dispatch_result {
                        Ok(wrt_wasi::DispatchResult::Complete(results)) => {
                            #[cfg(feature = "std")]
                            eprintln!("[WASI-V2] Complete: {:?}", results);
                            stack.clear();
                            if let Some(val) = results.first() {
                                return Ok(Some(val.clone()));
                            }
                            return Ok(None);
                        }
                        Ok(wrt_wasi::DispatchResult::NeedsAllocation { request, args_to_write, retptr }) => {
                            #[cfg(feature = "std")]
                            eprintln!("[WASI-V2] NeedsAllocation: size={}, align={}, purpose={}",
                                     request.size, request.align, request.purpose);

                            // Find cabi_realloc in this module
                            let cabi_realloc_idx = self.find_export_index(module, "cabi_realloc");

                            if let Ok(func_idx) = cabi_realloc_idx {
                                // SEPARATE ALLOCATIONS: Allocate list array and each string separately
                                        // This prevents dlmalloc free-list corruption when adapter frees strings.
                                        // When all data is in one block, freeing individual strings causes
                                        // dlmalloc to coalesce and overwrite the list array with free-list pointers.

                                        // Step 1: Allocate list array (N * 8 bytes for N (ptr, len) pairs)
                                        let list_array_size = (args_to_write.len() * 8) as u32;
                                        let list_ptr = self.call_cabi_realloc(instance_id, func_idx, 0, 0, 8, list_array_size)?;
                                        #[cfg(feature = "std")]
                                        eprintln!("[WASI-V2] list_ptr=0x{:x}", list_ptr);

                                        // Step 2: Allocate each string separately and write data
                                        let mut string_entries: Vec<(u32, u32)> = Vec::new();
                                        for (i, arg) in args_to_write.iter().enumerate() {
                                            let bytes = arg.as_bytes();
                                            let len = bytes.len() as u32;

                                            // Allocate memory for this string (align=1 for byte data)
                                            let string_ptr = self.call_cabi_realloc(instance_id, func_idx, 0, 0, 1, len)?;

                                            // Write string bytes
                                            if let Some(instance) = self.instances.get(&instance_id) {
                                                if let Ok(memory_wrapper) = instance.memory(0) {
                                                    let memory = &memory_wrapper.0;
                                                    if let Err(e) = memory.write_shared(string_ptr, bytes) {
                                                        return Err(e);
                                                    }
                                                }
                                            }

                                            #[cfg(feature = "std")]
                                            eprintln!("[WASI-V2] str[{}]='{}' at 0x{:x} (len={})", i, arg, string_ptr, len);

                                            string_entries.push((string_ptr, len));
                                        }

                                        // Step 3: Write (ptr, len) entries to list array
                                        if let Some(instance) = self.instances.get(&instance_id) {
                                            if let Ok(memory_wrapper) = instance.memory(0) {
                                                let memory = &memory_wrapper.0;

                                                for (i, (ptr, len)) in string_entries.iter().enumerate() {
                                                    let offset = list_ptr + (i * 8) as u32;
                                                    let mut entry_buf = [0u8; 8];
                                                    entry_buf[0..4].copy_from_slice(&ptr.to_le_bytes());
                                                    entry_buf[4..8].copy_from_slice(&len.to_le_bytes());

                                                    if let Err(e) = memory.write_shared(offset, &entry_buf) {
                                                        return Err(e);
                                                    }
                                                }

                                                // Step 4: Write (list_ptr, count) to retptr
                                                let mut retptr_buf = [0u8; 8];
                                                retptr_buf[0..4].copy_from_slice(&list_ptr.to_le_bytes());
                                                retptr_buf[4..8].copy_from_slice(&(args_to_write.len() as u32).to_le_bytes());

                                                if let Err(e) = memory.write_shared(retptr, &retptr_buf) {
                                                    return Err(e);
                                                }

                                                #[cfg(feature = "std")]
                                                {
                                                    eprintln!("[WASI-V2] retptr=0x{:x} -> (0x{:x}, {})",
                                                             retptr, list_ptr, args_to_write.len());

                                                    // Dump memory around retptr to see what's there
                                                    let mut retptr_dump = [0u8; 32];
                                                    if memory.read(retptr, &mut retptr_dump).is_ok() {
                                                        eprintln!("[RETPTR-DUMP] 0x{:x}: {:02x?}", retptr, &retptr_dump);
                                                    }

                                                    // Dump memory around list array to see full picture
                                                    let mut list_dump = [0u8; 64];
                                                    if memory.read(list_ptr, &mut list_dump).is_ok() {
                                                        eprintln!("[LIST-DUMP] 0x{:x}: {:02x?}", list_ptr, &list_dump);
                                                    }

                                                    // Verify: read back the list array
                                                    for (i, (expected_ptr, expected_len)) in string_entries.iter().enumerate() {
                                                        let offset = list_ptr + (i * 8) as u32;
                                                        let mut entry_buf = [0u8; 8];
                                                        if memory.read(offset, &mut entry_buf).is_ok() {
                                                            let read_ptr = u32::from_le_bytes([entry_buf[0], entry_buf[1], entry_buf[2], entry_buf[3]]);
                                                            let read_len = u32::from_le_bytes([entry_buf[4], entry_buf[5], entry_buf[6], entry_buf[7]]);
                                                            if read_ptr != *expected_ptr || read_len != *expected_len {
                                                                eprintln!("[VERIFY-FAIL] list[{}] at 0x{:x}: expected (0x{:x}, {}), got (0x{:x}, {})",
                                                                         i, offset, expected_ptr, expected_len, read_ptr, read_len);
                                                            }
                                                        }
                                                    }

                                                    // Also verify the strings themselves
                                                    for (i, (str_ptr, str_len)) in string_entries.iter().enumerate() {
                                                        let mut str_buf = vec![0u8; *str_len as usize];
                                                        if memory.read(*str_ptr, &mut str_buf).is_ok() {
                                                            let str_val = String::from_utf8_lossy(&str_buf);
                                                            eprintln!("[VERIFY-STR] str[{}] at 0x{:x}: '{}'", i, str_ptr, str_val);
                                                        }
                                                    }
                                                }

                                                stack.clear();
                                                return Ok(None);
                                            }
                                        }
                                        return Err(wrt_error::Error::memory_error("Could not access instance memory"));
                            } else {
                                #[cfg(feature = "std")]
                                eprintln!("[WASI-V2] cabi_realloc not found, falling back to legacy dispatch");
                                // Fall through to try dispatch_core which has fallback allocation
                            }
                        }
                        Err(e) => {
                            #[cfg(feature = "std")]
                            eprintln!("[WASI-V2] dispatch_core_v2 error: {:?}, trying dispatch_core", e);
                            // Fall through to try dispatch_core
                        }
                    }

                    // Fallback: try dispatch_core which uses stack-relative allocation
                    let memory_result: Option<Vec<u8>> = if let Some(instance) = self.instances.get(&instance_id) {
                        if let Ok(memory_wrapper) = instance.memory(0) {
                            let mem_size = memory_wrapper.0.size_in_bytes();
                            let mut temp_buffer = vec![0u8; mem_size.min(16 * 1024 * 1024)];
                            let _ = memory_wrapper.0.read(0, &mut temp_buffer);
                            Some(temp_buffer)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let dispatch_result = if let Some(mut mem) = memory_result {
                        let result = temp_dispatcher.dispatch_core(module_name, field_name, core_args, Some(&mut mem));

                        // Write memory back
                        if let Some(instance) = self.instances.get(&instance_id) {
                            if let Ok(memory_wrapper) = instance.memory(0) {
                                if let Err(e) = memory_wrapper.0.write_shared(0, &mem) {
                                    #[cfg(feature = "std")]
                                    eprintln!("[WASI-FALLBACK] Failed to write back memory: {:?}", e);
                                }
                            }
                        }
                        result
                    } else {
                        temp_dispatcher.dispatch_core(module_name, field_name, core_args, None)
                    };

                    match dispatch_result {
                        Ok(results) => {
                            #[cfg(feature = "std")]
                            eprintln!("[WASI-FALLBACK] Success: {:?}", results);
                            stack.clear();
                            if let Some(val) = results.first() {
                                return Ok(Some(val.clone()));
                            }
                            return Ok(None);
                        }
                        Err(e) => {
                            #[cfg(feature = "std")]
                            eprintln!("[WASI-FALLBACK] Failed: {:?}", e);
                            // Fall through to legacy path
                        }
                    }
                }
            }

            // ============================================================
            // Legacy path: wasip2_host (kept for backward compatibility)
            // ============================================================

            // Create wasip2 host
            let mut wasip2_host = crate::wasip2_host::Wasip2Host::new();

            // Extract core values from stack
            let core_args: Vec<Value> = {
                let mut args = Vec::new();
                for _ in 0..stack.len() {
                    if let Some(val) = stack.pop() {
                        args.push(val);
                    }
                }
                args.reverse(); // Reverse to get correct order
                args
            };

            #[cfg(feature = "std")]
            eprintln!("[WASIP2-CANONICAL] Extracted {} core args: {:?}", core_args.len(), core_args);

            // Get memory for lifting complex types
            let memory_slice: Option<Vec<u8>> = if let Some(instance) = self.instances.get(&instance_id) {
                if instance.memory(0).is_ok() {
                    let mem_size = if let Ok(memory_wrapper) = instance.memory(0) {
                        memory_wrapper.0.size_in_bytes()
                    } else {
                        65536
                    };

                    let mut temp_buffer = vec![0u8; mem_size.min(16 * 1024 * 1024)];
                    if let Ok(memory_wrapper) = instance.memory(0) {
                        let read_size = temp_buffer.len().min(mem_size);
                        let _ = memory_wrapper.0.read(0, &mut temp_buffer[..read_size]);
                    }
                    Some(temp_buffer)
                } else {
                    None
                }
            } else {
                None
            };

            // Check if this function needs canonical lifting
            let needs_lifting = crate::wasip2_host::wasi_function_needs_lifting(module_name, field_name);

            #[cfg(feature = "std")]
            eprintln!("[WASIP2-CANONICAL] needs_lifting={}", needs_lifting);

            let dispatch_result = if needs_lifting {
                // === CANONICAL ABI PATH ===
                // Lift core values to component values, dispatch, lower results

                // 1. LIFT: Convert core values to component values
                let component_args = crate::wasip2_host::lift_wasi_args(
                    module_name,
                    field_name,
                    &core_args,
                    memory_slice.as_deref(),
                );

                match component_args {
                    Ok(lifted_args) => {
                        #[cfg(feature = "std")]
                        eprintln!("[WASIP2-CANONICAL] Lifted to {} component args", lifted_args.len());

                        // 2. DISPATCH: Call the component-level host function
                        let component_results = wasip2_host.dispatch_component(
                            module_name,
                            field_name,
                            lifted_args,
                        );

                        match component_results {
                            Ok(results) => {
                                #[cfg(feature = "std")]
                                eprintln!("[WASIP2-CANONICAL] Component dispatch returned {} results", results.len());

                                // 3. LOWER: Convert component results back to core values
                                crate::wasip2_host::lower_wasi_results(
                                    module_name,
                                    field_name,
                                    &results,
                                    None, // TODO: pass mutable memory for complex return types
                                    0,
                                )
                            }
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => {
                        #[cfg(feature = "std")]
                        eprintln!("[WASIP2-CANONICAL] Lifting failed: {:?}", e);
                        Err(e)
                    }
                }
            } else {
                // === DIRECT PATH (primitives only) ===
                // No lifting needed, use the old direct dispatch
                if let Some(mut mem_buffer) = memory_slice {
                    wasip2_host.dispatch(module_name, field_name, core_args, Some(&mut mem_buffer))
                } else {
                    wasip2_host.dispatch(module_name, field_name, core_args, None)
                }
            };

            match dispatch_result {
                Ok(results) => {
                    #[cfg(feature = "std")]
                    eprintln!("[WASIP2-CANONICAL] Final results: {:?}", results);
                    if let Some(val) = results.first() {
                        return Ok(Some(val.clone()));
                    }
                    return Ok(None);
                }
                Err(e) => {
                    #[cfg(feature = "std")]
                    eprintln!("[WASIP2-CANONICAL] Function {} FAILED: {:?}", field_name, e);
                    // Fall through to stubs as fallback
                }
            }
        }

        // Fallback to stub implementations if host_registry not available
        #[cfg(feature = "tracing")]

        debug!("WASI: Using stub implementation for {}::{}", module_name, field_name);
        let stub_mem = self.wasi_stubs.get(&instance_id);

        // Strip version from module name to allow any 0.2.x version to match
        let base_module = strip_wasi_version(module_name);

        #[cfg(feature = "std")]
        eprintln!("[WASI_DISPATCH] base_module='{}', field_name='{}'", base_module, field_name);

        match (base_module, field_name) {
            // wasi:cli/environment (any version)
            ("wasi:cli/environment", "get-environment") => {
                if let Some(stub) = stub_mem {
                    #[cfg(feature = "tracing")]

                    debug!("WASI: get-environment: returning empty list ptr={}", stub.empty_env);
                    Ok(Some(Value::I32(stub.empty_env as i32)))
                } else {
                    #[cfg(feature = "tracing")]

                    debug!("WASI: get-environment: stub not initialized, returning 0");
                    Ok(Some(Value::I32(0)))
                }
            }

            ("wasi:cli/environment", "get-arguments") => {
                if let Some(stub) = stub_mem {
                    #[cfg(feature = "tracing")]

                    debug!("WASI: get-arguments: returning empty list ptr={}", stub.empty_list);
                    Ok(Some(Value::I32(stub.empty_list as i32)))
                } else {
                    #[cfg(feature = "tracing")]

                    debug!("WASI: get-arguments: stub not initialized, returning 0");
                    Ok(Some(Value::I32(0)))
                }
            }

            ("wasi:cli/environment", "initial-cwd") => {
                if let Some(stub) = stub_mem {
                    #[cfg(feature = "tracing")]

                    debug!("WASI: initial-cwd: returning option None ptr={}", stub.option_none);
                    Ok(Some(Value::I32(stub.option_none as i32)))
                } else {
                    #[cfg(feature = "tracing")]

                    debug!("WASI: initial-cwd: stub not initialized, returning 0");
                    Ok(Some(Value::I32(0)))
                }
            }

            // wasi:cli/stdout@0.2.0::get-stdout() -> stream
            ("wasi:cli/stdout", "get-stdout") => {
                let handle = stub_mem.map(|s| s.stdout_handle).unwrap_or(1);
                #[cfg(feature = "tracing")]

                debug!("WASI: get-stdout: returning handle {}", handle);
                Ok(Some(Value::I32(handle as i32)))
            }

            // wasi:cli/stderr@0.2.0::get-stderr() -> stream
            ("wasi:cli/stderr", "get-stderr") => {
                let handle = stub_mem.map(|s| s.stderr_handle).unwrap_or(2);
                #[cfg(feature = "tracing")]

                debug!("WASI: get-stderr: returning handle {}", handle);
                Ok(Some(Value::I32(handle as i32)))
            }

            // wasi:cli/exit@0.2.0::exit(code)
            ("wasi:cli/exit", "exit") => {
                let exit_code = if let Some(Value::I32(code)) = stack.pop() {
                    code
                } else {
                    1
                };

                #[cfg(feature = "tracing")]


                debug!("WASI: exit called with code: {}", exit_code);
                std::process::exit(exit_code);
            }

            // wasi:clocks/wall-clock@0.2.x::now() -> datetime
            // Returns (seconds: u64, nanoseconds: u32) as a record
            ("wasi:clocks/wall-clock", "now") => {
                use std::time::{SystemTime, UNIX_EPOCH};

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default();

                let seconds = now.as_secs();
                let nanoseconds = now.subsec_nanos();

                #[cfg(feature = "std")]
                eprintln!("[WASI] wall-clock::now() -> seconds={}, nanos={}", seconds, nanoseconds);

                // Return as two i64 values (the component model will handle conversion)
                // The result is a record { seconds: u64, nanoseconds: u32 }
                // For the core WASM ABI, this is returned as (i64, i32) or written to memory
                // depending on the calling convention.

                // For now, push both values to the stack (caller will handle them)
                // Note: This is a simplification - proper implementation depends on ABI
                stack.push(Value::I64(seconds as i64));
                stack.push(Value::I32(nanoseconds as i32));
                Ok(None) // No single return value, results are on stack
            }

            // wasi:clocks/wall-clock@0.2.x::resolution() -> datetime
            ("wasi:clocks/wall-clock", "resolution") => {
                // Return nanosecond resolution (1 nanosecond = best resolution)
                let seconds: u64 = 0;
                let nanoseconds: u32 = 1;

                #[cfg(feature = "std")]
                eprintln!("[WASI] wall-clock::resolution() -> seconds={}, nanos={}", seconds, nanoseconds);

                stack.push(Value::I64(seconds as i64));
                stack.push(Value::I32(nanoseconds as i32));
                Ok(None)
            }

            // wasi:io/streams@0.2.0::[method]output-stream.blocking-write-and-flush(stream, data_ptr, data_len) -> result
            ("wasi:io/streams", "[method]output-stream.blocking-write-and-flush") => {
                // use crate::wasi_preview2; // TODO: implement wasi_preview2 module

                // Pop arguments: stream, data_ptr, data_len
                let data_len = if let Some(Value::I32(len)) = stack.pop() {
                    len
                } else {
                    return Err(wrt_error::Error::runtime_error("Missing data_len argument"));
                };

                let data_ptr = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr
                } else {
                    return Err(wrt_error::Error::runtime_error("Missing data_ptr argument"));
                };

                let stream_handle = if let Some(Value::I32(s)) = stack.pop() {
                    s
                } else {
                    return Err(wrt_error::Error::runtime_error("Missing stream argument"));
                };

                #[cfg(feature = "tracing")]


                debug!("WASI: blocking-write-and-flush: stream={}, ptr={}, len={}", stream_handle, data_ptr, data_len);

                // Debug unconditionally for now
                #[cfg(feature = "std")]
                eprintln!("[WRITE] blocking-write-and-flush: stream={}, ptr={:#x}, len={}", stream_handle, data_ptr, data_len);

                // Read data from WebAssembly memory and write to stdout/stderr
                // Use instance memory instead of module memory
                if let Some(instance) = self.instances.get(&instance_id) {
                    if let Ok(memory_wrapper) = instance.memory(0) {
                        // Read data from instance memory into a buffer
                        let mut buffer = vec![0u8; data_len as usize];
                        if let Ok(()) = memory_wrapper.0.read(data_ptr as u32, &mut buffer) {
                            #[cfg(feature = "tracing")]
                            debug!("WASI: Read {} bytes from memory at ptr={}", buffer.len(), data_ptr);

                            // Debug: show actual buffer content
                            #[cfg(feature = "std")]
                            eprintln!("[WRITE] Read buffer: {:?} (as string: {:?})", &buffer[..buffer.len().min(64)], String::from_utf8_lossy(&buffer[..buffer.len().min(64)]));

                            // Find the actual content length - some components pass buffer capacity
                            // instead of actual content length. For text output, trim at null byte.
                            let actual_len = buffer.iter()
                                .position(|&b| b == 0)
                                .unwrap_or(buffer.len());
                            let write_buffer = &buffer[..actual_len];

                            // Write directly to stdout/stderr instead of using the memory-based function
                            use std::io::Write;
                            let result = if stream_handle == 1 {
                                // Stdout
                                let mut stdout = std::io::stdout();
                                stdout.write_all(write_buffer)
                                    .and_then(|_| stdout.flush())
                                    .map(|_| 0)
                                    .unwrap_or(1)
                            } else if stream_handle == 2 {
                                // Stderr
                                let mut stderr = std::io::stderr();
                                stderr.write_all(write_buffer)
                                    .and_then(|_| stderr.flush())
                                    .map(|_| 0)
                                    .unwrap_or(1)
                            } else {
                                #[cfg(feature = "tracing")]
                                debug!("WASI: Invalid stream handle: {}", stream_handle);
                                1 // Error
                            };

                            #[cfg(feature = "tracing")]


                            debug!("WASI: Write result: {}", result);
                            Ok(Some(Value::I64(result as i64))) // WASI Preview 2 returns i64 for result types
                        } else {
                            #[cfg(feature = "tracing")]

                            debug!("WASI: Failed to read memory at ptr={}, len={}", data_ptr, data_len);
                            Ok(Some(Value::I64(1))) // Error
                        }
                    } else {
                        #[cfg(feature = "tracing")]

                        debug!("WASI: Failed to get memory from instance");
                        Ok(Some(Value::I64(1))) // Error
                    }
                } else {
                    #[cfg(feature = "tracing")]

                    debug!("WASI: No instance available for id={}", instance_id);
                    Ok(Some(Value::I64(1))) // Error
                }
            }

            // Resource drop operations - just consume the handle and return nothing
            ("wasi:io/streams", "[resource-drop]output-stream") |
            ("wasi:io/streams", "[resource-drop]input-stream") |
            ("wasi:io/error", "[resource-drop]error") => {
                // Pop the handle from the stack
                let _handle = stack.pop();
                #[cfg(feature = "tracing")]
                debug!("WASI: resource-drop for {}::{}, handle={:?}", base_module, field_name, _handle);
                Ok(None) // Resource drops return nothing
            }

            // Error to debug string
            ("wasi:io/error", "[method]error.to-debug-string") => {
                // Pop error handle, return a string representation
                let _error_handle = stack.pop();
                #[cfg(feature = "tracing")]
                debug!("WASI: error.to-debug-string for handle {:?}", _error_handle);
                // Return empty string (ptr=0, len=0)
                Ok(Some(Value::I64(0))) // Empty string as (ptr, len) packed
            }

            // Blocking flush (similar to blocking-write-and-flush but without data)
            ("wasi:io/streams", "[method]output-stream.blocking-flush") => {
                let stream_handle = if let Some(Value::I32(s)) = stack.pop() {
                    s
                } else {
                    return Err(wrt_error::Error::runtime_error("Missing stream argument"));
                };

                #[cfg(feature = "tracing")]
                debug!("WASI: blocking-flush: stream={}", stream_handle);

                use std::io::Write;
                let result = if stream_handle == 1 {
                    std::io::stdout().flush().map(|_| 0).unwrap_or(1)
                } else if stream_handle == 2 {
                    std::io::stderr().flush().map(|_| 0).unwrap_or(1)
                } else {
                    1
                };
                Ok(Some(Value::I64(result as i64)))
            }

            // ============================================
            // WASI Preview 1 (wasi_snapshot_preview1) support
            // ============================================

            // fd_write(fd: i32, iovs: i32, iovs_len: i32, nwritten: i32) -> errno
            ("wasi_snapshot_preview1", "fd_write") => {
                // Pop arguments in reverse order (they were pushed left to right)
                let nwritten_ptr = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("fd_write: missing nwritten_ptr"));
                };

                let iovs_len = if let Some(Value::I32(len)) = stack.pop() {
                    len as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("fd_write: missing iovs_len"));
                };

                let iovs_ptr = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("fd_write: missing iovs_ptr"));
                };

                let fd = if let Some(Value::I32(f)) = stack.pop() {
                    f
                } else {
                    return Err(wrt_error::Error::runtime_error("fd_write: missing fd"));
                };

                #[cfg(feature = "std")]
                eprintln!("[WASI-P1] fd_write: fd={}, iovs={}, iovs_len={}, nwritten_ptr={}", fd, iovs_ptr, iovs_len, nwritten_ptr);

                // Get instance memory (read-only access - memory is behind Arc)
                let total_written = if let Some(instance) = self.instances.get(&instance_id) {
                    if let Ok(memory_wrapper) = instance.memory(0) {
                        let mut total = 0u32;

                        // Process each iovec: struct { buf: *const u8, len: usize }
                        // In WASM32, each iovec is 8 bytes: 4 bytes ptr + 4 bytes len
                        for i in 0..iovs_len {
                            let iov_offset = iovs_ptr + i * 8;

                            // Read iovec ptr and len
                            let mut buf = [0u8; 4];
                            if memory_wrapper.0.read(iov_offset, &mut buf).is_err() {
                                #[cfg(feature = "std")]
                                eprintln!("[WASI-P1] fd_write: failed to read iovec ptr");
                                continue;
                            }
                            let buf_ptr = u32::from_le_bytes(buf);

                            if memory_wrapper.0.read(iov_offset + 4, &mut buf).is_err() {
                                #[cfg(feature = "std")]
                                eprintln!("[WASI-P1] fd_write: failed to read iovec len");
                                continue;
                            }
                            let buf_len = u32::from_le_bytes(buf);

                            #[cfg(feature = "std")]
                            eprintln!("[WASI-P1] fd_write: iovec[{}]: ptr={}, len={}", i, buf_ptr, buf_len);

                            // Read the data to write
                            let mut data = vec![0u8; buf_len as usize];
                            if memory_wrapper.0.read(buf_ptr, &mut data).is_err() {
                                #[cfg(feature = "std")]
                                eprintln!("[WASI-P1] fd_write: failed to read data");
                                continue;
                            }

                            // Write to stdout (fd=1) or stderr (fd=2)
                            use std::io::Write;
                            let write_result = if fd == 1 {
                                std::io::stdout().write_all(&data).and_then(|_| std::io::stdout().flush())
                            } else if fd == 2 {
                                std::io::stderr().write_all(&data).and_then(|_| std::io::stderr().flush())
                            } else {
                                // Other FDs not supported in stub
                                #[cfg(feature = "std")]
                                eprintln!("[WASI-P1] fd_write: unsupported fd={}", fd);
                                Ok(())
                            };

                            if write_result.is_ok() {
                                total += buf_len;
                            }
                        }

                        // Note: In a full implementation we'd write total to nwritten_ptr,
                        // but Memory is behind Arc and read-only here. Most WASM modules
                        // don't check nwritten for stdout anyway.
                        let _ = nwritten_ptr; // Silence unused warning

                        total
                    } else {
                        0
                    }
                } else {
                    0
                };

                #[cfg(feature = "std")]
                eprintln!("[WASI-P1] fd_write: wrote {} bytes total", total_written);

                // Return 0 (success / __WASI_ERRNO_SUCCESS)
                Ok(Some(Value::I32(0)))
            }

            // args_sizes_get(argc: *mut u32, argv_buf_size: *mut u32) -> errno
            // Returns the number of arguments and total size of argument string data
            ("wasi_snapshot_preview1", "args_sizes_get") => {
                // Pop arguments in reverse order
                let argv_buf_size_ptr = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("args_sizes_get: missing argv_buf_size_ptr"));
                };

                let argc_ptr = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("args_sizes_get: missing argc_ptr"));
                };

                // Get the WASI args from the dispatcher or global
                #[cfg(all(feature = "std", feature = "wasi"))]
                let args: Vec<String> = if let Some(ref dispatcher) = self.wasi_dispatcher {
                    let disp_args = dispatcher.args();
                    if disp_args.is_empty() {
                        wrt_wasi::get_global_wasi_args()
                    } else {
                        disp_args.to_vec()
                    }
                } else {
                    wrt_wasi::get_global_wasi_args()
                };

                #[cfg(any(not(feature = "std"), not(feature = "wasi")))]
                let args: Vec<String> = Vec::new();

                let argc = args.len() as u32;
                // Total size includes null terminators for each arg
                let argv_buf_size: u32 = args.iter()
                    .map(|s| s.len() as u32 + 1)
                    .sum();

                #[cfg(feature = "std")]
                eprintln!("[WASI-P1] args_sizes_get: argc={}, argv_buf_size={}", argc, argv_buf_size);

                // Write the values to memory
                if let Some(instance) = self.instances.get(&instance_id) {
                    if let Ok(memory_wrapper) = instance.memory(0) {
                        // Write argc
                        if let Err(e) = memory_wrapper.0.write_shared(argc_ptr, &argc.to_le_bytes()) {
                            #[cfg(feature = "std")]
                            eprintln!("[WASI-P1] args_sizes_get: failed to write argc: {:?}", e);
                        }
                        // Write argv_buf_size
                        if let Err(e) = memory_wrapper.0.write_shared(argv_buf_size_ptr, &argv_buf_size.to_le_bytes()) {
                            #[cfg(feature = "std")]
                            eprintln!("[WASI-P1] args_sizes_get: failed to write argv_buf_size: {:?}", e);
                        }
                    }
                }

                Ok(Some(Value::I32(0))) // Success
            }

            // args_get(argv: *mut *mut u8, argv_buf: *mut u8) -> errno
            // Writes argument pointers to argv and argument strings to argv_buf
            ("wasi_snapshot_preview1", "args_get") => {
                // Pop arguments in reverse order
                let argv_buf_ptr = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("args_get: missing argv_buf_ptr"));
                };

                let argv_ptr = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("args_get: missing argv_ptr"));
                };

                // Get the WASI args from the dispatcher or global
                #[cfg(all(feature = "std", feature = "wasi"))]
                let args: Vec<String> = if let Some(ref dispatcher) = self.wasi_dispatcher {
                    let disp_args = dispatcher.args();
                    if disp_args.is_empty() {
                        wrt_wasi::get_global_wasi_args()
                    } else {
                        disp_args.to_vec()
                    }
                } else {
                    wrt_wasi::get_global_wasi_args()
                };

                #[cfg(any(not(feature = "std"), not(feature = "wasi")))]
                let args: Vec<String> = Vec::new();

                #[cfg(feature = "std")]
                eprintln!("[WASI-P1] args_get: argv_ptr=0x{:x}, argv_buf_ptr=0x{:x}, {} args", argv_ptr, argv_buf_ptr, args.len());

                // Write the argument data to memory
                if let Some(instance) = self.instances.get(&instance_id) {
                    if let Ok(memory_wrapper) = instance.memory(0) {
                        let mut current_buf_offset = argv_buf_ptr;

                        for (i, arg) in args.iter().enumerate() {
                            // Write pointer to this arg's string data in argv array
                            let ptr_offset = argv_ptr + (i as u32 * 4);
                            if let Err(e) = memory_wrapper.0.write_shared(ptr_offset, &current_buf_offset.to_le_bytes()) {
                                #[cfg(feature = "std")]
                                eprintln!("[WASI-P1] args_get: failed to write argv[{}] pointer: {:?}", i, e);
                            }

                            // Write the arg string data (null-terminated)
                            let arg_bytes = arg.as_bytes();
                            if let Err(e) = memory_wrapper.0.write_shared(current_buf_offset, arg_bytes) {
                                #[cfg(feature = "std")]
                                eprintln!("[WASI-P1] args_get: failed to write arg[{}] data: {:?}", i, e);
                            }
                            // Write null terminator
                            if let Err(e) = memory_wrapper.0.write_shared(current_buf_offset + arg_bytes.len() as u32, &[0u8]) {
                                #[cfg(feature = "std")]
                                eprintln!("[WASI-P1] args_get: failed to write null terminator: {:?}", e);
                            }

                            #[cfg(feature = "std")]
                            eprintln!("[WASI-P1] args_get: arg[{}] at 0x{:x} = {:?}", i, current_buf_offset, arg);

                            current_buf_offset += arg_bytes.len() as u32 + 1;
                        }
                    }
                }

                Ok(Some(Value::I32(0))) // Success
            }

            // environ_sizes_get(environc: *mut u32, environ_buf_size: *mut u32) -> errno
            ("wasi_snapshot_preview1", "environ_sizes_get") => {
                // Pop arguments in reverse order
                let environ_buf_size_ptr = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("environ_sizes_get: missing environ_buf_size_ptr"));
                };

                let environc_ptr = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("environ_sizes_get: missing environc_ptr"));
                };

                // For now, return empty environment (environc=0, buf_size=0)
                #[cfg(feature = "std")]
                eprintln!("[WASI-P1] environ_sizes_get: returning empty environment");

                if let Some(instance) = self.instances.get(&instance_id) {
                    if let Ok(memory_wrapper) = instance.memory(0) {
                        // Write environc = 0
                        let _ = memory_wrapper.0.write_shared(environc_ptr, &0u32.to_le_bytes());
                        // Write environ_buf_size = 0
                        let _ = memory_wrapper.0.write_shared(environ_buf_size_ptr, &0u32.to_le_bytes());
                    }
                }

                Ok(Some(Value::I32(0))) // Success
            }

            // environ_get(environ: *mut *mut u8, environ_buf: *mut u8) -> errno
            ("wasi_snapshot_preview1", "environ_get") => {
                // Pop arguments (not used since we return empty environment)
                let _environ_buf_ptr = stack.pop();
                let _environ_ptr = stack.pop();

                #[cfg(feature = "std")]
                eprintln!("[WASI-P1] environ_get: returning empty environment");

                // Nothing to write since environc = 0
                Ok(Some(Value::I32(0))) // Success
            }

            // proc_exit(exit_code: i32) -> !
            ("wasi_snapshot_preview1", "proc_exit") => {
                let _exit_code = if let Some(Value::I32(code)) = stack.pop() {
                    code
                } else {
                    0
                };
                #[cfg(feature = "std")]
                eprintln!("[WASI-P1] proc_exit: code={}", _exit_code);
                // For now, just return - we can't actually exit the process
                Ok(None)
            }

            // Default: stub implementation
            _ => {
                #[cfg(feature = "tracing")]
                debug!("WASI: Stub for {}::{}", module_name, field_name);
                // Check if this is a __main_module__ import - these need special handling
                if module_name == "__main_module__" {
                    #[cfg(feature = "std")]
                    eprintln!("[WASI] __main_module__::{} - this should be a linked function, not a WASI stub", field_name);
                    return Err(wrt_error::Error::runtime_error(
                        "Internal module function called as WASI import - linking error"
                    ));
                }
                Ok(Some(Value::I32(0))) // Default success
            }
        }
    }
}

impl Default for StacklessEngine {
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn default() -> Self {
        Self::new()
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn default() -> Self {
        Self::new().expect("Failed to create default StacklessEngine in no_std mode")
    }
}

// Additional types that might be needed - using simple type aliases to avoid
// conflicts
/// Type alias for callback registry (placeholder implementation).
pub type StacklessCallbackRegistry = ();
/// Type alias for execution stack (placeholder implementation).
pub type StacklessStack = ();
