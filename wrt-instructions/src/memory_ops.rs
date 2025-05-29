// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Memory operations for WebAssembly instructions.
//!
//! This module provides implementations for WebAssembly memory access
//! instructions, including load and store operations for various value types.
//!
//! # Memory Operation Architecture
//!
//! This module separates memory operations from the underlying memory
//! implementation, allowing different execution engines to share the same
//! memory access code. The key components are:
//!
//! - `MemoryLoad`: Handles all WebAssembly load operations with various data
//!   types and widths
//! - `MemoryStore`: Handles all WebAssembly store operations with various data
//!   types and widths
//!
//! Both structures work with the `Memory` implementation from the `wrt-runtime`
//! crate.
//!
//! # Features
//!
//! - Support for all WebAssembly numeric types (i32, i64, f32, f64)
//! - Support for partial-width loads and stores (8-bit, 16-bit, 32-bit)
//! - Signed and unsigned operations
//! - Alignment checking and validation
//! - Bounds checking
//!
//! # Memory Safety
//!
//! All memory operations perform proper bounds and alignment checking before
//! accessing memory. This ensures that WebAssembly's memory safety guarantees
//! are preserved.
//!
//! # Usage
//!
//! ```no_run
//! use wrt_instructions::memory_ops::{MemoryLoad, MemoryStore};
//! use wrt_instructions::Value;
//! use wrt_runtime::Memory;
//! use wrt_foundation::types::Limits;
//!
//! // Create a memory instance
//! let mem_type = MemoryType {
//!     limits: Limits { min: 1, max: Some(2) },
//! };
//! let mut memory = Memory::new(mem_type).unwrap();
//!
//! // Store a value
//! let store = MemoryStore::i32(0, 4); // offset 0, 4-byte aligned
//! store.execute(&mut memory, &Value::I32(0), &Value::I32(42)).unwrap();
//!
//! // Load a value
//! let load = MemoryLoad::i32(0, 4); // offset 0, 4-byte aligned
//! let result = load.execute(&memory, &Value::I32(0)).unwrap();
//! assert_eq!(result, Value::I32(42));
//! ```

use crate::prelude::*;
use crate::validation::{Validate, ValidationContext, validate_memory_op};


/// Memory trait defining the requirements for memory operations
pub trait MemoryOperations {
    /// Read bytes from memory
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn read_bytes(&self, offset: u32, len: u32) -> Result<Vec<u8>>;
    
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn read_bytes(&self, offset: u32, len: u32) -> Result<wrt_foundation::BoundedVec<u8, 65536, wrt_foundation::NoStdProvider<65536>>>;

    /// Write bytes to memory
    fn write_bytes(&mut self, offset: u32, bytes: &[u8]) -> Result<()>;

    /// Get the size of memory in bytes
    fn size_in_bytes(&self) -> Result<usize>;

    /// Grow memory by the specified number of pages
    fn grow(&mut self, pages: u32) -> Result<u32>;

    /// Fill memory region with a byte value (bulk memory operation)
    fn fill(&mut self, offset: u32, value: u8, size: u32) -> Result<()>;

    /// Copy memory region within the same memory (bulk memory operation)
    fn copy(&mut self, dest: u32, src: u32, size: u32) -> Result<()>;
}

/// Memory load operation
#[derive(Debug, Clone)]
pub struct MemoryLoad {
    /// Memory index (for multi-memory support)
    pub memory_index: u32,
    /// Memory offset
    pub offset: u32,
    /// Required alignment
    pub align: u32,
    /// Value type to load
    pub value_type: ValueType,
    /// Whether this is a signed load (for smaller-than-register loads)
    pub signed: bool,
    /// Memory access width in bytes (8, 16, 32, 64)
    pub width: u32,
}

/// Memory store operation
#[derive(Debug, Clone)]
pub struct MemoryStore {
    /// Memory index (for multi-memory support)
    pub memory_index: u32,
    /// Memory offset
    pub offset: u32,
    /// Required alignment
    pub align: u32,
    /// Value type to store
    pub value_type: ValueType,
    /// Memory access width in bytes (8, 16, 32, 64)
    pub width: u32,
}

impl MemoryLoad {
    /// Creates a new i32 load operation
    ///
    /// # Arguments
    ///
    /// * `memory_index` - Memory index (for multi-memory support)
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryLoad for i32 values
    pub fn i32(memory_index: u32, offset: u32, align: u32) -> Self {
        Self { memory_index, offset, align, value_type: ValueType::I32, signed: false, width: 32 }
    }

    /// Creates a new i32 load operation (legacy - assumes memory 0)
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryLoad for i32 values from memory 0
    pub fn i32_legacy(offset: u32, align: u32) -> Self {
        Self::i32(0, offset, align)
    }

