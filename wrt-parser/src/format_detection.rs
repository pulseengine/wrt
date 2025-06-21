//! Binary format detection for WebAssembly binaries
//!
//! This module provides automatic detection of WebAssembly binary formats,
//! distinguishing between Core WebAssembly modules and Component Model binaries.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::binary_constants;

/// WebAssembly binary format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    /// Core WebAssembly module (.wasm)
    CoreModule,
    /// Component Model binary (.wasm component)
    Component,
    /// Unknown or invalid format
    Unknown,
}

/// Component Model magic number and version detection
pub mod component_constants {
    /// Component Model magic number (same as core WebAssembly)
    pub const COMPONENT_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];
    /// Component Model version (1)
    pub const COMPONENT_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];
    /// Component Model layer identifier (1)
    pub const COMPONENT_LAYER: [u8; 4] = [0x01, 0x00, 0x00, 0x00];
}

/// Binary format detector with ASIL-D compliant analysis
#[derive(Debug)]
pub struct FormatDetector {
    /// Minimum bytes required for format detection
    min_bytes_required: usize,
}

impl FormatDetector {
    /// Create a new format detector
    pub fn new() -> Self {
        Self {
            min_bytes_required: 12, // Magic (4) + Version (4) + Layer/Extra (4)
        }
    }
    
    /// Detect the format of a WebAssembly binary
    /// 
    /// This method analyzes the binary header to determine if it's a
    /// Core WebAssembly module or a Component Model binary.
    pub fn detect_format(&self, binary: &[u8]) -> Result<BinaryFormat> {
        // Validate minimum size for a valid WebAssembly binary (8 bytes: magic + version)
        if binary.len() < 8 {
            return Ok(BinaryFormat::Unknown);
        }
        
        // Check magic number first
        if !self.has_valid_magic(binary) {
            return Ok(BinaryFormat::Unknown);
        }
        
        // Parse version to distinguish formats
        let version = self.parse_version(binary)?;
        
        // First check if this is definitely a Component Model binary
        if self.is_component_format(binary, version)? {
            Ok(BinaryFormat::Component)
        } else if version == 1 {
            // If it has valid version but isn't a component, it's a core module
            Ok(BinaryFormat::CoreModule)
        } else {
            Ok(BinaryFormat::Unknown)
        }
    }
    
    /// Validate that the binary has a valid WebAssembly magic number
    fn has_valid_magic(&self, binary: &[u8]) -> bool {
        binary.len() >= 4 && &binary[0..4] == &binary_constants::WASM_MAGIC
    }
    
