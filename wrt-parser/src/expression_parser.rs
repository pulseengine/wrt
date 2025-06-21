//! WebAssembly constant expression parsing
//!
//! This module handles parsing and evaluation of constant expressions
//! used in global initializers, element offsets, and data offsets.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::leb128;
use crate::types::ValueType;
use crate::bounded_types::SimpleBoundedVec;

/// A constant expression value
#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    FuncRef(u32),
    RefNull(ValueType),
    GlobalRef(u32),
}

/// A constant expression AST node
#[derive(Debug, Clone, PartialEq)]
pub enum ConstExpr {
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    GlobalGet(u32),
    RefNull(ValueType),
    RefFunc(u32),
}

/// Constant expression parser
#[derive(Debug)]
pub struct ExpressionParser;

impl ExpressionParser {
    /// Create a new expression parser
    pub fn new() -> Self {
        ExpressionParser
    }
    
    /// Parse a constant expression from binary data
    /// Returns the parsed expression and the number of bytes consumed
    pub fn parse_const_expr(&self, data: &[u8], mut offset: usize) -> Result<(ConstExpr, usize)> {
        let start_offset = offset;
        
        if offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end of data in constant expression"
            ));
        }
        
        let opcode = data[offset];
        offset += 1;
        
        let expr = match opcode {
            0x41 => { // i32.const
                let (value, bytes) = leb128::read_leb128_i32(data, offset)?;
                offset += bytes;
                ConstExpr::I32Const(value)
            }
            0x42 => { // i64.const
                let (value, bytes) = leb128::read_leb128_i64(data, offset)?;
                offset += bytes;
                ConstExpr::I64Const(value)
            }
            0x43 => { // f32.const
                if offset + 4 > data.len() {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "f32.const operand extends beyond data"
                    ));
                }
                let bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
                let value = f32::from_le_bytes(bytes);
                offset += 4;
                ConstExpr::F32Const(value)
            }
            0x44 => { // f64.const
                if offset + 8 > data.len() {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "f64.const operand extends beyond data"
                    ));
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[offset..offset + 8]);
                let value = f64::from_le_bytes(bytes);
                offset += 8;
                ConstExpr::F64Const(value)
            }
            0x23 => { // global.get
                let (global_idx, bytes) = leb128::read_leb128_u32(data, offset)?;
                offset += bytes;
                ConstExpr::GlobalGet(global_idx)
            }
            0xD0 => { // ref.null
                if offset >= data.len() {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "ref.null heap type extends beyond data"
                    ));
                }
                let heap_type = ValueType::from_byte(data[offset])?;
                offset += 1;
                ConstExpr::RefNull(heap_type)
            }
            0xD2 => { // ref.func
                let (func_idx, bytes) = leb128::read_leb128_u32(data, offset)?;
                offset += bytes;
                ConstExpr::RefFunc(func_idx)
            }
            _ => return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Invalid opcode in constant expression"
            )),
        };
        
        // Expect end opcode
        if offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Expected 'end' opcode in constant expression"
            ));
        }
        
        if data[offset] != 0x0B {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Expected 'end' opcode in constant expression"
            ));
        }
        offset += 1;
        
        Ok((expr, offset - start_offset))
    }
    
    /// Skip a constant expression without parsing it
    /// Returns the number of bytes consumed
    pub fn skip_const_expr(&self, data: &[u8], mut offset: usize) -> Result<usize> {
        let start_offset = offset;
        
        loop {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unexpected end of data in constant expression"
                ));
            }
            
            let opcode = data[offset];
            offset += 1;
            
            match opcode {
                0x0B => break, // end
                0x41 => { // i32.const
                    let (_, bytes) = leb128::read_leb128_i32(data, offset)?;
                    offset += bytes;
                }
                0x42 => { // i64.const
                    let (_, bytes) = leb128::read_leb128_i64(data, offset)?;
                    offset += bytes;
                }
                0x43 => { // f32.const
                    if offset + 4 > data.len() {
                        return Err(Error::new(
                            ErrorCategory::Parse,
                            codes::PARSE_ERROR,
                            "f32.const operand extends beyond data"
                        ));
                    }
                    offset += 4;
                }
                0x44 => { // f64.const
                    if offset + 8 > data.len() {
                        return Err(Error::new(
                            ErrorCategory::Parse,
                            codes::PARSE_ERROR,
                            "f64.const operand extends beyond data"
                        ));
                    }
                    offset += 8;
                }
                0x23 => { // global.get
                    let (_, bytes) = leb128::read_leb128_u32(data, offset)?;
                    offset += bytes;
                }
                0xD0 => { // ref.null
                    if offset >= data.len() {
                        return Err(Error::new(
                            ErrorCategory::Parse,
                            codes::PARSE_ERROR,
                            "ref.null heap type extends beyond data"
                        ));
                    }
                    offset += 1; // heap type
                }
                0xD2 => { // ref.func
                    let (_, bytes) = leb128::read_leb128_u32(data, offset)?;
                    offset += bytes;
                }
                _ => return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid opcode in constant expression"
                )),
            }
        }
        
        Ok(offset - start_offset)
    }
    
    /// Evaluate a constant expression to get its value
    /// This requires access to global values for global.get expressions
    pub fn evaluate(&self, expr: &ConstExpr, _globals: &[ConstValue]) -> Result<ConstValue> {
        match expr {
            ConstExpr::I32Const(value) => Ok(ConstValue::I32(*value)),
            ConstExpr::I64Const(value) => Ok(ConstValue::I64(*value)),
            ConstExpr::F32Const(value) => Ok(ConstValue::F32(*value)),
            ConstExpr::F64Const(value) => Ok(ConstValue::F64(*value)),
            ConstExpr::RefNull(heap_type) => Ok(ConstValue::RefNull(*heap_type)),
            ConstExpr::RefFunc(func_idx) => Ok(ConstValue::FuncRef(*func_idx)),
            ConstExpr::GlobalGet(global_idx) => {
                // For now, return a placeholder - full evaluation would need global values
                Ok(ConstValue::GlobalRef(*global_idx))
            }
        }
    }
}

