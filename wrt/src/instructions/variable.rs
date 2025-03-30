//! WebAssembly variable instructions
//!
//! This module contains implementations for all WebAssembly variable instructions,
//! including local and global variable access.

use crate::{behavior::FrameBehavior, error::Result, stack::Stack, stackless::StacklessFrame};

pub struct LocalGet {
    pub local_idx: u32,
}

impl LocalGet {
    pub const fn new(local_idx: u32) -> Self {
        Self { local_idx }
    }

    pub fn execute<S: Stack>(&self, stack: &mut S, frame: &mut StacklessFrame) -> Result<()> {
        let value = frame.get_local(self.local_idx as usize)?;
        stack.push(value)?;
        Ok(())
    }
}

pub struct LocalSet {
    pub local_idx: u32,
}

impl LocalSet {
    pub const fn new(local_idx: u32) -> Self {
        Self { local_idx }
    }

    pub fn execute<S: Stack>(&self, stack: &mut S, frame: &mut StacklessFrame) -> Result<()> {
        let value = stack.pop()?;
        frame.set_local(self.local_idx as usize, value)?;
        Ok(())
    }
}

pub struct LocalTee {
    pub local_idx: u32,
}

impl LocalTee {
    pub const fn new(local_idx: u32) -> Self {
        Self { local_idx }
    }

    pub fn execute<S: Stack>(&self, stack: &mut S, frame: &mut StacklessFrame) -> Result<()> {
        let value = stack.pop()?;
        frame.set_local(self.local_idx as usize, value.clone())?;
        stack.push(value)?;
        Ok(())
    }
}

pub struct GlobalGet {
    pub global_idx: u32,
}

impl GlobalGet {
    pub const fn new(global_idx: u32) -> Self {
        Self { global_idx }
    }

    pub fn execute<S: Stack>(&self, stack: &mut S, frame: &mut StacklessFrame) -> Result<()> {
        let global = frame.get_global(self.global_idx)?;
        stack.push(global.value.clone())?;
        Ok(())
    }
}

pub struct GlobalSet {
    pub global_idx: u32,
}

impl GlobalSet {
    pub const fn new(global_idx: u32) -> Self {
        Self { global_idx }
    }

    pub fn execute<S: Stack>(&self, stack: &mut S, frame: &mut StacklessFrame) -> Result<()> {
        let value = stack.pop()?;
        frame.set_global(self.global_idx as usize, value)?;
        Ok(())
    }
}

pub fn local_get(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    idx: u32,
) -> Result<()> {
    println!("DEBUG: local_get - Executing with idx: {idx}");
    println!(
        "DEBUG: local_get - Frame locals length: {}",
        frame.get_locals().len()
    );

    // Get the local variable value
    let value = frame.get_local(idx as usize)?;
    println!("DEBUG: local_get - Retrieved value at index {idx}: {value:?}");

    // Push the value to the stack
    stack.push(value)?;
    println!("DEBUG: local_get - Stack after push: {:?}", stack.values());

    Ok(())
}

pub fn local_set(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    idx: u32,
) -> Result<()> {
    let value = stack.pop()?;
    frame.set_local(idx as usize, value)?;
    Ok(())
}

pub fn local_tee(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    idx: u32,
) -> Result<()> {
    let value = stack.pop()?;
    frame.set_local(idx as usize, value.clone())?;
    stack.push(value)?;
    Ok(())
}

pub fn global_get(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    idx: u32,
) -> Result<()> {
    let value = frame.get_global(idx as usize)?;
    stack.push(value)?;
    Ok(())
}

pub fn global_set(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    idx: u32,
) -> Result<()> {
    let value = stack.pop()?;
    frame.set_global(idx as usize, value)?;
    Ok(())
}
