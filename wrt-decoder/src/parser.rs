//! Streaming parser for WebAssembly modules and components
//!
//! This module provides a streaming parser interface for WebAssembly modules and components,
//! allowing for efficient incremental processing without requiring the entire binary
//! to be parsed at once.

use crate::module::Module;
use crate::prelude::*;
use crate::section_error;
use crate::utils::{self, BinaryType};
use wrt_error::{codes, Error, ErrorCategory, Result};
use wrt_format::section::CustomSection;
use wrt_types::safe_memory::SafeSlice;

// Comment out conflicting imports
/*
use crate::module::{
    parse_type_section,
    parse_import_section,
    parse_function_section,
    parse_table_section,
    parse_memory_section,
    parse_global_section,
    parse_export_section,
    parse_element_section,
    parse_code_section,
    parse_data_section
};
*/

// Section ID constants
pub const CUSTOM_ID: u8 = 0;
pub const TYPE_ID: u8 = 1;
pub const IMPORT_ID: u8 = 2;
pub const FUNCTION_ID: u8 = 3;
pub const TABLE_ID: u8 = 4;
pub const MEMORY_ID: u8 = 5;
pub const GLOBAL_ID: u8 = 6;
pub const EXPORT_ID: u8 = 7;
pub const START_ID: u8 = 8;
pub const ELEMENT_ID: u8 = 9;
pub const CODE_ID: u8 = 10;
pub const DATA_ID: u8 = 11;

