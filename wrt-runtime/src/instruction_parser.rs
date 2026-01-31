//! Instruction parser for converting WebAssembly bytecode to runtime
//! instructions
//!
//! This module bridges the gap between raw bytecode from the parser and
//! the parsed instruction format expected by the runtime execution engine.

use wrt_error::{
    Error,
    ErrorCategory,
    Result,
};
use wrt_foundation::{
    bounded::BoundedVec,
    budget_aware_provider::CrateId,
    safe_managed_alloc,
    safe_memory::NoStdProvider,
    types::{
        BlockType,
        CatchHandler,
        Instruction,
        MemArg,
        MAX_CATCH_HANDLERS,
    },
};

// Type aliases for capability-based memory allocation
use crate::bounded_runtime_infra::{
    create_runtime_provider,
    RuntimeProvider,
};
type InstructionProvider = RuntimeProvider;

// Match WrtExpr type: Vec in std mode, BoundedVec in no_std mode
#[cfg(feature = "std")]
type InstructionVec = Vec<Instruction<InstructionProvider>>;
#[cfg(not(feature = "std"))]
type InstructionVec = BoundedVec<Instruction<InstructionProvider>, 1024, InstructionProvider>;

type TargetVec = BoundedVec<u32, 256, InstructionProvider>;

/// Parse WebAssembly bytecode into runtime instructions with a provided memory provider
pub fn parse_instructions_with_provider(
    bytecode: &[u8],
    provider: InstructionProvider
) -> Result<InstructionVec> {
    // Validate that bytecode is not empty - WebAssembly requires at least an End instruction
    if bytecode.is_empty() {
        return Err(Error::parse_error("Empty bytecode - WebAssembly requires at least an End instruction"));
    }

    let provider_clone = provider.clone();

    #[cfg(feature = "std")]
    let mut instructions = Vec::new();
    #[cfg(not(feature = "std"))]
    let mut instructions = BoundedVec::new(provider)
        .map_err(|_| Error::memory_error("Failed to allocate instruction vector"))?;

    let mut offset = 0;

    #[cfg(feature = "std")]
    let mut instruction_count = 0u32;
    #[cfg(feature = "std")]
    static mut FUNC_COUNTER: u32 = 0;
    #[cfg(feature = "std")]
    let func_id = unsafe {
        FUNC_COUNTER += 1;
        FUNC_COUNTER
    };

    #[cfg(feature = "tracing")]
    if func_id == 34 || func_id == 44 {
        wrt_foundation::tracing::trace!(func_id = func_id, bytecode_len = bytecode.len(), "Starting parse");
    }

    // WebAssembly function bodies should end with 0x0B (End)
    // Parse until we reach the end of bytecode - the last byte should be 0x0B
    while offset < bytecode.len() {
        #[cfg(feature = "tracing")]
        {
            instruction_count += 1;
            if instruction_count % 1000 == 0 {
                wrt_foundation::tracing::trace!(func_id = func_id, instruction_count = instruction_count, offset = offset, "Parsing progress");
            }
            if instruction_count > 50000 {
                wrt_foundation::tracing::error!(func_id = func_id, instruction_count = instruction_count, "Function appears stuck in infinite loop");
                return Err(Error::parse_error("Function parsing appears stuck in infinite loop"));
            }
        }

        let (instruction, consumed) = parse_instruction_with_provider(bytecode, offset, &provider_clone)?;

        #[cfg(feature = "tracing")]
        if consumed == 0 {
            wrt_foundation::tracing::error!(offset = offset, opcode = format!("0x{:02X}", bytecode[offset]), "Instruction consumed 0 bytes");
            return Err(Error::parse_error("Instruction consumed 0 bytes"));
        }

        #[cfg(feature = "std")]
        instructions.push(instruction.clone());
        #[cfg(not(feature = "std"))]
        instructions
            .push(instruction.clone())
            .map_err(|_| Error::capacity_limit_exceeded("Too many instructions in function"))?;

        offset += consumed;

        // Check if this was the final End instruction
        // The function body ends when we've consumed all bytecode and the last instruction was End
        if matches!(instruction, Instruction::End) && offset >= bytecode.len() {
            #[cfg(feature = "tracing")]
            if func_id == 34 || func_id == 44 {
                wrt_foundation::tracing::trace!(func_id = func_id, offset = offset, "Hit final End, done parsing");
            }
            break;
        }
    }

    Ok(instructions)
}

/// Parse a single instruction from bytecode (backward-compatible wrapper)
fn parse_instruction(
    bytecode: &[u8],
    offset: usize,
) -> Result<(Instruction<InstructionProvider>, usize)> {
    let provider = create_runtime_provider()?;
    parse_instruction_with_provider(bytecode, offset, &provider)
}

