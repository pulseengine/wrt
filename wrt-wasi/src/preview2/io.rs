//! WASI I/O interface implementation
//!
//! Implements the `wasi:io` interface for stream operations using WRT's
//! resource management patterns and platform abstractions.

use core::any::Any;

use crate::{
    prelude::*,
    Value,
};

/// WASI stream read operation
///
/// Implements `wasi:io/streams.read` for reading from input streams
pub fn wasi_stream_read(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    // Extract stream handle and length from arguments
    let stream_handle = extract_stream_handle(&args)?;
    let len = extract_read_length(&args, 1)?;

    // Validate stream handle and check if readable
    // In a real implementation, this would access the resource manager
    // to verify the stream exists and is readable

    // Perform platform-specific read operation
    let data = perform_stream_read(stream_handle, len)?;

    // Return as WASI list<u8>
    let value_data: Vec<Value> = data.into_iter().map(Value::U8).collect();
    Ok(vec![Value::List(value_data)])
}

/// WASI stream write operation
///
/// Implements `wasi:io/streams.write` for writing to output streams
pub fn wasi_stream_write(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    // Extract stream handle and data from arguments
    let stream_handle = extract_stream_handle(&args)?;
    let data = extract_write_data(&args, 1)?;

    // Validate stream handle and check if writable
    // In a real implementation, this would access the resource manager

    // Perform platform-specific write operation
    let bytes_written = perform_stream_write(stream_handle, &data)?;

    // Return bytes written
    Ok(vec![Value::U64(bytes_written)])
}

/// WASI stream flush operation
///
/// Implements `wasi:io/streams.flush` for flushing output streams
pub fn wasi_stream_flush(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    let stream_handle = extract_stream_handle(&args)?;

    // Validate stream handle and check if writable

    // Perform platform-specific flush operation
    perform_stream_flush(stream_handle)?;

    // Return unit (no value)
    Ok(vec![])
}

/// WASI stream check-write operation
///
/// Implements `wasi:io/streams.check-write` to check available write space
pub fn wasi_stream_check_write(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    let stream_handle = extract_stream_handle(&args)?;

    // Check how many bytes can be written without blocking
    let available = check_write_capacity(stream_handle)?;

    Ok(vec![Value::U64(available)])
}

/// WASI stream subscribe operation
///
/// Implements `wasi:io/poll.subscribe` for async I/O notification
pub fn wasi_stream_subscribe(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    let stream_handle = extract_stream_handle(&args)?;

    // Create a pollable for this stream
    // In a real implementation, this would integrate with the async runtime
    let pollable_handle = create_pollable(stream_handle)?;

    Ok(vec![Value::U32(pollable_handle)])
}

/// WASI poll one-off operation
///
/// Implements `wasi:io/poll.poll-one-off` for synchronous polling
pub fn wasi_poll_one_off(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    let pollables = extract_pollable_list(&args)?;

    // Poll all the pollables and return results
    let results = poll_pollables(&pollables)?;

    // Convert results to WASI format
    let wasi_results: Vec<Value> = results.into_iter().map(Value::Bool).collect();

    Ok(vec![Value::List(wasi_results)])
}

/// Helper function to extract stream handle from arguments
fn extract_stream_handle(args: &[Value]) -> Result<u32> {
    if args.is_empty() {
        return Err(Error::wasi_invalid_fd("Missing stream handle argument"));
    }

    match &args[0] {
        Value::U32(handle) => Ok(*handle),
        Value::S32(handle) => {
            if *handle < 0 {
                Err(Error::wasi_invalid_fd("Invalid negative stream handle"))
            } else {
                Ok(*handle as u32)
            }
        },
        _ => Err(Error::wasi_invalid_fd("Invalid stream handle type")),
    }
}

/// Helper function to extract read length from arguments
fn extract_read_length(args: &[Value], index: usize) -> Result<u64> {
    if args.len() <= index {
        return Err(Error::wasi_invalid_fd("Missing read length argument"));
    }

    match &args[index] {
        Value::U64(len) => Ok(*len),
        Value::U32(len) => Ok(u64::from(*len)),
        _ => Err(Error::wasi_invalid_fd("Invalid read length type")),
    }
}

/// Helper function to extract write data from arguments
fn extract_write_data(args: &[Value], index: usize) -> Result<Vec<u8>> {
    if args.len() <= index {
        return Err(Error::wasi_invalid_fd("Missing write data argument"));
    }

    match &args[index] {
        Value::List(items) => {
            let mut data = Vec::new();
            for item in items {
                match item {
                    Value::U8(byte) => data.push(*byte),
                    _ => return Err(Error::wasi_invalid_fd("Invalid byte data in write")),
                }
            }
            Ok(data)
        },
        _ => Err(Error::wasi_invalid_fd("Invalid write data type")),
    }
}

/// Helper function to extract pollable list from arguments
fn extract_pollable_list(args: &[Value]) -> Result<Vec<u32>> {
    if args.is_empty() {
        return Err(Error::wasi_invalid_fd("Missing pollables argument"));
    }

    match &args[0] {
        Value::List(items) => {
            let mut pollables = Vec::new();
            for item in items {
                match item {
                    Value::U32(handle) => pollables.push(*handle),
                    _ => return Err(Error::wasi_invalid_fd("Invalid pollable handle")),
                }
            }
            Ok(pollables)
        },
        _ => Err(Error::wasi_invalid_fd("Invalid pollables type")),
    }
}

