//! WebAssembly control flow instructions
//!
//! This module contains implementations for all WebAssembly control flow instructions,
//! including blocks, branches, calls, and returns.
//!
//! This module integrates with the pure implementations in `wrt-instructions/control_ops.rs`,
//! providing the runtime-specific context needed for execution.

use crate::{
    behavior::{
        self, ControlFlow, ControlFlowBehavior, FrameBehavior, InstructionExecutor, Label,
        StackBehavior,
    },
    error::{kinds, Error, Result},
    module::{self, Data, Element, ExportKind},
    stackless_frame::StacklessFrame,
    types::{BlockType, FuncType, ValueType},
    values::{Value, Value::*},
    StacklessEngine,
};
use log::trace;
use std::borrow::Borrow;
use std::sync::Arc;

// Import the pure implementations from wrt-instructions
use wrt_instructions::control_ops::{
    Block as ControlBlock, BranchTarget, ControlBlockType, ControlContext, ControlOp,
};
use wrt_instructions::instruction_traits::PureInstruction;

#[cfg(feature = "std")]
use std::vec;

#[cfg(not(feature = "std"))]
use alloc::vec;

/// Label type for control flow
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelType {
    /// Block instruction label
    Block,
    /// Loop instruction label
    Loop,
    /// If instruction label
    If,
}

/// Runtime adapter that bridges the pure control operations with the stackless engine
struct RuntimeControlContext<'a> {
    stack: &'a mut dyn StackBehavior,
    frame: &'a mut StacklessFrame,
    engine: &'a mut StacklessEngine,
}

