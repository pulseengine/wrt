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

    /// Add two 32-bit integers
    I32Add,

    /// Subtract one 32-bit integer from another
    I32Sub,

    /// Multiply two 32-bit integers
    I32Mul,

    /// Divide two 32-bit integers (signed)
    I32DivS,

    /// Divide two 32-bit integers (unsigned)
    I32DivU,

    /// Remainder of dividing two 32-bit integers (signed)
    I32RemS,

    /// Remainder of dividing two 32-bit integers (unsigned)
    I32RemU,

    /// Bitwise AND of two 32-bit integers
    I32And,

    /// Bitwise OR of two 32-bit integers
    I32Or,

    /// Bitwise XOR of two 32-bit integers
    I32Xor,

    /// Shift a 32-bit integer left
    I32Shl,

    /// Shift a 32-bit integer right (signed)
    I32ShrS,

    /// Shift a 32-bit integer right (unsigned)
    I32ShrU,

    /// Rotate a 32-bit integer left
    I32Rotl,

    /// Rotate a 32-bit integer right
    I32Rotr,

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

    /// Add two 64-bit integers
    I64Add,

    /// Subtract one 64-bit integer from another
    I64Sub,

    /// Multiply two 64-bit integers
    I64Mul,

    /// Divide two 64-bit integers (signed)
    I64DivS,

    /// Divide two 64-bit integers (unsigned)
    I64DivU,

    /// Remainder of dividing two 64-bit integers (signed)
    I64RemS,

    /// Remainder of dividing two 64-bit integers (unsigned)
    I64RemU,

    /// Bitwise AND of two 64-bit integers
    I64And,

    /// Bitwise OR of two 64-bit integers
    I64Or,

    /// Bitwise XOR of two 64-bit integers
    I64Xor,

    /// Shift a 64-bit integer left
    I64Shl,

    /// Shift a 64-bit integer right (signed)
    I64ShrS,

    /// Shift a 64-bit integer right (unsigned)
    I64ShrU,

    /// Rotate a 64-bit integer left
    I64Rotl,

    /// Rotate a 64-bit integer right
    I64Rotr,

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

    /// F32 absolute value
    F32Abs,

    /// F32 negate
    F32Neg,

    /// F32 ceiling operation
    F32Ceil,

    /// F32 floor operation
    F32Floor,

    /// F32 truncate operation
    F32Trunc,

    /// F32 nearest integer operation
    F32Nearest,

    /// F32 square root
    F32Sqrt,

    /// F32 addition
    F32Add,

    /// F32 subtraction
    F32Sub,

    /// F32 multiplication
    F32Mul,

    /// F32 division
    F32Div,

    /// F32 minimum
    F32Min,

    /// F32 maximum
    F32Max,

    /// F32 copysign
    F32Copysign,

    /// F64 absolute value
    F64Abs,

    /// F64 negate
    F64Neg,

    /// F64 ceiling operation
    F64Ceil,

    /// F64 floor operation
    F64Floor,

    /// F64 truncate operation
    F64Trunc,

    /// F64 nearest integer operation
    F64Nearest,

    /// F64 square root
    F64Sqrt,

    /// F64 addition
    F64Add,

    /// F64 subtraction
    F64Sub,

    /// F64 multiplication
    F64Mul,

    /// F64 division
    F64Div,

    /// F64 minimum
    F64Min,

    /// F64 maximum
    F64Max,

    /// F64 copysign
    F64Copysign,

    /// Convert i64 to i32 by wrapping
    I32WrapI64,

    /// Convert f32 to i32 (signed)
    I32TruncF32S,

    /// Convert f32 to i32 (unsigned)
    I32TruncF32U,

    /// Convert f64 to i32 (signed)
    I32TruncF64S,

    /// Convert f64 to i32 (unsigned)
    I32TruncF64U,

    /// Extend i32 to i64 (signed)
    I64ExtendI32S,

    /// Extend i32 to i64 (unsigned)
    I64ExtendI32U,

    /// Convert f32 to i64 (signed)
    I64TruncF32S,

    /// Convert f32 to i64 (unsigned)
    I64TruncF32U,

    /// Convert f64 to i64 (signed)
    I64TruncF64S,

    /// Convert f64 to i64 (unsigned)
    I64TruncF64U,

    /// Convert i32 to f32 (signed)
    F32ConvertI32S,

    /// Convert i32 to f32 (unsigned)
    F32ConvertI32U,

    /// Convert i64 to f32 (signed)
    F32ConvertI64S,

    /// Convert i64 to f32 (unsigned)
    F32ConvertI64U,

    /// Demote f64 to f32
    F32DemoteF64,

    /// Convert i32 to f64 (signed)
    F64ConvertI32S,

    /// Convert i32 to f64 (unsigned)
    F64ConvertI32U,

    /// Convert i64 to f64 (signed)
    F64ConvertI64S,

    /// Convert i64 to f64 (unsigned)
    F64ConvertI64U,

    /// Promote f32 to f64
    F64PromoteF32,

    /// Reinterpret f32 as i32
    I32ReinterpretF32,

    /// Reinterpret f64 as i64
    I64ReinterpretF64,

    /// Reinterpret i32 as f32
    F32ReinterpretI32,

    /// Reinterpret i64 as f64
    F64ReinterpretI64,

    /// Count leading zeros in an i32
    I32Clz,

    /// Count trailing zeros in an i32
    I32Ctz,

    /// Count number of bits set to 1 in an i32
    I32Popcnt,

    /// Count leading zeros in an i64
    I64Clz,

    /// Count trailing zeros in an i64
    I64Ctz,

    /// Count number of bits set to 1 in an i64
    I64Popcnt,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_variants() {
        // Control instructions
        let block = Instruction::Block(BlockType::Empty);
        let loop_instr = Instruction::Loop(BlockType::Empty);
        let if_instr = Instruction::If(BlockType::Empty);
        let br = Instruction::Br(1);
        let br_if = Instruction::BrIf(2);
        let br_table = Instruction::BrTable(vec![1, 2, 3], 0);
        let call = Instruction::Call(0);
        let call_indirect = Instruction::CallIndirect(1, 0);

        // Variable instructions
        let local_get = Instruction::LocalGet(0);
        let local_set = Instruction::LocalSet(1);
        let local_tee = Instruction::LocalTee(2);
        let global_get = Instruction::GlobalGet(0);
        let global_set = Instruction::GlobalSet(1);

        // Memory instructions
        let i32_load = Instruction::I32Load(0, 0);
        let i64_load = Instruction::I64Load(0, 0);
        let i32_store = Instruction::I32Store(0, 0);
        let i64_store = Instruction::I64Store(0, 0);

        // Numeric instructions
        let i32_const = Instruction::I32Const(42);
        let i64_const = Instruction::I64Const(42);
        let f32_const = Instruction::F32Const(42.0);
        let f64_const = Instruction::F64Const(42.0);

        // Unit variants (no parameters)
        let memory_size = Instruction::MemorySize;
        let memory_grow = Instruction::MemoryGrow;
        let i32_add = Instruction::I32Add;
        let i64_add = Instruction::I64Add;
        let unreachable = Instruction::Unreachable;
        let nop = Instruction::Nop;

        // Verify instructions can be constructed and compared
        assert!(matches!(block, Instruction::Block(_)));
        assert!(matches!(loop_instr, Instruction::Loop(_)));
        assert!(matches!(if_instr, Instruction::If(_)));
        assert!(matches!(br, Instruction::Br(_)));
        assert!(matches!(br_if, Instruction::BrIf(_)));
        assert!(matches!(br_table, Instruction::BrTable(_, _)));
        assert!(matches!(call, Instruction::Call(_)));
        assert!(matches!(call_indirect, Instruction::CallIndirect(_, _)));

        assert!(matches!(local_get, Instruction::LocalGet(_)));
        assert!(matches!(local_set, Instruction::LocalSet(_)));
        assert!(matches!(local_tee, Instruction::LocalTee(_)));
        assert!(matches!(global_get, Instruction::GlobalGet(_)));
        assert!(matches!(global_set, Instruction::GlobalSet(_)));

        assert!(matches!(i32_load, Instruction::I32Load(_, _)));
        assert!(matches!(i64_load, Instruction::I64Load(_, _)));
        assert!(matches!(i32_store, Instruction::I32Store(_, _)));
        assert!(matches!(i64_store, Instruction::I64Store(_, _)));

        assert!(matches!(i32_const, Instruction::I32Const(_)));
        assert!(matches!(i64_const, Instruction::I64Const(_)));
        assert!(matches!(f32_const, Instruction::F32Const(_)));
        assert!(matches!(f64_const, Instruction::F64Const(_)));

        // Unit variants should match exactly
        assert!(matches!(memory_size, Instruction::MemorySize));
        assert!(matches!(memory_grow, Instruction::MemoryGrow));
        assert!(matches!(i32_add, Instruction::I32Add));
        assert!(matches!(i64_add, Instruction::I64Add));
        assert!(matches!(unreachable, Instruction::Unreachable));
        assert!(matches!(nop, Instruction::Nop));
    }

    #[test]
    fn test_block_types() {
        let empty = BlockType::Empty;
        let i32_type = BlockType::Type(ValueType::I32);
        let i64_type = BlockType::Type(ValueType::I64);
        let f32_type = BlockType::Type(ValueType::F32);
        let f64_type = BlockType::Type(ValueType::F64);
        let type_index = BlockType::TypeIndex(42);

        // Verify block types can be constructed and compared
        assert!(matches!(empty, BlockType::Empty));
        assert!(matches!(i32_type, BlockType::Type(ValueType::I32)));
        assert!(matches!(i64_type, BlockType::Type(ValueType::I64)));
        assert!(matches!(f32_type, BlockType::Type(ValueType::F32)));
        assert!(matches!(f64_type, BlockType::Type(ValueType::F64)));
        assert!(matches!(type_index, BlockType::TypeIndex(42)));
    }
}
