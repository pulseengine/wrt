//! WebAssembly control flow instructions
//!
//! This module contains implementations for all WebAssembly control flow instructions,
//! including blocks, branches, calls, and returns.

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
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let stack_len = stack.len();
        // Cast directly to StacklessFrame rather than trying to downcast to the trait
        if let Some(frame_cast) = frame
            .as_any()
            .downcast_mut::<crate::stackless_frame::StacklessFrame>()
        {
            frame_cast.enter_block(self.block_type.clone(), stack_len)?;
            Ok(ControlFlow::Continue)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Frame doesn't implement ControlFlowBehavior".to_string(),
            )))
        }
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
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let stack_len = stack.len();
        // Cast directly to StacklessFrame rather than trying to downcast to the trait
        if let Some(frame_cast) = frame
            .as_any()
            .downcast_mut::<crate::stackless_frame::StacklessFrame>()
        {
            frame_cast.enter_loop(self.block_type.clone(), stack_len)?;
            Ok(ControlFlow::Continue)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Frame doesn't implement ControlFlowBehavior".to_string(),
            )))
        }
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
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let condition_val = stack.pop_i32()?;
        let stack_len = stack.len();

        // Cast directly to StacklessFrame rather than trying to downcast to the trait
        if let Some(frame_cast) = frame
            .as_any()
            .downcast_mut::<crate::stackless_frame::StacklessFrame>()
        {
            frame_cast.enter_if(self.block_type.clone(), stack_len, condition_val != 0)?;
            Ok(ControlFlow::Continue)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Frame doesn't implement ControlFlowBehavior".to_string(),
            )))
        }
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
        _stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        br(self.label_idx, engine)
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
        _stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        br_if(self.label_idx, engine)
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
        _stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        execute_br_table(&self.labels, self.default_label, engine)
    }
}

#[derive(Debug, Default)]
pub struct Return;

impl InstructionExecutor for Return {
    fn execute(
        &self,
        _stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        // Pop frame from the engine's stack
        let frame = engine
            .exec_stack
            .frames
            .pop()
            .ok_or_else(|| Error::trap("Frame stack underflow during return".to_string()))?;
        let mut results = Vec::with_capacity(frame.arity());
        for _ in 0..frame.arity() {
            results.push(engine.exec_stack.pop()?);
        }
        results.reverse(); // Ensure correct order
        engine.exec_stack.push_n(&results);

        // Set PC in the new current frame (the caller), if one exists
        if let Some(caller_frame) = engine.current_frame_mut().ok() {
            // Use engine.current_frame_mut()
            caller_frame.set_pc(frame.return_pc());
            Ok(ControlFlow::Return { values: results })
        } else {
            // If no frame left, execution finished
            Ok(ControlFlow::Return { values: results }) // TODO: Maybe signal finished state?
        }
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
        _stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        call_internal(self.func_idx, engine)
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
        _stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        call_indirect(self.type_idx, self.table_idx, engine)
    }
}

#[derive(Debug, Default)]
pub struct Nop;

impl InstructionExecutor for Nop {
    fn execute(
        &self,
        _stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        Ok(ControlFlow::Continue)
    }
}

#[derive(Debug, Default)]
pub struct Unreachable;

impl InstructionExecutor for Unreachable {
    fn execute(
        &self,
        _stack: &mut dyn StackBehavior,
        _frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        Err(Error::new(kinds::Trap("unreachable executed".to_string())))
    }
}

#[derive(Debug, Default)]
pub struct Else;

impl InstructionExecutor for Else {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let stack_len = stack.len();
        // Cast directly to StacklessFrame similar to Block, Loop, and If
        if let Some(frame_cast) = frame
            .as_any()
            .downcast_mut::<crate::stackless_frame::StacklessFrame>()
        {
            frame_cast.enter_else(stack_len)?;
            Ok(ControlFlow::Continue)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Frame doesn't implement ControlFlowBehavior".to_string(),
            )))
        }
    }
}

#[derive(Debug, Default)]
pub struct End;

impl InstructionExecutor for End {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        // Cast directly to StacklessFrame similar to Block, Loop, and If
        if let Some(frame_cast) = frame
            .as_any()
            .downcast_mut::<crate::stackless_frame::StacklessFrame>()
        {
            frame_cast.exit_block(stack)?;
            Ok(ControlFlow::Continue)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Frame doesn't implement ControlFlowBehavior".to_string(),
            )))
        }
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