    /// Creates a new i64 load operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryLoad for i64 values
    pub fn i64(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::I64, signed: false, width: 64 }
    }

    /// Creates a new f32 load operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryLoad for f32 values
    pub fn f32(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::F32, signed: false, width: 32 }
    }

    /// Creates a new f64 load operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryLoad for f64 values
    pub fn f64(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::F64, signed: false, width: 64 }
    }

    /// Creates a new i32 load8 operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    /// * `signed` - Whether this is a signed load
    ///
    /// # Returns
    ///
    /// A new MemoryLoad for i32 values loading from 8-bit memory
    pub fn i32_load8(offset: u32, align: u32, signed: bool) -> Self {
        Self { offset, align, value_type: ValueType::I32, signed, width: 8 }
    }

    /// Creates a new i32 load16 operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    /// * `signed` - Whether this is a signed load
    ///
    /// # Returns
    ///
    /// A new MemoryLoad for i32 values loading from 16-bit memory
    pub fn i32_load16(offset: u32, align: u32, signed: bool) -> Self {
        Self { offset, align, value_type: ValueType::I32, signed, width: 16 }
    }

    /// Creates a new i64 load8 operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    /// * `signed` - Whether this is a signed load
    ///
    /// # Returns
    ///
    /// A new MemoryLoad for i64 values loading from 8-bit memory
    pub fn i64_load8(offset: u32, align: u32, signed: bool) -> Self {
        Self { offset, align, value_type: ValueType::I64, signed, width: 8 }
    }

    /// Creates a new i64 load16 operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    /// * `signed` - Whether this is a signed load
    ///
    /// # Returns
    ///
    /// A new MemoryLoad for i64 values loading from 16-bit memory
    pub fn i64_load16(offset: u32, align: u32, signed: bool) -> Self {
        Self { offset, align, value_type: ValueType::I64, signed, width: 16 }
    }

    /// Creates a new i64 load32 operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    /// * `signed` - Whether this is a signed load
    ///
    /// # Returns
    ///
    /// A new MemoryLoad for i64 values loading from 32-bit memory
    pub fn i64_load32(offset: u32, align: u32, signed: bool) -> Self {
        Self { offset, align, value_type: ValueType::I64, signed, width: 32 }
    }

    /// Execute the memory load operation
    ///
    /// # Arguments
    ///
    /// * `memory` - The memory to operate on
    /// * `addr_arg` - The base address to load from
    ///
    /// # Returns
    ///
    /// The loaded value or an error
    ///
    /// Returns an error if the memory access is invalid
    pub fn execute(&self, memory: &impl MemoryOperations, addr_arg: &Value) -> Result<Value> {
        // Extract address from argument
        let addr = match addr_arg {
            Value::I32(a) => *a as u32,
            _ => {
                return Err(Error::type_error(
                    "Memory load expects I32 address, got unexpected value"
                ));
            }
        };

        // Calculate effective address
        let effective_addr = addr.checked_add(self.offset).ok_or_else(|| {
            Error::memory_error(
                "Address overflow in memory load"
            )
        })?;

        // Verify alignment if required - make configurable later
        if self.align > 1 && effective_addr % self.align != 0 {
            return Err(Error::memory_error(
                "Unaligned memory access"
            ));
        }

        // Perform the load based on the type and width
        match (self.value_type, self.width) {
            (ValueType::I32, 32) => {
                let bytes = memory.read_bytes(effective_addr, 4)?;
                if bytes.len() < 4 {
                    return Err(Error::memory_error("Insufficient bytes read for i32 value"));
                }
                #[cfg(any(feature = "std", feature = "alloc"))]
                let value = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                let value = {
                    let mut arr = [0u8; 4];
                    for i in 0..4 {
                        arr[i] = bytes.get(i).map_err(|_| Error::memory_error("Index out of bounds"))?;
                    }
                    i32::from_le_bytes(arr)
                };
                Ok(Value::I32(value))
            }
            (ValueType::I64, 64) => {
                let bytes = memory.read_bytes(effective_addr, 8)?;
                if bytes.len() < 8 {
                    return Err(Error::memory_error("Insufficient bytes read for i64 value"));
                }
                #[cfg(any(feature = "std", feature = "alloc"))]
                let value = i64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]);
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                let value = {
                    let mut arr = [0u8; 8];
                    for i in 0..8 {
                        arr[i] = bytes.get(i).map_err(|_| Error::memory_error("Index out of bounds"))?;
                    }
                    i64::from_le_bytes(arr)
                };
                Ok(Value::I64(value))
            }
            (ValueType::F32, 32) => {
                let bytes = memory.read_bytes(effective_addr, 4)?;
                if bytes.len() < 4 {
                    return Err(Error::memory_error("Insufficient bytes read for f32 value"));
                }
                #[cfg(any(feature = "std", feature = "alloc"))]
                let value = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                let value = {
                    let mut arr = [0u8; 4];
                    for i in 0..4 {
                        arr[i] = bytes.get(i).map_err(|_| Error::memory_error("Index out of bounds"))?;
                    }
                    f32::from_le_bytes(arr)
                };
                Ok(Value::F32(wrt_foundation::FloatBits32::from_float(value)))
            }
            (ValueType::F64, 64) => {
                let bytes = memory.read_bytes(effective_addr, 8)?;
                if bytes.len() < 8 {
                    return Err(Error::memory_error("Insufficient bytes read for f64 value"));
                }
                #[cfg(any(feature = "std", feature = "alloc"))]
                let value = f64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]);
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                let value = {
                    let mut arr = [0u8; 8];
                    for i in 0..8 {
                        arr[i] = bytes.get(i).map_err(|_| Error::memory_error("Index out of bounds"))?;
                    }
                    f64::from_le_bytes(arr)
                };
                Ok(Value::F64(wrt_foundation::FloatBits64::from_float(value)))
            }
            (ValueType::I32, 8) => {
                let bytes = memory.read_bytes(effective_addr, 1)?;
                if bytes.is_empty() {
                    return Err(Error::memory_error("Insufficient bytes read for i8 value"));
                }
                #[cfg(any(feature = "std", feature = "alloc"))]
                let byte = bytes.get(0).copied().ok_or_else(|| Error::memory_error("Index out of bounds"))?;
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                let byte = bytes.get(0).map_err(|_| Error::memory_error("Index out of bounds"))?;
                let value = if self.signed { (byte as i8) as i32 } else { byte as i32 };
                Ok(Value::I32(value))
            }
            (ValueType::I64, 8) => {
                let bytes = memory.read_bytes(effective_addr, 1)?;
                if bytes.is_empty() {
                    return Err(Error::memory_error("Insufficient bytes read for i8 value"));
                }
                #[cfg(any(feature = "std", feature = "alloc"))]
                let byte = bytes.get(0).copied().ok_or_else(|| Error::memory_error("Index out of bounds"))?;
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                let byte = bytes.get(0).map_err(|_| Error::memory_error("Index out of bounds"))?;
                let value = if self.signed { (byte as i8) as i64 } else { byte as i64 };
                Ok(Value::I64(value))
            }
            (ValueType::I32, 16) => {
                let bytes = memory.read_bytes(effective_addr, 2)?;
                if bytes.len() < 2 {
                    return Err(Error::memory_error("Insufficient bytes read for i16 value"));
                }
                #[cfg(any(feature = "std", feature = "alloc"))]
                let value = if self.signed {
                    (i16::from_le_bytes([bytes[0], bytes[1]])) as i32
                } else {
                    (u16::from_le_bytes([bytes[0], bytes[1]])) as i32
                };
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                let value = if self.signed {
                    let mut arr = [0u8; 2];
                    for i in 0..2 {
                        arr[i] = bytes.get(i).map_err(|_| Error::memory_error("Index out of bounds"))?;
                    }
                    (i16::from_le_bytes(arr)) as i32
                } else {
                    let mut arr = [0u8; 2];
                    for i in 0..2 {
                        arr[i] = bytes.get(i).map_err(|_| Error::memory_error("Index out of bounds"))?;
                    }
                    (u16::from_le_bytes(arr)) as i32
                };
                Ok(Value::I32(value))
            }
            (ValueType::I64, 16) => {
                let bytes = memory.read_bytes(effective_addr, 2)?;
                if bytes.len() < 2 {
                    return Err(Error::memory_error("Insufficient bytes read for i16 value"));
                }
                #[cfg(any(feature = "std", feature = "alloc"))]
                let value = if self.signed {
                    (i16::from_le_bytes([bytes[0], bytes[1]])) as i64
                } else {
                    (u16::from_le_bytes([bytes[0], bytes[1]])) as i64
                };
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                let value = if self.signed {
                    let mut arr = [0u8; 2];
                    for i in 0..2 {
                        arr[i] = bytes.get(i).map_err(|_| Error::memory_error("Index out of bounds"))?;
                    }
                    (i16::from_le_bytes(arr)) as i64
                } else {
                    let mut arr = [0u8; 2];
                    for i in 0..2 {
                        arr[i] = bytes.get(i).map_err(|_| Error::memory_error("Index out of bounds"))?;
                    }
                    (u16::from_le_bytes(arr)) as i64
                };
                Ok(Value::I64(value))
            }
            (ValueType::I64, 32) => {
                let bytes = memory.read_bytes(effective_addr, 4)?;
                if bytes.len() < 4 {
                    return Err(Error::memory_error("Insufficient bytes read for i32 value"));
                }
                #[cfg(any(feature = "std", feature = "alloc"))]
                let value = if self.signed {
                    (i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])) as i64
                } else {
                    (u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])) as i64
                };
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                let value = if self.signed {
                    let mut arr = [0u8; 4];
                    for i in 0..4 {
                        arr[i] = bytes.get(i).map_err(|_| Error::memory_error("Index out of bounds"))?;
                    }
                    (i32::from_le_bytes(arr)) as i64
                } else {
                    let mut arr = [0u8; 4];
                    for i in 0..4 {
                        arr[i] = bytes.get(i).map_err(|_| Error::memory_error("Index out of bounds"))?;
                    }
                    (u32::from_le_bytes(arr)) as i64
                };
                Ok(Value::I64(value))
            }
            _ => Err(Error::type_error(
                "Unsupported memory load operation"
            )),
        }
    }
}

