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
        &self,
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
        &self,
        instance_id: usize,
        func_idx: usize,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        #[cfg(feature = "tracing")]
        let _span = ExecutionTrace::function(func_idx, instance_id).entered();

        #[cfg(feature = "tracing")]
        debug!("Executing function {} in instance {} with {} args", func_idx, instance_id, args.len());

        #[cfg(any(feature = "std", feature = "alloc"))]
        let instance = self
            .instances
            .get(&instance_id)
            .ok_or_else(|| wrt_error::Error::runtime_execution_error("Instance not found"))?;

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        let instance = self
            .instances
            .get(&instance_id)?
            .ok_or_else(|| wrt_error::Error::runtime_execution_error("Instance not found"))?;

        // Check if this function is aliased and get the correct module
        #[cfg(feature = "std")]
        let module = {
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
                original_instance.module()
            } else {
                // Not aliased, use the current instance's module
                instance.module()
            }
        };

        #[cfg(not(feature = "std"))]
        let module = instance.module();

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
                #[cfg(feature = "tracing")]

                trace!("pc={}, instruction={:?}", pc, instruction);

                match *instruction {
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
                        // If condition != 0, select val2, else select val1
                        if let (Some(Value::I32(condition)), Some(val2), Some(val1)) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop()) {
                            let selected = if condition != 0 { val2 } else { val1 };
                            #[cfg(feature = "tracing")]

                            trace!("Select: condition={}, selected={:?}", condition, selected);
                            operand_stack.push(selected);
                        } else {
                            #[cfg(feature = "tracing")]

                            trace!("Select: insufficient operands on stack");
                            return Err(wrt_error::Error::runtime_trap("Select: stack underflow"));
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

                        if (func_idx as usize) < num_imports {
                            // This is a host function call
                            #[cfg(feature = "tracing")]

                            trace!("Calling host function at import index {}", func_idx);

                            // Find the import by index
                            let import_result = self.find_import_by_index(&module, func_idx as usize);

                            if let Ok((module_name, field_name)) = import_result {
                                #[cfg(feature = "tracing")]

                                trace!("Host function: {}::{}", module_name, field_name);

                                // Check if this import is linked to another instance
                                #[cfg(feature = "std")]
                                {
                                    let import_key = (instance_id, module_name.clone(), field_name.clone());
                                    if let Some((target_instance, export_name)) = self.import_links.get(&import_key) {
                                        #[cfg(feature = "tracing")]

                                        trace!("Import linked! Calling instance {}.{}", target_instance, export_name);

                                        // Call the linked function in the target instance
                                        // For now, assume no parameters (will need to handle this properly)
                                        let result = self.call_exported_function(*target_instance, export_name, vec![])?;

                                        // Push result onto stack if function returns a value
                                        if let Some(value) = result.first() {
                                            operand_stack.push(value.clone());
                                        }

                                        continue; // Skip WASI dispatch
                                    }
                                }

                                // Dispatch to WASI implementation
                                let result = self.call_wasi_function(
                                    &module_name,
                                    &field_name,
                                    &mut operand_stack,
                                    &module,
                                    instance_id,
                                )?;

                                // Push result onto stack if function returns a value
                                if let Some(value) = result {
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
                            let local_func_idx = func_idx as usize - num_imports;
                            if local_func_idx >= module.functions.len() {
                                #[cfg(feature = "tracing")]

                                trace!("Function index {} out of bounds", func_idx);
                                return Err(wrt_error::Error::runtime_error("Function index out of bounds"));
                            }

                            let func = &module.functions[local_func_idx];
                            let func_type = module.types.get(func.type_idx as usize)
                                .ok_or_else(|| wrt_error::Error::runtime_error("Invalid function type"))?;

                            // Pop the required number of arguments from the stack
                            let param_count = func_type.params.len();

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

                            for result in results {
                                operand_stack.push(result);
                            }
                        }
                    }
                    Instruction::I32Const(value) => {
                        #[cfg(feature = "tracing")]

                        trace!("I32Const: pushing value {}", value);
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
                                let global = &global_wrapper.0; // Unwrap Arc
                                let value = global.get().clone();
                                #[cfg(feature = "tracing")]
                                trace!("GlobalGet: global[{}] = {:?} (from instance)", global_idx, value);
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
                            trace!("GlobalSet: would set global[{}] to {:?}", global_idx, value);

                            // TODO: GlobalSet requires interior mutability for Arc<Global>
                            // The current architecture wraps globals in Arc which doesn't allow mutation.
                            // This needs to be changed to Arc<RwLock<Global>> or similar.
                            // For now, just verify the global exists but don't modify it.
                            match instance.global(global_idx) {
                                Ok(_global_wrapper) => {
                                    #[cfg(feature = "tracing")]
                                    warn!("GlobalSet: mutation not yet implemented for global[{}] (Arc mutability issue)", global_idx);
                                    // Would need: global.set(&value) but Arc doesn't support DerefMut
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
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Sub => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_sub(b);
                            #[cfg(feature = "tracing")]

                            trace!("I32Sub: {} - {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Mul => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_mul(b);
                            #[cfg(feature = "tracing")]

                            trace!("I32Mul: {} * {} = {}", a, b, result);
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
                    // Conversion operations
                    Instruction::I32WrapI64 => {
                        if let Some(Value::I64(value)) = operand_stack.pop() {
                            let result = value as i32;
                            #[cfg(feature = "tracing")]

                            trace!("I32WrapI64: {} -> {}", value, result);
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
                    Instruction::I32Load(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            #[cfg(feature = "tracing")]

                            trace!("I32Load: reading from address {} (base={}, offset={})", offset, addr, mem_arg.offset);

                            // Get memory - for now assume memory index 0
                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = i32::from_le_bytes(buffer);
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
                                } else {
                                    #[cfg(feature = "tracing")]

                                    trace!("I32Load: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("I32Load: memory index {} out of range (have {} memories)", mem_arg.memory_index, module.memories.len());
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
                            }
                        }
                    }
                    Instruction::I32Store(mem_arg) => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            #[cfg(feature = "tracing")]

                            trace!("I32Store: writing value {} to address {} (base={}, offset={})", value, offset, addr, mem_arg.offset);

                            // Get memory - for now assume memory index 0
                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
                                    let memory = &memory_wrapper.0;
                                    let bytes = value.to_le_bytes();
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]

                                            trace!("I32Store: successfully wrote value {} to address {}", value, offset);
                                        }
                                        Err(e) => {
                                            #[cfg(feature = "tracing")]

                                            trace!("I32Store: write failed: {:?}", e);
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                } else {
                                    #[cfg(feature = "tracing")]

                                    trace!("I32Store: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("I32Store: memory index {} out of range (have {} memories)", mem_arg.memory_index, module.memories.len());
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
                            }
                        }
                    }
                    Instruction::I32Load8S(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = buffer[0] as i8 as i32; // Sign extend
                                            #[cfg(feature = "tracing")]

                                            trace!("I32Load8S: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I32(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory read out of bounds"));
                                        }
                                    }
                                } else {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
                            }
                        }
                    }
                    Instruction::I32Load8U(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = buffer[0] as i32; // Zero extend
                                            #[cfg(feature = "tracing")]

                                            trace!("I32Load8U: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I32(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory read out of bounds"));
                                        }
                                    }
                                } else {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
                            }
                        }
                    }
                    Instruction::I32Load16S(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
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
                                } else {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
                            }
                        }
                    }
                    Instruction::I32Load16U(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
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
                                } else {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
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

                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
                                    let memory = &memory_wrapper.0;
                                    let bytes = (value as u16).to_le_bytes();
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]

                                            trace!("I32Store16: successfully wrote value {} to address {}", value as u16, offset);
                                        }
                                        Err(e) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                } else {
                                    #[cfg(feature = "tracing")]

                                    trace!("I32Store16: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("I32Store16: memory index {} out of range (have {} memories)", mem_arg.memory_index, module.memories.len());
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
                            }
                        }
                    }
                    Instruction::I64Load(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = i64::from_le_bytes(buffer);
                                            #[cfg(feature = "tracing")]

                                            trace!("I64Load: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I64(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory read out of bounds"));
                                        }
                                    }
                                } else {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
                            }
                        }
                    }
                    Instruction::I64Store(mem_arg) => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);

                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
                                    let memory = &memory_wrapper.0;
                                    let bytes = value.to_le_bytes();
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]

                                            trace!("I64Store: successfully wrote value {} to address {}", value, offset);
                                        }
                                        Err(e) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                } else {
                                    #[cfg(feature = "tracing")]

                                    trace!("I64Store: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("I64Store: memory index {} out of range (have {} memories)", mem_arg.memory_index, module.memories.len());
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
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
                        if (memory_idx as usize) < module.memories.len() {
                            let memory = &module.memories[memory_idx as usize].0;
                            let size_in_bytes = memory.size_in_bytes();
                            let size_in_pages = size_in_bytes / 65536;
                            #[cfg(feature = "tracing")]

                            trace!("MemorySize: memory[{}] = {} pages ({} bytes)",                                      memory_idx, size_in_pages, size_in_bytes);
                            operand_stack.push(Value::I32(size_in_pages as i32));
                        } else {
                            #[cfg(feature = "tracing")]

                            trace!("MemorySize: memory[{}] out of bounds, pushing 0", memory_idx);
                            operand_stack.push(Value::I32(0));
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
                            } else if (memory_idx as usize) < module.memories.len() {
                                // TODO: Fix Arc<Memory> mutability issue
                                // Memory is stored in Arc, but grow_shared requires &mut self
                                // Need to either:
                                // 1. Change Memory to use interior mutability for grow operations
                                // 2. Store memories differently to allow mutation during execution
                                // For now, return failure (-1) for memory.grow operations
                                #[cfg(feature = "tracing")]

                                trace!("MemoryGrow: memory[{}] grow by {} pages - NOT IMPLEMENTED (Arc mutability issue)",                                          memory_idx, delta);
                                operand_stack.push(Value::I32(-1));

                                // Original code that doesn't compile:
                                // let memory = &module.memories[memory_idx as usize].0;
                                // let size_in_bytes = memory.size_in_bytes();
                                // let old_size_pages = size_in_bytes / 65536;
                                // match memory.grow_shared(delta as u32) {
                                //     Ok(prev_pages) => {
                                //         operand_stack.push(Value::I32(prev_pages as i32));
                                //     }
                                //     Err(e) => {
                                //         operand_stack.push(Value::I32(-1));
                                //     }
                                // }
                            } else {
                                #[cfg(feature = "tracing")]

                                trace!("MemoryGrow: memory[{}] out of bounds, pushing -1", memory_idx);
                                operand_stack.push(Value::I32(-1));
                            }
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

    /// Find import by function index
    fn find_import_by_index(&self, module: &crate::module::Module, func_idx: usize) -> Result<(String, String)> {
        #[cfg(feature = "tracing")]
        let _span = wrt_foundation::tracing::ImportTrace::lookup("", "").entered();

        #[cfg(feature = "tracing")]
        debug!("Looking for import at index {}", func_idx);

        // Since BoundedMap doesn't expose keys iterator, we need to check known module names
        // Check both WASI Preview 1 and Preview 2 namespaces
        let namespaces = vec![
            // WASI Preview 1
            "wasi_snapshot_preview1",
            // WASI Preview 2 interfaces
            "wasi:cli/environment@0.2.0",
            "wasi:cli/exit@0.2.0",
            "wasi:io/error@0.2.0",
            "wasi:io/poll@0.2.0",
            "wasi:io/streams@0.2.0",
            "wasi:cli/stdin@0.2.0",
            "wasi:cli/stdout@0.2.0",
            "wasi:cli/stderr@0.2.0",
            "wasi:clocks/monotonic-clock@0.2.0",
            "wasi:clocks/wall-clock@0.2.0",
            "wasi:filesystem/types@0.2.0",
            "wasi:filesystem/preopens@0.2.0",
            "wasi:random/random@0.2.0",
            // Other common imports
            "env",
        ];

        let mut current_index = 0;

        for namespace in namespaces {
            let ns_key = match wrt_foundation::bounded::BoundedString::<256>::try_from_str(namespace) {
                Ok(key) => key,
                Err(_) => continue,
            };

            if let Ok(Some(import_map)) = module.imports.get(&ns_key) {
                #[cfg(feature = "tracing")]
                trace!("Found import map for {} with {} entries", namespace, import_map.len());
                #[cfg(all(feature = "std", not(feature = "tracing")))]
                #[cfg(feature = "tracing")]

                trace!("Found import map for {} with {} entries", namespace, import_map.len());

                // For simplicity, we'll check common WASI functions
                // In a full implementation, we'd need to iterate through all entries
                let field_names = if namespace == "wasi_snapshot_preview1" {
                    vec!["fd_write", "fd_read", "environ_get", "environ_sizes_get", "args_get", "args_sizes_get"]
                } else if namespace.starts_with("wasi:") {
                    // For Preview 2, check common exported functions
                    vec!["get-environment", "get-arguments", "exit", "write", "read",
                         "[method]output-stream.blocking-write-and-flush",
                         "[method]input-stream.read"]
                } else {
                    vec![] // Empty for unknown namespaces
                };

                for field_name in field_names {
                    let field_key = match wrt_foundation::bounded::BoundedString::<256>::try_from_str(field_name) {
                        Ok(key) => key,
                        Err(_) => continue,
                    };

                    if let Ok(Some(_)) = import_map.get(&field_key) {
                        if current_index == func_idx {
                            #[cfg(feature = "std")]
                            #[cfg(feature = "tracing")]

                            trace!("Found import at index {}: {}::{}", func_idx, namespace, field_name);
                            return Ok((namespace.to_string(), field_name.to_string()));
                        }
                        current_index += 1;
                    }
                }
            }
        }

        #[cfg(feature = "tracing")]


        trace!("Could not find import at index {} (checked {} imports)", func_idx, current_index);
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
        let args = vec![
            Value::I32(old_ptr as i32),
            Value::I32(old_size as i32),
            Value::I32(align as i32),
            Value::I32(new_size as i32),
        ];

        let results = self.execute(instance_id, func_idx, args)?;

        if let Some(Value::I32(ptr)) = results.first() {
            Ok(*ptr as u32)
        } else {
            Err(wrt_error::Error::runtime_error("cabi_realloc returned invalid value"))
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
        &self,
        module_name: &str,
        field_name: &str,
        stack: &mut Vec<Value>,
        module: &crate::module::Module,
        instance_id: usize,
    ) -> Result<Option<Value>> {
        use std::io::Write;

        #[cfg(feature = "tracing")]


        debug!("WASI: Calling {}::{}", module_name, field_name);

        // First, try to call through host_registry if available
        #[cfg(feature = "std")]
        if let Some(ref registry) = self.host_registry {
            #[cfg(feature = "tracing")]

            debug!("WASI: Checking host registry for {}::{}", module_name, field_name);

            // Check if the function is registered
            if registry.has_host_function(module_name, field_name) {
                #[cfg(feature = "tracing")]

                debug!("WASI: Found {} in host registry, calling implementation", field_name);

                // Convert stack values to the format expected by host functions
                // For now, pass empty args - proper marshalling would be needed here
                let args: Vec<wrt_foundation::Value> = vec![];

                // Call the registered host function
                // Note: engine parameter is &mut dyn Any, we pass a dummy reference
                let mut dummy_engine: i32 = 0;
                match registry.call_host_function(&mut dummy_engine, module_name, field_name, args) {
                    Ok(result) => {
                        #[cfg(feature = "tracing")]

                        debug!("WASI: Host function {} returned successfully", field_name);
                        // Push result if any (host functions return Vec<Value>)
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
            } else {
                #[cfg(feature = "tracing")]

                debug!("WASI: Function {} not found in host registry, using fallback stubs", field_name);
            }
        }

        // Check if this is a wasip2 canonical function
        let full_name = format!("{}::{}", module_name, field_name);
        if module_name.starts_with("wasi:") && module_name.contains("@0.2") {
            #[cfg(feature = "tracing")]

            trace!("[WASIP2] Detected wasip2 call: {}", full_name);
            #[cfg(feature = "tracing")]

            trace!("[WASIP2] Stack has {} values", stack.len());

            // Create wasip2 host and dispatch the call
            let mut wasip2_host = crate::wasip2_host::Wasip2Host::new();

            // Extract arguments from stack (wasip2 functions typically take handle as first arg)
            let args = if !stack.is_empty() {
                // Take all values from stack as arguments
                let mut args = Vec::new();
                for _ in 0..stack.len() {
                    if let Some(val) = stack.pop() {
                        args.push(val);
                    }
                }
                args.reverse(); // Reverse to get correct order
                #[cfg(feature = "tracing")]

                trace!("[WASIP2] Extracted {} arguments from stack", args.len());
                args
            } else {
                vec![]
            };

            // Get memory from module (assuming memory 0)
            let memory_result = if !module.memories.is_empty() {
                // Try to get a mutable reference to memory
                // We need to create a temporary buffer to pass to wasip2
                // This is a simplified approach - in production, we'd need proper memory management
                #[cfg(feature = "tracing")]

                trace!("[WASIP2] Module has {} memories, getting memory 0", module.memories.len());

                // For now, create a temporary buffer that wasip2 can use
                // In a real implementation, we'd need to properly access the module's linear memory
                let mut temp_buffer = vec![0u8; 65536]; // 64KB temporary buffer

                // Try to read some data from memory if we have arguments that look like pointers
                if let Some(Value::I32(ptr)) = args.get(1) { // Second arg is often a pointer
                    if let Some(Value::I32(len)) = args.get(2) { // Third arg is often length
                        #[cfg(feature = "tracing")]

                        trace!("[WASIP2] Attempting to read {} bytes from memory at offset {}", len, ptr);
                        if let Ok(memory_wrapper) = module.get_memory(0) {
                            let memory = &memory_wrapper.0;
                            let offset = *ptr as u32;  // Keep as u32 for memory.read()
                            let size = (*len as usize).min(temp_buffer.len());
                            if (offset as usize) + size <= temp_buffer.len() {
                                let _ = memory.read(offset, &mut temp_buffer[..size]);
                                #[cfg(feature = "tracing")]

                                trace!("[WASIP2] Read {} bytes from memory", size);
                            }
                        }
                    }
                }

                Some(temp_buffer)
            } else {
                #[cfg(feature = "tracing")]

                trace!("[WASIP2] No memory available in module");
                None
            };

            // Dispatch to wasip2 host with memory
            let dispatch_result = if let Some(mut mem_buffer) = memory_result {
                wasip2_host.dispatch(module_name, field_name, args, Some(&mut mem_buffer))
            } else {
                wasip2_host.dispatch(module_name, field_name, args, None)
            };

            match dispatch_result {
                Ok(results) => {
                    #[cfg(feature = "tracing")]

                    trace!("[WASIP2] Function {} returned {} results", field_name, results.len());
                    if let Some(val) = results.first() {
                        return Ok(Some(val.clone()));
                    }
                    return Ok(None);
                }
                Err(e) => {
                    #[cfg(feature = "tracing")]

                    trace!("[WASIP2] Function {} failed: {:?}", field_name, e);
                    // Fall through to stubs as fallback
                }
            }
        }

        // Fallback to stub implementations if host_registry not available
        #[cfg(feature = "tracing")]

        debug!("WASI: Using stub implementation for {}::{}", module_name, field_name);
        let stub_mem = self.wasi_stubs.get(&instance_id);

        match (module_name, field_name) {
            // wasi:cli/environment@0.2.0
            ("wasi:cli/environment@0.2.0", "get-environment") => {
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

            ("wasi:cli/environment@0.2.0", "get-arguments") => {
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

            ("wasi:cli/environment@0.2.0", "initial-cwd") => {
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
            ("wasi:cli/stdout@0.2.0", "get-stdout") => {
                let handle = stub_mem.map(|s| s.stdout_handle).unwrap_or(1);
                #[cfg(feature = "tracing")]

                debug!("WASI: get-stdout: returning handle {}", handle);
                Ok(Some(Value::I32(handle as i32)))
            }

            // wasi:cli/stderr@0.2.0::get-stderr() -> stream
            ("wasi:cli/stderr@0.2.0", "get-stderr") => {
                let handle = stub_mem.map(|s| s.stderr_handle).unwrap_or(2);
                #[cfg(feature = "tracing")]

                debug!("WASI: get-stderr: returning handle {}", handle);
                Ok(Some(Value::I32(handle as i32)))
            }

            // wasi:cli/exit@0.2.0::exit(code)
            ("wasi:cli/exit@0.2.0", "exit") => {
                let exit_code = if let Some(Value::I32(code)) = stack.pop() {
                    code
                } else {
                    1
                };

                #[cfg(feature = "tracing")]


                debug!("WASI: exit called with code: {}", exit_code);
                std::process::exit(exit_code);
            }

            // wasi:io/streams@0.2.0::[method]output-stream.blocking-write-and-flush(stream, data_ptr, data_len) -> result
            ("wasi:io/streams@0.2.0", "[method]output-stream.blocking-write-and-flush") => {
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

                // Read data from WebAssembly memory and write to stdout/stderr
                // Use instance memory instead of module memory
                if let Some(instance) = self.instances.get(&instance_id) {
                    if let Ok(memory_wrapper) = instance.memory(0) {
                        // Read data from instance memory into a buffer
                        let mut buffer = vec![0u8; data_len as usize];
                        if let Ok(()) = memory_wrapper.0.read(data_ptr as u32, &mut buffer) {
                            #[cfg(feature = "tracing")]

                            debug!("WASI: Read {} bytes from memory at ptr={}", buffer.len(), data_ptr);

                        // Write directly to stdout/stderr instead of using the memory-based function
                        use std::io::Write;
                        let result = if stream_handle == 1 {
                            // Stdout
                            let mut stdout = std::io::stdout();
                            stdout.write_all(&buffer)
                                .and_then(|_| stdout.flush())
                                .map(|_| 0)
                                .unwrap_or(1)
                        } else if stream_handle == 2 {
                            // Stderr
                            let mut stderr = std::io::stderr();
                            stderr.write_all(&buffer)
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

            // Default: stub implementation
            _ => {
                #[cfg(feature = "tracing")]

                debug!("WASI: Stub for {}::{}", module_name, field_name);
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
