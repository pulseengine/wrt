// Enhanced error handling for WebAssembly section parsing
//
// This module provides specialized error types for WebAssembly section
// parsing with improved context information and position tracking.

use wrt_error::{kinds, Error, Result, WrtError, codes, ErrorCategory};

/// Specialized error enum for section parsing failures
#[derive(Debug, Clone, PartialEq)]
pub enum SectionError {
    /// Error when a required section is missing
    MissingSection { 
        id: u8,
        description: String,
    },
    
    /// Error when a section is invalid
    InvalidSection { 
        id: u8, 
        offset: usize,
        msg: String,
    },
    
    /// Error when unexpected end of data is encountered
    UnexpectedEnd { 
        offset: usize, 
        expected: usize,
        actual: usize,
    },
    
    /// Error when section content is malformed
    MalformedContent { 
        offset: usize,
        section_id: u8,
        msg: String,
    },
    
    /// Error when a section size exceeds the module size
    SectionSizeExceedsModule {
        section_id: u8,
        section_size: u32,
        module_size: usize,
        offset: usize,
    },
    
    /// Error when an incorrect magic header is encountered
    InvalidMagic {
        offset: usize,
        expected: [u8; 4],
        actual: [u8; 4],
    },
    
    /// Error when an unsupported version is encountered
    UnsupportedVersion {
        offset: usize,
        expected: [u8; 4],
        actual: [u8; 4],
    },
    
    /// Error when an invalid value is encountered in a section
    InvalidValue {
        offset: usize,
        section_id: u8,
        description: String,
    },
}

/// Extension trait to convert section errors to wrt_error::Error
pub trait SectionErrorExt {
    /// Convert a SectionError to an Error with appropriate context
    fn to_error(self) -> Error;
}

impl SectionErrorExt for SectionError {
    fn to_error(self) -> Error {
        match self {
            SectionError::MissingSection { id, description } => {
                Error::new(
                    ErrorCategory::Validation, 
                    codes::PARSE_ERROR,
                    kinds::ParseError(
                        format!("Missing section (ID: 0x{:02x}): {}", id, description)
                    )
                )
            },
            SectionError::InvalidSection { id, offset, msg } => {
                Error::new(
                    ErrorCategory::Validation,
                    codes::PARSE_ERROR,
                    kinds::ParseError(
                        format!("Invalid section (ID: 0x{:02x}) at offset 0x{:x}: {}", id, offset, msg)
                    )
                )
            },
            SectionError::UnexpectedEnd { offset, expected, actual } => {
                Error::new(
                    ErrorCategory::Validation,
                    codes::PARSE_ERROR,
                    kinds::ParseError(
                        format!("Unexpected end of data at offset 0x{:x}: expected {} bytes, but only {} available", 
                            offset, expected, actual)
                    )
                )
            },
            SectionError::MalformedContent { offset, section_id, msg } => {
                Error::new(
                    ErrorCategory::Validation,
                    codes::PARSE_ERROR,
                    kinds::ParseError(
                        format!("Malformed content in section (ID: 0x{:02x}) at offset 0x{:x}: {}", 
                            section_id, offset, msg)
                    )
                )
            },
            SectionError::SectionSizeExceedsModule { section_id, section_size, module_size, offset } => {
                Error::new(
                    ErrorCategory::Validation,
                    codes::PARSE_ERROR,
                    kinds::ParseError(
                        format!("Section size exceeds module size: section (ID: 0x{:02x}) at offset 0x{:x} has size {}, but module only has {} bytes remaining", 
                            section_id, offset, section_size, module_size)
                    )
                )
            },
            SectionError::InvalidMagic { offset, expected, actual } => {
                Error::new(
                    ErrorCategory::Validation,
                    codes::PARSE_ERROR,
                    kinds::ParseError(
                        format!("Invalid WebAssembly magic bytes at offset 0x{:x}: expected {:?}, found {:?}", 
                            offset, expected, actual)
                    )
                )
            },
            SectionError::UnsupportedVersion { offset, expected, actual } => {
                Error::new(
                    ErrorCategory::Validation,
                    codes::PARSE_ERROR,
                    kinds::ParseError(
                        format!("Unsupported WebAssembly version at offset 0x{:x}: expected {:?}, found {:?}", 
                            offset, expected, actual)
                    )
                )
            },
            SectionError::InvalidValue { offset, section_id, description } => {
                Error::new(
                    ErrorCategory::Validation,
                    codes::PARSE_ERROR,
                    kinds::ParseError(
                        format!("Invalid value in section (ID: 0x{:02x}) at offset 0x{:x}: {}", 
                            section_id, offset, description)
                    )
                )
            },
        }
    }
}

/// Helper function to create a missing section error
pub fn missing_section(id: u8, description: &str) -> Error {
    SectionError::MissingSection { 
        id, 
        description: description.to_string() 
    }.to_error()
}

/// Helper function to create an invalid section error
pub fn invalid_section(id: u8, offset: usize, msg: &str) -> Error {
    SectionError::InvalidSection { 
        id, 
        offset, 
        msg: msg.to_string() 
    }.to_error()
}

/// Helper function to create an unexpected end error
pub fn unexpected_end(offset: usize, expected: usize, actual: usize) -> Error {
    SectionError::UnexpectedEnd { 
        offset, 
        expected, 
        actual 
    }.to_error()
}

/// Helper function to create a malformed content error
pub fn malformed_content(offset: usize, section_id: u8, msg: &str) -> Error {
    SectionError::MalformedContent { 
        offset, 
        section_id, 
        msg: msg.to_string() 
    }.to_error()
}