impl MemoryStore {
    /// Creates a new i32 store operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryStore for i32 values
    pub fn i32(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::I32, width: 32 }
    }

    /// Creates a new i64 store operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryStore for i64 values
    pub fn i64(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::I64, width: 64 }
    }

    /// Creates a new f32 store operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryStore for f32 values
    pub fn f32(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::F32, width: 32 }
    }

    /// Creates a new f64 store operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryStore for f64 values
    pub fn f64(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::F64, width: 64 }
    }

    /// Creates a new i32 store8 operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryStore for storing an i32 value as 8 bits
    pub fn i32_store8(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::I32, width: 8 }
    }

    /// Creates a new i32 store16 operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryStore for storing an i32 value as 16 bits
    pub fn i32_store16(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::I32, width: 16 }
    }

    /// Creates a new i64 store8 operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryStore for storing an i64 value as 8 bits
    pub fn i64_store8(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::I64, width: 8 }
    }

    /// Creates a new i64 store16 operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryStore for storing an i64 value as 16 bits
    pub fn i64_store16(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::I64, width: 16 }
    }

    /// Creates a new i64 store32 operation
    ///
    /// # Arguments
    ///
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryStore for storing an i64 value as 32 bits
    pub fn i64_store32(offset: u32, align: u32) -> Self {
        Self { offset, align, value_type: ValueType::I64, width: 32 }
    }

    /// Execute the memory store operation
    ///
    /// # Arguments
    ///
    /// * `memory` - The memory to operate on
    /// * `addr_arg` - The base address to store to
    /// * `value` - The value to store
    ///
    /// # Returns
    ///
    /// Success or an error
    ///
    /// Returns an error if the memory access is invalid
    pub fn execute(
        &self,
        memory: &mut impl MemoryOperations,
        addr_arg: &Value,
        value: &Value,
    ) -> Result<()> {
        // Extract address from argument
        let addr = match addr_arg {
            Value::I32(a) => *a as u32,
            _ => {
                return Err(Error::type_error(
                    "Memory store expects I32 address, got unexpected value"
                ));
            }
        };

        // Calculate effective address
        let effective_addr = addr.checked_add(self.offset).ok_or_else(|| {
            Error::memory_error(
                "Address overflow in memory store"
            )
        })?;

        // Verify alignment if required
        if self.align > 1 && effective_addr % self.align != 0 {
            return Err(Error::memory_error(
                "Unaligned memory access"
            ));
        }

        // Perform the store based on the type and width
        match (self.value_type, self.width, value) {
            (ValueType::I32, 32, Value::I32(v)) => {
                let bytes = v.to_le_bytes();
                memory.write_bytes(effective_addr, &bytes)
            }
            (ValueType::I64, 64, Value::I64(v)) => {
                let bytes = v.to_le_bytes();
                memory.write_bytes(effective_addr, &bytes)
            }
            (ValueType::F32, 32, Value::F32(v)) => {
                let bytes = v.to_bits().to_le_bytes();
                memory.write_bytes(effective_addr, &bytes)
            }
            (ValueType::F64, 64, Value::F64(v)) => {
                let bytes = v.to_bits().to_le_bytes();
                memory.write_bytes(effective_addr, &bytes)
            }

            (ValueType::I32, 8, Value::I32(v)) => {
                let bytes = [((*v) & 0xFF) as u8];
                memory.write_bytes(effective_addr, &bytes)
            }
            (ValueType::I64, 8, Value::I64(v)) => {
                let bytes = [((*v) & 0xFF) as u8];
                memory.write_bytes(effective_addr, &bytes)
            }
            (ValueType::I32, 16, Value::I32(v)) => {
                let bytes = (*v as u16).to_le_bytes();
                memory.write_bytes(effective_addr, &bytes)
            }
            (ValueType::I64, 16, Value::I64(v)) => {
                let bytes = (*v as u16).to_le_bytes();
                memory.write_bytes(effective_addr, &bytes)
            }
            (ValueType::I64, 32, Value::I64(v)) => {
                let bytes = (*v as u32).to_le_bytes();
                memory.write_bytes(effective_addr, &bytes)
            }
            _ => Err(Error::type_error(
                "Type mismatch for memory store"
            )),
        }
    }
}

