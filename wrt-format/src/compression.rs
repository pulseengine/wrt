//! Compression utilities for WebAssembly state serialization.
//!
//! This module provides compression algorithms for WebAssembly state data,
//! focusing on run-length encoding (RLE) which is efficient for memory sections.

use crate::Vec;
use wrt_error::{kinds, Error, Result};

#[cfg(feature = "std")]
use std::cmp;

#[cfg(not(feature = "std"))]
use core::{cmp, iter};

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
            return Err(Error::new(kinds::ParseError(
                "Truncated RLE data".to_string(),
            )));
        }

        let control = input[i];
        i += 1;

        if control & 0x80 == 0 {
            // Run of repeated bytes (0-127 times)
            let run_length = control as usize + 1;
            if i >= input.len() {
                return Err(Error::new(kinds::ParseError(
                    "Truncated RLE sequence".to_string(),
                )));
            }
            let value = input[i];
            i += 1;
            result.extend(std::iter::repeat_n(value, run_length));
        } else {
            // Literal sequence ((control & 0x7F) + 1 bytes)
            let literal_length = (control & 0x7F) as usize + 1;
            if i + literal_length > input.len() {
                return Err(Error::new(kinds::ParseError(
                    "Truncated literal sequence".to_string(),
                )));
            }
            result.extend_from_slice(&input[i..i + literal_length]);
            i += literal_length;
        }
    }

    Ok(result)
}

#[cfg(feature = "kani")]
mod verification {
    use super::*;
    use kani::*;

    #[kani::proof]
    fn verify_rle_roundtrip_small() {
        // Create a small array of test data - Kani works better with small bounds
        let data_len: u8 = any_where(|x| *x < 10);
        let mut test_data = Vec::with_capacity(data_len as usize);

        for _ in 0..data_len {
            test_data.push(any::<u8>());
        }

        // Encode and then decode
        let encoded = rle_encode(&test_data);
        let decoded = rle_decode(&encoded).unwrap();

        // Verify roundtrip preservation
        assert_eq!(test_data.len(), decoded.len());
        for i in 0..test_data.len() {
            assert_eq!(test_data[i], decoded[i]);
        }
    }

    #[kani::proof]
    fn verify_rle_empty_data() {
        let empty: [u8; 0] = [];
        let encoded = rle_encode(&empty);
        let decoded = rle_decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 0);
    }

    #[kani::proof]
    fn verify_rle_repeated_data() {
        // Test with identical bytes
        let repeated_value = any::<u8>();
        let len: u8 = any_where(|x| *x > 4 && *x < 10); // Keep small for Kani

        let mut test_data = Vec::with_capacity(len as usize);
        for _ in 0..len {
            test_data.push(repeated_value);
        }

        let encoded = rle_encode(&test_data);
        let decoded = rle_decode(&encoded).unwrap();

        // Verify compression is efficient for repeated data
        assert!(encoded.len() < test_data.len());

        // Verify decompression is correct
        assert_eq!(decoded.len(), test_data.len());
        for i in 0..test_data.len() {
            assert_eq!(decoded[i], repeated_value);
        }
    }
}