impl Default for ExpressionParser {
    fn default() -> Self {
        Self::new()
    }
}

/// A bounded vector for storing constant expressions
pub type BoundedConstExprs<const N: usize> = SimpleBoundedVec<ConstExpr, N>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_i32_const() {
        let parser = ExpressionParser::new();
        
        // i32.const 42, end
        let data = [0x41, 0x2A, 0x0B];
        let (expr, bytes_consumed) = parser.parse_const_expr(&data, 0).unwrap();
        
        assert_eq!(expr, ConstExpr::I32Const(42));
        assert_eq!(bytes_consumed, 3);
    }
    
    #[test]
    fn test_parse_global_get() {
        let parser = ExpressionParser::new();
        
        // global.get 0, end
        let data = [0x23, 0x00, 0x0B];
        let (expr, bytes_consumed) = parser.parse_const_expr(&data, 0).unwrap();
        
        assert_eq!(expr, ConstExpr::GlobalGet(0));
        assert_eq!(bytes_consumed, 3);
    }
    
    #[test]
    fn test_skip_const_expr() {
        let parser = ExpressionParser::new();
        
        // i32.const 42, end
        let data = [0x41, 0x2A, 0x0B];
        let bytes_consumed = parser.skip_const_expr(&data, 0).unwrap();
        
        assert_eq!(bytes_consumed, 3);
    }
    
    #[test]
    fn test_invalid_opcode() {
        let parser = ExpressionParser::new();
        
        // Invalid opcode 0xFF, end
        let data = [0xFF, 0x0B];
        let result = parser.parse_const_expr(&data, 0);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_missing_end() {
        let parser = ExpressionParser::new();
        
        // i32.const 42 (missing end)
        let data = [0x41, 0x2A];
        let result = parser.parse_const_expr(&data, 0);
        
        assert!(result.is_err());
    }
}