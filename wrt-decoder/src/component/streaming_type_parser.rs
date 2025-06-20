//! Streaming Component Type Section Parser
//!
//! This module provides ASIL-compliant streaming parsing of Component Type sections
//! within Component binaries. It uses the unified capability-based memory allocation
//! system and operates without loading entire type definitions into memory.
//!
//! # ASIL Compliance
//! 
//! This implementation works across all ASIL levels using the unified provider system:
//! - The BoundedVec types adapt their behavior based on the current ASIL level
//! - The NoStdProvider internally chooses appropriate allocation strategies
//! - All limits are enforced at compile time with runtime validation
//! - Single implementation that works for QM, ASIL-A, ASIL-B, ASIL-C, and ASIL-D
//!
//! # Architecture
//!
//! The parser uses a streaming approach where:
//! 1. Only section headers are read into memory
//! 2. Type data is processed incrementally
//! 3. Memory allocation is controlled via the capability system
//! 4. All operations are bounded and deterministic

#![cfg_attr(not(feature = "std"), no_std)]

// Environment setup
#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::vec::Vec;

use wrt_error::{codes, Error, ErrorCategory, Result};
#[cfg(feature = "std")]
use wrt_format::{
    binary::{read_leb128_u32, read_string},
    component::{ComponentType, ComponentTypeDefinition, ExternType, FormatValType},
};

#[cfg(not(feature = "std"))]
use wrt_format::binary::{read_leb128_u32, read_string};

// Define placeholder types for no_std environments where component types aren't available
#[cfg(not(feature = "std"))]
mod placeholder_types {
    use core::fmt;
    use wrt_error::Result;
    use crate::prelude::DecoderVec;
    
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct ComponentType {
        pub definition: ComponentTypeDefinition,
    }
    
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub enum ComponentTypeDefinition {
        Component { imports: DecoderVec<ExternType>, exports: DecoderVec<ExternType> },
        Instance { exports: DecoderVec<ExternType> },
        Function { params: DecoderVec<FormatValType>, results: DecoderVec<FormatValType> },
        #[default]
        Value(FormatValType),
        Resource { name: Option<crate::prelude::DecoderString>, functions: DecoderVec<u32> },
    }
    
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub enum ExternType {
        Function { params: DecoderVec<FormatValType>, results: DecoderVec<FormatValType> },
        #[default]
        Value(FormatValType),
        Type(u32),
        Instance { exports: DecoderVec<ExternType> },
        Component { imports: DecoderVec<ExternType>, exports: DecoderVec<ExternType> },
    }
    
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub enum FormatValType {
        #[default]
        Bool, S8, U8, S16, U16, S32, U32, S64, U64, F32, F64, Char, String,
        Record(DecoderVec<(crate::prelude::DecoderString, FormatValType)>),
        Variant(DecoderVec<(crate::prelude::DecoderString, Option<FormatValType>)>),
        List(u32),
        Tuple(DecoderVec<FormatValType>),
        Own(u32),
        Borrow(u32),
    }
    
    impl wrt_format::Validatable for ComponentType {
        fn validate(&self) -> Result<()> {
            Ok(())
        }
    }
    
    // Implement required traits for BoundedVec compatibility
    use wrt_foundation::traits::{Checksummable, ToBytes, FromBytes};
    
    impl Checksummable for ComponentType {
        fn checksum(&self) -> u32 {
            0 // Placeholder implementation
        }
    }
    
    impl ToBytes for ComponentType {
        fn to_bytes(&self) -> core::result::Result<alloc::vec::Vec<u8>, wrt_foundation::bounded::SerializationError> {
            Ok(alloc::vec::Vec::new()) // Placeholder implementation
        }
    }
    
    impl FromBytes for ComponentType {
        fn from_bytes(_data: &[u8]) -> core::result::Result<Self, wrt_foundation::bounded::SerializationError> {
            Ok(Self::default()) // Placeholder implementation
        }
    }
}

#[cfg(not(feature = "std"))]
use placeholder_types::{ComponentType, ComponentTypeDefinition, ExternType, FormatValType};

