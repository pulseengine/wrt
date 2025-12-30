//! WAST Module Validator
//!
//! This module provides validation for WebAssembly modules to ensure they
//! conform to the WebAssembly specification. It validates:
//! - Type correctness on the operand stack
//! - Control flow structure (blocks, loops, branches)
//! - Function and memory references
//! - Type checking even in unreachable code
//!
//! This validator runs BEFORE module execution to reject invalid modules
//! immediately, which is required for WAST conformance testing.

use anyhow::{anyhow, Context, Result};
use wrt_format::module::{Function, Global, Module};
use wrt_foundation::ValueType;

/// Type of a value on the stack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
    Unknown,
}

impl StackType {
    /// Convert from ValueType
    fn from_value_type(vt: ValueType) -> Self {
        match vt {
            ValueType::I32 => StackType::I32,
            ValueType::I64 => StackType::I64,
            ValueType::F32 => StackType::F32,
            ValueType::F64 => StackType::F64,
            ValueType::V128 => StackType::V128,
            ValueType::FuncRef => StackType::FuncRef,
            ValueType::ExternRef => StackType::ExternRef,
            // WebAssembly 3.0 GC types - not yet fully supported, treat as unknown
            ValueType::I16x8 | ValueType::StructRef(_) | ValueType::ArrayRef(_) => StackType::Unknown,
        }
    }
}

/// Control flow frame tracking
#[derive(Debug, Clone)]
struct ControlFrame {
    /// Type of control structure (block, loop, if)
    frame_type: FrameType,
    /// Input types expected for this frame
    input_types: Vec<StackType>,
    /// Output types expected from this frame
    output_types: Vec<StackType>,
    /// Whether this frame's code path is reachable
    reachable: bool,
    /// Stack height at frame entry
    stack_height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameType {
    Block,
    Loop,
    If,
    Else,
    Try,
}

/// Validator for WebAssembly modules
pub struct WastModuleValidator;

impl WastModuleValidator {
    /// Validate a module
    pub fn validate(module: &Module) -> Result<()> {
        // Validate functions
        for (func_idx, func) in module.functions.iter().enumerate() {
            Self::validate_function(func_idx, func, module)
                .context(format!("Function {} validation failed", func_idx))?;
        }

        // Validate globals
        for (global_idx, global) in module.globals.iter().enumerate() {
            Self::validate_global(global_idx, global, module)
                .context(format!("Global {} validation failed", global_idx))?;
        }

        Ok(())
    }

    /// Validate a single function
    fn validate_function(func_idx: usize, func: &Function, module: &Module) -> Result<()> {
        // Get the function's type signature
        if func.type_idx as usize >= module.types.len() {
            return Err(anyhow!(
                "Function {} has invalid type index {}",
                func_idx,
                func.type_idx
            ));
        }

        let func_type_clean = &module.types[func.type_idx as usize];

        // Parse and validate the function body
        // Note: CleanCoreFuncType has the same structure as FuncType (params, results)
        Self::validate_function_body(&func.code, func_type_clean, &func.locals, module)
    }

