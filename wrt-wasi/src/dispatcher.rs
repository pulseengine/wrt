//! WASI Dispatcher - Unified entry point for all WASI calls
//!
//! This module provides a centralized dispatcher that routes WASI calls
//! to the appropriate preview2 implementations. This replaces ad-hoc
//! stub implementations scattered throughout the codebase.
//!
//! # Architecture
//!
//! The dispatcher handles both:
//! - Core WASM values (Value) - raw i32/i64 + memory pointers
//! - Component Model values (our Value enum) - lifted strings, lists, etc.
//!
//! # Usage from wrt-runtime
//!
//! ```ignore
//! use wrt_wasi::WasiDispatcher;
//!
//! let mut dispatcher = WasiDispatcher::new(capabilities)?;
//! let result = dispatcher.dispatch("wasi:clocks/wall-clock", "now", args)?;
//! ```

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(feature = "std")]
use std::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::{
    capabilities::WasiCapabilities,
    prelude::*,
    Value,
};

// Global storage for WASI args and env using Mutex for thread-safety
// This allows passing args from wrtd to the engine without threading through all layers
#[cfg(feature = "std")]
use std::sync::Mutex;

#[cfg(feature = "std")]
static GLOBAL_WASI_ARGS: Mutex<Vec<String>> = Mutex::new(Vec::new());
#[cfg(feature = "std")]
static GLOBAL_WASI_ENV: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

/// Set the global WASI arguments that will be used by all dispatchers
#[cfg(feature = "std")]
pub fn set_global_wasi_args(args: Vec<String>) {
    if let Ok(mut guard) = GLOBAL_WASI_ARGS.lock() {
        *guard = args;
    }
}

/// Get the global WASI arguments
#[cfg(feature = "std")]
pub fn get_global_wasi_args() -> Vec<String> {
    GLOBAL_WASI_ARGS.lock().map(|g| g.clone()).unwrap_or_default()
}

/// Set the global WASI environment variables
#[cfg(feature = "std")]
pub fn set_global_wasi_env(env: Vec<(String, String)>) {
    if let Ok(mut guard) = GLOBAL_WASI_ENV.lock() {
        *guard = env;
    }
}

#[cfg(feature = "wasi-clocks")]
use crate::preview2::clocks::{
    wasi_monotonic_clock_now,
    wasi_monotonic_clock_resolution,
    wasi_wall_clock_now,
    wasi_wall_clock_resolution,
};

#[cfg(feature = "wasi-io")]
use crate::preview2::io::{
    wasi_stream_write,
    wasi_stream_flush,
    wasi_stream_check_write,
};

/// WASI Dispatcher - unified entry point for all WASI function calls
pub struct WasiDispatcher {
    /// WASI capabilities for this dispatcher
    capabilities: WasiCapabilities,
    /// Next available resource handle
    #[allow(dead_code)]
    next_handle: u32,
    /// Stdout handle (pre-allocated)
    stdout_handle: u32,
    /// Stderr handle (pre-allocated)
    stderr_handle: u32,
    /// Command-line arguments
    args: Vec<String>,
    /// Environment variables as (key, value) pairs
    env_vars: Vec<(String, String)>,
    /// Pre-allocated memory for args (list_ptr, string_data_ptr)
    /// Set by the engine after calling cabi_realloc
    args_alloc: Option<(u32, u32)>,
}

/// Describes memory that needs to be allocated via cabi_realloc
/// before a WASI function can complete its return value.
#[derive(Debug, Clone)]
pub struct AllocationRequest {
    /// Total size in bytes to allocate
    pub size: u32,
    /// Required alignment (typically 4 or 8)
    pub align: u32,
    /// Description for debugging
    pub purpose: &'static str,
}

/// Result of a WASI dispatch that may need allocation
#[derive(Debug)]
pub enum DispatchResult {
    /// Dispatch completed successfully with these return values
    Complete(Vec<wrt_foundation::values::Value>),
    /// Dispatch needs memory allocation before it can complete
    /// Returns (allocation request, continuation data)
    NeedsAllocation {
        request: AllocationRequest,
        /// The args to return (we'll write them once memory is allocated)
        args_to_write: Vec<String>,
        /// The return pointer where we'll write (list_ptr, len)
        retptr: u32,
    },
}

impl WasiDispatcher {
    /// Create a new WASI dispatcher with the given capabilities
    pub fn new(capabilities: WasiCapabilities) -> Result<Self> {
        Ok(Self {
            capabilities,
            next_handle: 3, // 1=stdout, 2=stderr, start user handles at 3
            stdout_handle: 1,
            stderr_handle: 2,
            args: Vec::new(),
            env_vars: Vec::new(),
            args_alloc: None,
        })
    }

