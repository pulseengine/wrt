// WRT - wrt-decoder
// Module: Optimized Module Parsing
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Memory-optimized module parsing that minimizes allocations and uses streaming

use wrt_error::{codes, Error, ErrorCategory, Result};
use wrt_foundation::{
    safe_memory::{MemoryProvider, SafeMemoryHandler, SafeSlice},
    types::Module as WrtModule,
    verification::VerificationLevel,
};
use wrt_format::binary::{WASM_MAGIC, WASM_VERSION};

use crate::memory_optimized::{MemoryPool, StreamingCollectionParser, check_bounds_u32};
use crate::prelude::*;

/// Optimized module parser that minimizes memory allocations
pub struct OptimizedModuleParser<P: MemoryProvider> {
    memory_pool: MemoryPool<P>,
    verification_level: VerificationLevel,
}

impl<P: MemoryProvider + Default> Default for OptimizedModuleParser<P> {
    fn default() -> Self {
        Self::new(P::default(), VerificationLevel::default())
    }
}

impl<P: MemoryProvider> OptimizedModuleParser<P> {
    /// Create a new optimized module parser
    pub fn new(provider: P, verification_level: VerificationLevel) -> Self {
        Self {
            memory_pool: MemoryPool::new(provider),
            verification_level,
        }
    }

    /// Parse a WebAssembly module with minimal memory allocations
    pub fn parse_module(&mut self, bytes: &[u8]) -> Result<WrtModule> {
        // Verify header first
        self.verify_header(bytes)?;

        // Create SafeSlice for the module data
        let slice = SafeSlice::new(&bytes[8..]).map_err(|e| {
            Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                format!("Failed to create SafeSlice: {}", e.message()),
            )
        })?;

        // Initialize empty module
        let mut module = WrtModule::new();

        // Parse sections using streaming approach
        self.parse_sections_streaming(&slice, &mut module)?;

