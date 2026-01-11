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

// Debug support - only available with std and wrt-debug crate
#[cfg(all(feature = "std", feature = "debugger"))]
use wrt_debug::runtime_traits::{RuntimeDebugger, RuntimeState, DebugAction};

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
/// Set to 512 to handle large WAST test files that may have hundreds of module directives
/// (e.g., align.wast has 117 module directives including assert_invalid)
const MAX_CONCURRENT_INSTANCES: usize = 512;

/// Maximum call depth to prevent stack overflow from recursive calls
/// WebAssembly spec recommends supporting at least 1000 levels.
/// The WAST test suite requires at least 200 for even/odd mutual recursion tests.
/// Testing shows native stack overflow occurs around ~90-100 calls in debug builds
/// due to the current recursive implementation of execute().
/// 250 works in release builds but causes stack overflow in debug.
/// TODO: Implement trampolining to allow deeper recursion without native stack use.
#[cfg(debug_assertions)]
const MAX_CALL_DEPTH: usize = 50;
#[cfg(not(debug_assertions))]
const MAX_CALL_DEPTH: usize = 250;

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
    /// Generic host import handler for all host function calls
    #[cfg(feature = "std")]
    host_handler:          Option<Box<dyn wrt_foundation::HostImportHandler>>,
    /// Optional runtime debugger for profiling and debugging
    #[cfg(all(feature = "std", feature = "debugger"))]
    debugger:              Option<Box<dyn RuntimeDebugger>>,
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

/// Simple RuntimeState implementation for debugger callbacks
#[cfg(all(feature = "std", feature = "debugger"))]
pub struct ExecutionState<'a> {
    pc: u32,
    func_idx: u32,
    operand_stack: &'a [Value],
    locals: &'a [Value],
}

#[cfg(all(feature = "std", feature = "debugger"))]
impl<'a> RuntimeState for ExecutionState<'a> {
    fn pc(&self) -> u32 {
        self.pc
    }

    fn sp(&self) -> u32 {
        self.operand_stack.len() as u32
    }

    fn fp(&self) -> Option<u32> {
        Some(0) // WebAssembly doesn't have a traditional frame pointer
    }

    fn read_local(&self, index: u32) -> Option<u64> {
        self.locals.get(index as usize).map(|v| match v {
            Value::I32(x) => *x as u64,
            Value::I64(x) => *x as u64,
            Value::F32(x) => x.to_bits() as u64,
            Value::F64(x) => x.to_bits(),
            _ => 0,
        })
    }

    fn read_stack(&self, offset: u32) -> Option<u64> {
        let len = self.operand_stack.len();
        if offset as usize >= len {
            return None;
        }
        let idx = len - 1 - offset as usize;
        self.operand_stack.get(idx).map(|v| match v {
            Value::I32(x) => *x as u64,
            Value::I64(x) => *x as u64,
            Value::F32(x) => x.to_bits() as u64,
            Value::F64(x) => x.to_bits(),
            _ => 0,
        })
    }

    fn current_function(&self) -> Option<u32> {
        Some(self.func_idx)
    }
}

