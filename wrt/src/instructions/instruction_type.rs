//! WebAssembly instruction type definition

use crate::types::{BlockType, ValueType};

/// Represents a WebAssembly instruction
#[derive(Clone, Debug)]
pub enum Instruction {
    // Control flow instructions
    /// Block instruction: begins a block of code with a given signature
    Block(BlockType),
    /// Loop instruction: begins a loop with a given signature
    Loop(BlockType),
    /// If instruction: begins an if block with a given signature
    If(BlockType),
    /// Else instruction: marks the beginning of the else branch of an if block
    Else,
    /// End instruction: marks the end of a block, loop, if, or function
    End,
    /// Branch instruction: jumps to the specified label depth
    Br(u32),
    /// Conditional branch instruction: jumps to the specified label depth if the condition is true
    BrIf(u32),
    /// Table branch instruction: jumps to the label selected by an index from a table of label targets
    BrTable(Vec<u32>, u32),
    /// Return instruction: returns from the current function
    Return,
    /// Unreachable instruction: indicates an unreachable code path
    Unreachable,
    /// No-operation instruction: does nothing
    Nop,

    // Call instructions
    /// Call instruction: directly calls a function by its index
    Call(u32),
    /// Indirect call instruction: calls a function from a table at the given table index and type
    CallIndirect(u32, u32),
    /// Tail call instruction: calls a function and returns directly to the caller
    ReturnCall(u32),
    /// Indirect tail call instruction: calls a function from a table and returns directly to the caller
    ReturnCallIndirect(u32, u32),

    // Parametric instructions
    /// Drop instruction: pops and discards the top value from the stack
    Drop,
    /// Select instruction: selects one of two values based on a condition
    Select,
    /// Typed select instruction: selects one of two values based on a condition, with explicit type
    SelectTyped(ValueType),

    // Variable instructions
    /// Local get instruction: pushes the value of a local variable onto the stack
    LocalGet(u32),
    /// Local set instruction: sets the value of a local variable from the stack
    LocalSet(u32),
    /// Local tee instruction: sets a local variable and keeps the value on the stack
    LocalTee(u32),
    /// Global get instruction: pushes the value of a global variable onto the stack
    GlobalGet(u32),
    /// Global set instruction: sets the value of a global variable from the stack
    GlobalSet(u32),

    // Table instructions
    /// Table get instruction: gets an element from a table
    TableGet(u32),
    /// Table set instruction: sets an element in a table
    TableSet(u32),
    /// Table size instruction: gets the current size of a table
    TableSize(u32),
    /// Table grow instruction: grows a table by a given number of elements
    TableGrow(u32),
    /// Table init instruction: initializes a table segment from an element segment
    TableInit(u32, u32),
    /// Table copy instruction: copies elements from one table to another
    TableCopy(u32, u32),
    /// Table fill instruction: fills a table range with a given value
    TableFill(u32),
    /// Element drop instruction: drops an element segment
    ElemDrop(u32),

