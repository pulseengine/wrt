//! Memory operations for WebAssembly instructions.
//!
//! This module provides implementations for WebAssembly memory access instructions,
//! including load and store operations for various value types.
//!
//! # Memory Operation Architecture
//!
//! This module separates memory operations from the underlying memory implementation,
//! allowing different execution engines to share the same memory access code. The key
//! components are:
//!
//! - `MemoryLoad`: Handles all WebAssembly load operations with various data types and widths
//! - `MemoryStore`: Handles all WebAssembly store operations with various data types and widths
//!
//! Both structures work with the `Memory` implementation from the `wrt-runtime` crate.
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
//! All memory operations perform proper bounds and alignment checking before accessing
//! memory. This ensures that WebAssembly's memory safety guarantees are preserved.
//!
//! # Usage
//!
//! ```no_run
//! use wrt_instructions::memory_ops::{MemoryLoad, MemoryStore};
//! use wrt_instructions::Value;
//! use wrt_runtime::{Memory, MemoryType};
//! use wrt_types::types::Limits;
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

use crate::{Error, Result, Value, ValueType};
use wrt_error::kinds;
use wrt_runtime::Memory;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{sync::Arc, vec::Vec};

/// Memory load operation
#[derive(Debug, Clone)]
pub struct MemoryLoad {
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
    /// * `offset` - Memory offset
    /// * `align` - Required alignment
    ///
    /// # Returns
    ///
    /// A new MemoryLoad for i32 values
    pub fn i32(offset: u32, align: u32) -> Self {
        Self {
            offset,
            align,
            value_type: ValueType::I32,
            signed: false,
            width: 32,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I64,
            signed: false,
            width: 64,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::F32,
            signed: false,
            width: 32,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::F64,
            signed: false,
            width: 64,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I32,
            signed,
            width: 8,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I32,
            signed,
            width: 16,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I64,
            signed,
            width: 8,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I64,
            signed,
            width: 16,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I64,
            signed,
            width: 32,
        }
    }

