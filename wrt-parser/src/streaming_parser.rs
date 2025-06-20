//! Streaming WebAssembly parser with minimal memory usage
//!
//! This module provides the core streaming parser that processes WebAssembly
//! binaries section by section without loading the entire binary into memory.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::{binary_constants, leb128, ParserProvider};
use crate::module_builder::Module;
use crate::section_parser::SectionParser;
use crate::validation::ValidationConfig;

/// Result of parsing operation
#[derive(Debug)]
pub enum ParseResult<P: wrt_foundation::safe_memory::MemoryProvider> {
    /// Parsing completed successfully
    Complete(Module<P>),
    /// Need more data to continue parsing
    NeedMoreData,
    /// Parsing failed with error
    Error(Error),
}

/// Streaming WebAssembly parser
pub struct StreamingParser {
    /// Current offset in the binary
    offset: usize,
    /// The module being built
    module: Module<ParserProvider>,
    /// Section parser for handling individual sections
    section_parser: SectionParser,
    /// Validation configuration
    validation_config: ValidationConfig,
    /// Whether header has been parsed
    header_parsed: bool,
}

impl StreamingParser {
    /// Create a new streaming parser
    pub fn new() -> Result<Self> {
        let provider = ParserProvider::default();
        Ok(StreamingParser {
            offset: 0,
            module: Module::new(provider)?,
            section_parser: SectionParser::new()?,
            validation_config: ValidationConfig::default(),
            header_parsed: false,
        })
    }
    
    /// Parse a complete WebAssembly binary
    pub fn parse(&mut self, binary: &[u8]) -> Result<Module<ParserProvider>> {
        // Validate and parse header
        if !self.header_parsed {
            self.parse_header(binary)?;
            self.header_parsed = true;
        }
        
        // Process all sections
        while self.offset < binary.len() {
            self.process_next_section(binary)?;
        }
        
        // Validate the completed module
        self.validate_module()?;
        
        // Return the completed module
        let provider = ParserProvider::default();
        let mut completed_module = Module::new(provider)?;
        core::mem::swap(&mut completed_module, &mut self.module);
        Ok(completed_module)
    }
    
    /// Parse the WebAssembly header
    fn parse_header(&mut self, binary: &[u8]) -> Result<()> {
        if binary.len() < 8 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Binary too small for WebAssembly header"
            ));
        }
        
        // Check magic number
        if &binary[0..4] != &binary_constants::WASM_MAGIC {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Invalid WebAssembly magic number"
            ));
        }
        
        // Check version
        if &binary[4..8] != &binary_constants::WASM_VERSION {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unsupported WebAssembly version"
            ));
        }
        
        self.offset = 8;
        Ok(())
    }
    
    /// Process the next section in the stream
    fn process_next_section(&mut self, binary: &[u8]) -> Result<()> {
        if self.offset >= binary.len() {
            return Ok(()); // No more sections
        }
        
        // Read section ID
        if self.offset >= binary.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end of data while reading section ID"
            ));
        }
        
        let section_id = binary[self.offset];
        self.offset += 1;
        
        // Read section size
        let (section_size, bytes_read) = leb128::read_leb128_u32(binary, self.offset)?;
        self.offset += bytes_read;
        
        let section_end = self.offset + section_size as usize;
        if section_end > binary.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Section extends beyond binary"
            ));
        }
        
        // Extract section data without copying
        let section_data = &binary[self.offset..section_end];
        
        // Process the section
        self.section_parser.parse_section(section_id, section_data, &mut self.module)?;
        
        self.offset = section_end;
        Ok(())
    }
    
    /// Validate the completed module
    fn validate_module(&self) -> Result<()> {
        // Basic validation - can be extended based on validation_config
        
        // Check that function count matches code count
        if self.module.functions.len() != self.count_code_sections() {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Function count does not match code count"
            ));
        }
        
        Ok(())
    }
    
    /// Count the number of functions with code
    fn count_code_sections(&self) -> usize {
        self.module.functions.iter()
            .filter(|f| !f.code.is_empty())
            .count()
    }
    
    /// Get the current parsing offset
    pub fn offset(&self) -> usize {
        self.offset
    }
    
    /// Check if parsing is complete
    pub fn is_complete(&self, binary_len: usize) -> bool {
        self.offset >= binary_len && self.header_parsed
    }
}

/// Component streaming parser for WebAssembly Component Model
pub struct ComponentStreamingParser {
    /// Current offset in the binary
    offset: usize,
    /// Validation configuration
    validation_config: ValidationConfig,
    /// Whether header has been parsed
    header_parsed: bool,
}

impl ComponentStreamingParser {
    /// Create a new component streaming parser
    pub fn new() -> Result<Self> {
        Ok(ComponentStreamingParser {
            offset: 0,
            validation_config: ValidationConfig::default(),
            header_parsed: false,
        })
    }
    
    /// Parse a complete WebAssembly Component binary
    pub fn parse(&mut self, binary: &[u8]) -> Result<crate::component_parser::Component<ParserProvider>> {
        // Validate and parse header
        if !self.header_parsed {
            self.parse_component_header(binary)?;
            self.header_parsed = true;
        }
        
        // Create and return a placeholder component for now
        crate::component_parser::Component::new(ParserProvider::default())
    }
    
    /// Parse the WebAssembly Component header
    fn parse_component_header(&mut self, binary: &[u8]) -> Result<()> {
        if binary.len() < 8 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Binary too small for WebAssembly Component header"
            ));
        }
        
        // Check magic number (same as regular WASM)
        if &binary[0..4] != &binary_constants::COMPONENT_MAGIC {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Invalid WebAssembly Component magic number"
            ));
        }
        
        // Check component version
        if &binary[4..8] != &binary_constants::COMPONENT_VERSION {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unsupported WebAssembly Component version"
            ));
        }
        
        self.offset = 8;
        Ok(())
    }
}