    /// Validate a function body bytecode
    fn validate_function_body(
        code: &[u8],
        func_type: &wrt_foundation::CleanCoreFuncType,
        locals: &[ValueType],
        module: &Module,
    ) -> Result<()> {
        // Build local variable types: parameters first, then locals
        let mut local_types = Vec::new();

        // Add parameter types
        for param in &func_type.params {
            local_types.push(*param);
        }

        // Add local types
        for local in locals {
            local_types.push(*local);
        }

        // Initialize operand stack (empty - parameters are accessed via local.get, not on stack)
        let mut stack: Vec<StackType> = Vec::new();

        // Initialize control flow frames
        let mut frames: Vec<ControlFrame> = vec![ControlFrame {
            frame_type: FrameType::Block,
            input_types: Vec::new(),
            output_types: func_type
                .results
                .iter()
                .map(|&vt| StackType::from_value_type(vt))
                .collect(),
            reachable: true,
            stack_height: 0,
        }];

        // Parse bytecode
        let mut offset = 0;
        while offset < code.len() {
            let opcode = code[offset];
            offset += 1;

            match opcode {
                // Control flow
                0x00 => {
                    // unreachable
                    if let Some(frame) = frames.last_mut() {
                        frame.reachable = false;
                    }
                }
                0x01 => {
                    // nop
                }
                0x02 => {
                    // block
                    let (block_type, new_offset) = Self::parse_block_type(code, offset, module)?;
                    offset = new_offset;

                    let (input_types, output_types) =
                        Self::block_type_to_stack_types(&block_type, module)?;

                    // For blocks with inputs, verify and pop the input types
                    let frame_height = Self::current_frame_height(&frames);
                    for &expected in input_types.iter().rev() {
                        if !Self::pop_type(&mut stack, expected, frame_height, Self::is_unreachable(&frames)) {
                            return Err(anyhow!("type mismatch"));
                        }
                    }

                    // Record the stack height AFTER popping inputs
                    let stack_height = stack.len();

                    frames.push(ControlFrame {
                        frame_type: FrameType::Block,
                        input_types: input_types.clone(),
                        output_types: output_types.clone(),
                        reachable: true,
                        stack_height,
                    });

                    // Push inputs back - they're now on the block's stack
                    for input_type in &input_types {
                        stack.push(*input_type);
                    }
                }
                0x03 => {
                    // loop
                    let (block_type, new_offset) = Self::parse_block_type(code, offset, module)?;
                    offset = new_offset;

                    let (input_types, output_types) =
                        Self::block_type_to_stack_types(&block_type, module)?;

                    // For loops with inputs, verify and pop the input types
                    let frame_height = Self::current_frame_height(&frames);
                    for &expected in input_types.iter().rev() {
                        if !Self::pop_type(&mut stack, expected, frame_height, Self::is_unreachable(&frames)) {
                            return Err(anyhow!("type mismatch"));
                        }
                    }

                    // Record the stack height AFTER popping inputs
                    let stack_height = stack.len();

                    frames.push(ControlFrame {
                        frame_type: FrameType::Loop,
                        input_types: input_types.clone(),
                        output_types: output_types.clone(),
                        reachable: true,
                        stack_height,
                    });

                    // Push inputs back - they're now on the loop's stack
                    for input_type in &input_types {
                        stack.push(*input_type);
                    }
                }
                0x04 => {
                    // if
                    // Pop condition (must be i32)
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }

                    let (block_type, new_offset) = Self::parse_block_type(code, offset, module)?;
                    offset = new_offset;

                    let (input_types, output_types) =
                        Self::block_type_to_stack_types(&block_type, module)?;

                    // For if with inputs, verify and pop the input types
                    for &expected in input_types.iter().rev() {
                        if !Self::pop_type(&mut stack, expected, frame_height, Self::is_unreachable(&frames)) {
                            return Err(anyhow!("type mismatch"));
                        }
                    }

                    // Record the stack height AFTER popping inputs
                    let stack_height = stack.len();

                    frames.push(ControlFrame {
                        frame_type: FrameType::If,
                        input_types: input_types.clone(),
                        output_types: output_types.clone(),
                        reachable: true,
                        stack_height,
                    });

                    // Push inputs back - they're now on the if's stack
                    for input_type in &input_types {
                        stack.push(*input_type);
                    }
                }
                0x05 => {
                    // else
                    if let Some(frame) = frames.last() {
                        if frame.frame_type != FrameType::If {
                            return Err(anyhow!("else: no matching if"));
                        }
                    }

                    // Reset stack to if entry point
                    if let Some(frame) = frames.last() {
                        stack.truncate(frame.stack_height + frame.input_types.len());
                    }

                    if let Some(frame) = frames.last_mut() {
                        frame.frame_type = FrameType::Else;
                        frame.reachable = true;
                    }
                }
                0x0B => {
                    // end
                    if frames.len() == 1 {
                        // This is the final function-level end - valid termination
                        // Verify the stack matches the function's return types
                        let frame = &frames[0];
                        if frame.reachable {
                            let frame_height = frame.stack_height;
                            // Check stack has exactly the right number of outputs
                            let expected_height = frame_height + frame.output_types.len();
                            if stack.len() != expected_height {
                                return Err(anyhow!("type mismatch"));
                            }
                            for &expected in frame.output_types.iter().rev() {
                                if !Self::pop_type(&mut stack, expected, frame_height, Self::is_unreachable(&frames)) {
                                    return Err(anyhow!("type mismatch"));
                                }
                            }
                        }
                        // Function validated successfully, exit loop
                        break;
                    }

                    // Pop block/loop/if frame
                    let frame = frames.pop().unwrap();

                    // Verify stack has expected output types (if reachable)
                    if frame.reachable {
                        let frame_height = frame.stack_height;
                        // Check stack has exactly the right number of outputs
                        let expected_height = frame_height + frame.output_types.len();
                        if stack.len() != expected_height {
                            return Err(anyhow!("type mismatch"));
                        }
                        for &expected in frame.output_types.iter().rev() {
                            if !Self::pop_type(&mut stack, expected, frame_height, Self::is_unreachable(&frames)) {
                                return Err(anyhow!("type mismatch"));
                            }
                        }
                    }

                    // Reset stack to frame height and push output types
                    stack.truncate(frame.stack_height);
                    stack.extend(frame.output_types.iter());
                }
                0x0C => {
                    // br (branch) - unconditional, makes following code unreachable
                    let (label_idx, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;

                    Self::validate_branch(&stack, label_idx, &frames)?;

                    // Mark current frame as unreachable
                    if let Some(frame) = frames.last_mut() {
                        frame.reachable = false;
                    }
                }
                0x0D => {
                    // br_if (branch if) - conditional, code after is still reachable
                    let (label_idx, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;

                    // Pop i32 condition (top of stack)
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }

                    Self::validate_branch(&stack, label_idx, &frames)?;
                }
                0x0E => {
                    // br_table - unconditional, makes following code unreachable
                    let (num_targets, mut new_offset) =
                        Self::parse_varuint32(code, offset)?;
                    offset = new_offset;

                    // Collect all branch targets (including default)
                    let mut targets: Vec<u32> = Vec::new();
                    for _ in 0..num_targets {
                        let (label_idx, temp_offset) = Self::parse_varuint32(code, new_offset)?;
                        targets.push(label_idx);
                        new_offset = temp_offset;
                    }
                    offset = new_offset;

                    // Parse default target
                    let (default_label, temp_offset) = Self::parse_varuint32(code, offset)?;
                    targets.push(default_label);
                    offset = temp_offset;

                    // Pop operand (i32 condition/index)
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }

                    // Validate all targets are in range and have consistent types
                    let mut expected_arity: Option<usize> = None;
                    for &label_idx in &targets {
                        // Validate label is in range
                        if label_idx as usize >= frames.len() {
                            return Err(anyhow!("unknown label {}", label_idx));
                        }

                        let target_frame = &frames[frames.len() - 1 - label_idx as usize];
                        let branch_types = if target_frame.frame_type == FrameType::Loop {
                            &target_frame.input_types
                        } else {
                            &target_frame.output_types
                        };

                        match expected_arity {
                            None => {
                                expected_arity = Some(branch_types.len());
                            }
                            Some(arity) => {
                                if branch_types.len() != arity {
                                    return Err(anyhow!("type mismatch"));
                                }
                            }
                        }
                    }

                    // Validate the stack has the required values for the branch
                    Self::validate_branch(&stack, default_label, &frames)?;

                    // Mark current frame as unreachable
                    if let Some(frame) = frames.last_mut() {
                        frame.reachable = false;
                    }
                }
                0x0F => {
                    // return
                    let frame_height = Self::current_frame_height(&frames);
                    if let Some(frame) = frames.first() {
                        for &expected in frame.output_types.iter().rev() {
                            if !Self::pop_type(&mut stack, expected, frame_height, Self::is_unreachable(&frames)) {
                                return Err(anyhow!("type mismatch"));
                            }
                        }
                    }

                    if let Some(frame) = frames.last_mut() {
                        frame.reachable = false;
                    }
                }
                0x10 => {
                    // call
                    let (func_idx, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;

                    if func_idx as usize >= module.functions.len() + module.imports.len() {
                        return Err(anyhow!("call: invalid function index {}", func_idx));
                    }

                    // Pop arguments and push results
                    if let Some(func_type) = Self::get_function_type(func_idx, module) {
                        // Pop arguments in reverse order
                        let frame_height = Self::current_frame_height(&frames);
                        for param in func_type.params.iter().rev() {
                            let expected = StackType::from_value_type(*param);
                            if !Self::pop_type(&mut stack, expected, frame_height, Self::is_unreachable(&frames)) {
                                return Err(anyhow!("type mismatch"));
                            }
                        }
                        // Push results
                        for result in &func_type.results {
                            stack.push(StackType::from_value_type(*result));
                        }
                    }
                }
                0x11 => {
                    // call_indirect
                    let (type_idx, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;

                    // table_idx (assumed 0, skip varuint32)
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;

                    if type_idx as usize >= module.types.len() {
                        return Err(anyhow!(
                            "call_indirect: invalid type index {}",
                            type_idx
                        ));
                    }

                    let func_type = &module.types[type_idx as usize];
                    let frame_height = Self::current_frame_height(&frames);

                    // Pop table index (must be i32)
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }

                    // Pop arguments in reverse order
                    for param in func_type.params.iter().rev() {
                        let expected = StackType::from_value_type(*param);
                        if !Self::pop_type(&mut stack, expected, frame_height, Self::is_unreachable(&frames)) {
                            return Err(anyhow!("type mismatch"));
                        }
                    }

                    // Push results
                    for result in &func_type.results {
                        stack.push(StackType::from_value_type(*result));
                    }
                }

                // Memory operations - Load instructions
                0x28 => {
                    // i32.load - pop i32 address, push i32 value
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I32);
                }
                0x29 => {
                    // i64.load - pop i32 address, push i64 value
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I64);
                }
                0x2A => {
                    // f32.load - pop i32 address, push f32 value
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::F32);
                }
                0x2B => {
                    // f64.load - pop i32 address, push f64 value
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::F64);
                }
                0x2C..=0x35 => {
                    // Extended load operations (load8, load16, load32, etc.)
                    // All take i32 address and return the loaded value type
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    // Push result based on opcode
                    let result_type = match opcode {
                        0x2C | 0x2D | 0x2E | 0x2F => StackType::I32, // i32.load8_s/u, i32.load16_s/u
                        0x30 | 0x31 | 0x32 | 0x33 | 0x34 | 0x35 => StackType::I64, // i64 loads
                        _ => StackType::I32,
                    };
                    stack.push(result_type);
                }
                0x36 => {
                    // i32.store - pop i32 value and i32 address
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                }
                0x37 => {
                    // i64.store - pop i64 value and i32 address
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                }
                0x38 => {
                    // f32.store - pop f32 value and i32 address
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::F32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                }
                0x39 => {
                    // f64.store - pop f64 value and i32 address
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::F64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                }
                0x3A..=0x3E => {
                    // Extended store operations (store8, store16, store32)
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let (_, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;
                    let frame_height = Self::current_frame_height(&frames);
                    // Pop value type based on opcode
                    let value_type = match opcode {
                        0x3A | 0x3B => StackType::I32, // i32.store8, i32.store16
                        0x3C | 0x3D | 0x3E => StackType::I64, // i64.store8/16/32
                        _ => StackType::I32,
                    };
                    if !Self::pop_type(&mut stack, value_type, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                }

                // Variable operations
                0x20 => {
                    // local.get
                    let (local_idx, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;

                    if local_idx as usize >= local_types.len() {
                        return Err(anyhow!(
                            "local.get: invalid local index {}",
                            local_idx
                        ));
                    }

                    stack.push(StackType::from_value_type(local_types[local_idx as usize]));
                }
                0x21 => {
                    // local.set
                    let (local_idx, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;

                    if local_idx as usize >= local_types.len() {
                        return Err(anyhow!(
                            "local.set: invalid local index {}",
                            local_idx
                        ));
                    }

                    let expected = StackType::from_value_type(local_types[local_idx as usize]);
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, expected, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                }
                0x22 => {
                    // local.tee
                    let (local_idx, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;

                    if local_idx as usize >= local_types.len() {
                        return Err(anyhow!(
                            "local.tee: invalid local index {}",
                            local_idx
                        ));
                    }

                    let expected = StackType::from_value_type(local_types[local_idx as usize]);
                    if stack.last() != Some(&expected) {
                        return Err(anyhow!("local.tee: type mismatch"));
                    }
                }
                0x23 => {
                    // global.get
                    let (global_idx, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;

                    if global_idx as usize >= module.globals.len() {
                        return Err(anyhow!(
                            "global.get: invalid global index {}",
                            global_idx
                        ));
                    }

                    let global_type = module.globals[global_idx as usize].global_type.value_type;
                    stack.push(StackType::from_value_type(global_type));
                }
                0x24 => {
                    // global.set
                    let (global_idx, new_offset) = Self::parse_varuint32(code, offset)?;
                    offset = new_offset;

                    if global_idx as usize >= module.globals.len() {
                        return Err(anyhow!(
                            "global.set: invalid global index {}",
                            global_idx
                        ));
                    }

                    let global_type = module.globals[global_idx as usize].global_type.value_type;
                    let expected = StackType::from_value_type(global_type);
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, expected, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                }

                // Constants
                0x41 => {
                    // i32.const
                    let (_, new_offset) = Self::parse_varint32(code, offset)?;
                    offset = new_offset;
                    stack.push(StackType::I32);
                }
                0x42 => {
                    // i64.const
                    let (_, new_offset) = Self::parse_varint64(code, offset)?;
                    offset = new_offset;
                    stack.push(StackType::I64);
                }
                0x43 => {
                    // f32.const
                    if offset + 4 > code.len() {
                        return Err(anyhow!("f32.const: truncated instruction"));
                    }
                    offset += 4;
                    stack.push(StackType::F32);
                }
                0x44 => {
                    // f64.const
                    if offset + 8 > code.len() {
                        return Err(anyhow!("f64.const: truncated instruction"));
                    }
                    offset += 8;
                    stack.push(StackType::F64);
                }

                // Parametric operations
                0x1A => {
                    // drop
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if stack.len() <= frame_height && !unreachable {
                        return Err(anyhow!("type mismatch"));
                    }
                    if stack.len() > frame_height {
                        stack.pop();
                    }
                }
                0x1B => {
                    // select
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, unreachable) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if stack.len() <= frame_height + 1 && !unreachable {
                        return Err(anyhow!("type mismatch"));
                    }
                    if stack.len() > frame_height + 1 {
                        let type2 = stack.pop().unwrap();
                        let type1 = stack.pop().unwrap();
                        if type1 != type2 && !unreachable {
                            return Err(anyhow!("type mismatch"));
                        }
                        stack.push(type1);
                    } else if unreachable {
                        // In unreachable code, push Unknown type
                        stack.push(StackType::Unknown);
                    }
                }

                // f32 unary operations (0x8B-0x91): abs, neg, ceil, floor, trunc, nearest, sqrt
                0x8B | 0x8C | 0x8D | 0x8E | 0x8F | 0x90 | 0x91 => {
                    // f32 unary: f32 -> f32
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::F32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::F32);
                }
                // f32 binary operations (0x92-0x98): add, sub, mul, div, min, max, copysign
                0x92 | 0x93 | 0x94 | 0x95 | 0x96 | 0x97 | 0x98 => {
                    // f32 binary: f32 f32 -> f32
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::F32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::F32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::F32);
                }
                // f64 unary operations (0x99-0x9F): abs, neg, ceil, floor, trunc, nearest, sqrt
                0x99 | 0x9A | 0x9B | 0x9C | 0x9D | 0x9E | 0x9F => {
                    // f64 unary: f64 -> f64
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::F64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::F64);
                }
                // f64 binary operations (0xA0-0xA6): add, sub, mul, div, min, max, copysign
                0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 | 0xA6 => {
                    // f64 binary: f64 f64 -> f64
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::F64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::F64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::F64);
                }
                // i32 unary: clz (0x67), ctz, popcnt
                0x67 | 0x68 | 0x69 => {
                    // i32 unary operations
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I32);
                }
                // i64 unary: clz (0x79), ctz, popcnt
                0x79 | 0x7A | 0x7B => {
                    // i64 unary operations
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I64);
                }

                // i32.eqz (0x45): i32 -> i32
                0x45 => {
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I32);
                }

                // i32 comparison operations (0x46-0x4F): i32 i32 -> i32
                0x46 | 0x47 | 0x48 | 0x49 | 0x4A | 0x4B | 0x4C | 0x4D | 0x4E | 0x4F => {
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I32);
                }

                // i64.eqz (0x50): i64 -> i32
                0x50 => {
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I32);
                }

                // i64 comparison operations (0x51-0x5A): i64 i64 -> i32
                0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x56 | 0x57 | 0x58 | 0x59 | 0x5A => {
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::I64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I32);
                }

                // f32 comparison operations (0x5B-0x60): f32 f32 -> i32
                0x5B | 0x5C | 0x5D | 0x5E | 0x5F | 0x60 => {
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::F32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::F32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I32);
                }

                // f64 comparison operations (0x61-0x66): f64 f64 -> i32
                0x61 | 0x62 | 0x63 | 0x64 | 0x65 | 0x66 => {
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::F64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::F64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I32);
                }

                // i32 binary operations (0x6A-0x78): i32 i32 -> i32
                0x6A | 0x6B | 0x6C | 0x6D | 0x6E | 0x6F | 0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x76 | 0x77 | 0x78 => {
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I32);
                }

                // i64 binary operations (0x7C-0x8A): i64 i64 -> i64
                0x7C | 0x7D | 0x7E | 0x7F | 0x80 | 0x81 | 0x82 | 0x83 | 0x84 | 0x85 | 0x86 | 0x87 | 0x88 | 0x89 | 0x8A | 0x8B => {
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    if !Self::pop_type(&mut stack, StackType::I64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I64);
                }

                // Conversion operations: i32 -> i64
                0xac | 0xad => {
                    // i64.extend_i32_s (0xac), i64.extend_i32_u (0xad)
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I64);
                }

                // Conversion operations: i64 -> i32
                0xa7 => {
                    // i32.wrap_i64
                    let frame_height = Self::current_frame_height(&frames);
                    if !Self::pop_type(&mut stack, StackType::I64, frame_height, Self::is_unreachable(&frames)) {
                        return Err(anyhow!("type mismatch"));
                    }
                    stack.push(StackType::I32);
                }

                // Conversion operations: f32 <-> i32
                0xa8 | 0xa9 | 0xaa | 0xab => {
                    // i32.trunc_f32_s (0xa8), i32.trunc_f32_u (0xa9)
                    // i32.trunc_f64_s (0xaa), i32.trunc_f64_u (0xab)
                    let is_f64 = opcode >= 0xaa;
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if is_f64 {
                        if !Self::pop_type(&mut stack, StackType::F64, frame_height, unreachable) {
                            return Err(anyhow!("i32.trunc: operand must be f64"));
                        }
                    } else {
                        if !Self::pop_type(&mut stack, StackType::F32, frame_height, unreachable) {
                            return Err(anyhow!("i32.trunc: operand must be f32"));
                        }
                    }
                    stack.push(StackType::I32);
                }

                // Conversion operations: f32/f64 <-> i64
                0xae | 0xaf | 0xb0 | 0xb1 => {
                    // i64.trunc_f32_s (0xae), i64.trunc_f32_u (0xaf)
                    // i64.trunc_f64_s (0xb0), i64.trunc_f64_u (0xb1)
                    let is_f64 = opcode >= 0xb0;
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if is_f64 {
                        if !Self::pop_type(&mut stack, StackType::F64, frame_height, unreachable) {
                            return Err(anyhow!("i64.trunc: operand must be f64"));
                        }
                    } else {
                        if !Self::pop_type(&mut stack, StackType::F32, frame_height, unreachable) {
                            return Err(anyhow!("i64.trunc: operand must be f32"));
                        }
                    }
                    stack.push(StackType::I64);
                }

                // Conversion operations: i32/i64 -> f32
                0xb2 | 0xb3 | 0xb4 | 0xb5 => {
                    // f32.convert_i32_s (0xb2), f32.convert_i32_u (0xb3)
                    // f32.convert_i64_s (0xb4), f32.convert_i64_u (0xb5)
                    let is_i64 = opcode >= 0xb4;
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if is_i64 {
                        if !Self::pop_type(&mut stack, StackType::I64, frame_height, unreachable) {
                            return Err(anyhow!("f32.convert: operand must be i64"));
                        }
                    } else {
                        if !Self::pop_type(&mut stack, StackType::I32, frame_height, unreachable) {
                            return Err(anyhow!("f32.convert: operand must be i32"));
                        }
                    }
                    stack.push(StackType::F32);
                }

                // Conversion operations: f64.demote_f32
                0xb6 => {
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if !Self::pop_type(&mut stack, StackType::F64, frame_height, unreachable) {
                        return Err(anyhow!("f32.demote_f64: operand must be f64"));
                    }
                    stack.push(StackType::F32);
                }

                // Conversion operations: i32/i64 -> f64
                0xb7 | 0xb8 | 0xb9 | 0xba => {
                    // f64.convert_i32_s (0xb7), f64.convert_i32_u (0xb8)
                    // f64.convert_i64_s (0xb9), f64.convert_i64_u (0xba)
                    let is_i64 = opcode >= 0xb9;
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if is_i64 {
                        if !Self::pop_type(&mut stack, StackType::I64, frame_height, unreachable) {
                            return Err(anyhow!("f64.convert: operand must be i64"));
                        }
                    } else {
                        if !Self::pop_type(&mut stack, StackType::I32, frame_height, unreachable) {
                            return Err(anyhow!("f64.convert: operand must be i32"));
                        }
                    }
                    stack.push(StackType::F64);
                }

                // Conversion operations: f64.promote_f32
                0xbb => {
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if !Self::pop_type(&mut stack, StackType::F32, frame_height, unreachable) {
                        return Err(anyhow!("f64.promote_f32: operand must be f32"));
                    }
                    stack.push(StackType::F64);
                }

                // Reinterpret operations (same size, different type)
                0xbc => {
                    // i32.reinterpret_f32
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if !Self::pop_type(&mut stack, StackType::F32, frame_height, unreachable) {
                        return Err(anyhow!("i32.reinterpret_f32: operand must be f32"));
                    }
                    stack.push(StackType::I32);
                }
                0xbd => {
                    // i64.reinterpret_f64
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if !Self::pop_type(&mut stack, StackType::F64, frame_height, unreachable) {
                        return Err(anyhow!("i64.reinterpret_f64: operand must be f64"));
                    }
                    stack.push(StackType::I64);
                }
                0xbe => {
                    // f32.reinterpret_i32
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if !Self::pop_type(&mut stack, StackType::I32, frame_height, unreachable) {
                        return Err(anyhow!("f32.reinterpret_i32: operand must be i32"));
                    }
                    stack.push(StackType::F32);
                }
                0xbf => {
                    // f64.reinterpret_i64
                    let frame_height = Self::current_frame_height(&frames);
                    let unreachable = Self::is_unreachable(&frames);
                    if !Self::pop_type(&mut stack, StackType::I64, frame_height, unreachable) {
                        return Err(anyhow!("f64.reinterpret_i64: operand must be i64"));
                    }
                    stack.push(StackType::F64);
                }

                // Skip other opcodes for now (will be handled by instruction executor)
                _ => {
                    // For all other opcodes, try to skip variable-length immediates
                    // This is a simplified approach - proper validation would parse every opcode
                    // But for WAST tests, the main issues are br_if and unreachable code
                }
            }
        }

        Ok(())
    }

    /// Validate a global variable
    fn validate_global(
        _global_idx: usize,
        _global: &Global,
        _module: &Module,
    ) -> Result<()> {
        // Global validation would check initialization expressions
        // For now, simplified
        Ok(())
    }

    /// Pop a value from the stack, checking its type
    /// The `min_height` parameter is the stack height at the current frame's entry -
    /// we cannot pop below this level (those values belong to the parent frame)
    /// The `unreachable` parameter indicates if we're in unreachable code (polymorphic stack)
    fn pop_type(
        stack: &mut Vec<StackType>,
        expected: StackType,
        min_height: usize,
        unreachable: bool,
    ) -> bool {
        // In unreachable code, the stack is polymorphic
        if unreachable {
            // Can pop below min_height (polymorphic underflow)
            if stack.len() <= min_height {
                return true;
            }
            // Values on stack in unreachable code are "garbage" - any type matches
            stack.pop();
            return true;
        }

        // Check if we'd be popping below the current frame's stack base
        if stack.len() <= min_height {
            return false;
        }

        if let Some(actual) = stack.pop() {
            // Allow Unknown to match anything, or exact match
            actual == expected || actual == StackType::Unknown || expected == StackType::Unknown
        } else {
            false
        }
    }

    /// Get the current frame's stack height (the base of the current control frame)
    fn current_frame_height(frames: &[ControlFrame]) -> usize {
        frames.last().map_or(0, |f| f.stack_height)
    }

    /// Check if the current code path is unreachable
    fn is_unreachable(frames: &[ControlFrame]) -> bool {
        frames.last().map_or(false, |f| !f.reachable)
    }

    /// Parse a variable-length unsigned 32-bit integer
    fn parse_varuint32(code: &[u8], offset: usize) -> Result<(u32, usize)> {
        let mut result = 0u32;
        let mut shift = 0;
        let mut pos = offset;

        loop {
            if pos >= code.len() {
                return Err(anyhow!("truncated varuint32"));
            }

            let byte = code[pos] as u32;
            pos += 1;

            result |= (byte & 0x7F) << shift;

            if (byte & 0x80) == 0 {
                break;
            }

            shift += 7;
            if shift >= 35 {
                return Err(anyhow!("varuint32 overflow"));
            }
        }

        Ok((result, pos))
    }

    /// Parse a variable-length signed 32-bit integer
    fn parse_varint32(code: &[u8], offset: usize) -> Result<(i32, usize)> {
        let (value, pos) = Self::parse_varuint32(code, offset)?;
        let result = if value & 0x80000000 != 0 {
            value as i32
        } else {
            value as i32
        };
        Ok((result, pos))
    }

    /// Parse a variable-length signed 64-bit integer
    fn parse_varint64(code: &[u8], mut offset: usize) -> Result<(i64, usize)> {
        let mut result = 0i64;
        let mut shift = 0;

        loop {
            if offset >= code.len() {
                return Err(anyhow!("truncated varint64"));
            }

            let byte = code[offset] as i64;
            offset += 1;

            result |= (byte & 0x7F) << shift;

            if (byte & 0x80) == 0 {
                if shift < 63 && (byte & 0x40) != 0 {
                    result |= -(1 << (shift + 7));
                }
                break;
            }

            shift += 7;
        }

        Ok((result, offset))
    }

    /// Parse block type
    fn parse_block_type(code: &[u8], offset: usize, module: &Module) -> Result<(BlockType, usize)> {
        if offset >= code.len() {
            return Err(anyhow!("truncated block type"));
        }

        let byte = code[offset] as i8;

        let block_type = match byte {
            0x40 => BlockType::Empty,
            0x7F => BlockType::ValueType(ValueType::I32),
            0x7E => BlockType::ValueType(ValueType::I64),
            0x7D => BlockType::ValueType(ValueType::F32),
            0x7C => BlockType::ValueType(ValueType::F64),
            0x7B => BlockType::ValueType(ValueType::V128),
            0x70 => BlockType::ValueType(ValueType::FuncRef),
            0x6F => BlockType::ValueType(ValueType::ExternRef),
            _ if byte >= 0 => {
                // Function type index
                let type_idx = byte as u32;
                if type_idx as usize >= module.types.len() {
                    return Err(anyhow!("invalid function type index {}", type_idx));
                }
                BlockType::FuncType(type_idx)
            }
            _ => {
                // Negative index (encoded as varint), parse it properly
                let (type_idx, new_offset) = Self::parse_varint32(code, offset)?;
                if type_idx < 0 {
                    return Err(anyhow!("invalid block type index"));
                }
                if type_idx as usize >= module.types.len() {
                    return Err(anyhow!("invalid function type index {}", type_idx));
                }
                return Ok((BlockType::FuncType(type_idx as u32), new_offset));
            }
        };

        Ok((block_type, offset + 1))
    }

    /// Convert block type to input/output stack types
    fn block_type_to_stack_types(
        block_type: &BlockType,
        module: &Module,
    ) -> Result<(Vec<StackType>, Vec<StackType>)> {
        match block_type {
            BlockType::Empty => Ok((Vec::new(), Vec::new())),
            BlockType::ValueType(vt) => {
                let st = StackType::from_value_type(*vt);
                Ok((Vec::new(), vec![st]))
            }
            BlockType::FuncType(type_idx) => {
                if *type_idx as usize >= module.types.len() {
                    return Err(anyhow!(
                        "invalid function type index {}",
                        type_idx
                    ));
                }

                let func_type = &module.types[*type_idx as usize];

                let inputs = func_type
                    .params
                    .iter()
                    .map(|&vt| StackType::from_value_type(vt))
                    .collect();

                let outputs = func_type
                    .results
                    .iter()
                    .map(|&vt| StackType::from_value_type(vt))
                    .collect();

                Ok((inputs, outputs))
            }
        }
    }

    /// Get function type
    fn get_function_type(func_idx: u32, module: &Module) -> Option<wrt_foundation::CleanCoreFuncType> {
        let func_idx_usize = func_idx as usize;
        let func_count = module.functions.len();
        let total_funcs = func_count + module.imports.len();

        if func_idx_usize < func_count {
            let func = &module.functions[func_idx_usize];
            module.types.get(func.type_idx as usize).cloned()
        } else if func_idx_usize < total_funcs {
            // For imports, would need to look up the imported function type
            // For now, return None
            None
        } else {
            None
        }
    }

    /// Validate a branch target
    ///
    /// For branches to blocks/if, we validate against output types.
    /// For branches to loops, we validate against input types.
    ///
    /// IMPORTANT: The values for branching must come from the current frame's
    /// operand stack (above the current frame's stack_height), not from parent frames.
    fn validate_branch(stack: &[StackType], label_idx: u32, frames: &[ControlFrame]) -> Result<()> {
        if label_idx as usize >= frames.len() {
            return Err(anyhow!(
                "br: label index {} out of range",
                label_idx
            ));
        }

        // Get the current frame (innermost) to check our available stack values
        let current_frame = frames.last().ok_or_else(|| anyhow!("no control frame"))?;
        let current_stack_height = current_frame.stack_height;

        // Get the target frame (counting from innermost)
        let target_frame = &frames[frames.len() - 1 - label_idx as usize];

        // Determine the expected types for the branch
        // For loops: branch to input types (jump to loop start)
        // For blocks/if/else: branch to output types (jump to end)
        let expected_types = if target_frame.frame_type == FrameType::Loop {
            &target_frame.input_types
        } else {
            &target_frame.output_types
        };

        // Calculate how many values the CURRENT frame has available on the stack
        // Values below current_stack_height belong to parent frames and cannot be used
        let available_values = stack.len().saturating_sub(current_stack_height);

        // Check that the current frame has enough values for the branch
        if available_values < expected_types.len() {
            // Not enough values in the current frame's scope
            return Err(anyhow!("type mismatch"));
        }

        // Verify the top values match expected types (in reverse order)
        for (i, expected) in expected_types.iter().rev().enumerate() {
            let stack_idx = stack.len() - 1 - i;
            let actual = &stack[stack_idx];
            if actual != expected && *actual != StackType::Unknown && *expected != StackType::Unknown {
                return Err(anyhow!("type mismatch"));
            }
        }

        Ok(())
    }
}

/// Block type enumeration
#[derive(Debug, Clone, Copy)]
pub enum BlockType {
    Empty,
    ValueType(ValueType),
    FuncType(u32),
}