/// Represents a payload produced by the WebAssembly parser
#[derive(Debug)]
pub enum Payload<'a> {
    /// WebAssembly version
    Version(u32, &'a [u8]),

    /// Type section
    TypeSection(SafeSlice<'a>, usize),

    /// Import section
    ImportSection(SafeSlice<'a>, usize),

    /// Function section
    FunctionSection(SafeSlice<'a>, usize),

    /// Table section
    TableSection(SafeSlice<'a>, usize),

    /// Memory section
    MemorySection(SafeSlice<'a>, usize),

    /// Global section
    GlobalSection(SafeSlice<'a>, usize),

    /// Export section
    ExportSection(SafeSlice<'a>, usize),

    /// Start section
    StartSection(u32),

    /// Element section
    ElementSection(SafeSlice<'a>, usize),

    /// Code section
    CodeSection(SafeSlice<'a>, usize),

    /// Data section
    DataSection(SafeSlice<'a>, usize),

    /// Data count section (for bulk memory operations)
    DataCountSection {
        /// Number of data segments
        count: u32,
    },

    /// Custom section
    CustomSection {
        /// Name of the custom section
        name: String,
        /// Data of the custom section
        data: SafeSlice<'a>,
        /// Size of the data
        size: usize,
    },

    /// Component section (for component model)
    ComponentSection {
        /// Component data
        data: SafeSlice<'a>,
        /// Size of the data
        size: usize,
    },

    /// End of module
    End,
}

/// WebAssembly binary parser
pub struct Parser<'a> {
    /// Current offset in the binary
    current_offset: usize,
    /// Binary data to parse (raw byte slice for better no_std compatibility)
    binary: Option<&'a [u8]>,
    /// Whether to skip unknown custom sections
    skip_unknown_custom: bool,
    /// Whether the version has been read
    version_read: bool,
    /// Whether the parser has finished processing
    finished: bool,
    /// Type of binary (core module or component)
    binary_type: Option<BinaryType>,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given binary data
    pub fn new(binary: impl Into<Option<&'a [u8]>>, skip_unknown_custom: bool) -> Self {
        // Convert into Option
        let binary = binary.into();

        // Determine binary type if binary is provided
        let binary_type = binary.and_then(|data| utils::detect_binary_type(data).ok());

        Self {
            current_offset: 0,
            binary,
            skip_unknown_custom,
            version_read: false,
            finished: false,
            binary_type,
        }
    }

    /// Convenient constructor that takes a slice directly (for backward compatibility)
    #[deprecated(since = "0.2.0", note = "Use Parser::new(Some(binary), false) instead")]
    pub fn with_binary(binary: &'a [u8]) -> Self {
        Self::new(Some(binary), false)
    }

    /// INTERNAL USE ONLY: For compatibility with tests that use the old API
    /// This is not part of the public API and will be removed
    /// Do not use this method in new code!
    pub fn _new_compat(binary: &'a [u8]) -> Self {
        Self::new(Some(binary), false)
    }

    /// Get the current offset in the binary
    pub fn current_offset(&self) -> usize {
        self.current_offset
    }

    /// Get the detected binary type
    pub fn binary_type(&self) -> Option<BinaryType> {
        self.binary_type
    }

    /// Create a new parser from a SafeSlice
    pub fn from_safe_slice(slice: SafeSlice<'a>) -> Self {
        // Convert SafeSlice to &[u8] for parsing
        // Use data() to access the underlying bytes, handling error gracefully
        let binary = slice.data().ok();
        Self::new(binary, false)
    }

    /// Read the next payload from the binary
    pub fn read(&mut self) -> Result<Option<Payload<'a>>> {
        match self.next() {
            Some(Ok(payload)) => Ok(Some(payload)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    /// Process the WebAssembly header
    fn process_header(&mut self) -> Result<Payload<'a>> {
        // Get the underlying data safely
        let data = match self.binary {
            Some(binary) => binary,
            None => return Err(section_error::binary_required(0)),
        };

        // Check if binary has at least 8 bytes (magic + version)
        if data.len() < 8 {
            return Err(section_error::unexpected_end(0, 8, data.len()));
        }

        // Check based on binary type
        match self.binary_type {
            Some(BinaryType::CoreModule) => {
                // Core WebAssembly module
                utils::verify_binary_header(data)?;
                self.current_offset = 8;
                self.version_read = true;
                Ok(Payload::Version(1, data))
            }
            Some(BinaryType::Component) => {
                // Component Model component
                // Verify component header (similarly to module header)
                if data[0..4] != [0x00, 0x63, 0x6D, 0x70] {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Invalid Component Model magic number",
                    ));
                }

                if data[4..8] != [0x01, 0x00, 0x00, 0x00] {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Unsupported Component version",
                    ));
                }

                self.current_offset = 8;
                self.version_read = true;
                Ok(Payload::Version(1, data))
            }
            None => {
                // Try to detect binary type
                self.binary_type = Some(utils::detect_binary_type(data)?);
                self.process_header()
            }
        }
    }

    /// Process a section (delegate to the appropriate parser)
    fn process_section(&mut self, section_id: u8, section_size: usize) -> Result<Payload<'a>> {
        // Get the binary data safely
        let binary_data = match self.binary {
            Some(binary) => binary,
            None => return Err(section_error::binary_required(0)),
        };

        // Store section data for processing
        let data = &binary_data[self.current_offset..self.current_offset + section_size];
        let start_offset = self.current_offset;

        // Always advance the offset past this section to prevent infinite loops
        self.current_offset += section_size;

        // Delegate based on binary type
        match self.binary_type {
            Some(BinaryType::CoreModule) => {
                // Process section based on ID for core modules
                self.process_core_section(section_id, data, section_size, start_offset)
            }
            Some(BinaryType::Component) => {
                // Process section based on ID for components
                self.process_component_section(section_id, data, section_size, start_offset)
            }
            None => {
                // We shouldn't get here, but just in case...
                Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Binary type not detected",
                ))
            }
        }
    }

    /// Process a core module section
    fn process_core_section(
        &mut self,
        section_id: u8,
        data: &'a [u8],
        section_size: usize,
        start_offset: usize,
    ) -> Result<Payload<'a>> {
        match section_id {
            0x00 => {
                // Custom section - parse and return
                let mut module = Module::new();
                // Updated to use module methods directly for now, as decoder_core::parse is being reorganized
                let (name, bytes_read) = crate::utils::read_name_as_string(data, 0)?;
                module.custom_sections.push(CustomSection {
                    name,
                    data: data[bytes_read..].to_vec(),
                });

                // Extract the name and data from the parsed section
                if let Some(custom_section) = module.custom_sections.first() {
                    Ok(Payload::CustomSection {
                        name: custom_section.name.clone(),
                        data: SafeSlice::new(data),
                        size: section_size,
                    })
                } else {
                    Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Failed to parse custom section",
                    ))
                }
            }
            0x01 => Ok(Payload::TypeSection(SafeSlice::new(data), section_size)),
            0x02 => Ok(Payload::ImportSection(SafeSlice::new(data), section_size)),
            0x03 => Ok(Payload::FunctionSection(SafeSlice::new(data), section_size)),
            0x04 => Ok(Payload::TableSection(SafeSlice::new(data), section_size)),
            0x05 => Ok(Payload::MemorySection(SafeSlice::new(data), section_size)),
            0x06 => Ok(Payload::GlobalSection(SafeSlice::new(data), section_size)),
            0x07 => Ok(Payload::ExportSection(SafeSlice::new(data), section_size)),
            0x08 => {
                // Start section - parse directly
                if section_size == 0 {
                    return Err(section_error::invalid_section(
                        section_id,
                        start_offset,
                        "Start section cannot be empty",
                    ));
                }

                let (start_index, _) = wrt_format::binary::read_leb128_u32(data, 0)?;
                Ok(Payload::StartSection(start_index))
            }
            0x09 => Ok(Payload::ElementSection(SafeSlice::new(data), section_size)),
            0x0A => Ok(Payload::CodeSection(SafeSlice::new(data), section_size)),
            0x0B => Ok(Payload::DataSection(SafeSlice::new(data), section_size)),
            0x0C => {
                // Data count section
                if section_size == 0 {
                    return Err(section_error::invalid_section(
                        section_id,
                        start_offset,
                        "Data count section cannot be empty",
                    ));
                }

                let (count, _) = wrt_format::binary::read_leb128_u32(data, 0)?;
                Ok(Payload::DataCountSection { count })
            }
            _ => {
                // Unknown section
                if self.skip_unknown_custom {
                    self.next().ok_or_else(|| {
                        Error::new(
                            ErrorCategory::Parse,
                            codes::PARSE_ERROR,
                            "No more sections to parse",
                        )
                    })?
                } else {
                    Ok(Payload::CustomSection {
                        name: format!("unknown_{}", section_id),
                        data: SafeSlice::new(data),
                        size: section_size,
                    })
                }
            }
        }
    }

    /// Process a component section
    fn process_component_section(
        &mut self,
        section_id: u8,
        data: &'a [u8],
        section_size: usize,
        _start_offset: usize,
    ) -> Result<Payload<'a>> {
        // For component model parsing, we'll delegate to the component parser
        // but wrap the result in our Payload type
        match section_id {
            0x00 => {
                // Custom section - use similar code as for core modules
                let (name, bytes_read) = utils::read_name_as_string(data, 0)?;
                let section_data = &data[bytes_read..];

                Ok(Payload::CustomSection {
                    name,
                    data: SafeSlice::new(section_data),
                    size: section_size - bytes_read,
                })
            }
            _ => {
                // For all other component sections, package them as ComponentSection
                // The component parser will handle them later
                Ok(Payload::ComponentSection {
                    data: SafeSlice::new(data),
                    size: section_size,
                })
            }
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Payload<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        // If we've finished, return None
        if self.finished {
            return None;
        }

        // If we haven't processed the header yet, start with that
        if !self.version_read {
            return Some(self.process_header());
        }

        // Check if we've reached the end of the binary
        if self.current_offset >= self.binary.as_ref().map_or(0, |v| v.len()) {
            self.finished = true;
            return Some(Ok(Payload::End));
        }

        // Ensure we have at least 1 byte left (section ID)
        if self.current_offset + 1 > self.binary.as_ref().map_or(0, |v| v.len()) {
            self.finished = true;
            return Some(Err(section_error::unexpected_end(
                self.current_offset,
                1,
                0,
            )));
        }

        // Read the section ID
        let section_id = self.binary.as_ref().unwrap()[self.current_offset];
        self.current_offset += 1;

        // Read section size
        if self.current_offset >= self.binary.as_ref().map_or(0, |v| v.len()) {
            self.finished = true;
            return Some(Err(section_error::unexpected_end(
                self.current_offset,
                1,
                0,
            )));
        }

        // Use read_leb128_u32 for the section size
        let section_size_result =
            wrt_format::binary::read_leb128_u32(self.binary.as_ref().unwrap(), self.current_offset);

        let (section_size, size_len) = match section_size_result {
            Ok(result) => result,
            Err(e) => {
                self.finished = true;
                return Some(Err(e));
            }
        };

        self.current_offset += size_len;

        // Ensure the section fits in the binary
        if self.current_offset + section_size as usize > self.binary.as_ref().map_or(0, |v| v.len())
        {
            self.finished = true;
            return Some(Err(section_error::section_too_large(
                section_id,
                section_size,
                self.current_offset,
            )));
        }

        // Process the section based on its ID and the binary type
        // Handle any ? operator errors properly in this context
        match self.process_section(section_id, section_size as usize) {
            Ok(payload) => Some(Ok(payload)),
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

/// Parse a module using the streaming parser
///
/// This function takes a binary and parses it into a Module structure,
/// using the appropriate parser based on the binary format.
///
/// # Arguments
///
/// * `binary` - The WebAssembly binary data
///
/// # Returns
///
/// * `Result<Module>` - The parsed module or an error
pub fn parse_module(binary: &[u8]) -> Result<Module> {
    // Detect binary type
    match utils::detect_binary_type(binary)? {
        BinaryType::CoreModule => {
            // Use the core module parser
            // Use module's own decode method instead
            crate::module::decode_module_with_binary(binary)
        }
        BinaryType::Component => {
            // Return an error - this function is specifically for core modules
            Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Cannot parse a Component Model binary as a core module",
            ))
        }
    }
}

/// Parse a component using the streaming parser
///
/// # Arguments
///
/// * `binary` - The WebAssembly Component Model binary data
///
/// # Returns
///
/// * `Result<Component>` - The parsed component or an error
pub fn parse_component(binary: &[u8]) -> Result<wrt_format::component::Component> {
    // Detect binary type
    match utils::detect_binary_type(binary)? {
        BinaryType::CoreModule => {
            // Return an error - this function is specifically for components
            Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Cannot parse a core module as a Component Model component",
            ))
        }
        BinaryType::Component => {
            // Use the component parser
            crate::component::decode_component(binary)
        }
    }
}