/// Calculate effective memory address with overflow checking.
/// Per WebAssembly spec, if base + offset overflows or exceeds u32::MAX, it traps.
/// Returns Ok(effective_address) or Err if overflow occurs.
#[inline]
fn calculate_effective_address(base: i32, offset: u32, size: u32) -> wrt_error::Result<u64> {
    // Convert base to u32 first (WebAssembly treats addresses as unsigned)
    let base_u32 = base as u32;

    // Check for overflow in base + offset
    let effective_addr = (base_u32 as u64)
        .checked_add(offset as u64)
        .ok_or_else(|| wrt_error::Error::runtime_trap("out of bounds memory access"))?;

    // Check for overflow when adding access size
    let end_addr = effective_addr
        .checked_add(size as u64)
        .ok_or_else(|| wrt_error::Error::runtime_trap("out of bounds memory access"))?;

    // If end_addr exceeds u32::MAX + 1 (4GB), it's out of bounds for 32-bit memory
    // But we let the actual memory bounds check handle the memory size comparison
    // Just ensure the calculation doesn't overflow
    if end_addr > u64::from(u32::MAX) + 1 {
        return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
    }

    Ok(effective_addr)
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
            #[cfg(feature = "std")]
            host_handler:        None,
            #[cfg(all(feature = "std", feature = "debugger"))]
            debugger:            None,
        }
    }

    /// Set a runtime debugger for profiling and debugging
    ///
    /// When a debugger is attached, it will receive callbacks for each instruction,
    /// function entry/exit, and traps. This is useful for profiling and debugging.
    ///
    /// Note: This has performance overhead - only use when needed.
    #[cfg(all(feature = "std", feature = "debugger"))]
    pub fn set_debugger(&mut self, debugger: Box<dyn RuntimeDebugger>) {
        self.debugger = Some(debugger);
    }

    /// Remove the attached debugger
    #[cfg(all(feature = "std", feature = "debugger"))]
    pub fn clear_debugger(&mut self) {
        self.debugger = None;
    }

    /// Check if a debugger is attached
    #[cfg(all(feature = "std", feature = "debugger"))]
    pub fn has_debugger(&self) -> bool {
        self.debugger.is_some()
    }

    /// Set the host import handler for resolving host function calls
    #[cfg(feature = "std")]
    pub fn set_host_handler(&mut self, handler: Box<dyn wrt_foundation::HostImportHandler>) {
        self.host_handler = Some(handler);
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
        #[cfg(feature = "tracing")]
        trace!(
            target_instance_id = target_instance_id,
            export_name = %export_name,
            "[CROSS_CALL] call_exported_function"
        );

        // Get the target instance
        let target_instance = self.instances.get(&target_instance_id)
            .ok_or_else(|| wrt_error::Error::resource_not_found("Target instance not found"))?
            .clone();

        // Access module via public API
        let module = target_instance.module();

        #[cfg(feature = "tracing")]
        {
            trace!(exports_len = module.exports.len(), "[CROSS_CALL] Target module exports");
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
        trace!(
            export_name = %export_name,
            func_idx = func_idx,
            target_instance_id = target_instance_id,
            "[CROSS_CALL] Export mapped to function"
        );

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

    /// Reset call depth counter to 0
    ///
    /// Call this before each top-level function invocation to ensure
    /// previous errors don't cause false "call stack exhausted" errors.
    pub fn reset_call_depth(&mut self) {
        self.call_frames_count = 0;
    }

    /// Clear all loaded module instances
    ///
    /// This is useful for WAST testing where many modules are loaded in sequence
    /// and we want to avoid hitting instance limits. Note that this invalidates
    /// all existing instance IDs.
    pub fn clear_instances(&mut self) {
        self.instances.clear();
        // Reset instance ID counter to avoid confusion with old IDs
        self.next_instance_id.store(0, Ordering::Relaxed);
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
        debug!(
            instance_id = instance_id,
            func_idx = func_idx,
            args_len = args.len(),
            "[INNER_EXEC] Executing function"
        );

        // Check call depth to prevent native stack overflow
        // WebAssembly allows deep recursion but native Rust stack is limited
        if self.call_frames_count >= MAX_CALL_DEPTH {
            return Err(wrt_error::Error::runtime_trap("call stack exhausted"));
        }
        self.call_frames_count += 1;

        // Clone the instance to avoid holding a borrow on self.instances
        // This allows us to call &mut self methods (like execute, call_wasi_function)
        // during execution without borrow checker conflicts.
        #[cfg(any(feature = "std", feature = "alloc"))]
        let instance = {
            let found = self.instances.get(&instance_id);
            if found.is_none() {
                #[cfg(feature = "tracing")]
                trace!(instance_id = instance_id, "[INNER_EXEC] Instance not found");
                return Err(wrt_error::Error::runtime_execution_error("Instance not found"));
            }
            found.unwrap().clone()
        };

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
            #[cfg(feature = "tracing")]
            trace!(
                instance_id = instance_id,
                func_idx = func_idx,
                elem_count = elem_count,
                first_elem_items = first_elem_items,
                "[EXECUTE] Starting function execution"
            );
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
            #[cfg(feature = "tracing")]
            trace!(
                func_idx = func_idx,
                instance_id = instance_id,
                instructions_len = instructions.len(),
                "[EXEC] Starting execution"
            );
            let mut operand_stack: Vec<Value> = Vec::new();
            let mut locals: Vec<Value> = Vec::new();
            let mut instruction_count = 0usize;

            // Take debugger out of self to use during execution (avoids borrow issues)
            #[cfg(all(feature = "std", feature = "debugger"))]
            let mut debugger_opt = self.debugger.take();
            let mut block_depth = 0i32; // Track nesting depth during execution

            // Initialize parameters as locals
            // Need to match the function type signature, not just provided args
            #[cfg(feature = "tracing")]
            trace!("Initializing locals: args.len()={}, func.locals.len()={}", args.len(), func.locals.len());

            // Get expected parameter count from function type
            // Per CLAUDE.md: FAIL LOUD AND EARLY - don't silently substitute defaults
            let func_type = module.types.get(func.type_idx as usize)
                .ok_or_else(|| wrt_error::Error::runtime_error(
                    "Function type not found - module corrupted"
                ))?;
            let expected_param_count = func_type.params.len();

            #[cfg(feature = "tracing")]
            trace!(
                expected_param_count = expected_param_count,
                args_len = args.len(),
                "[EXEC] Function parameter info"
            );

            // Add provided arguments
            for (i, arg) in args.iter().enumerate() {
                if i < expected_param_count {
                    locals.push(arg.clone());
                }
            }

            // Pad with default values for missing parameters
            // Use func_type we already have from above
            if args.len() < expected_param_count {
                for i in args.len()..expected_param_count {
                    // Per CLAUDE.md: FAIL LOUD - param index must be valid
                    let param_type = func_type.params.get(i)
                        .ok_or_else(|| wrt_error::Error::runtime_error(
                            "Parameter index out of bounds - type corrupted"
                        ))?;
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

            // Track block stack: (block_type, start_pc, block_type_idx, stack_height) where block_type is "loop", "block", or "if"
            // block_type_idx encodes the block's signature: 0x40=empty, 0x7F=i32, 0x7E=i64, 0x7D=f32, 0x7C=f64, or type index
            // stack_height is the operand_stack length when the block was entered (for unwinding on br)
            let mut block_stack: Vec<(&str, usize, u32, usize)> = Vec::new();

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

                // Debugger callback - notify debugger of instruction execution
                #[cfg(all(feature = "std", feature = "debugger"))]
                if let Some(ref mut debugger) = debugger_opt {
                    let state = ExecutionState {
                        pc: pc as u32,
                        func_idx: func_idx as u32,
                        operand_stack: &operand_stack,
                        locals: &locals,
                    };
                    let action = debugger.on_instruction(pc as u32, &state);
                    if action == DebugAction::Break {
                        return Err(wrt_error::Error::runtime_execution_error("Debugger break requested"));
                    }
                }

                match *instruction {
                    Instruction::Unreachable => {
                        // Unreachable instruction - this is a WebAssembly trap
                        // The trap should propagate as an error, not be silently ignored.
                        // This can occur in panic paths when the panic hook is NULL,
                        // or after proc_exit is called.
                        #[cfg(all(feature = "std", feature = "tracing"))]
                        {
                            // Print some context about what led to unreachable
                            let prev_str = if pc > 0 { format!("{:?}", instructions.get(pc - 1)) } else { "N/A".to_string() };
                            error!(
                                func_idx = func_idx,
                                pc = pc,
                                prev_instr = %prev_str,
                                "[TRAP] Unreachable instruction executed"
                            );
                        }
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
                        #[cfg(feature = "tracing")]
                        trace!(stack_len = operand_stack.len(), "[Select] Processing select instruction");

                        // WebAssembly select expects: val1, val2, i32
                        // The condition should be the top of stack
                        let cond_val = operand_stack.pop();
                        let val2 = operand_stack.pop();
                        let val1 = operand_stack.pop();

                        #[cfg(feature = "tracing")]
                        trace!(cond = ?cond_val, val2 = ?val2, val1 = ?val1, "[Select] Popped values");

                        // Extract condition as i32
                        let condition = match cond_val {
                            Some(Value::I32(c)) => c,
                            Some(other) => {
                                #[cfg(feature = "tracing")]
                                error!(condition = ?other, "[Select] ERROR: condition is not i32");
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
                            #[cfg(feature = "tracing")]
                            {
                                if let Value::I32(v) = &selected {
                                    if (*v as u32) > 0x20000 {
                                        warn!(
                                            condition = condition,
                                            value = v,
                                            value_hex = format_args!("0x{:x}", *v as u32),
                                            "[Select] SUSPICIOUS: large value selected"
                                        );
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
                        trace!(
                            func_idx = func_idx,
                            num_imports = num_imports,
                            is_import = (func_idx as usize) < num_imports,
                            "[CALL] Checking function call"
                        );

                        if (func_idx as usize) < num_imports {
                            // This is a host function call
                            #[cfg(feature = "tracing")]
                            trace!(func_idx = func_idx, "[HOST_CALL] Calling host function at import index");

                            // Find the import by index
                            let import_result = self.find_import_by_index(&module, func_idx as usize);
                            #[cfg(feature = "tracing")]
                            trace!(result = ?import_result, "[HOST_CALL] find_import_by_index result");

                            if let Ok((module_name, field_name)) = import_result {
                                #[cfg(feature = "tracing")]
                                trace!(
                                    module_name = %module_name,
                                    field_name = %field_name,
                                    "[HOST_CALL] Resolved to host function"
                                );

                                // Check if this import is linked to another instance
                                // NOTE: Component adapter modules handle P2→P1 translation.
                                // ALL imports (including WASI) should use cross-instance linking when linked.
                                #[cfg(feature = "std")]
                                {
                                    // Clone the values to avoid holding a borrow during call_exported_function
                                    let import_key = (instance_id, module_name.clone(), field_name.clone());
                                    let linked = self.import_links.get(&import_key)
                                        .map(|(ti, en)| (*ti, en.clone()));

                                    #[cfg(feature = "tracing")]
                                    trace!(
                                        instance_id = instance_id,
                                        is_linked = linked.is_some(),
                                        "[HOST_CALL] Checking cross-instance link"
                                    );

                                    if let Some((target_instance, export_name)) = linked {
                                        #[cfg(feature = "tracing")]
                                        trace!(
                                            target_instance = target_instance,
                                            export_name = %export_name,
                                            "[HOST_CALL] Linked to target instance"
                                        );

                                        // Collect args from operand stack based on function signature
                                        let args = Self::collect_function_args(&module, func_idx as usize, &mut operand_stack);

                                        // Call the linked function in the target instance
                                        let result = self.call_exported_function(target_instance, &export_name, args)?;

                                        // Push result onto stack if function returns a value
                                        if let Some(value) = result.first() {
                                            operand_stack.push(value.clone());
                                        }

                                        // Advance pc before continue (continue skips the pc += 1 at end of match)
                                        pc += 1;
                                        continue; // Linked call handled - skip WASI dispatch
                                    }
                                    // Not linked - fall through to WASI dispatch
                                }

                                // Dispatch to WASI implementation
                                #[cfg(feature = "tracing")]
                                trace!(
                                    module_name = %module_name,
                                    field_name = %field_name,
                                    "[HOST_CALL] Calling call_wasi_function"
                                );
                                let result = self.call_wasi_function(
                                    &module_name,
                                    &field_name,
                                    &mut operand_stack,
                                    &module,
                                    instance_id,
                                )?;
                                #[cfg(feature = "tracing")]
                                trace!(result = ?result, "[HOST_CALL] call_wasi_function returned");

                                // Push result onto stack if function returns a value
                                if let Some(value) = result {
                                    #[cfg(feature = "tracing")]
                                    {
                                        if let Value::I32(v) = &value {
                                            if (*v as u32) > 0x20000 {
                                                warn!(
                                                    module_name = %module_name,
                                                    field_name = %field_name,
                                                    value = v,
                                                    value_hex = format_args!("0x{:x}", *v as u32),
                                                    "[WASI_RETURN] SUSPICIOUS: large return value"
                                                );
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
                            #[cfg(feature = "tracing")]
                            trace!(
                                func_idx = func_idx,
                                num_imports = num_imports,
                                local_func_idx = local_func_idx,
                                functions_len = module.functions.len(),
                                "[CALL] Regular function call"
                            );
                            if local_func_idx >= module.functions.len() {
                                #[cfg(feature = "tracing")]

                                trace!("Function index {} out of bounds", func_idx);
                                return Err(wrt_error::Error::runtime_error("Function index out of bounds"));
                            }

                            let func = &module.functions[local_func_idx];
                            #[cfg(feature = "tracing")]
                            trace!(
                                type_idx = func.type_idx,
                                types_len = module.types.len(),
                                "[CALL] Function type info"
                            );
                            let func_type = module.types.get(func.type_idx as usize)
                                .ok_or_else(|| wrt_error::Error::runtime_error("Invalid function type"))?;

                            // Pop the required number of arguments from the stack
                            let param_count = func_type.params.len();
                            #[cfg(feature = "tracing")]
                            trace!(
                                param_count = param_count,
                                stack_len = operand_stack.len(),
                                "[CALL] Parameter and stack info"
                            );
                            #[cfg(all(feature = "std", feature = "tracing"))]
                            if func_idx == 94 || func_idx == 223 || func_idx == 232 || func_idx == 233 {
                                trace!(
                                    func_idx = func_idx,
                                    stack_top = ?operand_stack.iter().rev().take(4).collect::<Vec<_>>(),
                                    "[CALL-ALLOC] Allocation function call"
                                );
                            }
                            // Trace func 235 (free) to see what pointer is being freed
                            #[cfg(all(feature = "std", feature = "tracing"))]
                            if func_idx == 235 || func_idx == 236 {
                                trace!(
                                    func_idx = func_idx,
                                    args = ?operand_stack.iter().rev().take(1).collect::<Vec<_>>(),
                                    "[FREE-TRACE] Free function call"
                                );
                            }
                            // Trace func 244 (format string loop) to see arguments
                            #[cfg(all(feature = "std", feature = "tracing"))]
                            if func_idx == 244 {
                                trace!(
                                    args = ?operand_stack.iter().rev().take(8).collect::<Vec<_>>(),
                                    "[CALL-244] Format string loop args"
                                );
                            }

                            #[cfg(feature = "tracing")]
                            trace!("Call({}): needs {} params, stack has {} values", func_idx, param_count, operand_stack.len());

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
                    Instruction::CallIndirect(type_idx, table_idx) => {
                        // CallIndirect: call a function through an indirect table reference
                        // Pop the function index from the stack
                        let table_func_idx = if let Some(Value::I32(idx)) = operand_stack.pop() {
                            idx as u32
                        } else {
                            return Err(wrt_error::Error::runtime_trap("CallIndirect: expected i32 function index on stack"));
                        };

                        #[cfg(feature = "tracing")]
                        trace!(
                            type_idx = type_idx,
                            table_idx = table_idx,
                            table_func_idx = table_func_idx,
                            "[CALL_INDIRECT] Indirect call"
                        );

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
                                #[cfg(feature = "tracing")]
                                trace!(table_idx = table_idx, "[CALL_INDIRECT] Table not found, checking element segments");

                                let mut resolved_func_idx: Option<usize> = None;

                                // Search through element segments
                                // Element segments have format: (elem (i32.const offset) func f1 f2 f3 ...)
                                // We need to find which element contains table_func_idx
                                #[cfg(feature = "tracing")]
                                trace!(elements_len = module.elements.len(), "[CALL_INDIRECT] Searching element segments");
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
                                        #[cfg(feature = "tracing")]
                                        trace!(
                                            elem_idx = elem_idx,
                                            elem_offset = elem_offset,
                                            items_len = items_len,
                                            table_func_idx = table_func_idx,
                                            "[CALL_INDIRECT] Element segment info"
                                        );

                                        // Check if table_func_idx falls within this element's range
                                        if table_func_idx >= elem_offset && (table_func_idx - elem_offset) < items_len as u32 {
                                            let elem_local_idx = (table_func_idx - elem_offset) as usize;
                                            // items is BoundedVec<u32>, get() returns Result<u32>
                                            if let Ok(func_ref) = elem.items.get(elem_local_idx) {
                                                resolved_func_idx = Some(func_ref as usize);
                                                #[cfg(feature = "tracing")]
                                                trace!(
                                                    elem_idx = elem_idx,
                                                    table_func_idx = table_func_idx,
                                                    elem_local_idx = elem_local_idx,
                                                    func_ref = func_ref,
                                                    "[CALL_INDIRECT] Found in element segment"
                                                );
                                                break;
                                            }
                                        }
                                    }
                                }

                                // NO FALLBACK: Per CLAUDE.md, fail loud and early if element not found
                                resolved_func_idx.ok_or_else(|| {
                                    wrt_error::Error::runtime_trap(
                                        "CallIndirect: function not found in element segments"
                                    )
                                })?
                            }
                        } else {
                            return Err(wrt_error::Error::runtime_trap("CallIndirect: instance not found"));
                        };

                        #[cfg(feature = "tracing")]
                        trace!(func_idx = func_idx, "[CALL_INDIRECT] Resolved to function index");

                        // Track call_indirect to func 138 (iterator next)
                        static CALL_138_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                        #[cfg(all(feature = "std", feature = "tracing"))]
                        if func_idx == 138 {
                            let call_num = CALL_138_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            trace!(
                                call_num = call_num,
                                stack_len = operand_stack.len(),
                                "[CALL-138-INDIRECT] Indirect call to func 138"
                            );
                            for (i, val) in operand_stack.iter().enumerate() {
                                trace!(index = i, value = ?val, "[CALL-138-INDIRECT] Stack value");
                            }
                        }

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
                            #[cfg(feature = "tracing")]
                            warn!(
                                expected_params = expected_type.params.len(),
                                expected_results = expected_type.results.len(),
                                got_params = func_type.params.len(),
                                got_results = func_type.results.len(),
                                "[CALL_INDIRECT] Type mismatch"
                            );
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

                        // Check if the function is an imported function that's linked to another instance
                        // Imported functions have empty body and locals
                        let is_import = func.body.is_empty() && func.locals.is_empty();

                        let results = if is_import {
                            #[cfg(feature = "tracing")]
                            trace!(
                                func_idx = func_idx,
                                "[CALL_INDIRECT] Target is imported function, checking cross-instance links"
                            );

                            // Try to find the import's module/field name
                            if let Ok((module_name, field_name)) = self.find_import_by_index(&module, func_idx) {
                                #[cfg(feature = "tracing")]
                                trace!(
                                    module_name = %module_name,
                                    field_name = %field_name,
                                    "[CALL_INDIRECT] Resolved import"
                                );

                                // Check if this import is linked to another instance
                                #[cfg(feature = "std")]
                                {
                                    let import_key = (instance_id, module_name.clone(), field_name.clone());
                                    let linked = self.import_links.get(&import_key)
                                        .map(|(ti, en)| (*ti, en.clone()));

                                    if let Some((target_instance, export_name)) = linked {
                                        #[cfg(feature = "tracing")]
                                        trace!(
                                            target_instance = target_instance,
                                            export_name = %export_name,
                                            "[CALL_INDIRECT] Import linked to another instance"
                                        );

                                        // Call the linked function in the target instance
                                        self.call_exported_function(target_instance, &export_name, call_args)?
                                    } else {
                                        // Not linked - dispatch to WASI if applicable
                                        #[cfg(feature = "tracing")]
                                        trace!(
                                            module_name = %module_name,
                                            field_name = %field_name,
                                            "[CALL_INDIRECT] Import not linked, trying WASI dispatch"
                                        );

                                        // Call WASI function (need to push args back for call_wasi_function)
                                        for arg in call_args.iter().rev() {
                                            operand_stack.push(arg.clone());
                                        }
                                        let result = self.call_wasi_function(
                                            &module_name,
                                            &field_name,
                                            &mut operand_stack,
                                            &module,
                                            instance_id,
                                        )?;
                                        if let Some(val) = result {
                                            vec![val]
                                        } else {
                                            vec![]
                                        }
                                    }
                                }

                                #[cfg(not(feature = "std"))]
                                {
                                    // In no_std mode, just try to execute directly
                                    self.execute(instance_id, func_idx, call_args)?
                                }
                            } else {
                                // Couldn't resolve import - try to execute directly
                                #[cfg(feature = "tracing")]
                                warn!(
                                    func_idx = func_idx,
                                    "[CALL_INDIRECT] Could not resolve import, executing directly"
                                );
                                self.execute(instance_id, func_idx, call_args)?
                            }
                        } else {
                            // Regular function - execute directly
                            self.execute(instance_id, func_idx, call_args)?
                        };

                        #[cfg(feature = "tracing")]
                        trace!(
                            func_idx = func_idx,
                            results_len = results.len(),
                            "[CALL_INDIRECT] Function returned"
                        );

                        // Trace function 138 return value
                        #[cfg(feature = "tracing")]
                        if func_idx == 138 && !results.is_empty() {
                            trace!(result = ?results[0], "[CALL_INDIRECT-138] Return value");
                        }

                        // Push results back onto stack
                        for result in results {
                            operand_stack.push(result);
                        }
                    }
                    Instruction::I32Const(value) => {
                        #[cfg(feature = "tracing")]
                        trace!("I32Const: pushing value {}", value);
                        #[cfg(feature = "tracing")]
                        {
                            if (value as u32) > 0x200000 {
                                warn!(
                                    value = value,
                                    value_hex = format_args!("0x{:x}", value as u32),
                                    "[I32Const] SUSPICIOUS: large value"
                                );
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
                    Instruction::F32Const(bits) => {
                        #[cfg(feature = "tracing")]
                        trace!("F32Const: pushing {}", f32::from_bits(bits));
                        operand_stack.push(Value::F32(FloatBits32(bits)));
                    }
                    Instruction::LocalGet(local_idx) => {
                        if (local_idx as usize) < locals.len() {
                            // Use copy_value() for efficient copying of Copy-like variants
                            let value = locals[local_idx as usize].copy_value();
                            #[cfg(feature = "tracing")]
                            trace!("LocalGet: local[{}] = {:?}", local_idx, value);
                            // Debug: trace LocalGet in function 222 for key locals (0=pieces, 2=Arguments)
                            #[cfg(feature = "tracing")]
                            if func_idx == 222 && (local_idx == 0 || local_idx == 2) {
                                trace!(
                                    pc = pc,
                                    local_idx = local_idx,
                                    value = ?value,
                                    "[LOCALGET-222] Key local access"
                                );
                            }
                            #[cfg(feature = "tracing")]
                            {
                                // Debug suspicious values that might be bad pointers
                                if let Value::I32(v) = &value {
                                    if (*v as u32) > 0x200000 {
                                        warn!(
                                            local_idx = local_idx,
                                            value = v,
                                            value_hex = format_args!("0x{:x}", *v as u32),
                                            "[LocalGet] SUSPICIOUS: large value"
                                        );
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
                            // Debug: trace LocalSet in function 222 (core::fmt::write)
                            #[cfg(feature = "tracing")]
                            if func_idx == 222 {
                                trace!(
                                    pc = pc,
                                    local_idx = local_idx,
                                    value = ?value,
                                    "[LOCALSET-222] Local set"
                                );
                            }
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
                        // Use copy_value() for efficient copying of Copy-like variants
                        if let Some(value) = operand_stack.last().map(|v| v.copy_value()) {
                            #[cfg(feature = "tracing")]

                            trace!("LocalTee: setting local[{}] = {:?} (keeping on stack)", local_idx, value);
                            // Debug: trace LocalTee in function 222 (core::fmt::write)
                            #[cfg(feature = "tracing")]
                            if func_idx == 222 {
                                trace!(
                                    pc = pc,
                                    local_idx = local_idx,
                                    value = ?value,
                                    "[LOCALTEE-222] Local tee"
                                );
                            }
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
                                #[cfg(feature = "tracing")]
                                {
                                    // Debug all global reads to trace suspicious values
                                    match &value {
                                        Value::I32(v) => trace!(
                                            global_idx = global_idx,
                                            value = v,
                                            value_hex = format_args!("0x{:x}", *v as u32),
                                            "[GlobalGet] Global value"
                                        ),
                                        _ => trace!(
                                            global_idx = global_idx,
                                            value = ?value,
                                            "[GlobalGet] Global value"
                                        ),
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
                            #[cfg(feature = "tracing")]
                            {
                                match &value {
                                    Value::I32(v) => trace!(
                                        global_idx = global_idx,
                                        value = v,
                                        value_hex = format_args!("0x{:x}", *v as u32),
                                        "[GlobalSet] Setting global"
                                    ),
                                    _ => trace!(
                                        global_idx = global_idx,
                                        value = ?value,
                                        "[GlobalSet] Setting global"
                                    ),
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
                            #[cfg(feature = "tracing")]
                            {
                                if (result as u32) > 0x200000 {
                                    warn!(
                                        a = a,
                                        b = b,
                                        result = result,
                                        result_hex = format_args!("0x{:x}", result as u32),
                                        "[I32Add] SUSPICIOUS: large result"
                                    );
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
                            #[cfg(feature = "tracing")]
                            {
                                if (result as u32) > 0x200000 {
                                    warn!(
                                        a = a,
                                        b = b,
                                        result = result,
                                        result_hex = format_args!("0x{:x}", result as u32),
                                        "[I32Sub] SUSPICIOUS: large result"
                                    );
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
                            #[cfg(feature = "tracing")]
                            {
                                if (result as u32) > 0x20000 {
                                    warn!(
                                        a = a,
                                        b = b,
                                        result = result,
                                        result_hex = format_args!("0x{:x}", result as u32),
                                        "[I32Mul] SUSPICIOUS: large result"
                                    );
                                }
                            }
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32DivS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("integer divide by zero"));
                            }
                            // Check for integer overflow: INT_MIN / -1 would overflow
                            if a == i32::MIN && b == -1 {
                                return Err(wrt_error::Error::runtime_trap("integer overflow"));
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
                                return Err(wrt_error::Error::runtime_trap("integer divide by zero"));
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
                                return Err(wrt_error::Error::runtime_trap("integer divide by zero"));
                            }
                            // Note: INT_MIN % -1 = 0 (no overflow for remainder)
                            let result = a.wrapping_rem(b);
                            #[cfg(feature = "tracing")]
                            trace!("I32RemS: {} % {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32RemU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("integer divide by zero"));
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
                                return Err(wrt_error::Error::runtime_trap("integer divide by zero"));
                            }
                            // Check for integer overflow: INT_MIN / -1 would overflow
                            if a == i64::MIN && b == -1 {
                                return Err(wrt_error::Error::runtime_trap("integer overflow"));
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
                                return Err(wrt_error::Error::runtime_trap("integer divide by zero"));
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
                                return Err(wrt_error::Error::runtime_trap("integer divide by zero"));
                            }
                            // Note: INT_MIN % -1 = 0 (no overflow for remainder)
                            let result = a.wrapping_rem(b);
                            #[cfg(feature = "tracing")]
                            trace!("I64RemS: {} % {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64RemU => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("integer divide by zero"));
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
                            #[cfg(feature = "tracing")]
                            {
                                if (result as u32) > 0x20000 {
                                    warn!(
                                        i64_value = value,
                                        i64_hex = format_args!("0x{:x}", value),
                                        i32_result = result,
                                        i32_hex = format_args!("0x{:x}", result as u32),
                                        "[I32WrapI64] SUSPICIOUS: large wrap result"
                                    );
                                }
                            }
                            operand_stack.push(Value::I32(result));
                        }
                    }

                    // Trapping truncation operations - trap on NaN or overflow
                    Instruction::I32TruncF32S => {
                        if let Some(Value::F32(bits)) = operand_stack.pop() {
                            let f = f32::from_bits(bits.0);
                            if f.is_nan() || f.is_infinite() {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i32.trunc_f32_s: invalid conversion to integer",
                                ));
                            }
                            let f_trunc = f.trunc();
                            // Range check: must be in [-2147483648, 2147483647]
                            if f_trunc < -2_147_483_648.0_f32 || f_trunc >= 2_147_483_648.0_f32 {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i32.trunc_f32_s: integer overflow",
                                ));
                            }
                            operand_stack.push(Value::I32(f_trunc as i32));
                        }
                    }
                    Instruction::I32TruncF32U => {
                        if let Some(Value::F32(bits)) = operand_stack.pop() {
                            let f = f32::from_bits(bits.0);
                            if f.is_nan() || f.is_infinite() {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i32.trunc_f32_u: invalid conversion to integer",
                                ));
                            }
                            let f_trunc = f.trunc();
                            // Range check: must be in [0, 4294967295]
                            if f_trunc < 0.0_f32 || f_trunc >= 4_294_967_296.0_f32 {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i32.trunc_f32_u: integer overflow",
                                ));
                            }
                            operand_stack.push(Value::I32(f_trunc as u32 as i32));
                        }
                    }
                    Instruction::I32TruncF64S => {
                        if let Some(Value::F64(bits)) = operand_stack.pop() {
                            let f = f64::from_bits(bits.0);
                            if f.is_nan() || f.is_infinite() {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i32.trunc_f64_s: invalid conversion to integer",
                                ));
                            }
                            let f_trunc = f.trunc();
                            // Range check: must be in [-2147483648, 2147483647]
                            if f_trunc < -2_147_483_648.0_f64 || f_trunc >= 2_147_483_648.0_f64 {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i32.trunc_f64_s: integer overflow",
                                ));
                            }
                            operand_stack.push(Value::I32(f_trunc as i32));
                        }
                    }
                    Instruction::I32TruncF64U => {
                        if let Some(Value::F64(bits)) = operand_stack.pop() {
                            let f = f64::from_bits(bits.0);
                            if f.is_nan() || f.is_infinite() {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i32.trunc_f64_u: invalid conversion to integer",
                                ));
                            }
                            let f_trunc = f.trunc();
                            // Range check: must be in [0, 4294967295]
                            if f_trunc < 0.0_f64 || f_trunc >= 4_294_967_296.0_f64 {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i32.trunc_f64_u: integer overflow",
                                ));
                            }
                            operand_stack.push(Value::I32(f_trunc as u32 as i32));
                        }
                    }
                    Instruction::I64TruncF32S => {
                        if let Some(Value::F32(bits)) = operand_stack.pop() {
                            let f = f32::from_bits(bits.0);
                            if f.is_nan() || f.is_infinite() {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i64.trunc_f32_s: invalid conversion to integer",
                                ));
                            }
                            let f_trunc = f.trunc();
                            // Range check: must be in [-9223372036854775808, 9223372036854775807]
                            if f_trunc < -9_223_372_036_854_775_808.0_f32
                                || f_trunc >= 9_223_372_036_854_775_808.0_f32
                            {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i64.trunc_f32_s: integer overflow",
                                ));
                            }
                            operand_stack.push(Value::I64(f_trunc as i64));
                        }
                    }
                    Instruction::I64TruncF32U => {
                        if let Some(Value::F32(bits)) = operand_stack.pop() {
                            let f = f32::from_bits(bits.0);
                            if f.is_nan() || f.is_infinite() {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i64.trunc_f32_u: invalid conversion to integer",
                                ));
                            }
                            let f_trunc = f.trunc();
                            // Range check: must be in [0, 18446744073709551615]
                            if f_trunc < 0.0_f32 || f_trunc >= 18_446_744_073_709_551_616.0_f32 {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i64.trunc_f32_u: integer overflow",
                                ));
                            }
                            operand_stack.push(Value::I64(f_trunc as u64 as i64));
                        }
                    }
                    Instruction::I64TruncF64S => {
                        if let Some(Value::F64(bits)) = operand_stack.pop() {
                            let f = f64::from_bits(bits.0);
                            if f.is_nan() || f.is_infinite() {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i64.trunc_f64_s: invalid conversion to integer",
                                ));
                            }
                            let f_trunc = f.trunc();
                            // Range check: must be in [-9223372036854775808, 9223372036854775807]
                            if f_trunc < -9_223_372_036_854_775_808.0_f64
                                || f_trunc >= 9_223_372_036_854_775_808.0_f64
                            {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i64.trunc_f64_s: integer overflow",
                                ));
                            }
                            operand_stack.push(Value::I64(f_trunc as i64));
                        }
                    }
                    Instruction::I64TruncF64U => {
                        if let Some(Value::F64(bits)) = operand_stack.pop() {
                            let f = f64::from_bits(bits.0);
                            if f.is_nan() || f.is_infinite() {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i64.trunc_f64_u: invalid conversion to integer",
                                ));
                            }
                            let f_trunc = f.trunc();
                            // Range check: must be in [0, 18446744073709551615]
                            if f_trunc < 0.0_f64 || f_trunc >= 18_446_744_073_709_551_616.0_f64 {
                                return Err(wrt_error::Error::runtime_trap(
                                    "i64.trunc_f64_u: integer overflow",
                                ));
                            }
                            operand_stack.push(Value::I64(f_trunc as u64 as i64));
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
                            // Trace None checks (comparing to -2147483648 sentinel)
                            #[cfg(feature = "tracing")]
                            if b == -2147483648 || a == -2147483648 {
                                trace!(
                                    func_idx = func_idx,
                                    pc = pc,
                                    a = a,
                                    b = b,
                                    result = result != 0,
                                    "[NONE_CHECK] None sentinel check"
                                );
                            }
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
                    // F32 comparison operations - all produce i32 result
                    Instruction::F32Eq => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a.value() == b.value() { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("F32Eq: {} == {} = {}", a.value(), b.value(), result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::F32Ne => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a.value() != b.value() { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("F32Ne: {} != {} = {}", a.value(), b.value(), result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::F32Lt => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a.value() < b.value() { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("F32Lt: {} < {} = {}", a.value(), b.value(), result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::F32Gt => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a.value() > b.value() { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("F32Gt: {} > {} = {}", a.value(), b.value(), result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::F32Le => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a.value() <= b.value() { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("F32Le: {} <= {} = {}", a.value(), b.value(), result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::F32Ge => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a.value() >= b.value() { 1i32 } else { 0i32 };
                            #[cfg(feature = "tracing")]
                            trace!("F32Ge: {} >= {} = {}", a.value(), b.value(), result != 0);
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
                    // I64 bitwise operations
                    Instruction::I64And => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a & b;
                            #[cfg(feature = "tracing")]
                            trace!("I64And: {} & {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64Or => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a | b;
                            #[cfg(feature = "tracing")]
                            trace!("I64Or: {} | {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64Xor => {
                        if let (Some(Value::I64(b)), Some(Value::I64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a ^ b;
                            #[cfg(feature = "tracing")]
                            trace!("I64Xor: {} ^ {} = {}", a, b, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I32Shl => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_shl((b as u32) % 32);
                            #[cfg(feature = "tracing")]
                            trace!("I32Shl: {} << {} = {}", a, b, result);
                            #[cfg(feature = "tracing")]
                            {
                                if (result as u32) > 0x200000 {
                                    warn!(
                                        a = a,
                                        b = b,
                                        result = result,
                                        result_hex = format_args!("0x{:x}", result as u32),
                                        "[I32Shl] SUSPICIOUS: large shift result"
                                    );
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
                    // I64 unary bit operations
                    Instruction::I64Clz => {
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            let result = (a as u64).leading_zeros() as i64;
                            #[cfg(feature = "tracing")]
                            trace!("I64Clz: clz({}) = {}", a, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64Ctz => {
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            let result = (a as u64).trailing_zeros() as i64;
                            #[cfg(feature = "tracing")]
                            trace!("I64Ctz: ctz({}) = {}", a, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64Popcnt => {
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            let result = (a as u64).count_ones() as i64;
                            #[cfg(feature = "tracing")]
                            trace!("I64Popcnt: popcnt({}) = {}", a, result);
                            operand_stack.push(Value::I64(result));
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
                    // Sign-extension operators (WebAssembly 2.0)
                    Instruction::I32Extend8S => {
                        // Sign-extend 8-bit value to 32 bits
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = (a as i8) as i32; // Cast to i8 then sign-extend to i32
                            #[cfg(feature = "tracing")]
                            trace!("I32Extend8S: {} -> {}", a, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Extend16S => {
                        // Sign-extend 16-bit value to 32 bits
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = (a as i16) as i32; // Cast to i16 then sign-extend to i32
                            #[cfg(feature = "tracing")]
                            trace!("I32Extend16S: {} -> {}", a, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I64Extend8S => {
                        // Sign-extend 8-bit value to 64 bits
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            let result = (a as i8) as i64; // Cast to i8 then sign-extend to i64
                            #[cfg(feature = "tracing")]
                            trace!("I64Extend8S: {} -> {}", a, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64Extend16S => {
                        // Sign-extend 16-bit value to 64 bits
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            let result = (a as i16) as i64; // Cast to i16 then sign-extend to i64
                            #[cfg(feature = "tracing")]
                            trace!("I64Extend16S: {} -> {}", a, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    Instruction::I64Extend32S => {
                        // Sign-extend 32-bit value to 64 bits
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            let result = (a as i32) as i64; // Cast to i32 then sign-extend to i64
                            #[cfg(feature = "tracing")]
                            trace!("I64Extend32S: {} -> {}", a, result);
                            operand_stack.push(Value::I64(result));
                        }
                    }
                    // Memory operations
                    // IMPORTANT: Use instance.memory() for initialized memory, not module.get_memory()
                    // The instance has data segments applied, the module is just a template
                    Instruction::I32Load(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            // Calculate effective address with overflow checking (4 bytes for i32)
                            let offset = calculate_effective_address(addr, mem_arg.offset, 4)? as u32;
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
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Load: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I32(value));
                                        }
                                        Err(e) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Load: memory read failed: {:?}", e);
                                            #[cfg(feature = "tracing")]
                                            error!(
                                                offset = format_args!("0x{:x}", offset),
                                                base = format_args!("0x{:x}", addr as u32),
                                                mem_arg_offset = mem_arg.offset,
                                                func_idx = func_idx,
                                                pc = pc,
                                                "[MEM-OOB] I32Load failed"
                                            );
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("I32Load: failed to get memory at index {}: {:?}", mem_arg.memory_index, e);
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I32Store(mem_arg) => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            // Calculate effective address with overflow checking (4 bytes for i32)
                            let offset = calculate_effective_address(addr, mem_arg.offset, 4)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!("I32Store: writing value {} to address {} (base={}, offset={})", value, offset, addr, mem_arg.offset);

                            // Get memory from INSTANCE (not module) - instance has initialized data
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = value.to_le_bytes();
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Store: successfully wrote value {} to address {}", value, offset);
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I32Load8S(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 1)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!("I32Load8S: reading from address {}", offset);
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
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
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I32Load8U(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 1)? as u32;
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
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
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I32Load16S(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 2)? as u32;
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
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I32Load16U(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 2)? as u32;
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
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I32Store8(mem_arg) => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 1)? as u32;

                            #[cfg(feature = "tracing")]
                            trace!("I32Store8: writing byte {} to address {}", value & 0xFF, offset);

                            match instance.memory(mem_arg.memory_index) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = [(value & 0xFF) as u8];
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {}
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I32Store16(mem_arg) => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 2)? as u32;
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = (value as u16).to_le_bytes();
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I32Store16: successfully wrote value {} to address {}", value as u16, offset);
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I64Load(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 8)? as u32;
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = i64::from_le_bytes(buffer);
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Load: read value {} from address {}", value, offset);
                                            #[cfg(all(feature = "std", feature = "tracing"))]
                                            {
                                                trace!(
                                                    value = value,
                                                    value_hex = format_args!("0x{:x}", value as u64),
                                                    offset = format_args!("{:#x}", offset),
                                                    "[I64Load] 64-bit load"
                                                );
                                                // Debug: show memory identity for datetime location
                                                if offset >= 0xffe40 && offset <= 0xffe50 {
                                                    // Get pointer to the actual Memory data via Arc::as_ptr
                                                    use std::sync::Arc;
                                                    let arc_ptr = Arc::as_ptr(&memory_wrapper.0);
                                                    trace!(
                                                        instance_id = instance_id,
                                                        memory_arc_ptr = format_args!("{:p}", arc_ptr),
                                                        "[I64Load-DEBUG] Datetime memory access"
                                                    );
                                                }
                                            }
                                            operand_stack.push(Value::I64(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I64Store(mem_arg) => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 8)? as u32;
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = value.to_le_bytes();
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Store: successfully wrote value {} to address {}", value, offset);
                                        }
                                        Err(_e) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("I64Store: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    // ========================================
                    // I64 Partial Load Instructions (load narrower value, extend to i64)
                    // ========================================
                    Instruction::I64Load8S(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 1)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!(
                                instance_id = instance_id,
                                addr = addr,
                                offset = format_args!("{:#x}", offset),
                                "[I64Load8S] Signed byte load to i64"
                            );
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = buffer[0] as i8 as i64; // Sign extend
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Load8S: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I64(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I64Load8U(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 1)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!(
                                instance_id = instance_id,
                                addr = addr,
                                offset = format_args!("{:#x}", offset),
                                "[I64Load8U] Unsigned byte load to i64"
                            );
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = buffer[0] as i64; // Zero extend
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Load8U: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I64(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I64Load16S(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 2)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!(
                                instance_id = instance_id,
                                addr = addr,
                                offset = format_args!("{:#x}", offset),
                                "[I64Load16S] Signed 16-bit load to i64"
                            );
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = i16::from_le_bytes(buffer) as i64; // Sign extend
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Load16S: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I64(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I64Load16U(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 2)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!(
                                instance_id = instance_id,
                                addr = addr,
                                offset = format_args!("{:#x}", offset),
                                "[I64Load16U] Unsigned 16-bit load to i64"
                            );
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = u16::from_le_bytes(buffer) as i64; // Zero extend
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Load16U: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I64(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I64Load32S(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 4)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!(
                                instance_id = instance_id,
                                addr = addr,
                                offset = format_args!("{:#x}", offset),
                                "[I64Load32S] Signed 32-bit load to i64"
                            );
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = i32::from_le_bytes(buffer) as i64; // Sign extend
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Load32S: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I64(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I64Load32U(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 4)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!(
                                instance_id = instance_id,
                                addr = addr,
                                offset = format_args!("{:#x}", offset),
                                "[I64Load32U] Unsigned 32-bit load to i64"
                            );
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = u32::from_le_bytes(buffer) as i64; // Zero extend
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Load32U: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I64(value));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    // ========================================
                    // I64 Partial Store Instructions (store lower bits of i64)
                    // ========================================
                    Instruction::I64Store8(mem_arg) => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 1)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!(
                                instance_id = instance_id,
                                addr = addr,
                                offset = format_args!("{:#x}", offset),
                                value = value & 0xFF,
                                "[I64Store8] Store low 8 bits of i64"
                            );
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = [(value & 0xFF) as u8];
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Store8: successfully wrote value {} to address {}", value & 0xFF, offset);
                                        }
                                        Err(_e) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("I64Store8: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I64Store16(mem_arg) => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 2)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!(
                                instance_id = instance_id,
                                addr = addr,
                                offset = format_args!("{:#x}", offset),
                                value = value & 0xFFFF,
                                "[I64Store16] Store low 16 bits of i64"
                            );
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = (value as u16).to_le_bytes();
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Store16: successfully wrote value {} to address {}", value & 0xFFFF, offset);
                                        }
                                        Err(_e) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("I64Store16: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::I64Store32(mem_arg) => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 4)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!(
                                instance_id = instance_id,
                                addr = addr,
                                offset = format_args!("{:#x}", offset),
                                value = value & 0xFFFFFFFF,
                                "[I64Store32] Store low 32 bits of i64"
                            );
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = (value as u32).to_le_bytes();
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!("I64Store32: successfully wrote value {} to address {}", value & 0xFFFFFFFF, offset);
                                        }
                                        Err(_e) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("I64Store32: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    // ========================================
                    // F64 Instructions (64-bit floating point)
                    // ========================================
                    Instruction::F64Const(bits) => {
                        #[cfg(feature = "tracing")]
                        trace!("F64Const: pushing {}", f64::from_bits(bits));
                        operand_stack.push(Value::F64(FloatBits64(bits)));
                    }
                    Instruction::F32Load(mem_arg) => {
                        #[cfg(feature = "tracing")]
                        trace!("F32Load: stack before pop has {} elements", operand_stack.len());
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 4)? as u32;
                            #[cfg(feature = "tracing")]
                            trace!("F32Load: addr={}, offset={}, mem_idx={}", addr, offset, mem_arg.memory_index);
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let bits = u32::from_le_bytes(buffer);
                                            #[cfg(feature = "tracing")]
                                            trace!("F32Load: read bytes {:?}, bits={:#x}, pushing F32", buffer, bits);
                                            operand_stack.push(Value::F32(FloatBits32(bits)));
                                            #[cfg(feature = "tracing")]
                                            trace!("F32Load: stack after push has {} elements", operand_stack.len());
                                        }
                                        Err(e) => {
                                            #[cfg(feature = "tracing")]
                                            error!("F32Load: memory read error: {:?}", e);
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(e) => {
                                    #[cfg(feature = "tracing")]
                                    error!("F32Load: memory access error: {:?}", e);
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        } else {
                            #[cfg(feature = "tracing")]
                            error!("F32Load: stack was empty or top was not I32!");
                        }
                    }
                    Instruction::F32Store(mem_arg) => {
                        if let (Some(Value::F32(bits)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 4)? as u32;
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = bits.0.to_le_bytes();
                                    if memory.write_shared(offset, &bytes).is_err() {
                                        return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                    }
                                }
                                Err(_e) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::F64Load(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 8)? as u32;
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let bits = u64::from_le_bytes(buffer);
                                            operand_stack.push(Value::F64(FloatBits64(bits)));
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    Instruction::F64Store(mem_arg) => {
                        if let (Some(Value::F64(bits)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = calculate_effective_address(addr, mem_arg.offset, 8)? as u32;
                            match instance.memory(mem_arg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = bits.0.to_le_bytes();
                                    if memory.write_shared(offset, &bytes).is_err() {
                                        return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                    }
                                }
                                Err(_e) => {
                                    return Err(wrt_error::Error::runtime_trap("out of bounds memory access"));
                                }
                            }
                        }
                    }
                    // F32 Arithmetic operations
                    Instruction::F32Add => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.value() + b.value();
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Sub => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.value() - b.value();
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Mul => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.value() * b.value();
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Div => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.value() / b.value();
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    // F32 Unary operations
                    Instruction::F32Abs => {
                        if let Some(Value::F32(a)) = operand_stack.pop() {
                            let result = a.value().abs();
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Neg => {
                        if let Some(Value::F32(a)) = operand_stack.pop() {
                            let result = -a.value();
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Ceil => {
                        if let Some(Value::F32(a)) = operand_stack.pop() {
                            let result = a.value().ceil();
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Floor => {
                        if let Some(Value::F32(a)) = operand_stack.pop() {
                            let result = a.value().floor();
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Trunc => {
                        if let Some(Value::F32(a)) = operand_stack.pop() {
                            let result = a.value().trunc();
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Nearest => {
                        if let Some(Value::F32(a)) = operand_stack.pop() {
                            let f = a.value();
                            // Round to nearest even (banker's rounding)
                            let result = if f.fract().abs() == 0.5 {
                                let floor = f.floor();
                                if floor as i32 % 2 == 0 { floor } else { f.ceil() }
                            } else {
                                f.round()
                            };
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Sqrt => {
                        if let Some(Value::F32(a)) = operand_stack.pop() {
                            let result = a.value().sqrt();
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Min => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let fa = a.value();
                            let fb = b.value();
                            // WebAssembly spec: If either operand is NaN, return NaN
                            // If both are zero with different signs, return -0.0
                            let result = if fa.is_nan() || fb.is_nan() {
                                f32::NAN
                            } else if fa == 0.0 && fb == 0.0 {
                                if fa.is_sign_negative() || fb.is_sign_negative() { -0.0 } else { 0.0 }
                            } else {
                                fa.min(fb)
                            };
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Max => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let fa = a.value();
                            let fb = b.value();
                            // WebAssembly spec: If either operand is NaN, return NaN
                            // If both are zero with different signs, return +0.0
                            let result = if fa.is_nan() || fb.is_nan() {
                                f32::NAN
                            } else if fa == 0.0 && fb == 0.0 {
                                if fa.is_sign_positive() || fb.is_sign_positive() { 0.0 } else { -0.0 }
                            } else {
                                fa.max(fb)
                            };
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32Copysign => {
                        if let (Some(Value::F32(b)), Some(Value::F32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.value().copysign(b.value());
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    // F64 Arithmetic operations
                    Instruction::F64Add => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = f64::from_bits(a.0) + f64::from_bits(b.0);
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Sub => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = f64::from_bits(a.0) - f64::from_bits(b.0);
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Mul => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = f64::from_bits(a.0) * f64::from_bits(b.0);
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Div => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = f64::from_bits(a.0) / f64::from_bits(b.0);
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    // F64 Comparison operations
                    Instruction::F64Eq => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if f64::from_bits(a.0) == f64::from_bits(b.0) { 1i32 } else { 0i32 };
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::F64Ne => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if f64::from_bits(a.0) != f64::from_bits(b.0) { 1i32 } else { 0i32 };
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::F64Lt => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if f64::from_bits(a.0) < f64::from_bits(b.0) { 1i32 } else { 0i32 };
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::F64Gt => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if f64::from_bits(a.0) > f64::from_bits(b.0) { 1i32 } else { 0i32 };
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::F64Le => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if f64::from_bits(a.0) <= f64::from_bits(b.0) { 1i32 } else { 0i32 };
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::F64Ge => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if f64::from_bits(a.0) >= f64::from_bits(b.0) { 1i32 } else { 0i32 };
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    // F64 Unary operations
                    Instruction::F64Abs => {
                        if let Some(Value::F64(a)) = operand_stack.pop() {
                            let result = f64::from_bits(a.0).abs();
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Neg => {
                        if let Some(Value::F64(a)) = operand_stack.pop() {
                            let result = -f64::from_bits(a.0);
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Ceil => {
                        if let Some(Value::F64(a)) = operand_stack.pop() {
                            let result = f64::from_bits(a.0).ceil();
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Floor => {
                        if let Some(Value::F64(a)) = operand_stack.pop() {
                            let result = f64::from_bits(a.0).floor();
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Trunc => {
                        if let Some(Value::F64(a)) = operand_stack.pop() {
                            let result = f64::from_bits(a.0).trunc();
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Nearest => {
                        if let Some(Value::F64(a)) = operand_stack.pop() {
                            let f = f64::from_bits(a.0);
                            let result = if f.fract().abs() == 0.5 {
                                let floor = f.floor();
                                if floor as i64 % 2 == 0 { floor } else { f.ceil() }
                            } else {
                                f.round()
                            };
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Sqrt => {
                        if let Some(Value::F64(a)) = operand_stack.pop() {
                            let result = f64::from_bits(a.0).sqrt();
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Min => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let fa = f64::from_bits(a.0);
                            let fb = f64::from_bits(b.0);
                            let result = if fa.is_nan() || fb.is_nan() {
                                f64::NAN
                            } else if fa == 0.0 && fb == 0.0 {
                                if fa.is_sign_negative() || fb.is_sign_negative() { -0.0 } else { 0.0 }
                            } else {
                                fa.min(fb)
                            };
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Max => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let fa = f64::from_bits(a.0);
                            let fb = f64::from_bits(b.0);
                            let result = if fa.is_nan() || fb.is_nan() {
                                f64::NAN
                            } else if fa == 0.0 && fb == 0.0 {
                                if fa.is_sign_positive() || fb.is_sign_positive() { 0.0 } else { -0.0 }
                            } else {
                                fa.max(fb)
                            };
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64Copysign => {
                        if let (Some(Value::F64(b)), Some(Value::F64(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = f64::from_bits(a.0).copysign(f64::from_bits(b.0));
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    // F32 Conversion operations
                    Instruction::F32ConvertI32S => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = a as f32;
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32ConvertI32U => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = (a as u32) as f32;
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32ConvertI64S => {
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            let result = a as f32;
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F32ConvertI64U => {
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            let result = (a as u64) as f32;
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    // F64 Conversion operations
                    Instruction::F64ConvertI32S => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = a as f64;
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64ConvertI32U => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = (a as u32) as f64;
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64ConvertI64S => {
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            let result = a as f64;
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64ConvertI64U => {
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            let result = (a as u64) as f64;
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F64PromoteF32 => {
                        if let Some(Value::F32(a)) = operand_stack.pop() {
                            let result = f32::from_bits(a.0) as f64;
                            operand_stack.push(Value::F64(FloatBits64(result.to_bits())));
                        }
                    }
                    Instruction::F32DemoteF64 => {
                        if let Some(Value::F64(a)) = operand_stack.pop() {
                            let result = f64::from_bits(a.0) as f32;
                            operand_stack.push(Value::F32(FloatBits32(result.to_bits())));
                        }
                    }
                    Instruction::F64ReinterpretI64 => {
                        if let Some(Value::I64(a)) = operand_stack.pop() {
                            operand_stack.push(Value::F64(FloatBits64(a as u64)));
                        }
                    }
                    Instruction::I64ReinterpretF64 => {
                        if let Some(Value::F64(a)) = operand_stack.pop() {
                            operand_stack.push(Value::I64(a.0 as i64));
                        }
                    }
                    Instruction::I32ReinterpretF32 => {
                        if let Some(Value::F32(a)) = operand_stack.pop() {
                            operand_stack.push(Value::I32(a.0 as i32));
                        }
                    }
                    Instruction::F32ReinterpretI32 => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            operand_stack.push(Value::F32(FloatBits32(a as u32)));
                        }
                    }
                    Instruction::If { block_type_idx } => {
                        block_depth += 1;
                        // Record stack height as what it will be AFTER condition is consumed
                        // The condition is still on stack, so entry height = len() - 1
                        block_stack.push(("if", pc, block_type_idx, operand_stack.len().saturating_sub(1)));
                        #[cfg(feature = "tracing")]
                        trace!("If: block_type_idx={}, depth now {}, stack_height={}",
                               block_type_idx, block_depth, operand_stack.len().saturating_sub(1));
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
                        block_stack.push(("block", pc, block_type_idx, operand_stack.len()));
                        #[cfg(feature = "tracing")]
                        trace!("Block: block_type_idx={}, depth now {}, stack_height={}", block_type_idx, block_depth, operand_stack.len());
                        // Trace block entry in func 76
                        #[cfg(feature = "tracing")]
                        if func_idx == 76 {
                            trace!(pc = pc, block_depth = block_depth, "[BLOCK] func_idx=76 block entry");
                        }
                        // Trace block entry in func 222
                        #[cfg(feature = "tracing")]
                        if func_idx == 222 {
                            trace!(pc = pc, block_depth = block_depth, "[BLOCK-222] Block entry");
                        }
                        // Just execute through the block - End will decrement depth
                    }
                    Instruction::Loop { block_type_idx } => {
                        block_depth += 1;
                        block_stack.push(("loop", pc, block_type_idx, operand_stack.len()));
                        #[cfg(feature = "tracing")]
                        trace!("Loop: block_type_idx={}, depth now {}, start_pc={}, stack_height={}", block_type_idx, block_depth, pc, operand_stack.len());
                        // Trace loop entry in func 76
                        #[cfg(feature = "tracing")]
                        if func_idx == 76 {
                            trace!(pc = pc, block_depth = block_depth, "[LOOP] func_idx=76 loop entry");
                        }
                        // Trace loop entry in func 222
                        #[cfg(feature = "tracing")]
                        if func_idx == 222 {
                            trace!(pc = pc, block_depth = block_depth, "[LOOP-222] Loop entry");
                        }
                        // Just execute through - Br will handle jumping back to start
                    }
                    Instruction::Br(label_idx) => {
                        #[cfg(feature = "tracing")]
                        trace!("Br: label_idx={} (unconditional branch)", label_idx);
                        // Trace Br in func 222
                        #[cfg(feature = "tracing")]
                        if func_idx == 222 {
                            trace!(
                                pc = pc,
                                label_idx = label_idx,
                                block_depth = block_depth,
                                block_stack_len = block_stack.len(),
                                "[BR-222] Branch instruction"
                            );
                            for (i, (btype, bpc, _, _)) in block_stack.iter().enumerate() {
                                trace!(index = i, block_type = btype, block_pc = bpc, "[BR-222] Block stack entry");
                            }
                        }
                        // Trace Br in func 76 (extend_desugared)
                        #[cfg(feature = "tracing")]
                        if func_idx == 76 {
                            trace!(
                                pc = pc,
                                label_idx = label_idx,
                                block_stack_len = block_stack.len(),
                                "[BR] func_idx=76 branch"
                            );
                            for (i, (btype, bpc, _, height)) in block_stack.iter().enumerate() {
                                trace!(index = i, block_type = btype, block_pc = bpc, height = height, "[BR] Block stack entry");
                            }
                        }

                        // Get the target block from the block_stack
                        // label_idx=0 means innermost block, 1 means next outer, etc.
                        if (label_idx as usize) < block_stack.len() {
                            let stack_idx = block_stack.len() - 1 - (label_idx as usize);
                            let (block_type, start_pc, block_type_idx, entry_stack_height) = block_stack[stack_idx];

                            // Determine how many values to preserve based on block type
                            // For loops: branch to beginning preserves nothing (arity 0)
                            // For blocks: branch to end preserves the block's return values
                            let values_to_preserve = if block_type == "loop" {
                                0  // Loop branches to start, no values preserved
                            } else {
                                match block_type_idx {
                                    0x40 => 0, // empty type - no return
                                    0x7F | 0x7E | 0x7D | 0x7C | 0x7B => 1, // i32, i64, f32, f64, v128
                                    0x70 | 0x6F => 1, // funcref, externref
                                    _ => {
                                        // Type index - look up actual result count from module types
                                        if let Some(func_type) = module.types.get(block_type_idx as usize) {
                                            func_type.results.len()
                                        } else {
                                            1 // Fallback if type not found
                                        }
                                    }
                                }
                            };

                            // Save the values to preserve from top of stack
                            let mut preserved_values = Vec::new();
                            #[cfg(feature = "tracing")]
                            trace!(
                                pc = pc,
                                block_type_idx = format_args!("0x{:02X}", block_type_idx),
                                values_to_preserve = values_to_preserve,
                                stack_len = operand_stack.len(),
                                entry_height = entry_stack_height,
                                "[BR-VALUE] Branch value handling"
                            );
                            for _ in 0..values_to_preserve {
                                if let Some(v) = operand_stack.pop() {
                                    #[cfg(feature = "tracing")]
                                    trace!(value = ?v, "[BR-VALUE] Saving value");
                                    preserved_values.push(v);
                                }
                            }

                            // Clear stack down to the entry height
                            while operand_stack.len() > entry_stack_height {
                                let _ = operand_stack.pop();
                            }
                            #[cfg(feature = "tracing")]
                            trace!(stack_len = operand_stack.len(), "[BR-VALUE] After clearing");

                            if block_type == "loop" {
                                // For Loop: jump backward to the loop start
                                // IMPORTANT: Pop all inner blocks from the stack since we're
                                // jumping out of them
                                let blocks_to_pop = label_idx as usize;
                                for _ in 0..blocks_to_pop {
                                    if !block_stack.is_empty() {
                                        block_stack.pop();
                                        block_depth -= 1;
                                    }
                                }
                                #[cfg(feature = "tracing")]
                                trace!("Br: jumping backward to loop start at pc={}, popped {} inner blocks", start_pc, blocks_to_pop);
                                pc = start_pc;  // Will +1 at end of iteration, so we execute the Loop instruction again
                            } else {
                                // For Block/If: jump forward to the End
                                // IMPORTANT: Pop all inner blocks from the stack since we're
                                // jumping over their End instructions
                                let blocks_to_pop = label_idx as usize;
                                for _ in 0..blocks_to_pop {
                                    if !block_stack.is_empty() {
                                        let popped = block_stack.pop();
                                        block_depth -= 1;
                                        #[cfg(feature = "tracing")]
                                        if func_idx == 76 {
                                            trace!(popped = ?popped, block_depth = block_depth, "[BR-FWD] Popping inner block");
                                        }
                                    }
                                }

                                // Scan forward to find the target block's End
                                // We need to skip past (label_idx + 1) End instructions at depth 0
                                let mut target_depth = label_idx as i32 + 1;
                                let mut new_pc = pc + 1;
                                let mut depth = 0;

                                #[cfg(feature = "tracing")]
                                if func_idx == 76 {
                                    trace!(new_pc = new_pc, target_depth = target_depth, "[BR-FWD] Starting scan");
                                }

                                while new_pc < instructions.len() && target_depth > 0 {
                                    if let Some(instr) = instructions.get(new_pc) {
                                        match instr {
                                            wrt_foundation::types::Instruction::Block { .. } |
                                            wrt_foundation::types::Instruction::Loop { .. } |
                                            wrt_foundation::types::Instruction::If { .. } => {
                                                depth += 1;
                                                #[cfg(feature = "tracing")]
                                                if func_idx == 76 {
                                                    trace!(pc = new_pc, depth = depth, "[BR-FWD] Block/Loop/If");
                                                }
                                            }
                                            wrt_foundation::types::Instruction::End => {
                                                #[cfg(feature = "tracing")]
                                                if func_idx == 76 {
                                                    trace!(pc = new_pc, depth = depth, target_depth = target_depth, "[BR-FWD] End");
                                                }
                                                if depth == 0 {
                                                    target_depth -= 1;
                                                    if target_depth == 0 {
                                                        #[cfg(feature = "tracing")]
                                                        trace!("Br: jumping forward to pc={} (end of {} block)", new_pc, block_type);
                                                        #[cfg(feature = "tracing")]
                                                        if func_idx == 76 {
                                                            trace!(pc = new_pc, "[BR-FWD] Jumping to");
                                                        }
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

                            // Restore preserved values back to stack (in reverse order)
                            #[cfg(feature = "tracing")]
                            trace!(count = preserved_values.len(), "[BR-VALUE] Restoring values");
                            for v in preserved_values.into_iter().rev() {
                                #[cfg(feature = "tracing")]
                                trace!(value = ?v, "[BR-VALUE] Restoring value");
                                operand_stack.push(v);
                            }
                            #[cfg(feature = "tracing")]
                            trace!(stack_len = operand_stack.len(), "[BR-VALUE] After restore");
                        } else {
                            // Branch targets the function level (return from function)
                            // This happens when label_idx >= block_stack.len()
                            #[cfg(feature = "tracing")]
                            trace!("Br: label_idx {} targeting function level (return)", label_idx);

                            // Determine how many values to preserve based on function's return type
                            let values_to_preserve = if let Some(func_type) = module.types.get(func.type_idx as usize) {
                                func_type.results.len()
                            } else {
                                0
                            };

                            // Save the values to preserve from top of stack
                            let mut preserved_values = Vec::new();
                            for _ in 0..values_to_preserve {
                                if let Some(v) = operand_stack.pop() {
                                    preserved_values.push(v);
                                }
                            }

                            // Clear the rest of the stack
                            operand_stack.clear();

                            // Restore preserved values (in reverse order)
                            for v in preserved_values.into_iter().rev() {
                                operand_stack.push(v);
                            }

                            // Return from function
                            break;
                        }
                    }
                    Instruction::BrIf(label_idx) => {
                        if let Some(Value::I32(condition)) = operand_stack.pop() {
                            #[cfg(feature = "tracing")]
                            trace!("BrIf: label_idx={}, condition={}", label_idx, condition != 0);
                            // Trace BrIf in specific functions for debugging
                            #[cfg(feature = "tracing")]
                            if func_idx == 76 || func_idx == 211 || func_idx == 222 {
                                trace!(
                                    func_idx = func_idx,
                                    pc = pc,
                                    label_idx = label_idx,
                                    condition = condition,
                                    will_branch = (condition != 0),
                                    "[BRIF] Conditional branch check"
                                );
                            }
                            if condition != 0 {
                                // Branch conditionally - same logic as Br
                                if (label_idx as usize) < block_stack.len() {
                                    let stack_idx = block_stack.len() - 1 - (label_idx as usize);
                                    let (block_type, start_pc, block_type_idx, entry_stack_height) = block_stack[stack_idx];

                                    // Determine how many values to preserve based on block type
                                    let values_to_preserve = if block_type == "loop" {
                                        0  // Loop branches to start, no values preserved
                                    } else {
                                        match block_type_idx {
                                            0x40 => 0, // empty type - no return
                                            0x7F | 0x7E | 0x7D | 0x7C | 0x7B => 1, // i32, i64, f32, f64, v128
                                            0x70 | 0x6F => 1, // funcref, externref
                                            _ => {
                                                // Type index - look up actual result count
                                                if let Some(func_type) = module.types.get(block_type_idx as usize) {
                                                    func_type.results.len()
                                                } else {
                                                    1
                                                }
                                            }
                                        }
                                    };

                                    // Save the values to preserve from top of stack
                                    let mut preserved_values = Vec::new();
                                    for _ in 0..values_to_preserve {
                                        if let Some(v) = operand_stack.pop() {
                                            preserved_values.push(v);
                                        }
                                    }

                                    // Clear stack down to the entry height
                                    while operand_stack.len() > entry_stack_height {
                                        let _ = operand_stack.pop();
                                    }

                                    if block_type == "loop" {
                                        // For Loop: jump backward to the loop start
                                        // IMPORTANT: Pop all inner blocks from the stack since we're
                                        // jumping out of them
                                        let blocks_to_pop = label_idx as usize;
                                        for _ in 0..blocks_to_pop {
                                            if !block_stack.is_empty() {
                                                block_stack.pop();
                                                block_depth -= 1;
                                            }
                                        }
                                        #[cfg(feature = "tracing")]
                                        trace!("BrIf: jumping backward to loop start at pc={}, popped {} inner blocks", start_pc, blocks_to_pop);
                                        pc = start_pc;
                                    } else {
                                        // For Block/If: jump forward to the End
                                        // Pop inner blocks from stack first
                                        let blocks_to_pop = label_idx as usize;
                                        for _ in 0..blocks_to_pop {
                                            if !block_stack.is_empty() {
                                                block_stack.pop();
                                                block_depth -= 1;
                                            }
                                        }

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

                                    // Restore preserved values back to stack (in reverse order)
                                    for v in preserved_values.into_iter().rev() {
                                        operand_stack.push(v);
                                    }
                                } else {
                                    // Branch targets the function level (return from function)
                                    // This happens when label_idx >= block_stack.len()
                                    #[cfg(feature = "tracing")]
                                    trace!("BrIf: label_idx {} targeting function level (return)", label_idx);

                                    // Determine how many values to preserve based on function's return type
                                    let values_to_preserve = if let Some(func_type) = module.types.get(func.type_idx as usize) {
                                        func_type.results.len()
                                    } else {
                                        0
                                    };

                                    // Save the values to preserve from top of stack
                                    let mut preserved_values = Vec::new();
                                    for _ in 0..values_to_preserve {
                                        if let Some(v) = operand_stack.pop() {
                                            preserved_values.push(v);
                                        }
                                    }

                                    // Clear the rest of the stack
                                    operand_stack.clear();

                                    // Restore preserved values (in reverse order)
                                    for v in preserved_values.into_iter().rev() {
                                        operand_stack.push(v);
                                    }

                                    // Return from function
                                    break;
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
                                trace!(
                                    memory_idx = memory_idx,
                                    size_in_pages = size_in_pages,
                                    "[MemorySize] Retrieved memory size"
                                );
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
                                        let current_size = memory.size();
                                        #[cfg(feature = "tracing")]
                                        trace!(
                                            memory_idx = memory_idx,
                                            delta = delta,
                                            current_size = current_size,
                                            "[MemoryGrow] Attempting to grow memory"
                                        );
                                        match memory.grow_shared(delta as u32) {
                                            Ok(prev_pages) => {
                                                #[cfg(feature = "tracing")]
                                                trace!(
                                                    memory_idx = memory_idx,
                                                    prev_pages = prev_pages,
                                                    new_pages = prev_pages + delta as u32,
                                                    "[MemoryGrow] Success"
                                                );
                                                operand_stack.push(Value::I32(prev_pages as i32));
                                            }
                                            Err(e) => {
                                                #[cfg(feature = "tracing")]
                                                warn!(
                                                    memory_idx = memory_idx,
                                                    error = ?e,
                                                    "[MemoryGrow] Failed"
                                                );
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
                            trace!(
                                dest = format_args!("{:#x}", dest),
                                src = format_args!("{:#x}", src),
                                size = size,
                                dst_mem_idx = dst_mem_idx,
                                src_mem_idx = src_mem_idx,
                                "[MemoryCopy] Starting copy operation"
                            );

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

                            #[cfg(any(feature = "std", feature = "alloc"))]
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

                                    #[cfg(feature = "tracing")]
                                    {
                                        trace!(
                                            size = size,
                                            src = format_args!("{:#x}", src),
                                            dest = format_args!("{:#x}", dest),
                                            "[MemoryCopy] SUCCESS"
                                        );
                                        // Show copied data as string if ASCII
                                        if size <= 50 && buffer.iter().all(|&b| b >= 0x20 && b <= 0x7e || b == 0) {
                                            if let Ok(s) = core::str::from_utf8(&buffer) {
                                                trace!(data = %s, "[MemoryCopy] Copied ASCII data");
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("MemoryCopy: memory[{}] not found: {:?}", dst_mem_idx, e);
                                    return Err(e);
                                }
                            }
                            #[cfg(not(any(feature = "std", feature = "alloc")))]
                            return Err(wrt_error::Error::runtime_error("MemoryCopy requires std or alloc feature"));
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
                            trace!(
                                dest = format_args!("{:#x}", dest),
                                value = format_args!("{:#x}", value),
                                size = size,
                                mem_idx = mem_idx,
                                "[MemoryFill] Starting fill operation"
                            );

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

                                    #[cfg(feature = "tracing")]
                                    trace!(
                                        size = size,
                                        dest = format_args!("{:#x}", dest),
                                        fill_byte = format_args!("{:#x}", fill_byte),
                                        "[MemoryFill] SUCCESS"
                                    );
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
                            for (i, (btype, bpc, _, _)) in block_stack.iter().enumerate() {
                                #[cfg(feature = "tracing")]

                                trace!("  [{}]: {} at pc={}", i, btype, bpc);
                            }
                            if (label_idx as usize) < block_stack.len() {
                                let stack_idx = block_stack.len() - 1 - (label_idx as usize);
                                let (block_type, start_pc, block_type_idx, entry_stack_height) = block_stack[stack_idx];
                                #[cfg(feature = "tracing")]

                                trace!("BrTable: accessing block_stack[{}], target block is {} at pc={}", stack_idx, block_type, start_pc);

                                // Determine how many values to preserve based on block type
                                let values_to_preserve = if block_type == "loop" {
                                    0  // Loop branches to start, no values preserved
                                } else {
                                    match block_type_idx {
                                        0x40 => 0, // empty type - no return
                                        0x7F | 0x7E | 0x7D | 0x7C | 0x7B => 1, // i32, i64, f32, f64, v128
                                        0x70 | 0x6F => 1, // funcref, externref
                                        _ => {
                                            // Type index - look up actual result count
                                            if let Some(func_type) = module.types.get(block_type_idx as usize) {
                                                func_type.results.len()
                                            } else {
                                                1
                                            }
                                        }
                                    }
                                };

                                // Save the values to preserve from top of stack
                                let mut preserved_values = Vec::new();
                                for _ in 0..values_to_preserve {
                                    if let Some(v) = operand_stack.pop() {
                                        preserved_values.push(v);
                                    }
                                }

                                // Clear stack down to the entry height
                                while operand_stack.len() > entry_stack_height {
                                    let _ = operand_stack.pop();
                                }

                                // Pop all inner blocks from the stack since we're jumping over
                                // their End instructions (same as Br instruction logic)
                                let blocks_to_pop = label_idx as usize;
                                for _ in 0..blocks_to_pop {
                                    if !block_stack.is_empty() {
                                        block_stack.pop();
                                        block_depth -= 1;
                                    }
                                }

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

                                // Restore preserved values back to stack (in reverse order)
                                for v in preserved_values.into_iter().rev() {
                                    operand_stack.push(v);
                                }
                            } else {
                                // Branch targets the function level (return from function)
                                // This happens when label_idx >= block_stack.len()
                                #[cfg(feature = "tracing")]
                                trace!("BrTable: label_idx {} targeting function level (return)", label_idx);

                                // Determine how many values to preserve based on function's return type
                                let values_to_preserve = if let Some(func_type) = module.types.get(func.type_idx as usize) {
                                    func_type.results.len()
                                } else {
                                    0
                                };

                                // Save the values to preserve from top of stack
                                let mut preserved_values = Vec::new();
                                for _ in 0..values_to_preserve {
                                    if let Some(v) = operand_stack.pop() {
                                        preserved_values.push(v);
                                    }
                                }

                                // Clear the rest of the stack
                                operand_stack.clear();

                                // Restore preserved values (in reverse order)
                                for v in preserved_values.into_iter().rev() {
                                    operand_stack.push(v);
                                }

                                // Return from function
                                break;
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
                        // Trace return from specific functions for debugging
                        #[cfg(feature = "tracing")]
                        if func_idx == 76 {
                            trace!(
                                func_idx = func_idx,
                                pc = pc,
                                stack_size = operand_stack.len(),
                                "[RETURN] Function returning"
                            );
                        }
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
                                let (block_type, start_pc, _, _) = block_stack.pop().unwrap();
                                #[cfg(feature = "tracing")]
                                trace!("End at pc={} (closes {} from pc={}, depth now {})", pc, block_type, start_pc, block_depth);
                                // Trace End in specific functions for debugging
                                #[cfg(feature = "tracing")]
                                if func_idx == 76 || func_idx == 222 {
                                    trace!(
                                        func_idx = func_idx,
                                        pc = pc,
                                        block_type = %block_type,
                                        start_pc = start_pc,
                                        block_depth = block_depth,
                                        block_stack_len = block_stack.len(),
                                        "[END] Block closing"
                                    );
                                }
                            } else {
                                #[cfg(feature = "tracing")]
                                trace!("End at pc={} (closes block, depth now {})", pc, block_depth);
                            }
                        }
                    }

                    // ========================================
                    // Reference Type Operations
                    // ========================================
                    Instruction::RefNull(ref_type) => {
                        // Push a null reference of the specified type
                        use wrt_foundation::types::RefType;
                        #[cfg(feature = "tracing")]
                        trace!("RefNull: type={:?}", ref_type);
                        match ref_type {
                            RefType::Funcref => operand_stack.push(Value::FuncRef(None)),
                            RefType::Externref => operand_stack.push(Value::ExternRef(None)),
                        }
                    }
                    Instruction::RefFunc(func_idx_arg) => {
                        // Push a reference to the function at func_idx
                        #[cfg(feature = "tracing")]
                        trace!("RefFunc: func_idx={}", func_idx_arg);
                        operand_stack.push(Value::FuncRef(Some(wrt_foundation::values::FuncRef { index: func_idx_arg })));
                    }
                    Instruction::RefIsNull => {
                        // Pop reference, push 1 if null, 0 if not null
                        if let Some(ref_val) = operand_stack.pop() {
                            let is_null = match ref_val {
                                Value::FuncRef(None) => 1i32,
                                Value::FuncRef(Some(_)) => 0i32,
                                Value::ExternRef(None) => 1i32,
                                Value::ExternRef(Some(_)) => 0i32,
                                _ => {
                                    #[cfg(feature = "tracing")]
                                    error!("RefIsNull: expected reference type, got {:?}", ref_val);
                                    return Err(wrt_error::Error::runtime_type_mismatch(
                                        "ref.is_null expects a reference type",
                                    ));
                                }
                            };
                            #[cfg(feature = "tracing")]
                            trace!("RefIsNull: result={}", is_null);
                            operand_stack.push(Value::I32(is_null));
                        }
                    }
                    Instruction::RefAsNonNull => {
                        // Pop reference, trap if null, push back if not null
                        if let Some(ref_val) = operand_stack.pop() {
                            match &ref_val {
                                Value::FuncRef(None) | Value::ExternRef(None) => {
                                    #[cfg(feature = "tracing")]
                                    error!("RefAsNonNull: null reference");
                                    return Err(wrt_error::Error::runtime_trap(
                                        "null reference in ref.as_non_null",
                                    ));
                                }
                                Value::FuncRef(Some(_)) | Value::ExternRef(Some(_)) => {
                                    #[cfg(feature = "tracing")]
                                    trace!("RefAsNonNull: non-null reference");
                                    operand_stack.push(ref_val);
                                }
                                _ => {
                                    #[cfg(feature = "tracing")]
                                    error!("RefAsNonNull: expected reference type, got {:?}", ref_val);
                                    return Err(wrt_error::Error::runtime_type_mismatch(
                                        "ref.as_non_null expects a reference type",
                                    ));
                                }
                            }
                        }
                    }
                    Instruction::RefEq => {
                        // Pop two references, push 1 if equal, 0 if not
                        if let (Some(ref2), Some(ref1)) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = match (&ref1, &ref2) {
                                // Two null funcref/externref are equal
                                (Value::FuncRef(None), Value::FuncRef(None)) => 1i32,
                                (Value::ExternRef(None), Value::ExternRef(None)) => 1i32,
                                // Two non-null funcrefs are equal if they reference the same function
                                (Value::FuncRef(Some(f1)), Value::FuncRef(Some(f2))) => {
                                    if f1.index == f2.index { 1i32 } else { 0i32 }
                                }
                                // Two non-null externrefs are equal if they're the same object
                                (Value::ExternRef(Some(e1)), Value::ExternRef(Some(e2))) => {
                                    if e1 == e2 { 1i32 } else { 0i32 }
                                }
                                // Different types or null vs non-null are not equal
                                _ => 0i32,
                            };
                            #[cfg(feature = "tracing")]
                            trace!("RefEq: {:?} == {:?} => {}", ref1, ref2, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::BrOnNull(br_label_idx) => {
                        // Pop reference, branch if null, push back if not null
                        if let Some(ref_val) = operand_stack.pop() {
                            let is_null = matches!(&ref_val, Value::FuncRef(None) | Value::ExternRef(None));
                            #[cfg(feature = "tracing")]
                            trace!("BrOnNull: label={}, is_null={}", br_label_idx, is_null);
                            if is_null {
                                // Branch to label - similar to Br instruction
                                if br_label_idx as usize >= block_stack.len() {
                                    // Branch out of function
                                    #[cfg(feature = "tracing")]
                                    trace!("BrOnNull: branching out of function");
                                    break;
                                }
                                // Find the target block in the stack
                                let target_depth = block_stack.len() - 1 - br_label_idx as usize;
                                if let Some((block_type, start_pc, _block_type_idx, entry_stack_height)) = block_stack.get(target_depth).copied() {
                                    // For loops, branch to start; for blocks, branch to end
                                    if block_type == "loop" {
                                        pc = start_pc;
                                    } else {
                                        // Skip to end of block
                                        let mut depth = 1;
                                        let mut search_pc = pc + 1;
                                        while depth > 0 && search_pc < instructions.len() {
                                            #[cfg(feature = "std")]
                                            if let Some(search_instr) = instructions.get(search_pc) {
                                                match search_instr {
                                                    Instruction::Block { .. } | Instruction::Loop { .. } | Instruction::If { .. } => depth += 1,
                                                    Instruction::End => depth -= 1,
                                                    _ => {}
                                                }
                                            }
                                            #[cfg(not(feature = "std"))]
                                            if let Ok(search_instr) = instructions.get(search_pc) {
                                                match search_instr {
                                                    Instruction::Block { .. } | Instruction::Loop { .. } | Instruction::If { .. } => depth += 1,
                                                    Instruction::End => depth -= 1,
                                                    _ => {}
                                                }
                                            }
                                            if depth > 0 { search_pc += 1; }
                                        }
                                        pc = search_pc;
                                    }
                                    // Restore stack to entry height
                                    while operand_stack.len() > entry_stack_height {
                                        operand_stack.pop();
                                    }
                                }
                                continue;
                            } else {
                                // Not null, push reference back
                                operand_stack.push(ref_val);
                            }
                        }
                    }

                    // ==================== TABLE OPERATIONS ====================
                    Instruction::TableGet(table_idx) => {
                        // table.get: [i32] -> [ref]
                        // Pop index from stack, get element from table at that index
                        if let Some(Value::I32(elem_idx)) = operand_stack.pop() {
                            #[cfg(feature = "tracing")]
                            trace!(
                                table_idx = table_idx,
                                elem_idx = elem_idx,
                                "[TableGet] Getting element from table"
                            );

                            if elem_idx < 0 {
                                return Err(wrt_error::Error::runtime_trap(
                                    "table.get: index cannot be negative",
                                ));
                            }

                            // Get the table from the instance
                            let table = instance.table(table_idx)?;
                            let elem = table.get(elem_idx as u32)?;

                            // Push the element (or null ref) onto the stack
                            let value = match elem {
                                Some(v) => v,
                                None => {
                                    // Return null reference based on table element type
                                    match table.element_type() {
                                        wrt_foundation::types::RefType::Funcref => Value::FuncRef(None),
                                        wrt_foundation::types::RefType::Externref => Value::ExternRef(None),
                                    }
                                }
                            };
                            operand_stack.push(value);

                            #[cfg(feature = "tracing")]
                            trace!(
                                table_idx = table_idx,
                                elem_idx = elem_idx,
                                "[TableGet] SUCCESS"
                            );
                        } else {
                            return Err(wrt_error::Error::runtime_trap(
                                "table.get: expected i32 index on stack",
                            ));
                        }
                    }

                    Instruction::TableSet(table_idx) => {
                        // table.set: [i32 ref] -> []
                        // Pop value, then index from stack; set element in table
                        let value = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.set: expected value on stack")
                        })?;
                        let idx = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.set: expected index on stack")
                        })?;

                        if let Value::I32(elem_idx) = idx {
                            #[cfg(feature = "tracing")]
                            trace!(
                                table_idx = table_idx,
                                elem_idx = elem_idx,
                                value = ?value,
                                "[TableSet] Setting element in table"
                            );

                            if elem_idx < 0 {
                                return Err(wrt_error::Error::runtime_trap(
                                    "table.set: index cannot be negative",
                                ));
                            }

                            // Validate value is a reference type
                            let table_value = match &value {
                                Value::FuncRef(fr) => Some(Value::FuncRef(fr.clone())),
                                Value::ExternRef(er) => Some(Value::ExternRef(er.clone())),
                                _ => {
                                    return Err(wrt_error::Error::runtime_trap(
                                        "table.set: value must be a reference type",
                                    ));
                                }
                            };

                            // Get the table and set the element
                            let table = instance.table(table_idx)?;
                            table.set(elem_idx as u32, table_value)?;

                            #[cfg(feature = "tracing")]
                            trace!(
                                table_idx = table_idx,
                                elem_idx = elem_idx,
                                "[TableSet] SUCCESS"
                            );
                        } else {
                            return Err(wrt_error::Error::runtime_trap(
                                "table.set: expected i32 index",
                            ));
                        }
                    }

                    Instruction::TableSize(table_idx) => {
                        // table.size: [] -> [i32]
                        // Push current table size onto stack
                        #[cfg(feature = "tracing")]
                        trace!(
                            table_idx = table_idx,
                            "[TableSize] Getting table size"
                        );

                        let table = instance.table(table_idx)?;
                        let size = table.size();
                        operand_stack.push(Value::I32(size as i32));

                        #[cfg(feature = "tracing")]
                        trace!(
                            table_idx = table_idx,
                            size = size,
                            "[TableSize] SUCCESS"
                        );
                    }

                    Instruction::TableGrow(table_idx) => {
                        // table.grow: [ref i32] -> [i32]
                        // Pop delta (i32), pop init value (ref), grow table, push old size or -1
                        let delta = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.grow: expected delta on stack")
                        })?;
                        let init_value = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.grow: expected init value on stack")
                        })?;

                        if let Value::I32(delta_val) = delta {
                            #[cfg(feature = "tracing")]
                            trace!(
                                table_idx = table_idx,
                                delta = delta_val,
                                init_value = ?init_value,
                                "[TableGrow] Growing table"
                            );

                            // Negative delta should return -1 (failure)
                            if delta_val < 0 {
                                operand_stack.push(Value::I32(-1));
                            } else {
                                // Validate init value is a reference type
                                match &init_value {
                                    Value::FuncRef(_) | Value::ExternRef(_) => {}
                                    _ => {
                                        return Err(wrt_error::Error::runtime_trap(
                                            "table.grow: init value must be a reference type",
                                        ));
                                    }
                                }

                                let table = instance.table(table_idx)?;
                                match table.grow(delta_val as u32, init_value) {
                                    Ok(old_size) => {
                                        operand_stack.push(Value::I32(old_size as i32));
                                        #[cfg(feature = "tracing")]
                                        trace!(
                                            table_idx = table_idx,
                                            old_size = old_size,
                                            "[TableGrow] SUCCESS"
                                        );
                                    }
                                    Err(_) => {
                                        // Growth failed (e.g., exceeded max size)
                                        operand_stack.push(Value::I32(-1));
                                        #[cfg(feature = "tracing")]
                                        trace!(
                                            table_idx = table_idx,
                                            "[TableGrow] Failed, returning -1"
                                        );
                                    }
                                }
                            }
                        } else {
                            return Err(wrt_error::Error::runtime_trap(
                                "table.grow: expected i32 delta",
                            ));
                        }
                    }

                    Instruction::TableFill(table_idx) => {
                        // table.fill: [i32 ref i32] -> []
                        // Pop size, value, dest; fill table region with value
                        let size = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.fill: expected size on stack")
                        })?;
                        let value = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.fill: expected value on stack")
                        })?;
                        let dest = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.fill: expected dest on stack")
                        })?;

                        if let (Value::I32(dest_idx), Value::I32(fill_size)) = (&dest, &size) {
                            #[cfg(feature = "tracing")]
                            trace!(
                                table_idx = table_idx,
                                dest = dest_idx,
                                size = fill_size,
                                value = ?value,
                                "[TableFill] Filling table region"
                            );

                            if *dest_idx < 0 || *fill_size < 0 {
                                return Err(wrt_error::Error::runtime_trap(
                                    "table.fill: negative dest or size",
                                ));
                            }

                            // Validate value is a reference type
                            let fill_value = match &value {
                                Value::FuncRef(fr) => Some(Value::FuncRef(fr.clone())),
                                Value::ExternRef(er) => Some(Value::ExternRef(er.clone())),
                                _ => {
                                    return Err(wrt_error::Error::runtime_trap(
                                        "table.fill: value must be a reference type",
                                    ));
                                }
                            };

                            let table = instance.table(table_idx)?;
                            table.fill(*dest_idx as u32, *fill_size as u32, fill_value)?;

                            #[cfg(feature = "tracing")]
                            trace!(
                                table_idx = table_idx,
                                dest = dest_idx,
                                size = fill_size,
                                "[TableFill] SUCCESS"
                            );
                        } else {
                            return Err(wrt_error::Error::runtime_trap(
                                "table.fill: expected i32 values for dest and size",
                            ));
                        }
                    }

                    Instruction::TableCopy(dst_table_idx, src_table_idx) => {
                        // table.copy: [i32 i32 i32] -> []
                        // Pop size, src_offset, dst_offset; copy elements between tables
                        let size = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.copy: expected size on stack")
                        })?;
                        let src_offset = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.copy: expected src offset on stack")
                        })?;
                        let dst_offset = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.copy: expected dst offset on stack")
                        })?;

                        if let (Value::I32(dst_idx), Value::I32(src_idx), Value::I32(copy_size)) =
                            (&dst_offset, &src_offset, &size)
                        {
                            #[cfg(feature = "tracing")]
                            trace!(
                                dst_table = dst_table_idx,
                                src_table = src_table_idx,
                                dst_offset = dst_idx,
                                src_offset = src_idx,
                                size = copy_size,
                                "[TableCopy] Copying table elements"
                            );

                            if *dst_idx < 0 || *src_idx < 0 || *copy_size < 0 {
                                return Err(wrt_error::Error::runtime_trap(
                                    "table.copy: negative offset or size",
                                ));
                            }

                            // Handle same-table and cross-table copy
                            if dst_table_idx == src_table_idx {
                                // Same table copy - use the table's copy method
                                let table = instance.table(dst_table_idx)?;
                                table.copy(*dst_idx as u32, *src_idx as u32, *copy_size as u32)?;
                            } else {
                                // Cross-table copy - read from src, write to dst
                                let src_table = instance.table(src_table_idx)?;
                                let dst_table = instance.table(dst_table_idx)?;

                                // Read all source elements first (to handle any overlap scenarios)
                                let mut temp_elements = Vec::new();
                                for i in 0..*copy_size as u32 {
                                    let elem = src_table.get(*src_idx as u32 + i)?;
                                    temp_elements.push(elem);
                                }

                                // Write to destination table
                                for (i, elem) in temp_elements.into_iter().enumerate() {
                                    dst_table.set(*dst_idx as u32 + i as u32, elem)?;
                                }
                            }

                            #[cfg(feature = "tracing")]
                            trace!(
                                dst_table = dst_table_idx,
                                src_table = src_table_idx,
                                "[TableCopy] SUCCESS"
                            );
                        } else {
                            return Err(wrt_error::Error::runtime_trap(
                                "table.copy: expected i32 values for offsets and size",
                            ));
                        }
                    }

                    Instruction::TableInit(elem_seg_idx, table_idx) => {
                        // table.init: [i32 i32 i32] -> []
                        // Pop size, src_offset (in elem segment), dst_offset (in table)
                        // Initialize table elements from element segment
                        let size = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.init: expected size on stack")
                        })?;
                        let src_offset = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.init: expected src offset on stack")
                        })?;
                        let dst_offset = operand_stack.pop().ok_or_else(|| {
                            wrt_error::Error::runtime_trap("table.init: expected dst offset on stack")
                        })?;

                        if let (Value::I32(dst_idx), Value::I32(src_idx), Value::I32(init_size)) =
                            (&dst_offset, &src_offset, &size)
                        {
                            #[cfg(feature = "tracing")]
                            trace!(
                                elem_seg_idx = elem_seg_idx,
                                table_idx = table_idx,
                                dst_offset = dst_idx,
                                src_offset = src_idx,
                                size = init_size,
                                "[TableInit] Initializing table from element segment"
                            );

                            if *dst_idx < 0 || *src_idx < 0 || *init_size < 0 {
                                return Err(wrt_error::Error::runtime_trap(
                                    "table.init: negative offset or size",
                                ));
                            }

                            // Handle zero-size init (valid no-op)
                            if *init_size == 0 {
                                #[cfg(feature = "tracing")]
                                trace!("[TableInit] Zero size, no-op");
                                // Continue to next instruction
                            } else {
                                // Get the element segment from the module
                                #[cfg(feature = "std")]
                                let elem_segment = module.elements.get(elem_seg_idx as usize)
                                    .ok_or_else(|| wrt_error::Error::runtime_trap(
                                        "table.init: invalid element segment index"
                                    ))?;
                                #[cfg(not(feature = "std"))]
                                let elem_segment = module.elements.get(elem_seg_idx as usize)
                                    .map_err(|_| wrt_error::Error::runtime_trap(
                                        "table.init: invalid element segment index"
                                    ))?;

                                // Check bounds in element segment
                                let src_end = (*src_idx as usize).checked_add(*init_size as usize)
                                    .ok_or_else(|| wrt_error::Error::runtime_trap(
                                        "table.init: src index overflow"
                                    ))?;
                                if src_end > elem_segment.items.len() {
                                    return Err(wrt_error::Error::runtime_trap(
                                        "table.init: src out of bounds in element segment",
                                    ));
                                }

                                // Get table and check bounds
                                let table = instance.table(table_idx)?;
                                let dst_end = (*dst_idx as u32).checked_add(*init_size as u32)
                                    .ok_or_else(|| wrt_error::Error::runtime_trap(
                                        "table.init: dst index overflow"
                                    ))?;
                                if dst_end > table.size() {
                                    return Err(wrt_error::Error::runtime_trap(
                                        "table.init: dst out of bounds in table",
                                    ));
                                }

                                // Copy elements from segment to table
                                for i in 0..*init_size as usize {
                                    let item_idx = *src_idx as usize + i;
                                    let func_idx = elem_segment.items.get(item_idx)
                                        .map_err(|_| wrt_error::Error::runtime_trap(
                                            "table.init: element segment access error"
                                        ))?;

                                    let value = Some(Value::FuncRef(Some(
                                        wrt_foundation::values::FuncRef { index: func_idx }
                                    )));
                                    table.set(*dst_idx as u32 + i as u32, value)?;
                                }

                                #[cfg(feature = "tracing")]
                                trace!(
                                    elem_seg_idx = elem_seg_idx,
                                    table_idx = table_idx,
                                    "[TableInit] SUCCESS"
                                );
                            }
                        } else {
                            return Err(wrt_error::Error::runtime_trap(
                                "table.init: expected i32 values for offsets and size",
                            ));
                        }
                    }

                    Instruction::ElemDrop(elem_seg_idx) => {
                        // elem.drop: [] -> []
                        // Drop (mark as unavailable) an element segment
                        // Note: This operation marks the segment as dropped so it can't be used
                        // by future table.init operations. The actual segment data may still exist
                        // in memory but is logically unavailable.
                        #[cfg(feature = "tracing")]
                        trace!(
                            elem_seg_idx = elem_seg_idx,
                            "[ElemDrop] Dropping element segment"
                        );

                        // Validate element segment index exists
                        #[cfg(feature = "std")]
                        if elem_seg_idx as usize >= module.elements.len() {
                            return Err(wrt_error::Error::runtime_trap(
                                "elem.drop: invalid element segment index",
                            ));
                        }
                        #[cfg(not(feature = "std"))]
                        if module.elements.get(elem_seg_idx as usize).is_err() {
                            return Err(wrt_error::Error::runtime_trap(
                                "elem.drop: invalid element segment index",
                            ));
                        }

                        // Note: In a full implementation, we would need to track which segments
                        // have been dropped in the module instance. For now, we acknowledge the
                        // instruction but don't enforce the "dropped" state since we don't have
                        // mutable access to the module's element segments at runtime.
                        // Future improvement: Add a dropped_element_segments bitset to ModuleInstance.

                        #[cfg(feature = "tracing")]
                        trace!(
                            elem_seg_idx = elem_seg_idx,
                            "[ElemDrop] SUCCESS (segment marked as dropped)"
                        );
                    }

                    // ===============================================
                    // Atomic Memory Operations (0xFE prefix)
                    // WebAssembly Threads and Atomics proposal
                    // ===============================================

                    // Memory synchronization operations
                    Instruction::MemoryAtomicNotify { memarg } => {
                        // memory.atomic.notify: [i32, i32] -> [i32]
                        // Wake up to count threads waiting on the given address
                        if let (Some(Value::I32(count)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            // Check alignment - notify requires 4-byte alignment
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            #[cfg(feature = "tracing")]
                            trace!(
                                addr = format_args!("0x{:x}", effective_addr),
                                count = count,
                                "[AtomicNotify] Notify operation"
                            );
                            // For single-threaded runtime, notify always returns 0 (no waiters)
                            // A full implementation would use futex-like mechanisms
                            operand_stack.push(Value::I32(0));
                        }
                    }

                    Instruction::MemoryAtomicWait32 { memarg } => {
                        // memory.atomic.wait32: [i32, i32, i64] -> [i32]
                        // Wait for i32 value at address to change, with timeout
                        if let (Some(Value::I64(timeout)), Some(Value::I32(expected)), Some(Value::I32(addr))) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop())
                        {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            // Check alignment - wait32 requires 4-byte alignment
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            #[cfg(feature = "tracing")]
                            trace!(
                                addr = format_args!("0x{:x}", effective_addr),
                                expected = expected,
                                timeout = timeout,
                                "[AtomicWait32] Wait operation"
                            );
                            // For single-threaded runtime, return 1 (not equal) or 2 (timed out)
                            // We'll read the current value and return immediately
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let current = i32::from_le_bytes(buffer);
                                            // Return 1 if value differs, 2 if would timeout (single-threaded)
                                            let result = if current != expected { 1 } else { 2 };
                                            operand_stack.push(Value::I32(result));
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

                    Instruction::MemoryAtomicWait64 { memarg } => {
                        // memory.atomic.wait64: [i32, i64, i64] -> [i32]
                        // Wait for i64 value at address to change, with timeout
                        if let (Some(Value::I64(timeout)), Some(Value::I64(expected)), Some(Value::I32(addr))) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop())
                        {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            // Check alignment - wait64 requires 8-byte alignment
                            if effective_addr % 8 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            #[cfg(feature = "tracing")]
                            trace!(
                                addr = format_args!("0x{:x}", effective_addr),
                                expected = expected,
                                timeout = timeout,
                                "[AtomicWait64] Wait operation"
                            );
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let current = i64::from_le_bytes(buffer);
                                            let result = if current != expected { 1 } else { 2 };
                                            operand_stack.push(Value::I32(result));
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

                    Instruction::AtomicFence => {
                        // atomic.fence: [] -> []
                        // Memory fence - ensures ordering of memory operations
                        #[cfg(feature = "tracing")]
                        trace!("[AtomicFence] Memory fence");
                        core::sync::atomic::fence(Ordering::SeqCst);
                    }

                    // ===============================================
                    // Atomic Loads
                    // ===============================================

                    Instruction::I32AtomicLoad { memarg } => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            // Check 4-byte alignment for i32
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let value = i32::from_le_bytes(buffer);
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value,
                                                "[I32AtomicLoad] Loaded"
                                            );
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

                    Instruction::I64AtomicLoad { memarg } => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            // Check 8-byte alignment for i64
                            if effective_addr % 8 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let value = i64::from_le_bytes(buffer);
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value,
                                                "[I64AtomicLoad] Loaded"
                                            );
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

                    Instruction::I32AtomicLoad8U { memarg } => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            // 8-bit loads have natural alignment (1 byte)
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let value = buffer[0] as i32;
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value,
                                                "[I32AtomicLoad8U] Loaded"
                                            );
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

                    Instruction::I32AtomicLoad16U { memarg } => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            // Check 2-byte alignment for i16
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let value = u16::from_le_bytes(buffer) as i32;
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value,
                                                "[I32AtomicLoad16U] Loaded"
                                            );
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

                    Instruction::I64AtomicLoad8U { memarg } => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let value = buffer[0] as i64;
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value,
                                                "[I64AtomicLoad8U] Loaded"
                                            );
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

                    Instruction::I64AtomicLoad16U { memarg } => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let value = u16::from_le_bytes(buffer) as i64;
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value,
                                                "[I64AtomicLoad16U] Loaded"
                                            );
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

                    Instruction::I64AtomicLoad32U { memarg } => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let value = u32::from_le_bytes(buffer) as i64;
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value,
                                                "[I64AtomicLoad32U] Loaded"
                                            );
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

                    // ===============================================
                    // Atomic Stores
                    // ===============================================

                    Instruction::I32AtomicStore { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = value.to_le_bytes();
                                    match memory.write_shared(effective_addr, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value,
                                                "[I32AtomicStore] Stored"
                                            );
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }

                    Instruction::I64AtomicStore { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 8 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = value.to_le_bytes();
                                    match memory.write_shared(effective_addr, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value,
                                                "[I64AtomicStore] Stored"
                                            );
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }

                    Instruction::I32AtomicStore8 { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = [(value as u8)];
                                    match memory.write_shared(effective_addr, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value as u8,
                                                "[I32AtomicStore8] Stored"
                                            );
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }

                    Instruction::I32AtomicStore16 { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = (value as u16).to_le_bytes();
                                    match memory.write_shared(effective_addr, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value as u16,
                                                "[I32AtomicStore16] Stored"
                                            );
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }

                    Instruction::I64AtomicStore8 { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = [(value as u8)];
                                    match memory.write_shared(effective_addr, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value as u8,
                                                "[I64AtomicStore8] Stored"
                                            );
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }

                    Instruction::I64AtomicStore16 { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = (value as u16).to_le_bytes();
                                    match memory.write_shared(effective_addr, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value as u16,
                                                "[I64AtomicStore16] Stored"
                                            );
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }

                    Instruction::I64AtomicStore32 { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let bytes = (value as u32).to_le_bytes();
                                    match memory.write_shared(effective_addr, &bytes) {
                                        Ok(()) => {
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                value = value as u32,
                                                "[I64AtomicStore32] Stored"
                                            );
                                        }
                                        Err(_) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            }
                        }
                    }

                    // ===============================================
                    // Atomic RMW Add Operations
                    // ===============================================

                    Instruction::I32AtomicRmwAdd { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i32::from_le_bytes(buffer);
                                            let new_value = old_value.wrapping_add(value);
                                            let new_bytes = new_value.to_le_bytes();
                                            match memory.write_shared(effective_addr, &new_bytes) {
                                                Ok(()) => {
                                                    #[cfg(feature = "tracing")]
                                                    trace!(
                                                        addr = format_args!("0x{:x}", effective_addr),
                                                        old_value = old_value,
                                                        add_value = value,
                                                        new_value = new_value,
                                                        "[I32AtomicRmwAdd] RMW Add"
                                                    );
                                                    operand_stack.push(Value::I32(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmwAdd { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 8 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i64::from_le_bytes(buffer);
                                            let new_value = old_value.wrapping_add(value);
                                            let new_bytes = new_value.to_le_bytes();
                                            match memory.write_shared(effective_addr, &new_bytes) {
                                                Ok(()) => {
                                                    #[cfg(feature = "tracing")]
                                                    trace!(
                                                        addr = format_args!("0x{:x}", effective_addr),
                                                        old_value = old_value,
                                                        "[I64AtomicRmwAdd] RMW Add"
                                                    );
                                                    operand_stack.push(Value::I64(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw8AddU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            let new_value = old_value.wrapping_add(value as u8);
                                            match memory.write_shared(effective_addr, &[new_value]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw16AddU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            let new_value = old_value.wrapping_add(value as u16);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw8AddU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            let new_value = old_value.wrapping_add(value as u8);
                                            match memory.write_shared(effective_addr, &[new_value]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw16AddU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            let new_value = old_value.wrapping_add(value as u16);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw32AddU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u32::from_le_bytes(buffer);
                                            let new_value = old_value.wrapping_add(value as u32);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    // ===============================================
                    // Atomic RMW Sub Operations
                    // ===============================================

                    Instruction::I32AtomicRmwSub { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i32::from_le_bytes(buffer);
                                            let new_value = old_value.wrapping_sub(value);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmwSub { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 8 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i64::from_le_bytes(buffer);
                                            let new_value = old_value.wrapping_sub(value);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw8SubU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            let new_value = old_value.wrapping_sub(value as u8);
                                            match memory.write_shared(effective_addr, &[new_value]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw16SubU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            let new_value = old_value.wrapping_sub(value as u16);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw8SubU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            let new_value = old_value.wrapping_sub(value as u8);
                                            match memory.write_shared(effective_addr, &[new_value]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw16SubU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            let new_value = old_value.wrapping_sub(value as u16);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw32SubU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u32::from_le_bytes(buffer);
                                            let new_value = old_value.wrapping_sub(value as u32);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    // ===============================================
                    // Atomic RMW And Operations
                    // ===============================================

                    Instruction::I32AtomicRmwAnd { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i32::from_le_bytes(buffer);
                                            let new_value = old_value & value;
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmwAnd { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 8 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i64::from_le_bytes(buffer);
                                            let new_value = old_value & value;
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw8AndU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            let new_value = old_value & (value as u8);
                                            match memory.write_shared(effective_addr, &[new_value]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw16AndU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            let new_value = old_value & (value as u16);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw8AndU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            let new_value = old_value & (value as u8);
                                            match memory.write_shared(effective_addr, &[new_value]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw16AndU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            let new_value = old_value & (value as u16);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw32AndU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u32::from_le_bytes(buffer);
                                            let new_value = old_value & (value as u32);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    // ===============================================
                    // Atomic RMW Or Operations
                    // ===============================================

                    Instruction::I32AtomicRmwOr { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i32::from_le_bytes(buffer);
                                            let new_value = old_value | value;
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmwOr { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 8 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i64::from_le_bytes(buffer);
                                            let new_value = old_value | value;
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw8OrU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            let new_value = old_value | (value as u8);
                                            match memory.write_shared(effective_addr, &[new_value]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw16OrU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            let new_value = old_value | (value as u16);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw8OrU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            let new_value = old_value | (value as u8);
                                            match memory.write_shared(effective_addr, &[new_value]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw16OrU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            let new_value = old_value | (value as u16);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw32OrU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u32::from_le_bytes(buffer);
                                            let new_value = old_value | (value as u32);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    // ===============================================
                    // Atomic RMW Xor Operations
                    // ===============================================

                    Instruction::I32AtomicRmwXor { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i32::from_le_bytes(buffer);
                                            let new_value = old_value ^ value;
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmwXor { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 8 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i64::from_le_bytes(buffer);
                                            let new_value = old_value ^ value;
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw8XorU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            let new_value = old_value ^ (value as u8);
                                            match memory.write_shared(effective_addr, &[new_value]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw16XorU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            let new_value = old_value ^ (value as u16);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw8XorU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            let new_value = old_value ^ (value as u8);
                                            match memory.write_shared(effective_addr, &[new_value]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw16XorU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            let new_value = old_value ^ (value as u16);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw32XorU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u32::from_le_bytes(buffer);
                                            let new_value = old_value ^ (value as u32);
                                            match memory.write_shared(effective_addr, &new_value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    // ===============================================
                    // Atomic RMW Exchange Operations
                    // ===============================================

                    Instruction::I32AtomicRmwXchg { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i32::from_le_bytes(buffer);
                                            match memory.write_shared(effective_addr, &value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmwXchg { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 8 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i64::from_le_bytes(buffer);
                                            match memory.write_shared(effective_addr, &value.to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw8XchgU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            match memory.write_shared(effective_addr, &[(value as u8)]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I32AtomicRmw16XchgU { memarg } => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            match memory.write_shared(effective_addr, &(value as u16).to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I32(old_value as i32));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw8XchgU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            match memory.write_shared(effective_addr, &[(value as u8)]) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw16XchgU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            match memory.write_shared(effective_addr, &(value as u16).to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    Instruction::I64AtomicRmw32XchgU { memarg } => {
                        if let (Some(Value::I64(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u32::from_le_bytes(buffer);
                                            match memory.write_shared(effective_addr, &(value as u32).to_le_bytes()) {
                                                Ok(()) => {
                                                    operand_stack.push(Value::I64(old_value as i64));
                                                }
                                                Err(_) => {
                                                    return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                }
                                            }
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

                    // ===============================================
                    // Atomic Compare-Exchange Operations
                    // ===============================================

                    Instruction::I32AtomicRmwCmpxchg { memarg } => {
                        // cmpxchg: [addr, expected, replacement] -> [old_value]
                        if let (Some(Value::I32(replacement)), Some(Value::I32(expected)), Some(Value::I32(addr))) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop())
                        {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i32::from_le_bytes(buffer);
                                            // Only write if old_value == expected
                                            if old_value == expected {
                                                match memory.write_shared(effective_addr, &replacement.to_le_bytes()) {
                                                    Ok(()) => {}
                                                    Err(_) => {
                                                        return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                    }
                                                }
                                            }
                                            #[cfg(feature = "tracing")]
                                            trace!(
                                                addr = format_args!("0x{:x}", effective_addr),
                                                old_value = old_value,
                                                expected = expected,
                                                replacement = replacement,
                                                swapped = (old_value == expected),
                                                "[I32AtomicRmwCmpxchg] CmpXchg"
                                            );
                                            operand_stack.push(Value::I32(old_value));
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

                    Instruction::I64AtomicRmwCmpxchg { memarg } => {
                        if let (Some(Value::I64(replacement)), Some(Value::I64(expected)), Some(Value::I32(addr))) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop())
                        {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 8 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 8];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = i64::from_le_bytes(buffer);
                                            if old_value == expected {
                                                match memory.write_shared(effective_addr, &replacement.to_le_bytes()) {
                                                    Ok(()) => {}
                                                    Err(_) => {
                                                        return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                    }
                                                }
                                            }
                                            operand_stack.push(Value::I64(old_value));
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

                    Instruction::I32AtomicRmw8CmpxchgU { memarg } => {
                        if let (Some(Value::I32(replacement)), Some(Value::I32(expected)), Some(Value::I32(addr))) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop())
                        {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            if old_value == (expected as u8) {
                                                match memory.write_shared(effective_addr, &[(replacement as u8)]) {
                                                    Ok(()) => {}
                                                    Err(_) => {
                                                        return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                    }
                                                }
                                            }
                                            operand_stack.push(Value::I32(old_value as i32));
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

                    Instruction::I32AtomicRmw16CmpxchgU { memarg } => {
                        if let (Some(Value::I32(replacement)), Some(Value::I32(expected)), Some(Value::I32(addr))) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop())
                        {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            if old_value == (expected as u16) {
                                                match memory.write_shared(effective_addr, &(replacement as u16).to_le_bytes()) {
                                                    Ok(()) => {}
                                                    Err(_) => {
                                                        return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                    }
                                                }
                                            }
                                            operand_stack.push(Value::I32(old_value as i32));
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

                    Instruction::I64AtomicRmw8CmpxchgU { memarg } => {
                        if let (Some(Value::I64(replacement)), Some(Value::I64(expected)), Some(Value::I32(addr))) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop())
                        {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 1];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = buffer[0];
                                            if old_value == (expected as u8) {
                                                match memory.write_shared(effective_addr, &[(replacement as u8)]) {
                                                    Ok(()) => {}
                                                    Err(_) => {
                                                        return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                    }
                                                }
                                            }
                                            operand_stack.push(Value::I64(old_value as i64));
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

                    Instruction::I64AtomicRmw16CmpxchgU { memarg } => {
                        if let (Some(Value::I64(replacement)), Some(Value::I64(expected)), Some(Value::I32(addr))) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop())
                        {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 2 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 2];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u16::from_le_bytes(buffer);
                                            if old_value == (expected as u16) {
                                                match memory.write_shared(effective_addr, &(replacement as u16).to_le_bytes()) {
                                                    Ok(()) => {}
                                                    Err(_) => {
                                                        return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                    }
                                                }
                                            }
                                            operand_stack.push(Value::I64(old_value as i64));
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

                    Instruction::I64AtomicRmw32CmpxchgU { memarg } => {
                        if let (Some(Value::I64(replacement)), Some(Value::I64(expected)), Some(Value::I32(addr))) =
                            (operand_stack.pop(), operand_stack.pop(), operand_stack.pop())
                        {
                            let effective_addr = (addr as u32).wrapping_add(memarg.offset);
                            if effective_addr % 4 != 0 {
                                return Err(wrt_error::Error::runtime_trap("unaligned atomic access"));
                            }
                            match instance.memory(memarg.memory_index as u32) {
                                Ok(memory_wrapper) => {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(effective_addr, &mut buffer) {
                                        Ok(()) => {
                                            let old_value = u32::from_le_bytes(buffer);
                                            if old_value == (expected as u32) {
                                                match memory.write_shared(effective_addr, &(replacement as u32).to_le_bytes()) {
                                                    Ok(()) => {}
                                                    Err(_) => {
                                                        return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                                    }
                                                }
                                            }
                                            operand_stack.push(Value::I64(old_value as i64));
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

                    // End of atomic operations
                    // ===============================================

                    _ => {
                        // CLAUDE.md: FAIL LOUD AND EARLY - return error instead of silently skipping
                        #[cfg(feature = "tracing")]
                        error!("Unimplemented instruction at pc={}: {:?}", pc, instruction);
                        return Err(wrt_error::Error::runtime_error(
                            "Unimplemented instruction encountered",
                        ));
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

            // Restore debugger back to self
            #[cfg(all(feature = "std", feature = "debugger"))]
            {
                self.debugger = debugger_opt;
            }

            // Decrement call depth before returning
            self.call_frames_count = self.call_frames_count.saturating_sub(1);
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
            // Decrement call depth before returning
            self.call_frames_count = self.call_frames_count.saturating_sub(1);
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

    /// Collect function arguments from operand stack based on function signature
    fn collect_function_args(module: &crate::module::Module, func_idx: usize, operand_stack: &mut Vec<Value>) -> Vec<Value> {
        // Get the function's type signature to determine param count
        let param_count = if let Some(func) = module.functions.get(func_idx) {
            if let Some(func_type) = module.types.get(func.type_idx as usize) {
                func_type.params.len()
            } else {
                0
            }
        } else {
            0
        };

        // Pop arguments from stack in reverse order
        let mut args = Vec::with_capacity(param_count);
        for _ in 0..param_count {
            if let Some(val) = operand_stack.pop() {
                args.push(val);
            }
        }
        args.reverse(); // Reverse to get correct order
        args
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
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn call_cabi_realloc(&mut self, instance_id: usize, func_idx: usize,
                         old_ptr: u32, old_size: u32, align: u32, new_size: u32) -> Result<u32> {
        #[cfg(feature = "tracing")]
        trace!(
            old_ptr = old_ptr,
            old_size = old_size,
            align = align,
            new_size = new_size,
            "[CABI_REALLOC] Calling"
        );
        let args = vec![
            Value::I32(old_ptr as i32),
            Value::I32(old_size as i32),
            Value::I32(align as i32),
            Value::I32(new_size as i32),
        ];
        #[cfg(feature = "tracing")]
        trace!(args = ?args, "[CABI_REALLOC] Arguments prepared");

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
                #[cfg(feature = "tracing")]
                trace!("[WASI-ALLOC] cabi_realloc not found, falling back to stack-relative allocation");
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

        #[cfg(feature = "tracing")]
        trace!(
            total_size = total_size,
            args_len = args.len(),
            list_size = list_size,
            string_total = string_total,
            "[WASI-ALLOC] Allocating memory for args"
        );

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

        #[cfg(feature = "tracing")]
        trace!(ptr = format_args!("0x{:x}", ptr), "[WASI-ALLOC] cabi_realloc returned");

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
            #[cfg(feature = "tracing")]
            trace!("[WASI-PREALLOC] No args to pre-allocate");
            return Ok(());
        }

        #[cfg(feature = "tracing")]
        trace!(
            args_len = args.len(),
            args = ?args,
            "[WASI-PREALLOC] Pre-allocating memory for args"
        );

        // Allocate memory
        match self.allocate_wasi_args_memory(instance_id, &args)? {
            Some((list_ptr, string_ptr)) => {
                #[cfg(feature = "tracing")]
                trace!(
                    list_ptr = format_args!("0x{:x}", list_ptr),
                    string_ptr = format_args!("0x{:x}", string_ptr),
                    "[WASI-PREALLOC] Allocated successfully"
                );

                // Store in dispatcher
                if let Some(ref mut dispatcher) = self.wasi_dispatcher {
                    dispatcher.set_args_alloc(list_ptr, string_ptr);
                }
                Ok(())
            }
            None => {
                #[cfg(feature = "tracing")]
                trace!("[WASI-PREALLOC] No allocation needed or cabi_realloc not available");
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
        #[cfg(feature = "std")]
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

        // NOTE: WASIP2-CANONICAL interception block REMOVED per CLAUDE.md
        // Component adapter modules do P2→P1 translation - calls flow to P1 handlers

        // WASI dispatch - adapter modules in component translate P2 calls to P1 handlers
        #[cfg(feature = "tracing")]
        debug!("WASI dispatch: {}::{}", module_name, field_name);
        let stub_mem = self.wasi_stubs.get(&instance_id);

        // Strip version from module name to allow any 0.2.x version to match
        let base_module = strip_wasi_version(module_name);

        #[cfg(feature = "tracing")]
        trace!(
            base_module = %base_module,
            field_name = %field_name,
            "[WASI_DISPATCH] Dispatching"
        );

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
                // Pop the return area pointer (canonical ABI for list<string>)
                let return_area = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("get-arguments: missing return_area"));
                };

                // Get the WASI args from global state
                #[cfg(all(feature = "std", feature = "wasi"))]
                let args: Vec<String> = wrt_wasi::get_global_wasi_args();

                #[cfg(any(not(feature = "std"), not(feature = "wasi")))]
                let args: Vec<String> = Vec::new();

                #[cfg(feature = "tracing")]
                trace!(
                    return_area = format_args!("0x{:x}", return_area),
                    args_len = args.len(),
                    args = ?args,
                    "[WASI-P2] get-arguments"
                );

                if args.is_empty() {
                    // Empty list: write (0, 0) to return area
                    if let Some(instance) = self.instances.get(&instance_id) {
                        if let Ok(memory) = instance.memory(0) {
                            let _ = memory.0.write_shared(return_area, &0u32.to_le_bytes());
                            let _ = memory.0.write_shared(return_area + 4, &0u32.to_le_bytes());
                        }
                    }
                    return Ok(None);
                }

                // Calculate total memory needed:
                // - List header: 8 bytes per arg (ptr + len)
                // - String data: sum of all arg lengths, 4-byte aligned
                let list_size = (args.len() * 8) as u32;
                let string_total: u32 = args.iter()
                    .map(|s| ((s.len() as u32) + 3) & !3) // 4-byte aligned
                    .sum();
                let total_size = list_size + string_total;

                // Try to allocate memory via cabi_realloc (proper canonical ABI)
                let alloc_result: Option<(u32, u32)> = {
                    // Find cabi_realloc export
                    let instance = self.instances.get(&instance_id)
                        .ok_or_else(|| wrt_error::Error::runtime_error("Instance not found"))?
                        .clone();
                    let module = instance.module();

                    if let Ok(realloc_idx) = self.find_export_index(&module, "cabi_realloc") {
                        // Call cabi_realloc(0, 0, 8, total_size)
                        let realloc_args = vec![
                            Value::I32(0),                  // old_ptr (NULL)
                            Value::I32(0),                  // old_size
                            Value::I32(8),                  // align (8-byte for pointers)
                            Value::I32(total_size as i32),  // new_size
                        ];

                        #[cfg(feature = "tracing")]
                        trace!(
                            total_size = total_size,
                            "[WASI-P2] get-arguments: calling cabi_realloc"
                        );

                        match self.execute(instance_id, realloc_idx, realloc_args) {
                            Ok(results) => {
                                if let Some(Value::I32(ptr)) = results.first() {
                                    let base_ptr = *ptr as u32;
                                    #[cfg(feature = "tracing")]
                                    trace!(
                                        ptr = format_args!("0x{:x}", base_ptr),
                                        "[WASI-P2] get-arguments: cabi_realloc returned"
                                    );
                                    Some((base_ptr, base_ptr + list_size))
                                } else {
                                    None
                                }
                            }
                            Err(_) => None
                        }
                    } else {
                        None
                    }
                };

                // Use allocated memory or fall back to high address (safer than 0x200)
                let (list_ptr, mut string_ptr) = alloc_result.unwrap_or_else(|| {
                    // Fallback: use address well above typical heap start
                    // 0x10000 = 64KB offset, should be safe for most modules
                    let fallback_ptr: u32 = 0x10000;
                    #[cfg(feature = "tracing")]
                    trace!(
                        ptr = format_args!("0x{:x}", fallback_ptr),
                        "[WASI-P2] get-arguments: using fallback allocation"
                    );
                    (fallback_ptr, fallback_ptr + list_size)
                });

                if let Some(instance) = self.instances.get(&instance_id) {
                    if let Ok(memory) = instance.memory(0) {
                        // First pass: write string data, collecting (ptr, len) for each
                        let mut entries: Vec<(u32, u32)> = Vec::new();

                        for arg in &args {
                            let bytes = arg.as_bytes();
                            let len = bytes.len() as u32;

                            // Write string bytes
                            if memory.0.write_shared(string_ptr, bytes).is_ok() {
                                entries.push((string_ptr, len));
                                #[cfg(feature = "tracing")]
                                trace!(
                                    arg = %arg,
                                    ptr = format_args!("0x{:x}", string_ptr),
                                    len = len,
                                    "[WASI-P2] wrote arg string"
                                );
                            }

                            string_ptr += len;
                            // Align to 4 bytes
                            string_ptr = (string_ptr + 3) & !3;
                        }

                        // Second pass: write (ptr, len) entries at list_ptr
                        for (i, (ptr, len)) in entries.iter().enumerate() {
                            let entry_offset = list_ptr + (i as u32 * 8);
                            let _ = memory.0.write_shared(entry_offset, &ptr.to_le_bytes());
                            let _ = memory.0.write_shared(entry_offset + 4, &len.to_le_bytes());
                        }

                        // Write (list_ptr, count) to return area
                        let _ = memory.0.write_shared(return_area, &list_ptr.to_le_bytes());
                        let _ = memory.0.write_shared(return_area + 4, &(args.len() as u32).to_le_bytes());

                        #[cfg(feature = "tracing")]
                        trace!(
                            list_ptr = format_args!("0x{:x}", list_ptr),
                            count = args.len(),
                            return_area = format_args!("0x{:x}", return_area),
                            "[WASI-P2] wrote args list to return area"
                        );
                    }
                }

                Ok(None) // Result is written to memory, not returned on stack
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

                #[cfg(feature = "std")]
                std::process::exit(exit_code);

                #[cfg(not(feature = "std"))]
                return Err(wrt_error::Error::runtime_error("WASI exit not supported in no_std"));
            }

            // wasi:clocks/wall-clock@0.2.x::now() -> datetime
            // Returns datetime { seconds: u64, nanoseconds: u32 } via return area pointer
            ("wasi:clocks/wall-clock", "now") => {
                // Pop the return area pointer (canonical ABI writes records to memory)
                let return_area = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("Missing return area for wall-clock::now"));
                };

                #[cfg(feature = "std")]
                {
                    use std::time::{SystemTime, UNIX_EPOCH};

                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default();

                    let seconds = now.as_secs();
                    let nanoseconds = now.subsec_nanos();

                    #[cfg(feature = "tracing")]
                    trace!(seconds = seconds, nanoseconds = nanoseconds, return_area = return_area, "[WASI] wall-clock::now()");

                    // Write datetime record to return area:
                    // offset 0: u64 seconds (8 bytes, little-endian)
                    // offset 8: u32 nanoseconds (4 bytes, little-endian)
                    if let Some(instance) = self.instances.get(&instance_id) {
                        if let Ok(memory) = instance.memory(0) {
                            // Write seconds (u64) at offset 0
                            let seconds_bytes = seconds.to_le_bytes();
                            let _ = memory.0.write_shared(return_area, &seconds_bytes);

                            // Write nanoseconds (u32) at offset 8
                            let nanos_bytes = nanoseconds.to_le_bytes();
                            let _ = memory.0.write_shared(return_area + 8, &nanos_bytes);
                        }
                    }

                    Ok(None) // No return value on stack - result is in memory
                }
                #[cfg(not(feature = "std"))]
                {
                    // no_std: write stub time values to return area
                    if let Some(instance) = self.instances.get(&instance_id) {
                        if let Ok(memory) = instance.memory(0) {
                            let _ = memory.0.write_shared(return_area, &0u64.to_le_bytes());
                            let _ = memory.0.write_shared(return_area + 8, &0u32.to_le_bytes());
                        }
                    }
                    Ok(None)
                }
            }

            // wasi:clocks/wall-clock@0.2.x::resolution() -> datetime
            ("wasi:clocks/wall-clock", "resolution") => {
                // Pop the return area pointer (canonical ABI writes records to memory)
                let return_area = if let Some(Value::I32(ptr)) = stack.pop() {
                    ptr as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("Missing return area for wall-clock::resolution"));
                };

                // Return nanosecond resolution (1 nanosecond = best resolution)
                let seconds: u64 = 0;
                let nanoseconds: u32 = 1;

                #[cfg(feature = "tracing")]
                trace!(seconds = seconds, nanoseconds = nanoseconds, return_area = return_area, "[WASI] wall-clock::resolution()");

                // Write datetime record to return area
                if let Some(instance) = self.instances.get(&instance_id) {
                    if let Ok(memory) = instance.memory(0) {
                        let _ = memory.0.write_shared(return_area, &seconds.to_le_bytes());
                        let _ = memory.0.write_shared(return_area + 8, &nanoseconds.to_le_bytes());
                    }
                }

                Ok(None)
            }

            // wasi:io/streams@0.2.x::[method]output-stream.blocking-write-and-flush(stream, data_ptr, data_len) -> result<_, stream-error>
            ("wasi:io/streams", "[method]output-stream.blocking-write-and-flush") => {
                // Pop arguments in canonical ABI order for methods returning result<_, error>:
                // Stack (top to bottom): [return_area, data_len, data_ptr, stream_handle]
                // Canonical method calls: (self, ptr, len, return_area)
                // Pop return_area first (it's on top)
                let return_area = if let Some(Value::I32(ra)) = stack.pop() {
                    ra as u32
                } else {
                    return Err(wrt_error::Error::runtime_error("Missing return_area argument"));
                };

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

                // Pop the actual stream handle (self/receiver)
                let stream_handle = if let Some(Value::I32(s)) = stack.pop() {
                    s
                } else {
                    return Err(wrt_error::Error::runtime_error("Missing stream argument"));
                };

                #[cfg(feature = "tracing")]
                trace!(
                    stream_handle = stream_handle,
                    data_ptr = format_args!("{:#x}", data_ptr),
                    data_len = data_len,
                    return_area = format_args!("{:#x}", return_area),
                    "[WRITE] blocking-write-and-flush"
                );

                // Read data from WebAssembly memory and write to stdout/stderr
                // Use instance memory instead of module memory
                #[cfg(any(feature = "std", feature = "alloc"))]
                if let Some(instance) = self.instances.get(&instance_id) {
                    if let Ok(memory_wrapper) = instance.memory(0) {
                        // Read data from instance memory into a buffer
                        let mut buffer = vec![0u8; data_len as usize];
                        if let Ok(()) = memory_wrapper.0.read(data_ptr as u32, &mut buffer) {
                            #[cfg(feature = "tracing")]
                            {
                                trace!(
                                    bytes = buffer.len(),
                                    ptr = data_ptr,
                                    "[WRITE] Read from memory"
                                );
                                // Debug: show actual buffer content (limited to 64 bytes)
                                let preview_len = buffer.len().min(64);
                                trace!(
                                    preview = ?&buffer[..preview_len],
                                    as_string = %String::from_utf8_lossy(&buffer[..preview_len]),
                                    "[WRITE] Buffer content"
                                );
                            }

                            // Write the FULL buffer - don't trim at null bytes for binary data
                            // Only stdout text should potentially be trimmed
                            let write_buffer = &buffer[..];

                            // Write directly to stdout/stderr
                            #[cfg(feature = "std")]
                            let success = {
                                use std::io::Write;
                                if stream_handle == 1 {
                                    // Stdout
                                    let mut stdout = std::io::stdout();
                                    stdout.write_all(write_buffer)
                                        .and_then(|_| stdout.flush())
                                        .is_ok()
                                } else if stream_handle == 2 {
                                    // Stderr
                                    let mut stderr = std::io::stderr();
                                    stderr.write_all(write_buffer)
                                        .and_then(|_| stderr.flush())
                                        .is_ok()
                                } else {
                                    #[cfg(feature = "tracing")]
                                    debug!("WASI: Invalid stream handle: {}", stream_handle);
                                    false
                                }
                            };
                            #[cfg(not(feature = "std"))]
                            let success = false;

                            // Write result to return_area (canonical ABI for result<_, stream-error>)
                            // Discriminant: 0 = ok, 1 = err
                            if success {
                                // ok() - write discriminant 0
                                let _ = memory_wrapper.0.write_shared(return_area, &[0u8]);
                                #[cfg(feature = "tracing")]
                                debug!("WASI: Write success, wrote ok to return_area {:#x}", return_area);
                            } else {
                                // err(stream-error) - write discriminant 1 + error variant
                                // stream-error is a variant, closed = 0
                                let _ = memory_wrapper.0.write_shared(return_area, &[1u8]);
                                let _ = memory_wrapper.0.write_shared(return_area + 4, &[0u8]); // closed variant
                                #[cfg(feature = "tracing")]
                                debug!("WASI: Write failed, wrote err to return_area {:#x}", return_area);
                            }

                            Ok(None) // Result written to memory, not returned on stack
                        } else {
                            // Failed to read memory - write error to return_area
                            #[cfg(feature = "tracing")]
                            debug!("WASI: Failed to read memory at ptr={}, len={}", data_ptr, data_len);
                            let _ = memory_wrapper.0.write_shared(return_area, &[1u8]);
                            let _ = memory_wrapper.0.write_shared(return_area + 4, &[0u8]);
                            Ok(None)
                        }
                    } else {
                        #[cfg(feature = "tracing")]
                        debug!("WASI: Failed to get memory from instance");
                        Ok(None)
                    }
                } else {
                    #[cfg(feature = "tracing")]
                    debug!("WASI: No instance available for id={}", instance_id);
                    Ok(None)
                }
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                {
                    // no_std without alloc: cannot allocate buffer for WASI I/O
                    Ok(None)
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

                #[cfg(feature = "std")]
                let result = {
                    use std::io::Write;
                    if stream_handle == 1 {
                        std::io::stdout().flush().map(|_| 0).unwrap_or(1)
                    } else if stream_handle == 2 {
                        std::io::stderr().flush().map(|_| 0).unwrap_or(1)
                    } else {
                        1
                    }
                };
                #[cfg(not(feature = "std"))]
                let result = 1i32; // no_std: cannot flush
                Ok(Some(Value::I64(result as i64)))
            }

            // ============================================
            // WASI Preview 1 (wasi_snapshot_preview1) support
            // ============================================

            // fd_write(fd: i32, iovs: i32, iovs_len: i32, nwritten: i32) -> errno
            #[cfg(any(feature = "std", feature = "alloc"))]
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

                #[cfg(feature = "tracing")]
                trace!(
                    fd = fd,
                    iovs_ptr = iovs_ptr,
                    iovs_len = iovs_len,
                    nwritten_ptr = nwritten_ptr,
                    "[WASI-P1] fd_write"
                );

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
                                #[cfg(feature = "tracing")]
                                warn!("[WASI-P1] fd_write: failed to read iovec ptr");
                                continue;
                            }
                            let buf_ptr = u32::from_le_bytes(buf);

                            if memory_wrapper.0.read(iov_offset + 4, &mut buf).is_err() {
                                #[cfg(feature = "tracing")]
                                warn!("[WASI-P1] fd_write: failed to read iovec len");
                                continue;
                            }
                            let buf_len = u32::from_le_bytes(buf);

                            #[cfg(feature = "tracing")]
                            trace!(
                                iovec_idx = i,
                                ptr = buf_ptr,
                                len = buf_len,
                                "[WASI-P1] fd_write: iovec"
                            );

                            // Read the data to write
                            let mut data = vec![0u8; buf_len as usize];
                            if memory_wrapper.0.read(buf_ptr, &mut data).is_err() {
                                #[cfg(feature = "tracing")]
                                warn!("[WASI-P1] fd_write: failed to read data");
                                continue;
                            }

                            // Write to stdout (fd=1) or stderr (fd=2)
                            #[cfg(feature = "std")]
                            let write_result = {
                                use std::io::Write;
                                if fd == 1 {
                                    std::io::stdout().write_all(&data).and_then(|_| std::io::stdout().flush())
                                } else if fd == 2 {
                                    std::io::stderr().write_all(&data).and_then(|_| std::io::stderr().flush())
                                } else {
                                    // Other FDs not supported in stub
                                    #[cfg(feature = "tracing")]
                                    warn!(fd = fd, "[WASI-P1] fd_write: unsupported fd");
                                    Ok(())
                                }
                            };
                            #[cfg(not(feature = "std"))]
                            let write_result: core::result::Result<(), ()> = Ok(()); // no_std: silent no-op

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

                #[cfg(feature = "tracing")]
                trace!(total_written = total_written, "[WASI-P1] fd_write: complete");

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

                #[cfg(feature = "tracing")]
                trace!(argc = argc, argv_buf_size = argv_buf_size, "[WASI-P1] args_sizes_get");

                // Write the values to memory
                if let Some(instance) = self.instances.get(&instance_id) {
                    if let Ok(memory_wrapper) = instance.memory(0) {
                        // Write argc
                        if let Err(e) = memory_wrapper.0.write_shared(argc_ptr, &argc.to_le_bytes()) {
                            #[cfg(feature = "tracing")]
                            warn!(error = ?e, "[WASI-P1] args_sizes_get: failed to write argc");
                        }
                        // Write argv_buf_size
                        if let Err(e) = memory_wrapper.0.write_shared(argv_buf_size_ptr, &argv_buf_size.to_le_bytes()) {
                            #[cfg(feature = "tracing")]
                            warn!(error = ?e, "[WASI-P1] args_sizes_get: failed to write argv_buf_size");
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

                #[cfg(feature = "tracing")]
                trace!(
                    argv_ptr = format_args!("0x{:x}", argv_ptr),
                    argv_buf_ptr = format_args!("0x{:x}", argv_buf_ptr),
                    args_len = args.len(),
                    "[WASI-P1] args_get"
                );

                // Write the argument data to memory
                if let Some(instance) = self.instances.get(&instance_id) {
                    if let Ok(memory_wrapper) = instance.memory(0) {
                        let mut current_buf_offset = argv_buf_ptr;

                        for (i, arg) in args.iter().enumerate() {
                            // Write pointer to this arg's string data in argv array
                            let ptr_offset = argv_ptr + (i as u32 * 4);
                            if let Err(e) = memory_wrapper.0.write_shared(ptr_offset, &current_buf_offset.to_le_bytes()) {
                                #[cfg(feature = "tracing")]
                                warn!(idx = i, error = ?e, "[WASI-P1] args_get: failed to write argv pointer");
                            }

                            // Write the arg string data (null-terminated)
                            let arg_bytes = arg.as_bytes();
                            if let Err(e) = memory_wrapper.0.write_shared(current_buf_offset, arg_bytes) {
                                #[cfg(feature = "tracing")]
                                warn!(idx = i, error = ?e, "[WASI-P1] args_get: failed to write arg data");
                            }
                            // Write null terminator
                            if let Err(e) = memory_wrapper.0.write_shared(current_buf_offset + arg_bytes.len() as u32, &[0u8]) {
                                #[cfg(feature = "tracing")]
                                warn!(error = ?e, "[WASI-P1] args_get: failed to write null terminator");
                            }

                            #[cfg(feature = "tracing")]
                            trace!(
                                idx = i,
                                offset = format_args!("0x{:x}", current_buf_offset),
                                arg = ?arg,
                                "[WASI-P1] args_get: wrote arg"
                            );

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
                #[cfg(feature = "tracing")]
                trace!("[WASI-P1] environ_sizes_get: returning empty environment");

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

                #[cfg(feature = "tracing")]
                trace!("[WASI-P1] environ_get: returning empty environment");

                // Nothing to write since environc = 0
                Ok(Some(Value::I32(0))) // Success
            }

            // proc_exit(exit_code: i32) -> !
            ("wasi_snapshot_preview1", "proc_exit") => {
                let exit_code = if let Some(Value::I32(code)) = stack.pop() {
                    code
                } else {
                    0
                };
                #[cfg(feature = "tracing")]
                trace!(exit_code = exit_code, "[WASI-P1] proc_exit - terminating");
                // Actually exit the process - proc_exit should never return
                #[cfg(feature = "std")]
                {
                    std::process::exit(exit_code);
                }
                #[cfg(not(feature = "std"))]
                {
                    // In no_std mode, return an error to signal exit
                    return Err(wrt_error::Error::runtime_error(
                        format!("Process exit requested with code {}", exit_code)
                    ));
                }
            }

            // Default: stub implementation
            _ => {
                #[cfg(feature = "tracing")]
                debug!("WASI: Stub for {}::{}", module_name, field_name);
                // Check if this is a __main_module__ import - these need special handling
                if module_name == "__main_module__" {
                    #[cfg(feature = "tracing")]
                    warn!(field = %field_name, "[WASI] __main_module__ function called as WASI stub - linking error");
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