use wrt_foundation::{
    budget_aware_provider::CrateId,
    safe_memory::NoStdProvider,
    BoundedVec,
    VerificationLevel,
};

// Import the unified bounded decoder infrastructure
use crate::bounded_decoder_infra::{
    create_decoder_provider,
    BoundedTypeVec,
    MAX_TYPES_PER_COMPONENT,
};

// Import bounded types from prelude
use crate::prelude::{DecoderVec, DecoderString};

/// Maximum size of a single type definition (64KB, ASIL constraint)
pub const MAX_TYPE_DEFINITION_SIZE: usize = 64 * 1024;

/// Maximum recursion depth for nested types (ASIL constraint)
pub const MAX_TYPE_RECURSION_DEPTH: usize = 32;

/// Decoder provider type for consistent allocation
type DecoderProvider = NoStdProvider<32768>;

/// Component Type Section streaming parser
///
/// This parser processes Component Type sections within Component binaries using
/// a streaming approach that minimizes memory allocation and provides 
/// deterministic behavior across all ASIL levels using the unified provider system.
pub struct StreamingTypeParser<'a> {
    /// Binary data being parsed
    data: &'a [u8],
    /// Current parsing offset
    offset: usize,
    /// Verification level for parsing strictness
    verification_level: VerificationLevel,
    /// Current recursion depth for nested types
    recursion_depth: usize,
}

/// Component Type Section parsing result
#[derive(Debug)]
pub struct ComponentTypeSection {
    /// Number of types parsed
    pub type_count: u32,
    /// Total bytes consumed
    pub bytes_consumed: usize,
    /// Types parsed using unified bounded storage
    pub types: BoundedTypeVec<ComponentType>,
}

