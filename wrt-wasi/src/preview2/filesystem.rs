//! WASI filesystem interface implementation
//!
//! Implements the `wasi:filesystem` interface for file operations using WRT's
//! resource management patterns and platform abstractions.

use core::any::Any;

#[cfg(feature = "std")]
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::{Mutex, RwLock},
};

use crate::{
    prelude::*,
    Value,
};

/// File descriptor table for tracking open files
#[cfg(feature = "std")]
static FILE_TABLE: RwLock<Option<FileDescriptorTable>> = RwLock::new(None);

/// File descriptor table implementation
#[cfg(feature = "std")]
pub struct FileDescriptorTable {
    /// Map of file descriptors to open files
    files: HashMap<u32, OpenFile>,
    /// Next available file descriptor (starts at 3 after stdin/stdout/stderr)
    next_fd: u32,
}

#[cfg(feature = "std")]
impl FileDescriptorTable {
    /// Create a new file descriptor table
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            next_fd: 3, // 0=stdin, 1=stdout, 2=stderr
        }
    }

    /// Allocate a new file descriptor
    fn allocate(&mut self) -> u32 {
        let fd = self.next_fd;
        self.next_fd += 1;
        fd
    }

    /// Insert a file and return its descriptor
    fn insert(&mut self, file: OpenFile) -> u32 {
        let fd = self.allocate();
        self.files.insert(fd, file);
        fd
    }

    /// Get a file by descriptor
    fn get(&self, fd: u32) -> Option<&OpenFile> {
        self.files.get(&fd)
    }

    /// Get a mutable file by descriptor
    fn get_mut(&mut self, fd: u32) -> Option<&mut OpenFile> {
        self.files.get_mut(&fd)
    }

    /// Remove a file by descriptor
    fn remove(&mut self, fd: u32) -> Option<OpenFile> {
        self.files.remove(&fd)
    }
}

#[cfg(feature = "std")]
impl Default for FileDescriptorTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an open file with its associated metadata
#[cfg(feature = "std")]
pub struct OpenFile {
    /// The underlying file handle
    file: File,
    /// The path to the file
    path: PathBuf,
    /// Whether the file is readable
    readable: bool,
    /// Whether the file is writable
    writable: bool,
}

/// Initialize the file descriptor table
#[cfg(feature = "std")]
fn ensure_file_table() -> Result<()> {
    let mut table = FILE_TABLE.write()
        .map_err(|_| Error::wasi_capability_unavailable("Failed to acquire file table lock"))?;
    if table.is_none() {
        *table = Some(FileDescriptorTable::new());
    }
    Ok(())
}

// ============================================================================
// WASI Filesystem Operations
// ============================================================================

/// WASI filesystem open-at operation
///
/// Opens a file relative to a directory descriptor.
/// Implements `wasi:filesystem/types.open-at`
pub fn wasi_filesystem_open_at(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    #[cfg(feature = "std")]
    {
        ensure_file_table()?;

        // Extract arguments: dir_fd, path_flags, path, open_flags, descriptor_flags
        let _dir_fd = extract_file_descriptor(&args)?;
        let path = extract_string(&args, 2)?;
        let open_flags = extract_open_flags(&args, 3)?;
        let descriptor_flags = extract_descriptor_flags(&args, 4)?;

        // Build open options
        let mut options = OpenOptions::new();

        // Handle open flags
        if open_flags.create {
            options.create(true);
        }
        if open_flags.exclusive {
            options.create_new(true);
        }
        if open_flags.truncate {
            options.truncate(true);
        }

        // Handle descriptor flags (read/write permissions)
        let readable = descriptor_flags.read;
        let writable = descriptor_flags.write;

        if readable {
            options.read(true);
        }
        if writable {
            options.write(true);
        }

        // Open the file
        let file = options.open(path)
            .map_err(|e| map_io_error(e, "Failed to open file"))?;

        // Register in file table
        let mut table = FILE_TABLE.write()
            .map_err(|_| Error::wasi_capability_unavailable("Failed to acquire file table lock"))?;
        let table = table.as_mut()
            .ok_or_else(|| Error::wasi_capability_unavailable("File table not initialized"))?;

        let open_file = OpenFile {
            file,
            path: PathBuf::from(path),
            readable,
            writable,
        };

        let fd = table.insert(open_file);

        // Return result<descriptor, error-code>
        Ok(vec![Value::Result(Ok(Box::new(Value::U32(fd))))])
    }

    #[cfg(not(feature = "std"))]
    {
        // In no_std environment, filesystem operations are not available
        Ok(vec![Value::Result(Err(Box::new(Value::U32(76))))])  // ENOSYS
    }
}

