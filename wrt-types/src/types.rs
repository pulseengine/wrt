//! WebAssembly type definitions
//!
//! This module provides type definitions for WebAssembly entities
//! with functional safety verification built in.

use crate::verification::Hasher;
use wrt_error::{kinds, Error, Result};

#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// WebAssembly value types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ValueType {
    /// 32-bit integer
    I32 = 0x7F,
    /// 64-bit integer
    I64 = 0x7E,
    /// 32-bit float
    F32 = 0x7D,
    /// 64-bit float
    F64 = 0x7C,
    /// 128-bit vector (SIMD)
    V128 = 0x7B,
    /// Function reference
    FuncRef = 0x70,
    /// Extern reference
    ExternRef = 0x6F,
}

impl ValueType {
    /// Convert from the WebAssembly binary format value
    pub fn from_binary(byte: u8) -> Result<Self> {
        match byte {
            0x7F => Ok(Self::I32),
            0x7E => Ok(Self::I64),
            0x7D => Ok(Self::F32),
            0x7C => Ok(Self::F64),
            0x7B => Ok(Self::V128),
            0x70 => Ok(Self::FuncRef),
            0x6F => Ok(Self::ExternRef),
            _ => Err(Error::new(kinds::ParseError(format!(
                "Invalid value type byte: 0x{:x}",
                byte
            )))),
        }
    }

    /// Convert to the WebAssembly binary format value
    pub fn to_binary(self) -> u8 {
        self as u8
    }

    /// Get the size of this value type in bytes
    pub fn size_in_bytes(self) -> usize {
        match self {
            Self::I32 | Self::F32 => 4,
            Self::I64 | Self::F64 => 8,
            Self::V128 => 16,
            Self::FuncRef | Self::ExternRef => {
                // Reference types are pointer-sized
                #[cfg(target_pointer_width = "64")]
                return 8;

                #[cfg(target_pointer_width = "32")]
                return 4;

                #[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
                return 8; // Default to 64-bit
            }
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::V128 => write!(f, "v128"),
            Self::FuncRef => write!(f, "funcref"),
            Self::ExternRef => write!(f, "externref"),
        }
    }
}

/// WebAssembly function type
///
/// Represents a function's parameter and result types.
#[derive(Clone)]
pub struct FuncType {
    /// Parameter types
    pub params: Vec<ValueType>,
    /// Result types
    pub results: Vec<ValueType>,
    /// Type hash for validation
    type_hash: u32,
}

impl FuncType {
    /// Create a new function type
    pub fn new(params: Vec<ValueType>, results: Vec<ValueType>) -> Self {
        let mut func_type = Self {
            params,
            results,
            type_hash: 0,
        };

        // Compute hash for safety validation
        func_type.compute_hash();

        func_type
    }

    /// Compute the type hash for validation
    fn compute_hash(&mut self) {
        let mut hasher = Hasher::new();

        // Hash parameter count
        hasher.update(&(self.params.len() as u32).to_le_bytes());

        // Hash parameter types
        for param in &self.params {
            hasher.update(&[(*param as u8)]);
        }

        // Hash result count
        hasher.update(&(self.results.len() as u32).to_le_bytes());

        // Hash result types
        for result in &self.results {
            hasher.update(&[(*result as u8)]);
        }

        self.type_hash = hasher.finalize();
    }

    /// Verify type integrity
    pub fn verify(&self) -> Result<()> {
        // Create a copy for verification
        let mut copy = Self {
            params: self.params.clone(),
            results: self.results.clone(),
            type_hash: 0,
        };

        // Compute hash on the copy
        copy.compute_hash();

        // Compare with original hash
        if copy.type_hash != self.type_hash {
            return Err(Error::new(kinds::ValidationError(
                "Type integrity check failed: hash mismatch".into(),
            )));
        }

        Ok(())
    }

    /// Check if two function types are structurally equal
    pub fn matches(&self, other: &Self) -> Result<bool> {
        // Verify both types first
        self.verify()?;
        other.verify()?;

        // Quick check using hash
        if self.type_hash == other.type_hash {
            return Ok(true);
        }

        // Detailed structural equality check
        if self.params.len() != other.params.len() || self.results.len() != other.results.len() {
            return Ok(false);
        }

        for (a, b) in self.params.iter().zip(other.params.iter()) {
            if a != b {
                return Ok(false);
            }
        }

        for (a, b) in self.results.iter().zip(other.results.iter()) {
            if a != b {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

// Implement PartialEq for FuncType
impl PartialEq for FuncType {
    fn eq(&self, other: &Self) -> bool {
        // If hashes are the same, they're equal
        if self.type_hash == other.type_hash {
            return true;
        }

        // Otherwise, do a structural comparison
        if self.params.len() != other.params.len() || self.results.len() != other.results.len() {
            return false;
        }

        // Compare all params
        for (a, b) in self.params.iter().zip(other.params.iter()) {
            if a != b {
                return false;
            }
        }

        // Compare all results
        for (a, b) in self.results.iter().zip(other.results.iter()) {
            if a != b {
                return false;
            }
        }

        true
    }
}

// Implement Eq for FuncType
impl Eq for FuncType {}

impl fmt::Debug for FuncType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FuncType({} -> {}, hash=0x{:08x})",
            func_params_to_string(&self.params),
            func_results_to_string(&self.results),
            self.type_hash
        )
    }
}

/// Convert function parameters to a string representation
fn func_params_to_string(params: &[ValueType]) -> String {
    if params.is_empty() {
        return "[]".to_string();
    }

    let mut result = String::new();
    result.push('[');

    for (i, param) in params.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        result.push_str(&param.to_string());
    }

    result.push(']');
    result
}

