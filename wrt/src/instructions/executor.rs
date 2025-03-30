//! Instruction executor implementation.

use crate::{
    behavior::{FrameBehavior, InstructionExecutor},
    error::{Error, Result},
    instructions::{
        arithmetic, comparison, control, memory, numeric, parametric, simd, table, variable,
        Instruction,
    },
    stack::Stack,
};

// Implement the InstructionExecutor trait for Instruction
impl InstructionExecutor for Instruction {
    fn execute(&self, stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
        match self {
            // Control flow instructions
            Self::Block(block_type) => control::block_dyn(stack, frame, block_type.clone()),
            Self::Loop(block_type) => control::loop_dyn(stack, frame, block_type.clone()),
            Self::If(block_type) => control::if_dyn(stack, frame, block_type.clone()),
            Self::Else => control::else_dyn(stack, frame),
            Self::End => control::end_dyn(stack, frame),
            Self::Br(label_idx) => control::br_dyn(stack, frame, *label_idx),
            Self::BrIf(label_idx) => control::br_if_dyn(stack, frame, *label_idx),
            Self::BrTable(label_indices, default_label) => {
                control::br_table_dyn(stack, frame, label_indices.clone(), *default_label)
            }
            Self::Return => control::return_dyn(stack, frame),
            Self::Unreachable => control::unreachable_dyn(stack, frame),
            Self::Nop => control::nop_dyn(stack, frame),

            // Call instructions
            Self::Call(func_idx) => control::call_dyn(stack, frame, *func_idx),
            Self::CallIndirect(type_idx, table_idx) => {
                control::call_indirect_dyn(stack, frame, *type_idx, *table_idx)
            }
            Self::ReturnCall(func_idx) => control::return_call_dyn(stack, frame, *func_idx),
            Self::ReturnCallIndirect(type_idx, table_idx) => {
                control::return_call_indirect_dyn(stack, frame, *type_idx, *table_idx)
            }

            // Parametric instructions
            Self::Drop => parametric::drop(stack, frame),
            Self::Select => parametric::select(stack, frame),
            Self::SelectTyped(value_type) => parametric::select_typed(stack, frame, *value_type),

            // Variable instructions
            Self::LocalGet(idx) => variable::local_get(stack, frame, *idx),
            Self::LocalSet(idx) => variable::local_set(stack, frame, *idx),
            Self::LocalTee(idx) => variable::local_tee(stack, frame, *idx),
            Self::GlobalGet(idx) => variable::global_get(stack, frame, *idx),
            Self::GlobalSet(idx) => variable::global_set(stack, frame, *idx),

            // Table instructions
            Self::TableGet(idx) => table::table_get(stack, frame, *idx),
            Self::TableSet(idx) => table::table_set(stack, frame, *idx),
            Self::TableSize(idx) => table::table_size(stack, frame, *idx),
            Self::TableGrow(idx) => table::table_grow(stack, frame, *idx),
            Self::TableInit(table_idx, elem_idx) => {
                table::table_init(stack, frame, *table_idx, *elem_idx)
            }
            Self::TableCopy(dst_idx, src_idx) => {
                table::table_copy(stack, frame, *dst_idx, *src_idx)
            }
            Self::TableFill(idx) => table::table_fill(stack, frame, *idx),
            Self::ElemDrop(idx) => table::elem_drop(stack, frame, *idx),

            // Memory instructions
            Self::I32Load(offset, align) => memory::i32_load(stack, frame, *offset, *align),
            Self::I64Load(offset, align) => memory::i64_load(stack, frame, *offset, *align),
            Self::F32Load(offset, align) => memory::f32_load(stack, frame, *offset, *align),
            Self::F64Load(offset, align) => memory::f64_load(stack, frame, *offset, *align),
            Self::I32Load8S(offset, align) => memory::i32_load8_s(stack, frame, *offset, *align),
            Self::I32Load8U(offset, align) => memory::i32_load8_u(stack, frame, *offset, *align),
            Self::I32Load16S(offset, align) => memory::i32_load16_s(stack, frame, *offset, *align),
            Self::I32Load16U(offset, align) => memory::i32_load16_u(stack, frame, *offset, *align),
            Self::I64Load8S(offset, align) => memory::i64_load8_s(stack, frame, *offset, *align),
            Self::I64Load8U(offset, align) => memory::i64_load8_u(stack, frame, *offset, *align),
            Self::I64Load16S(offset, align) => memory::i64_load16_s(stack, frame, *offset, *align),
            Self::I64Load16U(offset, align) => memory::i64_load16_u(stack, frame, *offset, *align),
            Self::I64Load32S(offset, align) => memory::i64_load32_s(stack, frame, *offset, *align),
            Self::I64Load32U(offset, align) => memory::i64_load32_u(stack, frame, *offset, *align),
            Self::I32Store(offset, align) => memory::i32_store(stack, frame, *offset, *align),
            Self::I64Store(offset, align) => memory::i64_store(stack, frame, *offset, *align),
            Self::F32Store(offset, align) => memory::f32_store(stack, frame, *offset, *align),
            Self::F64Store(offset, align) => memory::f64_store(stack, frame, *offset, *align),
            Self::I32Store8(offset, align) => memory::i32_store8(stack, frame, *offset, *align),
            Self::I32Store16(offset, align) => memory::i32_store16(stack, frame, *offset, *align),
            Self::I64Store8(offset, align) => memory::i64_store8(stack, frame, *offset, *align),
            Self::I64Store16(offset, align) => memory::i64_store16(stack, frame, *offset, *align),
            Self::I64Store32(offset, align) => memory::i64_store32(stack, frame, *offset, *align),
            Self::MemorySize => memory::memory_size(stack, frame),
            Self::MemoryGrow => memory::memory_grow(stack, frame),
            Self::MemoryInit(idx) => memory::memory_init(stack, frame, *idx),
            Self::DataDrop(idx) => memory::data_drop(stack, frame, *idx),
            Self::MemoryCopy => memory::memory_copy(stack, frame),
            Self::MemoryFill => memory::memory_fill(stack, frame),

            // Numeric constant instructions
            Self::I32Const(value) => numeric::i32_const(stack, frame, *value),
            Self::I64Const(value) => numeric::i64_const(stack, frame, *value),
            Self::F32Const(value) => numeric::f32_const(stack, frame, *value),
            Self::F64Const(value) => numeric::f64_const(stack, frame, *value),

            // Comparison instructions
            Self::I32Eqz => comparison::i32_eqz(stack, frame),
            Self::I32Eq => comparison::i32_eq(stack, frame),
            Self::I32Ne => comparison::i32_ne(stack, frame),
            Self::I32LtS => comparison::i32_lt_s(stack, frame),
            Self::I32LtU => comparison::i32_lt_u(stack, frame),
            Self::I32GtS => comparison::i32_gt_s(stack, frame),
            Self::I32GtU => comparison::i32_gt_u(stack, frame),
            Self::I32LeS => comparison::i32_le_s(stack, frame),
            Self::I32LeU => comparison::i32_le_u(stack, frame),
            Self::I32GeS => comparison::i32_ge_s(stack, frame),
            Self::I32GeU => comparison::i32_ge_u(stack, frame),
            Self::I64Eqz => comparison::i64_eqz(stack, frame),
            Self::I64Eq => comparison::i64_eq(stack, frame),
            Self::I64Ne => comparison::i64_ne(stack, frame),
            Self::I64LtS => comparison::i64_lt_s(stack, frame),
            Self::I64LtU => comparison::i64_lt_u(stack, frame),
            Self::I64GtS => comparison::i64_gt_s(stack, frame),
            Self::I64GtU => comparison::i64_gt_u(stack, frame),
            Self::I64LeS => comparison::i64_le_s(stack, frame),
            Self::I64LeU => comparison::i64_le_u(stack, frame),
            Self::I64GeS => comparison::i64_ge_s(stack, frame),
            Self::I64GeU => comparison::i64_ge_u(stack, frame),
            Self::F32Eq => comparison::f32_eq(stack, frame),
            Self::F32Ne => comparison::f32_ne(stack, frame),
            Self::F32Lt => comparison::f32_lt(stack, frame),
            Self::F32Gt => comparison::f32_gt(stack, frame),
            Self::F32Le => comparison::f32_le(stack, frame),
            Self::F32Ge => comparison::f32_ge(stack, frame),
            Self::F64Eq => comparison::f64_eq(stack, frame),
            Self::F64Ne => comparison::f64_ne(stack, frame),
            Self::F64Lt => comparison::f64_lt(stack, frame),
            Self::F64Gt => comparison::f64_gt(stack, frame),
            Self::F64Le => comparison::f64_le(stack, frame),
            Self::F64Ge => comparison::f64_ge(stack, frame),

            // Arithmetic instructions
            Self::I32Clz => numeric::i32_clz(stack, frame),
            Self::I32Ctz => numeric::i32_ctz(stack, frame),
            Self::I32Popcnt => numeric::i32_popcnt(stack, frame),
            Self::I32Add => arithmetic::i32_add(stack, frame),
            Self::I32Sub => arithmetic::i32_sub(stack, frame),
            Self::I32Mul => arithmetic::i32_mul(stack, frame),
            Self::I32DivS => arithmetic::i32_div_s(stack, frame),
            Self::I32DivU => arithmetic::i32_div_u(stack, frame),
            Self::I32RemS => arithmetic::i32_rem_s(stack, frame),
            Self::I32RemU => arithmetic::i32_rem_u(stack, frame),
            Self::I32And => arithmetic::i32_and(stack, frame),
            Self::I32Or => arithmetic::i32_or(stack, frame),
            Self::I32Xor => arithmetic::i32_xor(stack, frame),
            Self::I32Shl => arithmetic::i32_shl(stack, frame),
            Self::I32ShrS => arithmetic::i32_shr_s(stack, frame),
            Self::I32ShrU => arithmetic::i32_shr_u(stack, frame),
            Self::I32Rotl => arithmetic::i32_rotl(stack, frame),
            Self::I32Rotr => arithmetic::i32_rotr(stack, frame),
            Self::I64Clz => numeric::i64_clz(stack, frame),
            Self::I64Ctz => numeric::i64_ctz(stack, frame),
            Self::I64Popcnt => numeric::i64_popcnt(stack, frame),
            Self::I64Add => arithmetic::i64_add(stack, frame),
            Self::I64Sub => arithmetic::i64_sub(stack, frame),
            Self::I64Mul => arithmetic::i64_mul(stack, frame),
            Self::I64DivS => arithmetic::i64_div_s(stack, frame),
            Self::I64DivU => arithmetic::i64_div_u(stack, frame),
            Self::I64RemS => arithmetic::i64_rem_s(stack, frame),
            Self::I64RemU => arithmetic::i64_rem_u(stack, frame),
            Self::I64And => arithmetic::i64_and(stack, frame),
            Self::I64Or => arithmetic::i64_or(stack, frame),
            Self::I64Xor => arithmetic::i64_xor(stack, frame),
            Self::I64Shl => arithmetic::i64_shl(stack, frame),
            Self::I64ShrS => arithmetic::i64_shr_s(stack, frame),
            Self::I64ShrU => arithmetic::i64_shr_u(stack, frame),
            Self::I64Rotl => arithmetic::i64_rotl(stack, frame),
            Self::I64Rotr => arithmetic::i64_rotr(stack, frame),
            Self::F32Abs => numeric::f32_abs(stack, frame),
            Self::F32Neg => numeric::f32_neg(stack, frame),
            Self::F32Ceil => numeric::f32_ceil(stack, frame),
            Self::F32Floor => numeric::f32_floor(stack, frame),
            Self::F32Trunc => numeric::f32_trunc(stack, frame),
            Self::F32Nearest => numeric::f32_nearest(stack, frame),
            Self::F32Sqrt => numeric::f32_sqrt(stack, frame),
            Self::F32Add => numeric::f32_add(stack, frame),
            Self::F32Sub => numeric::f32_sub(stack, frame),
            Self::F32Mul => numeric::f32_mul(stack, frame),
            Self::F32Div => numeric::f32_div(stack, frame),
            Self::F32Min => numeric::f32_min(stack, frame),
            Self::F32Max => numeric::f32_max(stack, frame),
            Self::F32Copysign => numeric::f32_copysign(stack, frame),
            Self::F64Abs => arithmetic::f64_abs(stack, frame),
            Self::F64Neg => arithmetic::f64_neg(stack, frame),
            Self::F64Ceil => arithmetic::f64_ceil(stack, frame),
            Self::F64Floor => arithmetic::f64_floor(stack, frame),
            Self::F64Trunc => arithmetic::f64_trunc(stack, frame),
            Self::F64Nearest => arithmetic::f64_nearest(stack, frame),
            Self::F64Sqrt => arithmetic::f64_sqrt(stack, frame),
            Self::F64Add => arithmetic::f64_add(stack, frame),
            Self::F64Sub => arithmetic::f64_sub(stack, frame),
            Self::F64Mul => arithmetic::f64_mul(stack, frame),
            Self::F64Div => arithmetic::f64_div(stack, frame),
            Self::F64Min => arithmetic::f64_min(stack, frame),
            Self::F64Max => arithmetic::f64_max(stack, frame),
            Self::F64Copysign => arithmetic::f64_copysign(stack, frame),

            // SIMD instructions
            Self::F32x4Splat => simd::f32x4_splat(stack, frame),
            Self::F64x2Splat => simd::f64x2_splat(stack, frame),
            Self::V128Load(_offset, _align) => {
                // We need to implement the v128_load here directly since it needs to access memory
                // The actual implementation would be something like:
                // 1. Pop the address from the stack
                // 2. Load 16 bytes from memory at the address + offset
                // 3. Push the result v128 onto the stack
                Err(Error::Unimplemented(
                    "V128Load not fully implemented".to_string(),
                ))
            }
            Self::V128Store(_offset, _align) => {
                // We need to implement the v128_store here directly since it needs to access memory
                // The actual implementation would be something like:
                // 1. Pop the v128 value from the stack
                // 2. Pop the address from the stack
                // 3. Store the 16 bytes to memory at the address + offset
                Err(Error::Unimplemented(
                    "V128Store not fully implemented".to_string(),
                ))
            }

            _ => Err(Error::Unimplemented(format!(
                "Instruction not implemented: {self:?}"
            ))),
        }
    }
}
