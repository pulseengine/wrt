//! Streaming decoder for WebAssembly binaries with minimal memory usage
//!
//! This module provides a streaming API for decoding WebAssembly modules
//! that processes sections one at a time without loading the entire binary
//! into memory.

#[cfg(not(feature = "std"))]
extern crate alloc;

use alloc::vec::Vec;

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
    /// Number of function imports (for proper function indexing)
    num_function_imports: usize,
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
            num_function_imports: 0,
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
            num_function_imports: 0,
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

                    // Track function imports for proper function indexing
                    self.num_function_imports += 1;

                    // Store the import in module.imports for runtime resolution
                    #[cfg(feature = "std")]
                    {
                        use wrt_format::module::{Import, ImportDesc};
                        let import = Import {
                            module: module_name.to_string(),
                            name: field_name.to_string(),
                            desc: ImportDesc::Function(type_idx),
                        };
                        self.module.imports.push(import);
                    }

                    eprintln!("DEBUG: Recorded import {}::{} at function index {}",
                             module_name, field_name, self.num_function_imports - 1);
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
                    let value_type_byte = data[offset];
                    offset += 1;
                    let mutability_byte = data[offset];
                    offset += 1;

                    // WebAssembly spec: only 0x00 (immutable) and 0x01 (mutable) are valid
                    if mutability_byte != 0x00 && mutability_byte != 0x01 {
                        return Err(Error::parse_error("malformed mutability"));
                    }

                    // Parse value type
                    let value_type = match value_type_byte {
                        0x7F => wrt_foundation::ValueType::I32,
                        0x7E => wrt_foundation::ValueType::I64,
                        0x7D => wrt_foundation::ValueType::F32,
                        0x7C => wrt_foundation::ValueType::F64,
                        0x7B => wrt_foundation::ValueType::V128,
                        0x70 => wrt_foundation::ValueType::FuncRef,
                        0x6F => wrt_foundation::ValueType::ExternRef,
                        _ => return Err(Error::parse_error("Invalid global import value type")),
                    };

                    #[cfg(feature = "std")]
                    eprintln!("DEBUG import #{}: global type={:?} mutable={}", i, value_type, mutability_byte != 0);

                    // Store the global import in module.imports for runtime resolution
                    #[cfg(feature = "std")]
                    {
                        use wrt_format::module::{Import, ImportDesc};
                        use wrt_format::types::FormatGlobalType;
                        let global_type = FormatGlobalType {
                            value_type,
                            mutable: mutability_byte != 0,
                        };
                        let import = Import {
                            module: module_name.to_string(),
                            name: field_name.to_string(),
                            desc: ImportDesc::Global(global_type),
                        };
                        self.module.imports.push(import);
                    }
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

        #[cfg(feature = "std")]
        eprintln!("[FUNC_SECTION] Processing {} functions from {} bytes", count, data.len());

        // Reserve space for functions
        for i in 0..count {
            let (type_idx, bytes_read) = read_leb128_u32(data, offset)?;
            offset += bytes_read;

            #[cfg(feature = "std")]
            eprintln!("[FUNC_SECTION] Function {} has type_idx={}", i, type_idx);

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
        use wrt_format::binary::read_leb128_u32;
        use wrt_foundation::types::{Limits, RefType, TableType};

        let mut offset = 0;
        let (count, bytes_read) = read_leb128_u32(data, offset)?;
        offset += bytes_read;

        #[cfg(feature = "std")]
        eprintln!("[TABLE_SECTION] Processing {} tables", count);

        // Process each table one at a time
        for i in 0..count {
            // Parse ref_type (element type)
            // WebAssembly 2.0 tables can have init expressions with 0x40 0x00 prefix
            if offset >= data.len() {
                return Err(Error::parse_error("Unexpected end of table section"));
            }
            let first_byte = data[offset];

            // Check for table with init expression (0x40 0x00 prefix)
            let has_init_expr = first_byte == 0x40;
            if has_init_expr {
                offset += 1;
                // Read the reserved 0x00 byte
                if offset >= data.len() || data[offset] != 0x00 {
                    return Err(Error::parse_error("Expected 0x00 after 0x40 in table with init expr"));
                }
                offset += 1;
            }

            // Now parse the ref_type
            if offset >= data.len() {
                return Err(Error::parse_error("Unexpected end of table section (ref_type)"));
            }
            let ref_type_byte = data[offset];
            offset += 1;

            let element_type = match ref_type_byte {
                0x70 => RefType::Funcref,   // funcref
                0x6F => RefType::Externref, // externref
                _ => {
                    return Err(Error::parse_error("Unknown table element type"));
                }
            };

            // Parse limits (flags + min, optional max)
            if offset >= data.len() {
                return Err(Error::parse_error("Unexpected end of table limits"));
            }
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

            // Parse and validate init expression if present (ends with 0x0B)
            if has_init_expr {
                // Count global imports - at table section time, only imported globals exist
                use wrt_format::module::ImportDesc;
                let num_global_imports = self.module.imports.iter()
                    .filter(|imp| matches!(imp.desc, ImportDesc::Global(_)))
                    .count();

                // Scan for the end opcode (0x0B), handling nested blocks
                let mut block_depth = 0u32;
                loop {
                    if offset >= data.len() {
                        return Err(Error::parse_error("Unexpected end of table init expression"));
                    }
                    let opcode = data[offset];
                    offset += 1;

                    match opcode {
                        // Block-starting opcodes
                        0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x11 => {
                            block_depth += 1;
                        }
                        // End opcode
                        0x0B => {
                            if block_depth == 0 {
                                break;
                            }
                            block_depth -= 1;
                        }
                        // global.get - validate global index
                        0x23 => {
                            // Read global index (LEB128)
                            let (global_idx, bytes_read) = read_leb128_u32(data, offset)?;
                            offset += bytes_read;
                            // At table section time, only imported globals are available
                            // Defined globals (section 9) haven't been parsed yet
                            if global_idx as usize >= num_global_imports {
                                return Err(Error::validation_error("unknown global"));
                            }
                        }
                        // Skip LEB128 immediates for common opcodes
                        0x41 => {
                            // i32.const - skip LEB128
                            while offset < data.len() && (data[offset] & 0x80) != 0 {
                                offset += 1;
                            }
                            if offset < data.len() {
                                offset += 1;
                            }
                        }
                        0x42 => {
                            // i64.const - skip LEB128
                            while offset < data.len() && (data[offset] & 0x80) != 0 {
                                offset += 1;
                            }
                            if offset < data.len() {
                                offset += 1;
                            }
                        }
                        0xD0 => {
                            // ref.null - skip heap type byte
                            if offset < data.len() {
                                offset += 1;
                            }
                        }
                        _ => {
                            // Other opcodes - continue scanning
                        }
                    }
                }
                #[cfg(feature = "std")]
                eprintln!("[TABLE_SECTION] Table {} has init expression (validated)", i);
            }

            #[cfg(feature = "std")]
            eprintln!(
                "[TABLE_SECTION] Table {}: type={:?}, min={}, max={:?}",
                i, element_type, min, max
            );

            // Create table type and add to module
            let table_type = TableType::new(element_type, Limits { min, max });
            self.module.tables.push(table_type);

            #[cfg(feature = "std")]
            eprintln!(
                "[TABLE_SECTION] Added table {}, total tables: {}",
                i,
                self.module.tables.len()
            );
        }

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
            if offset >= data.len() {
                return Err(Error::parse_error("Memory section truncated: missing limits flag"));
            }
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

            // WebAssembly spec: memory size must be at most 65536 pages (4GB)
            const MAX_MEMORY_PAGES: u32 = 65536;
            if min > MAX_MEMORY_PAGES {
                return Err(Error::validation_error("memory size must be at most 65536 pages (4 GiB)"));
            }
            if let Some(max_val) = max {
                if max_val > MAX_MEMORY_PAGES {
                    return Err(Error::validation_error("memory size must be at most 65536 pages (4 GiB)"));
                }
            }

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
        use wrt_format::binary::read_leb128_u32;
        use wrt_foundation::types::ValueType;
        use wrt_format::types::FormatGlobalType;
        use wrt_format::module::Global;

        let (count, mut offset) = read_leb128_u32(data, 0)?;

        #[cfg(feature = "std")]
        {
            eprintln!("DEBUG process_global_section: count={}, data.len()={}, offset after count={}", count, data.len(), offset);
            eprint!("DEBUG first 20 bytes: ");
            for i in 0..data.len().min(20) {
                eprint!("{:02x} ", data[i]);
            }
            eprintln!();
        }

        for i in 0..count {
            #[cfg(feature = "std")]
            eprintln!("DEBUG global #{}: starting at offset {}", i, offset);

            // Parse global type: value_type + mutability
            if offset >= data.len() {
                return Err(Error::parse_error("Unexpected end of global type"));
            }

            // Parse value type
            let value_type = match data[offset] {
                0x7F => ValueType::I32,
                0x7E => ValueType::I64,
                0x7D => ValueType::F32,
                0x7C => ValueType::F64,
                0x7B => ValueType::V128,
                0x70 => ValueType::FuncRef,
                0x6F => ValueType::ExternRef,
                _ => return Err(Error::parse_error("Invalid global value type")),
            };
            offset += 1;

            // Parse mutability (0x00 = const, 0x01 = var)
            if offset >= data.len() {
                return Err(Error::parse_error("Unexpected end of global mutability"));
            }
            let mutability_byte = data[offset];
            // WebAssembly spec: only 0x00 (immutable) and 0x01 (mutable) are valid
            if mutability_byte != 0x00 && mutability_byte != 0x01 {
                return Err(Error::parse_error("malformed mutability"));
            }
            let mutable = mutability_byte == 0x01;
            offset += 1;

            // Parse init expression - must properly parse opcodes and their arguments
            // Cannot just scan for 0x0b because it can appear as a value (e.g., i32.const 11)
            let init_start = offset;

            // Parse init expression by understanding opcodes
            loop {
                if offset >= data.len() {
                    return Err(Error::parse_error("Init expression missing end marker"));
                }

                let opcode = data[offset];
                offset += 1;

                match opcode {
                    0x0b => {
                        // End marker - we're done
                        break;
                    }
                    0x41 => {
                        // i32.const - followed by LEB128 i32
                        let (_, bytes_read) = wrt_format::binary::read_leb128_i32(data, offset)?;
                        offset += bytes_read;
                    }
                    0x42 => {
                        // i64.const - followed by LEB128 i64
                        let (_, bytes_read) = wrt_format::binary::read_leb128_i64(data, offset)?;
                        offset += bytes_read;
                    }
                    0x43 => {
                        // f32.const - followed by 4 bytes
                        offset += 4;
                    }
                    0x44 => {
                        // f64.const - followed by 8 bytes
                        offset += 8;
                    }
                    0x23 => {
                        // global.get - followed by LEB128 global index
                        let (_, bytes_read) = wrt_format::binary::read_leb128_u32(data, offset)?;
                        offset += bytes_read;
                    }
                    0xd0 => {
                        // ref.null - followed by heap type (single byte for common types)
                        if offset < data.len() {
                            offset += 1; // Skip the heap type byte
                        }
                    }
                    0xd2 => {
                        // ref.func - followed by LEB128 func index
                        let (_, bytes_read) = wrt_format::binary::read_leb128_u32(data, offset)?;
                        offset += bytes_read;
                    }
                    _ => {
                        // Unknown opcode in init expression - skip to find 0x0b
                        // This is a fallback for any opcodes we don't handle
                        #[cfg(feature = "std")]
                        eprintln!("DEBUG global: unknown init opcode 0x{:02x} at offset {}", opcode, offset - 1);
                        // Continue to next byte
                    }
                }
            }

            // Extract init expression bytes (including the 0x0b end marker)
            let init_bytes = data[init_start..offset].to_vec();

            let global_type = FormatGlobalType {
                value_type,
                mutable,
            };

            let global = Global {
                global_type,
                init: init_bytes,
            };

            self.module.globals.push(global);

            #[cfg(feature = "std")]
            eprintln!("DEBUG global #{}: type={:?}, mutable={}, init_len={}", i, value_type, mutable, offset - init_start);
        }

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
        use wrt_format::pure_format_types::{PureElementInit, PureElementMode, PureElementSegment};

        let mut offset = 0;
        let (count, bytes_read) = read_leb128_u32(data, offset)?;
        offset += bytes_read;

        #[cfg(feature = "std")]
        eprintln!("[ELEM_SECTION] Parsing {} element segments from {} bytes", count, data.len());

        for elem_idx in 0..count {
            // Parse element segment flags (see WebAssembly spec 5.5.10)
            // Flags determine the mode and encoding:
            // 0: Active, implicit table 0, offset expr, funcidx vec
            // 1: Passive, reftype, expression vec
            // 2: Active, explicit table, offset expr, elemkind=0x00, funcidx vec
            // 3: Declarative, reftype, expression vec
            // 4: Active, explicit table, offset expr, funcidx vec
            // 5: Passive, reftype, expression vec
            // 6: Active, explicit table, offset expr, reftype, expression vec
            // 7: Declarative, reftype, expression vec
            let (flags, bytes_read) = read_leb128_u32(data, offset)?;
            offset += bytes_read;

            #[cfg(feature = "std")]
            eprintln!("[ELEM_SECTION] Element {} has flags 0x{:02x}", elem_idx, flags);

            let (mode, offset_expr_bytes, element_type) = match flags {
                0 => {
                    // Active, table 0, funcref, func indices
                    // Parse offset expression (ends with 0x0B = end)
                    let expr_start = offset;
                    while offset < data.len() && data[offset] != 0x0B {
                        offset += 1;
                    }
                    if offset < data.len() {
                        offset += 1; // consume 0x0B
                    }
                    let offset_expr_bytes: Vec<u8> = data[expr_start..offset].to_vec();
                    #[cfg(feature = "std")]
                    eprintln!("[ELEM_SECTION] Element {}: Active table 0, offset expr {} bytes",
                             elem_idx, offset_expr_bytes.len());

                    (
                        PureElementMode::Active { table_index: 0, offset_expr_len: offset_expr_bytes.len() as u32 },
                        offset_expr_bytes,
                        wrt_format::types::RefType::Funcref,
                    )
                },
                1 => {
                    // Passive, element type, expressions
                    let elem_type = data[offset];
                    offset += 1;
                    let ref_type = match elem_type {
                        0x70 => wrt_format::types::RefType::Funcref,
                        0x6F => wrt_format::types::RefType::Externref,
                        _ => wrt_format::types::RefType::Funcref,
                    };
                    #[cfg(feature = "std")]
                    eprintln!("[ELEM_SECTION] Element {}: Passive with type {:?}", elem_idx, ref_type);
                    (PureElementMode::Passive, Vec::new(), ref_type)
                },
                2 => {
                    // Active, explicit table index, offset expr, elemkind=0x00, funcidx vec
                    // Per WebAssembly spec: flags=2 is "2:flags x:tableidx e:expr 0x00:elemkind y*:vec(funcidx)"

                    // Parse table index
                    let (table_index, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;

                    // Parse offset expression (ends with 0x0B)
                    let expr_start = offset;
                    while offset < data.len() && data[offset] != 0x0B {
                        offset += 1;
                    }
                    if offset < data.len() {
                        offset += 1; // consume 0x0B
                    }
                    let offset_expr_bytes: Vec<u8> = data[expr_start..offset].to_vec();

                    // Parse elemkind (must be 0x00 = funcref)
                    let _elemkind = data[offset];
                    offset += 1;

                    #[cfg(feature = "std")]
                    eprintln!("[ELEM_SECTION] Element {}: Active table {}, offset expr {} bytes (legacy funcidx)",
                             elem_idx, table_index, offset_expr_bytes.len());

                    (
                        PureElementMode::Active { table_index, offset_expr_len: offset_expr_bytes.len() as u32 },
                        offset_expr_bytes,
                        wrt_format::types::RefType::Funcref,
                    )
                },
                3 => {
                    // Declarative, element type, expressions
                    let elem_type = data[offset];
                    offset += 1;
                    let ref_type = match elem_type {
                        0x70 => wrt_format::types::RefType::Funcref,
                        0x6F => wrt_format::types::RefType::Externref,
                        _ => wrt_format::types::RefType::Funcref,
                    };
                    #[cfg(feature = "std")]
                    eprintln!("[ELEM_SECTION] Element {}: Declarative with type {:?}", elem_idx, ref_type);
                    (PureElementMode::Declared, Vec::new(), ref_type)
                },
                4 => {
                    // Active, explicit table_idx, no element type (funcref implicit), offset expr, funcidx vec (legacy)
                    // Per WebAssembly spec: flags=4 has explicit table index
                    let (table_index, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;

                    // Parse offset expression
                    let expr_start = offset;
                    while offset < data.len() && data[offset] != 0x0B {
                        offset += 1;
                    }
                    if offset < data.len() {
                        offset += 1; // consume 0x0B
                    }
                    let offset_expr_bytes: Vec<u8> = data[expr_start..offset].to_vec();
                    #[cfg(feature = "std")]
                    eprintln!("[ELEM_SECTION] Element {}: Active table {}, offset expr {} bytes (legacy)",
                             elem_idx, table_index, offset_expr_bytes.len());

                    (
                        PureElementMode::Active { table_index, offset_expr_len: offset_expr_bytes.len() as u32 },
                        offset_expr_bytes,
                        wrt_format::types::RefType::Funcref,
                    )
                },
                5 => {
                    // Passive, expressions with type
                    let ref_type_byte = data[offset];
                    offset += 1;
                    let ref_type = match ref_type_byte {
                        0x70 => wrt_format::types::RefType::Funcref,
                        0x6F => wrt_format::types::RefType::Externref,
                        _ => wrt_format::types::RefType::Funcref,
                    };
                    #[cfg(feature = "std")]
                    eprintln!("[ELEM_SECTION] Element {}: Passive with type {:?}", elem_idx, ref_type);
                    (PureElementMode::Passive, Vec::new(), ref_type)
                },
                6 => {
                    // Active explicit table, expressions with type
                    // Format: table_idx offset_expr ref_type vec(expr)
                    let (table_index, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;

                    let expr_start = offset;
                    while offset < data.len() && data[offset] != 0x0B {
                        offset += 1;
                    }
                    if offset < data.len() {
                        offset += 1; // consume 0x0B
                    }
                    let offset_expr_bytes: Vec<u8> = data[expr_start..offset].to_vec();

                    // Parse ref_type (comes after offset expression, before items)
                    if offset >= data.len() {
                        return Err(Error::parse_error("Unexpected end of element segment (ref_type)"));
                    }
                    let ref_type_byte = data[offset];
                    offset += 1;
                    let ref_type = match ref_type_byte {
                        0x70 => wrt_format::types::RefType::Funcref,
                        0x6F => wrt_format::types::RefType::Externref,
                        _ => wrt_format::types::RefType::Funcref,
                    };

                    #[cfg(feature = "std")]
                    eprintln!("[ELEM_SECTION] Element {}: Active table {} with expressions, offset expr {} bytes, type {:?}",
                             elem_idx, table_index, offset_expr_bytes.len(), ref_type);

                    (
                        PureElementMode::Active { table_index, offset_expr_len: offset_expr_bytes.len() as u32 },
                        offset_expr_bytes,
                        ref_type,
                    )
                },
                7 => {
                    // Declarative, expressions with type
                    let ref_type_byte = data[offset];
                    offset += 1;
                    let ref_type = match ref_type_byte {
                        0x70 => wrt_format::types::RefType::Funcref,
                        0x6F => wrt_format::types::RefType::Externref,
                        _ => wrt_format::types::RefType::Funcref,
                    };
                    #[cfg(feature = "std")]
                    eprintln!("[ELEM_SECTION] Element {}: Declarative with type {:?}", elem_idx, ref_type);
                    (PureElementMode::Declared, Vec::new(), ref_type)
                },
                _ => {
                    return Err(Error::parse_error("Unknown element segment flags"));
                }
            };

            // Parse element items
            let (item_count, bytes_read) = read_leb128_u32(data, offset)?;
            offset += bytes_read;

            #[cfg(feature = "std")]
            eprintln!("[ELEM_SECTION] Element {} has {} items", elem_idx, item_count);

            let init_data = if flags == 0 || flags == 2 || flags == 4 {
                // Legacy function indices format (flags 0, 2, 4)
                let mut func_indices = Vec::with_capacity(item_count as usize);
                for i in 0..item_count {
                    let (func_idx, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;
                    func_indices.push(func_idx);
                    #[cfg(feature = "std")]
                    if i < 5 || i == item_count - 1 {
                        eprintln!("[ELEM_SECTION]   item[{}] = func {}", i, func_idx);
                    }
                }
                PureElementInit::FunctionIndices(func_indices)
            } else {
                // Expression format (flags 1, 2, 3, 5, 6, 7)
                let mut expr_bytes = Vec::with_capacity(item_count as usize);
                for i in 0..item_count {
                    let expr_start = offset;
                    // Find end of expression (0x0B)
                    while offset < data.len() && data[offset] != 0x0B {
                        offset += 1;
                    }
                    if offset < data.len() {
                        offset += 1; // consume 0x0B
                    }
                    let expr_data: Vec<u8> = data[expr_start..offset].to_vec();
                    #[cfg(feature = "std")]
                    if i < 5 {
                        eprintln!("[ELEM_SECTION]   item[{}] = expr {} bytes: {:02x?}",
                                 i, expr_data.len(), &expr_data[..expr_data.len().min(10)]);
                    }
                    expr_bytes.push(expr_data);
                }
                PureElementInit::ExpressionBytes(expr_bytes)
            };

            let elem_segment = PureElementSegment {
                mode,
                element_type,
                offset_expr_bytes,
                init_data,
            };

            self.module.elements.push(elem_segment);
            #[cfg(feature = "std")]
            eprintln!("[ELEM_SECTION] Element {} added, module.elements.len()={}",
                     elem_idx, self.module.elements.len());
        }

        #[cfg(feature = "std")]
        eprintln!("[ELEM_SECTION] Complete: {} element segments parsed", count);

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
        let num_imports = self.num_function_imports;

        #[cfg(feature = "std")]
        eprintln!("[CODE_SECTION] num_function_imports={}, total functions={}",
            num_imports, self.module.functions.len());

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
        use wrt_format::pure_format_types::{PureDataMode, PureDataSegment};

        let mut offset = 0;
        let (count, bytes_read) = read_leb128_u32(data, offset)?;
        offset += bytes_read;

        #[cfg(feature = "std")]
        eprintln!("[DATA_SECTION] Parsing {} data segments from {} bytes", count, data.len());

        for i in 0..count {
            if offset >= data.len() {
                return Err(Error::parse_error("Unexpected end of data section"));
            }

            let tag = data[offset];
            offset += 1;

            let segment = match tag {
                // Active data segment with implicit memory 0
                0x00 => {
                    // Parse offset expression - find the end (0x0B terminator)
                    let expr_start = offset;
                    let mut depth = 1u32;
                    while offset < data.len() {
                        let opcode = data[offset];
                        offset += 1;

                        match opcode {
                            0x02 | 0x03 | 0x04 => depth += 1,
                            0x0B => {
                                depth = depth.saturating_sub(1);
                                if depth == 0 {
                                    break;
                                }
                            }
                            0x41 | 0x42 | 0x23 => {
                                // i32.const, i64.const, global.get - skip LEB128
                                while offset < data.len() && data[offset] & 0x80 != 0 {
                                    offset += 1;
                                }
                                if offset < data.len() {
                                    offset += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    let offset_expr_bytes = data[expr_start..offset].to_vec();

                    // Parse data byte count and data
                    let (data_len, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;

                    if offset + data_len as usize > data.len() {
                        return Err(Error::parse_error("Data segment data exceeds bounds"));
                    }

                    let data_bytes = data[offset..offset + data_len as usize].to_vec();
                    offset += data_len as usize;

                    #[cfg(feature = "std")]
                    eprintln!("[DATA_SECTION] Segment {}: active (mem 0), offset_expr {} bytes, data {} bytes",
                             i, offset_expr_bytes.len(), data_bytes.len());

                    PureDataSegment {
                        mode: PureDataMode::Active {
                            memory_index: 0,
                            offset_expr_len: offset_expr_bytes.len() as u32,
                        },
                        offset_expr_bytes,
                        data_bytes,
                    }
                }
                // Passive data segment
                0x01 => {
                    // Parse data byte count and data
                    let (data_len, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;

                    if offset + data_len as usize > data.len() {
                        return Err(Error::parse_error("Data segment data exceeds bounds"));
                    }

                    let data_bytes = data[offset..offset + data_len as usize].to_vec();
                    offset += data_len as usize;

                    #[cfg(feature = "std")]
                    eprintln!("[DATA_SECTION] Segment {}: passive, data {} bytes", i, data_bytes.len());

                    PureDataSegment {
                        mode: PureDataMode::Passive,
                        offset_expr_bytes: Vec::new(),
                        data_bytes,
                    }
                }
                // Active data segment with explicit memory index
                0x02 => {
                    // Parse memory index
                    let (memory_index, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;

                    // Parse offset expression
                    let expr_start = offset;
                    let mut depth = 1u32;
                    while offset < data.len() {
                        let opcode = data[offset];
                        offset += 1;

                        match opcode {
                            0x02 | 0x03 | 0x04 => depth += 1,
                            0x0B => {
                                depth = depth.saturating_sub(1);
                                if depth == 0 {
                                    break;
                                }
                            }
                            0x41 | 0x42 | 0x23 => {
                                while offset < data.len() && data[offset] & 0x80 != 0 {
                                    offset += 1;
                                }
                                if offset < data.len() {
                                    offset += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    let offset_expr_bytes = data[expr_start..offset].to_vec();

                    // Parse data byte count and data
                    let (data_len, bytes_read) = read_leb128_u32(data, offset)?;
                    offset += bytes_read;

                    if offset + data_len as usize > data.len() {
                        return Err(Error::parse_error("Data segment data exceeds bounds"));
                    }

                    let data_bytes = data[offset..offset + data_len as usize].to_vec();
                    offset += data_len as usize;

                    #[cfg(feature = "std")]
                    eprintln!("[DATA_SECTION] Segment {}: active (mem {}), offset_expr {} bytes, data {} bytes",
                             i, memory_index, offset_expr_bytes.len(), data_bytes.len());

                    PureDataSegment {
                        mode: PureDataMode::Active {
                            memory_index,
                            offset_expr_len: offset_expr_bytes.len() as u32,
                        },
                        offset_expr_bytes,
                        data_bytes,
                    }
                }
                _ => {
                    return Err(Error::parse_error("Invalid data segment tag"));
                }
            };

            // Add segment to module
            #[cfg(feature = "std")]
            self.module.data.push(segment);

            #[cfg(not(feature = "std"))]
            {
                let _ = self.module.data.push(segment);
            }
        }

        #[cfg(feature = "std")]
        eprintln!("[DATA_SECTION] Finished parsing {} data segments, module.data.len()={}",
                 count, self.module.data.len());

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
        #[cfg(feature = "std")]
        eprintln!("[StreamingDecoder::finish] Returning module with {} imports",
            self.module.imports.len());
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
