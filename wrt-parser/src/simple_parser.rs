//! Simple streaming WebAssembly parser
//!
//! This module provides a complete WebAssembly parser that processes
//! binaries section by section with bounded memory usage.

use crate::binary_constants::*;
use crate::leb128;
use crate::simple_module::*;
use crate::types::{ValueType, FuncType, GlobalType, MemoryType, TableType, Limits};
use crate::bounded_types::SimpleBoundedVec;
use crate::expression_parser::ExpressionParser;
use crate::validation::{ModuleValidator, ValidationConfig};
use wrt_error::{Error, ErrorCategory, Result, codes};

/// Simple WebAssembly parser
#[derive(Debug)]
pub struct SimpleParser {
    module: SimpleModule,
    expr_parser: ExpressionParser,
    validator: ModuleValidator,
    validate_on_parse: bool,
}

impl SimpleParser {
    /// Create a new parser
    pub fn new() -> Self {
        SimpleParser {
            module: SimpleModule::new(),
            expr_parser: ExpressionParser::new(),
            validator: ModuleValidator::new(ValidationConfig::default()),
            validate_on_parse: true,
        }
    }
    
    /// Create a new parser with custom validation config
    pub fn with_validation(config: ValidationConfig) -> Self {
        SimpleParser {
            module: SimpleModule::new(),
            expr_parser: ExpressionParser::new(),
            validator: ModuleValidator::new(config),
            validate_on_parse: true,
        }
    }
    
    /// Create a new parser without validation
    pub fn without_validation() -> Self {
        SimpleParser {
            module: SimpleModule::new(),
            expr_parser: ExpressionParser::new(),
            validator: ModuleValidator::new(ValidationConfig::default()),
            validate_on_parse: false,
        }
    }
    
    /// Parse a complete WebAssembly binary
    pub fn parse(&mut self, binary: &[u8]) -> Result<SimpleModule> {
        let mut offset = 0;
        
        // Validate header
        offset = self.parse_header(binary, offset)?;
        
        // Parse sections
        while offset < binary.len() {
            offset = self.parse_section(binary, offset)?;
        }
        
        // Validate the parsed module if validation is enabled
        if self.validate_on_parse {
            self.validator.validate(&self.module)?;
        }
        
        Ok(core::mem::take(&mut self.module))
    }
    