    // Memory instructions
    /// Load a 32-bit integer from memory
    I32Load(u32, u32),
    /// Load a 64-bit integer from memory
    I64Load(u32, u32),
    /// Load a 32-bit float from memory
    F32Load(u32, u32),
    /// Load a 64-bit float from memory
    F64Load(u32, u32),
    /// Load an 8-bit integer from memory and sign-extend to 32 bits
    I32Load8S(u32, u32),
    /// Load an 8-bit integer from memory and zero-extend to 32 bits
    I32Load8U(u32, u32),
    /// Load a 16-bit integer from memory and sign-extend to 32 bits
    I32Load16S(u32, u32),
    /// Load a 16-bit integer from memory and zero-extend to 32 bits
    I32Load16U(u32, u32),
    /// Load an 8-bit integer from memory and sign-extend to 64 bits
    I64Load8S(u32, u32),
    /// Load an 8-bit integer from memory and zero-extend to 64 bits
    I64Load8U(u32, u32),
    /// Load a 16-bit integer from memory and sign-extend to 64 bits
    I64Load16S(u32, u32),
    /// Load a 16-bit integer from memory and zero-extend to 64 bits
    I64Load16U(u32, u32),
    /// Load a 32-bit integer from memory and sign-extend to 64 bits
    I64Load32S(u32, u32),
    /// Load a 32-bit integer from memory and zero-extend to 64 bits
    I64Load32U(u32, u32),
    /// Store a 32-bit integer to memory
    I32Store(u32, u32),
    /// Store a 64-bit integer to memory
    I64Store(u32, u32),
    /// Store a 32-bit float to memory
    F32Store(u32, u32),
    /// Store a 64-bit float to memory
    F64Store(u32, u32),
    /// Store the low 8 bits of a 32-bit integer to memory
    I32Store8(u32, u32),
    /// Store the low 16 bits of a 32-bit integer to memory
    I32Store16(u32, u32),
    /// Store the low 8 bits of a 64-bit integer to memory
    I64Store8(u32, u32),
    /// Store the low 16 bits of a 64-bit integer to memory
    I64Store16(u32, u32),
    /// Store the low 32 bits of a 64-bit integer to memory
    I64Store32(u32, u32),
    /// Get the current size of memory in pages
    MemorySize,
    /// Grow memory by a given number of pages
    MemoryGrow,
    /// Initialize a region of memory from a data segment
    MemoryInit(u32),
    /// Drop a data segment
    DataDrop(u32),
    /// Copy data from one memory region to another
    MemoryCopy,
    /// Fill a memory region with a given value
    MemoryFill,

    // Numeric constant instructions
    /// Push a 32-bit integer constant onto the stack
    I32Const(i32),
    /// Push a 64-bit integer constant onto the stack
    I64Const(i64),
    /// Push a 32-bit float constant onto the stack
    F32Const(f32),
    /// Push a 64-bit float constant onto the stack
    F64Const(f64),

    // Comparison instructions
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

    // Arithmetic instructions
    /// Count leading zeros in a 32-bit integer
    I32Clz,
    /// Count trailing zeros in a 32-bit integer
    I32Ctz,
    /// Count number of set bits in a 32-bit integer
    I32Popcnt,
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
    /// Get remainder after dividing two 32-bit integers (signed)
    I32RemS,
    /// Get remainder after dividing two 32-bit integers (unsigned)
    I32RemU,
    /// Perform bitwise AND on two 32-bit integers
    I32And,
    /// Perform bitwise OR on two 32-bit integers
    I32Or,
    /// Perform bitwise XOR on two 32-bit integers
    I32Xor,
    /// Shift 32-bit integer left
    I32Shl,
    /// Shift 32-bit integer right (signed)
    I32ShrS,
    /// Shift 32-bit integer right (unsigned)
    I32ShrU,
    /// Rotate 32-bit integer left
    I32Rotl,
    /// Rotate 32-bit integer right
    I32Rotr,
    /// Count leading zeros in a 64-bit integer
    I64Clz,
    /// Count trailing zeros in a 64-bit integer
    I64Ctz,
    /// Count number of set bits in a 64-bit integer
    I64Popcnt,
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
    /// Get remainder after dividing two 64-bit integers (signed)
    I64RemS,
    /// Get remainder after dividing two 64-bit integers (unsigned)
    I64RemU,
    /// Perform bitwise AND on two 64-bit integers
    I64And,
    /// Perform bitwise OR on two 64-bit integers
    I64Or,
    /// Perform bitwise XOR on two 64-bit integers
    I64Xor,
    /// Shift 64-bit integer left
    I64Shl,
    /// Shift 64-bit integer right (signed)
    I64ShrS,
    /// Shift 64-bit integer right (unsigned)
    I64ShrU,
    /// Rotate 64-bit integer left
    I64Rotl,
    /// Rotate 64-bit integer right
    I64Rotr,
    /// Get the absolute value of a 32-bit float
    F32Abs,
    /// Negate a 32-bit float
    F32Neg,
    /// Round a 32-bit float up to the nearest integer
    F32Ceil,
    /// Round a 32-bit float down to the nearest integer
    F32Floor,
    /// Truncate a 32-bit float to an integer
    F32Trunc,
    /// Round a 32-bit float to the nearest integer
    F32Nearest,
    /// Calculate the square root of a 32-bit float
    F32Sqrt,
    /// Add two 32-bit float values
    F32Add,
    /// Subtract 32-bit float values
    F32Sub,
    /// Multiply 32-bit float values
    F32Mul,
    /// Divide 32-bit float values
    F32Div,
    /// Get the minimum of two 32-bit float values
    F32Min,
    /// Get the maximum of two 32-bit float values
    F32Max,
    /// Copy sign from one 32-bit float to another
    F32Copysign,
    /// Get the absolute value of a 64-bit float
    F64Abs,
    /// Negate a 64-bit float
    F64Neg,
    /// Round a 64-bit float up to the nearest integer
    F64Ceil,
    /// Round a 64-bit float down to the nearest integer
    F64Floor,
    /// Truncate a 64-bit float to an integer
    F64Trunc,
    /// Round a 64-bit float to the nearest integer
    F64Nearest,
    /// Calculate the square root of a 64-bit float
    F64Sqrt,
    /// Add two 64-bit float values
    F64Add,
    /// Subtract 64-bit float values
    F64Sub,
    /// Multiply 64-bit float values
    F64Mul,
    /// Divide 64-bit float values
    F64Div,
    /// Get the minimum of two 64-bit float values
    F64Min,
    /// Get the maximum of two 64-bit float values
    F64Max,
    /// Copy sign from one 64-bit float to another
    F64Copysign,

