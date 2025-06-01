//! Tail call optimization implementation for the stackless engine.
//!
//! This module provides tail call optimization support, allowing functions to
//! make tail calls without growing the call stack. This is essential for
//! functional programming patterns and recursive algorithms.

use crate::prelude::*;
use crate::stackless::frame::StacklessFrame;
use crate::stackless::engine::StacklessEngine;
use crate::module_instance::ModuleInstance;
use wrt_instructions::control_ops::ControlContext;
use wrt_foundation::{Value, FuncType};
use wrt_error::{Error, Result};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Tail call implementation for the stackless engine
impl StacklessEngine {
    /// Execute a tail call to a function
    ///
    /// This replaces the current call frame with a new one for the target function,
    /// implementing proper tail call optimization.
    ///
    /// # Arguments
    ///
    /// * `func_idx` - Index of the function to tail call
    /// * `module` - The module instance containing the function
    ///
    /// # Returns
    ///
    /// Success or an error if the tail call fails
    pub fn execute_tail_call(
        &mut self, 
        func_idx: u32, 
        module: &mut ModuleInstance
    ) -> Result<()> {
        // Get the function to call
        let func = module.get_function(func_idx as usize)?;
        
        // Get function type for parameter/result validation
        let func_type = module.get_function_type(func_idx as usize)?;
        
        // Pop arguments from the operand stack
        let mut args = Vec::with_capacity(func_type.params.len());
        for _ in 0..func_type.params.len() {
            args.push(self.operand_stack.pop()?);
        }
        args.reverse(); // Arguments were popped in reverse order
        
        // For tail calls, we replace the current frame instead of pushing a new one
        if let Some(current_frame) = self.call_frames.last_mut() {
            // Save any necessary state from current frame if needed
            // (In a full implementation, we might need to handle locals differently)
            
            // Replace current frame with new frame for tail call
            *current_frame = StacklessFrame::new(
                func,
                args,
                func_type.params.clone(),
                func_type.results.clone(),
            )?;
            
            // Reset program counter to start of new function
            current_frame.set_pc(0);
        } else {
            return Err(Error::runtime_error("No active frame for tail call"));
        }
        
        // Update execution statistics
        self.stats.function_calls += 1;
        
        Ok(())
    }
    
    /// Execute a tail call through a table (return_call_indirect)
    ///
    /// This performs an indirect tail call through a function table.
    ///
    /// # Arguments
    ///
    /// * `table_idx` - Index of the table containing function references
    /// * `type_idx` - Expected function type index
    /// * `func_idx` - Function index within the table
    /// * `module` - The module instance
    ///
    /// # Returns
    ///
    /// Success or an error if the tail call fails
    pub fn execute_tail_call_indirect(
        &mut self,
        table_idx: u32,
        type_idx: u32,
        func_idx: u32,
        module: &mut ModuleInstance,
    ) -> Result<()> {
        // Get the table
        let table = module.get_table(table_idx as usize)?;
        
        // Get function reference from table
        let func_ref = table.get(func_idx)?;
        
        // Validate function reference
        let actual_func_idx = match func_ref {
            Value::FuncRef(Some(idx)) => idx,
            Value::FuncRef(None) => {
                return Err(Error::runtime_error("Null function reference in table"));
            }
            _ => {
                return Err(Error::type_error("Expected function reference in table"));
            }
        };
        
        // Get expected function type
        let expected_type = module.get_type(type_idx as usize)?;
        
        // Get actual function type
        let actual_type = module.get_function_type(actual_func_idx as usize)?;
        
        // Validate type compatibility
        if !actual_type.is_compatible_with(&expected_type) {
            return Err(Error::type_error("Function type mismatch in tail call indirect"));
        }
        
        // Execute the tail call
        self.execute_tail_call(actual_func_idx, module)
    }
}

/// Extension trait to add tail call methods to control context
pub trait TailCallContext: ControlContext {
    /// Execute a tail call
    fn execute_return_call(&mut self, func_idx: u32) -> Result<()>;
    
    /// Execute an indirect tail call
    fn execute_return_call_indirect(&mut self, table_idx: u32, type_idx: u32) -> Result<()>;
}

/// Helper functions for tail call validation
pub mod validation {
    use super::*;
    
    /// Validate that a tail call is valid in the current context
    ///
    /// Tail calls are valid when:
    /// 1. The current function's return type matches the called function's return type
    /// 2. The operand stack has exactly the right number of arguments
    pub fn validate_tail_call(
        current_func_type: &FuncType,
        target_func_type: &FuncType,
    ) -> Result<()> {
        // Check return type compatibility
        if current_func_type.results != target_func_type.results {
            return Err(Error::validation_error(
                "Tail call return type mismatch: current function and target function must have same return types"
            ));
        }
        
        Ok(())
    }
    
    /// Check if tail call optimization can be applied
    ///
    /// This checks various conditions that might prevent tail call optimization
    pub fn can_optimize_tail_call(
        has_try_catch_blocks: bool,
        in_multivalue_block: bool,
    ) -> bool {
        // Tail calls cannot be optimized if:
        // 1. We're inside a try-catch block (exception handling)
        // 2. We're in a block that expects multiple values
        !has_try_catch_blocks && !in_multivalue_block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_foundation::types::{ValueType, Limits};
    
    #[test]
    fn test_tail_call_validation() {
        // Test compatible types
        let func1 = FuncType {
            params: vec![ValueType::I32, ValueType::I32],
            results: vec![ValueType::I32],
        };
        
        let func2 = FuncType {
            params: vec![ValueType::I32],
            results: vec![ValueType::I32],
        };
        
        // Should succeed - same return types
        assert!(validation::validate_tail_call(&func1, &func2).is_ok());
        
        // Test incompatible return types
        let func3 = FuncType {
            params: vec![ValueType::I32],
            results: vec![ValueType::I64],
        };
        
        // Should fail - different return types
        assert!(validation::validate_tail_call(&func1, &func3).is_err());
    }
    
    #[test]
    fn test_can_optimize_tail_call() {
        // Normal case - should be optimizable
        assert!(validation::can_optimize_tail_call(false, false));
        
        // Inside try-catch - not optimizable
        assert!(!validation::can_optimize_tail_call(true, false));
        
        // In multivalue block - not optimizable
        assert!(!validation::can_optimize_tail_call(false, true));
    }
}