    /// Parse the version field from the binary
    fn parse_version(&self, binary: &[u8]) -> Result<u32> {
        if binary.len() < 8 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Binary too small to contain version field"
            ));
        }
        
        let version_bytes = [binary[4], binary[5], binary[6], binary[7]];
        Ok(u32::from_le_bytes(version_bytes))
    }
    
    /// Check if the binary is a Component Model format
    fn is_component_format(&self, binary: &[u8], version: u32) -> Result<bool> {
        // Component Model has version 1 and a layer field
        if version != 1 {
            return Ok(false);
        }
        
        // Component Model binaries have a layer field after version
        if binary.len() < 12 {
            return Ok(false);
        }
        
        let layer_bytes = [binary[8], binary[9], binary[10], binary[11]];
        let layer = u32::from_le_bytes(layer_bytes);
        
        // Component Model uses layer 1
        Ok(layer == 1)
    }
    
    /// Check if the binary is a Core WebAssembly format
    fn is_core_format(&self, binary: &[u8], version: u32) -> Result<bool> {
        // Core WebAssembly has version 1
        if version != 1 {
            return Ok(false);
        }
        
        // Core WebAssembly doesn't have a layer field, or has layer 0
        // Check if this could be a component first (has layer 1)
        if binary.len() >= 12 {
            let layer_bytes = [binary[8], binary[9], binary[10], binary[11]];
            let layer = u32::from_le_bytes(layer_bytes);
            
            if layer == 1 {
                return Ok(false); // This is likely a component
            }
        }
        
        // If we get here, it's either an empty module or starts with sections
        Ok(true)
    }
    
    /// Check if a section ID is valid for Core WebAssembly
    fn is_valid_core_section_id(&self, section_id: u8) -> bool {
        matches!(section_id, 
            binary_constants::CUSTOM_SECTION_ID |
            binary_constants::TYPE_SECTION_ID |
            binary_constants::IMPORT_SECTION_ID |
            binary_constants::FUNCTION_SECTION_ID |
            binary_constants::TABLE_SECTION_ID |
            binary_constants::MEMORY_SECTION_ID |
            binary_constants::GLOBAL_SECTION_ID |
            binary_constants::EXPORT_SECTION_ID |
            binary_constants::START_SECTION_ID |
            binary_constants::ELEMENT_SECTION_ID |
            binary_constants::CODE_SECTION_ID |
            binary_constants::DATA_SECTION_ID |
            binary_constants::DATA_COUNT_SECTION_ID
        )
    }
    
    /// Get minimum bytes required for reliable format detection
    pub fn min_bytes_required(&self) -> usize {
        self.min_bytes_required
    }
    
    /// Quick format detection for streaming scenarios
    /// 
    /// This method provides fast format detection using only the header,
    /// suitable for streaming parsers that need to route to the correct
    /// parser implementation quickly.
    pub fn quick_detect(&self, header: &[u8]) -> BinaryFormat {
        if header.len() < 8 {
            return BinaryFormat::Unknown;
        }
        
        // Check magic
        if &header[0..4] != &binary_constants::WASM_MAGIC {
            return BinaryFormat::Unknown;
        }
        
        // Check version
        let version_bytes = [header[4], header[5], header[6], header[7]];
        let version = u32::from_le_bytes(version_bytes);
        
        if version != 1 {
            return BinaryFormat::Unknown;
        }
        
        // If we have layer information, use it
        if header.len() >= 12 {
            let layer_bytes = [header[8], header[9], header[10], header[11]];
            let layer = u32::from_le_bytes(layer_bytes);
            
            if layer == 1 {
                return BinaryFormat::Component;
            }
        }
        
        // Default to core module if layer is 0 or not present
        BinaryFormat::CoreModule
    }
    
    /// Validate a binary format against expected type
    pub fn validate_format(&self, binary: &[u8], expected: BinaryFormat) -> Result<()> {
        let detected = self.detect_format(binary)?;
        
        if detected != expected {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Binary format does not match expected format"
            ));
        }
        
        Ok(())
    }
}

impl Default for FormatDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Format-specific binary information
#[derive(Debug, Clone)]
pub struct BinaryInfo {
    /// Detected format
    pub format: BinaryFormat,
    /// Binary version
    pub version: u32,
    /// Layer (for Component Model)
    pub layer: Option<u32>,
    /// Size of the binary
    pub size: usize,
    /// Header size consumed for format detection
    pub header_size: usize,
}

impl BinaryInfo {
    /// Create binary info from format detection
    pub fn from_binary(binary: &[u8]) -> Result<Self> {
        let detector = FormatDetector::new();
        let format = detector.detect_format(binary)?;
        
        if binary.len() < 8 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Binary too small for format analysis"
            ));
        }
        
        let version_bytes = [binary[4], binary[5], binary[6], binary[7]];
        let version = u32::from_le_bytes(version_bytes);
        
        let (layer, header_size) = if format == BinaryFormat::Component && binary.len() >= 12 {
            let layer_bytes = [binary[8], binary[9], binary[10], binary[11]];
            let layer = u32::from_le_bytes(layer_bytes);
            (Some(layer), 12)
        } else {
            (None, 8)
        };
        
        Ok(BinaryInfo {
            format,
            version,
            layer,
            size: binary.len(),
            header_size,
        })
    }
    
    /// Check if the binary is a Component Model binary
    pub fn is_component(&self) -> bool {
        self.format == BinaryFormat::Component
    }
    
    /// Check if the binary is a Core WebAssembly module
    pub fn is_core_module(&self) -> bool {
        self.format == BinaryFormat::CoreModule
    }
    
    /// Get the effective data start offset (after headers)
    pub fn data_start_offset(&self) -> usize {
        self.header_size
    }
}