    // Conversion instructions
    /// Wrap a 64-bit integer to a 32-bit integer
    I32WrapI64,
    /// Truncate a 32-bit float to a signed 32-bit integer
    I32TruncF32S,
    /// Truncate a 32-bit float to an unsigned 32-bit integer
    I32TruncF32U,
    /// Truncate a 64-bit float to a signed 32-bit integer
    I32TruncF64S,
    /// Truncate a 64-bit float to an unsigned 32-bit integer
    I32TruncF64U,
    /// Extend a signed 32-bit integer to a 64-bit integer
    I64ExtendI32S,
    /// Extend an unsigned 32-bit integer to a 64-bit integer
    I64ExtendI32U,
    /// Truncate a 32-bit float to a signed 64-bit integer
    I64TruncF32S,
    /// Truncate a 32-bit float to an unsigned 64-bit integer
    I64TruncF32U,
    /// Truncate a 64-bit float to a signed 64-bit integer
    I64TruncF64S,
    /// Truncate a 64-bit float to an unsigned 64-bit integer
    I64TruncF64U,
    /// Sign-extend a 8-bit integer to a 32-bit integer
    I32Extend8S,
    /// Sign-extend a 16-bit integer to a 32-bit integer
    I32Extend16S,
    /// Sign-extend a 8-bit integer to a 64-bit integer
    I64Extend8S,
    /// Sign-extend a 16-bit integer to a 64-bit integer
    I64Extend16S,
    /// Sign-extend a 32-bit integer to a 64-bit integer
    I64Extend32S,
    /// Convert a signed 32-bit integer to a 32-bit float
    F32ConvertI32S,
    /// Convert an unsigned 32-bit integer to a 32-bit float
    F32ConvertI32U,
    /// Convert a signed 64-bit integer to a 32-bit float
    F32ConvertI64S,
    /// Convert an unsigned 64-bit integer to a 32-bit float
    F32ConvertI64U,
    /// Demote a 64-bit float to a 32-bit float
    F32DemoteF64,
    /// Convert a signed 32-bit integer to a 64-bit float
    F64ConvertI32S,
    /// Convert an unsigned 32-bit integer to a 64-bit float
    F64ConvertI32U,
    /// Convert a signed 64-bit integer to a 64-bit float
    F64ConvertI64S,
    /// Convert an unsigned 64-bit integer to a 64-bit float
    F64ConvertI64U,
    /// Promote a 32-bit float to a 64-bit float
    F64PromoteF32,
    /// Reinterpret the bits of a 32-bit float as a 32-bit integer
    I32ReinterpretF32,
    /// Reinterpret the bits of a 64-bit float as a 64-bit integer
    I64ReinterpretF64,
    /// Reinterpret the bits of a 32-bit integer as a 32-bit float
    F32ReinterpretI32,
    /// Reinterpret the bits of a 64-bit integer as a 64-bit float
    F64ReinterpretI64,