    /// Create a dispatcher with default (full) capabilities
    pub fn with_defaults() -> Result<Self> {
        Self::new(WasiCapabilities::system_utility()?)
    }

    /// Create a dispatcher with arguments
    pub fn with_args(capabilities: WasiCapabilities, args: Vec<String>) -> Result<Self> {
        Ok(Self {
            capabilities,
            next_handle: 3,
            stdout_handle: 1,
            stderr_handle: 2,
            args,
            env_vars: Vec::new(),
            args_alloc: None,
        })
    }

    /// Create a dispatcher with arguments and environment variables
    pub fn with_args_and_env(
        capabilities: WasiCapabilities,
        args: Vec<String>,
        env_vars: Vec<(String, String)>,
    ) -> Result<Self> {
        Ok(Self {
            capabilities,
            next_handle: 3,
            stdout_handle: 1,
            stderr_handle: 2,
            args,
            env_vars,
            args_alloc: None,
        })
    }

    /// Set the command-line arguments
    pub fn set_args(&mut self, args: Vec<String>) {
        self.args = args;
    }

    /// Set the environment variables
    pub fn set_env(&mut self, env_vars: Vec<(String, String)>) {
        self.env_vars = env_vars;
    }

    /// Set pre-allocated memory for arguments (from cabi_realloc)
    ///
    /// This is set by the engine after calling cabi_realloc to allocate
    /// properly owned memory for argument strings.
    ///
    /// # Arguments
    /// * `list_ptr` - Pointer to the (ptr, len) array for list elements
    /// * `string_data_ptr` - Pointer to start writing string data
    pub fn set_args_alloc(&mut self, list_ptr: u32, string_data_ptr: u32) {
        self.args_alloc = Some((list_ptr, string_data_ptr));
    }

    /// Clear pre-allocated memory for arguments
    pub fn clear_args_alloc(&mut self) {
        self.args_alloc = None;
    }

    /// Get pre-allocated memory pointers for arguments
    pub fn get_args_alloc(&self) -> Option<(u32, u32)> {
        self.args_alloc
    }

    /// Get the capabilities for this dispatcher
    pub fn capabilities(&self) -> &WasiCapabilities {
        &self.capabilities
    }

    /// Get the command-line arguments
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Strip version suffix from interface name
    /// e.g., "wasi:clocks/wall-clock@0.2.4" -> "wasi:clocks/wall-clock"
    fn strip_version(interface: &str) -> &str {
        if let Some(idx) = interface.find('@') {
            &interface[..idx]
        } else {
            interface
        }
    }

