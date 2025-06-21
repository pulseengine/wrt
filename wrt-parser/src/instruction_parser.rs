//! WebAssembly instruction parsing and bytecode analysis
//!
//! This module provides comprehensive parsing of WebAssembly instructions
//! with full instruction set support, control flow analysis, and ASIL-D
//! compliant memory management.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::{binary_constants, leb128};
use crate::types::ValueType;
use crate::bounded_types::{SimpleBoundedVec, SimpleBoundedString};

/// Maximum depth for control flow nesting
pub const MAX_CONTROL_DEPTH: usize = 64;

/// Maximum number of branch targets in br_table
pub const MAX_BRANCH_TARGETS: usize = 256;

/// Maximum number of locals in a function
pub const MAX_FUNCTION_LOCALS: usize = 1024;

/// WebAssembly instruction opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // Control instructions
    Unreachable = 0x00,
    Nop = 0x01,
    Block = 0x02,
    Loop = 0x03,
    If = 0x04,
    Else = 0x05,
    End = 0x0B,
    Br = 0x0C,
    BrIf = 0x0D,
    BrTable = 0x0E,
    Return = 0x0F,
    Call = 0x10,
    CallIndirect = 0x11,
    
    // Parametric instructions
    Drop = 0x1A,
    Select = 0x1B,
    SelectT = 0x1C,
    
    // Variable instructions
    LocalGet = 0x20,
    LocalSet = 0x21,
    LocalTee = 0x22,
    GlobalGet = 0x23,
    GlobalSet = 0x24,
    
    // Table instructions
    TableGet = 0x25,
    TableSet = 0x26,
    
    // Memory instructions
    I32Load = 0x28,
    I64Load = 0x29,
    F32Load = 0x2A,
    F64Load = 0x2B,
    I32Load8S = 0x2C,
    I32Load8U = 0x2D,
    I32Load16S = 0x2E,
    I32Load16U = 0x2F,
    I64Load8S = 0x30,
    I64Load8U = 0x31,
    I64Load16S = 0x32,
    I64Load16U = 0x33,
    I64Load32S = 0x34,
    I64Load32U = 0x35,
    I32Store = 0x36,
    I64Store = 0x37,
    F32Store = 0x38,
    F64Store = 0x39,
    I32Store8 = 0x3A,
    I32Store16 = 0x3B,
    I64Store8 = 0x3C,
    I64Store16 = 0x3D,
    I64Store32 = 0x3E,
    MemorySize = 0x3F,
    MemoryGrow = 0x40,
    
    // Numeric instructions
    I32Const = 0x41,
    I64Const = 0x42,
    F32Const = 0x43,
    F64Const = 0x44,
    
    // Comparison instructions
    I32Eqz = 0x45,
    I32Eq = 0x46,
    I32Ne = 0x47,
    I32LtS = 0x48,
    I32LtU = 0x49,
    I32GtS = 0x4A,
    I32GtU = 0x4B,
    I32LeS = 0x4C,
    I32LeU = 0x4D,
    I32GeS = 0x4E,
    I32GeU = 0x4F,
    
    // Extended opcodes (0xFC prefix)
    ExtendedFC = 0xFC,
    
    // Reference instructions
    RefNull = 0xD0,
    RefIsNull = 0xD1,
    RefFunc = 0xD2,
}