impl<'a> ControlContext for RuntimeControlContext<'a> {
    fn push_control_value(&mut self, value: Value) -> wrt_instructions::Result<()> {
        self.stack
            .push(value)
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn pop_control_value(&mut self) -> wrt_instructions::Result<Value> {
        self.stack
            .pop()
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn get_block_depth(&self) -> usize {
        self.frame.get_label_count()
    }

    fn enter_block(&mut self, block_type: ControlBlock) -> wrt_instructions::Result<()> {
        let stack_len = self.stack.len();

        match block_type {
            ControlBlock::Block(block_type) => {
                let wrt_block_type = convert_block_type(block_type);
                self.frame
                    .enter_block(wrt_block_type, stack_len)
                    .map_err(|e| wrt_instructions::Error::new(e.kind()))
            }
            ControlBlock::Loop(block_type) => {
                let wrt_block_type = convert_block_type(block_type);
                self.frame
                    .enter_loop(wrt_block_type, stack_len)
                    .map_err(|e| wrt_instructions::Error::new(e.kind()))
            }
            ControlBlock::If(block_type) => {
                let wrt_block_type = convert_block_type(block_type);
                // Default to condition true since we've already evaluated it in the pure implementation
                self.frame
                    .enter_if(wrt_block_type, stack_len, true)
                    .map_err(|e| wrt_instructions::Error::new(e.kind()))
            }
            ControlBlock::Try(_) => {
                // Not supported yet
                Err(wrt_instructions::Error::new(
                    wrt_error::kinds::ExecutionError("Try blocks not supported yet".to_string()),
                ))
            }
        }
    }

    fn exit_block(&mut self) -> wrt_instructions::Result<ControlBlock> {
        self.frame
            .exit_block()
            .map(|_| ControlBlock::Block(ControlBlockType::ValueType(None))) // Just a placeholder
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn branch(&mut self, target: BranchTarget) -> wrt_instructions::Result<()> {
        // Use the internal br implementation
        br(target.label_idx, self.engine)
            .map(|_| ())
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn return_function(&mut self) -> wrt_instructions::Result<()> {
        // Use the internal return_call implementation
        return_call(self.stack, self.engine)
            .map(|_| ())
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn call_function(&mut self, func_idx: u32) -> wrt_instructions::Result<()> {
        // Use the internal call_internal implementation
        call_internal(func_idx, self.engine)
            .map(|_| ())
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn call_indirect(&mut self, table_idx: u32, type_idx: u32) -> wrt_instructions::Result<()> {
        // Use the internal call_indirect implementation
        call_indirect(type_idx, table_idx, self.engine)
            .map(|_| ())
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn trap(&mut self, message: &str) -> wrt_instructions::Result<()> {
        Err(wrt_instructions::Error::new(wrt_error::kinds::Trap(
            message.to_string(),
        )))
    }
}

/// Convert from wrt-instructions BlockType to wrt BlockType
fn convert_block_type(block_type: ControlBlockType) -> BlockType {
    match block_type {
        ControlBlockType::ValueType(Some(vt)) => BlockType::Type(vt),
        ControlBlockType::ValueType(None) => BlockType::Empty,
        ControlBlockType::FuncType(ft) => BlockType::FuncType(ft),
    }
}

/// Create and push a new label onto the label stack
///
/// This is used internally by block, loop, and if instructions.
pub fn push_label<S: StackBehavior>(
    pc: usize,
    stack: &mut S,
    _label_type: LabelType,
    block_type: BlockType,
    function_types: Option<&[FuncType]>,
) -> Result<()> {
    // First, resolve the block type to get the expected result count
    let arity = match block_type {
        BlockType::Empty => 0,
        BlockType::Type(_vt) | BlockType::Value(_vt) => 1,
        BlockType::FuncType(_) => {
            // For function types, need to look up the exact result count
            // This would typically require access to the module's types
            if let Some(_types) = function_types {
                // TODO: Implement type lookup logic
                // For now, assuming zero results if we can't resolve
                0
            } else {
                0 // Default to 0 if we can't resolve
            }
        }
        BlockType::TypeIndex(_) => 0, // Default to 0 for type indices
    };

    let label = Label {
        arity,
        pc,
        continuation: pc,
        stack_depth: stack.len(), // Current stack depth
        is_loop: matches!(_label_type, LabelType::Loop), // Add is_loop field
        is_if: matches!(_label_type, LabelType::If), // Add is_if field
    };

    // Push the new label using the StackBehavior trait's method
    <S as StackBehavior>::push_label(stack, label);

    Ok(())
}

pub fn block<S: StackBehavior>(stack: &mut S) -> Result<(), Error> {
    // Push a new label with arity 0
    let label = Label {
        arity: 0,
        pc: 0,           // Placeholder
        continuation: 0, // Placeholder
        stack_depth: 0,  // Will be filled in by the implementation
        is_loop: false,
        is_if: false,
    };
    <S as StackBehavior>::push_label(stack, label);
    Ok(())
}

#[derive(Debug)]
pub struct Block {
    pub block_type: BlockType,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self { block_type }
    }
}

impl InstructionExecutor for Block {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in Block".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Convert to wrt-instructions block type
        let control_block_type = match &self.block_type {
            BlockType::Empty => ControlBlockType::ValueType(None),
            BlockType::Type(vt) => ControlBlockType::ValueType(Some(*vt)),
            BlockType::Value(vt) => ControlBlockType::ValueType(Some(*vt)),
            BlockType::FuncType(ft) => ControlBlockType::FuncType(ft.clone()),
            BlockType::TypeIndex(_) => ControlBlockType::ValueType(None), // Default to empty for type indices
        };

        // Use the pure implementation from wrt-instructions
        ControlOp::Block(control_block_type)
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        Ok(ControlFlow::Continue)
    }
}

#[derive(Debug)]
pub struct Loop {
    pub block_type: BlockType,
}

impl Loop {
    pub fn new(block_type: BlockType) -> Self {
        Self { block_type }
    }
}

impl InstructionExecutor for Loop {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in Loop".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Convert to wrt-instructions block type
        let control_block_type = match &self.block_type {
            BlockType::Empty => ControlBlockType::ValueType(None),
            BlockType::Type(vt) => ControlBlockType::ValueType(Some(*vt)),
            BlockType::Value(vt) => ControlBlockType::ValueType(Some(*vt)),
            BlockType::FuncType(ft) => ControlBlockType::FuncType(ft.clone()),
            BlockType::TypeIndex(_) => ControlBlockType::ValueType(None), // Default to empty for type indices
        };

        // Use the pure implementation from wrt-instructions
        ControlOp::Loop(control_block_type)
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        Ok(ControlFlow::Continue)
    }
}

#[derive(Debug)]
pub struct If {
    pub block_type: BlockType,
}

impl If {
    pub fn new(block_type: BlockType) -> Self {
        Self { block_type }
    }
}

impl InstructionExecutor for If {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in If".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Convert to wrt-instructions control block type using the consolidated BlockType
        let control_block_type = match &self.block_type {
            BlockType::Empty => wrt_instructions::control_ops::ControlBlockType::ValueType(None),
            BlockType::Type(vt) => {
                wrt_instructions::control_ops::ControlBlockType::ValueType(Some(*vt))
            }
            BlockType::Value(vt) => {
                wrt_instructions::control_ops::ControlBlockType::ValueType(Some(*vt))
            }
            BlockType::FuncType(ft) => {
                wrt_instructions::control_ops::ControlBlockType::FuncType(ft.clone())
            }
            BlockType::TypeIndex(_) => {
                wrt_instructions::control_ops::ControlBlockType::ValueType(None)
            } // Default to empty for type indices
        };

        // Use the pure implementation from wrt-instructions
        wrt_instructions::control_ops::ControlOp::If(control_block_type)
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        Ok(ControlFlow::Continue)
    }
}

#[derive(Debug)]
pub struct Br {
    pub label_idx: u32,
}

impl Br {
    pub fn new(label_idx: u32) -> Self {
        Self { label_idx }
    }
}

impl InstructionExecutor for Br {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in Br".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        ControlOp::Br(self.label_idx)
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        // Branch always causes control flow change, but this will be handled by the engine
        Ok(ControlFlow::Branch(self.label_idx))
    }
}

#[derive(Debug)]
pub struct BrIf {
    pub label_idx: u32,
}

impl BrIf {
    pub fn new(label_idx: u32) -> Self {
        Self { label_idx }
    }
}

impl InstructionExecutor for BrIf {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in BrIf".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Get the condition before executing
        let condition = stack.pop_i32()?;

        // Use the pure implementation from wrt-instructions
        ControlOp::BrIf(self.label_idx)
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        // If condition is true, branch; otherwise continue
        if condition != 0 {
            Ok(ControlFlow::Branch(self.label_idx))
        } else {
            Ok(ControlFlow::Continue)
        }
    }
}

#[derive(Debug)]
pub struct BrTable {
    pub labels: Vec<u32>,
    pub default_label: u32,
}

impl BrTable {
    pub fn new(labels: Vec<u32>, default_label: u32) -> Self {
        Self {
            labels,
            default_label,
        }
    }
}

impl InstructionExecutor for BrTable {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in BrTable".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Get the index before executing
        let index = stack.pop_i32()?;