/// WASI filesystem read operation
///
/// Reads data from a file descriptor.
/// Implements `wasi:filesystem/types.read`
pub fn wasi_filesystem_read(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    #[cfg(feature = "std")]
    {
        ensure_file_table()?;

        let fd = extract_file_descriptor(&args)?;
        let len = extract_length(&args, 1)? as usize;
        let offset = args.get(2)
            .and_then(|v| match v {
                Value::U64(o) => Some(*o),
                _ => None,
            });

        let mut table = FILE_TABLE.write()
            .map_err(|_| Error::wasi_capability_unavailable("Failed to acquire file table lock"))?;
        let table = table.as_mut()
            .ok_or_else(|| Error::wasi_capability_unavailable("File table not initialized"))?;

        let open_file = table.get_mut(fd)
            .ok_or_else(|| Error::wasi_invalid_fd("Invalid file descriptor"))?;

        if !open_file.readable {
            return Ok(vec![Value::Result(Err(Box::new(Value::U32(8))))]);  // EBADF
        }

        // Seek if offset provided
        if let Some(off) = offset {
            open_file.file.seek(SeekFrom::Start(off))
                .map_err(|e| map_io_error(e, "Failed to seek"))?;
        }

        // Read data
        let mut buffer = vec![0u8; len.min(65536)];
        let bytes_read = open_file.file.read(&mut buffer)
            .map_err(|e| map_io_error(e, "Failed to read file"))?;

        buffer.truncate(bytes_read);

        // Convert to WASI list<u8>
        let data: Vec<Value> = buffer.into_iter().map(Value::U8).collect();

        // Return tuple of (data, eof)
        let eof = bytes_read == 0;
        Ok(vec![
            Value::Result(Ok(Box::new(Value::Tuple(vec![
                Value::List(data),
                Value::Bool(eof),
            ]))))
        ])
    }

    #[cfg(not(feature = "std"))]
    {
        Ok(vec![Value::Result(Err(Box::new(Value::U32(76))))])  // ENOSYS
    }
}

/// WASI filesystem write operation
///
/// Writes data to a file descriptor.
/// Implements `wasi:filesystem/types.write`
pub fn wasi_filesystem_write(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    #[cfg(feature = "std")]
    {
        ensure_file_table()?;

        let fd = extract_file_descriptor(&args)?;
        let data = extract_byte_data(&args, 1)?;
        let offset = args.get(2)
            .and_then(|v| match v {
                Value::U64(o) => Some(*o),
                _ => None,
            });

        let mut table = FILE_TABLE.write()
            .map_err(|_| Error::wasi_capability_unavailable("Failed to acquire file table lock"))?;
        let table = table.as_mut()
            .ok_or_else(|| Error::wasi_capability_unavailable("File table not initialized"))?;

        let open_file = table.get_mut(fd)
            .ok_or_else(|| Error::wasi_invalid_fd("Invalid file descriptor"))?;

        if !open_file.writable {
            return Ok(vec![Value::Result(Err(Box::new(Value::U32(8))))]);  // EBADF
        }

        // Seek if offset provided
        if let Some(off) = offset {
            open_file.file.seek(SeekFrom::Start(off))
                .map_err(|e| map_io_error(e, "Failed to seek"))?;
        }

        // Write data
        let bytes_written = open_file.file.write(&data)
            .map_err(|e| map_io_error(e, "Failed to write file"))?;

        Ok(vec![Value::Result(Ok(Box::new(Value::U64(bytes_written as u64))))])
    }

    #[cfg(not(feature = "std"))]
    {
        Ok(vec![Value::Result(Err(Box::new(Value::U32(76))))])  // ENOSYS
    }
}

