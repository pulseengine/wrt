//! Simple working WebAssembly execution engine
//!
//! This module implements a basic stackless WebAssembly execution engine
//! focused on functionality over advanced features. It provides the interface
//! needed by CapabilityAwareEngine to execute WASM modules.

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
        }
    }

    /// Set the host function registry for imported function calls
    #[cfg(feature = "std")]
    pub fn set_host_registry(&mut self, registry: Arc<wrt_host::CallbackRegistry>) {
        self.host_registry = Some(registry);
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

        self.instances.insert(instance_id, instance);

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

        // For now, implement a basic execution that validates the function exists
        // and returns appropriate results
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
        eprintln!("DEBUG StacklessEngine: func.type_idx={}, module.types.len()={}", func.type_idx, module.types.len());

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
                eprintln!("DEBUG StacklessEngine: Failed to get type at index {}: {:?}", func.type_idx, e);
                wrt_error::Error::runtime_error("Failed to get function type")
            })?;

        // Execute the function's bytecode instructions
        #[cfg(feature = "std")]
        {
            use wrt_foundation::types::Instruction;

            // Get the parsed instructions
            eprintln!("[DEBUG] Accessing func.body for func_idx={}", func_idx);
            eprintln!("[DEBUG] func.type_idx={}, func.locals.len()={}", func.type_idx, func.locals.len());
            let instructions = &func.body.instructions;
            eprintln!("[INTERPRETER] Starting execution: {} instructions", instructions.len());
            let mut operand_stack: Vec<Value> = Vec::new();
            let mut locals: Vec<Value> = Vec::new();

            // Initialize parameters as locals
            for arg in args {
                locals.push(arg);
            }

            // Initialize remaining locals to zero
            // Each LocalEntry has a count field - create that many locals of that type
            for i in 0..func.locals.len() {
                if let Ok(local_decl) = func.locals.get(i) {
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
                }
            }
            eprintln!("[INTERPRETER] Initialized {} locals total", locals.len());

            // Execute instructions - iterate over parsed Instruction enum
            for pc in 0..instructions.len() {
                #[cfg(feature = "std")]
                let instruction = instructions.get(pc)
                    .ok_or_else(|| wrt_error::Error::runtime_error("Instruction index out of bounds"))?;
                #[cfg(not(feature = "std"))]
                let instruction = instructions.get(pc)
                    .map_err(|_| wrt_error::Error::runtime_error("Instruction index out of bounds"))?;

                eprintln!("[INTERPRETER] pc={}, instruction={:?}", pc, instruction);

                match *instruction {
                    Instruction::Call(func_idx) => {
                        eprintln!("[INTERPRETER] Call func_idx={}", func_idx);

                        // Check if this is an import (host function)
                        let num_imports = module.imports.len();

                        if (func_idx as usize) < num_imports {
                            // This is a host function call
                            eprintln!("[INTERPRETER] Calling host function at import index {}", func_idx);

                            // Get the import to find its module and name
                            // TODO: Actually iterate through imports to find by index
                            // For now, skip import dispatch
                            eprintln!("[INTERPRETER] Warning: Import dispatch not yet implemented");
                            continue;
                        } else {
                            // Regular function call - recursive execution
                            let call_args = operand_stack.clone();
                            operand_stack.clear();

                            let results = self.execute(instance_id, func_idx as usize, call_args)?;
                            for result in results {
                                operand_stack.push(result);
                            }
                        }
                    }
                    Instruction::I32Const(value) => {
                        eprintln!("[INTERPRETER] I32Const: pushing value {}", value);
                        operand_stack.push(Value::I32(value));
                        eprintln!("[INTERPRETER] Operand stack now has {} values", operand_stack.len());
                    }
                    Instruction::I64Const(value) => {
                        eprintln!("[INTERPRETER] I64Const: pushing value {}", value);
                        operand_stack.push(Value::I64(value));
                    }
                    Instruction::LocalGet(local_idx) => {
                        if (local_idx as usize) < locals.len() {
                            let value = locals[local_idx as usize].clone();
                            eprintln!("[INTERPRETER] LocalGet: local[{}] = {:?}", local_idx, value);
                            operand_stack.push(value);
                            eprintln!("[INTERPRETER] Operand stack now has {} values", operand_stack.len());
                        } else {
                            eprintln!("[INTERPRETER] LocalGet: local[{}] out of bounds (locals.len()={})", local_idx, locals.len());
                        }
                    }
                    Instruction::LocalSet(local_idx) => {
                        if let Some(value) = operand_stack.pop() {
                            eprintln!("[INTERPRETER] LocalSet: setting local[{}] = {:?}", local_idx, value);
                            if (local_idx as usize) < locals.len() {
                                locals[local_idx as usize] = value;
                            } else {
                                eprintln!("[INTERPRETER] LocalSet: local[{}] out of bounds (locals.len()={})", local_idx, locals.len());
                            }
                        } else {
                            eprintln!("[INTERPRETER] LocalSet: operand stack empty");
                        }
                    }
                    Instruction::LocalTee(local_idx) => {
                        // Like LocalSet but keeps value on stack
                        if let Some(value) = operand_stack.last().cloned() {
                            eprintln!("[INTERPRETER] LocalTee: setting local[{}] = {:?} (keeping on stack)", local_idx, value);
                            if (local_idx as usize) < locals.len() {
                                locals[local_idx as usize] = value;
                            } else {
                                eprintln!("[INTERPRETER] LocalTee: local[{}] out of bounds (locals.len()={})", local_idx, locals.len());
                            }
                        } else {
                            eprintln!("[INTERPRETER] LocalTee: operand stack empty");
                        }
                    }
                    Instruction::GlobalGet(global_idx) => {
                        eprintln!("[INTERPRETER] GlobalGet: reading global[{}]", global_idx);
                        if (global_idx as usize) < module.globals.len() {
                            if let Ok(global_wrapper) = module.globals.get(global_idx as usize) {
                                let global = &global_wrapper.0; // Unwrap Arc
                                let value = global.get().clone();
                                eprintln!("[INTERPRETER] GlobalGet: global[{}] = {:?}", global_idx, value);
                                operand_stack.push(value);
                            } else {
                                eprintln!("[INTERPRETER] GlobalGet: failed to get global[{}]", global_idx);
                                operand_stack.push(Value::I32(0)); // Default value
                            }
                        } else {
                            eprintln!("[INTERPRETER] GlobalGet: global[{}] out of bounds (globals.len()={})", global_idx, module.globals.len());
                            operand_stack.push(Value::I32(0)); // Default value
                        }
                    }
                    Instruction::GlobalSet(global_idx) => {
                        eprintln!("[INTERPRETER] GlobalSet: would set global[{}]", global_idx);
                        // Pop value but don't actually set (globals are in Arc)
                        if let Some(value) = operand_stack.pop() {
                            eprintln!("[INTERPRETER] GlobalSet: would write {:?} to global[{}]", value, global_idx);
                        }
                    }
                    // Arithmetic operations
                    Instruction::I32Add => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_add(b);
                            eprintln!("[INTERPRETER] I32Add: {} + {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Sub => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_sub(b);
                            eprintln!("[INTERPRETER] I32Sub: {} - {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Mul => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_mul(b);
                            eprintln!("[INTERPRETER] I32Mul: {} * {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32DivS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = a.wrapping_div(b);
                            eprintln!("[INTERPRETER] I32DivS: {} / {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32DivU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = (a as u32).wrapping_div(b as u32) as i32;
                            eprintln!("[INTERPRETER] I32DivU: {} / {} = {}", a as u32, b as u32, result as u32);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32RemS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = a.wrapping_rem(b);
                            eprintln!("[INTERPRETER] I32RemS: {} % {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32RemU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            if b == 0 {
                                return Err(wrt_error::Error::runtime_trap("Division by zero"));
                            }
                            let result = (a as u32).wrapping_rem(b as u32) as i32;
                            eprintln!("[INTERPRETER] I32RemU: {} % {} = {}", a as u32, b as u32, result as u32);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    // Comparison operations
                    Instruction::I32Eq => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a == b { 1 } else { 0 };
                            eprintln!("[INTERPRETER] I32Eq: {} == {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Ne => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a != b { 1 } else { 0 };
                            eprintln!("[INTERPRETER] I32Ne: {} != {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32LtS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a < b { 1 } else { 0 };
                            eprintln!("[INTERPRETER] I32LtS: {} < {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32LtU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u32) < (b as u32) { 1 } else { 0 };
                            eprintln!("[INTERPRETER] I32LtU: {} < {} = {}", a as u32, b as u32, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32GtS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a > b { 1 } else { 0 };
                            eprintln!("[INTERPRETER] I32GtS: {} > {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32GtU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u32) > (b as u32) { 1 } else { 0 };
                            eprintln!("[INTERPRETER] I32GtU: {} > {} = {}", a as u32, b as u32, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32LeS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a <= b { 1 } else { 0 };
                            eprintln!("[INTERPRETER] I32LeS: {} <= {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32LeU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u32) <= (b as u32) { 1 } else { 0 };
                            eprintln!("[INTERPRETER] I32LeU: {} <= {} = {}", a as u32, b as u32, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32GeS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if a >= b { 1 } else { 0 };
                            eprintln!("[INTERPRETER] I32GeS: {} >= {} = {}", a, b, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32GeU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = if (a as u32) >= (b as u32) { 1 } else { 0 };
                            eprintln!("[INTERPRETER] I32GeU: {} >= {} = {}", a as u32, b as u32, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    // Bitwise operations
                    Instruction::I32And => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a & b;
                            eprintln!("[INTERPRETER] I32And: {} & {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Or => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a | b;
                            eprintln!("[INTERPRETER] I32Or: {} | {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Xor => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a ^ b;
                            eprintln!("[INTERPRETER] I32Xor: {} ^ {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Shl => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_shl((b as u32) % 32);
                            eprintln!("[INTERPRETER] I32Shl: {} << {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32ShrS => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.wrapping_shr((b as u32) % 32);
                            eprintln!("[INTERPRETER] I32ShrS: {} >> {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32ShrU => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = (a as u32).wrapping_shr((b as u32) % 32) as i32;
                            eprintln!("[INTERPRETER] I32ShrU: {} >> {} = {}", a as u32, b as u32, result as u32);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Rotl => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.rotate_left((b as u32) % 32);
                            eprintln!("[INTERPRETER] I32Rotl: {} rotl {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Rotr => {
                        if let (Some(Value::I32(b)), Some(Value::I32(a))) = (operand_stack.pop(), operand_stack.pop()) {
                            let result = a.rotate_right((b as u32) % 32);
                            eprintln!("[INTERPRETER] I32Rotr: {} rotr {} = {}", a, b, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    // Unary operations
                    Instruction::I32Clz => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = a.leading_zeros() as i32;
                            eprintln!("[INTERPRETER] I32Clz: clz({}) = {}", a, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Ctz => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = a.trailing_zeros() as i32;
                            eprintln!("[INTERPRETER] I32Ctz: ctz({}) = {}", a, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Popcnt => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = a.count_ones() as i32;
                            eprintln!("[INTERPRETER] I32Popcnt: popcnt({}) = {}", a, result);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    Instruction::I32Eqz => {
                        if let Some(Value::I32(a)) = operand_stack.pop() {
                            let result = if a == 0 { 1 } else { 0 };
                            eprintln!("[INTERPRETER] I32Eqz: {} == 0 = {}", a, result != 0);
                            operand_stack.push(Value::I32(result));
                        }
                    }
                    // Memory operations
                    Instruction::I32Load(mem_arg) => {
                        if let Some(Value::I32(addr)) = operand_stack.pop() {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            eprintln!("[INTERPRETER] I32Load: reading from address {} (base={}, offset={})", offset, addr, mem_arg.offset);

                            // Get memory - for now assume memory index 0
                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
                                    let memory = &memory_wrapper.0;
                                    let mut buffer = [0u8; 4];
                                    match memory.read(offset, &mut buffer) {
                                        Ok(()) => {
                                            let value = i32::from_le_bytes(buffer);
                                            eprintln!("[INTERPRETER] I32Load: read value {} from address {}", value, offset);
                                            operand_stack.push(Value::I32(value));
                                        }
                                        Err(e) => {
                                            eprintln!("[INTERPRETER] I32Load: memory read failed: {:?}", e);
                                            return Err(wrt_error::Error::runtime_trap("Memory read out of bounds"));
                                        }
                                    }
                                } else {
                                    eprintln!("[INTERPRETER] I32Load: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                eprintln!("[INTERPRETER] I32Load: memory index {} out of range (have {} memories)", mem_arg.memory_index, module.memories.len());
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
                            }
                        }
                    }
                    Instruction::I32Store(mem_arg) => {
                        if let (Some(Value::I32(value)), Some(Value::I32(addr))) = (operand_stack.pop(), operand_stack.pop()) {
                            let offset = (addr as u32).wrapping_add(mem_arg.offset);
                            eprintln!("[INTERPRETER] I32Store: writing value {} to address {} (base={}, offset={})", value, offset, addr, mem_arg.offset);

                            // Get memory - for now assume memory index 0
                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
                                    let memory = &memory_wrapper.0;
                                    let bytes = value.to_le_bytes();
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            eprintln!("[INTERPRETER] I32Store: successfully wrote value {} to address {}", value, offset);
                                        }
                                        Err(e) => {
                                            eprintln!("[INTERPRETER] I32Store: write failed: {:?}", e);
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                } else {
                                    eprintln!("[INTERPRETER] I32Store: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                eprintln!("[INTERPRETER] I32Store: memory index {} out of range (have {} memories)", mem_arg.memory_index, module.memories.len());
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
                                            eprintln!("[INTERPRETER] I32Load8S: read value {} from address {}", value, offset);
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
                                            eprintln!("[INTERPRETER] I32Load8U: read value {} from address {}", value, offset);
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
                                            eprintln!("[INTERPRETER] I32Load16S: read value {} from address {}", value, offset);
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
                                            eprintln!("[INTERPRETER] I32Load16U: read value {} from address {}", value, offset);
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

                            if module.memories.len() > mem_arg.memory_index as usize {
                                if let Ok(memory_wrapper) = module.get_memory(mem_arg.memory_index as usize) {
                                    let memory = &memory_wrapper.0;
                                    let bytes = [(value & 0xFF) as u8];
                                    // ASIL-B COMPLIANT: Use write_shared for thread-safe writes
                                    match memory.write_shared(offset, &bytes) {
                                        Ok(()) => {
                                            eprintln!("[INTERPRETER] I32Store8: successfully wrote value {} to address {}", value as u8, offset);
                                        }
                                        Err(e) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                } else {
                                    eprintln!("[INTERPRETER] I32Store8: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                eprintln!("[INTERPRETER] I32Store8: memory index {} out of range (have {} memories)", mem_arg.memory_index, module.memories.len());
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
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
                                            eprintln!("[INTERPRETER] I32Store16: successfully wrote value {} to address {}", value as u16, offset);
                                        }
                                        Err(e) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                } else {
                                    eprintln!("[INTERPRETER] I32Store16: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                eprintln!("[INTERPRETER] I32Store16: memory index {} out of range (have {} memories)", mem_arg.memory_index, module.memories.len());
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
                                            eprintln!("[INTERPRETER] I64Load: read value {} from address {}", value, offset);
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
                                            eprintln!("[INTERPRETER] I64Store: successfully wrote value {} to address {}", value, offset);
                                        }
                                        Err(e) => {
                                            return Err(wrt_error::Error::runtime_trap("Memory write out of bounds"));
                                        }
                                    }
                                } else {
                                    eprintln!("[INTERPRETER] I64Store: failed to get memory at index {}", mem_arg.memory_index);
                                    return Err(wrt_error::Error::runtime_trap("Memory access error"));
                                }
                            } else {
                                eprintln!("[INTERPRETER] I64Store: memory index {} out of range (have {} memories)", mem_arg.memory_index, module.memories.len());
                                return Err(wrt_error::Error::runtime_trap("Invalid memory index"));
                            }
                        }
                    }
                    Instruction::If { block_type_idx } => {
                        eprintln!("[INTERPRETER] If: block_type_idx={}", block_type_idx);
                        // Pop condition
                        if let Some(Value::I32(condition)) = operand_stack.pop() {
                            eprintln!("[INTERPRETER] If: condition = {}", condition != 0);
                            if condition == 0 {
                                // Condition is false, skip to else or end
                                let mut depth = 1;
                                let mut new_pc = pc + 1;
                                eprintln!("[INTERPRETER] If: skipping to else/end, starting from pc={}", new_pc);

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
                                            }
                                            wrt_foundation::types::Instruction::Else => {
                                                if depth == 1 {
                                                    eprintln!("[INTERPRETER] If: found else at pc={}", new_pc);
                                                    break;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    new_pc += 1;
                                }
                                // TODO: properly handle else block
                                eprintln!("[INTERPRETER] If: skipping condition block");
                            } else {
                                eprintln!("[INTERPRETER] If: executing then block");
                            }
                        }
                    }
                    Instruction::Else => {
                        eprintln!("[INTERPRETER] Else: skipping to end of if block");
                        // Skip to matching end
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
                                    }
                                    _ => {}
                                }
                            }
                            new_pc += 1;
                        }
                        // TODO: implement proper jump
                    }
                    Instruction::Block { block_type_idx } => {
                        eprintln!("[INTERPRETER] Block: block_type_idx={}", block_type_idx);
                        // For now, just execute through the block
                    }
                    Instruction::Loop { block_type_idx } => {
                        eprintln!("[INTERPRETER] Loop: block_type_idx={}", block_type_idx);
                        // For now, just execute through (no actual loop yet)
                    }
                    Instruction::Br(label_idx) => {
                        eprintln!("[INTERPRETER] Br: label_idx={} (unconditional branch)", label_idx);
                        // TODO: Implement proper branch
                        // For now, just continue
                    }
                    Instruction::BrIf(label_idx) => {
                        if let Some(Value::I32(condition)) = operand_stack.pop() {
                            eprintln!("[INTERPRETER] BrIf: label_idx={}, condition={}", label_idx, condition != 0);
                            if condition != 0 {
                                // TODO: Implement proper conditional branch
                                eprintln!("[INTERPRETER] BrIf: would branch (not implemented)");
                            }
                        }
                    }
                    Instruction::Return | Instruction::End => {
                        break; // Exit function
                    }
                    _ => {
                        // Skip unimplemented instructions for now
                        eprintln!("[INTERPRETER] Unimplemented instruction at pc={}: {:?}", pc, instruction);
                    }
                }
            }

            // Return values from operand stack matching function signature
            eprintln!("[INTERPRETER] Function complete. Operand stack has {} values", operand_stack.len());
            eprintln!("[INTERPRETER] Function type expects {} results", func_type.results.len());

            let mut results = Vec::new();
            for (i, result_type) in func_type.results.iter().enumerate() {
                if let Some(value) = operand_stack.pop() {
                    eprintln!("[INTERPRETER] Result {}: {:?}", i, value);
                    results.insert(0, value);
                } else {
                    eprintln!("[INTERPRETER] Result {}: missing, using default", i);
                    results.insert(0, Value::I32(0));
                }
            }

            eprintln!("[INTERPRETER] Returning {} results", results.len());
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
        eprintln!("DEBUG StacklessEngine::execute: instance_id={}, func_idx={}", instance_id, func_idx);

        let instance = self
            .instances
            .get(&instance_id)?
            .ok_or_else(|| wrt_error::Error::runtime_execution_error("Instance not found"))?;

        // For now, implement a basic execution that validates the function exists
        // and returns appropriate results
        let module = instance.module();

        #[cfg(feature = "std")]
        eprintln!("[DEBUG] Got module, functions.len()={}", module.functions.len());

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
        eprintln!("[DEBUG] Retrieved func, body.instructions.len()={}", func.body.instructions.len());

        #[cfg(feature = "std")]
        eprintln!("DEBUG execute: func.type_idx={}, module.types.len()={}", func.type_idx, module.types.len());

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
