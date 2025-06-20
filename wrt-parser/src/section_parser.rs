//! Section-specific parsing logic
//!
//! This module handles parsing of individual WebAssembly sections,
//! building up the module incrementally.

use wrt_error::{Error, ErrorCategory, Result, codes};
use wrt_foundation::bounded::BoundedVec;
use crate::{binary_constants, leb128, ParserProvider};
use crate::module_builder::{Module, Function, Local, Global, Export, Import, Element, Data};
use crate::types::{ValueType, FuncType, GlobalType, MemoryType, TableType, Limits, BlockType};
use crate::module_builder::{ExportDesc, ImportDesc};
use crate::expression_parser::ExpressionParser;

/// Section parser that handles individual WebAssembly sections
pub struct SectionParser {
    expr_parser: ExpressionParser,
}

impl SectionParser {
    /// Create a new section parser
    pub fn new() -> Result<Self> {
        Ok(SectionParser {
            expr_parser: ExpressionParser::new(),
        })
    }
    
    /// Parse a section and update the module
    pub fn parse_section(&mut self, section_id: u8, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        match section_id {
            binary_constants::TYPE_SECTION_ID => self.parse_type_section(data, module),
            binary_constants::IMPORT_SECTION_ID => self.parse_import_section(data, module),
            binary_constants::FUNCTION_SECTION_ID => self.parse_function_section(data, module),
            binary_constants::TABLE_SECTION_ID => self.parse_table_section(data, module),
            binary_constants::MEMORY_SECTION_ID => self.parse_memory_section(data, module),
            binary_constants::GLOBAL_SECTION_ID => self.parse_global_section(data, module),
            binary_constants::EXPORT_SECTION_ID => self.parse_export_section(data, module),
            binary_constants::START_SECTION_ID => self.parse_start_section(data, module),
            binary_constants::ELEMENT_SECTION_ID => self.parse_element_section(data, module),
            binary_constants::CODE_SECTION_ID => self.parse_code_section(data, module),
            binary_constants::DATA_SECTION_ID => self.parse_data_section(data, module),
            binary_constants::DATA_COUNT_SECTION_ID => self.parse_data_count_section(data, module),
            binary_constants::CUSTOM_SECTION_ID | _ => self.parse_custom_section(data, module),
        }
    }
    