/// Memory fill operation (WebAssembly bulk memory proposal)
#[derive(Debug, Clone)]
pub struct MemoryFill {
    /// Memory index (for multi-memory support)
    pub memory_index: u32,
}

/// Memory copy operation (WebAssembly bulk memory proposal)
#[derive(Debug, Clone)]
pub struct MemoryCopy {
    /// Destination memory index
    pub dest_memory_index: u32,
    /// Source memory index
    pub src_memory_index: u32,
}

/// Memory init operation (WebAssembly bulk memory proposal)
#[derive(Debug, Clone)]
pub struct MemoryInit {
    /// Memory index
    pub memory_index: u32,
    /// Data segment index
    pub data_index: u32,
}

/// Data drop operation (WebAssembly bulk memory proposal)
#[derive(Debug, Clone)]
pub struct DataDrop {
    /// Data segment index
    pub data_index: u32,
}

/// Trait for data segment operations (needed for memory.init and data.drop)
pub trait DataSegmentOperations {
    /// Get data segment bytes
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn get_data_segment(&self, data_index: u32) -> Result<Option<Vec<u8>>>;
    
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn get_data_segment(&self, data_index: u32) -> Result<Option<wrt_foundation::BoundedVec<u8, 65536, wrt_foundation::NoStdProvider<65536>>>>;
    
    /// Drop (mark as unavailable) a data segment
    fn drop_data_segment(&mut self, data_index: u32) -> Result<()>;
}

impl MemoryFill {
    /// Create a new memory fill operation
    pub fn new(memory_index: u32) -> Self {
        Self { memory_index }
    }

    /// Execute memory.fill operation
    ///
    /// # Arguments
    ///
    /// * `memory` - The memory to operate on
    /// * `dest` - Destination address (i32)
    /// * `value` - Fill byte value (i32, only low 8 bits used)
    /// * `size` - Number of bytes to fill (i32)
    ///
    /// # Returns
    ///
    /// Success or an error
    pub fn execute(
        &self,
        memory: &mut impl MemoryOperations,
        dest: &Value,
        value: &Value,
        size: &Value,
    ) -> Result<()> {
        // Extract arguments
        let dest_addr = match dest {
            Value::I32(addr) => *addr as u32,
            _ => return Err(Error::type_error("memory.fill dest must be i32")),
        };

        let fill_byte = match value {
            Value::I32(val) => (*val & 0xFF) as u8,
            _ => return Err(Error::type_error("memory.fill value must be i32")),
        };

        let fill_size = match size {
            Value::I32(sz) => *sz as u32,
            _ => return Err(Error::type_error("memory.fill size must be i32")),
        };

        // Check for overflow
        let end_addr = dest_addr.checked_add(fill_size).ok_or_else(|| {
            Error::memory_error("memory.fill address overflow")
        })?;

        // Check bounds
        let memory_size = memory.size_in_bytes()? as u32;
        if end_addr > memory_size {
            return Err(Error::memory_error("memory.fill out of bounds"));
        }

        // Perform fill operation
        memory.fill(dest_addr, fill_byte, fill_size)
    }
}