impl Opcode {
    /// Convert byte to opcode
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Opcode::Unreachable),
            0x01 => Some(Opcode::Nop),
            0x02 => Some(Opcode::Block),
            0x03 => Some(Opcode::Loop),
            0x04 => Some(Opcode::If),
            0x05 => Some(Opcode::Else),
            0x0B => Some(Opcode::End),
            0x0C => Some(Opcode::Br),
            0x0D => Some(Opcode::BrIf),
            0x0E => Some(Opcode::BrTable),
            0x0F => Some(Opcode::Return),
            0x10 => Some(Opcode::Call),
            0x11 => Some(Opcode::CallIndirect),
            0x1A => Some(Opcode::Drop),
            0x1B => Some(Opcode::Select),
            0x1C => Some(Opcode::SelectT),
            0x20 => Some(Opcode::LocalGet),
            0x21 => Some(Opcode::LocalSet),
            0x22 => Some(Opcode::LocalTee),
            0x23 => Some(Opcode::GlobalGet),
            0x24 => Some(Opcode::GlobalSet),
            0x25 => Some(Opcode::TableGet),
            0x26 => Some(Opcode::TableSet),
            0x28 => Some(Opcode::I32Load),
            0x29 => Some(Opcode::I64Load),
            0x2A => Some(Opcode::F32Load),
            0x2B => Some(Opcode::F64Load),
            0x2C => Some(Opcode::I32Load8S),
            0x2D => Some(Opcode::I32Load8U),
            0x2E => Some(Opcode::I32Load16S),
            0x2F => Some(Opcode::I32Load16U),
            0x30 => Some(Opcode::I64Load8S),
            0x31 => Some(Opcode::I64Load8U),
            0x32 => Some(Opcode::I64Load16S),
            0x33 => Some(Opcode::I64Load16U),
            0x34 => Some(Opcode::I64Load32S),
            0x35 => Some(Opcode::I64Load32U),
            0x36 => Some(Opcode::I32Store),
            0x37 => Some(Opcode::I64Store),
            0x38 => Some(Opcode::F32Store),
            0x39 => Some(Opcode::F64Store),
            0x3A => Some(Opcode::I32Store8),
            0x3B => Some(Opcode::I32Store16),
            0x3C => Some(Opcode::I64Store8),
            0x3D => Some(Opcode::I64Store16),
            0x3E => Some(Opcode::I64Store32),
            0x3F => Some(Opcode::MemorySize),
            0x40 => Some(Opcode::MemoryGrow),
            0x41 => Some(Opcode::I32Const),
            0x42 => Some(Opcode::I64Const),
            0x43 => Some(Opcode::F32Const),
            0x44 => Some(Opcode::F64Const),
            0x45 => Some(Opcode::I32Eqz),
            0x46 => Some(Opcode::I32Eq),
            0x47 => Some(Opcode::I32Ne),
            0x48 => Some(Opcode::I32LtS),
            0x49 => Some(Opcode::I32LtU),
            0x4A => Some(Opcode::I32GtS),
            0x4B => Some(Opcode::I32GtU),
            0x4C => Some(Opcode::I32LeS),
            0x4D => Some(Opcode::I32LeU),
            0x4E => Some(Opcode::I32GeS),
            0x4F => Some(Opcode::I32GeU),
            0xFC => Some(Opcode::ExtendedFC),
            0xD0 => Some(Opcode::RefNull),
            0xD1 => Some(Opcode::RefIsNull),
            0xD2 => Some(Opcode::RefFunc),
            _ => None,
        }
    }
    
    /// Check if opcode is a control instruction
    pub fn is_control(&self) -> bool {
        matches!(self, 
            Opcode::Block | Opcode::Loop | Opcode::If | Opcode::Else | 
            Opcode::End | Opcode::Br | Opcode::BrIf | Opcode::BrTable | 
            Opcode::Return | Opcode::Call | Opcode::CallIndirect
        )
    }
    
    /// Check if opcode starts a block
    pub fn is_block_start(&self) -> bool {
        matches!(self, Opcode::Block | Opcode::Loop | Opcode::If)
    }
    
    /// Check if opcode requires immediate values
    pub fn has_immediates(&self) -> bool {
        matches!(self,
            Opcode::Block | Opcode::Loop | Opcode::If | Opcode::Br | 
            Opcode::BrIf | Opcode::BrTable | Opcode::Call | 
            Opcode::CallIndirect | Opcode::LocalGet | Opcode::LocalSet | 
            Opcode::LocalTee | Opcode::GlobalGet | Opcode::GlobalSet |
            Opcode::I32Load | Opcode::I64Load | Opcode::F32Load | 
            Opcode::F64Load | Opcode::I32Store | Opcode::I64Store |
            Opcode::F32Store | Opcode::F64Store | Opcode::I32Const | 
            Opcode::I64Const | Opcode::F32Const | Opcode::F64Const |
            Opcode::RefNull | Opcode::RefFunc
        )
    }
}

/// Memory operation alignment and offset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

/// Branch table data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrTable {
    pub targets: SimpleBoundedVec<u32, MAX_BRANCH_TARGETS>,
    pub default_target: u32,
}

/// Control flow block type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Empty,
    Value(ValueType),
    Type(u32), // Type index
}

