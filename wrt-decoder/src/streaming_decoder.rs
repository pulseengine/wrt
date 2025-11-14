//! Streaming decoder for WebAssembly binaries with minimal memory usage
//!
//! This module provides a streaming API for decoding WebAssembly modules
//! that processes sections one at a time without loading the entire binary
//! into memory.

#[cfg(not(feature = "std"))]
extern crate alloc;

use wrt_format::module::{
    Function,
    Module as WrtModule,
};
use wrt_foundation::{
    bounded::BoundedVec,
    safe_memory::NoStdProvider,
};

use crate::{
    prelude::*,
    streaming_validator::{
        ComprehensivePlatformLimits,
        StreamingWasmValidator,
    },
};

/// Streaming decoder that processes WebAssembly modules section by section
pub struct StreamingDecoder<'a> {
    /// The WebAssembly binary data
    binary:          &'a [u8],
    /// Current offset in the binary
    offset:          usize,
    /// Platform limits for validation
    platform_limits: ComprehensivePlatformLimits,
    /// The module being built (std version)
    #[cfg(feature = "std")]
    module:          WrtModule,
    /// The module being built (no_std version)
    #[cfg(not(feature = "std"))]
    module:          WrtModule<NoStdProvider<8192>>,
}

impl<'a> StreamingDecoder<'a> {
    /// Create a new streaming decoder (std version)
    #[cfg(feature = "std")]
    pub fn new(binary: &'a [u8]) -> Result<Self> {
        let module = WrtModule::default();

        Ok(Self {
            binary,
            offset: 0,
            platform_limits: ComprehensivePlatformLimits::default(),
            module,
        })
    }

    /// Create a new streaming decoder (no_std version)
    #[cfg(not(feature = "std"))]
    pub fn new(binary: &'a [u8]) -> Result<Self> {
        let provider = wrt_foundation::safe_managed_alloc!(
            8192,
            wrt_foundation::budget_aware_provider::CrateId::Decoder
        )?;
        let module = WrtModule::default();

        Ok(Self {
            binary,
            offset: 0,
            platform_limits: ComprehensivePlatformLimits::default(),
            module,
        })
    }

    /// Decode the module header
    pub fn decode_header(&mut self) -> Result<()> {
        // Validate magic number and version
        if self.binary.len() < 8 {
            return Err(Error::parse_error(
                "Binary too small for WebAssembly header",
            ));
        }

        // Check magic number
        if &self.binary[0..4] != b"\0asm" {
            return Err(Error::parse_error("Invalid WebAssembly magic number"));
        }

        // Check version
        if self.binary[4..8] != [0x01, 0x00, 0x00, 0x00] {
            return Err(Error::parse_error("Unsupported WebAssembly version"));
        }

        self.offset = 8;
        Ok(())
    }

    /// Process the next section in the stream
    pub fn process_next_section(&mut self) -> Result<bool> {
        if self.offset >= self.binary.len() {
            return Ok(false); // No more sections
        }

        // Read section ID
        let section_id = self.binary[self.offset];
        self.offset += 1;

        // Read section size
        let (section_size, bytes_read) = read_leb128_u32(self.binary, self.offset)?;
        self.offset += bytes_read;

        let section_end = self.offset + section_size as usize;
        if section_end > self.binary.len() {
            return Err(Error::parse_error("Section extends beyond binary"));
        }

        // Process section data without loading it all into memory
        let section_data = &self.binary[self.offset..section_end];
        self.process_section(section_id, section_data)?;

        self.offset = section_end;
        Ok(true)
    }

    /// Process a specific section
    fn process_section(&mut self, section_id: u8, data: &[u8]) -> Result<()> {
        match section_id {
            1 => self.process_type_section(data),
            2 => self.process_import_section(data),
            3 => self.process_function_section(data),
            4 => self.process_table_section(data),
            5 => self.process_memory_section(data),
            6 => self.process_global_section(data),
            7 => self.process_export_section(data),
            8 => self.process_start_section(data),
            9 => self.process_element_section(data),
            10 => self.process_code_section(data),
            11 => self.process_data_section(data),
            12 => self.process_data_count_section(data),
            _ => self.process_custom_section(data),
        }
    }