impl MemoryCopy {
    /// Create a new memory copy operation
    pub fn new(dest_memory_index: u32, src_memory_index: u32) -> Self {
        Self { dest_memory_index, src_memory_index }
    }

    /// Execute memory.copy operation
    ///
    /// # Arguments
    ///
    /// * `memory` - The memory to operate on (currently assumes same memory for src/dest)
    /// * `dest` - Destination address (i32)
    /// * `src` - Source address (i32)
    /// * `size` - Number of bytes to copy (i32)
    ///
    /// # Returns
    ///
    /// Success or an error
    pub fn execute(
        &self,
        memory: &mut impl MemoryOperations,
        dest: &Value,
        src: &Value,
        size: &Value,
    ) -> Result<()> {
        // Extract arguments
        let dest_addr = match dest {
            Value::I32(addr) => *addr as u32,
            _ => return Err(Error::type_error("memory.copy dest must be i32")),
        };

        let src_addr = match src {
            Value::I32(addr) => *addr as u32,
            _ => return Err(Error::type_error("memory.copy src must be i32")),
        };

        let copy_size = match size {
            Value::I32(sz) => *sz as u32,
            _ => return Err(Error::type_error("memory.copy size must be i32")),
        };

        // Check for overflow
        let dest_end = dest_addr.checked_add(copy_size).ok_or_else(|| {
            Error::memory_error("memory.copy dest address overflow")
        })?;

        let src_end = src_addr.checked_add(copy_size).ok_or_else(|| {
            Error::memory_error("memory.copy src address overflow")
        })?;

        // Check bounds
        let memory_size = memory.size_in_bytes()? as u32;
        if dest_end > memory_size || src_end > memory_size {
            return Err(Error::memory_error("memory.copy out of bounds"));
        }

        // Perform copy operation (handles overlapping regions correctly)
        memory.copy(dest_addr, src_addr, copy_size)
    }
}

impl MemoryInit {
    /// Create a new memory init operation
    pub fn new(memory_index: u32, data_index: u32) -> Self {
        Self { memory_index, data_index }
    }

    /// Execute memory.init operation
    ///
    /// # Arguments
    ///
    /// * `memory` - The memory to operate on
    /// * `data_segments` - Access to data segments
    /// * `dest` - Destination address in memory (i32)
    /// * `src` - Source offset in data segment (i32)
    /// * `size` - Number of bytes to copy (i32)
    ///
    /// # Returns
    ///
    /// Success or an error
    pub fn execute(
        &self,
        memory: &mut impl MemoryOperations,
        data_segments: &impl DataSegmentOperations,
        dest: &Value,
        src: &Value,
        size: &Value,
    ) -> Result<()> {
        // Extract arguments
        let dest_addr = match dest {
            Value::I32(addr) => *addr as u32,
            _ => return Err(Error::type_error("memory.init dest must be i32")),
        };

        let src_offset = match src {
            Value::I32(offset) => *offset as u32,
            _ => return Err(Error::type_error("memory.init src must be i32")),
        };

        let copy_size = match size {
            Value::I32(sz) => *sz as u32,
            _ => return Err(Error::type_error("memory.init size must be i32")),
        };

        // Get data segment
        let data = data_segments.get_data_segment(self.data_index)?
            .ok_or_else(|| Error::memory_error("Data segment has been dropped"))?;

        // Check bounds in data segment
        let data_len = data.len() as u32;
        let src_end = src_offset.checked_add(copy_size).ok_or_else(|| {
            Error::memory_error("memory.init src offset overflow")
        })?;

        if src_end > data_len {
            return Err(Error::memory_error("memory.init src out of bounds"));
        }

        // Check bounds in memory
        let dest_end = dest_addr.checked_add(copy_size).ok_or_else(|| {
            Error::memory_error("memory.init dest address overflow")
        })?;

        let memory_size = memory.size_in_bytes()? as u32;
        if dest_end > memory_size {
            return Err(Error::memory_error("memory.init dest out of bounds"));
        }

        // Copy data from segment to memory
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let src_slice = &data[src_offset as usize..src_end as usize];
            memory.write_bytes(dest_addr, src_slice)
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            // For no_std, copy bytes one by one to avoid slice allocation
            for (i, offset) in (src_offset..src_end).enumerate() {
                let byte = data.get(offset as usize).map_err(|_| Error::memory_error("Data segment index out of bounds"))?;
                memory.write_bytes(dest_addr + i as u32, &[byte])?;
            }
            Ok(())
        }
    }
}

impl DataDrop {
    /// Create a new data drop operation
    pub fn new(data_index: u32) -> Self {
        Self { data_index }
    }

    /// Execute data.drop operation
    ///
    /// # Arguments
    ///
    /// * `data_segments` - Access to data segments
    ///
    /// # Returns
    ///
    /// Success or an error
    pub fn execute(
        &self,
        data_segments: &mut impl DataSegmentOperations,
    ) -> Result<()> {
        data_segments.drop_data_segment(self.data_index)
    }
}

#[cfg(test)]
mod tests {
    use wrt_foundation::types::Limits;
    use wrt_runtime::MemoryType;

    use super::*;

    /// Mock memory implementation for testing
    struct MockMemory {
        data: Vec<u8>,
    }

    impl MockMemory {
        fn new(size: usize) -> Self {
            Self { data: vec![0; size] }
        }
    }