    /// Main dispatch method for WASI calls using Component Model values
    ///
    /// This is the primary entry point for Component Model WASI calls.
    /// Values are already lifted (strings are String, lists are Vec, etc.)
    pub fn dispatch(
        &mut self,
        interface: &str,
        function: &str,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        let base_interface = Self::strip_version(interface);

        #[cfg(feature = "std")]
        eprintln!("[WASI-DISPATCH] {}::{} with {} args", base_interface, function, args.len());

        match (base_interface, function) {
            // ================================================================
            // wasi:cli/* - CLI interfaces
            // ================================================================

            ("wasi:cli/stdout", "get-stdout") => {
                Ok(vec![Value::U32(self.stdout_handle)])
            }

            ("wasi:cli/stderr", "get-stderr") => {
                Ok(vec![Value::U32(self.stderr_handle)])
            }

            ("wasi:cli/environment", "get-arguments") => {
                // Get args - prefer local args, fall back to global thread-local args
                #[cfg(feature = "std")]
                let args_to_use: Vec<String> = if self.args.is_empty() {
                    get_global_wasi_args()
                } else {
                    self.args.clone()
                };
                #[cfg(not(feature = "std"))]
                let args_to_use = &self.args;

                let arg_values: Vec<Value> = args_to_use
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect();

                #[cfg(feature = "std")]
                eprintln!("[WASI-DISPATCH] get-arguments returning {} args: {:?}", args_to_use.len(), args_to_use);

                Ok(vec![Value::List(arg_values)])
            }

            ("wasi:cli/environment", "get-environment") => {
                // Return the stored environment variables as a list of (string, string) tuples
                let env_values: Vec<Value> = self.env_vars
                    .iter()
                    .map(|(k, v)| Value::Tuple(vec![Value::String(k.clone()), Value::String(v.clone())]))
                    .collect();

                #[cfg(feature = "std")]
                eprintln!("[WASI-DISPATCH] get-environment returning {} vars", self.env_vars.len());

                Ok(vec![Value::List(env_values)])
            }

            ("wasi:cli/environment", "initial-cwd") => {
                // Return None for initial working directory
                #[cfg(feature = "std")]
                {
                    Ok(vec![Value::Option(None)])
                }
                #[cfg(not(feature = "std"))]
                {
                    Ok(vec![Value::Option(None)])
                }
            }

            ("wasi:cli/exit", "exit") => {
                // Extract exit status from args
                let success = match args.first() {
                    Some(Value::S32(0)) => true,
                    Some(Value::U32(0)) => true,
                    Some(Value::Bool(true)) => true,
                    _ => false,
                };

                #[cfg(feature = "std")]
                {
                    std::process::exit(if success { 0 } else { 1 });
                }
                #[cfg(not(feature = "std"))]
                {
                    if success {
                        Ok(vec![])
                    } else {
                        Err(Error::runtime_error("Component exited with failure"))
                    }
                }
            }

            // ================================================================
            // wasi:clocks/* - Clock interfaces
            // ================================================================

            #[cfg(feature = "wasi-clocks")]
            ("wasi:clocks/wall-clock", "now") => {
                // Check capability
                if !self.capabilities.clocks.realtime_access {
                    return Err(Error::wasi_permission_denied("Wall clock access denied"));
                }
                wasi_wall_clock_now(&mut (), args)
            }

            #[cfg(feature = "wasi-clocks")]
            ("wasi:clocks/wall-clock", "resolution") => {
                if !self.capabilities.clocks.realtime_access {
                    return Err(Error::wasi_permission_denied("Wall clock access denied"));
                }
                wasi_wall_clock_resolution(&mut (), args)
            }

            #[cfg(feature = "wasi-clocks")]
            ("wasi:clocks/monotonic-clock", "now") => {
                if !self.capabilities.clocks.monotonic_access {
                    return Err(Error::wasi_permission_denied("Monotonic clock access denied"));
                }
                wasi_monotonic_clock_now(&mut (), args)
            }

            #[cfg(feature = "wasi-clocks")]
            ("wasi:clocks/monotonic-clock", "resolution") => {
                if !self.capabilities.clocks.monotonic_access {
                    return Err(Error::wasi_permission_denied("Monotonic clock access denied"));
                }
                wasi_monotonic_clock_resolution(&mut (), args)
            }

            // ================================================================
            // wasi:io/* - I/O interfaces
            // ================================================================

            #[cfg(feature = "wasi-io")]
            ("wasi:io/streams", "[method]output-stream.blocking-write-and-flush") |
            ("wasi:io/streams", "output-stream.blocking-write-and-flush") => {
                if !self.capabilities.io.stdout_access {
                    return Err(Error::wasi_permission_denied("Stream write access denied"));
                }

                // Args: [handle, list<u8>]
                // The wasi_stream_write expects args to already be in the right format
                wasi_stream_write(&mut (), args)
            }

            #[cfg(feature = "wasi-io")]
            ("wasi:io/streams", "[method]output-stream.blocking-flush") |
            ("wasi:io/streams", "output-stream.blocking-flush") => {
                wasi_stream_flush(&mut (), args)
            }

            #[cfg(feature = "wasi-io")]
            ("wasi:io/streams", "[method]output-stream.check-write") |
            ("wasi:io/streams", "output-stream.check-write") => {
                wasi_stream_check_write(&mut (), args)
            }

            // Resource drops - no-op, just consume the handle
            ("wasi:io/streams", "[resource-drop]output-stream") |
            ("wasi:io/streams", "[resource-drop]input-stream") |
            ("wasi:io/error", "[resource-drop]error") => {
                Ok(vec![])
            }

            ("wasi:io/error", "[method]error.to-debug-string") => {
                #[cfg(feature = "std")]
                {
                    Ok(vec![Value::String("error".to_string())])
                }
                #[cfg(not(feature = "std"))]
                {
                    // For no_std, return a simplified value
                    Ok(vec![Value::U32(0)]) // Empty string representation
                }
            }

            // ================================================================
            // Unknown function - return error
            // ================================================================
            _ => {
                #[cfg(feature = "std")]
                eprintln!("[WASI-DISPATCH] Unknown: {}::{}", base_interface, function);

                Err(Error::runtime_not_implemented(
                    "Unknown WASI function"
                ))
            }
        }
    }

