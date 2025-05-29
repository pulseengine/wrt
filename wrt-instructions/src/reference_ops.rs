//! WebAssembly reference type operations implementation.
//!
//! This module implements WebAssembly reference type instructions including:
//! - ref.null: Create a null reference
//! - ref.is_null: Test if a reference is null
//! - ref.func: Create a function reference
//! - ref.as_non_null: Assert reference is not null
//!
//! These operations support the WebAssembly reference types proposal
//! and work across std, no_std+alloc, and pure no_std environments.

use crate::prelude::*;
use wrt_error::{Error, Result};
use wrt_foundation::{
    types::{RefType, ValueType},
    values::{Value, FuncRef, ExternRef},
};
use crate::validation::{Validate, ValidationContext, validate_ref_op};

/// Reference null operation - creates a null reference of specified type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefNull {
    /// Type of the null reference to create
    pub ref_type: RefType,
}

impl RefNull {
    /// Create a new ref.null instruction
    pub fn new(ref_type: RefType) -> Self {
        Self { ref_type }
    }

    /// Execute the ref.null instruction
    pub fn execute(&self) -> Result<Value> {
        match self.ref_type {
            RefType::Funcref => Ok(Value::FuncRef(None)),
            RefType::Externref => Ok(Value::ExternRef(None)),
        }
    }
}

/// Reference is null operation - tests if a reference is null
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefIsNull;

impl RefIsNull {
    /// Create a new ref.is_null instruction
    pub fn new() -> Self {
        Self
    }

    /// Execute the ref.is_null instruction
    pub fn execute(&self, reference: Value) -> Result<Value> {
        let is_null = match reference {
            Value::FuncRef(None) => true,
            Value::ExternRef(None) => true,
            Value::FuncRef(Some(_)) => false,
            Value::ExternRef(Some(_)) => false,
            _ => {
                return Err(Error::type_error(
                    "ref.is_null requires a reference type"
                ));
            }
        };
        Ok(Value::I32(if is_null { 1 } else { 0 }))
    }
}

/// Reference function operation - creates a function reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefFunc {
    /// Function index to create reference for
    pub function_index: u32,
}

impl RefFunc {
    /// Create a new ref.func instruction
    pub fn new(function_index: u32) -> Self {
        Self { function_index }
    }

    /// Execute the ref.func instruction
    /// Note: In a real implementation, this would validate the function index
    /// against the module's function table and create an actual function reference
    pub fn execute(&self) -> Result<Value> {
        // In a full implementation, this would:
        // 1. Validate that function_index exists in the module
        // 2. Create a proper function reference with the function's type signature
        // For now, we create a basic function reference
        Ok(Value::FuncRef(Some(FuncRef { index: self.function_index })))
    }
}

/// Reference as non-null operation - asserts reference is not null
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefAsNonNull;

impl RefAsNonNull {
    /// Create a new ref.as_non_null instruction
    pub fn new() -> Self {
        Self
    }

    /// Execute the ref.as_non_null instruction
    pub fn execute(&self, reference: Value) -> Result<Value> {
        match reference {
            Value::FuncRef(None) | Value::ExternRef(None) => {
                Err(Error::runtime_error("null reference"))
            }
            Value::FuncRef(Some(_)) | Value::ExternRef(Some(_)) => Ok(reference),
            _ => Err(Error::type_error(
                "ref.as_non_null requires a reference type"
            )),
        }
    }
}

/// Trait for reference type operations that can be implemented by execution contexts
pub trait ReferenceOperations {
    /// Get a function by its index for ref.func operations
    fn get_function(&self, function_index: u32) -> Result<Option<u32>>;
    
    /// Validate that a function index exists
    fn validate_function_index(&self, function_index: u32) -> Result<()>;
}

/// Reference operation enum for unified handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceOp {
    /// ref.null operation
    RefNull(RefNull),
    /// ref.is_null operation  
    RefIsNull(RefIsNull),
    /// ref.func operation
    RefFunc(RefFunc),
    /// ref.as_non_null operation
    RefAsNonNull(RefAsNonNull),
}

