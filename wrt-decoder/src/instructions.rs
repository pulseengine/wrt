//! WebAssembly instruction handling
//!
//! This module provides types and functions for parsing and encoding WebAssembly instructions.

use crate::prelude::*;
use crate::types::BlockType;
use wrt_error::{kinds, Error, Result};
use wrt_format::binary;

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

        // TODO: Implement block, loop, if
        binary::BR => {
            let (label_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::Br(label_idx), 1 + bytes_read))
        }
        binary::BR_IF => {
            let (label_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::BrIf(label_idx), 1 + bytes_read))
        }

        // TODO: Implement br_table
        binary::RETURN => Ok((Instruction::Return, 1)),

        binary::CALL => {
            let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::Call(func_idx), 1 + bytes_read))
        }

        // TODO: Implement call_indirect

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
        // TODO: Implement memory instructions

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

        // TODO: Implement other numeric instructions

        // Handle unsupported opcodes
        _ => Err(Error::new(kinds::ParseError(format!(
            "Unsupported instruction opcode: 0x{:02x}",
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
    let mut result = Vec::new();

    match instruction {
        // Control instructions
        Instruction::Unreachable => result.push(binary::UNREACHABLE),
        Instruction::Nop => result.push(binary::NOP),

        // TODO: Implement block, if, loop
        // TODO: Implement br, br_if, br_table
        // TODO: Implement call, call_indirect
        // TODO: Implement drop, select

        // Memory instructions
        // TODO: Implement memory instructions

        // Numeric instructions
        // TODO: Implement numeric instructions

        // Handle unsupported instructions
        _ => {
            return Err(Error::new(kinds::ParseError(format!(
                "Encoding not yet implemented for instruction: {:?}",
                instruction
            ))))
        }
    }

    Ok(result)
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

    #[test]
    fn test_parse_encode_i32_const() {
        let test_values = [0, 1, -1, 42, -42, 0x7FFFFFFF, -0x80000000];

        for &value in &test_values {
            // Create instruction
            let instr = Instruction::I32Const(value);

            // Encode
            let encoded = encode_instruction(&instr).unwrap();

            // Parse back
            let (decoded, _) = parse_instruction(&encoded).unwrap();

            // Verify
            assert_eq!(instr, decoded);
        }
    }

    #[test]
    fn test_parse_encode_call() {
        let test_values = [0, 1, 42, 0x10000];

        for &value in &test_values {
            // Create instruction
            let instr = Instruction::Call(value);

            // Encode
            let encoded = encode_instruction(&instr).unwrap();

            // Parse back
            let (decoded, _) = parse_instruction(&encoded).unwrap();

            // Verify
            assert_eq!(instr, decoded);
        }
    }
}
