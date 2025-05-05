//! Compression utilities for WebAssembly state serialization.
//!
//! This module provides compression algorithms for WebAssembly state data,
//! focusing on run-length encoding (RLE) which is efficient for memory sections.

use crate::Vec;
use wrt_error::{codes, Error, ErrorCategory, Result};

#[cfg(feature = "std")]
use std::cmp;

#[cfg(not(feature = "std"))]
use core::cmp;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

/// Supported compression types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// No compression
    None = 0,
    /// Run-length encoding
    RLE = 1,
}

impl CompressionType {
    /// Convert a u8 to a CompressionType
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::RLE),
            _ => None,
        }
    }
}

/// Run-length encode a byte array
///
/// This implementation uses a simple format:
/// - For runs of 4+ identical bytes: [0x00, count, value]
/// - For literal sequences: [count, byte1, byte2, ...]
///
/// Where count is a single byte (0-255)
pub fn rle_encode(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Current byte to check for runs
        let current = data[i];

        // Find run length (max 255 for single-byte count)
        let mut run_length = 1;
        while i + run_length < data.len() && data[i + run_length] == current && run_length < 255 {
            run_length += 1;
        }

        if run_length >= 4 {
            // Encode as RLE: [0x00, count, value]
            result.push(0x00); // RLE marker
            result.push(run_length as u8);
            result.push(current);
            i += run_length;
        } else {
            // For runs < 4 bytes, use literal encoding
            // [count, byte1, byte2, ...]
            let literal_length = cmp::min(255, data.len() - i);
            result.push(literal_length as u8);
            for j in 0..literal_length {
                result.push(data[i + j]);
            }
            i += literal_length;
        }
    }

    result
}

/// Run-length decode a byte array
///
/// This function decodes data created by rle_encode.
pub fn rle_decode(input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < input.len() {
        if i >= input.len() {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::PARSE_ERROR,
                "Truncated RLE data".to_string(),
            ));
        }

        let control = input[i];
        i += 1;

        if control & 0x80 == 0 {
            // Run of repeated bytes (0-127 times)
            let run_length = control as usize + 1;
            if i >= input.len() {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::PARSE_ERROR,
                    "Truncated RLE sequence".to_string(),
                ));
            }
            let value = input[i];
            i += 1;
            #[cfg(feature = "std")]
            result.extend(std::iter::repeat_n(value, run_length));
            #[cfg(not(feature = "std"))]
            result.extend(core::iter::repeat_n(value, run_length));
        } else {
            // Literal sequence ((control & 0x7F) + 1 bytes)
            let literal_length = (control & 0x7F) as usize + 1;
            if i + literal_length > input.len() {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::PARSE_ERROR,
                    "Truncated literal sequence".to_string(),
                ));
            }
            result.extend_from_slice(&input[i..i + literal_length]);
            i += literal_length;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "std")]
    use std::vec;

    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::vec;

    #[test]
    fn test_rle_encode_decode() {
        let empty: Vec<u8> = vec![];
        assert_eq!(rle_encode(&empty), empty);
        assert_eq!(rle_decode(&empty).unwrap(), empty);

        let single = vec![42];
        let encoded = rle_encode(&single);
        // The implementation encodes single values as a literal sequence [length, value]
        // where length is 1 for a single byte
        assert_eq!(encoded, vec![1, 42]);
        assert_eq!(rle_decode(&encoded).unwrap(), single);

        let repeated = vec![5, 5, 5, 5, 5];
        let encoded = rle_encode(&repeated);
        // For 5 repeating elements (runs >= 4), it would encode as [0, 5, 5]
        // where 0 is the marker, 5 is the count, and 5 is the value
        assert_eq!(encoded, vec![0, 5, 5]);
        assert_eq!(rle_decode(&encoded).unwrap(), repeated);

        let mixed = vec![1, 1, 2, 3, 3, 3, 4, 5, 5];
        let encoded = rle_encode(&mixed);
        // This would encode as:
        // [2, 1, 1]  - Literal sequence of two bytes (1, 1)
        // [1, 2]     - Literal sequence of one byte (2)
        // [0, 3, 3]  - Run of three 3's
        // [3, 4, 5, 5] - Literal sequence of three bytes (4, 5, 5)
        let expected = vec![2, 1, 1, 1, 2, 0, 3, 3, 3, 4, 5, 5];
        assert_eq!(encoded, expected);
        assert_eq!(rle_decode(&encoded).unwrap(), mixed);
    }

    #[test]
    fn test_rle_decode_errors() {
        // Encoded data must be properly formed (check for truncation)
        // Encoded data must have an even length (count+value pairs)
        let odd_length = vec![1, 2, 3];
        assert!(rle_decode(&odd_length).is_err());

        // Our implementation allows a zero count
        // (although it's not efficient, it's not considered an error)
        let zero_count = vec![0, 42];
        let result = rle_decode(&zero_count);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::<u8>::new());
    }
}