    /// Process type section
    fn process_type_section(&mut self, data: &[u8]) -> Result<()> {
        use wrt_format::binary::read_leb128_u32;
        use wrt_foundation::types::ValueType;

        let mut offset = 0;
        let (count, bytes_read) = read_leb128_u32(data, offset)?;
        offset += bytes_read;

        #[cfg(feature = "std")]
        eprintln!("DEBUG process_type_section: count={}, data.len()={}", count, data.len());

        // Process each type one at a time
        for i in 0..count {
            // Check for function type marker (0x60)
            if offset >= data.len() {
                return Err(Error::parse_error("Unexpected end of type section"));
            }

            let type_marker = data[offset];
            offset += 1;

            if type_marker != 0x60 {
                return Err(Error::parse_error("Invalid function type marker"));
            }

            // Parse parameter types
            let (param_count, bytes_read) = read_leb128_u32(data, offset)?;
            offset += bytes_read;

            #[cfg(feature = "std")]
            let mut params = Vec::new();
            #[cfg(not(feature = "std"))]
            let mut params = alloc::vec::Vec::new();

            for _ in 0..param_count {
                if offset >= data.len() {
                    return Err(Error::parse_error("Unexpected end of parameter types"));
                }
                let param_type = ValueType::from_binary(data[offset])?;
                offset += 1;
                params.push(param_type);
            }

            // Parse result types
            let (result_count, bytes_read) = read_leb128_u32(data, offset)?;
            offset += bytes_read;

            #[cfg(feature = "std")]
            let mut results = Vec::new();
            #[cfg(not(feature = "std"))]
            let mut results = alloc::vec::Vec::new();

            for _ in 0..result_count {
                if offset >= data.len() {
                    return Err(Error::parse_error("Unexpected end of result types"));
                }
                let result_type = ValueType::from_binary(data[offset])?;
                offset += 1;
                results.push(result_type);
            }

            // Create function type and add to module
            #[cfg(feature = "std")]
            {
                use wrt_foundation::CleanCoreFuncType;
                let func_type = CleanCoreFuncType { params, results };
                self.module.types.push(func_type);
            }

            #[cfg(not(feature = "std"))]
            {
                use wrt_foundation::types::FuncType;
                let func_type = FuncType::new(params.into_iter(), results.into_iter())?;
                let _ = self.module.types.push(func_type);
            }

            #[cfg(feature = "std")]
            eprintln!("DEBUG process_type_section: parsed type #{}", i);
        }

        #[cfg(feature = "std")]
        eprintln!("DEBUG process_type_section: module.types.len()={}", self.module.types.len());

        Ok(())
    }