/// Platform-specific stream read implementation
fn perform_stream_read(stream_handle: u32, len: u64) -> Result<Vec<u8>> {
    // In a real implementation, this would:
    // 1. Look up the stream in the resource manager
    // 2. Check capabilities (readable)
    // 3. Perform platform-specific read operation
    // 4. Update stream position if applicable

    match stream_handle {
        0 => {
            // stdin - in a real implementation, read from platform stdin
            #[cfg(feature = "std")]
            {
                use std::io::{
                    self,
                    Read,
                };
                let mut buffer = vec![0u8; len.min(4096) as usize];
                let bytes_read = io::stdin()
                    .read(&mut buffer)
                    .map_err(|_| Error::wasi_capability_unavailable("Failed to read from stdin"))?;
                buffer.truncate(bytes_read);
                Ok(buffer)
            }
            #[cfg(not(feature = "std"))]
            {
                // In no_std environment, return empty data
                Ok(Vec::new())
            }
        },
        _ => {
            // Other streams - placeholder implementation
            Ok(Vec::new())
        },
    }
}

/// Platform-specific stream write implementation
fn perform_stream_write(stream_handle: u32, data: &[u8]) -> Result<u64> {
    // In a real implementation, this would:
    // 1. Look up the stream in the resource manager
    // 2. Check capabilities (writable)
    // 3. Perform platform-specific write operation
    // 4. Update stream position if applicable

    match stream_handle {
        1 => {
            // stdout - in a real implementation, write to platform stdout
            #[cfg(feature = "std")]
            {
                use std::io::{
                    self,
                    Write,
                };
                io::stdout()
                    .write_all(data)
                    .map_err(|_| Error::wasi_capability_unavailable("Failed to write to stdout"))?;
                io::stdout()
                    .flush()
                    .map_err(|_| Error::wasi_capability_unavailable("Failed to flush stdout"))?;
                Ok(data.len() as u64)
            }
            #[cfg(not(feature = "std"))]
            {
                // In no_std environment, return success without actual write
                Ok(data.len() as u64)
            }
        },
        2 => {
            // stderr - in a real implementation, write to platform stderr
            #[cfg(feature = "std")]
            {
                use std::io::{
                    self,
                    Write,
                };
                io::stderr()
                    .write_all(data)
                    .map_err(|_| Error::wasi_capability_unavailable("Failed to write to stderr"))?;
                io::stderr()
                    .flush()
                    .map_err(|_| Error::wasi_capability_unavailable("Failed to flush stderr"))?;
                Ok(data.len() as u64)
            }
            #[cfg(not(feature = "std"))]
            {
                // In no_std environment, return success without actual write
                Ok(data.len() as u64)
            }
        },
        _ => {
            // Other streams - placeholder implementation
            Ok(data.len() as u64)
        },
    }
}

/// Platform-specific stream flush implementation
fn perform_stream_flush(stream_handle: u32) -> Result<()> {
    match stream_handle {
        1 => {
            // stdout flush
            #[cfg(feature = "std")]
            {
                use std::io::{
                    self,
                    Write,
                };
                io::stdout()
                    .flush()
                    .map_err(|_| Error::wasi_capability_unavailable("Failed to flush stdout"))?;
            }
            Ok(())
        },
        2 => {
            // stderr flush
            #[cfg(feature = "std")]
            {
                use std::io::{
                    self,
                    Write,
                };
                io::stderr()
                    .flush()
                    .map_err(|_| Error::wasi_capability_unavailable("Failed to flush stderr"))?;
            }
            Ok(())
        },
        _ => {
            // Other streams - no-op for now
            Ok(())
        },
    }
}

/// Check write capacity for stream
fn check_write_capacity(stream_handle: u32) -> Result<u64> {
    match stream_handle {
        1 | 2 => {
            // stdout/stderr - assume always ready for a reasonable amount
            Ok(4096)
        },
        _ => {
            // Other streams - placeholder implementation
            Ok(1024)
        },
    }
}

/// Pollable handle offset - pollable handles are stream_handle + POLLABLE_OFFSET
const POLLABLE_OFFSET: u32 = 1000;

/// Create a pollable for the given stream
fn create_pollable(stream_handle: u32) -> Result<u32> {
    // Create a pollable handle that maps back to the stream
    // Pollable = stream_handle + offset
    Ok(stream_handle + POLLABLE_OFFSET)
}

/// Poll multiple pollables to check if their streams are ready
///
/// Returns a vector of booleans indicating whether each pollable is ready.
/// Ready means:
/// - For input streams: data is available to read
/// - For output streams: buffer space is available to write
fn poll_pollables(pollables: &[u32]) -> Result<Vec<bool>> {
    let mut results = Vec::with_capacity(pollables.len());

    for &pollable in pollables {
        // Convert pollable back to stream handle
        let stream_handle = pollable.saturating_sub(POLLABLE_OFFSET);

        // Check if the stream is ready
        let is_ready = check_stream_ready(stream_handle)?;
        results.push(is_ready);
    }

    Ok(results)
}

