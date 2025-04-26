//! WebAssembly instruction implementations
//!
//! This module contains implementations for all WebAssembly instructions,
//! organized into submodules by instruction category.

// Only include the imports actually needed in this file

pub mod arithmetic;
mod bit_counting;
pub mod comparison;
pub mod control;
pub mod instruction_type;
mod memory;
pub mod numeric;
mod parametric;
mod refs;
pub mod simd;
pub mod table;
pub mod variable;

pub mod types {
    pub use crate::types::BlockType;
}

// Export only the instruction type
pub use instruction_type::Instruction;

// Use only the imports needed in this file
use crate::{
    behavior::{ControlFlow, FrameBehavior, InstructionExecutor, StackBehavior},
    error::{kinds, Error, Result},
    instructions_adapter::execute_pure_instruction,
    memory::{DataDrop, Load, LoadSigned, LoadUnsigned, MemoryInit, Store, StoreTruncated},
    stackless::StacklessEngine,
    values::Value,
};

// Import pure instruction implementations
use wrt_instructions::{
    arithmetic_ops::ArithmeticOp,
    comparison_ops::ComparisonOp,
    control_ops::ControlOp,
    memory_ops::{MemoryLoad, MemoryStore},
    table_ops::TableOp,
    variable_ops::VariableOp,
};

