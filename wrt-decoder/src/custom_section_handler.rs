//! Custom Section Handler for WebAssembly modules
//!
//! This module provides centralized handling for WebAssembly custom sections,
//! including automatic recognition and parsing of well-known sections like
//! branch hints, name sections, and others.

use crate::prelude::*;
use crate::branch_hint_section::{BranchHintSection, parse_branch_hint_section, BRANCH_HINT_SECTION_NAME};
use wrt_error::{Error, ErrorCategory, Result, codes};

#[cfg(feature = "std")]
use std::{vec::Vec, string::String, collections::{BTreeMap, HashMap}};

/// Represents a parsed custom section
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomSection {
    /// Branch hint section for performance optimization
    BranchHint(BranchHintSection),
    /// Name section for debugging information
    Name {
        /// Module name
        module_name: Option<String>,
        /// Function names
        #[cfg(feature = "std")]
        function_names: HashMap<u32, String>,
        #[cfg(all(not(feature = "std")))]
        function_names: BTreeMap<u32, String>,
    },
    /// Unknown custom section (raw data preserved)
    Unknown {
        /// Section name
        name: String,
        /// Raw section data
        data: Vec<u8>,
    },
}

/// Custom section handler that can parse and manage multiple custom sections
#[derive(Debug, Clone)]
pub struct CustomSectionHandler {
    /// Parsed custom sections by name
    #[cfg(feature = "std")]
    sections: HashMap<String, CustomSection>,
    #[cfg(all(not(feature = "std")))]
    sections: BTreeMap<String, CustomSection>,
}

impl CustomSectionHandler {
    /// Create a new custom section handler
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "std")]
            sections: HashMap::new(),
            #[cfg(all(not(feature = "std")))]
            sections: BTreeMap::new(),
        }
    }

    /// Parse and add a custom section
    pub fn add_section(&mut self, name: &str, data: &[u8]) -> Result<()> {
        let section = match name {
            BRANCH_HINT_SECTION_NAME => {
                let branch_hints = parse_branch_hint_section(data)?;
                CustomSection::BranchHint(branch_hints)
            }
            "name" => {
                let name_section = parse_name_section(data)?;
                name_section
            }
            _ => {
                // Unknown section - preserve raw data
                CustomSection::Unknown {
                    name: name.to_string(),
                    data: data.to_vec(),
                }
            }
        };

        self.sections.insert(name.to_string(), section);

        Ok(())
    }

    /// Get branch hint section if present
    pub fn get_branch_hints(&self) -> Option<&BranchHintSection> {
        if let Some(CustomSection::BranchHint(hints)) = self.sections.get(BRANCH_HINT_SECTION_NAME) {
            Some(hints)
        } else {
            None
        }
    }

    /// Get a specific branch hint
    pub fn get_branch_hint(&self, function_index: u32, instruction_offset: u32) -> Option<crate::branch_hint_section::BranchHintValue> {
        self.get_branch_hints()
            .and_then(|hints| hints.get_hint(function_index, instruction_offset))
    }

    /// Get name section information
    pub fn get_function_name(&self, function_index: u32) -> Option<&str> {
        if let Some(CustomSection::Name { function_names, .. }) = self.sections.get("name") {
            function_names.get(&function_index).map(|s| s.as_str())
        } else {
            None
        }
    }

    /// Get module name if present
    pub fn get_module_name(&self) -> Option<&str> {
        if let Some(CustomSection::Name { module_name, .. }) = self.sections.get("name") {
            module_name.as_ref().map(|s| s.as_str())
        } else {
            None
        }
    }

    /// Check if branch hints are available
    pub fn has_branch_hints(&self) -> bool {
        self.get_branch_hints().is_some()
    }

    /// Get all section names
    pub fn section_names(&self) -> Vec<String> {
        self.sections.keys().cloned().collect()
    }

    /// Get number of custom sections
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }
}

