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

        // Block, loop, if instructions
        binary::BLOCK => {
            let pos = 1;
            let (block_type, bt_bytes) = crate::types::parse_block_type(bytes, pos)?;
            let pos = pos + bt_bytes;

            let mut instructions = Vec::new();
            let mut current_pos = pos;

            // Parse instructions until we hit an END opcode
            while current_pos < bytes.len() && bytes[current_pos] != binary::END {
                // Handle ELSE opcode for if blocks
                if bytes[current_pos] == binary::ELSE {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected ELSE opcode in block".to_string(),
                    )));
                }

                let (instruction, bytes_read) = parse_instruction(&bytes[current_pos..])?;
                instructions.push(instruction);
                current_pos += bytes_read;
            }

            // Skip the END opcode
            if current_pos < bytes.len() && bytes[current_pos] == binary::END {
                current_pos += 1;
            } else {
                return Err(Error::new(kinds::ParseError(
                    "Missing END opcode for block".to_string(),
                )));
            }

            Ok((Instruction::Block(block_type, instructions), current_pos))
        }
        binary::LOOP => {
            let pos = 1;
            let (block_type, bt_bytes) = crate::types::parse_block_type(bytes, pos)?;
            let pos = pos + bt_bytes;

            let mut instructions = Vec::new();
            let mut current_pos = pos;

            // Parse instructions until we hit an END opcode
            while current_pos < bytes.len() && bytes[current_pos] != binary::END {
                // Handle ELSE opcode for if blocks
                if bytes[current_pos] == binary::ELSE {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected ELSE opcode in loop".to_string(),
                    )));
                }

                let (instruction, bytes_read) = parse_instruction(&bytes[current_pos..])?;
                instructions.push(instruction);
                current_pos += bytes_read;
            }

            // Skip the END opcode
            if current_pos < bytes.len() && bytes[current_pos] == binary::END {
                current_pos += 1;
            } else {
                return Err(Error::new(kinds::ParseError(
                    "Missing END opcode for loop".to_string(),
                )));
            }

            Ok((Instruction::Loop(block_type, instructions), current_pos))
        }
        binary::IF => {
            let pos = 1;
            let (block_type, bt_bytes) = crate::types::parse_block_type(bytes, pos)?;
            let pos = pos + bt_bytes;

            let mut then_instructions = Vec::new();
            let mut else_instructions = Vec::new();
            let mut current_pos = pos;
            let mut found_else = false;

            // Parse instructions until we hit an ELSE or END opcode
            while current_pos < bytes.len() && bytes[current_pos] != binary::END {
                if bytes[current_pos] == binary::ELSE {
                    if found_else {
                        return Err(Error::new(kinds::ParseError(
                            "Multiple ELSE opcodes in if block".to_string(),
                        )));
                    }
                    found_else = true;
                    current_pos += 1;
                    continue;
                }

                let (instruction, bytes_read) = parse_instruction(&bytes[current_pos..])?;
                if found_else {
                    else_instructions.push(instruction);
                } else {
                    then_instructions.push(instruction);
                }
                current_pos += bytes_read;
            }

            // Skip the END opcode
            if current_pos < bytes.len() && bytes[current_pos] == binary::END {
                current_pos += 1;
            } else {
                return Err(Error::new(kinds::ParseError(
                    "Missing END opcode for if".to_string(),
                )));
            }

            Ok((
                Instruction::If(block_type, then_instructions, else_instructions),
                current_pos,
            ))
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
            let mut offset = 1;

            // Read the vector of label indices
            let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut labels = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let (label, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                labels.push(label);
                offset += bytes_read;
            }

            // Read the default label
            let (default_label, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            Ok((Instruction::BrTable(labels, default_label), offset))
        }
        binary::RETURN => Ok((Instruction::Return, 1)),

        binary::CALL => {
            let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, 1)?;
            Ok((Instruction::Call(func_idx), 1 + bytes_read))
        }

        // TODO: Implement call_indirect
        binary::CALL_INDIRECT => {
            let mut offset = 1;

            // Read the type index
            let (type_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read the table index (currently always 0 in MVP, but encoded for future compatibility)
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of call_indirect instruction".to_string(),
                )));
            }

            let table_idx = bytes[offset];
            offset += 1;

            Ok((Instruction::CallIndirect(type_idx, table_idx), offset))
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
                BlockType::FuncType(type_idx) => {
                    bytes.extend_from_slice(&binary::write_leb128_i32(*type_idx as i32))
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
                BlockType::FuncType(type_idx) => {
                    bytes.extend_from_slice(&binary::write_leb128_i32(*type_idx as i32))
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
                BlockType::FuncType(type_idx) => {
                    bytes.extend_from_slice(&binary::write_leb128_i32(*type_idx as i32))
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
}
