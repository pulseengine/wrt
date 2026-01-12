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

// Import tracing utilities for structured logging
#[cfg(feature = "tracing")]
use wrt_foundation::tracing::{trace, warn};

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
#[cfg(feature = "std")]
use wrt_foundation::MemoryAccessor;

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
/// Falls back to std::env::args() if no global args are set
#[cfg(feature = "std")]
pub fn get_global_wasi_args() -> Vec<String> {
    // Per CLAUDE.md: fail loud and early. A poisoned mutex means a thread panicked
    // while holding the lock - the data may be in an inconsistent state.
    let global_args = GLOBAL_WASI_ARGS
        .lock()
        .expect("GLOBAL_WASI_ARGS mutex poisoned - a thread panicked while holding the lock")
        .clone();
    if global_args.is_empty() {
        // Fall back to actual process args
        std::env::args().collect()
    } else {
        global_args
    }
}

/// Get the global WASI environment variables
/// Falls back to std::env::vars() if no global env vars are set
#[cfg(feature = "std")]
pub fn get_global_wasi_env() -> Vec<(String, String)> {
    // Per CLAUDE.md: fail loud and early. A poisoned mutex means a thread panicked
    // while holding the lock - the data may be in an inconsistent state.
    let global_env = GLOBAL_WASI_ENV
        .lock()
        .expect("GLOBAL_WASI_ENV mutex poisoned - a thread panicked while holding the lock")
        .clone();
    if global_env.is_empty() {
        // Fall back to actual process environment
        std::env::vars().collect()
    } else {
        global_env
    }
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

#[cfg(all(feature = "wasi-sockets", feature = "std"))]
use crate::preview2::sockets::{
    wasi_tcp_create,
    wasi_tcp_connect,
    wasi_tcp_bind,
    wasi_tcp_listen,
    wasi_tcp_accept,
    wasi_tcp_send,
    wasi_tcp_recv,
    wasi_tcp_shutdown,
    wasi_udp_create,
    wasi_udp_bind,
    wasi_udp_send,
    wasi_udp_recv,
    wasi_resolve_addresses,
};

#[cfg(feature = "wasi-random")]
use crate::preview2::random::{
    wasi_get_random_bytes,
    wasi_get_random_u64,
    wasi_get_insecure_random_bytes,
    wasi_get_insecure_random_u64,
};

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::path::PathBuf;

/// Type of file descriptor in the WASI filesystem
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub enum FileDescriptorType {
    /// Standard output stream
    Stdout,
    /// Standard error stream
    Stderr,
    /// Standard input stream
    Stdin,
    /// Pre-opened directory with its path
    PreopenDirectory(PathBuf),
    /// Regular file with its path
    RegularFile(PathBuf),
}

/// A file descriptor entry in the descriptor table
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct FileDescriptorEntry {
    /// Type of this descriptor
    pub fd_type: FileDescriptorType,
    /// Whether this descriptor can be read from
    pub read: bool,
    /// Whether this descriptor can be written to
    pub write: bool,
}

/// WASI Dispatcher - unified entry point for all WASI function calls
pub struct WasiDispatcher {
    /// WASI capabilities for this dispatcher
    capabilities: WasiCapabilities,
    /// Next available resource handle
    next_handle: u32,
    /// Stdout handle (pre-allocated)
    stdout_handle: u32,
    /// Stderr handle (pre-allocated)
    stderr_handle: u32,
    /// Stdin handle (pre-allocated)
    #[allow(dead_code)]
    stdin_handle: u32,
    /// Command-line arguments
    args: Vec<String>,
    /// Environment variables as (key, value) pairs
    env_vars: Vec<(String, String)>,
    /// Pre-allocated memory for args (list_ptr, string_data_ptr)
    /// Set by the engine after calling cabi_realloc
    args_alloc: Option<(u32, u32)>,
    /// File descriptor table (maps handle -> descriptor info)
    #[cfg(feature = "std")]
    fd_table: HashMap<u32, FileDescriptorEntry>,
    /// Pre-opened directories (list of (handle, path) pairs)
    #[cfg(feature = "std")]
    preopens: Vec<(u32, PathBuf)>,
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
        #[cfg(feature = "std")]
        let mut fd_table = HashMap::new();

        // Pre-populate fd_table with stdin/stdout/stderr
        #[cfg(feature = "std")]
        {
            fd_table.insert(0, FileDescriptorEntry {
                fd_type: FileDescriptorType::Stdin,
                read: true,
                write: false,
            });
            fd_table.insert(1, FileDescriptorEntry {
                fd_type: FileDescriptorType::Stdout,
                read: false,
                write: true,
            });
            fd_table.insert(2, FileDescriptorEntry {
                fd_type: FileDescriptorType::Stderr,
                read: false,
                write: true,
            });
        }

        Ok(Self {
            capabilities,
            next_handle: 3, // 0=stdin, 1=stdout, 2=stderr, start user handles at 3
            stdin_handle: 0,
            stdout_handle: 1,
            stderr_handle: 2,
            args: Vec::new(),
            env_vars: Vec::new(),
            args_alloc: None,
            #[cfg(feature = "std")]
            fd_table,
            #[cfg(feature = "std")]
            preopens: Vec::new(),
        })
    }

    /// Create a dispatcher with default (full) capabilities
    pub fn with_defaults() -> Result<Self> {
        Self::new(WasiCapabilities::system_utility()?)
    }

    /// Create a dispatcher with arguments
    pub fn with_args(capabilities: WasiCapabilities, args: Vec<String>) -> Result<Self> {
        let mut dispatcher = Self::new(capabilities)?;
        dispatcher.args = args;
        Ok(dispatcher)
    }

    /// Create a dispatcher with arguments and environment variables
    pub fn with_args_and_env(
        capabilities: WasiCapabilities,
        args: Vec<String>,
        env_vars: Vec<(String, String)>,
    ) -> Result<Self> {
        let mut dispatcher = Self::new(capabilities)?;
        dispatcher.args = args;
        dispatcher.env_vars = env_vars;
        Ok(dispatcher)
    }

    /// Add a pre-opened directory to the WASI sandbox
    ///
    /// Returns the file descriptor handle for the directory
    #[cfg(feature = "std")]
    pub fn add_preopen(&mut self, path: impl Into<PathBuf>) -> u32 {
        let path = path.into();
        let handle = self.next_handle;
        self.next_handle += 1;

        // Add to fd_table
        self.fd_table.insert(handle, FileDescriptorEntry {
            fd_type: FileDescriptorType::PreopenDirectory(path.clone()),
            read: self.capabilities.filesystem.read_access,
            write: self.capabilities.filesystem.write_access,
        });

        // Add to preopens list
        self.preopens.push((handle, path));

        handle
    }

    /// Get the list of pre-opened directories
    #[cfg(feature = "std")]
    pub fn get_preopens(&self) -> &[(u32, PathBuf)] {
        &self.preopens
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

        #[cfg(feature = "tracing")]
        trace!(interface = %base_interface, function = %function, arg_count = args.len(), "WASI dispatch");

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

                #[cfg(feature = "tracing")]
                trace!(arg_count = args_to_use.len(), args = ?args_to_use, "get-arguments returning");

                Ok(vec![Value::List(arg_values)])
            }

            ("wasi:cli/environment", "get-environment") => {
                // Get env vars - prefer local env_vars, fall back to global then process env
                #[cfg(feature = "std")]
                let env_to_use: Vec<(String, String)> = if self.env_vars.is_empty() {
                    get_global_wasi_env()
                } else {
                    self.env_vars.clone()
                };
                #[cfg(not(feature = "std"))]
                let env_to_use = &self.env_vars;

                let env_values: Vec<Value> = env_to_use
                    .iter()
                    .map(|(k, v)| Value::Tuple(vec![Value::String(k.clone()), Value::String(v.clone())]))
                    .collect();

                #[cfg(feature = "tracing")]
                trace!(var_count = env_to_use.len(), "get-environment returning");

                Ok(vec![Value::List(env_values)])
            }

            ("wasi:cli/environment", "initial-cwd") => {
                // Return the actual current working directory
                #[cfg(feature = "std")]
                {
                    if let Ok(cwd) = std::env::current_dir() {
                        let cwd_string = cwd.to_string_lossy().to_string();
                        #[cfg(feature = "tracing")]
                        trace!(cwd = %cwd_string, "initial-cwd returning");
                        Ok(vec![Value::Option(Some(Box::new(Value::String(cwd_string))))])
                    } else {
                        Ok(vec![Value::Option(None)])
                    }
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
            // wasi:filesystem/* - Filesystem interfaces
            // ================================================================

            #[cfg(all(feature = "wasi-filesystem", feature = "std"))]
            ("wasi:filesystem/preopens", "get-directories") => {
                // Check filesystem capability
                if !self.capabilities.filesystem.directory_access {
                    return Err(Error::wasi_permission_denied("Filesystem access denied"));
                }

                // Return list of (descriptor, path) tuples
                let dirs: Vec<Value> = self.preopens
                    .iter()
                    .map(|(handle, path)| {
                        Value::Tuple(vec![
                            Value::U32(*handle),
                            Value::String(path.to_string_lossy().to_string()),
                        ])
                    })
                    .collect();

                #[cfg(feature = "tracing")]
                trace!(preopen_count = dirs.len(), "get-directories returning");
                Ok(vec![Value::List(dirs)])
            }

            #[cfg(all(feature = "wasi-filesystem", feature = "std"))]
            ("wasi:filesystem/types", "[method]descriptor.open-at") => {
                // Check filesystem capability
                if !self.capabilities.filesystem.read_access && !self.capabilities.filesystem.write_access {
                    return Err(Error::wasi_permission_denied("Filesystem access denied"));
                }

                // Args: [descriptor_handle, path_flags, path, open_flags, modes]
                // For now, simplified: [descriptor_handle, path]
                let base_handle = match args.first() {
                    Some(Value::U32(h)) => *h,
                    _ => return Err(Error::wasi_invalid_argument("Invalid descriptor")),
                };

                let path = match args.get(1) {
                    Some(Value::String(s)) => s.clone(),
                    _ => return Err(Error::wasi_invalid_argument("Invalid path")),
                };

                // Look up the base descriptor
                let base_entry = self.fd_table.get(&base_handle)
                    .ok_or_else(|| Error::wasi_invalid_fd("Bad descriptor"))?;

                // Get the base path
                let base_path = match &base_entry.fd_type {
                    FileDescriptorType::PreopenDirectory(p) => p.clone(),
                    _ => return Err(Error::wasi_invalid_argument("Not a directory descriptor")),
                };

                // Construct full path
                let full_path = base_path.join(&path);

                // Check path is within sandbox (no escape via ..)
                let canonical = match full_path.canonicalize() {
                    Ok(p) => p,
                    Err(_) => {
                        // File might not exist yet for write operations
                        full_path.clone()
                    }
                };

                // Basic safety check - path should start with base
                if !canonical.starts_with(&base_path) && full_path.canonicalize().is_ok() {
                    return Err(Error::wasi_permission_denied("Path escapes sandbox"));
                }

                // Allocate new handle
                let new_handle = self.next_handle;
                self.next_handle += 1;

                // Add to fd_table
                self.fd_table.insert(new_handle, FileDescriptorEntry {
                    fd_type: FileDescriptorType::RegularFile(full_path.clone()),
                    read: self.capabilities.filesystem.read_access,
                    write: self.capabilities.filesystem.write_access,
                });

                #[cfg(feature = "tracing")]
                trace!(base_path = %base_path.display(), path = %path, handle = new_handle, "open-at completed");

                // Return result<descriptor, error-code> - for now just Ok(handle)
                Ok(vec![Value::Result(Ok(Box::new(Value::U32(new_handle))))])
            }

            #[cfg(all(feature = "wasi-filesystem", feature = "std"))]
            ("wasi:filesystem/types", "[method]descriptor.stat") => {
                // Check filesystem capability
                if !self.capabilities.filesystem.metadata_access {
                    return Err(Error::wasi_permission_denied("Metadata access denied"));
                }

                // Args: [descriptor_handle]
                let handle = match args.first() {
                    Some(Value::U32(h)) => *h,
                    _ => return Err(Error::wasi_invalid_argument("Invalid descriptor")),
                };

                // Look up the descriptor
                let entry = self.fd_table.get(&handle)
                    .ok_or_else(|| Error::wasi_invalid_fd("Bad descriptor"))?;

                // Get metadata
                let path = match &entry.fd_type {
                    FileDescriptorType::RegularFile(p) => p,
                    FileDescriptorType::PreopenDirectory(p) => p,
                    _ => return Err(Error::wasi_invalid_argument("Cannot stat this descriptor type")),
                };

                match std::fs::metadata(path) {
                    Ok(meta) => {
                        // Return descriptorstat record:
                        // type: descriptor-type (0=unknown, 1=block-device, 2=character-device,
                        //       3=directory, 4=fifo, 5=symbolic-link, 6=regular-file, 7=socket)
                        let file_type = if meta.is_dir() { 3u8 } else if meta.is_file() { 6u8 } else { 0u8 };
                        let size = meta.len();

                        // Simplified stat result - just type and size
                        let stat_record = Value::Record(vec![
                            ("type".to_string(), Value::U8(file_type)),
                            ("size".to_string(), Value::U64(size)),
                        ]);

                        #[cfg(feature = "tracing")]
                        trace!(path = %path.display(), file_type = file_type, size = size, "stat completed");

                        Ok(vec![Value::Result(Ok(Box::new(stat_record)))])
                    }
                    Err(_e) => {
                        #[cfg(feature = "tracing")]
                        warn!(error = %_e, "stat failed");
                        // Return error code
                        Ok(vec![Value::Result(Err(Box::new(Value::U32(2))))])  // ENOENT
                    }
                }
            }

            #[cfg(all(feature = "wasi-filesystem", feature = "std"))]
            ("wasi:filesystem/types", "[method]descriptor.read-via-stream") => {
                // Check read capability
                if !self.capabilities.filesystem.read_access {
                    return Err(Error::wasi_permission_denied("Read access denied"));
                }

                // Args: [descriptor_handle, offset]
                let handle = match args.first() {
                    Some(Value::U32(h)) => *h,
                    _ => return Err(Error::wasi_invalid_argument("Invalid descriptor")),
                };

                // Verify descriptor exists and is readable
                let entry = self.fd_table.get(&handle)
                    .ok_or_else(|| Error::wasi_invalid_fd("Bad descriptor"))?;

                if !entry.read {
                    return Err(Error::wasi_permission_denied("Descriptor not readable"));
                }

                // For now, return the same handle as a stream handle
                // A full implementation would create a separate stream resource
                #[cfg(feature = "tracing")]
                trace!(handle = handle, stream_handle = handle, "read-via-stream completed");

                Ok(vec![Value::Result(Ok(Box::new(Value::U32(handle))))])
            }

            #[cfg(all(feature = "wasi-filesystem", feature = "std"))]
            ("wasi:filesystem/types", "[method]descriptor.write-via-stream") => {
                // Check write capability
                if !self.capabilities.filesystem.write_access {
                    return Err(Error::wasi_permission_denied("Write access denied"));
                }

                // Args: [descriptor_handle, offset]
                let handle = match args.first() {
                    Some(Value::U32(h)) => *h,
                    _ => return Err(Error::wasi_invalid_argument("Invalid descriptor")),
                };

                // Verify descriptor exists and is writable
                let entry = self.fd_table.get(&handle)
                    .ok_or_else(|| Error::wasi_invalid_fd("Bad descriptor"))?;

                if !entry.write {
                    return Err(Error::wasi_permission_denied("Descriptor not writable"));
                }

                // For now, return the same handle as a stream handle
                #[cfg(feature = "tracing")]
                trace!(handle = handle, stream_handle = handle, "write-via-stream completed");

                Ok(vec![Value::Result(Ok(Box::new(Value::U32(handle))))])
            }

            #[cfg(all(feature = "wasi-filesystem", feature = "std"))]
            ("wasi:filesystem/types", "[method]descriptor.readdir") => {
                // Check directory access capability
                if !self.capabilities.filesystem.directory_access {
                    return Err(Error::wasi_permission_denied("Directory access denied"));
                }

                // Args: [descriptor_handle]
                let handle = match args.first() {
                    Some(Value::U32(h)) => *h,
                    _ => return Err(Error::wasi_invalid_argument("Invalid descriptor")),
                };

                let entry = self.fd_table.get(&handle)
                    .ok_or_else(|| Error::wasi_invalid_fd("Bad descriptor"))?;

                let path = match &entry.fd_type {
                    FileDescriptorType::PreopenDirectory(p) => p,
                    _ => return Err(Error::wasi_invalid_argument("Not a directory descriptor")),
                };

                match std::fs::read_dir(path) {
                    Ok(entries) => {
                        let dir_entries: Vec<Value> = entries
                            .filter_map(|e| e.ok())
                            .map(|e| {
                                let name = e.file_name().to_string_lossy().to_string();
                                let file_type = if e.path().is_dir() { 3u8 } else { 6u8 };
                                Value::Record(vec![
                                    ("name".to_string(), Value::String(name)),
                                    ("type".to_string(), Value::U8(file_type)),
                                ])
                            })
                            .collect();

                        Ok(vec![Value::Result(Ok(Box::new(Value::List(dir_entries))))])
                    }
                    Err(_e) => {
                        Ok(vec![Value::Result(Err(Box::new(Value::U32(2))))])  // ENOENT
                    }
                }
            }

            #[cfg(all(feature = "wasi-filesystem", feature = "std"))]
            ("wasi:filesystem/types", "[method]descriptor.create-directory-at") => {
                // Check write capability
                if !self.capabilities.filesystem.write_access {
                    return Err(Error::wasi_permission_denied("Write access denied"));
                }

                // Args: [descriptor_handle, path]
                let base_handle = match args.first() {
                    Some(Value::U32(h)) => *h,
                    _ => return Err(Error::wasi_invalid_argument("Invalid descriptor")),
                };

                let path = match args.get(1) {
                    Some(Value::String(s)) => s.clone(),
                    _ => return Err(Error::wasi_invalid_argument("Invalid path")),
                };

                let base_entry = self.fd_table.get(&base_handle)
                    .ok_or_else(|| Error::wasi_invalid_fd("Bad descriptor"))?;

                let base_path = match &base_entry.fd_type {
                    FileDescriptorType::PreopenDirectory(p) => p.clone(),
                    _ => return Err(Error::wasi_invalid_argument("Not a directory descriptor")),
                };

                let full_path = base_path.join(&path);

                // Sandbox check
                if full_path.canonicalize().is_ok() && !full_path.starts_with(&base_path) {
                    return Err(Error::wasi_permission_denied("Path escapes sandbox"));
                }

                match std::fs::create_dir(&full_path) {
                    Ok(()) => Ok(vec![Value::Result(Ok(Box::new(Value::Tuple(vec![]))))]),
                    Err(_e) => Ok(vec![Value::Result(Err(Box::new(Value::U32(17))))]),  // EEXIST
                }
            }

            #[cfg(all(feature = "wasi-filesystem", feature = "std"))]
            ("wasi:filesystem/types", "[method]descriptor.unlink-file-at") => {
                // Check write capability
                if !self.capabilities.filesystem.write_access {
                    return Err(Error::wasi_permission_denied("Write access denied"));
                }

                // Args: [descriptor_handle, path]
                let base_handle = match args.first() {
                    Some(Value::U32(h)) => *h,
                    _ => return Err(Error::wasi_invalid_argument("Invalid descriptor")),
                };

                let path = match args.get(1) {
                    Some(Value::String(s)) => s.clone(),
                    _ => return Err(Error::wasi_invalid_argument("Invalid path")),
                };

                let base_entry = self.fd_table.get(&base_handle)
                    .ok_or_else(|| Error::wasi_invalid_fd("Bad descriptor"))?;

                let base_path = match &base_entry.fd_type {
                    FileDescriptorType::PreopenDirectory(p) => p.clone(),
                    _ => return Err(Error::wasi_invalid_argument("Not a directory descriptor")),
                };

                let full_path = base_path.join(&path);

                // Sandbox check
                if let Ok(canonical) = full_path.canonicalize() {
                    if !canonical.starts_with(&base_path) {
                        return Err(Error::wasi_permission_denied("Path escapes sandbox"));
                    }
                }

                match std::fs::remove_file(&full_path) {
                    Ok(()) => Ok(vec![Value::Result(Ok(Box::new(Value::Tuple(vec![]))))]),
                    Err(_e) => Ok(vec![Value::Result(Err(Box::new(Value::U32(2))))]),  // ENOENT
                }
            }

            #[cfg(all(feature = "wasi-filesystem", feature = "std"))]
            ("wasi:filesystem/types", "[resource-drop]descriptor") => {
                // Remove descriptor from table
                if let Some(Value::U32(handle)) = args.first() {
                    self.fd_table.remove(handle);
                }
                Ok(vec![])
            }

            // ================================================================
            // wasi:sockets/* - Socket interfaces
            // ================================================================

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/tcp", "create-tcp-socket") => {
                wasi_tcp_create(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/tcp", "start-connect") => {
                wasi_tcp_connect(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/tcp", "start-bind") => {
                wasi_tcp_bind(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/tcp", "listen") => {
                wasi_tcp_listen(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/tcp", "accept") => {
                wasi_tcp_accept(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/tcp", "send") => {
                wasi_tcp_send(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/tcp", "receive") => {
                wasi_tcp_recv(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/tcp", "shutdown") => {
                wasi_tcp_shutdown(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/udp", "create-udp-socket") => {
                wasi_udp_create(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/udp", "start-bind") => {
                wasi_udp_bind(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/udp", "send") => {
                wasi_udp_send(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/udp", "receive") => {
                wasi_udp_recv(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(all(feature = "wasi-sockets", feature = "std"))]
            ("wasi:sockets/ip-name-lookup", "resolve-addresses") => {
                wasi_resolve_addresses(&mut () as &mut dyn core::any::Any, args)
            }

            // ================================================================
            // wasi:random/* - Random number generation interfaces
            // ================================================================

            #[cfg(feature = "wasi-random")]
            ("wasi:random/random", "get-random-bytes") => {
                // Check random capability
                if !self.capabilities.random.secure_random {
                    return Err(Error::wasi_permission_denied("Secure random access denied"));
                }
                wasi_get_random_bytes(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(feature = "wasi-random")]
            ("wasi:random/random", "get-random-u64") => {
                if !self.capabilities.random.secure_random {
                    return Err(Error::wasi_permission_denied("Secure random access denied"));
                }
                wasi_get_random_u64(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(feature = "wasi-random")]
            ("wasi:random/insecure", "get-insecure-random-bytes") => {
                if !self.capabilities.random.pseudo_random {
                    return Err(Error::wasi_permission_denied("Pseudo-random access denied"));
                }
                wasi_get_insecure_random_bytes(&mut () as &mut dyn core::any::Any, args)
            }

            #[cfg(feature = "wasi-random")]
            ("wasi:random/insecure", "get-insecure-random-u64") => {
                if !self.capabilities.random.pseudo_random {
                    return Err(Error::wasi_permission_denied("Pseudo-random access denied"));
                }
                wasi_get_insecure_random_u64(&mut () as &mut dyn core::any::Any, args)
            }

            // ================================================================
            // Unknown function - return error
            // ================================================================
            _ => {
                #[cfg(feature = "tracing")]
                warn!(interface = %base_interface, function = %function, "unknown WASI function");

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
    /// * `memory` - Linear memory accessor for reading/writing complex types
    ///   Uses interior mutability for thread-safe writes.
    #[cfg(feature = "std")]
    pub fn dispatch_core(
        &mut self,
        interface: &str,
        function: &str,
        args: Vec<wrt_foundation::values::Value>,
        memory: Option<&dyn MemoryAccessor>,
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
                        #[cfg(feature = "tracing")]
                        warn!("get-arguments: no return pointer provided");
                        return Ok(vec![]);
                    }
                };

                #[cfg(feature = "tracing")]
                trace!(
                    arg_count = args_to_use.len(),
                    args = ?args_to_use,
                    retptr = format_args!("0x{:x}", retptr),
                    args_alloc = ?self.args_alloc,
                    "get-arguments dispatch"
                );

                if let Some(mem) = memory {
                    #[cfg(feature = "tracing")]
                    trace!(memory_size = mem.size(), "memory available");

                    if args_to_use.is_empty() {
                        // Empty list: write (0, 0) to return pointer
                        mem.write_bytes(retptr, &0u32.to_le_bytes())?;
                        mem.write_bytes(retptr + 4, &0u32.to_le_bytes())?;
                        return Ok(vec![]);
                    }

                    // Use pre-allocated memory from cabi_realloc - this is required
                    let (list_ptr, mut string_data_ptr) = if let Some((alloc_list, alloc_strings)) = self.args_alloc {
                        #[cfg(feature = "tracing")]
                        trace!(
                            list_ptr = format_args!("0x{:x}", alloc_list),
                            string_ptr = format_args!("0x{:x}", alloc_strings),
                            "using cabi_realloc memory"
                        );
                        (alloc_list, alloc_strings)
                    } else {
                        // NO FALLBACK - cabi_realloc must have been called
                        // Per CLAUDE.md: "FAIL LOUD AND EARLY" - no stack-relative hacks
                        #[cfg(feature = "tracing")]
                        warn!("args_alloc not set - cabi_realloc was not called");
                        return Err(wrt_error::Error::runtime_error(
                            "get-arguments requires cabi_realloc - allocation not performed"
                        ));
                    };

                    // First pass: write string data and collect pointers
                    let mut string_entries: Vec<(u32, u32)> = Vec::new();
                    for arg in &args_to_use {
                        let bytes = arg.as_bytes();
                        let len = bytes.len() as u32;

                        // Bounds check
                        if (string_data_ptr as usize + bytes.len()) > mem.size() {
                            #[cfg(feature = "tracing")]
                            warn!("not enough memory for arg data");
                            // Write empty list
                            mem.write_bytes(retptr, &0u32.to_le_bytes())?;
                            mem.write_bytes(retptr + 4, &0u32.to_le_bytes())?;
                            return Ok(vec![]);
                        }

                        // Write string bytes
                        mem.write_bytes(string_data_ptr, bytes)?;
                        #[cfg(feature = "tracing")]
                        trace!(
                            arg = %arg,
                            byte_count = bytes.len(),
                            addr = format_args!("0x{:x}", string_data_ptr),
                            "wrote string"
                        );

                        string_entries.push((string_data_ptr, len));
                        string_data_ptr += len;
                        // Align to 8 bytes
                        string_data_ptr = (string_data_ptr + 7) & !7;
                    }

                    // Second pass: write (ptr, len) array at list_ptr
                    for (i, (ptr, len)) in string_entries.iter().enumerate() {
                        let offset = list_ptr + (i as u32 * 8);
                        mem.write_bytes(offset, &ptr.to_le_bytes())?;
                        mem.write_bytes(offset + 4, &len.to_le_bytes())?;
                        #[cfg(feature = "tracing")]
                        trace!(index = i, ptr = format_args!("0x{:x}", ptr), len = len, "list entry written");
                    }

                    // Write (list_ptr, count) to the return pointer
                    mem.write_bytes(retptr, &list_ptr.to_le_bytes())?;
                    mem.write_bytes(retptr + 4, &(args_to_use.len() as u32).to_le_bytes())?;
                    #[cfg(feature = "tracing")]
                    trace!(
                        retptr = format_args!("0x{:x}", retptr),
                        list_ptr = format_args!("0x{:x}", list_ptr),
                        count = args_to_use.len(),
                        "wrote to retptr"
                    );

                    // Canonical ABI with return pointer returns void (no return values)
                    Ok(vec![])
                } else {
                    #[cfg(feature = "tracing")]
                    warn!("no memory available for get-arguments");
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

                // Per Canonical ABI: datetime record is written to retptr in memory
                // datetime = { seconds: u64, nanoseconds: u32 } = 12 bytes
                // The retptr is passed as the first argument
                if let Some(mem) = memory {
                    let retptr = match args.first() {
                        Some(CoreValue::I32(p)) => *p as u32,
                        _ => {
                            // No retptr provided - return flat values for non-indirected ABI
                            #[cfg(feature = "tracing")]
                            trace!("wall-clock::now - no retptr, returning flat values");
                            return Ok(vec![
                                CoreValue::I64(seconds as i64),
                                CoreValue::I32(nanoseconds as i32),
                            ]);
                        }
                    };

                    // Per Canonical ABI: datetime record layout
                    // seconds: u64 (8 bytes) at offset 0
                    // nanoseconds: u32 (4 bytes) at offset 8
                    // Write seconds (u64, 8 bytes) at retptr
                    mem.write_bytes(retptr, &seconds.to_le_bytes())?;
                    // Write nanoseconds as u32 (4 bytes) at retptr + 8
                    mem.write_bytes(retptr + 8, &nanoseconds.to_le_bytes())?;

                    #[cfg(feature = "tracing")]
                    trace!(
                        retptr = format_args!("0x{:x}", retptr),
                        seconds = seconds,
                        nanoseconds = nanoseconds,
                        "wall-clock::now wrote datetime"
                    );

                    // Return nothing - datetime was written to memory
                    Ok(vec![])
                } else {
                    // No memory available - return flat values
                    #[cfg(feature = "tracing")]
                    trace!("wall-clock::now - no memory, returning flat values");
                    Ok(vec![
                        CoreValue::I64(seconds as i64),
                        CoreValue::I32(nanoseconds as i32),
                    ])
                }
            }

            #[cfg(feature = "wasi-clocks")]
            ("wasi:clocks/wall-clock", "resolution") => {
                // 1 nanosecond resolution
                // Per Canonical ABI: datetime record is written to retptr in memory
                if let Some(mem) = memory {
                    let retptr = match args.first() {
                        Some(CoreValue::I32(p)) => *p as u32,
                        _ => {
                            return Ok(vec![
                                CoreValue::I64(0), // seconds
                                CoreValue::I32(1), // nanoseconds
                            ]);
                        }
                    };

                    // Write seconds (0) at retptr
                    mem.write_bytes(retptr, &0u64.to_le_bytes())?;
                    // Write nanoseconds (1) as i64 at retptr + 8
                    mem.write_bytes(retptr + 8, &1u64.to_le_bytes())?;
                    Ok(vec![])
                } else {
                    Ok(vec![
                        CoreValue::I64(0), // seconds
                        CoreValue::I32(1), // nanoseconds
                    ])
                }
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
                #[cfg(feature = "tracing")]
                trace!(args = ?args, has_memory = memory.is_some(), "blocking-write-and-flush dispatch");

                if !self.capabilities.io.stdout_access {
                    return Err(Error::wasi_permission_denied("Stream write access denied"));
                }

                // Args: handle (i32), data_ptr (i32), data_len (i32), retptr (i32)
                // The retptr is where we write the result<_, stream-error>
                if args.len() < 3 {
                    return Err(Error::wasi_invalid_argument("blocking-write-and-flush requires 3+ args"));
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

                // Get retptr if provided (4th argument)
                let retptr = if args.len() >= 4 {
                    match &args[3] {
                        CoreValue::I32(p) => Some(*p as u32),
                        _ => None,
                    }
                } else {
                    None
                };

                // Read data from memory
                let mem = memory.ok_or_else(||
                    Error::wasi_capability_unavailable("Memory required for write"))?;

                #[cfg(feature = "tracing")]
                trace!(
                    handle = handle,
                    data_ptr = format_args!("0x{:x}", data_ptr),
                    data_len = data_len,
                    retptr = ?retptr,
                    mem_size = mem.size(),
                    "write parameters"
                );

                // Read data from memory using MemoryAccessor
                let mut data = vec![0u8; data_len as usize];
                mem.read_bytes(data_ptr, &mut data)?;

                #[cfg(feature = "tracing")]
                trace!(
                    byte_count = data.len(),
                    preview = %String::from_utf8_lossy(&data[..data.len().min(100)]),
                    "data to write"
                );

                // Write to stdout or stderr
                #[cfg(feature = "std")]
                {
                    use std::io::{self, Write};

                    let result = if handle == self.stdout_handle {
                        #[cfg(feature = "tracing")]
                        trace!("writing to stdout");
                        io::stdout().write_all(&data).and_then(|_| io::stdout().flush())
                    } else if handle == self.stderr_handle {
                        #[cfg(feature = "tracing")]
                        trace!("writing to stderr");
                        io::stderr().write_all(&data).and_then(|_| io::stderr().flush())
                    } else {
                        #[cfg(feature = "tracing")]
                        warn!(
                            handle = handle,
                            stdout_handle = self.stdout_handle,
                            stderr_handle = self.stderr_handle,
                            "invalid handle"
                        );
                        return Err(Error::wasi_invalid_fd("Invalid stream handle"));
                    };

                    // Write result to retptr if provided
                    // For result<_, stream-error>:
                    //   discriminant 0 = ok (no payload)
                    //   discriminant 1 = err (followed by error info)
                    if let Some(rp) = retptr {
                        if (rp as usize) < mem.size() {
                            match &result {
                                Ok(()) => {
                                    // Ok variant: discriminant = 0
                                    mem.write_bytes(rp, &[0])?;
                                    #[cfg(feature = "tracing")]
                                    trace!(retptr = format_args!("0x{:x}", rp), "wrote Ok discriminant (0)");
                                }
                                Err(_) => {
                                    // Err variant: discriminant = 1
                                    // Note: stream-error payload would need more data, but for now just set discriminant
                                    mem.write_bytes(rp, &[1])?;
                                    #[cfg(feature = "tracing")]
                                    trace!(retptr = format_args!("0x{:x}", rp), "wrote Err discriminant (1)");
                                }
                            }
                        }
                    }

                    match result {
                        Ok(()) => {
                            #[cfg(feature = "tracing")]
                            trace!("write succeeded");
                            Ok(vec![]) // No stack return for functions with retptr
                        }
                        Err(_e) => {
                            #[cfg(feature = "tracing")]
                            warn!(error = ?_e, "write failed");
                            Ok(vec![]) // No stack return for functions with retptr
                        }
                    }
                }
                #[cfg(not(feature = "std"))]
                {
                    // In no_std, we can't write to stdout, but still write Ok to retptr
                    if let Some(rp) = retptr {
                        if (rp as usize) < mem.size() {
                            mem.write_bytes(rp, &[0])?; // Ok discriminant
                        }
                    }
                    Ok(vec![])
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

            // WASI Neural Network (wasi-nn) functions
            // These provide ML inference capabilities for WebAssembly components
            // Note: nn-preview2 feature enables the synchronous sync_bridge module
            #[cfg(all(feature = "nn-preview2", feature = "std"))]
            ("wasi:nn/inference", "load") => {
                use crate::nn::nn_load;

                // Args: (data_ptr: i32, data_len: i32, encoding: i32, target: i32)
                let data_ptr = match args.get(0) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_load: missing data_ptr")),
                };
                let data_len = match args.get(1) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_load: missing data_len")),
                };
                let encoding = match args.get(2) {
                    Some(CoreValue::I32(v)) => *v as u8,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_load: missing encoding")),
                };
                let target = match args.get(3) {
                    Some(CoreValue::I32(v)) => *v as u8,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_load: missing target")),
                };

                // Read model data from linear memory
                let mem = memory.ok_or_else(|| wrt_error::Error::runtime_error(
                    "nn_load requires memory access"
                ))?;

                let mut data = vec![0u8; data_len as usize];
                mem.read_bytes(data_ptr, &mut data)?;

                // Call the actual function
                let graph_id = nn_load(data, encoding, target)?;

                Ok(vec![CoreValue::I32(graph_id as i32)])
            }

            #[cfg(all(feature = "nn-preview2", feature = "std"))]
            ("wasi:nn/inference", "init-execution-context") => {
                use crate::nn::nn_init_execution_context;

                // Args: (graph_id: i32)
                let graph_id = match args.get(0) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_init_execution_context: missing graph_id")),
                };

                let context_id = nn_init_execution_context(graph_id)?;

                Ok(vec![CoreValue::I32(context_id as i32)])
            }

            #[cfg(all(feature = "nn-preview2", feature = "std"))]
            ("wasi:nn/inference", "set-input") => {
                use crate::nn::nn_set_input;

                // Args: (context_id, index, tensor_ptr, tensor_len, dims_ptr, dims_count, type)
                let context_id = match args.get(0) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_set_input: missing context_id")),
                };
                let index = match args.get(1) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_set_input: missing index")),
                };
                let tensor_ptr = match args.get(2) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_set_input: missing tensor_ptr")),
                };
                let tensor_len = match args.get(3) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_set_input: missing tensor_len")),
                };
                let dims_ptr = match args.get(4) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_set_input: missing dims_ptr")),
                };
                let dims_count = match args.get(5) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_set_input: missing dims_count")),
                };
                let tensor_type = match args.get(6) {
                    Some(CoreValue::I32(v)) => *v as u8,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_set_input: missing tensor_type")),
                };

                let mem = memory.ok_or_else(|| wrt_error::Error::runtime_error(
                    "nn_set_input requires memory access"
                ))?;

                // Read tensor data
                let mut tensor_data = vec![0u8; tensor_len as usize];
                mem.read_bytes(tensor_ptr, &mut tensor_data)?;

                // Read dimensions (as u32 array)
                let mut dims_bytes = vec![0u8; (dims_count * 4) as usize];
                mem.read_bytes(dims_ptr, &mut dims_bytes)?;
                let dimensions: Vec<u32> = dims_bytes
                    .chunks(4)
                    .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();

                nn_set_input(context_id, index, tensor_data, dimensions, tensor_type)?;

                Ok(vec![])
            }

            #[cfg(all(feature = "nn-preview2", feature = "std"))]
            ("wasi:nn/inference", "compute") => {
                use crate::nn::nn_compute;

                // Args: (context_id: i32)
                let context_id = match args.get(0) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_compute: missing context_id")),
                };

                nn_compute(context_id)?;

                Ok(vec![])
            }

            #[cfg(all(feature = "nn-preview2", feature = "std"))]
            ("wasi:nn/inference", "get-output") => {
                use crate::nn::nn_get_output;

                // Args: (context_id, index, out_ptr, out_capacity)
                let context_id = match args.get(0) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_get_output: missing context_id")),
                };
                let index = match args.get(1) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_get_output: missing index")),
                };
                let out_ptr = match args.get(2) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_get_output: missing out_ptr")),
                };
                let out_capacity = match args.get(3) {
                    Some(CoreValue::I32(v)) => *v as u32,
                    _ => return Err(wrt_error::Error::wasi_invalid_argument("nn_get_output: missing out_capacity")),
                };

                let mem = memory.ok_or_else(|| wrt_error::Error::runtime_error(
                    "nn_get_output requires memory access"
                ))?;

                let (data, _dimensions, _tensor_type) = nn_get_output(context_id, index)?;

                // Write as much data as fits in the provided buffer
                let write_len = core::cmp::min(data.len(), out_capacity as usize);
                mem.write_bytes(out_ptr, &data[..write_len])?;

                // Return actual data length (caller may need to reallocate)
                Ok(vec![CoreValue::I32(data.len() as i32)])
            }

            _ => {
                #[cfg(feature = "tracing")]
                warn!(interface = %base_interface, function = %function, "unknown WASI function (core)");

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
                        #[cfg(feature = "tracing")]
                        warn!("get-arguments v2: no return pointer provided");
                        return Ok(DispatchResult::Complete(vec![]));
                    }
                };

                #[cfg(feature = "tracing")]
                trace!(
                    arg_count = args_to_use.len(),
                    retptr = format_args!("0x{:x}", retptr),
                    has_alloc = self.args_alloc.is_some(),
                    "get-arguments v2 dispatch"
                );

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

                #[cfg(feature = "tracing")]
                trace!(
                    list_size = list_size,
                    string_total = string_total,
                    total = total_size,
                    "get-arguments v2 needs allocation"
                );

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
                #[cfg(feature = "tracing")]
                trace!(interface = %base_interface, function = %function, "delegating to dispatch_core");

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
        #[cfg(feature = "tracing")]
        trace!(
            arg_count = args_to_write.len(),
            ptr = format_args!("0x{:x}", allocated_ptr),
            retptr = format_args!("0x{:x}", retptr),
            "completing allocation"
        );

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

            #[cfg(feature = "tracing")]
            trace!(
                arg = %arg,
                byte_count = bytes.len(),
                addr = format_args!("0x{:x}", string_data_ptr),
                "wrote string (complete)"
            );

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

            #[cfg(feature = "tracing")]
            trace!(index = i, ptr = format_args!("0x{:x}", ptr), len = len, "list entry written (complete)");
        }

        // Write (list_ptr, count) to the return pointer
        let retptr_usize = retptr as usize;
        if retptr_usize + 8 <= memory.len() {
            memory[retptr_usize..retptr_usize + 4].copy_from_slice(&list_ptr.to_le_bytes());
            memory[retptr_usize + 4..retptr_usize + 8]
                .copy_from_slice(&(args_to_write.len() as u32).to_le_bytes());

            #[cfg(feature = "tracing")]
            trace!(
                retptr = format_args!("0x{:x}", retptr),
                list_ptr = format_args!("0x{:x}", list_ptr),
                count = args_to_write.len(),
                "wrote to retptr (complete)"
            );
        }

        Ok(())
    }
}

/// Implementation of HostImportHandler for WasiDispatcher
///
/// This allows the engine to call WASI functions through the generic trait
/// without knowing about WASI specifics.
#[cfg(feature = "std")]
impl wrt_foundation::HostImportHandler for WasiDispatcher {
    fn call_import(
        &mut self,
        module: &str,
        function: &str,
        args: &[wrt_foundation::Value],
        memory: Option<&dyn MemoryAccessor>,
    ) -> Result<Vec<wrt_foundation::Value>> {
        // Delegate to dispatch_core with MemoryAccessor
        self.dispatch_core(module, function, args.to_vec(), memory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_foundation::memory_init::MemoryInitializer;

    #[test]
    fn test_dispatcher_creation() -> Result<()> {
        MemoryInitializer::ensure_initialized()?;
        let caps = WasiCapabilities::minimal()?;
        let dispatcher = WasiDispatcher::new(caps)?;
        assert_eq!(dispatcher.stdout_handle, 1);
        assert_eq!(dispatcher.stderr_handle, 2);
        Ok(())
    }

    #[test]
    fn test_get_stdout() -> Result<()> {
        MemoryInitializer::ensure_initialized()?;
        let mut dispatcher = WasiDispatcher::with_defaults()?;
        let result = dispatcher.dispatch("wasi:cli/stdout@0.2.4", "get-stdout", vec![])?;
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::U32(1)));
        Ok(())
    }

    #[test]
    fn test_get_stderr() -> Result<()> {
        MemoryInitializer::ensure_initialized()?;
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
        MemoryInitializer::ensure_initialized()?;
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