        // Use the pure implementation from wrt-instructions
        ControlOp::BrTable {
            table: self.labels.clone(),
            default: self.default_label,
        }
        .execute(&mut context)
        .map_err(|e| Error::new(e.kind()))?;

        // Determine which label to branch to
        let label_idx = if index >= 0 && (index as usize) < self.labels.len() {
            self.labels[index as usize]
        } else {
            self.default_label
        };

        Ok(ControlFlow::Branch(label_idx))
    }
}

#[derive(Debug)]
pub struct Return;

impl InstructionExecutor for Return {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in Return".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        ControlOp::Return
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        Ok(ControlFlow::Return)
    }
}

#[derive(Debug)]
pub struct Call {
    pub func_idx: u32,
}

impl Call {
    pub fn new(func_idx: u32) -> Self {
        Self { func_idx }
    }
}

impl InstructionExecutor for Call {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in Call".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        ControlOp::Call(self.func_idx)
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        Ok(ControlFlow::Call(self.func_idx))
    }
}

#[derive(Debug)]
pub struct CallIndirect {
    pub type_idx: u32,
    pub table_idx: u32,
}

impl CallIndirect {
    pub fn new(type_idx: u32, table_idx: u32) -> Self {
        Self {
            type_idx,
            table_idx,
        }
    }
}

impl InstructionExecutor for CallIndirect {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in CallIndirect".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        ControlOp::CallIndirect {
            table_idx: self.table_idx,
            type_idx: self.type_idx,
        }
        .execute(&mut context)
        .map_err(|e| Error::new(e.kind()))?;

        // CallIndirect is handled internally in the engine
        // Just return Continue here as the actual call flow will be determined by the engine
        Ok(ControlFlow::Continue)
    }
}

#[derive(Debug)]
pub struct Nop;

impl InstructionExecutor for Nop {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in Nop".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        ControlOp::Nop
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        Ok(ControlFlow::Continue)
    }
}

#[derive(Debug)]
pub struct Unreachable;

impl InstructionExecutor for Unreachable {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in Unreachable".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        ControlOp::Unreachable
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))
    }
}

#[derive(Debug)]
pub struct Else;

impl InstructionExecutor for Else {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in Else".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        ControlOp::Else
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        // Else instruction is handled in the frame
        // depending on the current state of if execution
        Ok(frame.else_block()?)
    }
}

#[derive(Debug)]
pub struct End;

impl InstructionExecutor for End {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in End".to_string(),
                ))
            })?;

        let mut context = RuntimeControlContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        ControlOp::End
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        // End instruction is handled in the frame
        Ok(frame.end_block()?)
    }
}

pub(crate) fn br(label_idx: u32, engine: &mut StacklessEngine) -> Result<ControlFlow> {
    // Get the stack reference first
    let stack = &mut engine.exec_stack;

    // Get the label from stack before getting the frame
    let label = stack
        .get_label(label_idx as usize)
        .ok_or_else(|| Error::new(kinds::Trap(format!("Invalid label index: {}", label_idx))))?;

    let (target_pc, arity, values_to_pop) = (label.pc, label.arity, 0);

    let branch_args = if arity > 0 {
        stack.pop_n(arity)
    } else {
        vec![]
    };

    let _ = stack.pop_n(values_to_pop);

    stack.push_n(&branch_args);

    Ok(ControlFlow::Branch {
        target_pc,
        values_to_keep: arity,
    })
}