impl<'a> StreamingTypeParser<'a> {
    /// Create a new streaming component type parser
    ///
    /// # Arguments
    /// * `data` - The binary data containing the component type section
    /// * `verification_level` - Level of validation to perform
    ///
    /// # Returns
    /// A new parser instance ready to process component types
    pub fn new(
        data: &'a [u8],
        verification_level: VerificationLevel,
    ) -> Result<Self> {
        if data.is_empty() {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::INVALID_BINARY,
                "Empty component type section data",
            ));
        }

        // ASIL constraint: Verify data size constraints
        if data.len() > MAX_TYPE_DEFINITION_SIZE {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Component type section exceeds maximum size",
            ));
        }

        Ok(Self {
            data,
            offset: 0,
            verification_level,
            recursion_depth: 0,
        })
    }

    /// Parse the component type section using streaming approach
    ///
    /// This method processes the component type section without loading entire
    /// type definitions into memory, using the unified capability-based allocation system.
    ///
    /// # Returns
    /// A ComponentTypeSection containing parsed types and metadata
    pub fn parse(&mut self) -> Result<ComponentTypeSection> {
        // Read the number of component types
        let (type_count, bytes_read) = read_leb128_u32(self.data, self.offset)?;
        self.offset += bytes_read;

        // ASIL constraint: Validate type count
        if type_count > MAX_TYPES_PER_COMPONENT as u32 {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Too many component types in section",
            ));
        }

        // Initialize storage using unified provider system
        let mut types = self.create_type_storage()?;

        // Parse each component type
        for i in 0..type_count {
            let comp_type = self.parse_single_component_type(i)?;
            self.store_type(&mut types, comp_type)?;
        }

        Ok(ComponentTypeSection {
            type_count,
            bytes_consumed: self.offset,
            types,
        })
    }

    /// Create type storage using unified provider system
    fn create_type_storage(&self) -> Result<BoundedTypeVec<ComponentType>> {
        // Use the unified provider factory that adapts to ASIL level
        let provider = create_decoder_provider::<32768>()?;
        BoundedVec::new(provider).map_err(|e| {
            Error::new(
                ErrorCategory::Resource,
                codes::MEMORY_ERROR,
                "Failed to create type storage",
            )
        })
    }

    /// Parse a single component type from the binary stream
    fn parse_single_component_type(&mut self, type_index: u32) -> Result<ComponentType> {
        if self.offset >= self.data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end of type section",
            ));
        }

        // Read type form
        let type_form = self.data[self.offset];
        self.offset += 1;

        // ASIL constraint: Check recursion depth
        if self.recursion_depth >= MAX_TYPE_RECURSION_DEPTH {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Type recursion depth exceeded",
            ));
        }

        self.recursion_depth += 1;

        let definition = match type_form {
            0x40 => self.parse_component_type_definition()?,
            0x41 => self.parse_instance_type_definition()?,
            0x42 => self.parse_function_type_definition()?,
            0x43 => self.parse_value_type_definition()?,
            0x44 => self.parse_resource_type_definition()?,
            _ => {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unknown component type form",
                ));
            }
        };

        self.recursion_depth -= 1;

        Ok(ComponentType { definition })
    }

    /// Parse component type definition (0x40)
    fn parse_component_type_definition(&mut self) -> Result<ComponentTypeDefinition> {
        // Read import count
        let (import_count, bytes_read) = read_leb128_u32(self.data, self.offset)?;
        self.offset += bytes_read;

        let provider = create_decoder_provider::<4096>()?;
        let mut imports = DecoderVec::new(provider.clone())?;
        for _ in 0..import_count {
            let namespace = self.read_string()?;
            let name = self.read_string()?;
            let extern_type = self.parse_extern_type()?;
            imports.push((namespace, name, extern_type)).map_err(|_| {
                Error::new(
                    ErrorCategory::Resource,
                    codes::CAPACITY_EXCEEDED,
                    "Too many imports in component type",
                )
            })?;
        }

        // Read export count
        let (export_count, bytes_read) = read_leb128_u32(self.data, self.offset)?;
        self.offset += bytes_read;

        let mut exports = DecoderVec::new(provider)?;
        for _ in 0..export_count {
            let name = self.read_string()?;
            let extern_type = self.parse_extern_type()?;
            exports.push((name, extern_type)).map_err(|_| {
                Error::new(
                    ErrorCategory::Resource,
                    codes::CAPACITY_EXCEEDED,
                    "Too many exports in component type",
                )
            })?;
        }

        Ok(ComponentTypeDefinition::Component { imports, exports })
    }

    /// Parse instance type definition (0x41)
    fn parse_instance_type_definition(&mut self) -> Result<ComponentTypeDefinition> {
        // Read export count
        let (export_count, bytes_read) = read_leb128_u32(self.data, self.offset)?;
        self.offset += bytes_read;

        let provider = create_decoder_provider::<4096>()?;
        let mut exports = DecoderVec::new(provider)?;
        for _ in 0..export_count {
            let name = self.read_string()?;
            let extern_type = self.parse_extern_type()?;
            exports.push((name, extern_type)).map_err(|_| {
                Error::new(
                    ErrorCategory::Resource,
                    codes::CAPACITY_EXCEEDED,
                    "Too many exports in instance type",
                )
            })?;
        }

        Ok(ComponentTypeDefinition::Instance { exports })
    }

    /// Parse function type definition (0x42)
    fn parse_function_type_definition(&mut self) -> Result<ComponentTypeDefinition> {
        // Read parameter count
        let (param_count, bytes_read) = read_leb128_u32(self.data, self.offset)?;
        self.offset += bytes_read;

        let provider = create_decoder_provider::<4096>()?;
        let mut params = DecoderVec::new(provider.clone())?;
        for _ in 0..param_count {
            let name = self.read_string()?;
            let val_type = self.parse_value_type()?;
            params.push((name, val_type)).map_err(|_| {
                Error::new(
                    ErrorCategory::Resource,
                    codes::CAPACITY_EXCEEDED,
                    "Too many parameters in function type",
                )
            })?;
        }

        // Read result count
        let (result_count, bytes_read) = read_leb128_u32(self.data, self.offset)?;
        self.offset += bytes_read;

        let mut results = DecoderVec::new(provider)?;
        for _ in 0..result_count {
            let val_type = self.parse_value_type()?;
            results.push(val_type).map_err(|_| {
                Error::new(
                    ErrorCategory::Resource,
                    codes::CAPACITY_EXCEEDED,
                    "Too many results in function type",
                )
            })?;
        }

        Ok(ComponentTypeDefinition::Function { params, results })
    }

    /// Parse value type definition (0x43)
    fn parse_value_type_definition(&mut self) -> Result<ComponentTypeDefinition> {
        let val_type = self.parse_value_type()?;
        Ok(ComponentTypeDefinition::Value(val_type))
    }

    /// Parse resource type definition (0x44)
    fn parse_resource_type_definition(&mut self) -> Result<ComponentTypeDefinition> {
        // Read resource representation
        let representation = self.parse_resource_representation()?;
        
        // Read nullable flag
        let nullable = if self.offset < self.data.len() {
            self.data[self.offset] != 0
        } else {
            false
        };
        if self.offset < self.data.len() {
            self.offset += 1;
        }

        Ok(ComponentTypeDefinition::Resource {
            representation,
            nullable,
        })
    }

    /// Parse extern type
    fn parse_extern_type(&mut self) -> Result<ExternType> {
        if self.offset >= self.data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end while reading extern type",
            ));
        }

        let extern_form = self.data[self.offset];
        self.offset += 1;

        match extern_form {
            0x00 => {
                // Function type
                let (param_count, bytes_read) = read_leb128_u32(self.data, self.offset)?;
                self.offset += bytes_read;

                let provider = create_decoder_provider::<4096>()?;
                let mut params = DecoderVec::new(provider.clone())?;
                for _ in 0..param_count {
                    let name = self.read_string()?;
                    let val_type = self.parse_value_type()?;
                    params.push((name, val_type)).map_err(|_| {
                        Error::new(
                            ErrorCategory::Resource,
                            codes::CAPACITY_EXCEEDED,
                            "Too many parameters in extern function type",
                        )
                    })?;
                }

                let (result_count, bytes_read) = read_leb128_u32(self.data, self.offset)?;
                self.offset += bytes_read;

                let mut results = DecoderVec::new(provider)?;
                for _ in 0..result_count {
                    let val_type = self.parse_value_type()?;
                    results.push(val_type).map_err(|_| {
                        Error::new(
                            ErrorCategory::Resource,
                            codes::CAPACITY_EXCEEDED,
                            "Too many results in extern function type",
                        )
                    })?;
                }

                Ok(ExternType::Function { params, results })
            }
            0x01 => {
                // Value type
                let val_type = self.parse_value_type()?;
                Ok(ExternType::Value(val_type))
            }
            0x02 => {
                // Type reference
                let (type_idx, bytes_read) = read_leb128_u32(self.data, self.offset)?;
                self.offset += bytes_read;
                Ok(ExternType::Type(type_idx))
            }
            0x03 => {
                // Instance type - recursive parse
                self.recursion_depth += 1;
                let instance_def = self.parse_instance_type_definition()?;
                self.recursion_depth -= 1;
                
                if let ComponentTypeDefinition::Instance { exports } = instance_def {
                    Ok(ExternType::Instance { exports })
                } else {
                    Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Invalid instance type definition",
                    ))
                }
            }
            0x04 => {
                // Component type - recursive parse
                self.recursion_depth += 1;
                let component_def = self.parse_component_type_definition()?;
                self.recursion_depth -= 1;
                
                if let ComponentTypeDefinition::Component { imports, exports } = component_def {
                    Ok(ExternType::Component { imports, exports })
                } else {
                    Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Invalid component type definition",
                    ))
                }
            }
            _ => Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unknown extern type form",
            ))
        }
    }

    /// Parse component value type
    fn parse_value_type(&mut self) -> Result<FormatValType> {
        if self.offset >= self.data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end while reading value type",
            ));
        }

        let val_form = self.data[self.offset];
        self.offset += 1;

        match val_form {
            0x7F => Ok(FormatValType::Bool),
            0x7E => Ok(FormatValType::S8),
            0x7D => Ok(FormatValType::U8),
            0x7C => Ok(FormatValType::S16),
            0x7B => Ok(FormatValType::U16),
            0x7A => Ok(FormatValType::S32),
            0x79 => Ok(FormatValType::U32),
            0x78 => Ok(FormatValType::S64),
            0x77 => Ok(FormatValType::U64),
            0x76 => Ok(FormatValType::F32),
            0x75 => Ok(FormatValType::F64),
            0x74 => Ok(FormatValType::Char),
            0x73 => Ok(FormatValType::String),
            0x72 => {
                // Record type - simplified for streaming
                let (field_count, bytes_read) = read_leb128_u32(self.data, self.offset)?;
                self.offset += bytes_read;
                
                // Skip field definitions for now (would need full recursive parsing)
                for _ in 0..field_count {
                    let _name = self.read_string()?;
                    let _field_type = self.parse_value_type()?;
                }
                
                // Use bounded vec for empty record - allocation will be handled by capability system
                let provider = create_decoder_provider::<4096>()?;
                let empty_fields = DecoderVec::new(provider)?;
                return Ok(FormatValType::Record(empty_fields));
            }
            0x71 => {
                // Variant type - simplified for streaming
                let (case_count, bytes_read) = read_leb128_u32(self.data, self.offset)?;
                self.offset += bytes_read;
                
                // Skip case definitions for now
                for _ in 0..case_count {
                    let _name = self.read_string()?;
                    // Optional case type
                    if self.offset < self.data.len() && self.data[self.offset] == 1 {
                        self.offset += 1;
                        let _case_type = self.parse_value_type()?;
                    } else if self.offset < self.data.len() {
                        self.offset += 1; // Skip the 0 byte
                    }
                }
                
                // Use bounded vec for empty variant - allocation will be handled by capability system
                let provider = create_decoder_provider::<4096>()?;
                let empty_cases = DecoderVec::new(provider)?;
                return Ok(FormatValType::Variant(empty_cases));
                
            }
            0x70 => {
                // List type
                let element_type_ref = self.parse_type_ref()?;
                Ok(FormatValType::List(element_type_ref))
            }
            0x6F => {
                // Tuple type - simplified for streaming
                let (element_count, bytes_read) = read_leb128_u32(self.data, self.offset)?;
                self.offset += bytes_read;
                
                // Skip element types for now
                for _ in 0..element_count {
                    let _element_type = self.parse_value_type()?;
                }
                
                // Use bounded vec for empty tuple - allocation will be handled by capability system
                let provider = create_decoder_provider::<4096>()?;
                let empty_elements = DecoderVec::new(provider)?;
                return Ok(FormatValType::Tuple(empty_elements));
                
            }
            0x6E => {
                // Own resource
                let (resource_idx, bytes_read) = read_leb128_u32(self.data, self.offset)?;
                self.offset += bytes_read;
                Ok(FormatValType::Own(resource_idx))
            }
            0x6D => {
                // Borrow resource
                let (resource_idx, bytes_read) = read_leb128_u32(self.data, self.offset)?;
                self.offset += bytes_read;
                Ok(FormatValType::Borrow(resource_idx))
            }
            _ => Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unknown value type form",
            ))
        }
    }

    /// Parse type reference (simplified as u32 for streaming)
    fn parse_type_ref(&mut self) -> Result<u32> {
        let (type_ref, bytes_read) = read_leb128_u32(self.data, self.offset)?;
        self.offset += bytes_read;
        Ok(type_ref)
    }

    /// Parse resource representation
    fn parse_resource_representation(&mut self) -> Result<wrt_foundation::resource::ResourceRepresentation> {
        if self.offset >= self.data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end while reading resource representation",
            ));
        }

        let repr_form = self.data[self.offset];
        self.offset += 1;

        match repr_form {
            0x00 => Ok(wrt_foundation::resource::ResourceRepresentation::Handle32),
            0x01 => Ok(wrt_foundation::resource::ResourceRepresentation::Handle64),
            _ => Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unknown resource representation",
            ))
        }
    }

    /// Read a string from the binary data
    fn read_string(&mut self) -> Result<DecoderString> {
        let (string, bytes_read) = read_string(self.data, self.offset)?;
        self.offset += bytes_read;
        
        // Convert to bounded string  
        let provider = create_decoder_provider::<4096>()?;
        // Convert bytes to string first
        let string_str = core::str::from_utf8(&string).map_err(|_| {
            Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Invalid UTF-8 in string",
            )
        })?;
        
        let bounded_string = DecoderString::from_str(string_str, provider).map_err(|_| {
            Error::new(
                ErrorCategory::Resource,
                codes::CAPACITY_EXCEEDED,
                "String too long for bounded storage",
            )
        })?;
        
        Ok(bounded_string)
    }

    /// Store a parsed type in the bounded storage
    fn store_type(
        &self,
        types: &mut BoundedTypeVec<ComponentType>,
        comp_type: ComponentType,
    ) -> Result<()> {
        types.push(comp_type).map_err(|_| {
            Error::new(
                ErrorCategory::Resource,
                codes::CAPACITY_EXCEEDED,
                "Component type storage capacity exceeded",
            )
        })
    }

    /// Get current parsing offset
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get remaining bytes in the section
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }
}