    /// Dispatch with raw core WASM values and memory access
    ///
    /// This interface is for cases where the canonical ABI lifting hasn't
    /// happened yet - values are raw i32/i64 and complex types are ptr+len
    /// in memory.
    ///
    /// # Arguments
    /// * `interface` - WASI interface name (e.g., "wasi:cli/stdout@0.2.4")
    /// * `function` - Function name (e.g., "get-stdout")
    /// * `args` - Raw values from the WASM stack
    /// * `memory` - Linear memory for reading/writing complex types
    pub fn dispatch_core(
        &mut self,
        interface: &str,
        function: &str,
        args: Vec<wrt_foundation::values::Value>,
        memory: Option<&mut [u8]>,
    ) -> Result<Vec<wrt_foundation::values::Value>> {
        use wrt_foundation::values::Value as CoreValue;

        let base_interface = Self::strip_version(interface);

        match (base_interface, function) {
            // Simple functions that don't need memory
            ("wasi:cli/stdout", "get-stdout") => {
                Ok(vec![CoreValue::I32(self.stdout_handle as i32)])
            }

            ("wasi:cli/stderr", "get-stderr") => {
                Ok(vec![CoreValue::I32(self.stderr_handle as i32)])
            }

            // Environment functions
            ("wasi:cli/environment", "get-arguments") => {
                // Canonical ABI for list<string> return:
                // - First argument is the RETURN POINTER where we write (list_ptr, list_len)
                // - We need to allocate space for the list elements and string data
                // - Write element pointers array at one location
                // - Write string data at another location
                // - Write (element_array_ptr, count) to the return pointer

                // Get args - prefer local args, fall back to global args
                #[cfg(feature = "std")]
                let args_to_use: Vec<String> = if self.args.is_empty() {
                    get_global_wasi_args()
                } else {
                    self.args.clone()
                };
                #[cfg(not(feature = "std"))]
                let args_to_use = self.args.clone();

                // Extract return pointer from args (first argument in canonical ABI lowering)
                let retptr = match args.first() {
                    Some(CoreValue::I32(ptr)) => *ptr as u32,
                    _ => {
                        #[cfg(feature = "std")]
                        eprintln!("[WASI-DISPATCH-CORE] get-arguments: no return pointer provided");
                        return Ok(vec![]);
                    }
                };

                #[cfg(feature = "std")]
                eprintln!("[WASI-DISPATCH-CORE] get-arguments: {} args to use, args={:?}, retptr=0x{:x}, args_alloc={:?}",
                         args_to_use.len(), args_to_use, retptr, self.args_alloc);

                if let Some(mem) = memory {
                    #[cfg(feature = "std")]
                    eprintln!("[WASI-DISPATCH-CORE] Memory available, size={} bytes", mem.len());

                    if args_to_use.is_empty() {
                        // Empty list: write (0, 0) to return pointer
                        let retptr_usize = retptr as usize;
                        if retptr_usize + 8 <= mem.len() {
                            mem[retptr_usize..retptr_usize + 4].copy_from_slice(&0u32.to_le_bytes());
                            mem[retptr_usize + 4..retptr_usize + 8].copy_from_slice(&0u32.to_le_bytes());
                        }
                        return Ok(vec![]);
                    }

                    // Use pre-allocated memory from cabi_realloc if available,
                    // otherwise fall back to stack-relative allocation
                    let (list_ptr, mut string_data_ptr) = if let Some((alloc_list, alloc_strings)) = self.args_alloc {
                        #[cfg(feature = "std")]
                        eprintln!("[WASI-DISPATCH-CORE] Using cabi_realloc memory: list_ptr=0x{:x}, string_ptr=0x{:x}",
                                 alloc_list, alloc_strings);
                        (alloc_list, alloc_strings)
                    } else {
                        // Fallback: Use memory just after the return pointer (which is on the stack)
                        // WARNING: This may be overwritten by component execution!
                        let data_base: u32 = retptr + 16;
                        #[cfg(feature = "std")]
                        eprintln!("[WASI-DISPATCH-CORE] FALLBACK: Using stack-relative data_base=0x{:x}", data_base);
                        let list_ptr = data_base;
                        let string_data_ptr = list_ptr + (args_to_use.len() as u32 * 8);
                        (list_ptr, string_data_ptr)
                    };

                    // First pass: write string data and collect pointers
                    let mut string_entries: Vec<(u32, u32)> = Vec::new();
                    for arg in &args_to_use {
                        let bytes = arg.as_bytes();
                        let len = bytes.len() as u32;

                        // Bounds check
                        if (string_data_ptr as usize + bytes.len()) > mem.len() {
                            #[cfg(feature = "std")]
                            eprintln!("[WASI-DISPATCH-CORE] Not enough memory for arg data");
                            // Write empty list
                            let retptr_usize = retptr as usize;
                            if retptr_usize + 8 <= mem.len() {
                                mem[retptr_usize..retptr_usize + 4].copy_from_slice(&0u32.to_le_bytes());
                                mem[retptr_usize + 4..retptr_usize + 8].copy_from_slice(&0u32.to_le_bytes());
                            }
                            return Ok(vec![]);
                        }

                        // Write string bytes
                        mem[string_data_ptr as usize..(string_data_ptr as usize + bytes.len())].copy_from_slice(bytes);
                        #[cfg(feature = "std")]
                        eprintln!("[WASI-DISPATCH-CORE] Wrote string '{}' ({} bytes) at 0x{:x}", arg, bytes.len(), string_data_ptr);

                        string_entries.push((string_data_ptr, len));
                        string_data_ptr += len;
                        // Align to 8 bytes
                        string_data_ptr = (string_data_ptr + 7) & !7;
                    }

                    // Second pass: write (ptr, len) array at list_ptr
                    for (i, (ptr, len)) in string_entries.iter().enumerate() {
                        let offset = list_ptr as usize + i * 8;
                        if offset + 8 > mem.len() {
                            return Ok(vec![]);
                        }
                        mem[offset..offset + 4].copy_from_slice(&ptr.to_le_bytes());
                        mem[offset + 4..offset + 8].copy_from_slice(&len.to_le_bytes());
                        #[cfg(feature = "std")]
                        eprintln!("[WASI-DISPATCH-CORE] list[{}]: ptr=0x{:x}, len={}", i, ptr, len);
                    }

                    // Write (list_ptr, count) to the return pointer
                    let retptr_usize = retptr as usize;
                    if retptr_usize + 8 <= mem.len() {
                        mem[retptr_usize..retptr_usize + 4].copy_from_slice(&list_ptr.to_le_bytes());
                        mem[retptr_usize + 4..retptr_usize + 8].copy_from_slice(&(args_to_use.len() as u32).to_le_bytes());
                        #[cfg(feature = "std")]
                        eprintln!("[WASI-DISPATCH-CORE] Wrote to retptr 0x{:x}: list_ptr=0x{:x}, count={}",
                                 retptr, list_ptr, args_to_use.len());
                    }

                    // Canonical ABI with return pointer returns void (no return values)
                    Ok(vec![])
                } else {
                    #[cfg(feature = "std")]
                    eprintln!("[WASI-DISPATCH-CORE] No memory available for get-arguments");
                    Ok(vec![])
                }
            }

            ("wasi:cli/environment", "get-environment") => {
                // Similar to get-arguments but for env vars
                // For now, return empty list
                Ok(vec![CoreValue::I32(0), CoreValue::I32(0)])
            }

            // Clock functions
            #[cfg(feature = "wasi-clocks")]
            ("wasi:clocks/wall-clock", "now") => {
                use wrt_platform::time::PlatformTime;

                if !self.capabilities.clocks.realtime_access {
                    return Err(Error::wasi_permission_denied("Wall clock access denied"));
                }

                let total_ns = PlatformTime::wall_clock_ns()
                    .map_err(|_| Error::wasi_capability_unavailable("Wall clock not available"))?;

                let seconds = total_ns / 1_000_000_000;
                let nanoseconds = (total_ns % 1_000_000_000) as u32;

                // Return as two values that will be composed into the datetime record
                Ok(vec![
                    CoreValue::I64(seconds as i64),
                    CoreValue::I32(nanoseconds as i32),
                ])
            }

            #[cfg(feature = "wasi-clocks")]
            ("wasi:clocks/wall-clock", "resolution") => {
                // 1 nanosecond resolution
                Ok(vec![
                    CoreValue::I64(0), // seconds
                    CoreValue::I32(1), // nanoseconds
                ])
            }

            #[cfg(feature = "wasi-clocks")]
            ("wasi:clocks/monotonic-clock", "now") => {
                use wrt_platform::time::PlatformTime;

                if !self.capabilities.clocks.monotonic_access {
                    return Err(Error::wasi_permission_denied("Monotonic clock access denied"));
                }

                let ns = PlatformTime::monotonic_ns();
                Ok(vec![CoreValue::I64(ns as i64)])
            }

            // I/O functions that need memory access
            #[cfg(feature = "wasi-io")]
            ("wasi:io/streams", "[method]output-stream.blocking-write-and-flush") |
            ("wasi:io/streams", "output-stream.blocking-write-and-flush") => {
                #[cfg(feature = "std")]
                eprintln!("[WRITE-DISPATCH] blocking-write-and-flush: args={:?}, has_memory={}", args, memory.is_some());

                if !self.capabilities.io.stdout_access {
                    return Err(Error::wasi_permission_denied("Stream write access denied"));
                }

                // Args: handle (i32), data_ptr (i32), data_len (i32)
                if args.len() < 3 {
                    return Err(Error::wasi_invalid_argument("blocking-write-and-flush requires 3 args"));
                }

                let handle = match &args[0] {
                    CoreValue::I32(h) => *h as u32,
                    _ => return Err(Error::wasi_invalid_argument("Invalid handle type")),
                };

                let data_ptr = match &args[1] {
                    CoreValue::I32(p) => *p as u32,
                    _ => return Err(Error::wasi_invalid_argument("Invalid ptr type")),
                };

                let data_len = match &args[2] {
                    CoreValue::I32(l) => *l as u32,
                    _ => return Err(Error::wasi_invalid_argument("Invalid len type")),
                };

                // Read data from memory
                let mem = memory.ok_or_else(||
                    Error::wasi_capability_unavailable("Memory required for write"))?;

                #[cfg(feature = "std")]
                eprintln!("[WRITE-DISPATCH] handle={}, data_ptr=0x{:x}, data_len={}, mem_size={}",
                         handle, data_ptr, data_len, mem.len());

                let start = data_ptr as usize;
                let end = start + data_len as usize;
                if end > mem.len() {
                    #[cfg(feature = "std")]
                    eprintln!("[WRITE-DISPATCH] ERROR: out of bounds - end=0x{:x} > mem_len=0x{:x}", end, mem.len());
                    return Err(Error::wasi_invalid_argument("Write data out of bounds"));
                }

                let data = &mem[start..end];

                #[cfg(feature = "std")]
                eprintln!("[WRITE-DISPATCH] Data to write ({} bytes): {:?}", data.len(),
                         String::from_utf8_lossy(&data[..data.len().min(100)]));

                // Write to stdout or stderr
                #[cfg(feature = "std")]
                {
                    use std::io::{self, Write};

                    let result = if handle == self.stdout_handle {
                        eprintln!("[WRITE-DISPATCH] Writing to STDOUT");
                        io::stdout().write_all(data).and_then(|_| io::stdout().flush())
                    } else if handle == self.stderr_handle {
                        eprintln!("[WRITE-DISPATCH] Writing to STDERR");
                        io::stderr().write_all(data).and_then(|_| io::stderr().flush())
                    } else {
                        eprintln!("[WRITE-DISPATCH] Invalid handle: {} (stdout={}, stderr={})",
                                 handle, self.stdout_handle, self.stderr_handle);
                        return Err(Error::wasi_invalid_fd("Invalid stream handle"));
                    };

                    match result {
                        Ok(()) => {
                            eprintln!("[WRITE-DISPATCH] Write succeeded!");
                            Ok(vec![CoreValue::I32(0)]) // Success
                        }
                        Err(e) => {
                            eprintln!("[WRITE-DISPATCH] Write failed: {:?}", e);
                            Ok(vec![CoreValue::I32(1)]) // Error
                        }
                    }
                }
                #[cfg(not(feature = "std"))]
                {
                    // In no_std, we can't write to stdout
                    Ok(vec![CoreValue::I32(0)])
                }
            }

            #[cfg(feature = "wasi-io")]
            ("wasi:io/streams", "[method]output-stream.blocking-flush") |
            ("wasi:io/streams", "output-stream.blocking-flush") => {
                #[cfg(feature = "std")]
                {
                    use std::io::{self, Write};
                    let _ = io::stdout().flush();
                    let _ = io::stderr().flush();
                }
                Ok(vec![CoreValue::I32(0)])
            }

            // Resource drops
            ("wasi:io/streams", "[resource-drop]output-stream") |
            ("wasi:io/streams", "[resource-drop]input-stream") |
            ("wasi:io/error", "[resource-drop]error") => {
                Ok(vec![])
            }

            // CLI exit
            ("wasi:cli/exit", "exit") => {
                let status_ok = match args.first() {
                    Some(CoreValue::I32(0)) => true,
                    _ => false,
                };

                #[cfg(feature = "std")]
                {
                    std::process::exit(if status_ok { 0 } else { 1 });
                }
                #[cfg(not(feature = "std"))]
                {
                    if status_ok {
                        Ok(vec![])
                    } else {
                        Err(Error::runtime_error("Component exited with failure"))
                    }
                }
            }

            _ => {
                #[cfg(feature = "std")]
                eprintln!("[WASI-DISPATCH-CORE] Unknown: {}::{}", base_interface, function);

                Err(Error::runtime_not_implemented("Unknown WASI function"))
            }
        }
    }

