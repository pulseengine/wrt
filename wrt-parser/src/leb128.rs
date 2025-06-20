//! LEB128 (Little Endian Base 128) encoding and decoding utilities
//!
//! WebAssembly uses LEB128 encoding for variable-length integers.
//! This module provides efficient parsing and encoding functions.

use wrt_error::{Error, ErrorCategory, Result, codes};

/// Read a LEB128 encoded unsigned 32-bit integer
pub fn read_leb128_u32(data: &[u8], mut offset: usize) -> Result<(u32, usize)> {
    let mut result = 0u32;
    let mut shift = 0;
    let start_offset = offset;
    
    loop {
        if offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end of data while reading LEB128"
            ));
        }
        
        let byte = data[offset];
        offset += 1;
        
        if shift >= 32 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "LEB128 value too large for u32"
            ));
        }
        
        result |= ((byte & 0x7F) as u32) << shift;
        
        if (byte & 0x80) == 0 {
            break;
        }
        
        shift += 7;
    }
    
    Ok((result, offset - start_offset))
}

/// Read a LEB128 encoded signed 32-bit integer
pub fn read_leb128_i32(data: &[u8], mut offset: usize) -> Result<(i32, usize)> {
    let mut result = 0i32;
    let mut shift = 0;
    let start_offset = offset;
    let mut byte;
    
    loop {
        if offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end of data while reading LEB128"
            ));
        }
        
        byte = data[offset];
        offset += 1;
        
        if shift >= 32 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "LEB128 value too large for i32"
            ));
        }
        
        result |= ((byte & 0x7F) as i32) << shift;
        shift += 7;
        
        if (byte & 0x80) == 0 {
            break;
        }
    }
    
    // Sign extend if necessary
    if shift < 32 && (byte & 0x40) != 0 {
        result |= -(1 << shift);
    }
    
    Ok((result, offset - start_offset))
}

/// Read a LEB128 encoded unsigned 64-bit integer
pub fn read_leb128_u64(data: &[u8], mut offset: usize) -> Result<(u64, usize)> {
    let mut result = 0u64;
    let mut shift = 0;
    let start_offset = offset;
    
    loop {
        if offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end of data while reading LEB128"
            ));
        }
        
        let byte = data[offset];
        offset += 1;
        
        if shift >= 64 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "LEB128 value too large for u64"
            ));
        }
        
        result |= ((byte & 0x7F) as u64) << shift;
        
        if (byte & 0x80) == 0 {
            break;
        }
        
        shift += 7;
    }
    
    Ok((result, offset - start_offset))
}

/// Read a LEB128 encoded signed 64-bit integer
pub fn read_leb128_i64(data: &[u8], mut offset: usize) -> Result<(i64, usize)> {
    let mut result = 0i64;
    let mut shift = 0;
    let start_offset = offset;
    let mut byte;
    
    loop {
        if offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end of data while reading LEB128"
            ));
        }
        
        byte = data[offset];
        offset += 1;
        
        if shift >= 64 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "LEB128 value too large for i64"
            ));
        }
        
        result |= ((byte & 0x7F) as i64) << shift;
        shift += 7;
        
        if (byte & 0x80) == 0 {
            break;
        }
    }
    
    // Sign extend if necessary
    if shift < 64 && (byte & 0x40) != 0 {
        result |= -(1 << shift);
    }
    
    Ok((result, offset - start_offset))
}

/// Write a LEB128 encoded unsigned 32-bit integer to a buffer
#[cfg(feature = "std")]
pub fn write_leb128_u32(value: u32) -> alloc::vec::Vec<u8> {
    let mut result = alloc::vec::Vec::new();
    let mut value = value;
    
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        
        if value != 0 {
            byte |= 0x80;
        }
        
        result.push(byte);
        
        if value == 0 {
            break;
        }
    }
    
    result
}

/// Write a LEB128 encoded unsigned 32-bit integer to a slice
pub fn write_leb128_u32_to_slice(value: u32, buffer: &mut [u8]) -> Result<usize> {
    let mut value = value;
    let mut offset = 0;
    
    loop {
        if offset >= buffer.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Buffer too small for LEB128 encoding"
            ));
        }
        
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        
        if value != 0 {
            byte |= 0x80;
        }
        
        buffer[offset] = byte;
        offset += 1;
        
        if value == 0 {
            break;
        }
    }
    
    Ok(offset)
}