    /// Executes the load operation on the given memory
    ///
    /// # Arguments
    ///
    /// * `memory` - The memory to load from
    /// * `addr_arg` - The address argument from the stack
    ///
    /// # Returns
    ///
    /// The loaded value
    ///
    /// # Errors
    ///
    /// Returns an error if the memory access is invalid
    pub fn execute(&self, memory: &Memory, addr_arg: &Value) -> Result<Value> {
        // Extract address from argument
        let addr = match addr_arg {
            Value::I32(a) => *a as u32,
            Value::I64(a) => *a as u32, // Truncate to u32 as per WebAssembly spec
            _ => {
                return Err(Error::new(kinds::ValidationError(
                    "Invalid address type for memory load".to_string(),
                )))
            }
        };

        // Calculate effective address
        let effective_addr = addr.checked_add(self.offset).ok_or_else(|| {
            Error::new(kinds::ValidationError(
                "Memory address overflow".to_string(),
            ))
        })?;

        // Check alignment
        memory.check_alignment(effective_addr, self.width / 8, self.align)?;

        // Perform the load based on the value type and width
        match (self.value_type, self.width) {
            // Full-width loads
            (ValueType::I32, 32) => {
                let mut buffer = [0; 4];
                memory.read(effective_addr, &mut buffer)?;
                let value = i32::from_le_bytes(buffer);
                Ok(Value::I32(value))
            }
            (ValueType::I64, 64) => {
                let mut buffer = [0; 8];
                memory.read(effective_addr, &mut buffer)?;
                let value = i64::from_le_bytes(buffer);
                Ok(Value::I64(value))
            }
            (ValueType::F32, 32) => {
                let mut buffer = [0; 4];
                memory.read(effective_addr, &mut buffer)?;
                let value = f32::from_le_bytes(buffer);
                Ok(Value::F32(value))
            }
            (ValueType::F64, 64) => {
                let mut buffer = [0; 8];
                memory.read(effective_addr, &mut buffer)?;
                let value = f64::from_le_bytes(buffer);
                Ok(Value::F64(value))
            }

            // Partial-width loads for i32
            (ValueType::I32, 8) => {
                let byte = memory.get_byte(effective_addr)?;
                let value = if self.signed {
                    (byte as i8) as i32
                } else {
                    byte as i32
                };
                Ok(Value::I32(value))
            }
            (ValueType::I32, 16) => {
                let mut buffer = [0; 2];
                memory.read(effective_addr, &mut buffer)?;
                let value = if self.signed {
                    (i16::from_le_bytes(buffer)) as i32
                } else {
                    (u16::from_le_bytes(buffer)) as i32
                };
                Ok(Value::I32(value))
            }

            // Partial-width loads for i64
            (ValueType::I64, 8) => {
                let byte = memory.get_byte(effective_addr)?;
                let value = if self.signed {
                    (byte as i8) as i64
                } else {
                    byte as i64
                };
                Ok(Value::I64(value))
            }
            (ValueType::I64, 16) => {
                let mut buffer = [0; 2];
                memory.read(effective_addr, &mut buffer)?;
                let value = if self.signed {
                    (i16::from_le_bytes(buffer)) as i64
                } else {
                    (u16::from_le_bytes(buffer)) as i64
                };
                Ok(Value::I64(value))
            }
            (ValueType::I64, 32) => {
                let mut buffer = [0; 4];
                memory.read(effective_addr, &mut buffer)?;
                let value = if self.signed {
                    (i32::from_le_bytes(buffer)) as i64
                } else {
                    (u32::from_le_bytes(buffer)) as i64
                };
                Ok(Value::I64(value))
            }

            // Invalid combinations
            _ => Err(Error::new(kinds::ValidationError(format!(
                "Invalid memory load: type {:?} with width {}",
                self.value_type, self.width
            )))),
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
        Self {
            offset,
            align,
            value_type: ValueType::I32,
            width: 32,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I64,
            width: 64,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::F32,
            width: 32,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::F64,
            width: 64,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I32,
            width: 8,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I32,
            width: 16,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I64,
            width: 8,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I64,
            width: 16,
        }
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
        Self {
            offset,
            align,
            value_type: ValueType::I64,
            width: 32,
        }
    }

    /// Executes the store operation on the given memory
    ///
    /// # Arguments
    ///
    /// * `memory` - The memory to store to
    /// * `addr_arg` - The address argument from the stack
    /// * `value` - The value to store
    ///
    /// # Returns
    ///
    /// Ok(()) if the store was successful
    ///
    /// # Errors
    ///
    /// Returns an error if the memory access is invalid
    pub fn execute(&self, memory: &mut Memory, addr_arg: &Value, value: &Value) -> Result<()> {
        // Extract address from argument
        let addr = match addr_arg {
            Value::I32(a) => *a as u32,
            Value::I64(a) => *a as u32, // Truncate to u32 as per WebAssembly spec
            _ => {
                return Err(Error::new(kinds::ValidationError(
                    "Invalid address type for memory store".to_string(),
                )))
            }
        };

        // Calculate effective address
        let effective_addr = addr.checked_add(self.offset).ok_or_else(|| {
            Error::new(kinds::ValidationError(
                "Memory address overflow".to_string(),
            ))
        })?;

        // Check alignment
        memory.check_alignment(effective_addr, self.width / 8, self.align)?;

        // Perform the store based on the value type and width
        match (self.value_type, self.width, value) {
            // Full-width stores
            (ValueType::I32, 32, Value::I32(v)) => {
                let bytes = v.to_le_bytes();
                memory.write(effective_addr, &bytes)
            }
            (ValueType::I64, 64, Value::I64(v)) => {
                let bytes = v.to_le_bytes();
                memory.write(effective_addr, &bytes)
            }
            (ValueType::F32, 32, Value::F32(v)) => {
                let bytes = v.to_le_bytes();
                memory.write(effective_addr, &bytes)
            }
            (ValueType::F64, 64, Value::F64(v)) => {
                let bytes = v.to_le_bytes();
                memory.write(effective_addr, &bytes)
            }

            // Partial-width stores for i32
            (ValueType::I32, 8, Value::I32(v)) => {
                let byte = *v as u8;
                memory.set_byte(effective_addr, byte)
            }
            (ValueType::I32, 16, Value::I32(v)) => {
                let bytes = (*v as u16).to_le_bytes();
                memory.write(effective_addr, &bytes)
            }

            // Partial-width stores for i64
            (ValueType::I64, 8, Value::I64(v)) => {
                let byte = *v as u8;
                memory.set_byte(effective_addr, byte)
            }
            (ValueType::I64, 16, Value::I64(v)) => {
                let bytes = (*v as u16).to_le_bytes();
                memory.write(effective_addr, &bytes)
            }
            (ValueType::I64, 32, Value::I64(v)) => {
                let bytes = (*v as u32).to_le_bytes();
                memory.write(effective_addr, &bytes)
            }

            // Type mismatch or invalid combinations
            _ => Err(Error::new(kinds::ValidationError(format!(
                "Invalid memory store: expected {:?} with width {}, got {:?}",
                self.value_type, self.width, value
            )))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_runtime::MemoryType;
    use wrt_types::types::Limits;

    #[test]
    fn test_memory_load() {
        // Create a memory instance
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory = Memory::new(mem_type).unwrap();

        // Write test data
        let i32_value = 0x12345678u32 as i32;
        let i64_value = 0x1234567890ABCDEFu64 as i64;
        let f32_value = 3.14f32;
        let f64_value = 2.71828f64;

        memory.write(0, &i32_value.to_le_bytes()).unwrap();
        memory.write(4, &i64_value.to_le_bytes()).unwrap();
        memory.write(12, &f32_value.to_le_bytes()).unwrap();
        memory.write(16, &f64_value.to_le_bytes()).unwrap();

        // Test i32 load
        let load = MemoryLoad::i32(0, 4);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(i32_value));

        // Test i64 load
        let load = MemoryLoad::i64(4, 4);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I64(i64_value));

        // Test f32 load
        let load = MemoryLoad::f32(12, 4);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        if let Value::F32(v) = result {
            assert_eq!(v, f32_value);
        } else {
            panic!("Expected F32 value");
        }

        // Test f64 load
        let load = MemoryLoad::f64(16, 8);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        if let Value::F64(v) = result {
            assert_eq!(v, f64_value);
        } else {
            panic!("Expected F64 value");
        }

        // Test partial-width loads
        memory.set_byte(24, 0xFF).unwrap(); // -1 in i8
        memory.write(26, &(0xFFFFu16.to_le_bytes())).unwrap(); // -1 in i16

        // Test i32.load8_s
        let load = MemoryLoad::i32_load8(24, 1, true);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(-1));

        // Test i32.load8_u
        let load = MemoryLoad::i32_load8(24, 1, false);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(0xFF));

        // Test i32.load16_s
        let load = MemoryLoad::i32_load16(26, 2, true);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(-1));