    // Basic SIMD instructions (minimal set for compatibility)
    /// Create a 128-bit vector by duplicating a 32-bit float value to all lanes
    F32x4Splat,
    /// Create a 128-bit vector by duplicating a 64-bit float value to all lanes
    F64x2Splat,
    /// Load a 128-bit value from memory
    V128Load(u32, u32),
    /// Store a 128-bit value to memory
    V128Store(u32, u32),
    /// Load 8 bits into a lane of a 128-bit vector
    V128Load8Lane(u32, u32, u8),
    /// Load 16 bits into a lane of a 128-bit vector
    V128Load16Lane(u32, u32, u8),
    /// Load 32 bits into a lane of a 128-bit vector
    V128Load32Lane(u32, u32, u8),
    /// Load 64 bits into a lane of a 128-bit vector
    V128Load64Lane(u32, u32, u8),
    /// Store a lane (8 bits) of a 128-bit vector to memory
    V128Store8Lane(u32, u32, u8),
    /// Store a lane (16 bits) of a 128-bit vector to memory
    V128Store16Lane(u32, u32, u8),
    /// Store a lane (32 bits) of a 128-bit vector to memory
    V128Store32Lane(u32, u32, u8),
    /// Store a lane (64 bits) of a 128-bit vector to memory
    V128Store64Lane(u32, u32, u8),

    // Reference instructions
    /// Create a null reference of the given type
    RefNull(ValueType),
    /// Test if a reference is null
    RefIsNull,
    /// Create a reference to a function
    RefFunc(u32),

    // Non-trapping Float-to-int Conversions
    /// Truncate a 32-bit float to a signed 32-bit integer with saturation
    I32TruncSatF32S,
    /// Truncate a 32-bit float to an unsigned 32-bit integer with saturation
    I32TruncSatF32U,
    /// Truncate a 64-bit float to a signed 32-bit integer with saturation
    I32TruncSatF64S,
    /// Truncate a 64-bit float to an unsigned 32-bit integer with saturation
    I32TruncSatF64U,
    /// Truncate a 32-bit float to a signed 64-bit integer with saturation
    I64TruncSatF32S,
    /// Truncate a 32-bit float to an unsigned 64-bit integer with saturation
    I64TruncSatF32U,
    /// Truncate a 64-bit float to a signed 64-bit integer with saturation
    I64TruncSatF64S,
    /// Truncate a 64-bit float to an unsigned 64-bit integer with saturation
    I64TruncSatF64U,
}

impl Instruction {
    /// Returns true if the instruction is a SIMD instruction
    #[must_use]
    pub const fn is_simd(&self) -> bool {
        match self {
            Self::V128Load(_, _)
            | Self::V128Store(_, _)
            | Self::V128Load8Lane(_, _, _)
            | Self::V128Load16Lane(_, _, _)
            | Self::V128Load32Lane(_, _, _)
            | Self::V128Load64Lane(_, _, _)
            | Self::V128Store8Lane(_, _, _)
            | Self::V128Store16Lane(_, _, _)
            | Self::V128Store32Lane(_, _, _)
            | Self::V128Store64Lane(_, _, _)
            | Self::F32x4Splat
            | Self::F64x2Splat => true,
            _ => false,
        }
    }
}