    /// Parse type section
    fn parse_type_section(&mut self, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            // Parse function type
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unexpected end of data in type section"
                ));
            }
            
            let form = data[offset];
            offset += 1;
            
            if form != 0x60 {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid function type form"
                ));
            }
            
            let provider = ParserProvider::default();
            let mut func_type = FuncType::new(provider)?;
            
            // Parse parameters
            let (param_count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            for _ in 0..param_count {
                if offset >= data.len() {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Unexpected end of data reading parameter types"
                    ));
                }
                
                let value_type = ValueType::from_byte(data[offset])?;
                offset += 1;
                func_type.params.push(value_type)?;
            }
            
            // Parse results
            let (result_count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            for _ in 0..result_count {
                if offset >= data.len() {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Unexpected end of data reading result types"
                    ));
                }
                
                let value_type = ValueType::from_byte(data[offset])?;
                offset += 1;
                func_type.results.push(value_type)?;
            }
            
            module.types.push(func_type)?;
        }
        
        Ok(())
    }
    
    /// Parse import section
    fn parse_import_section(&mut self, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let provider = ParserProvider::default();
            let mut import = Import::new(provider)?;
            
            // Parse module name
            let (name_len, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            if offset + name_len as usize > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Import module name extends beyond section"
                ));
            }
            
            for &byte in &data[offset..offset + name_len as usize] {
                import.module.push(byte)?;
            }
            offset += name_len as usize;
            
            // Parse field name
            let (name_len, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            if offset + name_len as usize > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Import field name extends beyond section"
                ));
            }
            
            for &byte in &data[offset..offset + name_len as usize] {
                import.name.push(byte)?;
            }
            offset += name_len as usize;
            
            // Parse import description
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unexpected end of data reading import description"
                ));
            }
            
            let import_type = data[offset];
            offset += 1;
            
            import.desc = match import_type {
                0x00 => {
                    let (type_idx, bytes_read) = leb128::read_leb128_u32(data, offset)?;
                    offset += bytes_read;
                    ImportDesc::Func(type_idx)
                }
                0x01 => {
                    // Parse table type
                    if offset >= data.len() {
                        return Err(Error::new(
                            ErrorCategory::Parse,
                            codes::PARSE_ERROR,
                            "Unexpected end of data reading table type"
                        ));
                    }
                    
                    let element_type = ValueType::from_byte(data[offset])?;
                    offset += 1;
                    
                    let (limits, bytes_read) = Limits::parse(data, offset)?;
                    offset += bytes_read;
                    
                    ImportDesc::Table(TableType { element_type, limits })
                }
                0x02 => {
                    // Parse memory type
                    let (limits, bytes_read) = Limits::parse(data, offset)?;
                    offset += bytes_read;
                    ImportDesc::Memory(MemoryType { limits })
                }
                0x03 => {
                    // Parse global type
                    if offset + 1 >= data.len() {
                        return Err(Error::new(
                            ErrorCategory::Parse,
                            codes::PARSE_ERROR,
                            "Unexpected end of data reading global type"
                        ));
                    }
                    
                    let value_type = ValueType::from_byte(data[offset])?;
                    offset += 1;
                    
                    let mutable = data[offset] != 0;
                    offset += 1;
                    
                    ImportDesc::Global(GlobalType { value_type, mutable })
                }
                _ => return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid import description type"
                )),
            };
            
            module.imports.push(import)?;
        }
        
        Ok(())
    }
    
    /// Parse function section
    fn parse_function_section(&mut self, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let (type_idx, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            let provider = ParserProvider::default();
            let function = Function::new(type_idx, provider)?;
            module.functions.push(function)?;
        }
        
        Ok(())
    }
    
    /// Parse table section
    fn parse_table_section(&mut self, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unexpected end of data reading table type"
                ));
            }
            
            let element_type = ValueType::from_byte(data[offset])?;
            offset += 1;
            
            let (limits, bytes_read) = Limits::parse(data, offset)?;
            offset += bytes_read;
            
            let table = crate::module_builder::Table {
                table_type: TableType { element_type, limits },
            };
            
            module.tables.push(table)?;
        }
        
        Ok(())
    }
    
    /// Parse memory section
    fn parse_memory_section(&mut self, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let (limits, bytes_read) = Limits::parse(data, offset)?;
            offset += bytes_read;
            
            let memory = crate::module_builder::Memory {
                memory_type: MemoryType { limits },
            };
            
            module.memories.push(memory)?;
        }
        
        Ok(())
    }
    
    /// Parse global section
    fn parse_global_section(&mut self, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            if offset + 1 >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unexpected end of data reading global type"
                ));
            }
            
            let value_type = ValueType::from_byte(data[offset])?;
            offset += 1;
            
            let mutable = data[offset] != 0;
            offset += 1;
            
            let global_type = GlobalType { value_type, mutable };
            
            // Parse init expression
            let bytes_consumed = self.expr_parser.skip_const_expr(data, offset)?;
            offset += bytes_consumed;
            
            let provider = ParserProvider::default();
            let global = Global::new(global_type, provider)?;
            module.globals.push(global)?;
        }
        
        Ok(())
    }
    
    /// Parse export section
    fn parse_export_section(&mut self, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let provider = ParserProvider::default();
            let mut export = Export::new(provider)?;
            
            // Parse export name
            let (name_len, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            if offset + name_len as usize > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Export name extends beyond section"
                ));
            }
            
            for &byte in &data[offset..offset + name_len as usize] {
                export.name.push(byte)?;
            }
            offset += name_len as usize;
            
            // Parse export description
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unexpected end of data reading export description"
                ));
            }
            
            let export_type = data[offset];
            offset += 1;
            
            let (index, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            export.desc = match export_type {
                0x00 => ExportDesc::Func(index),
                0x01 => ExportDesc::Table(index),
                0x02 => ExportDesc::Memory(index),
                0x03 => ExportDesc::Global(index),
                _ => return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid export description type"
                )),
            };
            
            module.exports.push(export)?;
        }
        
        Ok(())
    }
    
    /// Parse start section
    fn parse_start_section(&mut self, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        let (start_idx, _) = leb128::read_leb128_u32(data, 0)?;
        module.start = Some(start_idx);
        Ok(())
    }
    
    /// Parse element section
    fn parse_element_section(&mut self, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let provider = ParserProvider::default();
            let mut element = Element::new(provider)?;
            
            // Parse table index
            let (table_idx, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            element.table_index = table_idx;
            
            // Parse offset expression
            let bytes_consumed = self.expr_parser.skip_const_expr(data, offset)?;
            offset += bytes_consumed;
            
            // Parse function indices
            let (elem_count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            for _ in 0..elem_count {
                let (func_idx, bytes_read) = leb128::read_leb128_u32(data, offset)?;
                offset += bytes_read;
                element.init.push(func_idx)?;
            }
            
            module.elements.push(element)?;
        }
        
        Ok(())
    }
    
    /// Parse code section
    fn parse_code_section(&mut self, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        for i in 0..count {
            let (body_size, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            let body_end = offset + body_size as usize;
            if body_end > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Function body extends beyond section"
                ));
            }
            
            // Parse locals count
            let mut body_offset = offset;
            let (locals_count, bytes_read) = leb128::read_leb128_u32(data, body_offset)?;
            body_offset += bytes_read;
            
            // Parse locals
            if let Some(func) = module.functions.get_mut(i as usize) {
                for _ in 0..locals_count {
                    let (count, bytes_read) = leb128::read_leb128_u32(data, body_offset)?;
                    body_offset += bytes_read;
                    
                    if body_offset >= data.len() {
                        return Err(Error::new(
                            ErrorCategory::Parse,
                            codes::PARSE_ERROR,
                            "Unexpected end of data reading local type"
                        ));
                    }
                    
                    let value_type = ValueType::from_byte(data[body_offset])?;
                    body_offset += 1;
                    
                    let local = Local { count, value_type };
                    func.locals.push(local)?;
                }
                
                // Copy the code body
                for &byte in &data[body_offset..body_end] {
                    func.code.push(byte)?;
                }
            }
            
            offset = body_end;
        }
        
        Ok(())
    }
    
    /// Parse data section
    fn parse_data_section(&mut self, data: &[u8], module: &mut Module<ParserProvider>) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        for _ in 0..count {
            let provider = ParserProvider::default();
            let mut data_segment = Data::new(provider)?;
            
            // Parse memory index
            let (mem_idx, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            data_segment.memory_index = mem_idx;
            
            // Parse offset expression
            let bytes_consumed = self.expr_parser.skip_const_expr(data, offset)?;
            offset += bytes_consumed;
            
            // Parse data bytes
            let (data_len, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            if offset + data_len as usize > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Data segment extends beyond section"
                ));
            }
            
            for &byte in &data[offset..offset + data_len as usize] {
                data_segment.data.push(byte)?;
            }
            offset += data_len as usize;
            
            module.data.push(data_segment)?;
        }
        
        Ok(())
    }
    
    /// Parse data count section
    fn parse_data_count_section(&mut self, _data: &[u8], _module: &mut Module<ParserProvider>) -> Result<()> {
        // Used for validation only
        Ok(())
    }
    
    /// Parse custom section
    fn parse_custom_section(&mut self, _data: &[u8], _module: &mut Module<ParserProvider>) -> Result<()> {
        // Skip custom sections or process specific ones
        Ok(())
    }
    
}