/// Convert function results to a string representation
fn func_results_to_string(results: &[ValueType]) -> String {
    if results.is_empty() {
        return "[]".to_string();
    }

    // Single result is shown without brackets for readability
    if results.len() == 1 {
        return results[0].to_string();
    }

    let mut result = String::new();
    result.push('[');

    for (i, res) in results.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        result.push_str(&res.to_string());
    }

    result.push(']');
    result
}

/// WebAssembly memory type
#[derive(Debug, Clone)]
pub struct MemoryType {
    /// Memory limits
    pub limits: Limits,
    /// Whether the memory can be shared between instances
    pub shared: bool,
}

// Implement PartialEq for MemoryType
impl PartialEq for MemoryType {
    fn eq(&self, other: &Self) -> bool {
        self.limits == other.limits && self.shared == other.shared
    }
}

// Implement Eq for MemoryType
impl Eq for MemoryType {}

/// WebAssembly table type
#[derive(Debug, Clone)]
pub struct TableType {
    /// Table limits
    pub limits: Limits,
    /// Type of elements in the table
    pub element_type: ValueType,
}

// Implement PartialEq for TableType
impl PartialEq for TableType {
    fn eq(&self, other: &Self) -> bool {
        self.limits == other.limits && self.element_type == other.element_type
    }
}

// Implement Eq for TableType
impl Eq for TableType {}

/// WebAssembly global variable type
#[derive(Debug, Clone)]
pub struct GlobalType {
    /// Type of values stored in the global
    pub value_type: ValueType,
    /// Whether the global is mutable
    pub mutable: bool,
}

// Implement PartialEq for GlobalType
impl PartialEq for GlobalType {
    fn eq(&self, other: &Self) -> bool {
        self.value_type == other.value_type && self.mutable == other.mutable
    }
}

// Implement Eq for GlobalType
impl Eq for GlobalType {}

/// WebAssembly limits for tables and memories
#[derive(Debug, Clone)]
pub struct Limits {
    /// Minimum size (required)
    pub min: u32,
    /// Maximum size (optional)
    pub max: Option<u32>,
}

// Implement PartialEq for Limits
impl PartialEq for Limits {
    fn eq(&self, other: &Self) -> bool {
        self.min == other.min && self.max == other.max
    }
}

// Implement Eq for Limits
impl Eq for Limits {}

impl Limits {
    /// Create new limits with minimum and optional maximum
    pub fn new(min: u32, max: Option<u32>) -> Self {
        Self { min, max }
    }

    /// Check if a size is within the limits
    pub fn check_size(&self, size: u32) -> bool {
        size >= self.min
            && match self.max {
                Some(max) => size <= max,
                None => true,
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_conversions() {
        assert_eq!(ValueType::from_binary(0x7F).unwrap(), ValueType::I32);
        assert_eq!(ValueType::from_binary(0x7E).unwrap(), ValueType::I64);
        assert_eq!(ValueType::from_binary(0x7D).unwrap(), ValueType::F32);
        assert_eq!(ValueType::from_binary(0x7C).unwrap(), ValueType::F64);
        assert_eq!(ValueType::from_binary(0x7B).unwrap(), ValueType::V128);
        assert_eq!(ValueType::from_binary(0x70).unwrap(), ValueType::FuncRef);
        assert_eq!(ValueType::from_binary(0x6F).unwrap(), ValueType::ExternRef);

        assert!(ValueType::from_binary(0x00).is_err());
    }

    #[test]
    fn test_func_type_verification() {
        let func_type = FuncType::new(vec![ValueType::I32, ValueType::I64], vec![ValueType::F32]);

        // Verification should pass
        assert!(func_type.verify().is_ok());

        // Create a copy with a different hash
        let mut bad_type = func_type.clone();
        bad_type.type_hash = 0; // intentionally invalid hash

        // Verification should fail
        assert!(bad_type.verify().is_err());
    }

    #[test]
    fn test_limits() {
        let limits = Limits::new(10, Some(20));

        assert!(limits.check_size(10));
        assert!(limits.check_size(15));
        assert!(limits.check_size(20));

        assert!(!limits.check_size(5));
        assert!(!limits.check_size(21));

        let unlimited = Limits::new(5, None);
        assert!(unlimited.check_size(5));
        assert!(unlimited.check_size(1000000));
    }

    #[test]
    fn test_type_equality() {
        let func_type1 = FuncType::new(vec![ValueType::I32, ValueType::I64], vec![ValueType::F32]);

        let func_type2 = FuncType::new(vec![ValueType::I32, ValueType::I64], vec![ValueType::F32]);

        let func_type3 = FuncType::new(vec![ValueType::I32], vec![ValueType::F32]);

        // Test PartialEq implementation
        assert_eq!(func_type1, func_type2);
        assert_ne!(func_type1, func_type3);

        // Test table type equality
        let table_type1 = TableType {
            limits: Limits::new(10, Some(20)),
            element_type: ValueType::FuncRef,
        };

        let table_type2 = TableType {
            limits: Limits::new(10, Some(20)),
            element_type: ValueType::FuncRef,
        };

        let table_type3 = TableType {
            limits: Limits::new(5, Some(20)),
            element_type: ValueType::FuncRef,
        };

        assert_eq!(table_type1, table_type2);
        assert_ne!(table_type1, table_type3);

        // Test global type equality
        let global_type1 = GlobalType {
            value_type: ValueType::I32,
            mutable: true,
        };

        let global_type2 = GlobalType {
            value_type: ValueType::I32,
            mutable: true,
        };

        let global_type3 = GlobalType {
            value_type: ValueType::I32,
            mutable: false,
        };

        assert_eq!(global_type1, global_type2);
        assert_ne!(global_type1, global_type3);
    }
}