    /// Parse and validate the WebAssembly header
    fn parse_header(&self, binary: &[u8], offset: usize) -> Result<usize> {
        if binary.len() < offset + 8 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Binary too small for WebAssembly header"
            ));
        }
        
        // Check magic number
        if &binary[offset..offset + 4] != &WASM_MAGIC {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Invalid WebAssembly magic number"
            ));
        }
        
        // Check version
        if &binary[offset + 4..offset + 8] != &WASM_VERSION {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unsupported WebAssembly version"
            ));
        }
        
        Ok(offset + 8)
    }
    
    /// Parse a single section
    fn parse_section(&mut self, binary: &[u8], mut offset: usize) -> Result<usize> {
        if offset >= binary.len() {
            return Ok(offset);
        }
        
        // Read section ID
        let section_id = binary[offset];
        offset += 1;
        
        // Read section size
        let (section_size, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
        offset += bytes_read;
        
        let section_end = offset + section_size as usize;
        if section_end > binary.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Section extends beyond binary"
            ));
        }
        
        // Parse section based on ID
        match section_id {
            TYPE_SECTION_ID => self.parse_type_section(binary, offset, section_end)?,
            IMPORT_SECTION_ID => self.parse_import_section(binary, offset, section_end)?,
            FUNCTION_SECTION_ID => self.parse_function_section(binary, offset, section_end)?,
            TABLE_SECTION_ID => self.parse_table_section(binary, offset, section_end)?,
            MEMORY_SECTION_ID => self.parse_memory_section(binary, offset, section_end)?,
            GLOBAL_SECTION_ID => self.parse_global_section(binary, offset, section_end)?,
            EXPORT_SECTION_ID => self.parse_export_section(binary, offset, section_end)?,
            START_SECTION_ID => self.parse_start_section(binary, offset, section_end)?,
            ELEMENT_SECTION_ID => self.parse_element_section(binary, offset, section_end)?,
            CODE_SECTION_ID => self.parse_code_section(binary, offset, section_end)?,
            DATA_SECTION_ID => self.parse_data_section(binary, offset, section_end)?,
            DATA_COUNT_SECTION_ID => self.parse_data_count_section(binary, offset, section_end)?,
            CUSTOM_SECTION_ID | _ => {}, // Skip custom sections
        }
        
        Ok(section_end)
    }
    
    /// Parse type section
    fn parse_type_section(&mut self, binary: &[u8], mut offset: usize, _section_end: usize) -> Result<()> {
        let (count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            // Check function type marker
            if offset >= binary.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unexpected end of type section"
                ));
            }
            
            let form = binary[offset];
            offset += 1;
            
            if form != 0x60 {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid function type form"
                ));
            }
            
            let mut func_type = FuncType::default();
            
            // Parse parameters
            let (param_count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            
            for _ in 0..param_count {
                if offset >= binary.len() {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Unexpected end of parameter types"
                    ));
                }
                
                let value_type = ValueType::from_byte(binary[offset])?;
                offset += 1;
                func_type.params.push(value_type)?;
            }
            
            // Parse results
            let (result_count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            
            for _ in 0..result_count {
                if offset >= binary.len() {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Unexpected end of result types"
                    ));
                }
                
                let value_type = ValueType::from_byte(binary[offset])?;
                offset += 1;
                func_type.results.push(value_type)?;
            }
            
            self.module.types.push(func_type)?;
        }
        
        Ok(())
    }
    
    /// Parse import section
    fn parse_import_section(&mut self, binary: &[u8], mut offset: usize, _section_end: usize) -> Result<()> {
        let (count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let mut import = Import {
                module: SimpleBoundedVec::new(),
                name: SimpleBoundedVec::new(),
                desc: ImportDesc::Func(0),
            };
            
            // Parse module name
            let (name_len, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            
            if offset + name_len as usize > binary.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Import module name extends beyond section"
                ));
            }
            
            for i in 0..name_len as usize {
                import.module.push(binary[offset + i])?;
            }
            offset += name_len as usize;
            
            // Parse field name
            let (name_len, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            
            if offset + name_len as usize > binary.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Import field name extends beyond section"
                ));
            }
            
            for i in 0..name_len as usize {
                import.name.push(binary[offset + i])?;
            }
            offset += name_len as usize;
            
            // Parse import kind
            if offset >= binary.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unexpected end of import section"
                ));
            }
            
            let kind = binary[offset];
            offset += 1;
            
            import.desc = match kind {
                0x00 => {
                    // Function import
                    let (type_idx, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
                    offset += bytes_read;
                    ImportDesc::Func(type_idx)
                }
                0x01 => {
                    // Table import
                    let elem_type = ValueType::from_byte(binary[offset])?;
                    offset += 1;
                    let (limits, bytes_consumed) = Limits::parse(binary, offset)?;
                    offset += bytes_consumed;
                    ImportDesc::Table(TableType {
                        element_type: elem_type,
                        limits,
                    })
                }
                0x02 => {
                    // Memory import
                    let (limits, bytes_consumed) = Limits::parse(binary, offset)?;
                    offset += bytes_consumed;
                    ImportDesc::Memory(MemoryType { limits })
                }
                0x03 => {
                    // Global import
                    let value_type = ValueType::from_byte(binary[offset])?;
                    offset += 1;
                    let mutable = binary[offset] != 0;
                    offset += 1;
                    ImportDesc::Global(GlobalType { value_type, mutable })
                }
                _ => return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid import kind"
                )),
            };
            
            self.module.imports.push(import)?;
        }
        
        Ok(())
    }
    
    /// Parse function section
    fn parse_function_section(&mut self, binary: &[u8], mut offset: usize, _section_end: usize) -> Result<()> {
        let (count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let (type_idx, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            self.module.functions.push(type_idx)?;
        }
        
        Ok(())
    }
    
    /// Parse table section
    fn parse_table_section(&mut self, binary: &[u8], mut offset: usize, _section_end: usize) -> Result<()> {
        let (count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let elem_type = ValueType::from_byte(binary[offset])?;
            offset += 1;
            let (limits, bytes_consumed) = Limits::parse(binary, offset)?;
            offset += bytes_consumed;
            
            let table_type = TableType {
                element_type: elem_type,
                limits,
            };
            
            self.module.tables.push(table_type)?;
        }
        
        Ok(())
    }
    
    /// Parse memory section
    fn parse_memory_section(&mut self, binary: &[u8], mut offset: usize, _section_end: usize) -> Result<()> {
        let (count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let (limits, bytes_consumed) = Limits::parse(binary, offset)?;
            offset += bytes_consumed;
            
            let memory_type = MemoryType { limits };
            self.module.memories.push(memory_type)?;
        }
        
        Ok(())
    }
    
    /// Parse global section
    fn parse_global_section(&mut self, binary: &[u8], mut offset: usize, _section_end: usize) -> Result<()> {
        let (count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let value_type = ValueType::from_byte(binary[offset])?;
            offset += 1;
            let mutable = binary[offset] != 0;
            offset += 1;
            
            let global_type = GlobalType { value_type, mutable };
            
            // Parse the initialization expression
            let bytes_consumed = self.expr_parser.skip_const_expr(binary, offset)?;
            offset += bytes_consumed;
            
            self.module.globals.push(global_type)?;
        }
        
        Ok(())
    }
    
    /// Parse export section
    fn parse_export_section(&mut self, binary: &[u8], mut offset: usize, _section_end: usize) -> Result<()> {
        let (count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let mut export = Export {
                name: SimpleBoundedVec::new(),
                kind: ExportKind::Func,
                index: 0,
            };
            
            // Parse export name
            let (name_len, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            
            if offset + name_len as usize > binary.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Export name extends beyond section"
                ));
            }
            
            for i in 0..name_len as usize {
                export.name.push(binary[offset + i])?;
            }
            offset += name_len as usize;
            
            // Parse export kind
            if offset >= binary.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unexpected end of export section"
                ));
            }
            
            let kind = binary[offset];
            offset += 1;
            
            export.kind = match kind {
                0x00 => ExportKind::Func,
                0x01 => ExportKind::Table,
                0x02 => ExportKind::Memory,
                0x03 => ExportKind::Global,
                _ => return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid export kind"
                )),
            };
            
            // Parse export index
            let (index, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            export.index = index;
            
            self.module.exports.push(export)?;
        }
        
        Ok(())
    }
    
    /// Parse start section
    fn parse_start_section(&mut self, binary: &[u8], offset: usize, _section_end: usize) -> Result<()> {
        let (start_idx, _) = leb128::read_leb128_u32(binary, offset)?;
        self.module.start = Some(start_idx);
        Ok(())
    }
    
    /// Parse element section
    fn parse_element_section(&mut self, binary: &[u8], mut offset: usize, _section_end: usize) -> Result<()> {
        let (count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let mut elem = ElementSegment {
                table_index: 0,
                offset: SimpleBoundedVec::new(),
                init: SimpleBoundedVec::new(),
            };
            
            // Parse table index
            let (table_idx, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            elem.table_index = table_idx;
            
            // Parse the offset expression
            let bytes_consumed = self.expr_parser.skip_const_expr(binary, offset)?;
            offset += bytes_consumed;
            
            // Parse function indices
            let (elem_count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            
            for _ in 0..elem_count {
                let (func_idx, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
                offset += bytes_read;
                elem.init.push(func_idx)?;
            }
            
            self.module.elements.push(elem)?;
        }
        
        Ok(())
    }
    
    /// Parse code section
    fn parse_code_section(&mut self, binary: &[u8], mut offset: usize, _section_end: usize) -> Result<()> {
        let (count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let (body_size, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            
            let body_end = offset + body_size as usize;
            if body_end > binary.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Function body extends beyond section"
                ));
            }
            
            let mut func_body = FunctionBody {
                locals: SimpleBoundedVec::new(),
                code: SimpleBoundedVec::new(),
            };
            
            // Parse locals
            let (local_count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            
            for _ in 0..local_count {
                let (count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
                offset += bytes_read;
                
                let value_type = ValueType::from_byte(binary[offset])?;
                offset += 1;
                
                func_body.locals.push(LocalDecl { count, value_type })?;
            }
            
            // Copy the code bytes
            while offset < body_end {
                func_body.code.push(binary[offset])?;
                offset += 1;
            }
            
            self.module.code.push(func_body)?;
        }
        
        Ok(())
    }
    
    /// Parse data section
    fn parse_data_section(&mut self, binary: &[u8], mut offset: usize, _section_end: usize) -> Result<()> {
        let (count, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let mut data = DataSegment {
                memory_index: 0,
                offset: SimpleBoundedVec::new(),
                data: SimpleBoundedVec::new(),
            };
            
            // Parse memory index
            let (mem_idx, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            data.memory_index = mem_idx;
            
            // Parse the offset expression
            let bytes_consumed = self.expr_parser.skip_const_expr(binary, offset)?;
            offset += bytes_consumed;
            
            // Parse data bytes
            let (data_len, bytes_read) = leb128::read_leb128_u32(binary, offset)?;
            offset += bytes_read;
            
            if offset + data_len as usize > binary.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Data segment extends beyond section"
                ));
            }
            
            for i in 0..data_len as usize {
                data.data.push(binary[offset + i])?;
            }
            offset += data_len as usize;
            
            self.module.data.push(data)?;
        }
        
        Ok(())
    }
    
    /// Parse data count section
    fn parse_data_count_section(&mut self, _binary: &[u8], _offset: usize, _section_end: usize) -> Result<()> {
        // This section is used for validation only
        Ok(())
    }
}

impl Default for SimpleParser {
    fn default() -> Self {
        Self::new()
    }
}