/// Parsed WebAssembly instruction
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Control instructions
    Unreachable,
    Nop,
    Block { block_type: BlockType },
    Loop { block_type: BlockType },
    If { block_type: BlockType },
    Else,
    End,
    Br { label_idx: u32 },
    BrIf { label_idx: u32 },
    BrTable { table: BrTable },
    Return,
    Call { func_idx: u32 },
    CallIndirect { type_idx: u32, table_idx: u32 },
    
    // Parametric instructions
    Drop,
    Select,
    SelectT { types: SimpleBoundedVec<ValueType, 16> },
    
    // Variable instructions
    LocalGet { local_idx: u32 },
    LocalSet { local_idx: u32 },
    LocalTee { local_idx: u32 },
    GlobalGet { global_idx: u32 },
    GlobalSet { global_idx: u32 },
    
    // Table instructions
    TableGet { table_idx: u32 },
    TableSet { table_idx: u32 },
    
    // Memory instructions
    I32Load { memarg: MemArg },
    I64Load { memarg: MemArg },
    F32Load { memarg: MemArg },
    F64Load { memarg: MemArg },
    I32Load8S { memarg: MemArg },
    I32Load8U { memarg: MemArg },
    I32Load16S { memarg: MemArg },
    I32Load16U { memarg: MemArg },
    I64Load8S { memarg: MemArg },
    I64Load8U { memarg: MemArg },
    I64Load16S { memarg: MemArg },
    I64Load16U { memarg: MemArg },
    I64Load32S { memarg: MemArg },
    I64Load32U { memarg: MemArg },
    I32Store { memarg: MemArg },
    I64Store { memarg: MemArg },
    F32Store { memarg: MemArg },
    F64Store { memarg: MemArg },
    I32Store8 { memarg: MemArg },
    I32Store16 { memarg: MemArg },
    I64Store8 { memarg: MemArg },
    I64Store16 { memarg: MemArg },
    I64Store32 { memarg: MemArg },
    MemorySize { mem_idx: u32 },
    MemoryGrow { mem_idx: u32 },
    
    // Numeric instructions
    I32Const { value: i32 },
    I64Const { value: i64 },
    F32Const { value: f32 },
    F64Const { value: f64 },
    
    // Comparison instructions
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    
    // Reference instructions
    RefNull { ref_type: ValueType },
    RefIsNull,
    RefFunc { func_idx: u32 },
}

/// Control flow frame for validation
#[derive(Debug, Clone)]
pub struct ControlFrame {
    pub opcode: Opcode,
    pub block_type: BlockType,
    pub start_types: SimpleBoundedVec<ValueType, 32>,
    pub end_types: SimpleBoundedVec<ValueType, 32>,
    pub height: usize,
    pub unreachable: bool,
}

/// WebAssembly instruction parser with validation
#[derive(Debug)]
pub struct InstructionParser {
    /// Control flow stack for validation
    control_stack: SimpleBoundedVec<ControlFrame, MAX_CONTROL_DEPTH>,
    
    /// Value type stack for validation
    value_stack: SimpleBoundedVec<ValueType, 256>,
    
    /// Local variable types
    locals: SimpleBoundedVec<ValueType, MAX_FUNCTION_LOCALS>,
}

impl InstructionParser {
    /// Create a new instruction parser
    pub fn new() -> Self {
        Self {
            control_stack: SimpleBoundedVec::new(),
            value_stack: SimpleBoundedVec::new(),
            locals: SimpleBoundedVec::new(),
        }
    }
    
    /// Initialize parser for a function with given locals
    pub fn init_function(&mut self, locals: &[ValueType]) -> Result<()> {
        self.control_stack.clear();
        self.value_stack.clear();
        self.locals.clear();
        
        // Add function parameters and locals
        for &local_type in locals {
            self.locals.push(local_type)?;
        }
        
        // Push function control frame
        let frame = ControlFrame {
            opcode: Opcode::Block, // Function acts like a block
            block_type: BlockType::Empty,
            start_types: SimpleBoundedVec::new(),
            end_types: SimpleBoundedVec::new(),
            height: 0,
            unreachable: false,
        };
        
        self.control_stack.push(frame)?;
        Ok(())
    }
    