    /// Two-phase dispatch for functions that may need memory allocation.
    ///
    /// This method implements the spec-compliant approach where:
    /// 1. If the function doesn't need allocation, it completes immediately
    /// 2. If allocation is needed (e.g., returning list<string>), it returns
    ///    a NeedsAllocation result with the size/alignment requirements
    /// 3. The caller then calls cabi_realloc, then calls complete_allocation
    ///
    /// This allows the HOST to call cabi_realloc as required by the Canonical ABI.
    pub fn dispatch_core_v2(
        &mut self,
        interface: &str,
        function: &str,
        args: Vec<wrt_foundation::values::Value>,
        _memory: Option<&mut [u8]>,
    ) -> Result<DispatchResult> {
        use wrt_foundation::values::Value as CoreValue;

        let base_interface = Self::strip_version(interface);

        match (base_interface, function) {
            // Simple functions that complete immediately
            ("wasi:cli/stdout", "get-stdout") => {
                Ok(DispatchResult::Complete(vec![CoreValue::I32(self.stdout_handle as i32)]))
            }

            ("wasi:cli/stderr", "get-stderr") => {
                Ok(DispatchResult::Complete(vec![CoreValue::I32(self.stderr_handle as i32)]))
            }

            // get-arguments needs allocation for list<string>
            ("wasi:cli/environment", "get-arguments") => {
                // Get args - prefer local args, fall back to global args
                #[cfg(feature = "std")]
                let args_to_use: Vec<String> = if self.args.is_empty() {
                    get_global_wasi_args()
                } else {
                    self.args.clone()
                };
                #[cfg(not(feature = "std"))]
                let args_to_use = self.args.clone();

                // Extract return pointer from args
                let retptr = match args.first() {
                    Some(CoreValue::I32(ptr)) => *ptr as u32,
                    _ => {
                        #[cfg(feature = "std")]
                        eprintln!("[WASI-V2] get-arguments: no return pointer provided");
                        return Ok(DispatchResult::Complete(vec![]));
                    }
                };

                #[cfg(feature = "std")]
                eprintln!("[WASI-V2] get-arguments: {} args, retptr=0x{:x}, has_alloc={}",
                         args_to_use.len(), retptr, self.args_alloc.is_some());

                // If no args, complete immediately with empty list
                if args_to_use.is_empty() {
                    return Ok(DispatchResult::Complete(vec![]));
                }

                // If we already have allocated memory, we can complete
                if self.args_alloc.is_some() {
                    return Ok(DispatchResult::Complete(vec![]));
                }

                // Calculate required allocation size:
                // - list array: args.len() * 8 bytes (ptr + len for each string)
                // - string data: sum of all string bytes, each aligned to 8 bytes
                let list_size = args_to_use.len() * 8;
                let mut string_total: usize = 0;
                for arg in &args_to_use {
                    let len = arg.len();
                    // Align each string to 8 bytes
                    string_total += (len + 7) & !7;
                }
                let total_size = list_size + string_total;

                #[cfg(feature = "std")]
                eprintln!("[WASI-V2] get-arguments needs allocation: list_size={}, string_total={}, total={}",
                         list_size, string_total, total_size);

                Ok(DispatchResult::NeedsAllocation {
                    request: AllocationRequest {
                        size: total_size as u32,
                        align: 8,
                        purpose: "list<string> for get-arguments",
                    },
                    args_to_write: args_to_use,
                    retptr,
                })
            }

            // All other functions - delegate to regular dispatch_core
            _ => {
                // For functions that don't need allocation, use regular dispatch
                // This is a fallback - most functions complete immediately
                #[cfg(feature = "std")]
                eprintln!("[WASI-V2] Delegating {}::{} to dispatch_core", base_interface, function);

                // We can't forward memory here since we consumed it, so return error for now
                // In practice, this path shouldn't be hit for functions needing memory
                Err(Error::runtime_not_implemented(
                    "dispatch_core_v2 fallback not implemented for this function"
                ))
            }
        }
    }

