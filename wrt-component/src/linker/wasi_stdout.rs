//! WASI stdout implementation for Component Model
//!
//! Implements the minimal WASI functions needed to print output:
//! - wasi:cli/stdout@0.2.0 → get-stdout
//! - wasi:io/streams@0.2.0 → [method]output-stream.blocking-write-and-flush

use wrt_error::Result;

#[cfg(feature = "std")]
use std::io::Write;

/// WASI stdout handle (file descriptor 1)
const STDOUT_HANDLE: u32 = 1;

/// Get stdout output stream handle
///
/// Returns a handle to stdout that can be used with write operations
pub fn wasi_get_stdout() -> Result<u32> {
    #[cfg(feature = "std")]
    eprintln!("[WASI-STDOUT] wasi_get_stdout() called!");
    Ok(STDOUT_HANDLE)
}

/// Write bytes to an output stream and flush
///
/// # Arguments
/// * `handle` - Output stream handle (1 for stdout)
/// * `bytes` - Bytes to write
///
/// # Returns
/// Number of bytes written
#[cfg(feature = "std")]
pub fn wasi_blocking_write_and_flush(handle: u32, bytes: &[u8]) -> Result<u64> {
    eprintln!("[WASI-STDOUT] wasi_blocking_write_and_flush() called with {} bytes", bytes.len());
    eprintln!("[WASI-STDOUT] Data: {:?}", String::from_utf8_lossy(bytes));

    if handle == STDOUT_HANDLE {
        std::io::stdout().write_all(bytes)
            .map_err(|_| wrt_error::Error::runtime_error("Write failed"))?;

        std::io::stdout().flush()
            .map_err(|_| wrt_error::Error::runtime_error("Flush failed"))?;

        eprintln!("[WASI-STDOUT] Successfully wrote {} bytes to stdout", bytes.len());
        Ok(bytes.len() as u64)
    } else {
        Err(wrt_error::Error::runtime_error("Invalid output stream handle"))
    }
}

/// Write bytes to an output stream and flush (no_std stub)
#[cfg(not(feature = "std"))]
pub fn wasi_blocking_write_and_flush(handle: u32, bytes: &[u8]) -> Result<u64> {
    // In no_std, we can't write to stdout
    // This is a stub that always succeeds
    if handle == STDOUT_HANDLE {
        Ok(bytes.len() as u64)
    } else {
        Err(wrt_error::Error::runtime_error("Invalid output stream handle"))
    }
}

/// Write bytes to stdout (convenience function)
pub fn write_stdout(bytes: &[u8]) -> Result<()> {
    wasi_blocking_write_and_flush(STDOUT_HANDLE, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_stdout() {
        let handle = wasi_get_stdout().unwrap();
        assert_eq!(handle, 1);
    }

    #[test]
    fn test_write_stdout() {
        let data = b"Test output\n";
        let result = wasi_blocking_write_and_flush(STDOUT_HANDLE, data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data.len() as u64);
    }
}