/// WASI filesystem close operation
///
/// Closes a file descriptor.
/// Implements closing via dropping the descriptor
pub fn wasi_filesystem_close(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    #[cfg(feature = "std")]
    {
        ensure_file_table()?;

        let fd = extract_file_descriptor(&args)?;

        let mut table = FILE_TABLE.write()
            .map_err(|_| Error::wasi_capability_unavailable("Failed to acquire file table lock"))?;
        let table = table.as_mut()
            .ok_or_else(|| Error::wasi_capability_unavailable("File table not initialized"))?;

        if table.remove(fd).is_some() {
            Ok(vec![])
        } else {
            Err(Error::wasi_invalid_fd("Invalid file descriptor"))
        }
    }

    #[cfg(not(feature = "std"))]
    {
        Err(Error::wasi_capability_unavailable("Filesystem not available in no_std"))
    }
}

/// WASI filesystem stat operation
///
/// Gets file metadata.
/// Implements `wasi:filesystem/types.stat`
pub fn wasi_filesystem_stat(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    #[cfg(feature = "std")]
    {
        ensure_file_table()?;

        let fd = extract_file_descriptor(&args)?;

        let table = FILE_TABLE.read()
            .map_err(|_| Error::wasi_capability_unavailable("Failed to acquire file table lock"))?;
        let table = table.as_ref()
            .ok_or_else(|| Error::wasi_capability_unavailable("File table not initialized"))?;

        let open_file = table.get(fd)
            .ok_or_else(|| Error::wasi_invalid_fd("Invalid file descriptor"))?;

        let metadata = open_file.file.metadata()
            .map_err(|e| map_io_error(e, "Failed to get file metadata"))?;

        // Build descriptor-stat record
        let stat = build_descriptor_stat(&metadata);

        Ok(vec![Value::Result(Ok(Box::new(stat)))])
    }

    #[cfg(not(feature = "std"))]
    {
        Ok(vec![Value::Result(Err(Box::new(Value::U32(76))))])  // ENOSYS
    }
}

/// WASI filesystem stat-at operation
///
/// Gets file metadata by path relative to a directory.
/// Implements `wasi:filesystem/types.stat-at`
pub fn wasi_filesystem_stat_at(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    #[cfg(feature = "std")]
    {
        let _dir_fd = extract_file_descriptor(&args)?;
        let path = extract_string(&args, 2)?;

        let metadata = std::fs::metadata(path)
            .map_err(|e| map_io_error(e, "Failed to get file metadata"))?;

        let stat = build_descriptor_stat(&metadata);

        Ok(vec![Value::Result(Ok(Box::new(stat)))])
    }

    #[cfg(not(feature = "std"))]
    {
        Ok(vec![Value::Result(Err(Box::new(Value::U32(76))))])  // ENOSYS
    }
}

/// WASI filesystem sync operation
///
/// Syncs file data to disk.
/// Implements `wasi:filesystem/types.sync`
pub fn wasi_filesystem_sync(_target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
    #[cfg(feature = "std")]
    {
        ensure_file_table()?;

        let fd = extract_file_descriptor(&args)?;

        let table = FILE_TABLE.read()
            .map_err(|_| Error::wasi_capability_unavailable("Failed to acquire file table lock"))?;
        let table = table.as_ref()
            .ok_or_else(|| Error::wasi_capability_unavailable("File table not initialized"))?;

        let open_file = table.get(fd)
            .ok_or_else(|| Error::wasi_invalid_fd("Invalid file descriptor"))?;

        open_file.file.sync_all()
            .map_err(|e| map_io_error(e, "Failed to sync file"))?;

        Ok(vec![Value::Result(Ok(Box::new(Value::Tuple(vec![]))))])
    }

    #[cfg(not(feature = "std"))]
    {
        Ok(vec![Value::Result(Err(Box::new(Value::U32(76))))])  // ENOSYS
    }
}

// ============================================================================
// Helper Types and Functions
// ============================================================================

/// Open flags for file operations
#[derive(Debug, Default)]
struct OpenFlags {
    create: bool,
    directory: bool,
    exclusive: bool,
    truncate: bool,
}

