//! WebAssembly instruction handling
//!
//! This module provides types and functions for parsing and encoding WebAssembly instructions.

use crate::prelude::*;
use crate::types::BlockType;
use wrt_error::{kinds, Error, Result};
use wrt_format::binary;
use wrt_format::types::value_type_to_byte;

#[cfg(feature = "std")]
use std::vec;

#[cfg(not(feature = "std"))]
use alloc::vec;

/// WebAssembly instruction enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Control instructions
    Unreachable,
    Nop,
    Block(BlockType, Vec<Instruction>),
    Loop(BlockType, Vec<Instruction>),
    If(BlockType, Vec<Instruction>, Vec<Instruction>),
    Br(u32),
    BrIf(u32),
    BrTable(Vec<u32>, u32),
    Return,
    Call(u32),
    CallIndirect(u32, u8),

    // Parametric instructions
    Drop,
    Select,

    // Variable instructions
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),

    // Memory instructions
    I32Load(u32, u32),
    I64Load(u32, u32),
    F32Load(u32, u32),
    F64Load(u32, u32),
    I32Load8S(u32, u32),
    I32Load8U(u32, u32),
    I32Load16S(u32, u32),
    I32Load16U(u32, u32),
    I64Load8S(u32, u32),
    I64Load8U(u32, u32),
    I64Load16S(u32, u32),
    I64Load16U(u32, u32),
    I64Load32S(u32, u32),
    I64Load32U(u32, u32),
    I32Store(u32, u32),
    I64Store(u32, u32),
    F32Store(u32, u32),
    F64Store(u32, u32),
    I32Store8(u32, u32),
    I32Store16(u32, u32),
    I64Store8(u32, u32),
    I64Store16(u32, u32),
    I64Store32(u32, u32),
    MemorySize,
    MemoryGrow,
    MemoryCopy(u32, u32),
    MemoryFill(u32),

    // Numeric instructions
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),

    // I32 operations
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
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,

    // I64 operations
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,

    // F32 operations
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,

    // F64 operations
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,

    // Conversions
    I32WrapI64,
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64ExtendI32S,
    I64ExtendI32U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F32DemoteF64,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,

    // Saturating truncation
    I32TruncSatF32S,
    I32TruncSatF32U,
    I32TruncSatF64S,
    I32TruncSatF64U,
    I64TruncSatF32S,
    I64TruncSatF32U,
    I64TruncSatF64S,
    I64TruncSatF64U,
}

/// Parse a sequence of WebAssembly instructions
pub fn parse_instructions(bytes: &[u8]) -> Result<Vec<Instruction>> {
    let mut result = Vec::new();
    let mut offset = 0;

    while offset < bytes.len() {
        let (instruction, bytes_read) = parse_instruction(&bytes[offset..])?;
        result.push(instruction);
        offset += bytes_read;
    }

    Ok(result)
}