    impl MemoryOperations for MockMemory {
        #[cfg(any(feature = "std", feature = "alloc"))]
        fn read_bytes(&self, offset: u32, len: u32) -> Result<Vec<u8>> {
            let start = offset as usize;
            let end = start + len as usize;

            if end > self.data.len() {
                return Err(Error::memory_error("Memory access out of bounds"));
            }

            Ok(self.data[start..end].to_vec())
        }

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        fn read_bytes(&self, offset: u32, len: u32) -> Result<wrt_foundation::BoundedVec<u8, 65536, wrt_foundation::NoStdProvider<65536>>> {
            let start = offset as usize;
            let end = start + len as usize;

            if end > self.data.len() {
                return Err(Error::memory_error("Memory access out of bounds"));
            }

            let mut result = wrt_foundation::BoundedVec::new();
            for &byte in &self.data[start..end] {
                result.push(byte).map_err(|_| Error::memory_error("BoundedVec capacity exceeded"))?;
            }
            Ok(result)
        }

        fn write_bytes(&mut self, offset: u32, bytes: &[u8]) -> Result<()> {
            let start = offset as usize;
            let end = start + bytes.len();

            if end > self.data.len() {
                return Err(Error::memory_error("Memory access out of bounds"));
            }

            self.data[start..end].copy_from_slice(bytes);
            Ok(())
        }

        fn size_in_bytes(&self) -> Result<usize> {
            Ok(self.data.len())
        }

        fn grow(&mut self, pages: u32) -> Result<u32> {
            let old_pages = (self.data.len() / 65536) as u32;
            let new_size = self.data.len() + (pages as usize * 65536);
            self.data.resize(new_size, 0);
            Ok(old_pages)
        }

        fn fill(&mut self, offset: u32, value: u8, size: u32) -> Result<()> {
            let start = offset as usize;
            let end = start + size as usize;

            if end > self.data.len() {
                return Err(Error::memory_error("Memory fill out of bounds"));
            }

            for i in start..end {
                self.data[i] = value;
            }
            Ok(())
        }

        fn copy(&mut self, dest: u32, src: u32, size: u32) -> Result<()> {
            let dest_start = dest as usize;
            let dest_end = dest_start + size as usize;
            let src_start = src as usize;
            let src_end = src_start + size as usize;

            if dest_end > self.data.len() || src_end > self.data.len() {
                return Err(Error::memory_error("Memory copy out of bounds"));
            }

            // Handle overlapping regions correctly by copying to a temporary buffer
            if size > 0 {
                let temp: Vec<u8> = self.data[src_start..src_end].to_vec();
                self.data[dest_start..dest_end].copy_from_slice(&temp);
            }
            Ok(())
        }
    }

    #[test]
    fn test_memory_load() {
        let mut memory = MockMemory::new(65536);

        // Store some test values
        memory.write_bytes(0, &[42, 0, 0, 0]).unwrap(); // i32 = 42
        memory.write_bytes(4, &[0, 1, 0, 0, 0, 0, 0, 0]).unwrap(); // i64 = 256
        memory.write_bytes(12, &[0, 0, 0x40, 0x40]).unwrap(); // f32 = 3.0
        memory.write_bytes(16, &[0, 0, 0, 0, 0, 0, 0x08, 0x40]).unwrap(); // f64 = 3.0
        memory.write_bytes(24, &[0xFF]).unwrap(); // i8 = -1 (signed)
        memory.write_bytes(25, &[0xFF, 0xFF]).unwrap(); // i16 = -1 (signed)

        // Test i32.load
        let load = MemoryLoad::i32(0, 4);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(42));