impl InstructionExecutor for Instruction {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow, crate::error::Error> {
        // Delegate to the specific instruction implementation
        match self {
            // Control Instructions
            Instruction::Unreachable => control::Unreachable.execute(stack, frame, engine),
            Instruction::Nop => control::Nop.execute(stack, frame, engine),
            Instruction::Block(block_type) => {
                control::Block::new(block_type.clone()).execute(stack, frame, engine)
            }
            Instruction::Loop(block_type) => {
                control::Loop::new(block_type.clone()).execute(stack, frame, engine)
            }
            Instruction::If(block_type) => {
                control::If::new(block_type.clone()).execute(stack, frame, engine)
            }
            Instruction::Else => control::Else.execute(stack, frame, engine),
            Instruction::End => control::End.execute(stack, frame, engine),
            Instruction::Br(depth) => control::Br::new(*depth).execute(stack, frame, engine),
            Instruction::BrIf(depth) => control::BrIf::new(*depth).execute(stack, frame, engine),
            Instruction::BrTable(table, default) => {
                control::BrTable::new(table.clone(), *default).execute(stack, frame, engine)
            }
            Instruction::Return => control::Return::default().execute(stack, frame, engine),
            Instruction::Call(func_idx) => {
                control::Call::new(*func_idx).execute(stack, frame, engine)
            }
            Instruction::CallIndirect(type_idx, table_idx) => {
                control::CallIndirect::new(*type_idx, *table_idx).execute(stack, frame, engine)
            }

            // Parametric Instructions
            Instruction::Drop => parametric::Drop.execute(stack, frame, engine),
            Instruction::Select => parametric::Select.execute(stack, frame, engine),
            Instruction::SelectTyped(types) => {
                parametric::SelectTyped::new(types.clone()).execute(stack, frame, engine)
            }

            // Variable Instructions
            Instruction::LocalGet(idx) => {
                variable::LocalGet::new(*idx).execute(stack, frame, engine)
            }
            Instruction::LocalSet(idx) => {
                variable::LocalSet::new(*idx).execute(stack, frame, engine)
            }
            Instruction::LocalTee(idx) => {
                variable::LocalTee::new(*idx).execute(stack, frame, engine)
            }
            Instruction::GlobalGet(idx) => {
                variable::GlobalGet::new(*idx).execute(stack, frame, engine)
            }
            Instruction::GlobalSet(idx) => {
                variable::GlobalSet::new(*idx).execute(stack, frame, engine)
            }

            // Memory Instructions - using imported memory types
            Instruction::I32Load(offset, align) => {
                Load::i32(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I64Load(offset, align) => {
                Load::i64(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::F32Load(offset, align) => {
                Load::f32(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::F64Load(offset, align) => {
                Load::f64(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I32Load8S(offset, align) => {
                LoadSigned::i8_i32(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I32Load8U(offset, align) => {
                LoadUnsigned::u8_i32(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I32Load16S(offset, align) => {
                LoadSigned::i16_i32(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I32Load16U(offset, align) => {
                LoadUnsigned::u16_i32(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I64Load8S(offset, align) => {
                LoadSigned::i8_i64(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I64Load8U(offset, align) => {
                LoadUnsigned::u8_i64(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I64Load16S(offset, align) => {
                LoadSigned::i16_i64(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I64Load16U(offset, align) => {
                LoadUnsigned::u16_i64(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I64Load32S(offset, align) => {
                LoadSigned::i32_i64(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I64Load32U(offset, align) => {
                LoadUnsigned::u32_i64(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I32Store(offset, align) => {
                Store::i32(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I64Store(offset, align) => {
                Store::i64(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::F32Store(offset, align) => {
                Store::f32(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::F64Store(offset, align) => {
                Store::f64(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I32Store8(offset, align) => {
                StoreTruncated::<i32, i8>::new(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I32Store16(offset, align) => {
                StoreTruncated::<i32, i16>::new(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I64Store8(offset, align) => {
                StoreTruncated::<i64, i8>::new(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I64Store16(offset, align) => {
                StoreTruncated::<i64, i16>::new(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::I64Store32(offset, align) => {
                StoreTruncated::<i64, i32>::new(*offset, *align).execute(stack, frame, engine)
            }
            Instruction::MemorySize(mem_idx) => {
                let size_pages = frame.memory_size(engine)?;
                stack.push(Value::I32(size_pages as i32))?;
                Ok(ControlFlow::Continue)
            }
            Instruction::MemoryGrow(mem_idx) => {
                let delta = stack.pop_i32()? as u32;
                let prev_size = frame.memory_grow(delta, engine)?;
                stack.push(Value::I32(prev_size as i32))?;
                Ok(ControlFlow::Continue)
            }

            // Numeric Instructions
            Instruction::I32Const(val) => {
                numeric::i32_const(stack, frame, *val, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Const(val) => {
                numeric::i64_const(stack, frame, *val, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Const(val) => {
                numeric::f32_const(stack, frame, *val, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Const(val) => {
                numeric::f64_const(stack, frame, *val, engine)?;
                Ok(ControlFlow::Continue)
            }

            Instruction::I32Eqz => {
                numeric::i32_eqz(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Eq => {
                numeric::i32_eq(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Ne => {
                numeric::i32_ne(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32LtS => {
                numeric::i32_lt_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32LtU => {
                numeric::i32_lt_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32GtS => {
                numeric::i32_gt_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32GtU => {
                numeric::i32_gt_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32LeS => {
                numeric::i32_le_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32LeU => {
                numeric::i32_le_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32GeS => {
                comparison::i32_ge_s(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32GeU => {
                comparison::i32_ge_u(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }

            Instruction::I64Eqz => {
                comparison::i64_eqz(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Eq => {
                comparison::i64_eq(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Ne => {
                comparison::i64_ne(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64LtS => {
                comparison::i64_lt_s(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64LtU => {
                comparison::i64_lt_u(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64GtS => {
                comparison::i64_gt_s(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64GtU => {
                comparison::i64_gt_u(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64LeS => {
                comparison::i64_le_s(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64LeU => {
                comparison::i64_le_u(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64GeS => {
                comparison::i64_ge_s(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64GeU => {
                comparison::i64_ge_u(frame, stack, engine)?;
                Ok(ControlFlow::Continue)
            }

            Instruction::F32Eq => {
                numeric::f32_eq(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Ne => {
                numeric::f32_ne(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Lt => {
                numeric::f32_lt(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Gt => {
                numeric::f32_gt(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Le => {
                numeric::f32_le(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Ge => {
                numeric::f32_ge(stack, frame)?;
                Ok(ControlFlow::Continue)
            }

            Instruction::F64Eq => {
                numeric::f64_eq(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Ne => {
                numeric::f64_ne(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Lt => {
                numeric::f64_lt(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Gt => {
                numeric::f64_gt(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Le => {
                numeric::f64_le(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Ge => {
                numeric::f64_ge(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Clz => {
                numeric::i32_clz(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Ctz => {
                numeric::i32_ctz(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Popcnt => {
                numeric::i32_popcnt(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Add => {
                numeric::i32_add(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Sub => {
                numeric::i32_sub(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Mul => {
                numeric::i32_mul(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32DivS => {
                numeric::i32_div_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32DivU => {
                numeric::i32_div_u(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32RemS => {
                numeric::i32_rem_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32RemU => {
                numeric::i32_rem_u(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32And => {
                numeric::i32_and(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Or => {
                numeric::i32_or(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Xor => {
                numeric::i32_xor(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Shl => {
                numeric::i32_shl(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32ShrS => {
                numeric::i32_shr_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32ShrU => {
                numeric::i32_shr_u(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Rotl => {
                numeric::i32_rotl(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Rotr => {
                numeric::i32_rotr(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }

            Instruction::I64Clz => {
                numeric::i64_clz(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Ctz => {
                numeric::i64_ctz(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Popcnt => {
                numeric::i64_popcnt(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Add => {
                numeric::i64_add(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Sub => {
                numeric::i64_sub(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Mul => {
                numeric::i64_mul(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64DivS => {
                numeric::i64_div_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64DivU => {
                numeric::i64_div_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64RemS => {
                numeric::i64_rem_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64RemU => {
                numeric::i64_rem_u(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64And => {
                numeric::i64_and(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Or => {
                numeric::i64_or(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Xor => {
                numeric::i64_xor(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Shl => {
                numeric::i64_shl(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64ShrS => {
                numeric::i64_shr_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64ShrU => {
                numeric::i64_shr_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Rotl => {
                numeric::i64_rotl(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Rotr => {
                numeric::i64_rotr(stack, frame)?;
                Ok(ControlFlow::Continue)
            }

            Instruction::F32Abs => {
                numeric::f32_abs(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Neg => {
                numeric::f32_neg(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Ceil => {
                numeric::f32_ceil(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Floor => {
                numeric::f32_floor(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Trunc => {
                numeric::f32_trunc(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Nearest => {
                numeric::f32_nearest(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Sqrt => {
                numeric::f32_sqrt(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Add => {
                numeric::f32_add(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Sub => {
                numeric::f32_sub(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Mul => {
                numeric::f32_mul(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Div => {
                numeric::f32_div(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Min => {
                numeric::f32_min(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Max => {
                numeric::f32_max(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32Copysign => {
                numeric::f32_copysign(stack, frame)?;
                Ok(ControlFlow::Continue)
            }

            Instruction::F64Abs => {
                numeric::f64_abs(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Neg => {
                numeric::f64_neg(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Ceil => {
                numeric::f64_ceil(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Floor => {
                numeric::f64_floor(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Trunc => {
                numeric::f64_trunc(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Nearest => {
                numeric::f64_nearest(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Sqrt => {
                numeric::f64_sqrt(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Add => {
                numeric::f64_add(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Sub => {
                numeric::f64_sub(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Mul => {
                numeric::f64_mul(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Div => {
                numeric::f64_div(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Min => {
                numeric::f64_min(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Max => {
                numeric::f64_max(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64Copysign => {
                numeric::f64_copysign(stack, frame)?;
                Ok(ControlFlow::Continue)
            }

            Instruction::I32WrapI64 => {
                numeric::i32_wrap_i64(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32TruncF32S => {
                numeric::i32_trunc_f32_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32TruncF32U => {
                numeric::i32_trunc_f32_u(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32TruncF64S => {
                numeric::i32_trunc_f64_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32TruncF64U => {
                numeric::i32_trunc_f64_u(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64ExtendI32S => {
                numeric::i64_extend_i32_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64ExtendI32U => {
                numeric::i64_extend_i32_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64TruncF32S => {
                numeric::i64_trunc_f32_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64TruncF32U => {
                numeric::i64_trunc_f32_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64TruncF64S => {
                numeric::i64_trunc_f64_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64TruncF64U => {
                numeric::i64_trunc_f64_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }

            Instruction::F32ConvertI32S => {
                numeric::f32_convert_i32_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32ConvertI32U => {
                numeric::f32_convert_i32_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32ConvertI64S => {
                numeric::f32_convert_i64_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32ConvertI64U => {
                numeric::f32_convert_i64_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32DemoteF64 => {
                numeric::f32_demote_f64(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64ConvertI32S => {
                numeric::f64_convert_i32_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64ConvertI32U => {
                numeric::f64_convert_i32_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64ConvertI64S => {
                numeric::f64_convert_i64_s(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64ConvertI64U => {
                numeric::f64_convert_i64_u(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64PromoteF32 => {
                numeric::f64_promote_f32(stack, frame)?;
                Ok(ControlFlow::Continue)
            }

            Instruction::I32ReinterpretF32 => {
                numeric::i32_reinterpret_f32(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64ReinterpretF64 => {
                numeric::i64_reinterpret_f64(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F32ReinterpretI32 => {
                numeric::f32_reinterpret_i32(stack, frame)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::F64ReinterpretI64 => {
                numeric::f64_reinterpret_i64(stack, frame)?;
                Ok(ControlFlow::Continue)
            }

            Instruction::I32Extend8S => {
                numeric::i32_extend8_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32Extend16S => {
                numeric::i32_extend16_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Extend8S => {
                numeric::i64_extend8_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Extend16S => {
                numeric::i64_extend16_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64Extend32S => {
                numeric::i64_extend32_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }

            // TruncSat instructions
            Instruction::I32TruncSatF32S => {
                numeric::i32_trunc_sat_f32_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32TruncSatF32U => {
                numeric::i32_trunc_sat_f32_u(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32TruncSatF64S => {
                numeric::i32_trunc_sat_f64_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I32TruncSatF64U => {
                numeric::i32_trunc_sat_f64_u(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64TruncSatF32S => {
                numeric::i64_trunc_sat_f32_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64TruncSatF32U => {
                numeric::i64_trunc_sat_f32_u(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64TruncSatF64S => {
                numeric::i64_trunc_sat_f64_s(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::I64TruncSatF64U => {
                numeric::i64_trunc_sat_f64_u(stack, frame, engine)?;
                Ok(ControlFlow::Continue)
            }

            // SIMD instructions
            #[cfg(feature = "simd")]
            Instruction::V128Load { memarg } => {
                simd::V128Load::new(*memarg).execute(stack, frame, engine)
            }
            #[cfg(feature = "simd")]
            Instruction::V128Store(offset, align) => {
                simd::V128Store::new(*offset, *align).execute(stack, frame, engine)
            }
            #[cfg(feature = "simd")]
            Instruction::V128Const(bytes) => simd::V128Const(*bytes).execute(stack, frame, engine),
            #[cfg(feature = "simd")]
            Instruction::I8x16Swizzle => simd::I8x16Swizzle.execute(stack, frame, engine),

            // Reference Types Instructions
            Instruction::RefNull(ht) => refs::RefNull::new(*ht).execute(stack, frame, engine),
            Instruction::RefIsNull => refs::RefIsNull.execute(stack, frame, engine),
            Instruction::RefFunc(idx) => refs::RefFunc::new(*idx).execute(stack, frame, engine),

            // Table Instructions
            Instruction::TableGet(table_idx) => {
                let idx = stack.pop_i32()?;
                let value = frame.table_get(*table_idx, idx as u32, engine)?;
                stack.push(value)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::TableSet(table_idx) => {
                let value = stack.pop()?;
                let idx = stack.pop_i32()?;
                frame.table_set(*table_idx, idx as u32, value, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::TableSize(table_idx) => {
                let size = frame.table_size(*table_idx, engine)?;
                stack.push(Value::I32(size as i32))?;
                Ok(ControlFlow::Continue)
            }
            Instruction::TableGrow(table_idx) => {
                let value = stack.pop()?;
                let delta = stack.pop_i32()? as u32;
                let prev_size = frame.table_grow(*table_idx, delta, value, engine)?;
                stack.push(Value::I32(prev_size as i32))?;
                Ok(ControlFlow::Continue)
            }
            Instruction::TableInit(elem_idx, table_idx) => {
                let n = stack.pop_i32()?.try_into().map_err(|_| {
                    Error::new(kinds::InvalidTypeError(
                        "Invalid size for table.init".to_string(),
                    ))
                })?;
                let src = stack.pop_i32()?.try_into().map_err(|_| {
                    Error::new(kinds::InvalidTypeError(
                        "Invalid source offset for table.init".to_string(),
                    ))
                })?;
                let dst = stack.pop_i32()?.try_into().map_err(|_| {
                    Error::new(kinds::InvalidTypeError(
                        "Invalid destination offset for table.init".to_string(),
                    ))
                })?;
                frame.table_init(*table_idx, *elem_idx, dst, src, n, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::TableCopy(dst_table_idx, src_table_idx) => {
                let n = stack.pop_i32()?.try_into().map_err(|_| {
                    Error::new(kinds::InvalidTypeError(
                        "Invalid size for table.copy".to_string(),
                    ))
                })?;
                let src = stack.pop_i32()?.try_into().map_err(|_| {
                    Error::new(kinds::InvalidTypeError(
                        "Invalid source offset for table.copy".to_string(),
                    ))
                })?;
                let dst = stack.pop_i32()?.try_into().map_err(|_| {
                    Error::new(kinds::InvalidTypeError(
                        "Invalid destination offset for table.copy".to_string(),
                    ))
                })?;
                frame.table_copy(*dst_table_idx, *src_table_idx, dst, src, n, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::ElemDrop(elem_idx) => {
                frame.elem_drop(*elem_idx, engine)?;
                Ok(ControlFlow::Continue)
            }
            Instruction::TableFill(table_idx) => {
                let n = stack.pop_i32()?.try_into().map_err(|_| {
                    Error::new(kinds::InvalidTypeError(
                        "Invalid count for table.fill".to_string(),
                    ))
                })?;
                let val = stack.pop()?;
                let dst = stack.pop_i32()?.try_into().map_err(|_| {
                    Error::new(kinds::InvalidTypeError(
                        "Invalid offset for table.fill".to_string(),
                    ))
                })?;
                frame.table_fill(*table_idx, dst, val, n, engine)?;
                Ok(ControlFlow::Continue)
            }

            // Memory Bulk Instructions
            Instruction::MemoryInit(data_idx, mem_idx) => {
                MemoryInit::new(*data_idx, *mem_idx).execute(stack, frame, engine)
            }
            Instruction::DataDrop(idx) => DataDrop::new(*idx).execute(stack, frame, engine),
            Instruction::MemoryCopy(dst_mem_idx, src_mem_idx) => {
                let n = stack.pop_i32()? as usize;
                let src_addr = stack.pop_i32()? as usize;
                let dst_addr = stack.pop_i32()? as usize;

                // Get memory references using frame, converting index to usize
                let dst_mem = frame.get_memory_mut((*dst_mem_idx).try_into().unwrap(), engine)?;
                let src_mem = if *src_mem_idx == *dst_mem_idx {
                    dst_mem.clone() // Clone the Arc if it's the same memory
                } else {
                    frame.get_memory((*src_mem_idx).try_into().unwrap(), engine)?
                    // Get a separate Arc otherwise
                };

                // Use the suggested copy_within_or_between method
                dst_mem.copy_within_or_between(src_mem, src_addr, dst_addr, n)?;

                Ok(ControlFlow::Continue)
            }
            Instruction::MemoryFill(mem_idx) => {
                let n = stack.pop_i32()? as usize;
                let val = stack.pop_i32()? as u8;
                let dst = stack.pop_i32()? as usize;
                // Use frame to get memory and fill, converting index to usize
                let memory = frame.get_memory_mut((*mem_idx).try_into().unwrap(), engine)?;
                memory.fill(dst, val, n)?;
                Ok(ControlFlow::Continue)
            }

            // Catch-all for unimplemented instructions
            _ => Err(Error::new(kinds::NotImplementedError(format!(
                "Instruction not implemented: {:?}",
                self
            )))),
        }
    }

    fn execute_with_frame_idx(
        &self,
        stack: &mut dyn StackBehavior,
        frame_idx: usize,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow, crate::error::Error> {
        // This implementation is no longer needed as stackless.rs directly uses execute
        // with a cloned frame, avoiding the borrow checker issues
        Err(crate::error::Error::new(
            crate::error::kinds::ExecutionError(
                "execute_with_frame_idx is deprecated, use execute directly".to_string(),
            ),
        ))
    }
}
