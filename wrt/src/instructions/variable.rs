//! WebAssembly variable instructions
//!
//! This module contains implementations for all WebAssembly variable instructions,
//! including local and global variable access.
//!
//! This module integrates with the pure implementations in `wrt-instructions/variable_ops.rs`,
//! providing the runtime-specific context needed for execution.

use log::debug;
use std::sync::Arc;

use crate::{
    behavior::{ControlFlow, FrameBehavior, InstructionExecutor, StackBehavior},
    error::{self, kinds, Error, Result},
    global::Global,
    module::Module,
    prelude::TypesValue as Value,
    stackless_frame::StacklessFrame,
    StacklessEngine,
};

// Import the pure implementations from wrt-instructions
use wrt_instructions::variable_ops::{VariableContext, VariableOp};

/// Implementation for `local.get` instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalGet {
    pub local_idx: u32,
}
impl LocalGet {
    pub const fn new(local_idx: u32) -> Self {
        Self { local_idx }
    }
}

/// Implementation for `local.set` instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalSet {
    pub local_idx: u32,
}
impl LocalSet {
    pub const fn new(local_idx: u32) -> Self {
        Self { local_idx }
    }
}

/// Implementation for `local.tee` instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalTee {
    pub local_idx: u32,
}
impl LocalTee {
    pub const fn new(local_idx: u32) -> Self {
        Self { local_idx }
    }
}

/// Implementation for `global.get` instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalGet {
    pub global_idx: u32,
}
impl GlobalGet {
    pub const fn new(global_idx: u32) -> Self {
        Self { global_idx }
    }
}

/// Implementation for `global.set` instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSet {
    pub global_idx: u32,
}
impl GlobalSet {
    pub const fn new(global_idx: u32) -> Self {
        Self { global_idx }
    }
}

/// Runtime adapter that bridges the pure variable operations with the stackless engine
struct RuntimeVariableContext<'a> {
    stack: &'a mut dyn StackBehavior,
    frame: &'a mut StacklessFrame,
    engine: &'a mut StacklessEngine,
}

impl<'a> VariableContext for RuntimeVariableContext<'a> {
    fn get_local(&self, index: u32) -> wrt_instructions::Result<Value> {
        self.frame
            .get_local(index as usize)
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn set_local(&mut self, index: u32, value: Value) -> wrt_instructions::Result<()> {
        self.frame
            .set_local(index as usize, value)
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn get_global(&self, index: u32) -> wrt_instructions::Result<Value> {
        self.frame
            .get_global(index as usize, self.engine)
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn set_global(&mut self, index: u32, value: Value) -> wrt_instructions::Result<()> {
        self.frame
            .set_global(index as usize, value, self.engine)
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn push_value(&mut self, value: Value) -> wrt_instructions::Result<()> {
        self.stack
            .push(value)
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }

    fn pop_value(&mut self) -> wrt_instructions::Result<Value> {
        self.stack
            .pop()
            .map_err(|e| wrt_instructions::Error::new(e.kind()))
    }
}

impl InstructionExecutor for LocalGet {
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
                error::Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in LocalGet".to_string(),
                ))
            })?;

        let mut context = RuntimeVariableContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        VariableOp::LocalGet(self.local_idx)
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        Ok(ControlFlow::Continue)
    }
}

impl InstructionExecutor for LocalSet {
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
                error::Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in LocalSet".to_string(),
                ))
            })?;

        let mut context = RuntimeVariableContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        VariableOp::LocalSet(self.local_idx)
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        Ok(ControlFlow::Continue)
    }
}

impl InstructionExecutor for LocalTee {
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
                error::Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in LocalTee".to_string(),
                ))
            })?;

        let mut context = RuntimeVariableContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        VariableOp::LocalTee(self.local_idx)
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        Ok(ControlFlow::Continue)
    }
}

impl InstructionExecutor for GlobalGet {
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
                error::Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in GlobalGet".to_string(),
                ))
            })?;

        let mut context = RuntimeVariableContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        VariableOp::GlobalGet(self.global_idx)
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        Ok(ControlFlow::Continue)
    }
}

impl InstructionExecutor for GlobalSet {
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
                error::Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in GlobalSet".to_string(),
                ))
            })?;

        let mut context = RuntimeVariableContext {
            stack,
            frame,
            engine,
        };

        // Use the pure implementation from wrt-instructions
        VariableOp::GlobalSet(self.global_idx)
            .execute(&mut context)
            .map_err(|e| Error::new(e.kind()))?;

        Ok(ControlFlow::Continue)
    }
}