/// Descriptor flags for file operations
#[derive(Debug, Default)]
struct DescriptorFlags {
    read: bool,
    write: bool,
    sync: bool,
    nonblocking: bool,
}

/// Helper function to extract file descriptor from WASI arguments
fn extract_file_descriptor(args: &[Value]) -> Result<u32> {
    args.first()
        .and_then(|v| match v {
            Value::U32(fd) => Some(*fd),
            _ => None,
        })
        .ok_or_else(|| Error::parameter_wasi_invalid_fd("Invalid file descriptor argument"))
}

/// Helper function to extract length parameter from WASI arguments
fn extract_length(args: &[Value], index: usize) -> Result<u64> {
    args.get(index)
        .and_then(|v| match v {
            Value::U64(len) => Some(*len),
            Value::U32(len) => Some(u64::from(*len)),
            _ => None,
        })
        .ok_or_else(|| Error::parameter_wasi_invalid_fd("Invalid length argument"))
}

/// Helper function to extract string from WASI arguments
fn extract_string(args: &[Value], index: usize) -> Result<&str> {
    args.get(index)
        .and_then(|v| match v {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        })
        .ok_or_else(|| Error::parameter_wasi_invalid_fd("Invalid string argument"))
}

/// Helper function to extract byte data from WASI arguments
fn extract_byte_data(args: &[Value], index: usize) -> Result<Vec<u8>> {
    args.get(index)
        .and_then(|v| match v {
            Value::List(list) => {
                let mut bytes = Vec::new();
                for item in list {
                    match item {
                        Value::U8(byte) => bytes.push(*byte),
                        _ => return None,
                    }
                }
                Some(bytes)
            },
            _ => None,
        })
        .ok_or_else(|| Error::parameter_wasi_invalid_fd("Invalid byte data argument"))
}

/// Extract open flags from arguments
fn extract_open_flags(args: &[Value], index: usize) -> Result<OpenFlags> {
    let flags_val = args.get(index)
        .ok_or_else(|| Error::parameter_wasi_invalid_fd("Missing open flags"))?;

    match flags_val {
        Value::U32(bits) => {
            Ok(OpenFlags {
                create: (bits & 0x01) != 0,
                directory: (bits & 0x02) != 0,
                exclusive: (bits & 0x04) != 0,
                truncate: (bits & 0x08) != 0,
            })
        },
        Value::Record(fields) => {
            // Handle record-style flags
            let mut flags = OpenFlags::default();
            for (name, value) in fields.iter() {
                match (name.as_str(), value) {
                    ("create", Value::Bool(b)) => flags.create = *b,
                    ("directory", Value::Bool(b)) => flags.directory = *b,
                    ("exclusive", Value::Bool(b)) => flags.exclusive = *b,
                    ("truncate", Value::Bool(b)) => flags.truncate = *b,
                    _ => {}
                }
            }
            Ok(flags)
        },
        _ => Ok(OpenFlags::default()),
    }
}

/// Extract descriptor flags from arguments
fn extract_descriptor_flags(args: &[Value], index: usize) -> Result<DescriptorFlags> {
    let flags_val = args.get(index)
        .ok_or_else(|| Error::parameter_wasi_invalid_fd("Missing descriptor flags"))?;

    match flags_val {
        Value::U32(bits) => {
            Ok(DescriptorFlags {
                read: (bits & 0x01) != 0,
                write: (bits & 0x02) != 0,
                sync: (bits & 0x04) != 0,
                nonblocking: (bits & 0x10) != 0,
            })
        },
        Value::Record(fields) => {
            let mut flags = DescriptorFlags::default();
            for (name, value) in fields.iter() {
                match (name.as_str(), value) {
                    ("read", Value::Bool(b)) => flags.read = *b,
                    ("write", Value::Bool(b)) => flags.write = *b,
                    ("sync", Value::Bool(b)) => flags.sync = *b,
                    ("nonblocking", Value::Bool(b)) => flags.nonblocking = *b,
                    _ => {}
                }
            }
            Ok(flags)
        },
        _ => Ok(DescriptorFlags { read: true, ..Default::default() }),
    }
}

