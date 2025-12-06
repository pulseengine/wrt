//! WASI Preview2 (wasip2) Host Implementation
//!
//! This module provides host implementations for WASI preview2 interfaces
//! that are used by WebAssembly components through the Component Model.

use wrt_foundation::values::Value;
use alloc::vec::Vec;
use wrt_error::Result;

/// Resource handle type for wasip2 resources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceHandle(pub u32);

/// Output stream resource for wasi:io/streams
pub struct OutputStreamResource {
    /// The actual output destination (stdout, stderr, file, etc.)
    pub target: OutputTarget,
    /// Buffer for pending writes
    pub buffer: Vec<u8>,
}

/// Output target for streams
#[derive(Debug, Clone)]
pub enum OutputTarget {
    Stdout,
    Stderr,
    File(String),
    Memory(Vec<u8>),
}

/// Error resource for wasi:io/error
pub struct ErrorResource {
    pub message: String,
    pub code: u32,
}

/// WASI Preview2 Host Functions
pub struct Wasip2Host {
    /// Next available resource handle
    next_handle: u32,
    /// Output stream resources
    output_streams: Vec<OutputStreamResource>,
    /// Error resources
    errors: Vec<ErrorResource>,
}

impl Default for Wasip2Host {
    fn default() -> Self {
        Self::new()
    }
}

impl Wasip2Host {
    pub fn new() -> Self {
        let mut host = Self {
            next_handle: 1,
            output_streams: Vec::new(),
            errors: Vec::new(),
        };

        // Pre-create stdout and stderr streams (handles 1 and 2)
        host.create_stdout();
        host.create_stderr();

        host
    }

    fn create_stdout(&mut self) -> ResourceHandle {
        let stream = OutputStreamResource {
            target: OutputTarget::Stdout,
            buffer: Vec::new(),
        };
        self.output_streams.push(stream);
        let handle = ResourceHandle(self.next_handle);
        self.next_handle += 1;
        handle
    }

    fn create_stderr(&mut self) -> ResourceHandle {
        let stream = OutputStreamResource {
            target: OutputTarget::Stderr,
            buffer: Vec::new(),
        };
        self.output_streams.push(stream);
        let handle = ResourceHandle(self.next_handle);
        self.next_handle += 1;
        handle
    }

    /// wasi:cli/stdout.get-stdout
    pub fn cli_stdout_get_stdout(&mut self) -> Result<Vec<Value>> {
        // Return handle to stdout stream (always handle 1)
        Ok(vec![Value::I32(1)])
    }

    /// wasi:cli/stderr.get-stderr
    pub fn cli_stderr_get_stderr(&mut self) -> Result<Vec<Value>> {
        // Return handle to stderr stream (always handle 2)
        Ok(vec![Value::I32(2)])
    }

    /// wasi:io/streams.output-stream.blocking-write-and-flush
    pub fn io_streams_output_stream_blocking_write_and_flush(
        &mut self,
        handle: u32,
        data_ptr: u32,
        data_len: u32,
        memory: &mut [u8],
    ) -> Result<Vec<Value>> {
        // Get the stream resource
        let stream_idx = (handle - 1) as usize;
        if stream_idx >= self.output_streams.len() {
            return Ok(vec![Value::I32(1)]); // Return error variant
        }

        // Read data from memory
        let start = data_ptr as usize;
        let end = start + data_len as usize;
        if end > memory.len() {
            return Ok(vec![Value::I32(1)]); // Return error variant
        }

        let data = &memory[start..end];

        // Write to the appropriate target
        match &self.output_streams[stream_idx].target {
            OutputTarget::Stdout => {
                #[cfg(feature = "std")]
                {
                    use std::io::{self, Write};
                    let _ = io::stdout().write_all(data);
                    let _ = io::stdout().flush();
                }
                #[cfg(not(feature = "std"))]
                {
                    // In no_std, just print debug output
                    if let Ok(s) = core::str::from_utf8(data) {
                        eprintln!("[STDOUT] {}", s);
                    }
                }
            },
            OutputTarget::Stderr => {
                #[cfg(feature = "std")]
                {
                    use std::io::{self, Write};
                    let _ = io::stderr().write_all(data);
                    let _ = io::stderr().flush();
                }
                #[cfg(not(feature = "std"))]
                {
                    // In no_std, just print debug output
                    if let Ok(s) = core::str::from_utf8(data) {
                        eprintln!("[STDERR] {}", s);
                    }
                }
            },
            OutputTarget::File(_path) => {
                // File output not yet implemented
            },
            OutputTarget::Memory(_buffer) => {
                // In a real implementation, we'd append to the buffer
            }
        }

        // Return ok variant (0) with no error
        Ok(vec![Value::I32(0)])
    }

    /// wasi:io/streams.output-stream.blocking-flush
    pub fn io_streams_output_stream_blocking_flush(
        &mut self,
        _handle: u32,
    ) -> Result<Vec<Value>> {

        // For now, we don't buffer, so flush is a no-op
        // Return ok variant (0)
        Ok(vec![Value::I32(0)])
    }

    /// Extract the base interface name without version suffix
    /// e.g., "wasi:cli/stdout@0.2.4" -> "wasi:cli/stdout"
    fn strip_version(interface: &str) -> &str {
        if let Some(idx) = interface.find('@') {
            &interface[..idx]
        } else {
            interface
        }
    }

    /// Dispatch a wasip2 function call based on interface and function name
    pub fn dispatch(
        &mut self,
        interface: &str,
        function: &str,
        args: Vec<Value>,
        memory: Option<&mut [u8]>,
    ) -> Result<Vec<Value>> {
        // Strip version for matching - accept any 0.2.x version
        let base_interface = Self::strip_version(interface);

        match (base_interface, function) {
            ("wasi:cli/stdout", "get-stdout") => {
                self.cli_stdout_get_stdout()
            },
            ("wasi:cli/stderr", "get-stderr") => {
                self.cli_stderr_get_stderr()
            },
            ("wasi:io/streams", "[method]output-stream.blocking-write-and-flush") |
            ("wasi:io/streams", "output-stream.blocking-write-and-flush") => {
                if args.len() < 3 {
                    return Err(wrt_error::Error::runtime_error("Invalid args for blocking-write-and-flush"));
                }
                let handle = match args[0] {
                    Value::I32(h) => h as u32,
                    _ => return Err(wrt_error::Error::runtime_error("Invalid handle type")),
                };
                let data_ptr = match args[1] {
                    Value::I32(p) => p as u32,
                    _ => return Err(wrt_error::Error::runtime_error("Invalid ptr type")),
                };
                let data_len = match args[2] {
                    Value::I32(l) => l as u32,
                    _ => return Err(wrt_error::Error::runtime_error("Invalid len type")),
                };

                if let Some(mem) = memory {
                    self.io_streams_output_stream_blocking_write_and_flush(handle, data_ptr, data_len, mem)
                } else {
                    Err(wrt_error::Error::runtime_error("No memory for blocking-write-and-flush"))
                }
            },
            ("wasi:io/streams", "[method]output-stream.blocking-flush") |
            ("wasi:io/streams", "output-stream.blocking-flush") => {
                if args.is_empty() {
                    return Err(wrt_error::Error::runtime_error("Invalid args for blocking-flush"));
                }
                let handle = match args[0] {
                    Value::I32(h) => h as u32,
                    _ => return Err(wrt_error::Error::runtime_error("Invalid handle type")),
                };
                self.io_streams_output_stream_blocking_flush(handle)
            },
            _ => {
                Err(wrt_error::Error::runtime_error("Unknown wasip2 function"))
            }
        }
    }
}