    /// Complete an allocation request by writing data to allocated memory.
    ///
    /// Called after the caller has used cabi_realloc to allocate memory.
    /// Writes the list<string> data to the allocated region and updates
    /// the return pointer.
    ///
    /// # Arguments
    /// * `allocated_ptr` - Pointer returned by cabi_realloc
    /// * `args_to_write` - The strings to write
    /// * `retptr` - Where to write (list_ptr, len)
    /// * `memory` - Linear memory to write to
    pub fn complete_allocation(
        &mut self,
        allocated_ptr: u32,
        args_to_write: &[String],
        retptr: u32,
        memory: &mut [u8],
    ) -> Result<()> {
        #[cfg(feature = "std")]
        eprintln!("[WASI-COMPLETE] Writing {} args to ptr=0x{:x}, retptr=0x{:x}",
                 args_to_write.len(), allocated_ptr, retptr);

        if args_to_write.is_empty() {
            // Write empty list
            let retptr_usize = retptr as usize;
            if retptr_usize + 8 <= memory.len() {
                memory[retptr_usize..retptr_usize + 4].copy_from_slice(&0u32.to_le_bytes());
                memory[retptr_usize + 4..retptr_usize + 8].copy_from_slice(&0u32.to_le_bytes());
            }
            return Ok(());
        }

        // Layout: [list_ptr_array][string_data...]
        // list_ptr_array: N * 8 bytes (ptr, len pairs)
        // string_data: strings packed with 8-byte alignment
        let list_ptr = allocated_ptr;
        let list_array_size = args_to_write.len() * 8;
        let mut string_data_ptr = allocated_ptr + list_array_size as u32;

        // First pass: write string data and collect pointers
        let mut string_entries: Vec<(u32, u32)> = Vec::new();
        for arg in args_to_write {
            let bytes = arg.as_bytes();
            let len = bytes.len() as u32;

            // Bounds check
            if (string_data_ptr as usize + bytes.len()) > memory.len() {
                return Err(Error::memory_out_of_bounds(
                    "Not enough memory for string data"
                ));
            }

            // Write string bytes
            memory[string_data_ptr as usize..(string_data_ptr as usize + bytes.len())]
                .copy_from_slice(bytes);

            #[cfg(feature = "std")]
            eprintln!("[WASI-COMPLETE] Wrote '{}' ({} bytes) at 0x{:x}",
                     arg, bytes.len(), string_data_ptr);

            string_entries.push((string_data_ptr, len));
            string_data_ptr += len;
            // Align to 8 bytes
            string_data_ptr = (string_data_ptr + 7) & !7;
        }

        // Second pass: write (ptr, len) array at list_ptr
        for (i, (ptr, len)) in string_entries.iter().enumerate() {
            let offset = list_ptr as usize + i * 8;
            if offset + 8 > memory.len() {
                return Err(Error::memory_out_of_bounds(
                    "Not enough memory for list array"
                ));
            }
            memory[offset..offset + 4].copy_from_slice(&ptr.to_le_bytes());
            memory[offset + 4..offset + 8].copy_from_slice(&len.to_le_bytes());

            #[cfg(feature = "std")]
            eprintln!("[WASI-COMPLETE] list[{}]: ptr=0x{:x}, len={}", i, ptr, len);
        }

        // Write (list_ptr, count) to the return pointer
        let retptr_usize = retptr as usize;
        if retptr_usize + 8 <= memory.len() {
            memory[retptr_usize..retptr_usize + 4].copy_from_slice(&list_ptr.to_le_bytes());
            memory[retptr_usize + 4..retptr_usize + 8]
                .copy_from_slice(&(args_to_write.len() as u32).to_le_bytes());

            #[cfg(feature = "std")]
            eprintln!("[WASI-COMPLETE] Wrote to retptr 0x{:x}: list_ptr=0x{:x}, count={}",
                     retptr, list_ptr, args_to_write.len());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_creation() -> Result<()> {
        let caps = WasiCapabilities::minimal()?;
        let dispatcher = WasiDispatcher::new(caps)?;
        assert_eq!(dispatcher.stdout_handle, 1);
        assert_eq!(dispatcher.stderr_handle, 2);
        Ok(())
    }

    #[test]
    fn test_get_stdout() -> Result<()> {
        let mut dispatcher = WasiDispatcher::with_defaults()?;
        let result = dispatcher.dispatch("wasi:cli/stdout@0.2.4", "get-stdout", vec![])?;
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::U32(1)));
        Ok(())
    }

    #[test]
    fn test_get_stderr() -> Result<()> {
        let mut dispatcher = WasiDispatcher::with_defaults()?;
        let result = dispatcher.dispatch("wasi:cli/stderr@0.2.4", "get-stderr", vec![])?;
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::U32(2)));
        Ok(())
    }

    #[test]
    fn test_strip_version() {
        assert_eq!(WasiDispatcher::strip_version("wasi:cli/stdout@0.2.4"), "wasi:cli/stdout");
        assert_eq!(WasiDispatcher::strip_version("wasi:cli/stdout"), "wasi:cli/stdout");
    }

    #[cfg(feature = "wasi-clocks")]
    #[test]
    fn test_wall_clock_now() -> Result<()> {
        let mut dispatcher = WasiDispatcher::with_defaults()?;
        let result = dispatcher.dispatch("wasi:clocks/wall-clock@0.2.4", "now", vec![])?;
        assert_eq!(result.len(), 1);
        // Should return a Tuple with (seconds, nanoseconds)
        if let Value::Tuple(parts) = &result[0] {
            assert_eq!(parts.len(), 2);
        } else {
            panic!("Expected tuple result");
        }
        Ok(())
    }
}
