//! Format-specific type conversion utilities
//!
//! This module provides standardized type conversion between format-specific types
//! in wrt-format and core types in wrt-types. This helps eliminate duplication and
//! ensure consistency across crates.

use crate::error::{parse_error, to_wrt_error, validation_error};
use crate::format;
use crate::types::{FormatBlockType, Limits};
use wrt_error::Result;
use wrt_types::conversion::{binary_to_val_type, val_type_to_binary};
use wrt_types::{BlockType, Limits as WrtLimits, ValueType};

/// Re-export binary_to_val_type and val_type_to_binary for backward compatibility
pub use wrt_types::conversion::val_type_to_binary as value_type_to_byte;

/// Convert from FormatBlockType to BlockType
///
/// Provides a standard conversion from format-specific block types
/// to the core BlockType representation.
pub fn format_block_type_to_block_type(format_block_type: &FormatBlockType) -> BlockType {
    match format_block_type {
        FormatBlockType::Empty => BlockType::Empty,
        FormatBlockType::ValueType(value_type) => BlockType::Value(*value_type),
        FormatBlockType::FuncType(func_type) => BlockType::FuncType(func_type.clone()),
        FormatBlockType::TypeIndex(idx) => BlockType::TypeIndex(*idx),
    }
}

/// Convert from BlockType to FormatBlockType
///
/// Provides a standard conversion from core BlockType
/// to the format-specific representation.
pub fn block_type_to_format_block_type(block_type: &BlockType) -> FormatBlockType {
    match block_type {
        BlockType::Empty => FormatBlockType::Empty,
        BlockType::Value(value_type) => FormatBlockType::ValueType(*value_type),
        BlockType::FuncType(func_type) => FormatBlockType::FuncType(func_type.clone()),
        BlockType::TypeIndex(idx) => FormatBlockType::TypeIndex(*idx),
    }
}

/// Convert from format-specific Limits to wrt_types::Limits
///
/// Validates and converts format limits to core limits.
pub fn format_limits_to_wrt_limits(limits: &Limits) -> Result<WrtLimits> {
    // Validate limits
    if let Some(max) = limits.max {
        if max < limits.min {
            return Err(to_wrt_error(validation_error(
                "Maximum limit cannot be less than minimum limit",
            )));
        }
    }

    // Memory64 means 64-bit indices, not supported in the current implementation
    if limits.memory64 {
        return Err(to_wrt_error(validation_error(
            "Memory64 shared memories not supported in current implementation",
        )));
    }

    // Check that memory size is within limits (e.g., 32-bit address space)
    if limits.min > u32::MAX as u64 {
        return Err(to_wrt_error(validation_error(
            "Memory size exceeds maximum allowed in standard WebAssembly",
        )));
    }

    // Convert max to Option<u32>
    let max = limits.max.map(|m| {
        if m > u32::MAX as u64 {
            u32::MAX
        } else {
            m as u32
        }
    });

    Ok(WrtLimits {
        min: limits.min as u32,
        max,
    })
}

/// Convert from wrt_types::Limits to format-specific Limits
///
/// Converts core limits to format-specific limits.
pub fn wrt_limits_to_format_limits(limits: &WrtLimits, shared: bool) -> Limits {
    Limits {
        min: limits.min as u64,
        max: limits.max.map(|m| m as u64),
        memory64: false,
        shared,
    }
}

/// Parse a value type from a binary representation
///
/// This is a wrapper around the core binary_to_val_type function
/// that provides format-specific error handling.
pub fn parse_value_type(byte: u8) -> Result<ValueType> {
    binary_to_val_type(byte).map_err(|_| {
        to_wrt_error(parse_error(format!(
            "Invalid value type byte: 0x{:02x}",
            byte
        )))
    })
}

/// Format a value type to a binary representation
///
/// This is a wrapper around the core val_type_to_binary function.
pub fn format_value_type(val_type: ValueType) -> u8 {
    val_type_to_binary(val_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_types::ValueType;

    #[test]
    fn test_format_block_type_conversion() {
        // Test FormatBlockType -> BlockType
        let format_empty = FormatBlockType::Empty;
        let format_value = FormatBlockType::ValueType(ValueType::I32);
        let format_type_idx = FormatBlockType::TypeIndex(42);

        let block_empty = format_block_type_to_block_type(&format_empty);
        let block_value = format_block_type_to_block_type(&format_value);
        let block_type_idx = format_block_type_to_block_type(&format_type_idx);

        assert!(matches!(block_empty, BlockType::Empty));
        assert!(matches!(block_value, BlockType::Value(ValueType::I32)));
        assert!(matches!(block_type_idx, BlockType::TypeIndex(42)));

        // Test BlockType -> FormatBlockType
        let format_empty_2 = block_type_to_format_block_type(&block_empty);
        let format_value_2 = block_type_to_format_block_type(&block_value);
        let format_type_idx_2 = block_type_to_format_block_type(&block_type_idx);

        assert!(matches!(format_empty_2, FormatBlockType::Empty));
        assert!(matches!(
            format_value_2,
            FormatBlockType::ValueType(ValueType::I32)
        ));
        assert!(matches!(format_type_idx_2, FormatBlockType::TypeIndex(42)));
    }

    #[test]
    fn test_limits_conversion() {
        // Test wrt-types Limits -> FormatLimits
        let wrt_limits_min = wrt_types::component::Limits { min: 10, max: None };
        let wrt_limits_both = wrt_types::component::Limits {
            min: 10,
            max: Some(20),
        };

        let format_limits_min = wrt_limits_to_format_limits(&wrt_limits_min, false);
        let format_limits_both = wrt_limits_to_format_limits(&wrt_limits_both, false);

        assert_eq!(format_limits_min.min, 10);
        assert_eq!(format_limits_min.max, None);

        assert_eq!(format_limits_both.min, 10);
        assert_eq!(format_limits_both.max, Some(20));

        // Test FormatLimits -> wrt-types Limits
        let wrt_limits_min_2 = format_limits_to_wrt_limits(&format_limits_min).unwrap();
        let wrt_limits_both_2 = format_limits_to_wrt_limits(&format_limits_both).unwrap();

        assert_eq!(wrt_limits_min_2.min, 10);
        assert_eq!(wrt_limits_min_2.max, None);

        assert_eq!(wrt_limits_both_2.min, 10);
        assert_eq!(wrt_limits_both_2.max, Some(20));
    }
}