impl ReferenceOp {
    /// Execute the reference operation with the given context and stack values
    pub fn execute<C: ReferenceOperations>(
        &self,
        context: &C,
        operands: &[Value],
    ) -> Result<Value> {
        match self {
            ReferenceOp::RefNull(op) => op.execute(),
            ReferenceOp::RefIsNull(op) => {
                if operands.is_empty() {
                    return Err(Error::runtime_error("ref.is_null requires one operand"));
                }
                op.execute(operands[0].clone())
            }
            ReferenceOp::RefFunc(op) => {
                // Validate function exists
                context.validate_function_index(op.function_index)?;
                op.execute()
            }
            ReferenceOp::RefAsNonNull(op) => {
                if operands.is_empty() {
                    return Err(Error::runtime_error("ref.as_non_null requires one operand"));
                }
                op.execute(operands[0].clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockReferenceContext;

    impl ReferenceOperations for MockReferenceContext {
        fn get_function(&self, function_index: u32) -> Result<Option<u32>> {
            // Mock: functions 0-9 exist
            if function_index < 10 {
                Ok(Some(function_index))
            } else {
                Ok(None)
            }
        }

        fn validate_function_index(&self, function_index: u32) -> Result<()> {
            if function_index < 10 {
                Ok(())
            } else {
                Err(Error::function_error("Function index out of bounds"))
            }
        }
    }

    #[test]
    fn test_ref_null_funcref() {
        let op = RefNull::new(RefType::Funcref);
        let result = op.execute().unwrap();
        assert_eq!(result, Value::FuncRef(None));
    }

    #[test]
    fn test_ref_null_externref() {
        let op = RefNull::new(RefType::Externref);
        let result = op.execute().unwrap();
        assert_eq!(result, Value::ExternRef(None));
    }

    #[test]
    fn test_ref_is_null_with_null_funcref() {
        let op = RefIsNull::new();
        let result = op.execute(Value::FuncRef(None)).unwrap();
        assert_eq!(result, Value::I32(1));
    }

    #[test]
    fn test_ref_is_null_with_non_null_funcref() {
        let op = RefIsNull::new();
        let result = op.execute(Value::FuncRef(Some(FuncRef { index: 42 }))).unwrap();
        assert_eq!(result, Value::I32(0));
    }

    #[test]
    fn test_ref_is_null_with_null_externref() {
        let op = RefIsNull::new();
        let result = op.execute(Value::ExternRef(None)).unwrap();
        assert_eq!(result, Value::I32(1));
    }

    #[test]
    fn test_ref_is_null_with_non_null_externref() {
        let op = RefIsNull::new();
        let result = op.execute(Value::ExternRef(Some(ExternRef { handle: 123 }))).unwrap();
        assert_eq!(result, Value::I32(0));
    }

    #[test]
    fn test_ref_is_null_with_non_reference() {
        let op = RefIsNull::new();
        let result = op.execute(Value::I32(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_ref_func_valid_index() {
        let op = RefFunc::new(5);
        let result = op.execute().unwrap();
        assert_eq!(result, Value::FuncRef(Some(FuncRef { index: 5 })));
    }

    #[test]
    fn test_ref_as_non_null_with_valid_funcref() {
        let op = RefAsNonNull::new();
        let input = Value::FuncRef(Some(FuncRef { index: 42 }));
        let result = op.execute(input.clone()).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_ref_as_non_null_with_null_funcref() {
        let op = RefAsNonNull::new();
        let result = op.execute(Value::FuncRef(None));
        assert!(result.is_err());
    }

    #[test]
    fn test_ref_as_non_null_with_valid_externref() {
        let op = RefAsNonNull::new();
        let input = Value::ExternRef(Some(ExternRef { handle: 123 }));
        let result = op.execute(input.clone()).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_ref_as_non_null_with_null_externref() {
        let op = RefAsNonNull::new();
        let result = op.execute(Value::ExternRef(None));
        assert!(result.is_err());
    }

    #[test]
    fn test_ref_as_non_null_with_non_reference() {
        let op = RefAsNonNull::new();
        let result = op.execute(Value::I32(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_reference_op_enum() {
        let context = MockReferenceContext;

        // Test RefNull
        let ref_null_op = ReferenceOp::RefNull(RefNull::new(RefType::Funcref));
        let result = ref_null_op.execute(&context, &[]).unwrap();
        assert_eq!(result, Value::FuncRef(None));

        // Test RefIsNull
        let ref_is_null_op = ReferenceOp::RefIsNull(RefIsNull::new());
        let result = ref_is_null_op.execute(&context, &[Value::FuncRef(None)]).unwrap();
        assert_eq!(result, Value::I32(1));

        // Test RefFunc with valid index
        let ref_func_op = ReferenceOp::RefFunc(RefFunc::new(3));
        let result = ref_func_op.execute(&context, &[]).unwrap();
        assert_eq!(result, Value::FuncRef(Some(FuncRef { index: 3 })));

        // Test RefFunc with invalid index
        let ref_func_op = ReferenceOp::RefFunc(RefFunc::new(15));
        let result = ref_func_op.execute(&context, &[]);
        assert!(result.is_err());

        // Test RefAsNonNull
        let ref_as_non_null_op = ReferenceOp::RefAsNonNull(RefAsNonNull::new());
        let result = ref_as_non_null_op.execute(&context, &[Value::FuncRef(Some(FuncRef { index: 5 }))]).unwrap();
        assert_eq!(result, Value::FuncRef(Some(FuncRef { index: 5 })));
    }
}

// Validation implementations

impl Validate for RefNull {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<()> {
        let ref_type = match self.ref_type {
            RefType::Funcref => ValueType::FuncRef,
            RefType::Externref => ValueType::ExternRef,
        };
        validate_ref_op("ref.null", Some(ref_type), ctx)
    }
}

impl Validate for RefIsNull {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<()> {
        validate_ref_op("ref.is_null", None, ctx)
    }
}

impl Validate for RefFunc {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<()> {
        validate_ref_op("ref.func", None, ctx)
    }
}

impl Validate for RefAsNonNull {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<()> {
        // ref.as_non_null: [ref] -> [ref]
        if !ctx.is_unreachable() {
            let ref_type = ctx.pop_type()?;
            match ref_type {
                ValueType::FuncRef | ValueType::ExternRef => {
                    ctx.push_type(ref_type)?;
                }
                _ => return Err(Error::type_error("ref.as_non_null expects reference type")),
            }
        }
        Ok(())
    }
}

impl Validate for ReferenceOp {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<()> {
        match self {
            ReferenceOp::RefNull(op) => op.validate(ctx),
            ReferenceOp::RefIsNull(op) => op.validate(ctx),
            ReferenceOp::RefFunc(op) => op.validate(ctx),
            ReferenceOp::RefAsNonNull(op) => op.validate(ctx),
        }
    }
}