/// Helper function to create a section size exceeds module error
pub fn section_size_exceeds_module(section_id: u8, section_size: u32, module_size: usize, offset: usize) -> Error {
    SectionError::SectionSizeExceedsModule { 
        section_id,
        section_size, 
        module_size, 
        offset 
    }.to_error()
}

/// Helper function to create an invalid magic error
pub fn invalid_magic(offset: usize, expected: [u8; 4], actual: [u8; 4]) -> Error {
    SectionError::InvalidMagic { 
        offset, 
        expected, 
        actual 
    }.to_error()
}

/// Helper function to create an unsupported version error
pub fn unsupported_version(offset: usize, expected: [u8; 4], actual: [u8; 4]) -> Error {
    SectionError::UnsupportedVersion { 
        offset, 
        expected, 
        actual 
    }.to_error()
}

/// Helper function to create an invalid value error
pub fn invalid_value(offset: usize, section_id: u8, description: &str) -> Error {
    SectionError::InvalidValue { 
        offset, 
        section_id, 
        description: description.to_string() 
    }.to_error()
}

/// Helper function to create an invalid UTF-8 error
pub fn invalid_utf8(offset: usize) -> Error {
    SectionError::InvalidValue {
        offset,
        section_id: 0, // Generic section ID as this could occur in various sections
        description: "Invalid UTF-8 string".to_string()
    }.to_error()
}

/// Helper function to create an invalid value type error
pub fn invalid_value_type(type_byte: u8, offset: usize) -> Error {
    SectionError::InvalidValue {
        offset,
        section_id: 0, // Generic section ID as this could occur in various sections
        description: format!("Invalid value type: 0x{:02x}", type_byte)
    }.to_error()
}

/// Helper function to create an invalid import kind error
pub fn invalid_import_kind(kind_byte: u8, offset: usize) -> Error {
    SectionError::InvalidValue {
        offset,
        section_id: 2, // Import section
        description: format!("Invalid import kind: 0x{:02x}", kind_byte)
    }.to_error()
}

/// Helper function to create an invalid mutability flag error
pub fn invalid_mutability(mutability_byte: u8, offset: usize) -> Error {
    SectionError::InvalidValue {
        offset,
        section_id: 2, // Import section (or 6 for global section)
        description: format!("Invalid mutability flag: 0x{:02x}, expected 0 or 1", mutability_byte)
    }.to_error()
}

pub fn invalid_section_id(id: u8) -> WrtError {
    WrtError::parse_error(format!("Invalid section ID: {}", id))
}

pub fn invalid_section_size(size: u32) -> WrtError {
    WrtError::parse_error(format!("Invalid section size: {}", size))
}

pub fn invalid_section_order(expected: u8, got: u8) -> WrtError {
    WrtError::parse_error(format!(
        "Invalid section order: expected {}, got {}",
        expected, got
    ))
}

pub fn invalid_section_content(message: &str) -> WrtError {
    WrtError::parse_error(message.to_string())
}

pub fn invalid_section_name(name: &str) -> WrtError {
    WrtError::parse_error(format!("Invalid section name: {}", name))
}

pub fn invalid_section_data(message: &str) -> WrtError {
    WrtError::parse_error(message.to_string())
}

pub fn invalid_section_format(message: &str) -> WrtError {
    WrtError::parse_error(message.to_string())
}

pub fn parse_error(message: &str) -> WrtError {
    WrtError::parse_error(message.to_string())
}

pub fn parse_error_with_context(message: &str, context: &str) -> WrtError {
    WrtError::parse_error(format!("{}: {}", message, context))
}

pub fn parse_error_with_position(message: &str, position: usize) -> WrtError {
    WrtError::parse_error(format!("{} at position {}", message, position))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_section_errors() {
        // Test MissingSection error
        let error = missing_section(1, "Import section required");
        assert!(format!("{}", error).contains("Missing section"));
        assert!(format!("{}", error).contains("Import section required"));
        
        // Test InvalidSection error
        let error = invalid_section(2, 0x20, "Invalid count");
        assert!(format!("{}", error).contains("Invalid section"));
        assert!(format!("{}", error).contains("0x20"));
        
        // Test UnexpectedEnd error
        let error = unexpected_end(0x30, 10, 5);
        assert!(format!("{}", error).contains("Unexpected end"));
        assert!(format!("{}", error).contains("0x30"));
        
        // Test MalformedContent error
        let error = malformed_content(0x40, 3, "Invalid function type");
        assert!(format!("{}", error).contains("Malformed content"));
        assert!(format!("{}", error).contains("Invalid function type"));
        
        // Test SectionSizeExceedsModule error
        let error = section_size_exceeds_module(4, 100, 50, 0x50);
        assert!(format!("{}", error).contains("Section size exceeds module size"));
        assert!(format!("{}", error).contains("100"));
        
        // Test InvalidMagic error
        let error = invalid_magic(0, [0x00, 0x61, 0x73, 0x6d], [0x01, 0x02, 0x03, 0x04]);
        assert!(format!("{}", error).contains("Invalid WebAssembly magic bytes"));
        
        // Test UnsupportedVersion error
        let error = unsupported_version(4, [0x01, 0x00, 0x00, 0x00], [0x02, 0x00, 0x00, 0x00]);
        assert!(format!("{}", error).contains("Unsupported WebAssembly version"));
        
        // Test InvalidValue error
        let error = invalid_value(0x60, 5, "Invalid limit type");
        assert!(format!("{}", error).contains("Invalid value"));
        assert!(format!("{}", error).contains("Invalid limit type"));
    }
} 