impl Default for CustomSectionHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a WebAssembly name section
fn parse_name_section(data: &[u8]) -> Result<CustomSection> {
    // Simplified name section parsing - normally this would be more complex
    // For now, just create an empty name section
    Ok(CustomSection::Name {
        module_name: None,
        #[cfg(feature = "std")]
        function_names: HashMap::new(),
        #[cfg(all(not(feature = "std")))]
        function_names: BTreeMap::new(),
    })
}

impl Default for CustomSection {
    fn default() -> Self {
        CustomSection::Unknown {
            name: String::new(),
            data: Vec::new(),
        }
    }
}

/// Utility function to extract custom section name and data from a complete custom section
pub fn extract_custom_section(section_data: &[u8]) -> Result<(String, &[u8])> {
    use wrt_format::binary::read_leb128_u32;
    
    // Read name length
    let (name_len, mut offset) = read_leb128_u32(section_data, 0)?;
    
    // Read name string
    if offset + name_len as usize > section_data.len() {
        return Err(Error::new(
            ErrorCategory::Parse,
            codes::PARSE_ERROR,
            "Custom section name length exceeds section size"
        ));
    }
    
    let name_bytes = section_data[offset..offset + name_len as usize].to_vec();
    let name = String::from_utf8(name_bytes).map_err(|_| Error::new(
        ErrorCategory::Parse,
        codes::PARSE_ERROR,
        "Invalid UTF-8 in custom section name"
    ))?;
    
    
    offset += name_len as usize;
    
    // Return name and remaining data
    Ok((name, &section_data[offset..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::branch_hint_section::{BranchHintValue, FunctionBranchHints};

    #[cfg(feature = "std")]
    #[test]
    fn test_custom_section_handler() {
        let mut handler = CustomSectionHandler::new();
        
        // Create test branch hint data
        let mut section = BranchHintSection::new();
        let mut func_hints = FunctionBranchHints::new(0);
        func_hints.add_hint(10, BranchHintValue::LikelyTrue).unwrap();
        section.add_function_hints(func_hints).unwrap();
        
        let encoded = crate::branch_hint_section::encode_branch_hint_section(&section).unwrap();
        
        // Add branch hint section
        handler.add_section(BRANCH_HINT_SECTION_NAME, &encoded).unwrap();
        
        // Verify it's accessible
        assert!(handler.has_branch_hints());
        assert_eq!(handler.get_branch_hint(0, 10), Some(BranchHintValue::LikelyTrue));
        assert_eq!(handler.get_branch_hint(0, 20), None);
        assert_eq!(handler.get_branch_hint(1, 10), None);
        
        // Add unknown section
        handler.add_section("unknown", &[1, 2, 3, 4]).unwrap();
        
        assert_eq!(handler.section_count(), 2);
        let names = handler.section_names();
        assert!(names.contains(&BRANCH_HINT_SECTION_NAME.to_string()));
        assert!(names.contains(&"unknown".to_string()));
    }

    #[test]
    fn test_extract_custom_section() {
        // Create test custom section data: name length + name + data
        let mut section_data = Vec::new();
        let name = "test";
        section_data.push(name.len() as u8); // LEB128 encoding of length
        section_data.extend_from_slice(name.as_bytes());
        section_data.extend_from_slice(&[1, 2, 3, 4]); // test data
        
        let (extracted_name, data) = extract_custom_section(&section_data).unwrap();
        assert_eq!(extracted_name, "test");
        assert_eq!(data, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_extract_custom_section_invalid() {
        // Test with truncated data
        let section_data = &[5, b't', b'e', b's']; // name length = 5, but only 4 bytes
        assert!(extract_custom_section(section_data).is_err());
        
        // Test with invalid UTF-8
        let section_data = &[2, 0xFF, 0xFE]; // invalid UTF-8 bytes
        assert!(extract_custom_section(section_data).is_err());
    }
}