        // Test i64.load
        let load = MemoryLoad::i64(4, 8);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I64(256));

        // Test f32.load
        let load = MemoryLoad::f32(12, 4);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::F32(3.0));

        // Test f64.load
        let load = MemoryLoad::f64(16, 8);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::F64(3.0));

        // Test i32.load8_s
        let load = MemoryLoad::i32_load8(24, 1, true);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(-1));

        // Test i32.load8_u
        let load = MemoryLoad::i32_load8(24, 1, false);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(255));

        // Test i32.load16_s
        let load = MemoryLoad::i32_load16(25, 2, true);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(-1));

        // Test i32.load16_u
        let load = MemoryLoad::i32_load16(25, 2, false);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(65535));

        // Test effective address calculation with offset
        let load = MemoryLoad::i32(4, 4);
        let result = load.execute(&memory, &Value::I32(4)).unwrap();
        assert_eq!(result, Value::I32(256));
    }

    #[test]
    fn test_memory_store() {
        let mut memory = MockMemory::new(65536);

        // Test i32.store
        let store = MemoryStore::i32(0, 4);
        store.execute(&mut memory, &Value::I32(0), &Value::I32(42)).unwrap();

        let load = MemoryLoad::i32(0, 4);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(42));

        // Test i64.store
        let store = MemoryStore::i64(8, 8);
        store.execute(&mut memory, &Value::I32(0), &Value::I64(0x0102030405060708)).unwrap();

        let load = MemoryLoad::i64(8, 8);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I64(0x0102030405060708));

        // Test f32.store
        let store = MemoryStore::f32(16, 4);
        store.execute(&mut memory, &Value::I32(0), &Value::F32(3.14159)).unwrap();

        let load = MemoryLoad::f32(16, 4);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::F32(3.14159));

        // Test f64.store
        let store = MemoryStore::f64(24, 8);
        store.execute(&mut memory, &Value::I32(0), &Value::F64(2.71828)).unwrap();

        let load = MemoryLoad::f64(24, 8);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::F64(2.71828));

        // Test i32.store8
        let store = MemoryStore::i32_store8(32, 1);
        store.execute(&mut memory, &Value::I32(0), &Value::I32(0xFF)).unwrap();

        let load = MemoryLoad::i32_load8(32, 1, false);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(0xFF));

        // Test i32.store16
        let store = MemoryStore::i32_store16(33, 1);
        store.execute(&mut memory, &Value::I32(0), &Value::I32(0xABCD)).unwrap();

        let load = MemoryLoad::i32_load16(33, 1, false);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(0xABCD));

        // Test effective address calculation with offset
        let store = MemoryStore::i32(4, 4);
        store.execute(&mut memory, &Value::I32(4), &Value::I32(0xDEADBEEF)).unwrap();

        let load = MemoryLoad::i32(4, 4);
        let result = load.execute(&memory, &Value::I32(4)).unwrap();
        assert_eq!(result, Value::I32(0xDEADBEEF));
    }

    #[test]
    fn test_memory_access_errors() {
        let mut memory = MockMemory::new(100);

        // Out of bounds access
        let load = MemoryLoad::i32(100, 4);
        let result = load.execute(&memory, &Value::I32(0));
        assert!(result.is_err());

        // Offset + address out of bounds
        let load = MemoryLoad::i32(50, 4);
        let result = load.execute(&memory, &Value::I32(60));
        assert!(result.is_err());

        // Store out of bounds
        let store = MemoryStore::i32(100, 4);
        let result = store.execute(&mut memory, &Value::I32(0), &Value::I32(42));
        assert!(result.is_err());
    }

    /// Mock data segment operations for testing
    struct MockDataSegments {
        #[cfg(any(feature = "std", feature = "alloc"))]
        segments: Vec<Option<Vec<u8>>>,
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        segments: wrt_foundation::BoundedVec<Option<wrt_foundation::BoundedVec<u8, 65536, wrt_foundation::NoStdProvider<65536>>>, 16, wrt_foundation::NoStdProvider<1024>>,
    }

    impl MockDataSegments {
        fn new() -> Self {
            #[cfg(any(feature = "std", feature = "alloc"))]
            {
                Self {
                    segments: vec![
                        Some(vec![1, 2, 3, 4, 5]),
                        Some(vec![0xAA, 0xBB, 0xCC, 0xDD]),
                        None, // Dropped segment
                    ],
                }
            }
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            {
                let mut segments = wrt_foundation::BoundedVec::new();
                
                let mut seg1 = wrt_foundation::BoundedVec::new();
                for &b in &[1, 2, 3, 4, 5] {
                    seg1.push(b).unwrap();
                }
                segments.push(Some(seg1)).unwrap();
                
                let mut seg2 = wrt_foundation::BoundedVec::new();
                for &b in &[0xAA, 0xBB, 0xCC, 0xDD] {
                    seg2.push(b).unwrap();
                }
                segments.push(Some(seg2)).unwrap();
                
                segments.push(None).unwrap(); // Dropped segment
                
                Self { segments }
            }
        }
    }

    impl DataSegmentOperations for MockDataSegments {
        #[cfg(any(feature = "std", feature = "alloc"))]
        fn get_data_segment(&self, data_index: u32) -> Result<Option<Vec<u8>>> {
            if (data_index as usize) < self.segments.len() {
                Ok(self.segments[data_index as usize].clone())
            } else {
                Err(Error::validation_error("Invalid data segment index"))
            }
        }

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        fn get_data_segment(&self, data_index: u32) -> Result<Option<wrt_foundation::BoundedVec<u8, 65536, wrt_foundation::NoStdProvider<65536>>>> {
            if (data_index as usize) < self.segments.len() {
                Ok(self.segments.get(data_index as usize).unwrap().clone())
            } else {
                Err(Error::validation_error("Invalid data segment index"))
            }
        }

        fn drop_data_segment(&mut self, data_index: u32) -> Result<()> {
            if (data_index as usize) < self.segments.len() {
                #[cfg(any(feature = "std", feature = "alloc"))]
                {
                    self.segments[data_index as usize] = None;
                }
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                {
                    *self.segments.get_mut(data_index as usize).unwrap() = None;
                }
                Ok(())
            } else {
                Err(Error::validation_error("Invalid data segment index"))
            }
        }
    }

    #[test]
    fn test_memory_fill() {
        let mut memory = MockMemory::new(1024);
        let fill_op = MemoryFill::new(0);

        // Fill 10 bytes with value 0x42 starting at offset 100
        fill_op
            .execute(&mut memory, &Value::I32(100), &Value::I32(0x42), &Value::I32(10))
            .unwrap();

        // Verify the fill worked
        let data = memory.read_bytes(100, 10).unwrap();
        assert_eq!(data.len(), 10);
        #[cfg(feature = "alloc")]
        assert!(data.iter().all(|&b| b == 0x42));
        #[cfg(not(feature = "alloc"))]
        for i in 0..10 {
            assert_eq!(data.get(i).unwrap(), 0x42);
        }
    }

    #[test]
    fn test_memory_copy() {
        let mut memory = MockMemory::new(1024);
        
        // Set up source data
        memory.write_bytes(0, &[1, 2, 3, 4, 5]).unwrap();
        
        let copy_op = MemoryCopy::new(0, 0);

        // Copy 5 bytes from offset 0 to offset 100
        copy_op
            .execute(&mut memory, &Value::I32(100), &Value::I32(0), &Value::I32(5))
            .unwrap();

        // Verify the copy worked
        let data = memory.read_bytes(100, 5).unwrap();
        #[cfg(feature = "alloc")]
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
        #[cfg(not(feature = "alloc"))]
        {
            assert_eq!(data.len(), 5);
            for i in 0..5 {
                assert_eq!(data.get(i).unwrap(), (i + 1) as u8);
            }
        }
    }

    #[test]
    fn test_memory_copy_overlapping() {
        let mut memory = MockMemory::new(1024);
        
        // Set up source data
        memory.write_bytes(0, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        
        let copy_op = MemoryCopy::new(0, 0);

        // Copy overlapping: copy 5 bytes from offset 0 to offset 2
        copy_op
            .execute(&mut memory, &Value::I32(2), &Value::I32(0), &Value::I32(5))
            .unwrap();

        // Verify overlapping copy worked correctly
        let data = memory.read_bytes(0, 8).unwrap();
        #[cfg(feature = "alloc")]
        assert_eq!(data, vec![1, 2, 1, 2, 3, 4, 5, 8]);
        #[cfg(not(feature = "alloc"))]
        {
            let expected = [1, 2, 1, 2, 3, 4, 5, 8];
            for i in 0..8 {
                assert_eq!(data.get(i).unwrap(), expected[i]);
            }
        }
    }

    #[test]
    fn test_memory_init() {
        let mut memory = MockMemory::new(1024);
        let data_segments = MockDataSegments::new();
        let init_op = MemoryInit::new(0, 0);

        // Copy 3 bytes from data segment 0 (starting at offset 1) to memory at offset 100
        init_op
            .execute(
                &mut memory,
                &data_segments,
                &Value::I32(100),
                &Value::I32(1),
                &Value::I32(3),
            )
            .unwrap();

        // Verify the init worked (should copy bytes [2, 3, 4] from segment [1, 2, 3, 4, 5])
        let data = memory.read_bytes(100, 3).unwrap();
        #[cfg(feature = "alloc")]
        assert_eq!(data, vec![2, 3, 4]);
        #[cfg(not(feature = "alloc"))]
        {
            assert_eq!(data.len(), 3);
            for i in 0..3 {
                assert_eq!(data.get(i).unwrap(), (i + 2) as u8);
            }
        }
    }

    #[test]
    fn test_data_drop() {
        let mut data_segments = MockDataSegments::new();
        let drop_op = DataDrop::new(0);

        // Verify segment 0 exists initially
        assert!(data_segments.get_data_segment(0).unwrap().is_some());

        // Drop segment 0
        drop_op.execute(&mut data_segments).unwrap();

        // Verify segment 0 is now dropped
        assert!(data_segments.get_data_segment(0).unwrap().is_none());
    }

    #[test]
    fn test_memory_init_dropped_segment() {
        let mut memory = MockMemory::new(1024);
        let mut data_segments = MockDataSegments::new();
        
        // Drop segment 0 first
        data_segments.drop_data_segment(0).unwrap();
        
        let init_op = MemoryInit::new(0, 0);

        // Try to init from dropped segment - should fail
        let result = init_op.execute(
            &mut memory,
            &data_segments,
            &Value::I32(100),
            &Value::I32(0),
            &Value::I32(3),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_bulk_memory_bounds_checking() {
        let mut memory = MockMemory::new(100);
        
        // Test memory.fill out of bounds
        let fill_op = MemoryFill::new(0);
        let result = fill_op.execute(&mut memory, &Value::I32(95), &Value::I32(0x42), &Value::I32(10));
        assert!(result.is_err());

        // Test memory.copy out of bounds
        let copy_op = MemoryCopy::new(0, 0);
        let result = copy_op.execute(&mut memory, &Value::I32(95), &Value::I32(0), &Value::I32(10));
        assert!(result.is_err());
    }
}

// Validation implementations

impl Validate for MemoryLoad {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<()> {
        validate_memory_op(
            "memory.load",
            0, // memory index - always 0 for now
            self.align,
            self.value_type,
            true, // is_load
            ctx
        )
    }
}

impl Validate for MemoryStore {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<()> {
        validate_memory_op(
            "memory.store",
            0, // memory index - always 0 for now
            self.align,
            self.value_type,
            false, // is_load
            ctx
        )
    }
}

impl Validate for MemoryFill {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<()> {
        // memory.fill: [i32, i32, i32] -> []
        // Stack: [dest_addr, value, size]
        if !ctx.is_unreachable() {
            ctx.pop_expect(ValueType::I32)?; // size
            ctx.pop_expect(ValueType::I32)?; // value
            ctx.pop_expect(ValueType::I32)?; // dest_addr
        }
        Ok(())
    }
}

impl Validate for MemoryCopy {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<()> {
        // memory.copy: [i32, i32, i32] -> []
        // Stack: [dest_addr, src_addr, size]
        if !ctx.is_unreachable() {
            ctx.pop_expect(ValueType::I32)?; // size
            ctx.pop_expect(ValueType::I32)?; // src_addr
            ctx.pop_expect(ValueType::I32)?; // dest_addr
        }
        Ok(())
    }
}

impl Validate for MemoryInit {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<()> {
        // memory.init: [i32, i32, i32] -> []
        // Stack: [dest_addr, src_offset, size]
        if !ctx.is_unreachable() {
            ctx.pop_expect(ValueType::I32)?; // size
            ctx.pop_expect(ValueType::I32)?; // src_offset
            ctx.pop_expect(ValueType::I32)?; // dest_addr
        }
        Ok(())
    }
}

impl Validate for DataDrop {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<()> {
        // data.drop: [] -> []
        // No stack operations required
        Ok(())
    }
}