    /// Parse a single instruction from bytecode
    pub fn parse_instruction(&mut self, data: &[u8], offset: usize) -> Result<(Instruction, usize)> {
        if offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end of bytecode"
            ));
        }
        
        let opcode_byte = data[offset];
        let opcode = Opcode::from_byte(opcode_byte)
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_INVALID_OPCODE_BYTE,
                "Unknown instruction opcode"
            ))?;
        
        let mut current_offset = offset + 1;
        
        let instruction = match opcode {
            Opcode::Unreachable => Instruction::Unreachable,
            Opcode::Nop => Instruction::Nop,
            
            Opcode::Block => {
                let (block_type, bytes_read) = self.parse_block_type(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::Block { block_type }
            }
            
            Opcode::Loop => {
                let (block_type, bytes_read) = self.parse_block_type(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::Loop { block_type }
            }
            
            Opcode::If => {
                let (block_type, bytes_read) = self.parse_block_type(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::If { block_type }
            }
            
            Opcode::Else => Instruction::Else,
            Opcode::End => Instruction::End,
            
            Opcode::Br => {
                let (label_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::Br { label_idx }
            }
            
            Opcode::BrIf => {
                let (label_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::BrIf { label_idx }
            }
            
            Opcode::BrTable => {
                let (table, bytes_read) = self.parse_br_table(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::BrTable { table }
            }
            
            Opcode::Return => Instruction::Return,
            
            Opcode::Call => {
                let (func_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::Call { func_idx }
            }
            
            Opcode::CallIndirect => {
                let (type_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                let (table_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::CallIndirect { type_idx, table_idx }
            }
            
            Opcode::Drop => Instruction::Drop,
            Opcode::Select => Instruction::Select,
            
            Opcode::LocalGet => {
                let (local_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::LocalGet { local_idx }
            }
            
            Opcode::LocalSet => {
                let (local_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::LocalSet { local_idx }
            }
            
            Opcode::LocalTee => {
                let (local_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::LocalTee { local_idx }
            }
            
            Opcode::GlobalGet => {
                let (global_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::GlobalGet { global_idx }
            }
            
            Opcode::GlobalSet => {
                let (global_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::GlobalSet { global_idx }
            }
            
            // Memory instructions with MemArg
            Opcode::I32Load => {
                let (memarg, bytes_read) = self.parse_memarg(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::I32Load { memarg }
            }
            
            Opcode::I64Load => {
                let (memarg, bytes_read) = self.parse_memarg(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::I64Load { memarg }
            }
            
            Opcode::F32Load => {
                let (memarg, bytes_read) = self.parse_memarg(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::F32Load { memarg }
            }
            
            Opcode::F64Load => {
                let (memarg, bytes_read) = self.parse_memarg(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::F64Load { memarg }
            }
            
            Opcode::I32Store => {
                let (memarg, bytes_read) = self.parse_memarg(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::I32Store { memarg }
            }
            
            Opcode::I64Store => {
                let (memarg, bytes_read) = self.parse_memarg(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::I64Store { memarg }
            }
            
            Opcode::F32Store => {
                let (memarg, bytes_read) = self.parse_memarg(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::F32Store { memarg }
            }
            
            Opcode::F64Store => {
                let (memarg, bytes_read) = self.parse_memarg(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::F64Store { memarg }
            }
            
            Opcode::MemorySize => {
                let (mem_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::MemorySize { mem_idx }
            }
            
            Opcode::MemoryGrow => {
                let (mem_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::MemoryGrow { mem_idx }
            }
            
            // Constant instructions
            Opcode::I32Const => {
                let (value, bytes_read) = leb128::read_leb128_i32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::I32Const { value }
            }
            
            Opcode::I64Const => {
                let (value, bytes_read) = leb128::read_leb128_i64(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::I64Const { value }
            }
            
            Opcode::F32Const => {
                if current_offset + 4 > data.len() {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Insufficient data for f32 constant"
                    ));
                }
                let bytes = [data[current_offset], data[current_offset + 1], 
                           data[current_offset + 2], data[current_offset + 3]];
                let value = f32::from_le_bytes(bytes);
                current_offset += 4;
                Instruction::F32Const { value }
            }
            
            Opcode::F64Const => {
                if current_offset + 8 > data.len() {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Insufficient data for f64 constant"
                    ));
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&data[current_offset..current_offset + 8]);
                let value = f64::from_le_bytes(bytes);
                current_offset += 8;
                Instruction::F64Const { value }
            }
            
            // Comparison instructions
            Opcode::I32Eqz => Instruction::I32Eqz,
            Opcode::I32Eq => Instruction::I32Eq,
            Opcode::I32Ne => Instruction::I32Ne,
            Opcode::I32LtS => Instruction::I32LtS,
            Opcode::I32LtU => Instruction::I32LtU,
            Opcode::I32GtS => Instruction::I32GtS,
            Opcode::I32GtU => Instruction::I32GtU,
            Opcode::I32LeS => Instruction::I32LeS,
            Opcode::I32LeU => Instruction::I32LeU,
            Opcode::I32GeS => Instruction::I32GeS,
            Opcode::I32GeU => Instruction::I32GeU,
            
            // Reference instructions
            Opcode::RefNull => {
                let (ref_type, bytes_read) = self.parse_value_type(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::RefNull { ref_type }
            }
            
            Opcode::RefIsNull => Instruction::RefIsNull,
            
            Opcode::RefFunc => {
                let (func_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                Instruction::RefFunc { func_idx }
            }
            
            _ => {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_INVALID_OPCODE_BYTE,
                    "Unimplemented instruction opcode"
                ));
            }
        };
        
        Ok((instruction, current_offset - offset))
    }
    
    /// Parse block type
    fn parse_block_type(&self, data: &[u8], offset: usize) -> Result<(BlockType, usize)> {
        if offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end while parsing block type"
            ));
        }
        
        let byte = data[offset];
        match byte {
            0x40 => Ok((BlockType::Empty, 1)),
            0x7F => Ok((BlockType::Value(ValueType::I32), 1)),
            0x7E => Ok((BlockType::Value(ValueType::I64), 1)),
            0x7D => Ok((BlockType::Value(ValueType::F32), 1)),
            0x7C => Ok((BlockType::Value(ValueType::F64), 1)),
            _ => {
                // Type index (signed LEB128)
                let (type_idx, bytes_read) = leb128::read_leb128_i32(data, offset)?;
                if type_idx < 0 {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::INVALID_TYPE,
                        "Invalid type index in block type"
                    ));
                }
                Ok((BlockType::Type(type_idx as u32), bytes_read))
            }
        }
    }
    
    /// Parse memory argument (alignment + offset)
    fn parse_memarg(&self, data: &[u8], offset: usize) -> Result<(MemArg, usize)> {
        let (align, bytes_read1) = leb128::read_leb128_u32(data, offset)?;
        let (offset_val, bytes_read2) = leb128::read_leb128_u32(data, offset + bytes_read1)?;
        
        Ok((MemArg { align, offset: offset_val }, bytes_read1 + bytes_read2))
    }
    
    /// Parse branch table
    fn parse_br_table(&self, data: &[u8], offset: usize) -> Result<(BrTable, usize)> {
        let mut current_offset = offset;
        
        // Read vector length
        let (len, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        if len > MAX_BRANCH_TARGETS as u32 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::MEMORY_LIMIT_EXCEEDED,
                "Too many branch targets in br_table"
            ));
        }
        
        // Read targets
        let mut targets = SimpleBoundedVec::new();
        for _ in 0..len {
            let (target, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
            current_offset += bytes_read;
            targets.push(target)?;
        }
        
        // Read default target
        let (default_target, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        Ok((BrTable { targets, default_target }, current_offset - offset))
    }
    
    /// Parse value type
    fn parse_value_type(&self, data: &[u8], offset: usize) -> Result<(ValueType, usize)> {
        if offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end while parsing value type"
            ));
        }
        
        let byte = data[offset];
        let value_type = match byte {
            0x7F => ValueType::I32,
            0x7E => ValueType::I64,
            0x7D => ValueType::F32,
            0x7C => ValueType::F64,
            0x70 => ValueType::FuncRef,
            0x6F => ValueType::ExternRef,
            _ => {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::INVALID_TYPE,
                    "Unknown value type"
                ));
            }
        };
        
        Ok((value_type, 1))
    }
    
    /// Parse complete function body
    pub fn parse_function_body(&mut self, data: &[u8], locals: &[ValueType]) -> Result<SimpleBoundedVec<Instruction, 1024>> {
        self.init_function(locals)?;
        
        let mut instructions = SimpleBoundedVec::new();
        let mut offset = 0;
        
        while offset < data.len() {
            let (instruction, bytes_consumed) = self.parse_instruction(data, offset)?;
            offset += bytes_consumed;
            
            // Check for end of function
            if matches!(instruction, Instruction::End) {
                instructions.push(instruction)?;
                break;
            }
            
            instructions.push(instruction)?;
        }
        
        Ok(instructions)
    }
}

impl Default for InstructionParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_opcode_from_byte() {
        assert_eq!(Opcode::from_byte(0x00), Some(Opcode::Unreachable));
        assert_eq!(Opcode::from_byte(0x01), Some(Opcode::Nop));
        assert_eq!(Opcode::from_byte(0x02), Some(Opcode::Block));
        assert_eq!(Opcode::from_byte(0xFF), None);
    }
    
    #[test]
    fn test_opcode_properties() {
        assert!(Opcode::Block.is_control());
        assert!(Opcode::Block.is_block_start());
        assert!(Opcode::Block.has_immediates());
        
        assert!(!Opcode::Nop.is_control());
        assert!(!Opcode::Nop.is_block_start());
        assert!(!Opcode::Nop.has_immediates());
    }
    
    #[test]
    fn test_instruction_parser_creation() {
        let parser = InstructionParser::new();
        assert_eq!(parser.control_stack.len(), 0);
        assert_eq!(parser.value_stack.len(), 0);
        assert_eq!(parser.locals.len(), 0);
    }
    
    #[test]
    fn test_parse_simple_instructions() {
        let mut parser = InstructionParser::new();
        
        // Test nop
        let data = [0x01]; // nop
        let (instruction, bytes_consumed) = parser.parse_instruction(&data, 0).unwrap();
        assert_eq!(instruction, Instruction::Nop);
        assert_eq!(bytes_consumed, 1);
        
        // Test unreachable
        let data = [0x00]; // unreachable
        let (instruction, bytes_consumed) = parser.parse_instruction(&data, 0).unwrap();
        assert_eq!(instruction, Instruction::Unreachable);
        assert_eq!(bytes_consumed, 1);
    }
    
    #[test]
    fn test_parse_i32_const() {
        let mut parser = InstructionParser::new();
        
        // i32.const 42
        let data = [0x41, 0x2A]; // i32.const with LEB128 encoded 42
        let (instruction, bytes_consumed) = parser.parse_instruction(&data, 0).unwrap();
        assert_eq!(instruction, Instruction::I32Const { value: 42 });
        assert_eq!(bytes_consumed, 2);
    }
    
    #[test]
    fn test_parse_local_get() {
        let mut parser = InstructionParser::new();
        
        // local.get 0
        let data = [0x20, 0x00]; // local.get with index 0
        let (instruction, bytes_consumed) = parser.parse_instruction(&data, 0).unwrap();
        assert_eq!(instruction, Instruction::LocalGet { local_idx: 0 });
        assert_eq!(bytes_consumed, 2);
    }
    
    #[test]
    fn test_parse_block_type() {
        let parser = InstructionParser::new();
        
        // Empty block type
        let data = [0x40];
        let (block_type, bytes_consumed) = parser.parse_block_type(&data, 0).unwrap();
        assert_eq!(block_type, BlockType::Empty);
        assert_eq!(bytes_consumed, 1);
        
        // i32 value type
        let data = [0x7F];
        let (block_type, bytes_consumed) = parser.parse_block_type(&data, 0).unwrap();
        assert_eq!(block_type, BlockType::Value(ValueType::I32));
        assert_eq!(bytes_consumed, 1);
    }
    
    #[test]
    fn test_parse_memarg() {
        let parser = InstructionParser::new();
        
        // align=2, offset=0
        let data = [0x02, 0x00];
        let (memarg, bytes_consumed) = parser.parse_memarg(&data, 0).unwrap();
        assert_eq!(memarg.align, 2);
        assert_eq!(memarg.offset, 0);
        assert_eq!(bytes_consumed, 2);
    }
    
    #[test]
    fn test_invalid_opcode() {
        let mut parser = InstructionParser::new();
        
        let data = [0xFF]; // Invalid opcode
        let result = parser.parse_instruction(&data, 0);
        assert!(result.is_err());
    }
}