/// Parse a single WebAssembly instruction
pub fn parse_instruction(bytes: &[u8]) -> Result<(Instruction, usize)> {
    if bytes.is_empty() {
        return Err(Error::new(kinds::ParseError(
            "Empty instruction bytes".to_string(),
        )));
    }

    let opcode = bytes[0];
    match opcode {
        // Control instructions
        binary::UNREACHABLE => Ok((Instruction::Unreachable, 1)),
        binary::NOP => Ok((Instruction::Nop, 1)),
        binary::BLOCK => {
            let (block_type, bytes_read) = parse_block_type(&bytes[1..])?;
            let (instructions, instructions_bytes_read) = parse_instructions(&bytes[1 + bytes_read..])?;
            Ok((Instruction::Block(block_type, instructions), 1 + bytes_read + instructions_bytes_read))
        }
        binary::LOOP => {
            let (block_type, bytes_read) = parse_block_type(&bytes[1..])?;
            let (instructions, instructions_bytes_read) = parse_instructions(&bytes[1 + bytes_read..])?;
            Ok((Instruction::Loop(block_type, instructions), 1 + bytes_read + instructions_bytes_read))
        }
        binary::IF => {
            let (block_type, bytes_read) = parse_block_type(&bytes[1..])?;
            let (then_instructions, then_bytes_read) = parse_instructions(&bytes[1 + bytes_read..])?;
            let (else_instructions, else_bytes_read) = parse_instructions(&bytes[1 + bytes_read + then_bytes_read..])?;
            Ok((Instruction::If(block_type, then_instructions, else_instructions), 1 + bytes_read + then_bytes_read + else_bytes_read))
        }
        binary::BR => {
            let (label_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::Br(label_idx), 1 + bytes_read))
        }
        binary::BR_IF => {
            let (label_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::BrIf(label_idx), 1 + bytes_read))
        }
        binary::BR_TABLE => {
            let (count, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let mut offset = 1 + bytes_read;
            let mut labels = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let (label_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                labels.push(label_idx);
                offset += bytes_read;
            }
            let (default_label, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            Ok((Instruction::BrTable(labels, default_label), offset + bytes_read))
        }
        binary::RETURN => Ok((Instruction::Return, 1)),
        binary::CALL => {
            let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::Call(func_idx), 1 + bytes_read))
        }
        binary::CALL_INDIRECT => {
            let (type_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let table_idx = bytes[1 + bytes_read];
            Ok((Instruction::CallIndirect(type_idx, table_idx), 2 + bytes_read))
        }

        // Variable instructions
        binary::LOCAL_GET => {
            let (local_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::LocalGet(local_idx), 1 + bytes_read))
        }
        binary::LOCAL_SET => {
            let (local_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::LocalSet(local_idx), 1 + bytes_read))
        }
        binary::LOCAL_TEE => {
            let (local_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::LocalTee(local_idx), 1 + bytes_read))
        }
        binary::GLOBAL_GET => {
            let (global_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::GlobalGet(global_idx), 1 + bytes_read))
        }
        binary::GLOBAL_SET => {
            let (global_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::GlobalSet(global_idx), 1 + bytes_read))
        }

        // Memory instructions
        binary::I32_LOAD => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I32Load(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I64_LOAD => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I64Load(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::F32_LOAD => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::F32Load(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::F64_LOAD => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::F64Load(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I32_LOAD8_S => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I32Load8S(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I32_LOAD8_U => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I32Load8U(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I32_LOAD16_S => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I32Load16S(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I32_LOAD16_U => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I32Load16U(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I64_LOAD8_S => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I64Load8S(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I64_LOAD8_U => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I64Load8U(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I64_LOAD16_S => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I64Load16S(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I64_LOAD16_U => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I64Load16U(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I64_LOAD32_S => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I64Load32S(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I64_LOAD32_U => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I64Load32U(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I32_STORE => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I32Store(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I64_STORE => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I64Store(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::F32_STORE => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::F32Store(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::F64_STORE => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::F64Store(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I32_STORE8 => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I32Store8(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I32_STORE16 => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I32Store16(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I64_STORE8 => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I64Store8(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I64_STORE16 => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I64Store16(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::I64_STORE32 => {
            let (align, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (offset, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::I64Store32(offset, align), 1 + bytes_read + bytes_read2))
        }
        binary::MEMORY_SIZE => Ok((Instruction::MemorySize, 1)),
        binary::MEMORY_GROW => Ok((Instruction::MemoryGrow, 1)),
        binary::MEMORY_COPY => {
            let (dst_memory_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            let (src_memory_idx, bytes_read2) = binary::read_leb128_u32(bytes, 1 + bytes_read)?;
            Ok((Instruction::MemoryCopy(dst_memory_idx, src_memory_idx), 1 + bytes_read + bytes_read2))
        }
        binary::MEMORY_FILL => {
            let (memory_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::MemoryFill(memory_idx), 1 + bytes_read))
        }

        // Numeric instructions
        binary::I32_CONST => {
            let (value, bytes_read) = binary::read_leb128_i32(bytes, 1)?;
            Ok((Instruction::I32Const(value), 1 + bytes_read))
        }
        binary::I64_CONST => {
            let (value, bytes_read) = binary::read_leb128_i64(bytes, 1)?;
            Ok((Instruction::I64Const(value), 1 + bytes_read))
        }
        binary::F32_CONST => {
            let (value, bytes_read) = binary::read_f32(bytes, 1)?;
            Ok((Instruction::F32Const(value), 1 + bytes_read))
        }
        binary::F64_CONST => {
            let (value, bytes_read) = binary::read_f64(bytes, 1)?;
            Ok((Instruction::F64Const(value), 1 + bytes_read))
        }

        // I32 operations
        binary::I32_EQZ => Ok((Instruction::I32Eqz, 1)),
        binary::I32_EQ => Ok((Instruction::I32Eq, 1)),
        binary::I32_NE => Ok((Instruction::I32Ne, 1)),
        binary::I32_LT_S => Ok((Instruction::I32LtS, 1)),
        binary::I32_LT_U => Ok((Instruction::I32LtU, 1)),
        binary::I32_GT_S => Ok((Instruction::I32GtS, 1)),
        binary::I32_GT_U => Ok((Instruction::I32GtU, 1)),
        binary::I32_LE_S => Ok((Instruction::I32LeS, 1)),
        binary::I32_LE_U => Ok((Instruction::I32LeU, 1)),
        binary::I32_GE_S => Ok((Instruction::I32GeS, 1)),
        binary::I32_GE_U => Ok((Instruction::I32GeU, 1)),
        binary::I32_CLZ => Ok((Instruction::I32Clz, 1)),
        binary::I32_CTZ => Ok((Instruction::I32Ctz, 1)),
        binary::I32_POPCNT => Ok((Instruction::I32Popcnt, 1)),
        binary::I32_ADD => Ok((Instruction::I32Add, 1)),
        binary::I32_SUB => Ok((Instruction::I32Sub, 1)),
        binary::I32_MUL => Ok((Instruction::I32Mul, 1)),
        binary::I32_DIV_S => Ok((Instruction::I32DivS, 1)),
        binary::I32_DIV_U => Ok((Instruction::I32DivU, 1)),
        binary::I32_REM_S => Ok((Instruction::I32RemS, 1)),
        binary::I32_REM_U => Ok((Instruction::I32RemU, 1)),
        binary::I32_AND => Ok((Instruction::I32And, 1)),
        binary::I32_OR => Ok((Instruction::I32Or, 1)),
        binary::I32_XOR => Ok((Instruction::I32Xor, 1)),
        binary::I32_SHL => Ok((Instruction::I32Shl, 1)),
        binary::I32_SHR_S => Ok((Instruction::I32ShrS, 1)),
        binary::I32_SHR_U => Ok((Instruction::I32ShrU, 1)),
        binary::I32_ROTL => Ok((Instruction::I32Rotl, 1)),
        binary::I32_ROTR => Ok((Instruction::I32Rotr, 1)),

        // I64 operations
        binary::I64_EQZ => Ok((Instruction::I64Eqz, 1)),
        binary::I64_EQ => Ok((Instruction::I64Eq, 1)),
        binary::I64_NE => Ok((Instruction::I64Ne, 1)),
        binary::I64_LT_S => Ok((Instruction::I64LtS, 1)),
        binary::I64_LT_U => Ok((Instruction::I64LtU, 1)),
        binary::I64_GT_S => Ok((Instruction::I64GtS, 1)),
        binary::I64_GT_U => Ok((Instruction::I64GtU, 1)),
        binary::I64_LE_S => Ok((Instruction::I64LeS, 1)),
        binary::I64_LE_U => Ok((Instruction::I64LeU, 1)),
        binary::I64_GE_S => Ok((Instruction::I64GeS, 1)),
        binary::I64_GE_U => Ok((Instruction::I64GeU, 1)),
        binary::I64_CLZ => Ok((Instruction::I64Clz, 1)),
        binary::I64_CTZ => Ok((Instruction::I64Ctz, 1)),
        binary::I64_POPCNT => Ok((Instruction::I64Popcnt, 1)),
        binary::I64_ADD => Ok((Instruction::I64Add, 1)),
        binary::I64_SUB => Ok((Instruction::I64Sub, 1)),
        binary::I64_MUL => Ok((Instruction::I64Mul, 1)),
        binary::I64_DIV_S => Ok((Instruction::I64DivS, 1)),
        binary::I64_DIV_U => Ok((Instruction::I64DivU, 1)),
        binary::I64_REM_S => Ok((Instruction::I64RemS, 1)),
        binary::I64_REM_U => Ok((Instruction::I64RemU, 1)),
        binary::I64_AND => Ok((Instruction::I64And, 1)),
        binary::I64_OR => Ok((Instruction::I64Or, 1)),
        binary::I64_XOR => Ok((Instruction::I64Xor, 1)),
        binary::I64_SHL => Ok((Instruction::I64Shl, 1)),
        binary::I64_SHR_S => Ok((Instruction::I64ShrS, 1)),
        binary::I64_SHR_U => Ok((Instruction::I64ShrU, 1)),
        binary::I64_ROTL => Ok((Instruction::I64Rotl, 1)),
        binary::I64_ROTR => Ok((Instruction::I64Rotr, 1)),

        // F32 operations
        binary::F32_EQ => Ok((Instruction::F32Eq, 1)),
        binary::F32_NE => Ok((Instruction::F32Ne, 1)),
        binary::F32_LT => Ok((Instruction::F32Lt, 1)),
        binary::F32_GT => Ok((Instruction::F32Gt, 1)),
        binary::F32_LE => Ok((Instruction::F32Le, 1)),
        binary::F32_GE => Ok((Instruction::F32Ge, 1)),
        binary::F32_ABS => Ok((Instruction::F32Abs, 1)),
        binary::F32_NEG => Ok((Instruction::F32Neg, 1)),
        binary::F32_CEIL => Ok((Instruction::F32Ceil, 1)),
        binary::F32_FLOOR => Ok((Instruction::F32Floor, 1)),
        binary::F32_TRUNC => Ok((Instruction::F32Trunc, 1)),
        binary::F32_NEAREST => Ok((Instruction::F32Nearest, 1)),
        binary::F32_SQRT => Ok((Instruction::F32Sqrt, 1)),
        binary::F32_ADD => Ok((Instruction::F32Add, 1)),
        binary::F32_SUB => Ok((Instruction::F32Sub, 1)),
        binary::F32_MUL => Ok((Instruction::F32Mul, 1)),
        binary::F32_DIV => Ok((Instruction::F32Div, 1)),
        binary::F32_MIN => Ok((Instruction::F32Min, 1)),
        binary::F32_MAX => Ok((Instruction::F32Max, 1)),
        binary::F32_COPYSIGN => Ok((Instruction::F32Copysign, 1)),

        // F64 operations
        binary::F64_EQ => Ok((Instruction::F64Eq, 1)),
        binary::F64_NE => Ok((Instruction::F64Ne, 1)),
        binary::F64_LT => Ok((Instruction::F64Lt, 1)),
        binary::F64_GT => Ok((Instruction::F64Gt, 1)),
        binary::F64_LE => Ok((Instruction::F64Le, 1)),
        binary::F64_GE => Ok((Instruction::F64Ge, 1)),
        binary::F64_ABS => Ok((Instruction::F64Abs, 1)),
        binary::F64_NEG => Ok((Instruction::F64Neg, 1)),
        binary::F64_CEIL => Ok((Instruction::F64Ceil, 1)),
        binary::F64_FLOOR => Ok((Instruction::F64Floor, 1)),
        binary::F64_TRUNC => Ok((Instruction::F64Trunc, 1)),
        binary::F64_NEAREST => Ok((Instruction::F64Nearest, 1)),
        binary::F64_SQRT => Ok((Instruction::F64Sqrt, 1)),
        binary::F64_ADD => Ok((Instruction::F64Add, 1)),
        binary::F64_SUB => Ok((Instruction::F64Sub, 1)),
        binary::F64_MUL => Ok((Instruction::F64Mul, 1)),
        binary::F64_DIV => Ok((Instruction::F64Div, 1)),
        binary::F64_MIN => Ok((Instruction::F64Min, 1)),
        binary::F64_MAX => Ok((Instruction::F64Max, 1)),
        binary::F64_COPYSIGN => Ok((Instruction::F64Copysign, 1)),

        // Conversion instructions
        binary::I32_WRAP_I64 => Ok((Instruction::I32WrapI64, 1)),
        binary::I32_TRUNC_F32_S => Ok((Instruction::I32TruncF32S, 1)),
        binary::I32_TRUNC_F32_U => Ok((Instruction::I32TruncF32U, 1)),
        binary::I32_TRUNC_F64_S => Ok((Instruction::I32TruncF64S, 1)),
        binary::I32_TRUNC_F64_U => Ok((Instruction::I32TruncF64U, 1)),
        binary::I64_EXTEND_I32_S => Ok((Instruction::I64ExtendI32S, 1)),
        binary::I64_EXTEND_I32_U => Ok((Instruction::I64ExtendI32U, 1)),
        binary::I64_TRUNC_F32_S => Ok((Instruction::I64TruncF32S, 1)),
        binary::I64_TRUNC_F32_U => Ok((Instruction::I64TruncF32U, 1)),
        binary::I64_TRUNC_F64_S => Ok((Instruction::I64TruncF64S, 1)),
        binary::I64_TRUNC_F64_U => Ok((Instruction::I64TruncF64U, 1)),
        binary::F32_CONVERT_I32_S => Ok((Instruction::F32ConvertI32S, 1)),
        binary::F32_CONVERT_I32_U => Ok((Instruction::F32ConvertI32U, 1)),
        binary::F32_CONVERT_I64_S => Ok((Instruction::F32ConvertI64S, 1)),
        binary::F32_CONVERT_I64_U => Ok((Instruction::F32ConvertI64U, 1)),
        binary::F32_DEMOTE_F64 => Ok((Instruction::F32DemoteF64, 1)),
        binary::F64_CONVERT_I32_S => Ok((Instruction::F64ConvertI32S, 1)),
        binary::F64_CONVERT_I32_U => Ok((Instruction::F64ConvertI32U, 1)),
        binary::F64_CONVERT_I64_S => Ok((Instruction::F64ConvertI64S, 1)),
        binary::F64_CONVERT_I64_U => Ok((Instruction::F64ConvertI64U, 1)),
        binary::F64_PROMOTE_F32 => Ok((Instruction::F64PromoteF32, 1)),
        binary::I32_REINTERPRET_F32 => Ok((Instruction::I32ReinterpretF32, 1)),
        binary::I64_REINTERPRET_F64 => Ok((Instruction::I64ReinterpretF64, 1)),
        binary::F32_REINTERPRET_I32 => Ok((Instruction::F32ReinterpretI32, 1)),
        binary::F64_REINTERPRET_I64 => Ok((Instruction::F64ReinterpretI64, 1)),

        // Saturating truncation instructions
        binary::I32_TRUNC_SAT_F32_S => Ok((Instruction::I32TruncSatF32S, 1)),
        binary::I32_TRUNC_SAT_F32_U => Ok((Instruction::I32TruncSatF32U, 1)),
        binary::I32_TRUNC_SAT_F64_S => Ok((Instruction::I32TruncSatF64S, 1)),
        binary::I32_TRUNC_SAT_F64_U => Ok((Instruction::I32TruncSatF64U, 1)),
        binary::I64_TRUNC_SAT_F32_S => Ok((Instruction::I64TruncSatF32S, 1)),
        binary::I64_TRUNC_SAT_F32_U => Ok((Instruction::I64TruncSatF32U, 1)),
        binary::I64_TRUNC_SAT_F64_S => Ok((Instruction::I64TruncSatF64S, 1)),
        binary::I64_TRUNC_SAT_F64_U => Ok((Instruction::I64TruncSatF64U, 1)),

        _ => Err(Error::new(kinds::ParseError(format!(
            "Unknown instruction opcode: {:#x}",
            opcode
        )))),
    }
}

/// Encode a sequence of WebAssembly instructions
pub fn encode_instructions(instructions: &[Instruction]) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    for instruction in instructions {
        let encoded = encode_instruction(instruction)?;
        result.extend_from_slice(&encoded);
    }

    Ok(result)
}

/// Encode a single WebAssembly instruction
pub fn encode_instruction(instruction: &Instruction) -> Result<Vec<u8>> {
    match instruction {
        // Control instructions
        Instruction::Unreachable => Ok(vec![binary::UNREACHABLE]),
        Instruction::Nop => Ok(vec![binary::NOP]),
        Instruction::Call(func_idx) => {
            let mut result = vec![binary::CALL];
            result.extend_from_slice(&wrt_format::binary::write_leb128_u32(*func_idx));
            Ok(result)
        }

        // Block, loop, if instructions
        Instruction::Block(block_type, instructions) => {
            let mut bytes = vec![binary::BLOCK];

            // Encode block type
            match block_type {
                BlockType::Empty => bytes.push(0x40),
                BlockType::Value(val_type) => bytes.push(value_type_to_byte(*val_type)),
                BlockType::TypeIndex(idx) => {
                    bytes.extend_from_slice(&binary::write_leb128_i32(*idx as i32))
                },
                BlockType::FuncType(_) => {
                    return Err(Error::new(kinds::EncodingError(
                        "Cannot directly encode BlockType::FuncType - use TypeIndex instead".into(),
                    )))
                }
            }

            // Encode nested instructions
            for inst in instructions {
                bytes.extend_from_slice(&encode_instruction(inst)?);
            }

            // Add END opcode
            bytes.push(binary::END);

            Ok(bytes)
        }
        Instruction::Loop(block_type, instructions) => {
            let mut bytes = vec![binary::LOOP];

            // Encode block type
            match block_type {
                BlockType::Empty => bytes.push(0x40),
                BlockType::Value(val_type) => bytes.push(value_type_to_byte(*val_type)),
                BlockType::TypeIndex(idx) => {
                    bytes.extend_from_slice(&binary::write_leb128_i32(*idx as i32))
                },
                BlockType::FuncType(_) => {
                    return Err(Error::new(kinds::EncodingError(
                        "Cannot directly encode BlockType::FuncType - use TypeIndex instead".into(),
                    )))
                }
            }

            // Encode nested instructions
            for inst in instructions {
                bytes.extend_from_slice(&encode_instruction(inst)?);
            }

            // Add END opcode
            bytes.push(binary::END);

            Ok(bytes)
        }
        Instruction::If(block_type, then_instructions, else_instructions) => {
            let mut bytes = vec![binary::IF];

            // Encode block type
            match block_type {
                BlockType::Empty => bytes.push(0x40),
                BlockType::Value(val_type) => bytes.push(value_type_to_byte(*val_type)),
                BlockType::TypeIndex(idx) => {
                    bytes.extend_from_slice(&binary::write_leb128_i32(*idx as i32))
                },
                BlockType::FuncType(_) => {
                    return Err(Error::new(kinds::EncodingError(
                        "Cannot directly encode BlockType::FuncType - use TypeIndex instead".into(),
                    )))
                }
            }

            // Encode then instructions
            for inst in then_instructions {
                bytes.extend_from_slice(&encode_instruction(inst)?);
            }

            // Add ELSE opcode if there are else instructions
            if !else_instructions.is_empty() {
                bytes.push(binary::ELSE);

                // Encode else instructions
                for inst in else_instructions {
                    bytes.extend_from_slice(&encode_instruction(inst)?);
                }
            }

            // Add END opcode
            bytes.push(binary::END);

            Ok(bytes)
        }
        Instruction::Br(label_idx) => {
            let mut bytes = vec![binary::BR];
            bytes.extend_from_slice(&binary::write_leb128_u32(*label_idx));
            Ok(bytes)
        }
        Instruction::BrIf(label_idx) => {
            let mut bytes = vec![binary::BR_IF];
            bytes.extend_from_slice(&binary::write_leb128_u32(*label_idx));
            Ok(bytes)
        }
        Instruction::BrTable(labels, default_label) => {
            let mut bytes = vec![binary::BR_TABLE];

            // Encode the label count
            bytes.extend_from_slice(&binary::write_leb128_u32(labels.len() as u32));

            // Encode each label
            for label in labels {
                bytes.extend_from_slice(&binary::write_leb128_u32(*label));
            }

            // Encode the default label
            bytes.extend_from_slice(&binary::write_leb128_u32(*default_label));

            Ok(bytes)
        }
        Instruction::Return => Ok(vec![binary::RETURN]),

        // Variable instructions
        Instruction::LocalGet(local_idx) => {
            let mut bytes = vec![binary::LOCAL_GET];
            bytes.extend_from_slice(&binary::write_leb128_u32(*local_idx));
            Ok(bytes)
        }
        Instruction::LocalSet(local_idx) => {
            let mut bytes = vec![binary::LOCAL_SET];
            bytes.extend_from_slice(&binary::write_leb128_u32(*local_idx));
            Ok(bytes)
        }
        Instruction::LocalTee(local_idx) => {
            let mut bytes = vec![binary::LOCAL_TEE];
            bytes.extend_from_slice(&binary::write_leb128_u32(*local_idx));
            Ok(bytes)
        }
        Instruction::GlobalGet(global_idx) => {
            let mut bytes = vec![binary::GLOBAL_GET];
            bytes.extend_from_slice(&binary::write_leb128_u32(*global_idx));
            Ok(bytes)
        }
        Instruction::GlobalSet(global_idx) => {
            let mut bytes = vec![binary::GLOBAL_SET];
            bytes.extend_from_slice(&binary::write_leb128_u32(*global_idx));
            Ok(bytes)
        }

        // Numeric instructions
        Instruction::I32Const(value) => {
            let mut bytes = vec![binary::I32_CONST];
            bytes.extend_from_slice(&binary::write_leb128_i32(*value));
            Ok(bytes)
        }
        Instruction::I64Const(value) => {
            let mut bytes = vec![binary::I64_CONST];
            bytes.extend_from_slice(&binary::write_leb128_i64(*value));
            Ok(bytes)
        }
        Instruction::F32Const(value) => {
            let mut bytes = vec![binary::F32_CONST];
            bytes.extend_from_slice(&binary::write_f32(*value));
            Ok(bytes)
        }
        Instruction::F64Const(value) => {
            let mut bytes = vec![binary::F64_CONST];
            bytes.extend_from_slice(&binary::write_f64(*value));
            Ok(bytes)
        }

        // Memory instructions
        Instruction::I32Load(offset, align) => {
            let mut bytes = vec![binary::I32_LOAD];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I64Load(offset, align) => {
            let mut bytes = vec![binary::I64_LOAD];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::F32Load(offset, align) => {
            let mut bytes = vec![binary::F32_LOAD];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::F64Load(offset, align) => {
            let mut bytes = vec![binary::F64_LOAD];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I32Load8S(offset, align) => {
            let mut bytes = vec![binary::I32_LOAD8_S];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I32Load8U(offset, align) => {
            let mut bytes = vec![binary::I32_LOAD8_U];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I32Load16S(offset, align) => {
            let mut bytes = vec![binary::I32_LOAD16_S];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I32Load16U(offset, align) => {
            let mut bytes = vec![binary::I32_LOAD16_U];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I64Load8S(offset, align) => {
            let mut bytes = vec![binary::I64_LOAD8_S];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I64Load8U(offset, align) => {
            let mut bytes = vec![binary::I64_LOAD8_U];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I64Load16S(offset, align) => {
            let mut bytes = vec![binary::I64_LOAD16_S];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I64Load16U(offset, align) => {
            let mut bytes = vec![binary::I64_LOAD16_U];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I64Load32S(offset, align) => {
            let mut bytes = vec![binary::I64_LOAD32_S];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I64Load32U(offset, align) => {
            let mut bytes = vec![binary::I64_LOAD32_U];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I32Store(offset, align) => {
            let mut bytes = vec![binary::I32_STORE];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I64Store(offset, align) => {
            let mut bytes = vec![binary::I64_STORE];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::F32Store(offset, align) => {
            let mut bytes = vec![binary::F32_STORE];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::F64Store(offset, align) => {
            let mut bytes = vec![binary::F64_STORE];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I32Store8(offset, align) => {
            let mut bytes = vec![binary::I32_STORE8];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I32Store16(offset, align) => {
            let mut bytes = vec![binary::I32_STORE16];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I64Store8(offset, align) => {
            let mut bytes = vec![binary::I64_STORE8];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I64Store16(offset, align) => {
            let mut bytes = vec![binary::I64_STORE16];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::I64Store32(offset, align) => {
            let mut bytes = vec![binary::I64_STORE32];
            bytes.extend_from_slice(&binary::write_leb128_u32(*offset));
            bytes.extend_from_slice(&binary::write_leb128_u32(*align));
            Ok(bytes)
        }
        Instruction::MemorySize => Ok(vec![binary::MEMORY_SIZE]),
        Instruction::MemoryGrow => Ok(vec![binary::MEMORY_GROW]),
        Instruction::MemoryCopy(dst_memory_idx, src_memory_idx) => {
            let mut bytes = vec![binary::MEMORY_COPY];
            bytes.extend_from_slice(&binary::write_leb128_u32(*dst_memory_idx));
            bytes.extend_from_slice(&binary::write_leb128_u32(*src_memory_idx));
            Ok(bytes)
        }
        Instruction::MemoryFill(memory_idx) => {
            let mut bytes = vec![binary::MEMORY_FILL];
            bytes.extend_from_slice(&binary::write_leb128_u32(*memory_idx));
            Ok(bytes)
        }

        // TODO: Complete memory and numeric instructions
        // For now, return an error for unimplemented instructions
        Instruction::CallIndirect(type_idx, table_idx) => {
            let mut result = vec![binary::CALL_INDIRECT];
            result.extend_from_slice(&wrt_format::binary::write_leb128_u32(*type_idx));
            result.push(*table_idx);
            Ok(result)
        }
        _ => Err(Error::new(kinds::EncodingError(format!(
            "Encoding not implemented for instruction: {:?}",
            instruction
        )))),
    }
}

/// Extract local declarations from a function body
pub fn parse_locals(bytes: &[u8]) -> Result<(Vec<(u32, u8)>, usize)> {
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut locals = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read count
        let (local_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        // Read type
        if offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of locals bytes".to_string(),
            )));
        }
        let local_type = bytes[offset];
        offset += 1;

        locals.push((local_count, local_type));
    }

    Ok((locals, offset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ValueType;

    #[cfg(feature = "std")]
    use std::vec;

    #[cfg(not(feature = "std"))]
    use alloc::vec;

    #[test]
    fn test_parse_encode_i32_const() {
        let bytes = vec![binary::I32_CONST, 0x2A]; // 42 in LEB128
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();

        assert_eq!(instruction, Instruction::I32Const(42));
        assert_eq!(bytes_read, 2);

        let encoded = encode_instruction(&instruction).unwrap();
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn test_parse_encode_call() {
        let bytes = vec![binary::CALL, 0x10]; // Function index 16 in LEB128
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();

        assert_eq!(instruction, Instruction::Call(16));
        assert_eq!(bytes_read, 2);

        let encoded = encode_instruction(&instruction).unwrap();
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn test_parse_encode_call_indirect() {
        let bytes = vec![binary::CALL_INDIRECT, 0x20, 0x00]; // Type index 32, table index 0
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();

        assert_eq!(instruction, Instruction::CallIndirect(32, 0));
        assert_eq!(bytes_read, 3);

        let encoded = encode_instruction(&instruction).unwrap();
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn test_parse_encode_block() {
        let bytes = [
            0x02, 0x7F, // block i32
            0x41, 0x2A, // i32.const 42
            0x0B, // end
        ];

        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(bytes_read, bytes.len());

        if let Instruction::Block(ref block_type, ref instructions) = instruction {
            assert_eq!(block_type, &BlockType::Value(ValueType::I32));
            assert_eq!(instructions.len(), 1);
            assert_eq!(instructions[0], Instruction::I32Const(42));
        } else {
            panic!("Expected Block instruction");
        }

        let encoded = encode_instruction(&instruction).unwrap();
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn test_parse_encode_loop() {
        let bytes = [
            0x03, 0x7F, // loop i32
            0x41, 0x2A, // i32.const 42
            0x0C, 0x00, // br 0
            0x0B, // end
        ];

        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(bytes_read, bytes.len());

        if let Instruction::Loop(ref block_type, ref instructions) = instruction {
            assert_eq!(block_type, &BlockType::Value(ValueType::I32));
            assert_eq!(instructions.len(), 2);
            assert_eq!(instructions[0], Instruction::I32Const(42));
            assert_eq!(instructions[1], Instruction::Br(0));
        } else {
            panic!("Expected Loop instruction");
        }

        let encoded = encode_instruction(&instruction).unwrap();
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn test_parse_encode_if() {
        let bytes = [
            0x04, 0x7F, // if i32
            0x41, 0x2A, // i32.const 42
            0x05, // else
            0x41, 0x37, // i32.const 55
            0x0B, // end
        ];

        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(bytes_read, bytes.len());

        if let Instruction::If(ref block_type, ref then_instructions, ref else_instructions) =
            instruction
        {
            assert_eq!(block_type, &BlockType::Value(ValueType::I32));
            assert_eq!(then_instructions.len(), 1);
            assert_eq!(then_instructions[0], Instruction::I32Const(42));
            assert_eq!(else_instructions.len(), 1);
            assert_eq!(else_instructions[0], Instruction::I32Const(55));
        } else {
            panic!("Expected If instruction");
        }

        let encoded = encode_instruction(&instruction).unwrap();
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn test_parse_encode_br_table() {
        let bytes = [
            0x0E, // br_table
            0x02, // 2 labels
            0x00, // label 0
            0x01, // label 1
            0x02, // default label
        ];

        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(bytes_read, bytes.len());

        if let Instruction::BrTable(ref labels, default_label) = instruction {
            assert_eq!(labels, &[0, 1]);
            assert_eq!(default_label, 2);
        } else {
            panic!("Expected BrTable instruction");
        }

        let encoded = encode_instruction(&instruction).unwrap();
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn test_parse_encode_memory_load() {
        // Test i32.load
        let bytes = vec![binary::I32_LOAD, 0x01, 0x02]; // offset=1, align=2
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I32Load(1, 2));
        assert_eq!(bytes_read, 3);

        // Test i64.load
        let bytes = vec![binary::I64_LOAD, 0x01, 0x03]; // offset=1, align=3
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I64Load(1, 3));
        assert_eq!(bytes_read, 3);

        // Test f32.load
        let bytes = vec![binary::F32_LOAD, 0x01, 0x02]; // offset=1, align=2
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::F32Load(1, 2));
        assert_eq!(bytes_read, 3);

        // Test f64.load
        let bytes = vec![binary::F64_LOAD, 0x01, 0x03]; // offset=1, align=3
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::F64Load(1, 3));
        assert_eq!(bytes_read, 3);
    }

    #[test]
    fn test_parse_encode_memory_load_partial() {
        // Test i32.load8_s
        let bytes = vec![binary::I32_LOAD8_S, 0x01, 0x00]; // offset=1, align=0
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I32Load8S(1, 0));
        assert_eq!(bytes_read, 3);

        // Test i32.load8_u
        let bytes = vec![binary::I32_LOAD8_U, 0x01, 0x00]; // offset=1, align=0
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I32Load8U(1, 0));
        assert_eq!(bytes_read, 3);

        // Test i32.load16_s
        let bytes = vec![binary::I32_LOAD16_S, 0x01, 0x01]; // offset=1, align=1
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I32Load16S(1, 1));
        assert_eq!(bytes_read, 3);

        // Test i32.load16_u
        let bytes = vec![binary::I32_LOAD16_U, 0x01, 0x01]; // offset=1, align=1
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I32Load16U(1, 1));
        assert_eq!(bytes_read, 3);
    }

    #[test]
    fn test_parse_encode_memory_store() {
        // Test i32.store
        let bytes = vec![binary::I32_STORE, 0x01, 0x02]; // offset=1, align=2
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I32Store(1, 2));
        assert_eq!(bytes_read, 3);

        // Test i64.store
        let bytes = vec![binary::I64_STORE, 0x01, 0x03]; // offset=1, align=3
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I64Store(1, 3));
        assert_eq!(bytes_read, 3);

        // Test f32.store
        let bytes = vec![binary::F32_STORE, 0x01, 0x02]; // offset=1, align=2
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::F32Store(1, 2));
        assert_eq!(bytes_read, 3);

        // Test f64.store
        let bytes = vec![binary::F64_STORE, 0x01, 0x03]; // offset=1, align=3
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::F64Store(1, 3));
        assert_eq!(bytes_read, 3);
    }

    #[test]
    fn test_parse_encode_memory_store_partial() {
        // Test i32.store8
        let bytes = vec![binary::I32_STORE8, 0x01, 0x00]; // offset=1, align=0
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I32Store8(1, 0));
        assert_eq!(bytes_read, 3);

        // Test i32.store16
        let bytes = vec![binary::I32_STORE16, 0x01, 0x01]; // offset=1, align=1
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I32Store16(1, 1));
        assert_eq!(bytes_read, 3);

        // Test i64.store8
        let bytes = vec![binary::I64_STORE8, 0x01, 0x00]; // offset=1, align=0
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I64Store8(1, 0));
        assert_eq!(bytes_read, 3);

        // Test i64.store16
        let bytes = vec![binary::I64_STORE16, 0x01, 0x01]; // offset=1, align=1
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I64Store16(1, 1));
        assert_eq!(bytes_read, 3);

        // Test i64.store32
        let bytes = vec![binary::I64_STORE32, 0x01, 0x02]; // offset=1, align=2
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::I64Store32(1, 2));
        assert_eq!(bytes_read, 3);
    }

    #[test]
    fn test_parse_encode_memory_size_grow() {
        // Test memory.size
        let bytes = vec![binary::MEMORY_SIZE];
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::MemorySize);
        assert_eq!(bytes_read, 1);

        // Test memory.grow
        let bytes = vec![binary::MEMORY_GROW];
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::MemoryGrow);
        assert_eq!(bytes_read, 1);
    }

    #[test]
    fn test_parse_encode_memory_copy() {
        let bytes = vec![binary::MEMORY_COPY, 0x01, 0x02]; // dst_memory_idx=1, src_memory_idx=2
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::MemoryCopy(1, 2));
        assert_eq!(bytes_read, 3);

        let encoded = encode_instruction(&instruction).unwrap();
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn test_parse_encode_memory_fill() {
        let bytes = vec![binary::MEMORY_FILL, 0x01]; // memory_idx=1
        let (instruction, bytes_read) = parse_instruction(&bytes).unwrap();
        assert_eq!(instruction, Instruction::MemoryFill(1));
        assert_eq!(bytes_read, 2);

        let encoded = encode_instruction(&instruction).unwrap();
        assert_eq!(encoded, bytes);
    }
}