pub(crate) fn br_if(label_idx: u32, engine: &mut StacklessEngine) -> Result<ControlFlow> {
    let condition = engine.exec_stack.pop_i32()? != 0;
    if condition {
        br(label_idx, engine)
    } else {
        Ok(ControlFlow::Continue)
    }
}

pub(crate) fn execute_br_table(
    indices: &Vec<u32>,
    default_idx: u32,
    engine: &mut StacklessEngine,
) -> Result<ControlFlow> {
    let index = engine.exec_stack.pop_i32()? as usize;
    let target_label_idx = indices.get(index).copied().unwrap_or(default_idx);
    br(target_label_idx, engine)
}

pub(crate) fn call_internal(func_idx: u32, engine: &mut StacklessEngine) -> Result<ControlFlow> {
    let current_instance_idx = engine.current_frame()?.instance_idx();
    let (_func_addr, func_type) =
        engine.with_instance(current_instance_idx as usize, |instance| {
            let addr = instance.get_func_addr(func_idx);
            let ty = instance.get_function_type(func_idx)?;
            Ok((addr, ty.clone()))
        })?;

    let return_pc = engine.current_frame()?.pc();

    let num_params = func_type.params.len();
    let args = engine.exec_stack.pop_n(num_params);

    let module_arc = engine.with_instance(current_instance_idx as usize, |instance| {
        Ok(instance.module.clone())
    })?; // Remove second `?`

    let new_frame = StacklessFrame::new(module_arc, func_idx, &args, current_instance_idx)?;

    engine.exec_stack.frames.push(new_frame);

    Ok(ControlFlow::Call {
        func_idx,
        args,
        return_pc,
    })
}

pub fn call_indirect(
    type_idx: u32,
    table_idx: u32,
    engine: &mut StacklessEngine,
) -> Result<ControlFlow> {
    let current_instance_idx = engine.current_frame()?.instance_idx();
    let table_entry_idx = engine.exec_stack.pop_i32()?;

    let func_ref_result = engine.with_instance(current_instance_idx as usize, |instance| {
        let table = instance.get_table(table_idx as usize)?;
        let table_ref = &*table; // Dereference Arc<Table>

        if table_entry_idx < 0 || table_entry_idx as usize >= table_ref.size() as usize {
            return Err(Error::new(kinds::Trap(
                "indirect call table index out of bounds".to_string(),
            )));
        }

        let func_ref = table_ref.get(table_entry_idx as u32)?;
        match func_ref {
            // Match against Option<Value>
            Some(Value::FuncRef(Some(idx))) => {
                // Wrap pattern in Some()
                let actual_type = instance.get_function_type(idx)?;
                Ok((idx, actual_type.clone()))
            }
            Some(Value::FuncRef(None)) => Err(Error::new(kinds::InvalidFunctionType(format!(
                "Invalid function reference in call_indirect: null reference"
            )))),
            None => Err(Error::new(kinds::Trap(
                "uninitialized element in table".to_string(),
            ))), // Handle None case explicitly
            _ => Err(Error::new(kinds::Trap(
                "indirect call type mismatch - expected funcref".to_string(),
            ))), // Handle other Some(Value) variants
        }
    })?;

    let (func_idx, actual_type) = func_ref_result;

    let expected_type = engine.with_instance(current_instance_idx as usize, |instance| {
        instance.get_function_type(type_idx).cloned()
    })?;

    if expected_type != actual_type {
        return Err(Error::new(kinds::Trap(
            "indirect call type mismatch".to_string(),
        )));
    }

    call_internal(func_idx, engine)
}

pub(crate) fn return_call(
    stack: &mut dyn StackBehavior,
    _engine: &mut StacklessEngine,
) -> Result<ControlFlow> {
    let _index = stack.pop_i32()?;
    // TODO: Determine actual return values based on current function signature
    // For now, assume no return values or handle in engine loop
    Ok(ControlFlow::Return { values: vec![] }) // Use struct variant
}

fn branch(
    label_idx: u32,
    frame: &mut dyn FrameBehavior,
    stack: &mut dyn StackBehavior,
    values_to_pop: usize,
) -> Result<ControlFlow> {
    // Get the label from stack
    let label = stack
        .get_label(label_idx as usize)
        .ok_or_else(|| Error::new(kinds::Trap(format!("Invalid label index: {}", label_idx))))?;

    let (target_pc, arity, values_to_pop) = (label.pc, label.arity, values_to_pop);

    let branch_args = if arity > 0 {
        stack.pop_n(arity)
    } else {
        vec![]
    };

    let _ = stack.pop_n(values_to_pop);

    stack.push_n(&branch_args);

    Ok(ControlFlow::Branch {
        target_pc,
        values_to_keep: arity,
    })
}
