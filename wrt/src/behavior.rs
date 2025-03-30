use crate::{
    error::{Error, Result},
    global::Global,
    memory::Memory,
    stack::Stack,
    table::Table,
    types::BlockType,
    values::Value,
    Vec,
};

/// Behavior for stack operations
pub trait StackBehavior {
    fn push(&mut self, value: Value) -> Result<()>;
    fn pop(&mut self) -> Result<Value>;
    fn pop_bool(&mut self) -> Result<bool> {
        match self.pop()? {
            Value::I32(0) => Ok(false),
            Value::I32(_) => Ok(true),
            _ => Err(Error::InvalidType("Expected i32 for boolean".to_string())),
        }
    }
    fn pop_i32(&mut self) -> Result<i32> {
        match self.pop()? {
            Value::I32(v) => Ok(v),
            _ => Err(Error::InvalidType("Expected i32".to_string())),
        }
    }
    fn peek(&self) -> Result<&Value>;
    fn peek_mut(&mut self) -> Result<&mut Value>;
    fn values(&self) -> &[Value];
    fn values_mut(&mut self) -> &mut [Value];
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn push_label(&mut self, arity: usize, pc: usize);
    fn pop_label(&mut self) -> Result<Label>;
    fn get_label(&self, index: usize) -> Option<&Label>;
}

/// Trait for accessing the frame state
pub trait FrameBehavior: ControlFlowBehavior {
    /// Get a mutable reference to the locals
    fn locals(&mut self) -> &mut Vec<Value>;

    /// Get a local by index
    fn get_local(&self, idx: usize) -> Result<Value>;

    /// Allow downcasting to concrete type
    fn as_any(&mut self) -> &mut dyn std::any::Any;

    /// Set a local by index
    fn set_local(&mut self, idx: usize, value: Value) -> Result<()>;

    /// Get a global by index
    fn get_global(&self, idx: usize) -> Result<Value>;

    /// Set a global by index
    fn set_global(&mut self, idx: usize, value: Value) -> Result<()>;

    /// Get a memory by index
    fn get_memory(&self, idx: usize) -> Result<&Memory>;

    /// Get a mutable memory by index
    fn get_memory_mut(&mut self, idx: usize) -> Result<&mut Memory>;

    /// Get a table by index
    fn get_table(&self, idx: usize) -> Result<&Table>;

    /// Get a mutable table by index
    fn get_table_mut(&mut self, idx: usize) -> Result<&mut Table>;

    /// Get a mutable global by index
    fn get_global_mut(&mut self, idx: usize) -> Option<&mut Global>;

    /// Get the current program counter
    fn pc(&self) -> usize;

    /// Set the program counter
    fn set_pc(&mut self, pc: usize);

    /// Get the function index
    fn func_idx(&self) -> u32;

    /// Get the instance index
    fn instance_idx(&self) -> usize;

    /// Get the number of locals
    fn locals_len(&self) -> usize;

    /// Get the label stack
    fn label_stack(&mut self) -> &mut Vec<Label>;

    /// Get the frame arity
    fn arity(&self) -> usize;
    /// Set the frame arity
    fn set_arity(&mut self, arity: usize);

    /// Get the label arity
    fn label_arity(&self) -> usize;

    /// Get the return program counter
    fn return_pc(&self) -> usize;

    /// Set the return program counter
    fn set_return_pc(&mut self, pc: usize);

    /// Load i32 value from memory
    fn load_i32(&mut self, addr: usize, align: u32) -> Result<i32>;

    /// Load i64 value from memory
    fn load_i64(&mut self, addr: usize, align: u32) -> Result<i64>;

    /// Load f32 value from memory
    fn load_f32(&mut self, addr: usize, align: u32) -> Result<f32>;

    /// Load f64 value from memory
    fn load_f64(&mut self, addr: usize, align: u32) -> Result<f64>;

    /// Load i8 value from memory
    fn load_i8(&mut self, addr: usize, align: u32) -> Result<i8>;

    /// Load u8 value from memory
    fn load_u8(&mut self, addr: usize, align: u32) -> Result<u8>;

    /// Load i16 value from memory
    fn load_i16(&mut self, addr: usize, align: u32) -> Result<i16>;

    /// Load u16 value from memory
    fn load_u16(&mut self, addr: usize, align: u32) -> Result<u16>;

    /// Store i32 value to memory
    fn store_i32(&mut self, addr: usize, align: u32, value: i32) -> Result<()>;

    /// Store i64 value to memory
    fn store_i64(&mut self, addr: usize, align: u32, value: i64) -> Result<()>;

    /// Get memory size
    fn memory_size(&mut self) -> Result<u32>;

    /// Grow memory
    fn memory_grow(&mut self, pages: u32) -> Result<u32>;

    /// Get table value
    fn table_get(&mut self, table_idx: u32, idx: u32) -> Result<Value>;

    /// Set table value
    fn table_set(&mut self, table_idx: u32, idx: u32, value: Value) -> Result<()>;