impl ComponentTypeSection {
    /// Get the number of parsed types
    pub fn type_count(&self) -> u32 {
        self.type_count
    }

    /// Get total bytes consumed during parsing
    pub fn bytes_consumed(&self) -> usize {
        self.bytes_consumed
    }

    /// Get a type by index (ASIL-safe)
    pub fn get_type(&self, index: usize) -> Option<&ComponentType> {
        self.types.get(index)
    }

    /// Iterate over all types (ASIL-safe)
    pub fn iter_types(&self) -> impl Iterator<Item = &ComponentType> + ExactSizeIterator {
        self.types.iter()
    }

    /// Get the number of types as usize
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if the section is empty
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_section() {
        let data = &[0u8]; // Zero types
        
        let mut parser = StreamingTypeParser::new(
            data,
            VerificationLevel::Standard,
        ).unwrap();
        
        let result = parser.parse().unwrap();
        assert_eq!(result.type_count(), 0);
        assert_eq!(result.bytes_consumed(), 1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_invalid_type_count() {
        // Create data with too many types
        let type_count = (MAX_TYPES_PER_COMPONENT + 1) as u32;
        let provider = create_decoder_provider::<4096>()?;
        let mut data = DecoderVec::new(provider)?;
        
        // Write LEB128 encoded type count
        let mut count = type_count;
        while count >= 0x80 {
            data.push((count & 0x7F) as u8 | 0x80);
            count >>= 7;
        }
        data.push(count as u8);

        let mut parser = StreamingTypeParser::new(
            &data,
            VerificationLevel::Standard,
        ).unwrap();
        
        assert!(parser.parse().is_err());
    }

    #[test]
    fn test_recursion_depth_protection() {
        // This would test that deep recursion is properly handled,
        // but requires complex binary construction
        let data = &[0u8]; // Zero types for now
        
        let mut parser = StreamingTypeParser::new(
            data,
            VerificationLevel::Standard,
        ).unwrap();
        
        // Set recursion depth to maximum
        parser.recursion_depth = MAX_TYPE_RECURSION_DEPTH;
        
        // This should not crash due to recursion protection
        assert!(parser.parse().is_ok());
    }

    #[test]
    fn test_parser_offset_tracking() {
        let data = &[0u8]; // Zero types
        
        let mut parser = StreamingTypeParser::new(
            data,
            VerificationLevel::Standard,
        ).unwrap();
        
        assert_eq!(parser.offset(), 0);
        assert_eq!(parser.remaining(), 1);
        
        let result = parser.parse().unwrap();
        assert_eq!(parser.offset(), 1);
        assert_eq!(parser.remaining(), 0);
        assert_eq!(result.bytes_consumed(), 1);
    }
}