        // Test i32.load16_u
        let load = MemoryLoad::i32_load16(26, 2, false);
        let result = load.execute(&memory, &Value::I32(0)).unwrap();
        assert_eq!(result, Value::I32(0xFFFF));
    }

    #[test]
    fn test_memory_store() {
        // Create a memory instance
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory = Memory::new(mem_type).unwrap();

        // Test values
        let i32_value = Value::I32(0x12345678);
        let i64_value = Value::I64(0x1234567890ABCDEF);
        let f32_value = Value::F32(3.14);
        let f64_value = Value::F64(2.71828);

        // Test i32 store
        let store = MemoryStore::i32(0, 4);
        store
            .execute(&mut memory, &Value::I32(0), &i32_value)
            .unwrap();

        // Test i64 store
        let store = MemoryStore::i64(4, 4);
        store
            .execute(&mut memory, &Value::I32(0), &i64_value)
            .unwrap();

        // Test f32 store
        let store = MemoryStore::f32(12, 4);
        store
            .execute(&mut memory, &Value::I32(0), &f32_value)
            .unwrap();

        // Test f64 store
        let store = MemoryStore::f64(16, 8);
        store
            .execute(&mut memory, &Value::I32(0), &f64_value)
            .unwrap();

        // Verify the values were stored correctly
        let mut buffer = [0; 4];
        memory.read(0, &mut buffer).unwrap();
        assert_eq!(i32::from_le_bytes(buffer), 0x12345678);

        let mut buffer = [0; 8];
        memory.read(4, &mut buffer).unwrap();
        assert_eq!(i64::from_le_bytes(buffer), 0x1234567890ABCDEF);

        let mut buffer = [0; 4];
        memory.read(12, &mut buffer).unwrap();
        assert_eq!(f32::from_le_bytes(buffer), 3.14);

        let mut buffer = [0; 8];
        memory.read(16, &mut buffer).unwrap();
        assert_eq!(f64::from_le_bytes(buffer), 2.71828);

        // Test partial-width stores
        let store = MemoryStore::i32_store8(24, 1);
        store
            .execute(&mut memory, &Value::I32(0), &Value::I32(0xAB))
            .unwrap();
        assert_eq!(memory.get_byte(24).unwrap(), 0xAB);

        let store = MemoryStore::i32_store16(26, 2);
        store
            .execute(&mut memory, &Value::I32(0), &Value::I32(0xABCD))
            .unwrap();
        let mut buffer = [0; 2];
        memory.read(26, &mut buffer).unwrap();
        assert_eq!(u16::from_le_bytes(buffer), 0xABCD);

        let store = MemoryStore::i64_store32(28, 4);
        store
            .execute(&mut memory, &Value::I32(0), &Value::I64(0x12345678ABCDEF))
            .unwrap();
        let mut buffer = [0; 4];
        memory.read(28, &mut buffer).unwrap();
        assert_eq!(u32::from_le_bytes(buffer), 0x78ABCDEF);
    }

    #[test]
    fn test_memory_access_errors() {
        // Create a small memory instance
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(1),
            },
        };
        let mut memory = Memory::new(mem_type).unwrap();

        // Test out-of-bounds access
        let load = MemoryLoad::i32(0, 4);
        assert!(load
            .execute(&memory, &Value::I32(wrt_runtime::PAGE_SIZE as i32 - 2))
            .is_err());

        let store = MemoryStore::i32(0, 4);
        assert!(store
            .execute(
                &mut memory,
                &Value::I32(wrt_runtime::PAGE_SIZE as i32 - 2),
                &Value::I32(42)
            )
            .is_err());

        // Test alignment errors
        let load = MemoryLoad::i32(0, 4); // 4-byte aligned
        assert!(load.execute(&memory, &Value::I32(1)).is_err());

        let store = MemoryStore::i64(0, 8); // 8-byte aligned
        assert!(store
            .execute(&mut memory, &Value::I32(4), &Value::I64(42))
            .is_err());
    }
}