    /// Get table size
    fn table_size(&mut self, table_idx: u32) -> Result<u32>;

    /// Grow table
    fn table_grow(&mut self, table_idx: u32, delta: u32, value: Value) -> Result<u32>;

    /// Initialize table
    fn table_init(
        &mut self,
        table_idx: u32,
        elem_idx: u32,
        dst: u32,
        src: u32,
        n: u32,
    ) -> Result<()>;

    /// Copy table
    fn table_copy(
        &mut self,
        dst_table: u32,
        src_table: u32,
        dst: u32,
        src: u32,
        n: u32,
    ) -> Result<()>;

    /// Drop element
    fn elem_drop(&mut self, elem_idx: u32) -> Result<()>;

    /// Fill table
    fn table_fill(&mut self, table_idx: u32, dst: u32, val: Value, n: u32) -> Result<()>;

    /// Pop a boolean value from the stack
    fn pop_bool(&mut self, stack: &mut dyn Stack) -> Result<bool>;

    /// Pop an i32 value from the stack
    fn pop_i32(&mut self, stack: &mut dyn Stack) -> Result<i32>;

    /// Get all locals for debugging purposes
    fn get_locals(&self) -> &[Value] {
        &[] // Default implementation returns empty slice
    }
}

/// Trait for control flow operations that require stack access
pub trait ControlFlowBehavior {
    fn enter_block(&mut self, ty: BlockType, stack_len: usize) -> Result<()>;
    fn enter_loop(&mut self, ty: BlockType, stack_len: usize) -> Result<()>;
    fn enter_if(&mut self, ty: BlockType, stack_len: usize, condition: bool) -> Result<()>;
    fn enter_else(&mut self, stack_len: usize) -> Result<()>;
    fn exit_block(&mut self, stack: &mut dyn Stack) -> Result<()>;
    fn branch(&mut self, label_idx: u32, stack: &mut dyn Stack) -> Result<()>;
    fn return_(&mut self, stack: &mut dyn Stack) -> Result<()>;
    fn call(&mut self, func_idx: u32, stack: &mut dyn Stack) -> Result<()>;
    fn call_indirect(
        &mut self,
        type_idx: u32,
        table_idx: u32,
        entry: u32,
        stack: &mut dyn Stack,
    ) -> Result<()>;
    fn set_label_arity(&mut self, arity: usize);
}

/// Trait for executing WebAssembly instructions
pub trait InstructionExecutor: std::fmt::Debug {
    /// Execute the instruction in the given context
    ///
    /// # Arguments
    /// * `stack` - The execution stack
    /// * `frame` - The current execution frame
    ///
    /// # Returns
    /// * `Ok(())` - If the instruction executed successfully
    /// * `Err(Error)` - If an error occurred
    fn execute(&self, stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()>;
}

/// Label for branching control flow
#[derive(Debug, Clone)]
pub struct Label {
    pub arity: usize,
    pub pc: usize,
    pub continuation: usize,
}

impl From<crate::stack::Label> for Label {
    fn from(label: crate::stack::Label) -> Self {
        Self {
            arity: label.arity,
            pc: label.pc,
            continuation: label.continuation,
        }
    }
}

/// A behavior that does nothing and returns default values
#[derive(Debug, Default)]
pub struct NullBehavior {
    pub locals: Vec<Value>,
    pub pc: usize,
    pub func_idx: u32,
    pub arity: usize,
    pub label_arity: usize,
    pub return_pc: usize,
    pub label_stack: Vec<Label>,
}

impl FrameBehavior for NullBehavior {
    fn locals(&mut self) -> &mut Vec<Value> {
        &mut self.locals
    }

    fn get_local(&self, idx: usize) -> Result<Value> {
        self.locals
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::InvalidLocal(format!("Local index out of bounds: {idx}")))
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn set_local(&mut self, idx: usize, value: Value) -> Result<()> {
        if idx < self.locals.len() {
            self.locals[idx] = value;
            Ok(())
        } else {
            Err(Error::InvalidLocal(format!(
                "Local index out of bounds: {idx}"
            )))
        }
    }

    fn get_global(&self, _idx: usize) -> Result<Value> {
        // Return a placeholder value
        Ok(Value::I32(0))
    }

    fn set_global(&mut self, _idx: usize, _value: Value) -> Result<()> {
        Ok(())
    }

    fn get_memory(&self, _idx: usize) -> Result<&Memory> {
        Err(Error::InvalidMemoryIndex(_idx))
    }

    fn get_memory_mut(&mut self, _idx: usize) -> Result<&mut Memory> {
        Err(Error::InvalidMemoryIndex(_idx))
    }

    fn get_table(&self, _idx: usize) -> Result<&Table> {
        Err(Error::InvalidTableIndex(_idx))
    }

    fn get_table_mut(&mut self, _idx: usize) -> Result<&mut Table> {
        Err(Error::InvalidTableIndex(_idx))
    }

    fn get_global_mut(&mut self, _idx: usize) -> Option<&mut Global> {
        None
    }

    fn pc(&self) -> usize {
        self.pc
    }

    fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    fn func_idx(&self) -> u32 {
        self.func_idx
    }

    fn instance_idx(&self) -> usize {
        0
    }

    fn locals_len(&self) -> usize {
        self.locals.len()
    }

    fn label_stack(&mut self) -> &mut Vec<Label> {
        &mut self.label_stack
    }

    fn arity(&self) -> usize {
        self.arity
    }

    fn set_arity(&mut self, arity: usize) {
        self.arity = arity;
    }

    fn label_arity(&self) -> usize {
        self.label_arity
    }

    fn return_pc(&self) -> usize {
        self.return_pc
    }

    fn set_return_pc(&mut self, pc: usize) {
        self.return_pc = pc;
    }

    fn load_i32(&mut self, _addr: usize, _align: u32) -> Result<i32> {
        // Placeholder implementation
        Ok(0)
    }

    fn load_i64(&mut self, _addr: usize, _align: u32) -> Result<i64> {
        // Placeholder implementation
        Ok(0)
    }

    fn load_f32(&mut self, _addr: usize, _align: u32) -> Result<f32> {
        // Placeholder implementation
        Ok(0.0)
    }

    fn load_f64(&mut self, _addr: usize, _align: u32) -> Result<f64> {
        // Placeholder implementation
        Ok(0.0)
    }

    fn load_i8(&mut self, _addr: usize, _align: u32) -> Result<i8> {
        // Placeholder implementation
        Ok(0)
    }

    fn load_u8(&mut self, _addr: usize, _align: u32) -> Result<u8> {
        // Placeholder implementation
        Ok(0)
    }

    fn load_i16(&mut self, _addr: usize, _align: u32) -> Result<i16> {
        // Placeholder implementation
        Ok(0)
    }

    fn load_u16(&mut self, _addr: usize, _align: u32) -> Result<u16> {
        // Placeholder implementation
        Ok(0)
    }

    fn store_i32(&mut self, _addr: usize, _align: u32, _value: i32) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn store_i64(&mut self, _addr: usize, _align: u32, _value: i64) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn memory_size(&mut self) -> Result<u32> {
        // Placeholder implementation
        Ok(0)
    }

    fn memory_grow(&mut self, _pages: u32) -> Result<u32> {
        // Placeholder implementation
        Ok(0)
    }

    fn table_get(&mut self, _table_idx: u32, _idx: u32) -> Result<Value> {
        // Placeholder implementation
        Ok(Value::I32(0))
    }

    fn table_set(&mut self, _table_idx: u32, _idx: u32, _value: Value) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn table_size(&mut self, _table_idx: u32) -> Result<u32> {
        // Placeholder implementation
        Ok(0)
    }

    fn table_grow(&mut self, _table_idx: u32, _delta: u32, _value: Value) -> Result<u32> {
        // Placeholder implementation
        Ok(0)
    }

    fn table_init(
        &mut self,
        _table_idx: u32,
        _elem_idx: u32,
        _dst: u32,
        _src: u32,
        _n: u32,
    ) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn table_copy(
        &mut self,
        _dst_table: u32,
        _src_table: u32,
        _dst: u32,
        _src: u32,
        _n: u32,
    ) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn elem_drop(&mut self, _elem_idx: u32) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn table_fill(&mut self, _table_idx: u32, _dst: u32, _val: Value, _n: u32) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn pop_bool(&mut self, stack: &mut dyn Stack) -> Result<bool> {
        match stack.pop()? {
            Value::I32(0) => Ok(false),
            Value::I32(_) => Ok(true),
            _ => Err(Error::TypeMismatch(
                "Expected i32 boolean value".to_string(),
            )),
        }
    }

    fn pop_i32(&mut self, stack: &mut dyn Stack) -> Result<i32> {
        match stack.pop()? {
            Value::I32(v) => Ok(v),
            _ => Err(Error::TypeMismatch("Expected i32 value".to_string())),
        }
    }

    fn get_locals(&self) -> &[Value] {
        &[] // Default implementation returns empty slice
    }
}

impl ControlFlowBehavior for NullBehavior {
    fn enter_block(&mut self, _ty: BlockType, _stack_len: usize) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn enter_loop(&mut self, _ty: BlockType, _stack_len: usize) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn enter_if(&mut self, _ty: BlockType, _stack_len: usize, _condition: bool) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn enter_else(&mut self, _stack_len: usize) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn exit_block(&mut self, _stack: &mut dyn Stack) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn branch(&mut self, _label_idx: u32, _stack: &mut dyn Stack) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn return_(&mut self, _stack: &mut dyn Stack) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn call(&mut self, _func_idx: u32, _stack: &mut dyn Stack) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn call_indirect(
        &mut self,
        _type_idx: u32,
        _table_idx: u32,
        _entry: u32,
        _stack: &mut dyn Stack,
    ) -> Result<()> {
        // Placeholder implementation
        unimplemented!("call_indirect not implemented for Module")
    }

    fn set_label_arity(&mut self, arity: usize) {
        self.label_arity = arity;
    }
}