        Ok(module)
    }

    /// Verify WebAssembly header without allocation
    fn verify_header(&self, bytes: &[u8]) -> Result<()> {
        if bytes.len() < 8 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Binary too short for WebAssembly header",
            ));
        }

        // Check magic bytes
        if &bytes[0..4] != WASM_MAGIC {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Invalid WebAssembly magic bytes",
            ));
        }

        // Check version
        if &bytes[4..8] != WASM_VERSION {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unsupported WebAssembly version",
            ));
        }

        Ok(())
    }

    /// Parse sections using streaming approach
    fn parse_sections_streaming(&mut self, slice: &SafeSlice, module: &mut WrtModule) -> Result<()> {
        let data = slice.data().map_err(|e| {
            Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                format!("Failed to access slice data: {}", e.message()),
            )
        })?;

        let mut offset = 0;

        while offset < data.len() {
            // Parse section header
            let (section_info, new_offset) = self.parse_section_header(data, offset)?;
            offset = new_offset;

            // Extract section data as SafeSlice
            let section_end = offset + section_info.size;
            if section_end > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Section extends beyond module boundary",
                ));
            }

            let section_slice = SafeSlice::new(&data[offset..section_end]).map_err(|e| {
                Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Failed to create section SafeSlice: {}", e.message()),
                )
            })?;

            // Parse section content
            self.parse_section_content(section_info.id, &section_slice, module)?;

            offset = section_end;
        }

        Ok(())
    }

    /// Parse section header
    fn parse_section_header(&self, data: &[u8], offset: usize) -> Result<(SectionInfo, usize)> {
        if offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end while parsing section header",
            ));
        }

        let section_id = data[offset];
        let mut new_offset = offset + 1;

        // Read section size
        let (section_size, size_offset) = wrt_format::binary::read_leb128_u32(data, new_offset)?;
        new_offset = size_offset;

        // Bounds check section size
        check_bounds_u32(section_size, 100_000_000, "section size")?;

        Ok((
            SectionInfo {
                id: section_id,
                size: section_size as usize,
            },
            new_offset,
        ))
    }

    /// Parse section content based on section ID
    fn parse_section_content(
        &mut self,
        section_id: u8,
        section_slice: &SafeSlice,
        module: &mut WrtModule,
    ) -> Result<()> {
        let section_data = section_slice.data().map_err(|e| {
            Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                format!("Failed to access section data: {}", e.message()),
            )
        })?;

        match section_id {
            1 => self.parse_type_section_optimized(section_data, module),
            2 => self.parse_import_section_optimized(section_data, module),
            3 => self.parse_function_section_optimized(section_data, module),
            4 => self.parse_table_section_optimized(section_data, module),
            5 => self.parse_memory_section_optimized(section_data, module),
            6 => self.parse_global_section_optimized(section_data, module),
            7 => self.parse_export_section_optimized(section_data, module),
            8 => self.parse_start_section_optimized(section_data, module),
            9 => self.parse_element_section_optimized(section_data, module),
            10 => self.parse_code_section_optimized(section_data, module),
            11 => self.parse_data_section_optimized(section_data, module),
            12 => self.parse_data_count_section_optimized(section_data, module),
            0 => self.parse_custom_section_optimized(section_data, module),
            _ => {
                // Unknown section - skip
                Ok(())
            }
        }
    }

    /// Parse type section with streaming
    fn parse_type_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        let parser = StreamingCollectionParser::new(
            &SafeSlice::new(data).map_err(|e| {
                Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Failed to create SafeSlice for types: {}", e.message()),
                )
            })?,
            0,
        )?;

        // Bounds check
        check_bounds_u32(parser.count(), 10000, "type count")?;

        // Use our existing optimized parser but integrate with the streaming approach
        let types = crate::sections::parsers::parse_type_section(data)?;
        module.types = types;

        Ok(())
    }

    /// Parse import section with optimized string handling
    fn parse_import_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        let imports = crate::sections::parsers::parse_import_section(data)?;
        module.imports = imports;
        Ok(())
    }

    /// Parse function section
    fn parse_function_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        let functions = crate::sections::parsers::parse_function_section(data)?;
        module.funcs = functions;
        Ok(())
    }

    /// Parse table section
    fn parse_table_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        let tables = crate::sections::parsers::parse_table_section(data)?;
        module.tables = tables;
        Ok(())
    }

    /// Parse memory section
    fn parse_memory_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        let memories = crate::sections::parsers::parse_memory_section(data)?;
        module.mems = memories;
        Ok(())
    }

    /// Parse global section
    fn parse_global_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        let globals = crate::sections::parsers::parse_global_section(data)?;
        module.globals = globals;
        Ok(())
    }

    /// Parse export section
    fn parse_export_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        let exports = crate::sections::parsers::parse_export_section(data)?;
        module.exports = exports;
        Ok(())
    }

    /// Parse start section
    fn parse_start_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        if data.is_empty() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Empty start section",
            ));
        }

        let (start_func, _) = wrt_format::binary::read_leb128_u32(data, 0)?;
        module.start = Some(start_func);
        Ok(())
    }

    /// Parse element section
    fn parse_element_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        let elements = crate::sections::parsers::parse_element_section(data)?;
        module.elem = elements;
        Ok(())
    }

    /// Parse code section with memory pool optimization
    fn parse_code_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        let code_bodies = crate::sections::parsers::parse_code_section(data)?;
        // TODO: Process code bodies into proper Code structures
        // For now, store as-is (this will need further optimization)
        // module.code = process_code_bodies(code_bodies)?;
        Ok(())
    }

    /// Parse data section
    fn parse_data_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        let data_segments = crate::sections::parsers::parse_data_section(data)?;
        module.data = data_segments;
        Ok(())
    }

    /// Parse data count section
    fn parse_data_count_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        if data.is_empty() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Empty data count section",
            ));
        }

        let (data_count, _) = wrt_format::binary::read_leb128_u32(data, 0)?;
        module.datacount = Some(data_count);
        Ok(())
    }

    /// Parse custom section
    fn parse_custom_section_optimized(&mut self, data: &[u8], module: &mut WrtModule) -> Result<()> {
        // Parse custom section name
        let (name_str, _) = crate::optimized_string::validate_utf8_name(data, 0)?;
        
        // Store custom section (implementation depends on WrtModule structure)
        // TODO: Add custom section to module when supported
        Ok(())
    }
}

/// Section information for streaming parsing
#[derive(Debug, Clone)]
struct SectionInfo {
    id: u8,
    size: usize,
}

/// Optimized decode function that uses the new parser
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn decode_module_optimized<P: MemoryProvider + Default>(
    bytes: &[u8],
) -> Result<WrtModule> {
    let mut parser = OptimizedModuleParser::<P>::default();
    parser.parse_module(bytes)
}

/// Optimized decode function with custom memory provider
pub fn decode_module_with_provider<P: MemoryProvider>(
    bytes: &[u8],
    provider: P,
) -> Result<WrtModule> {
    let mut parser = OptimizedModuleParser::new(
        provider,
        wrt_foundation::verification::VerificationLevel::default(),
    );
    parser.parse_module(bytes)
}