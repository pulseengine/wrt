//! WebAssembly variable instructions
//!
//! This module contains implementations for all WebAssembly variable instructions,
//! including local and global variable access.

use log::debug;
use std::sync::Arc;

use crate::{
    behavior::{ControlFlow, FrameBehavior, InstructionExecutor, StackBehavior},
    error::{self, kinds, Error, Result},
    global::Global,
    module::Module,
    stackless_frame::StacklessFrame,
    values::Value,
    StacklessEngine,
};

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

impl InstructionExecutor for LocalGet {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                error::Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in LocalGet".to_string(),
                ))
            })?;
        let value = frame.get_local(self.local_idx as usize)?;
        stack.push(value)?;
        Ok(ControlFlow::Continue)
    }
}

impl InstructionExecutor for LocalSet {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                error::Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in LocalSet".to_string(),
                ))
            })?;
        let value = stack.pop()?;
        frame.set_local(self.local_idx as usize, value)?;
        Ok(ControlFlow::Continue)
    }
}

impl InstructionExecutor for LocalTee {
    fn execute(
        &self,
        stack: &mut dyn StackBehavior,
        frame: &mut dyn FrameBehavior,
        _engine: &mut StacklessEngine,
    ) -> Result<ControlFlow> {
        let frame = frame
            .as_any()
            .downcast_mut::<StacklessFrame>()
            .ok_or_else(|| {
                error::Error::new(kinds::RuntimeError(
                    "FrameBehavior is not StacklessFrame in LocalTee".to_string(),
                ))
            })?;
        let value = stack.peek()?.clone();
        frame.set_local(self.local_idx as usize, value)?;
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
        let global = frame.get_global(self.global_idx as usize, engine)?;
        stack.push(global.get())?;
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
        let value = stack.pop()?;
        frame.set_global(self.global_idx as usize, value, engine)?;
        Ok(ControlFlow::Continue)
    }
}