/// Parse WebAssembly bytecode into runtime instructions
/// 
/// This is a backward-compatible wrapper that creates its own provider.
pub fn parse_instructions(bytecode: &[u8]) -> Result<InstructionVec> {
    let provider = create_runtime_provider()?;
    parse_instructions_with_provider(bytecode, provider)
}

/// Parse a single instruction from bytecode with a provided memory provider
fn parse_instruction_with_provider(
    bytecode: &[u8],
    offset: usize,
    provider: &InstructionProvider,
) -> Result<(Instruction<InstructionProvider>, usize)> {
    if offset >= bytecode.len() {
        return Err(Error::parse_error("Unexpected end of bytecode"));
    }

    let opcode = bytecode[offset];
    let mut consumed = 1;

    // Removed per-opcode debug logging for performance

    let instruction = match opcode {
        // Control instructions
        0x00 => Instruction::Unreachable,
        0x01 => Instruction::Nop,
        0x02 => {
            // Block with block type
            let block_type = parse_block_type(bytecode, offset + 1)?;
            consumed += 1; // Simplified - actual block type parsing may consume more
            let block_type_idx = block_type_to_index(&block_type);
            Instruction::Block { block_type_idx }
        },
        0x03 => {
            // Loop with block type
            let block_type = parse_block_type(bytecode, offset + 1)?;
            consumed += 1;
            let block_type_idx = block_type_to_index(&block_type);
            Instruction::Loop { block_type_idx }
        },
        0x04 => {
            // If with block type
            let block_type = parse_block_type(bytecode, offset + 1)?;
            consumed += 1;
            let block_type_idx = block_type_to_index(&block_type);
            Instruction::If { block_type_idx }
        },
        0x05 => Instruction::Else,
        // Exception handling instructions (exception handling proposal)
        0x06 => {
            // try (legacy) - takes block type
            let block_type = parse_block_type(bytecode, offset + 1)?;
            consumed += 1;
            let block_type_idx = block_type_to_index(&block_type);
            Instruction::Try { block_type_idx }
        },
        0x07 => {
            // catch (legacy) - takes tag index
            let (tag_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::Catch(tag_idx)
        },
        0x08 => {
            // throw - takes tag index
            let (tag_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::Throw(tag_idx)
        },
        0x09 => {
            // rethrow (legacy) - takes relative depth to try block
            let (depth, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::Rethrow(depth)
        },
        0x0A => Instruction::ThrowRef,
        0x0B => Instruction::End,
        0x0C => {
            // Br (branch)
            let (label_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::Br(label_idx)
        },
        0x0D => {
            // BrIf (conditional branch)
            let (label_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::BrIf(label_idx)
        },
        0x0E => {
            // BrTable
            #[cfg(feature = "tracing")]
            wrt_foundation::tracing::trace!(offset = offset, "Parsing BrTable instruction");
            let mut targets = BoundedVec::new(provider.clone())
                .map_err(|_| Error::parse_error("Failed to create BrTable targets vector"))?;

            let (count, mut bytes_consumed) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes_consumed;

            #[cfg(feature = "tracing")]
            wrt_foundation::tracing::trace!(count = count, "BrTable target count");

            // Sanity check - if count is suspiciously large, there's likely an issue
            if count > 10000 {
                return Err(Error::parse_error("BrTable has suspiciously large target count"));
            }

            // Parse all target labels
            for i in 0..count {
                let (target, bytes) = read_leb128_u32(bytecode, offset + consumed)?;
                consumed += bytes;
                #[cfg(feature = "tracing")]
                if i < 3 || i == count - 1 {
                    wrt_foundation::tracing::trace!(index = i, target = target, "BrTable target");
                }
                targets
                    .push(target)
                    .map_err(|_| Error::parse_error("Too many BrTable targets"))?;
            }

            // Parse default target
            let (default_target, bytes) = read_leb128_u32(bytecode, offset + consumed)?;
            consumed += bytes;

            Instruction::BrTable {
                targets,
                default_target,
            }
        },
        0x0F => Instruction::Return,
        0x10 => {
            // Call
            let (func_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::Call(func_idx)
        },
        0x11 => {
            // CallIndirect: type_idx (LEB128 u32) followed by table_idx (LEB128 u32)
            // Note: table_idx can be multi-byte LEB128, not always single byte!
            let (type_idx, type_bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += type_bytes;
            let (table_idx, table_bytes) = read_leb128_u32(bytecode, offset + 1 + type_bytes)?;
            consumed += table_bytes;
            Instruction::CallIndirect(type_idx, table_idx)
        },
        0x12 => {
            // ReturnCall (tail-call extension): func_idx (LEB128 u32)
            let (func_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::ReturnCall(func_idx)
        },
        0x13 => {
            // ReturnCallIndirect (tail-call extension): type_idx (LEB128 u32) followed by table_idx (LEB128 u32)
            let (type_idx, type_bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += type_bytes;
            let (table_idx, table_bytes) = read_leb128_u32(bytecode, offset + 1 + type_bytes)?;
            consumed += table_bytes;
            Instruction::ReturnCallIndirect(type_idx, table_idx)
        },

        // Exception handling instructions (continued)
        0x18 => {
            // delegate (legacy) - takes relative depth
            let (depth, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::Delegate(depth)
        },
        0x19 => Instruction::CatchAll,
        0x1F => {
            // try_table - takes block type + catch handler list
            let block_type = parse_block_type(bytecode, offset + 1)?;
            consumed += 1;
            let block_type_idx = block_type_to_index(&block_type);

            // Parse handler count
            let (handler_count, handler_bytes) = read_leb128_u32(bytecode, offset + consumed)?;
            consumed += handler_bytes;

            // Parse catch handlers
            let mut handlers = BoundedVec::new(provider.clone())
                .map_err(|_| Error::parse_error("Failed to create catch handlers vector"))?;

            for _ in 0..handler_count {
                // Each handler has: catch_kind (1 byte), then tag_idx (if applicable), then label
                if offset + consumed >= bytecode.len() {
                    return Err(Error::parse_error("Unexpected end in try_table handlers"));
                }
                let catch_kind = bytecode[offset + consumed];
                consumed += 1;

                let handler = match catch_kind {
                    0x00 => {
                        // catch: tag_idx + label
                        let (tag_idx, bytes) = read_leb128_u32(bytecode, offset + consumed)?;
                        consumed += bytes;
                        let (label, bytes) = read_leb128_u32(bytecode, offset + consumed)?;
                        consumed += bytes;
                        CatchHandler::Catch { tag_idx, label }
                    },
                    0x01 => {
                        // catch_ref: tag_idx + label
                        let (tag_idx, bytes) = read_leb128_u32(bytecode, offset + consumed)?;
                        consumed += bytes;
                        let (label, bytes) = read_leb128_u32(bytecode, offset + consumed)?;
                        consumed += bytes;
                        CatchHandler::CatchRef { tag_idx, label }
                    },
                    0x02 => {
                        // catch_all: label only
                        let (label, bytes) = read_leb128_u32(bytecode, offset + consumed)?;
                        consumed += bytes;
                        CatchHandler::CatchAll { label }
                    },
                    0x03 => {
                        // catch_all_ref: label only
                        let (label, bytes) = read_leb128_u32(bytecode, offset + consumed)?;
                        consumed += bytes;
                        CatchHandler::CatchAllRef { label }
                    },
                    _ => return Err(Error::parse_error("Invalid catch handler kind in try_table")),
                };

                handlers.push(handler)
                    .map_err(|_| Error::parse_error("Too many catch handlers in try_table"))?;
            }

            Instruction::TryTable {
                block_type_idx,
                handlers,
            }
        },

        // Parametric instructions
        0x1A => Instruction::Drop,
        0x1B => Instruction::Select,

        // Variable instructions
        0x20 => {
            let (local_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::LocalGet(local_idx)
        },
        0x21 => {
            let (local_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::LocalSet(local_idx)
        },
        0x22 => {
            let (local_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::LocalTee(local_idx)
        },
        0x23 => {
            let (global_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::GlobalGet(global_idx)
        },
        0x24 => {
            let (global_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::GlobalSet(global_idx)
        },

        // Table instructions
        0x25 => {
            // table.get
            let (table_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::TableGet(table_idx)
        },
        0x26 => {
            // table.set
            let (table_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::TableSet(table_idx)
        },

        // Memory instructions
        0x28 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I32Load(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x29 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I64Load(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x2A => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::F32Load(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x2B => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::F64Load(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x2C => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I32Load8S(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x2D => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I32Load8U(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x2E => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I32Load16S(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x2F => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I32Load16U(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x30 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I64Load8S(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x31 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I64Load8U(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x32 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I64Load16S(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x33 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I64Load16U(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x34 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I64Load32S(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x35 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I64Load32U(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x36 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I32Store(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x37 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I64Store(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x38 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::F32Store(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x39 => {
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::F64Store(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x3A => {
            // i32.store8
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I32Store8(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x3B => {
            // i32.store16
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I32Store16(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x3C => {
            // i64.store8
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I64Store8(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x3D => {
            // i64.store16
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I64Store16(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x3E => {
            // i64.store32
            let (align, bytes1) = read_leb128_u32(bytecode, offset + 1)?;
            let (offset, bytes2) = read_leb128_u32(bytecode, offset + 1 + bytes1)?;
            consumed += bytes1 + bytes2;
            Instruction::I64Store32(MemArg {
                align_exponent: align,
                offset,
                memory_index: 0,
            })
        },
        0x3F => {
            consumed += 1; // Skip reserved byte
            Instruction::MemorySize(0)
        },
        0x40 => {
            consumed += 1; // Skip reserved byte
            Instruction::MemoryGrow(0)
        },

        // Numeric instructions - Constants
        0x41 => {
            let (value, bytes) = read_leb128_i32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::I32Const(value)
        },
        0x42 => {
            let (value, bytes) = read_leb128_i64(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::I64Const(value)
        },
        0x43 => {
            if offset + 5 > bytecode.len() {
                return Err(Error::parse_error("F32 constant extends beyond bytecode"));
            }
            let bytes = [
                bytecode[offset + 1],
                bytecode[offset + 2],
                bytecode[offset + 3],
                bytecode[offset + 4],
            ];
            let value = u32::from_le_bytes(bytes); // Use bit representation
            consumed += 4;
            Instruction::F32Const(value)
        },
        0x44 => {
            if offset + 9 > bytecode.len() {
                return Err(Error::parse_error("F64 constant extends beyond bytecode"));
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&bytecode[offset + 1..offset + 9]);
            let value = u64::from_le_bytes(bytes); // Use bit representation
            consumed += 8;
            Instruction::F64Const(value)
        },

        // Numeric instructions - i32 operations
        0x67 => Instruction::I32Clz,     // Count leading zeros
        0x68 => Instruction::I32Ctz,     // Count trailing zeros
        0x69 => Instruction::I32Popcnt,  // Population count
        0x6A => Instruction::I32Add,
        0x6B => Instruction::I32Sub,
        0x6C => Instruction::I32Mul,
        0x6D => Instruction::I32DivS,
        0x6E => Instruction::I32DivU,
        0x6F => Instruction::I32RemS,
        0x70 => Instruction::I32RemU,
        0x71 => Instruction::I32And,
        0x72 => Instruction::I32Or,
        0x73 => Instruction::I32Xor,
        0x74 => Instruction::I32Shl,
        0x75 => Instruction::I32ShrS,
        0x76 => Instruction::I32ShrU,
        0x77 => Instruction::I32Rotl,
        0x78 => Instruction::I32Rotr,

        // Comparison
        0x45 => Instruction::I32Eqz,
        0x46 => Instruction::I32Eq,
        0x47 => Instruction::I32Ne,
        0x48 => Instruction::I32LtS,
        0x49 => Instruction::I32LtU,
        0x4A => Instruction::I32GtS,
        0x4B => Instruction::I32GtU,
        0x4C => Instruction::I32LeS,
        0x4D => Instruction::I32LeU,
        0x4E => Instruction::I32GeS,
        0x4F => Instruction::I32GeU,

        // i64 comparisons
        0x50 => Instruction::I64Eqz,
        0x51 => Instruction::I64Eq,
        0x52 => Instruction::I64Ne,
        0x53 => Instruction::I64LtS,
        0x54 => Instruction::I64LtU,
        0x55 => Instruction::I64GtS,
        0x56 => Instruction::I64GtU,
        0x57 => Instruction::I64LeS,
        0x58 => Instruction::I64LeU,
        0x59 => Instruction::I64GeS,
        0x5A => Instruction::I64GeU,

        // F32 comparison operations
        0x5B => Instruction::F32Eq,
        0x5C => Instruction::F32Ne,
        0x5D => Instruction::F32Lt,
        0x5E => Instruction::F32Gt,
        0x5F => Instruction::F32Le,
        0x60 => Instruction::F32Ge,

        // F64 comparison operations
        0x61 => Instruction::F64Eq,
        0x62 => Instruction::F64Ne,
        0x63 => Instruction::F64Lt,
        0x64 => Instruction::F64Gt,
        0x65 => Instruction::F64Le,
        0x66 => Instruction::F64Ge,

        // i64 operations
        0x79 => Instruction::I64Clz,     // Count leading zeros
        0x7A => Instruction::I64Ctz,     // Count trailing zeros
        0x7B => Instruction::I64Popcnt,  // Population count
        0x7C => Instruction::I64Add,
        0x7D => Instruction::I64Sub,
        0x7E => Instruction::I64Mul,
        0x7F => Instruction::I64DivS,
        0x80 => Instruction::I64DivU,
        0x81 => Instruction::I64RemS,
        0x82 => Instruction::I64RemU,
        0x83 => Instruction::I64And,
        0x84 => Instruction::I64Or,
        0x85 => Instruction::I64Xor,
        0x86 => Instruction::I64Shl,
        0x87 => Instruction::I64ShrS,
        0x88 => Instruction::I64ShrU,
        0x89 => Instruction::I64Rotl,
        0x8A => Instruction::I64Rotr,

        // f32 unary operations
        0x8B => Instruction::F32Abs,
        0x8C => Instruction::F32Neg,
        0x8D => Instruction::F32Ceil,
        0x8E => Instruction::F32Floor,
        0x8F => Instruction::F32Trunc,
        0x90 => Instruction::F32Nearest,
        0x91 => Instruction::F32Sqrt,

        // f32 binary operations
        0x92 => Instruction::F32Add,
        0x93 => Instruction::F32Sub,
        0x94 => Instruction::F32Mul,
        0x95 => Instruction::F32Div,
        0x96 => Instruction::F32Min,
        0x97 => Instruction::F32Max,
        0x98 => Instruction::F32Copysign,

        // f64 unary operations
        0x99 => Instruction::F64Abs,
        0x9A => Instruction::F64Neg,
        0x9B => Instruction::F64Ceil,
        0x9C => Instruction::F64Floor,
        0x9D => Instruction::F64Trunc,
        0x9E => Instruction::F64Nearest,
        0x9F => Instruction::F64Sqrt,

        // f64 binary operations
        0xA0 => Instruction::F64Add,
        0xA1 => Instruction::F64Sub,
        0xA2 => Instruction::F64Mul,
        0xA3 => Instruction::F64Div,
        0xA4 => Instruction::F64Min,
        0xA5 => Instruction::F64Max,
        0xA6 => Instruction::F64Copysign,

        // Conversions
        0xA7 => Instruction::I32WrapI64,
        0xA8 => Instruction::I32TruncF32S,
        0xA9 => Instruction::I32TruncF32U,
        0xAA => Instruction::I32TruncF64S,
        0xAB => Instruction::I32TruncF64U,
        0xAC => Instruction::I64ExtendI32S,
        0xAD => Instruction::I64ExtendI32U,
        0xAE => Instruction::I64TruncF32S,
        0xAF => Instruction::I64TruncF32U,
        0xB0 => Instruction::I64TruncF64S,
        0xB1 => Instruction::I64TruncF64U,
        0xB2 => Instruction::F32ConvertI32S,
        0xB3 => Instruction::F32ConvertI32U,
        0xB4 => Instruction::F32ConvertI64S,
        0xB5 => Instruction::F32ConvertI64U,
        0xB6 => Instruction::F32DemoteF64,
        0xB7 => Instruction::F64ConvertI32S,
        0xB8 => Instruction::F64ConvertI32U,
        0xB9 => Instruction::F64ConvertI64S,
        0xBA => Instruction::F64ConvertI64U,
        0xBB => Instruction::F64PromoteF32,

        // Reinterpret instructions (bit casting)
        0xBC => Instruction::I32ReinterpretF32,
        0xBD => Instruction::I64ReinterpretF64,
        0xBE => Instruction::F32ReinterpretI32,
        0xBF => Instruction::F64ReinterpretI64,

        // Sign-extension operators (proposal, but commonly used)
        0xC0 => Instruction::I32Extend8S,   // Sign-extend 8-bit to 32-bit
        0xC1 => Instruction::I32Extend16S,  // Sign-extend 16-bit to 32-bit
        0xC2 => Instruction::I64Extend8S,   // Sign-extend 8-bit to 64-bit
        0xC3 => Instruction::I64Extend16S,  // Sign-extend 16-bit to 64-bit
        0xC4 => Instruction::I64Extend32S,  // Sign-extend 32-bit to 64-bit

        // Reference types (WebAssembly 2.0)
        0xD0 => {
            // ref.null ht - create a null reference of the specified heap type
            if offset + 1 >= bytecode.len() {
                return Err(Error::parse_error("Unexpected end of bytecode in ref.null"));
            }
            let heap_type = bytecode[offset + 1];
            consumed += 1;
            // Map heap type to RefType
            let ref_type = match heap_type {
                0x70 => wrt_foundation::types::RefType::Funcref,
                0x6F => wrt_foundation::types::RefType::Externref,
                _ => {
                    // For other heap types, default to Externref for now
                    wrt_foundation::types::RefType::Externref
                }
            };
            Instruction::RefNull(ref_type)
        },
        0xD1 => Instruction::RefIsNull,
        0xD2 => {
            // ref.func x - create a reference to function x
            let (func_idx, bytes) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes;
            Instruction::RefFunc(func_idx)
        },

        // GC instructions (0xFB prefix)
        0xFB => {
            // Read the subopcode (LEB128 encoded)
            let (subopcode, bytes_read) = read_leb128_u32(bytecode, offset + 1)?;
            consumed += bytes_read;

            match subopcode {
                // Struct operations
                0x00 => {
                    // struct.new: type_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::StructNew(type_idx)
                }
                0x01 => {
                    // struct.new_default: type_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::StructNewDefault(type_idx)
                }
                0x02 => {
                    // struct.get: type_idx, field_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (field_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::StructGet(type_idx, field_idx)
                }
                0x03 => {
                    // struct.get_s: type_idx, field_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (field_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::StructGetS(type_idx, field_idx)
                }
                0x04 => {
                    // struct.get_u: type_idx, field_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (field_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::StructGetU(type_idx, field_idx)
                }
                0x05 => {
                    // struct.set: type_idx, field_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (field_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::StructSet(type_idx, field_idx)
                }
                // Array operations
                0x06 => {
                    // array.new: type_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayNew(type_idx)
                }
                0x07 => {
                    // array.new_default: type_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayNewDefault(type_idx)
                }
                0x08 => {
                    // array.new_fixed: type_idx, length
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (length, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayNewFixed(type_idx, length)
                }
                0x09 => {
                    // array.new_data: type_idx, data_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (data_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayNewData(type_idx, data_idx)
                }
                0x0A => {
                    // array.new_elem: type_idx, elem_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (elem_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayNewElem(type_idx, elem_idx)
                }
                0x0B => {
                    // array.get: type_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayGet(type_idx)
                }
                0x0C => {
                    // array.get_s: type_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayGetS(type_idx)
                }
                0x0D => {
                    // array.get_u: type_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayGetU(type_idx)
                }
                0x0E => {
                    // array.set: type_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArraySet(type_idx)
                }
                0x0F => {
                    // array.len
                    Instruction::ArrayLen
                }
                0x10 => {
                    // array.fill: type_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayFill(type_idx)
                }
                0x11 => {
                    // array.copy: dst_type_idx, src_type_idx
                    let (dst_type, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (src_type, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayCopy(dst_type, src_type)
                }
                0x12 => {
                    // array.init_data: type_idx, data_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (data_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayInitData(type_idx, data_idx)
                }
                0x13 => {
                    // array.init_elem: type_idx, elem_idx
                    let (type_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (elem_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::ArrayInitElem(type_idx, elem_idx)
                }
                // Reference type testing/casting
                0x14 => {
                    // ref.test: heaptype
                    let (heap_type, bytes_read) = parse_heap_type(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::RefTest(heap_type)
                }
                0x15 => {
                    // ref.test null: heaptype
                    let (heap_type, bytes_read) = parse_heap_type(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::RefTestNull(heap_type)
                }
                0x16 => {
                    // ref.cast: heaptype
                    let (heap_type, bytes_read) = parse_heap_type(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::RefCast(heap_type)
                }
                0x17 => {
                    // ref.cast null: heaptype
                    let (heap_type, bytes_read) = parse_heap_type(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::RefCastNull(heap_type)
                }
                // Branch on cast
                0x18 => {
                    // br_on_cast: flags, label, from_type, to_type
                    if offset + consumed >= bytecode.len() {
                        return Err(Error::parse_error("Unexpected end in br_on_cast"));
                    }
                    let flags = bytecode[offset + consumed];
                    consumed += 1;
                    let (label, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (from_type, bytes_read) = parse_heap_type(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (to_type, bytes_read) = parse_heap_type(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::BrOnCast { flags, label, from_type, to_type }
                }
                0x19 => {
                    // br_on_cast_fail: flags, label, from_type, to_type
                    if offset + consumed >= bytecode.len() {
                        return Err(Error::parse_error("Unexpected end in br_on_cast_fail"));
                    }
                    let flags = bytecode[offset + consumed];
                    consumed += 1;
                    let (label, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (from_type, bytes_read) = parse_heap_type(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    let (to_type, bytes_read) = parse_heap_type(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::BrOnCastFail { flags, label, from_type, to_type }
                }
                // Extern/any conversions
                0x1A => Instruction::AnyConvertExtern,
                0x1B => Instruction::ExternConvertAny,
                // i31 operations
                0x1C => Instruction::RefI31,
                0x1D => Instruction::I31GetS,
                0x1E => Instruction::I31GetU,
                _ => {
                    #[cfg(feature = "tracing")]
                    wrt_foundation::tracing::warn!(subopcode = format!("0xFB 0x{:02X}", subopcode), offset = offset, "Unknown FB subopcode");
                    return Err(Error::parse_error("Unknown GC instruction"));
                }
            }
        }
        // Multi-byte opcodes (bulk memory, SIMD, etc.)
        0xFC => {
            // Read the second byte to determine the actual instruction
            if offset + 1 >= bytecode.len() {
                return Err(Error::parse_error("Unexpected end of bytecode in multi-byte opcode"));
            }
            let subopcode = bytecode[offset + 1];
            consumed += 1;  // For the subopcode byte

            match subopcode {
                // Saturating truncation operations (0xFC 0x00 - 0xFC 0x07)
                // These saturate (clamp) on overflow instead of trapping
                0x00 => Instruction::I32TruncSatF32S,
                0x01 => Instruction::I32TruncSatF32U,
                0x02 => Instruction::I32TruncSatF64S,
                0x03 => Instruction::I32TruncSatF64U,
                0x04 => Instruction::I64TruncSatF32S,
                0x05 => Instruction::I64TruncSatF32U,
                0x06 => Instruction::I64TruncSatF64S,
                0x07 => Instruction::I64TruncSatF64U,
                // Bulk memory operations
                0x08 => {
                    // memory.init: data_idx, mem_idx
                    let (data_idx, bytes_read) = read_leb128_u32(bytecode, offset + 2)?;
                    consumed += bytes_read;
                    // mem_idx is always 0 in MVP
                    if offset + consumed >= bytecode.len() {
                        return Err(Error::parse_error("Unexpected end in memory.init"));
                    }
                    let mem_idx = bytecode[offset + consumed];
                    consumed += 1;
                    Instruction::MemoryInit(data_idx, mem_idx as u32)
                }
                0x09 => {
                    // data.drop: data_idx
                    let (data_idx, bytes_read) = read_leb128_u32(bytecode, offset + 2)?;
                    consumed += bytes_read;
                    Instruction::DataDrop(data_idx)
                }
                0x0A => {
                    // memory.copy: dst_mem_idx, src_mem_idx
                    // Both are typically 0x00 in MVP
                    if offset + 3 >= bytecode.len() {
                        return Err(Error::parse_error("Unexpected end in memory.copy"));
                    }
                    let dst_mem = bytecode[offset + 2];
                    let src_mem = bytecode[offset + 3];
                    consumed += 2;
                    Instruction::MemoryCopy(dst_mem as u32, src_mem as u32)
                }
                0x0B => {
                    // memory.fill: mem_idx
                    if offset + 2 >= bytecode.len() {
                        return Err(Error::parse_error("Unexpected end in memory.fill"));
                    }
                    let mem_idx = bytecode[offset + 2];
                    consumed += 1;
                    Instruction::MemoryFill(mem_idx as u32)
                }
                // Table operations (bulk memory proposal)
                0x0C => {
                    // table.init: elem_idx, table_idx
                    let (elem_idx, bytes_read) = read_leb128_u32(bytecode, offset + 2)?;
                    consumed += bytes_read;
                    let (table_idx, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::TableInit(elem_idx, table_idx)
                }
                0x0D => {
                    // elem.drop: elem_idx
                    let (elem_idx, bytes_read) = read_leb128_u32(bytecode, offset + 2)?;
                    consumed += bytes_read;
                    Instruction::ElemDrop(elem_idx)
                }
                0x0E => {
                    // table.copy: dst_table_idx, src_table_idx
                    let (dst_table, bytes_read) = read_leb128_u32(bytecode, offset + 2)?;
                    consumed += bytes_read;
                    let (src_table, bytes_read) = read_leb128_u32(bytecode, offset + consumed)?;
                    consumed += bytes_read;
                    Instruction::TableCopy(dst_table, src_table)
                }
                0x0F => {
                    // table.grow: table_idx
                    let (table_idx, bytes_read) = read_leb128_u32(bytecode, offset + 2)?;
                    consumed += bytes_read;
                    Instruction::TableGrow(table_idx)
                }
                0x10 => {
                    // table.size: table_idx
                    let (table_idx, bytes_read) = read_leb128_u32(bytecode, offset + 2)?;
                    consumed += bytes_read;
                    Instruction::TableSize(table_idx)
                }
                0x11 => {
                    // table.fill: table_idx
                    let (table_idx, bytes_read) = read_leb128_u32(bytecode, offset + 2)?;
                    consumed += bytes_read;
                    Instruction::TableFill(table_idx)
                }
                _ => {
                    #[cfg(feature = "tracing")]
                    wrt_foundation::tracing::warn!(subopcode = format!("0xFC 0x{:02X}", subopcode), offset = offset, "Unknown FC subopcode");
                    return Err(Error::parse_error("Unknown multi-byte instruction"));
                }
            }
        }
        _ => {
            // Show context around the unknown opcode
            #[cfg(feature = "tracing")]
            {
                let context_start = offset.saturating_sub(5);
                let context_end = (offset + 10).min(bytecode.len());
                let context = &bytecode[context_start..context_end];
                wrt_foundation::tracing::warn!(opcode = format!("0x{:02X}", opcode), offset = offset, "Unknown opcode");
                wrt_foundation::tracing::trace!(context = ?context, "Bytecode context");
            }
            return Err(Error::parse_error("Unknown instruction opcode"));
        },
    };

    Ok((instruction, consumed))
}

/// Parse a block type
///
/// Block types in WebAssembly are encoded as:
/// - 0x40: empty type (no results)
/// - Value type bytes (0x7F=i32, 0x7E=i64, 0x7D=f32, 0x7C=f64, etc.): single result type
/// - Otherwise: type index encoded as s33 (signed 33-bit LEB128)
fn parse_block_type(bytecode: &[u8], offset: usize) -> Result<BlockType> {
    if offset >= bytecode.len() {
        return Err(Error::parse_error(
            "Unexpected end while parsing block type",
        ));
    }

    let b = bytecode[offset];

    // Check for specific value type encodings first
    match b {
        0x40 => Ok(BlockType::Value(None)), // Empty type
        0x7F => Ok(BlockType::Value(Some(wrt_foundation::types::ValueType::I32))),
        0x7E => Ok(BlockType::Value(Some(wrt_foundation::types::ValueType::I64))),
        0x7D => Ok(BlockType::Value(Some(wrt_foundation::types::ValueType::F32))),
        0x7C => Ok(BlockType::Value(Some(wrt_foundation::types::ValueType::F64))),
        0x7B => Ok(BlockType::Value(Some(wrt_foundation::types::ValueType::V128))),
        0x70 => Ok(BlockType::Value(Some(wrt_foundation::types::ValueType::FuncRef))),
        0x6F => Ok(BlockType::Value(Some(wrt_foundation::types::ValueType::ExternRef))),
        0x69 => Ok(BlockType::Value(Some(wrt_foundation::types::ValueType::ExnRef))),
        _ => {
            // Type index: parse as s33 (for small positive values, it's just the byte)
            // For now, handle single-byte type indices (0-63)
            if b & 0x80 == 0 {
                // Single byte LEB128 - the value is the type index
                Ok(BlockType::FuncType(b as u32))
            } else {
                // Multi-byte LEB128 - parse as signed LEB128
                // For simplicity, treat as empty for now (rare case)
                // TODO: Implement full s33 parsing for large type indices
                Ok(BlockType::Value(None))
            }
        },
    }
}

/// Read a LEB128 encoded u32
pub(crate) fn read_leb128_u32(data: &[u8], offset: usize) -> Result<(u32, usize)> {
    let mut result = 0u32;
    let mut shift = 0;
    let mut consumed = 0;

    loop {
        if offset + consumed >= data.len() {
            return Err(Error::parse_error(
                "Unexpected end of data while reading LEB128",
            ));
        }

        let byte = data[offset + consumed];
        consumed += 1;

        result |= ((byte & 0x7F) as u32) << shift;

        if byte & 0x80 == 0 {
            break;
        }

        shift += 7;
        if shift >= 32 {
            return Err(Error::parse_error("LEB128 value too large for u32"));
        }
    }

    Ok((result, consumed))
}

/// Read a LEB128 encoded i32
pub(crate) fn read_leb128_i32(data: &[u8], offset: usize) -> Result<(i32, usize)> {
    let mut result = 0i32;
    let mut shift = 0;
    let mut consumed = 0;
    let mut byte;

    loop {
        if offset + consumed >= data.len() {
            return Err(Error::parse_error(
                "Unexpected end of data while reading LEB128",
            ));
        }

        byte = data[offset + consumed];
        consumed += 1;

        result |= ((byte & 0x7F) as i32) << shift;
        shift += 7;

        if byte & 0x80 == 0 {
            break;
        }
    }

    // Sign extend
    if shift < 32 && (byte & 0x40) != 0 {
        result |= !0 << shift;
    }

    Ok((result, consumed))
}

/// Read a LEB128 encoded i64
pub(crate) fn read_leb128_i64(data: &[u8], offset: usize) -> Result<(i64, usize)> {
    let mut result = 0i64;
    let mut shift = 0;
    let mut consumed = 0;
    let mut byte;

    loop {
        if offset + consumed >= data.len() {
            return Err(Error::parse_error(
                "Unexpected end of data while reading LEB128",
            ));
        }

        byte = data[offset + consumed];
        consumed += 1;

        result |= ((byte & 0x7F) as i64) << shift;
        shift += 7;

        if byte & 0x80 == 0 {
            break;
        }
    }

    // Sign extend
    if shift < 64 && (byte & 0x40) != 0 {
        result |= !0 << shift;
    }

    Ok((result, consumed))
}

/// Convert BlockType to a type index for instruction storage
fn block_type_to_index(block_type: &BlockType) -> u32 {
    match block_type {
        BlockType::Value(None) => 0x40, // Empty type
        BlockType::Value(Some(wrt_foundation::types::ValueType::I32)) => 0x7F,
        BlockType::Value(Some(wrt_foundation::types::ValueType::I64)) => 0x7E,
        BlockType::Value(Some(wrt_foundation::types::ValueType::F32)) => 0x7D,
        BlockType::Value(Some(wrt_foundation::types::ValueType::F64)) => 0x7C,
        BlockType::Value(Some(wrt_foundation::types::ValueType::V128)) => 0x7B,
        BlockType::Value(Some(wrt_foundation::types::ValueType::I16x8)) => 0x7A,
        BlockType::Value(Some(wrt_foundation::types::ValueType::FuncRef)) => 0x70,
        BlockType::Value(Some(wrt_foundation::types::ValueType::ExternRef)) => 0x6F,
        BlockType::Value(Some(wrt_foundation::types::ValueType::ExnRef)) => 0x69,
        BlockType::FuncType(idx) => *idx,
        // Handle any other value types with a default
        BlockType::Value(Some(_)) => 0x40, // Default to empty type for unknown types
    }
}