    /// Process import section
    fn process_import_section(&mut self, data: &[u8]) -> Result<()> {
        use crate::optimized_string::validate_utf8_name;

        let mut offset = 0;
        let (count, bytes_read) = read_leb128_u32(data, offset)?;
        offset += bytes_read;

        #[cfg(feature = "std")]
        eprintln!("DEBUG process_import_section: count={}, data.len()={}", count, data.len());

        // Process each import one at a time
        for i in 0..count {
            // Parse module name
            let (module_name, new_offset) = validate_utf8_name(data, offset)?;
            offset = new_offset;

            // Parse field name
            let (field_name, new_offset) = validate_utf8_name(data, offset)?;
            offset = new_offset;

            // Parse import kind
            if offset >= data.len() {
                return Err(Error::parse_error("Unexpected end of import kind"));
            }
            let kind = data[offset];
            offset += 1;

            #[cfg(feature = "std")]
            eprintln!("DEBUG import #{}: module='{}', field='{}', kind=0x{:02x}",
                     i, module_name, field_name, kind);

            // Parse import description and handle based on kind
            match kind {
                0x00 => {
                    // Function import
                    let (type_idx, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;

                    #[cfg(feature = "std")]
                    eprintln!("DEBUG import #{}: function type_idx={}", i, type_idx);

                    // Create placeholder function for imported function
                    // This ensures function index space includes imports
                    let func = Function {
                        type_idx,
                        locals: alloc::vec::Vec::new(),
                        code: alloc::vec::Vec::new(),
                    };
                    let _ = self.module.functions.push(func);

                    // Also add to imports list
                    #[cfg(feature = "std")]
                    {
                        use wrt_format::module::{Import, ImportDesc};
                        self.module.imports.push(Import {
                            module: String::from(module_name),
                            name: String::from(field_name),
                            desc: ImportDesc::Function(type_idx),
                        });
                    }
                },
                0x01 => {
                    // Table import - need to parse table type
                    // ref_type (1 byte) + limits (flags + min, optional max)
                    if offset >= data.len() {
                        return Err(Error::parse_error("Unexpected end of table import"));
                    }
                    let _ref_type = data[offset];
                    offset += 1;

                    // Parse limits
                    if offset >= data.len() {
                        return Err(Error::parse_error("Unexpected end of table limits"));
                    }
                    let flags = data[offset];
                    offset += 1;
                    let (min, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;
                    if flags & 0x01 != 0 {
                        let (max, bytes_read) = read_leb128_u32(data, offset)?;
                        offset += bytes_read;
                        #[cfg(feature = "std")]
                        eprintln!("DEBUG import #{}: table min={}, max={}", i, min, max);
                    } else {
                        #[cfg(feature = "std")]
                        eprintln!("DEBUG import #{}: table min={}", i, min);
                    }
                },
                0x02 => {
                    // Memory import - need to parse limits
                    if offset >= data.len() {
                        return Err(Error::parse_error("Unexpected end of memory import"));
                    }
                    let flags = data[offset];
                    offset += 1;
                    let (min, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;
                    if flags & 0x01 != 0 {
                        let (max, bytes_read) = read_leb128_u32(data, offset)?;
                        offset += bytes_read;
                        #[cfg(feature = "std")]
                        eprintln!("DEBUG import #{}: memory min={} pages, max={} pages", i, min, max);
                    } else {
                        #[cfg(feature = "std")]
                        eprintln!("DEBUG import #{}: memory min={} pages", i, min);
                    }
                },
                0x03 => {
                    // Global import - need to parse global type
                    // value_type (1 byte) + mutability (1 byte)
                    if offset + 1 >= data.len() {
                        return Err(Error::parse_error("Unexpected end of global import"));
                    }
                    let _value_type = data[offset];
                    offset += 1;
                    let _mutability = data[offset];
                    offset += 1;
                    #[cfg(feature = "std")]
                    eprintln!("DEBUG import #{}: global", i);
                },
                0x04 => {
                    // Tag import
                    let (type_idx, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;
                    #[cfg(feature = "std")]
                    eprintln!("DEBUG import #{}: tag type_idx={}", i, type_idx);
                },
                _ => {
                    return Err(Error::parse_error("Invalid import kind"));
                },
            }
        }

        #[cfg(feature = "std")]
        eprintln!("DEBUG process_import_section: done, functions.len()={}", self.module.functions.len());

        Ok(())
    }

    /// Process function section
    fn process_function_section(&mut self, data: &[u8]) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = read_leb128_u32(data, offset)?;
        offset += bytes_read;

        // Reserve space for functions
        for _ in 0..count {
            let (type_idx, bytes_read) = read_leb128_u32(data, offset)?;
            offset += bytes_read;

            // Create function with empty body for now
            let func = Function {
                type_idx,
                locals: alloc::vec::Vec::new(),
                code: alloc::vec::Vec::new(),
            };

            let _ = self.module.functions.push(func);
        }

        Ok(())
    }

    /// Process table section
    fn process_table_section(&mut self, data: &[u8]) -> Result<()> {
        // Parse tables one at a time
        Ok(())
    }

    /// Process memory section
    fn process_memory_section(&mut self, data: &[u8]) -> Result<()> {
        use wrt_format::binary::read_leb128_u32;

        let mut offset = 0;
        let (count, bytes_read) = read_leb128_u32(data, offset)?;
        offset += bytes_read;

        #[cfg(feature = "std")]
        eprintln!("[MEMORY_SECTION] Processing {} memories", count);

        // Check memory count against platform limits
        if count > 1 && !self.platform_limits.max_components > 0 {
            return Err(Error::resource_exhausted(
                "Multiple memories not supported on this platform",
            ));
        }

        // Process each memory one at a time
        for i in 0..count {
            // Parse limits flag (0x00 = min only, 0x01 = min and max, 0x03 = min/max/shared)
            let flags = data[offset];
            offset += 1;

            let (min, bytes_read) = read_leb128_u32(data, offset)?;
            offset += bytes_read;

            let max = if flags & 0x01 != 0 {
                let (max_val, bytes_read) = read_leb128_u32(data, offset)?;
                offset += bytes_read;
                Some(max_val)
            } else {
                None
            };

            let shared = (flags & 0x02) != 0;

            #[cfg(feature = "std")]
            eprintln!("[MEMORY_SECTION] Memory {}: min={}, max={:?}, shared={}", i, min, max, shared);

            // Create memory type
            let memory_type = wrt_foundation::types::MemoryType {
                limits: wrt_foundation::types::Limits { min, max },
                shared,
            };

            // Add to module
            self.module.memories.push(memory_type);

            #[cfg(feature = "std")]
            eprintln!("[MEMORY_SECTION] Added memory {}, total memories: {}", i, self.module.memories.len());
        }

        Ok(())
    }

    /// Process global section
    fn process_global_section(&mut self, data: &[u8]) -> Result<()> {
        // Parse globals one at a time
        Ok(())
    }

    /// Process export section
    fn process_export_section(&mut self, data: &[u8]) -> Result<()> {
        use wrt_format::binary::read_leb128_u32;

        use crate::optimized_string::validate_utf8_name;

        let (count, mut offset) = read_leb128_u32(data, 0)?;

        #[cfg(feature = "std")]
        {
            eprintln!("DEBUG process_export_section: count={}, initial_offset={}, data.len()={}", count, offset, data.len());
            eprintln!("DEBUG first 20 bytes: {:02x?}", &data[..20.min(data.len())]);
        }

        for i in 0..count {
            // Parse export name - use validate_utf8_name for std builds to avoid
            // BoundedString issues
            #[cfg(feature = "std")]
            {
                eprintln!("DEBUG export #{}: before validate_utf8_name, offset={}", i, offset);
            }
            let (export_name_str, new_offset) = validate_utf8_name(data, offset)?;
            #[cfg(feature = "std")]
            {
                eprintln!(
                    "DEBUG export #{}: after validate_utf8_name, name='{}', new_offset={}",
                    i, export_name_str, new_offset
                );
            }
            offset = new_offset;

            #[cfg(feature = "std")]
            {
                eprintln!("DEBUG export #{}: after name, offset now = {}", i, offset);
            }

            if offset >= data.len() {
                return Err(Error::parse_error("Unexpected end of export kind"));
            }

            // Parse export kind
            let kind_byte = data[offset];
            offset += 1;

            #[cfg(feature = "std")]
            {
                eprintln!("DEBUG export #{}: after kind, offset now = {}, kind_byte = 0x{:02x}", i, offset, kind_byte);
            }
            let kind = match kind_byte {
                0x00 => wrt_format::module::ExportKind::Function,
                0x01 => wrt_format::module::ExportKind::Table,
                0x02 => wrt_format::module::ExportKind::Memory,
                0x03 => wrt_format::module::ExportKind::Global,
                _ => {
                    #[cfg(feature = "std")]
                    {
                        eprintln!("DEBUG: Invalid export kind byte: 0x{:02x}", kind_byte);
                    }
                    return Err(Error::parse_error("Invalid export kind"));
                },
            };

            // Parse export index
            let (index, bytes_consumed) = read_leb128_u32(data, offset)?;
            offset += bytes_consumed;

            #[cfg(feature = "std")]
            {
                eprintln!("DEBUG export #{}: after index, offset now = {}, index={}", i, offset, index);
            }

            // Add export to module
            #[cfg(feature = "std")]
            {
                self.module.exports.push(wrt_format::module::Export {
                    name: String::from(export_name_str),
                    kind,
                    index,
                });
            }
            #[cfg(not(feature = "std"))]
            {
                use wrt_foundation::BoundedString;

                let name = BoundedString::<1024>::try_from_str(export_name_str)
                    .map_err(|_| wrt_error::Error::parse_error("Export name too long"))?;

                let _ = self.module.exports.push(wrt_format::module::Export {
                    name,
                    kind,
                    index,
                });
            }
        }

        Ok(())
    }

    /// Process start section
    fn process_start_section(&mut self, data: &[u8]) -> Result<()> {
        let (start_idx, _) = read_leb128_u32(data, 0)?;
        self.module.start = Some(start_idx);
        Ok(())
    }

    /// Process element section
    fn process_element_section(&mut self, data: &[u8]) -> Result<()> {
        // Parse elements one at a time
        Ok(())
    }

    /// Process code section
    fn process_code_section(&mut self, data: &[u8]) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = read_leb128_u32(data, offset)?;
        offset += bytes_read;

        #[cfg(feature = "std")]
        eprintln!("[CODE_SECTION] Processing {} function bodies", count);

        // Code bodies are for module-defined functions only (not imports)
        // So code[i] goes to function[num_imports + i]
        let num_imports = self.module.imports.len();

        // Process each function body one at a time
        for i in 0..count {
            let (body_size, bytes_read) = read_leb128_u32(data, offset)?;
            offset += bytes_read;

            let body_start = offset;
            let body_end = offset + body_size as usize;
            if body_end > data.len() {
                return Err(Error::parse_error("Function body extends beyond section"));
            }

            // Parse locals first (they come before instructions in the function body)
            let mut body_offset = 0;
            let (local_count, local_bytes) = read_leb128_u32(&data[body_start..body_end], body_offset)?;
            body_offset += local_bytes;

            #[cfg(feature = "std")]
            eprintln!("[CODE_SECTION] Function {}: {} local groups", i, local_count);

            // Code section index i corresponds to module-defined function at index (num_imports + i)
            let func_index = num_imports + i as usize;
            if let Some(func) = self.module.functions.get_mut(func_index) {
                // Parse local variable declarations
                for _ in 0..local_count {
                    let (count, bytes) = read_leb128_u32(&data[body_start..body_end], body_offset)?;
                    body_offset += bytes;

                    if body_offset >= body_size as usize {
                        return Err(Error::parse_error("Unexpected end of function body"));
                    }

                    let value_type = data[body_start + body_offset];
                    body_offset += 1;

                    // Convert to ValueType and add to locals
                    let vt = match value_type {
                        0x7F => wrt_foundation::types::ValueType::I32,
                        0x7E => wrt_foundation::types::ValueType::I64,
                        0x7D => wrt_foundation::types::ValueType::F32,
                        0x7C => wrt_foundation::types::ValueType::F64,
                        _ => return Err(Error::parse_error("Invalid local type")),
                    };

                    // Add 'count' locals of this type
                    for _ in 0..count {
                        let _ = func.locals.push(vt);
                    }
                }

                // Now copy only the instruction bytes (after locals, before the implicit 'end')
                let instructions_start = body_start + body_offset;
                let instructions_data = &data[instructions_start..body_end];
                func.code.extend_from_slice(instructions_data);

                #[cfg(feature = "std")]
                eprintln!("[CODE_SECTION] Function {}: {} locals, {} instruction bytes",
                         i, func.locals.len(), func.code.len());
            }

            offset = body_end;
        }

        Ok(())
    }

    /// Process data section
    fn process_data_section(&mut self, data: &[u8]) -> Result<()> {
        // Parse data segments one at a time
        Ok(())
    }

    /// Process data count section
    fn process_data_count_section(&mut self, _data: &[u8]) -> Result<()> {
        // Used for validation only
        Ok(())
    }

    /// Process custom section
    fn process_custom_section(&mut self, _data: &[u8]) -> Result<()> {
        // Skip custom sections or process specific ones
        Ok(())
    }

    /// Finish decoding and return the module
    /// Finish decoding and return the module (std version)
    #[cfg(feature = "std")]
    pub fn finish(self) -> Result<WrtModule> {
        Ok(self.module)
    }

    /// Finish decoding and return the module (no_std version)
    #[cfg(not(feature = "std"))]
    pub fn finish(self) -> Result<WrtModule<NoStdProvider<8192>>> {
        Ok(self.module)
    }
}

/// Decode a WebAssembly module using streaming processing (std version)
#[cfg(feature = "std")]
pub fn decode_module_streaming(binary: &[u8]) -> Result<WrtModule> {
    // Enter module scope for bump allocator - all Vec allocations will be tracked
    let _scope = wrt_foundation::capabilities::MemoryFactory::enter_module_scope(
        wrt_foundation::budget_aware_provider::CrateId::Decoder,
    )?;

    let mut decoder = StreamingDecoder::new(binary)?;
    decoder.decode_header()?;

    // Process all sections
    while decoder.process_next_section()? {
        // Process sections one at a time
    }

    decoder.finish()
    // Scope drops here, memory available for reuse
}

/// Decode a WebAssembly module using streaming processing (no_std version)
#[cfg(not(feature = "std"))]
pub fn decode_module_streaming(binary: &[u8]) -> Result<WrtModule<NoStdProvider<8192>>> {
    let mut decoder = StreamingDecoder::new(binary)?;

    // First validate and decode the header
    decoder.decode_header()?;

    // Process sections one at a time
    while decoder.process_next_section()? {
        // Each section is processed with minimal memory usage
    }

    // Return the completed module
    decoder.finish()
}