/// Map std::io::Error to WASI error
#[cfg(feature = "std")]
fn map_io_error(e: std::io::Error, _context: &str) -> Error {
    use std::io::ErrorKind;

    match e.kind() {
        ErrorKind::NotFound => Error::wasi_invalid_fd("File not found"),
        ErrorKind::PermissionDenied => Error::wasi_permission_denied("Permission denied"),
        ErrorKind::AlreadyExists => Error::wasi_invalid_argument("File already exists"),
        ErrorKind::InvalidInput => Error::wasi_invalid_argument("Invalid input"),
        ErrorKind::NotADirectory => Error::wasi_invalid_argument("Not a directory"),
        ErrorKind::IsADirectory => Error::wasi_invalid_argument("Is a directory"),
        ErrorKind::DirectoryNotEmpty => Error::wasi_invalid_argument("Directory not empty"),
        ErrorKind::WouldBlock => Error::wasi_resource_exhausted("Would block"),
        ErrorKind::TimedOut => Error::wasi_timeout("Operation timed out"),
        _ => Error::wasi_runtime_error("I/O error"),
    }
}

/// Build a descriptor-stat record from file metadata
#[cfg(feature = "std")]
fn build_descriptor_stat(metadata: &std::fs::Metadata) -> Value {
    use std::time::UNIX_EPOCH;

    let file_type = if metadata.is_dir() {
        3u8  // directory
    } else if metadata.is_file() {
        4u8  // regular-file
    } else if metadata.is_symlink() {
        7u8  // symbolic-link
    } else {
        0u8  // unknown
    };

    let size = metadata.len();

    // Get timestamps (nanoseconds since UNIX epoch)
    let atime = metadata.accessed()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    let mtime = metadata.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    let ctime = mtime; // Use mtime as ctime fallback

    Value::Record(vec![
        ("type".to_string(), Value::U8(file_type)),
        ("link-count".to_string(), Value::U64(1)),
        ("size".to_string(), Value::U64(size)),
        ("data-access-timestamp".to_string(), Value::U64(atime)),
        ("data-modification-timestamp".to_string(), Value::U64(mtime)),
        ("status-change-timestamp".to_string(), Value::U64(ctime)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_descriptor() {
        let args = vec![Value::U32(42)];
        assert_eq!(extract_file_descriptor(&args).unwrap(), 42);

        let invalid_args = vec![Value::String("not_a_fd".to_string())];
        assert!(extract_file_descriptor(&invalid_args).is_err());
    }

    #[test]
    fn test_extract_length() {
        let args = vec![Value::U32(0), Value::U64(1024)];
        assert_eq!(extract_length(&args, 1).unwrap(), 1024);

        let args_u32 = vec![Value::U32(0), Value::U32(512)];
        assert_eq!(extract_length(&args_u32, 1).unwrap(), 512);
    }

    #[test]
    fn test_extract_byte_data() -> Result<()> {
        let data = vec![Value::U8(1), Value::U8(2), Value::U8(3)];
        let args = vec![Value::U32(42), Value::List(data)];

        let bytes = extract_byte_data(&args, 1)?;
        assert_eq!(bytes, vec![1, 2, 3]);

        Ok(())
    }

    #[test]
    fn test_extract_string() -> Result<()> {
        let args = vec![Value::U32(42), Value::String("test.txt".to_string())];
        let path = extract_string(&args, 1)?;
        assert_eq!(path, "test.txt");

        Ok(())
    }

    #[test]
    fn test_open_flags_extraction() {
        // Test bit-based flags
        let args = vec![Value::U32(0), Value::U32(0), Value::String("test".to_string()), Value::U32(0x05)];
        let flags = extract_open_flags(&args, 3).unwrap();
        assert!(flags.create);
        assert!(!flags.directory);
        assert!(flags.exclusive);
        assert!(!flags.truncate);
    }

    #[test]
    fn test_descriptor_flags_extraction() {
        let args = vec![Value::U32(0), Value::U32(0), Value::String("test".to_string()), Value::U32(0), Value::U32(0x03)];
        let flags = extract_descriptor_flags(&args, 4).unwrap();
        assert!(flags.read);
        assert!(flags.write);
        assert!(!flags.sync);
    }
}