/// Convenience function for quick format detection
pub fn detect_format(binary: &[u8]) -> Result<BinaryFormat> {
    FormatDetector::new().detect_format(binary)
}

/// Convenience function for quick format detection without error handling
pub fn quick_detect_format(header: &[u8]) -> BinaryFormat {
    FormatDetector::new().quick_detect(header)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_detector_creation() {
        let detector = FormatDetector::new();
        assert_eq!(detector.min_bytes_required(), 12);
    }
    
    #[test]
    fn test_core_module_detection() {
        let detector = FormatDetector::new();
        
        // Core WebAssembly module: magic + version + type section
        let core_binary = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
            0x01,                   // Type section ID
            0x00,                   // Section size (empty)
        ];
        
        let format = detector.detect_format(&core_binary).unwrap();
        assert_eq!(format, BinaryFormat::CoreModule);
    }
    
    #[test]
    fn test_component_detection() {
        let detector = FormatDetector::new();
        
        // Component Model binary: magic + version + layer
        let component_binary = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
            0x01, 0x00, 0x00, 0x00, // Layer 1 (Component Model)
            0x07,                   // Component type section
        ];
        
        let format = detector.detect_format(&component_binary).unwrap();
        assert_eq!(format, BinaryFormat::Component);
    }
    
    #[test]
    fn test_invalid_magic() {
        let detector = FormatDetector::new();
        
        let invalid_binary = [
            0xFF, 0xFF, 0xFF, 0xFF, // Invalid magic
            0x01, 0x00, 0x00, 0x00, // Version 1
        ];
        
        let format = detector.detect_format(&invalid_binary).unwrap();
        assert_eq!(format, BinaryFormat::Unknown);
    }
    
    #[test]
    fn test_too_small_binary() {
        let detector = FormatDetector::new();
        
        let small_binary = [0x00, 0x61]; // Only 2 bytes
        
        let format = detector.detect_format(&small_binary).unwrap();
        assert_eq!(format, BinaryFormat::Unknown);
    }
    
    #[test]
    fn test_quick_detection() {
        let detector = FormatDetector::new();
        
        // Component header
        let component_header = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
            0x01, 0x00, 0x00, 0x00, // Layer 1
        ];
        
        let format = detector.quick_detect(&component_header);
        assert_eq!(format, BinaryFormat::Component);
        
        // Core module header (no layer or layer 0)
        let core_header = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
            0x00, 0x00, 0x00, 0x00, // Layer 0 (or section start)
        ];
        
        let format = detector.quick_detect(&core_header);
        assert_eq!(format, BinaryFormat::CoreModule);
    }
    
    #[test]
    fn test_binary_info_creation() {
        let component_binary = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
            0x01, 0x00, 0x00, 0x00, // Layer 1
            0x07, 0x00,             // Component type section
        ];
        
        let info = BinaryInfo::from_binary(&component_binary).unwrap();
        assert_eq!(info.format, BinaryFormat::Component);
        assert_eq!(info.version, 1);
        assert_eq!(info.layer, Some(1));
        assert_eq!(info.size, 14);
        assert_eq!(info.header_size, 12);
        assert!(info.is_component());
        assert!(!info.is_core_module());
    }
    
    #[test]
    fn test_format_validation() {
        let detector = FormatDetector::new();
        
        let core_binary = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
            0x01, 0x00,             // Type section
        ];
        
        // Should succeed for correct format
        assert!(detector.validate_format(&core_binary, BinaryFormat::CoreModule).is_ok());
        
        // Should fail for incorrect format
        assert!(detector.validate_format(&core_binary, BinaryFormat::Component).is_err());
    }
    
    #[test]
    fn test_convenience_functions() {
        let core_binary = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
            0x01, 0x00,             // Type section
        ];
        
        let format = detect_format(&core_binary).unwrap();
        assert_eq!(format, BinaryFormat::CoreModule);
        
        let quick_format = quick_detect_format(&core_binary[0..8]);
        assert_eq!(quick_format, BinaryFormat::CoreModule);
    }
}