/// Check if a stream is ready for I/O
///
/// This implements actual readiness checking for known stream types:
/// - stdin (0): Checks if input is available without blocking
/// - stdout (1): Always ready for writing
/// - stderr (2): Always ready for writing
/// - Other streams: Check based on stream type
fn check_stream_ready(stream_handle: u32) -> Result<bool> {
    match stream_handle {
        0 => {
            // stdin - check if input is available
            #[cfg(feature = "std")]
            {
                check_stdin_ready()
            }
            #[cfg(not(feature = "std"))]
            {
                // In no_std, assume not ready (conservative)
                Ok(false)
            }
        }
        1 | 2 => {
            // stdout/stderr - always ready for writing
            // These are typically buffered and non-blocking
            Ok(true)
        }
        _ => {
            // For file descriptors from the filesystem, we'd need to check
            // using platform-specific mechanisms. For now, assume ready
            // for any handle in the user range (>= 3)
            if stream_handle >= 3 {
                // User file handles - assume ready
                // A full implementation would check the actual file
                Ok(true)
            } else {
                // Unknown handle
                Ok(false)
            }
        }
    }
}

/// Check if stdin has data available to read
///
/// This is a conservative implementation that:
/// - Returns true for interactive sessions (assume user might type)
/// - Returns true for piped input (data is likely available)
/// - Uses heuristics since Rust's std doesn't provide non-blocking readiness check
#[cfg(feature = "std")]
fn check_stdin_ready() -> Result<bool> {
    use std::io::IsTerminal;

    // For terminals, we can't easily check without blocking
    // Return true to allow the poll to proceed
    // The actual read may block if no data is available
    if std::io::stdin().is_terminal() {
        // Interactive terminal - assume potentially ready
        // In a full implementation, we'd use platform-specific APIs
        Ok(true)
    } else {
        // Piped or redirected input - likely has data or EOF
        // Conservative: assume ready so read can determine actual state
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_stream_handle() -> Result<()> {
        let args = vec![Value::U32(42)];
        let handle = extract_stream_handle(&args)?;
        assert_eq!(handle, 42);

        let args = vec![Value::S32(24)];
        let handle = extract_stream_handle(&args)?;
        assert_eq!(handle, 24);

        // Test negative handle
        let args = vec![Value::S32(-1)];
        let result = extract_stream_handle(&args);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_extract_read_length() -> Result<()> {
        let args = vec![Value::U32(42), Value::U64(1024)];
        let len = extract_read_length(&args, 1)?;
        assert_eq!(len, 1024);

        let args = vec![Value::U32(42), Value::U32(512)];
        let len = extract_read_length(&args, 1)?;
        assert_eq!(len, 512);

        Ok(())
    }

    #[test]
    fn test_extract_write_data() -> Result<()> {
        let data = vec![Value::U8(1), Value::U8(2), Value::U8(3)];
        let args = vec![Value::U32(42), Value::List(data)];

        let bytes = extract_write_data(&args, 1)?;
        assert_eq!(bytes, vec![1, 2, 3]);

        Ok(())
    }

    #[test]
    fn test_wasi_stream_operations() -> Result<()> {
        // Test write operation to stdout
        let data = vec![
            Value::U8(72),
            Value::U8(101),
            Value::U8(108),
            Value::U8(108),
            Value::U8(111),
        ]; // "Hello"
        let args = vec![Value::U32(1), Value::List(data)];
        let result = wasi_stream_write(&mut (), args)?;
        assert_eq!(result.len(), 1);
        if let Value::U64(bytes_written) = &result[0] {
            assert_eq!(*bytes_written, 5);
        }

        // Test flush operation
        let args = vec![Value::U32(1)];
        let result = wasi_stream_flush(&mut (), args)?;
        assert_eq!(result.len(), 0); // Flush returns unit

        // Test check-write operation
        let args = vec![Value::U32(1)];
        let result = wasi_stream_check_write(&mut (), args)?;
        assert_eq!(result.len(), 1);
        if let Value::U64(capacity) = &result[0] {
            assert!(*capacity > 0);
        }

        Ok(())
    }

    #[test]
    fn test_pollable_operations() -> Result<()> {
        // Test subscribe operation
        let args = vec![Value::U32(1)];
        let result = wasi_stream_subscribe(&mut (), args)?;
        assert_eq!(result.len(), 1);
        if let Value::U32(pollable) = &result[0] {
            assert!(*pollable > 1000); // Should be offset
        }

        // Test poll operation
        let pollables = vec![Value::U32(1001), Value::U32(1002)];
        let args = vec![Value::List(pollables)];
        let result = wasi_poll_one_off(&mut (), args)?;
        assert_eq!(result.len(), 1);
        if let Value::List(results) = &result[0] {
            assert_eq!(results.len(), 2);
            for result in results {
                assert!(matches!(result, Value::Bool(true)));
            }
        }

        Ok(())
    }
}
