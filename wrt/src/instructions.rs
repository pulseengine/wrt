use crate::types::ValueType;
use crate::Vec;

/// Represents a WebAssembly instruction.
///
/// This enum provides a representation of all instructions in the WebAssembly
/// specification, organized into categories: control flow, parametric, variable,
/// table, memory, and numeric instructions.
///
/// # Examples
///
/// ```
/// use wrt::Instruction;
///
/// let call_instr = Instruction::Call(0); // Call function at index 0
/// let const_instr = Instruction::I32Const(42); // Push constant 42 on the stack
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    //
    // Control instructions - control flow constructs
    //
    /// Trap immediately
    Unreachable,

    /// Do nothing
    Nop,

    /// Begin a block construct with the specified signature
    Block(BlockType),

    /// Begin a loop construct with the specified signature
    Loop(BlockType),

    /// Begin an if construct with the specified signature
    If(BlockType),

    /// Begin the else branch of an if construct
    Else,

    /// End a block, loop, if, or function body
    End,

    /// Branch to a given label
    Br(u32),

    /// Conditionally branch to a label if the top of the stack is non-zero
    BrIf(u32),

    /// Branch to a label from a table based on the value at the top of the stack
    BrTable(Vec<u32>, u32),

    /// Return from the current function
    Return,

    /// Call a function by its index
    Call(u32),

    /// Call a function indirectly through a table
    /// First parameter is the type index, second is the table index
    CallIndirect(u32, u32),

    /// Tail call optimization version of Call
    ReturnCall(u32),

    /// Tail call optimization version of CallIndirect
    /// First parameter is the type index, second is the table index
    ReturnCallIndirect(u32, u32),

    //
    // Parametric instructions - operand stack manipulation
    //
    /// Drop the top value from the stack
    Drop,

    /// Select one of two values based on a condition
    Select,

    /// Select one of two values based on a condition, with explicit type
    SelectTyped(ValueType),

    //
    // Variable instructions - access local or global variables
    //
    /// Get the value of a local variable
    LocalGet(u32),

    /// Set the value of a local variable
    LocalSet(u32),

    /// Set the value of a local variable and keep the value on the stack
    LocalTee(u32),

    /// Get the value of a global variable
    GlobalGet(u32),

    /// Set the value of a global variable
    GlobalSet(u32),

    //
    // Table instructions - operate on tables
    //
    /// Get an element from a table
    TableGet(u32),

    /// Set an element in a table
    TableSet(u32),

    /// Get the current size of a table
    TableSize(u32),

    /// Grow a table by a number of elements
    TableGrow(u32),

    /// Fill a range of a table with a value
    TableFill(u32),

    /// Copy elements from one table to another
    /// First parameter is the destination table index, second is the source table index
    TableCopy(u32, u32),

    /// Initialize a table from an element segment
    /// First parameter is the table index, second is the element index
    TableInit(u32, u32),

    /// Drop an element segment
    ElemDrop(u32),

    //
    // Memory instructions - operate on linear memory
    //
    /// Load a 32-bit integer from memory
    /// First parameter is alignment, second is offset
    I32Load(u32, u32),

    /// Load a 64-bit integer from memory
    /// First parameter is alignment, second is offset
    I64Load(u32, u32),

    /// Load a 32-bit float from memory
    /// First parameter is alignment, second is offset
    F32Load(u32, u32),

    /// Load a 64-bit float from memory
    /// First parameter is alignment, second is offset
    F64Load(u32, u32),

    /// Load an 8-bit integer from memory and sign-extend to a 32-bit integer
    /// First parameter is alignment, second is offset
    I32Load8S(u32, u32),

    /// Load an 8-bit integer from memory and zero-extend to a 32-bit integer
    /// First parameter is alignment, second is offset
    I32Load8U(u32, u32),

    /// Load a 16-bit integer from memory and sign-extend to a 32-bit integer
    /// First parameter is alignment, second is offset
    I32Load16S(u32, u32),

    /// Load a 16-bit integer from memory and zero-extend to a 32-bit integer
    /// First parameter is alignment, second is offset
    I32Load16U(u32, u32),

    /// Load an 8-bit integer from memory and sign-extend to a 64-bit integer
    /// First parameter is alignment, second is offset
    I64Load8S(u32, u32),

    /// Load an 8-bit integer from memory and zero-extend to a 64-bit integer
    /// First parameter is alignment, second is offset
    I64Load8U(u32, u32),

    /// Load a 16-bit integer from memory and sign-extend to a 64-bit integer
    /// First parameter is alignment, second is offset
    I64Load16S(u32, u32),

    /// Load a 16-bit integer from memory and zero-extend to a 64-bit integer
    /// First parameter is alignment, second is offset
    I64Load16U(u32, u32),

    /// Load a 32-bit integer from memory and sign-extend to a 64-bit integer
    /// First parameter is alignment, second is offset
    I64Load32S(u32, u32),

    /// Load a 32-bit integer from memory and zero-extend to a 64-bit integer
    /// First parameter is alignment, second is offset
    I64Load32U(u32, u32),

    /// Store a 32-bit integer to memory
    /// First parameter is alignment, second is offset
    I32Store(u32, u32),

    /// Store a 64-bit integer to memory
    /// First parameter is alignment, second is offset
    I64Store(u32, u32),

    /// Store a 32-bit float to memory
    /// First parameter is alignment, second is offset
    F32Store(u32, u32),

    /// Store a 64-bit float to memory
    /// First parameter is alignment, second is offset
    F64Store(u32, u32),

    /// Store the low 8 bits of a 32-bit integer to memory
    /// First parameter is alignment, second is offset
    I32Store8(u32, u32),

    /// Store the low 16 bits of a 32-bit integer to memory
    /// First parameter is alignment, second is offset
    I32Store16(u32, u32),

    /// Store the low 8 bits of a 64-bit integer to memory
    /// First parameter is alignment, second is offset
    I64Store8(u32, u32),

    /// Store the low 16 bits of a 64-bit integer to memory
    /// First parameter is alignment, second is offset
    I64Store16(u32, u32),

    /// Store the low 32 bits of a 64-bit integer to memory
    /// First parameter is alignment, second is offset
    I64Store32(u32, u32),

    /// Get the current size of memory in pages
    MemorySize,

    /// Grow memory by a number of pages
    MemoryGrow,

    /// Fill a range of memory with a value
    MemoryFill,

    /// Copy from one region of memory to another
    MemoryCopy,

    /// Initialize a region of memory from a data segment
    MemoryInit(u32),

    /// Drop a data segment
    DataDrop(u32),

    //
    // Numeric instructions - constants and operations on numeric values
    //
    /// Push a 32-bit integer constant onto the stack
    I32Const(i32),

    /// Push a 64-bit integer constant onto the stack
    I64Const(i64),

    /// Push a 32-bit float constant onto the stack
    F32Const(f32),

    /// Push a 64-bit float constant onto the stack
    F64Const(f64),

    /// Test if a 32-bit integer is equal to zero
    I32Eqz,

    /// Test if two 32-bit integers are equal
    I32Eq,

    /// Test if two 32-bit integers are not equal
    I32Ne,

    /// Test if one 32-bit integer is less than another (signed)
    I32LtS,

    /// Test if one 32-bit integer is less than another (unsigned)
    I32LtU,

    /// Test if one 32-bit integer is greater than another (signed)
    I32GtS,

    /// Test if one 32-bit integer is greater than another (unsigned)
    I32GtU,

    /// Test if one 32-bit integer is less than or equal to another (signed)
    I32LeS,

    /// Test if one 32-bit integer is less than or equal to another (unsigned)
    I32LeU,

    /// Test if one 32-bit integer is greater than or equal to another (signed)
    I32GeS,

    /// Test if one 32-bit integer is greater than or equal to another (unsigned)
    I32GeU,

    /// Test if a 64-bit integer is equal to zero
    I64Eqz,

    /// Test if two 64-bit integers are equal
    I64Eq,

    /// Test if two 64-bit integers are not equal
    I64Ne,

    /// Test if one 64-bit integer is less than another (signed)
    I64LtS,

    /// Test if one 64-bit integer is less than another (unsigned)
    I64LtU,

    /// Test if one 64-bit integer is greater than another (signed)
    I64GtS,

    /// Test if one 64-bit integer is greater than another (unsigned)
    I64GtU,

    /// Test if one 64-bit integer is less than or equal to another (signed)
    I64LeS,

    /// Test if one 64-bit integer is less than or equal to another (unsigned)
    I64LeU,

    /// Test if one 64-bit integer is greater than or equal to another (signed)
    I64GeS,

    /// Test if one 64-bit integer is greater than or equal to another (unsigned)
    I64GeU,

    /// Test if two 32-bit floats are equal
    F32Eq,

    /// Test if two 32-bit floats are not equal
    F32Ne,

    /// Test if one 32-bit float is less than another
    F32Lt,

    /// Test if one 32-bit float is greater than another
    F32Gt,

    /// Test if one 32-bit float is less than or equal to another
    F32Le,

    /// Test if one 32-bit float is greater than or equal to another
    F32Ge,

    /// Test if two 64-bit floats are equal
    F64Eq,

    /// Test if two 64-bit floats are not equal
    F64Ne,

    /// Test if one 64-bit float is less than another
    F64Lt,

    /// Test if one 64-bit float is greater than another
    F64Gt,

    /// Test if one 64-bit float is less than or equal to another
    F64Le,

    /// Test if one 64-bit float is greater than or equal to another
    F64Ge,
}

/// Represents the type signature of a WebAssembly block structure.
///
/// Block structures in WebAssembly (block, loop, if) can have type signatures
/// that define their parameters and return values. This enum represents the
/// possible forms of these signatures.
///
/// # Examples
///
/// ```
/// use wrt::{Instruction, BlockType, ValueType};
///
/// // A block that returns nothing
/// let empty_block = Instruction::Block(BlockType::Empty);
///
/// // A block that returns an i32
/// let typed_block = Instruction::Block(BlockType::Type(ValueType::I32));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    /// Block has no parameters and returns nothing
    Empty,

    /// Block has a single return type
    Type(ValueType),

    /// Block references a function type by index
    /// This allows for blocks with multiple parameters and/or return values
    TypeIndex(u32),
}
