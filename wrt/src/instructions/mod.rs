//! WebAssembly instruction implementations
//!
//! This module contains implementations for all WebAssembly instructions,
//! organized into submodules by instruction category.

use crate::types::ValueType;
use crate::Vec;

pub mod arithmetic;
pub mod bit_counting;
pub mod comparison;
pub mod control;
pub mod conversion;
pub mod memory;
pub mod numeric_constants;
pub mod parametric;
pub mod simd;
pub mod table;
pub mod variable;

pub use arithmetic::*;
pub use bit_counting::*;
pub use comparison::*;
pub use control::*;
pub use conversion::*;
pub use memory::*;
pub use numeric_constants::*;
pub use parametric::*;
pub use simd::*;
pub use table::*;
pub use variable::*;

/// Represents a WebAssembly instruction
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Control flow instructions
    Block(BlockType),
    Loop(BlockType),
    If(BlockType),
    Else,
    End,
    Br(u32),
    BrIf(u32),
    BrTable(Vec<u32>, u32),
    Return,
    Unreachable,
    Nop,

    // Call instructions
    Call(u32),
    CallIndirect(u32, u32),
    ReturnCall(u32),
    ReturnCallIndirect(u32, u32),

    // Parametric instructions
    Drop,
    Select,
    SelectTyped(ValueType),

    // Variable instructions
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),

    // Table instructions
    TableGet(u32),
    TableSet(u32),
    TableSize(u32),
    TableGrow(u32),
    TableInit(u32, u32),
    TableCopy(u32, u32),
    TableFill(u32),
    ElemDrop(u32),

    // Memory instructions
    I32Load(u32, u32),
    I64Load(u32, u32),
    F32Load(u32, u32),
    F64Load(u32, u32),
    I32Load8S(u32, u32),
    I32Load8U(u32, u32),
    I32Load16S(u32, u32),
    I32Load16U(u32, u32),
    I64Load8S(u32, u32),
    I64Load8U(u32, u32),
    I64Load16S(u32, u32),
    I64Load16U(u32, u32),
    I64Load32S(u32, u32),
    I64Load32U(u32, u32),
    I32Store(u32, u32),
    I64Store(u32, u32),
    F32Store(u32, u32),
    F64Store(u32, u32),
    I32Store8(u32, u32),
    I32Store16(u32, u32),
    I64Store8(u32, u32),
    I64Store16(u32, u32),
    I64Store32(u32, u32),
    MemorySize,
    MemoryGrow,
    MemoryInit(u32),
    DataDrop(u32),
    MemoryCopy,
    MemoryFill,

    // Numeric constant instructions
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
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

    // SIMD - v128 manipulation
    /// Load a 128-bit value from memory
    V128Load(u32, u32),
    /// Store a 128-bit value to memory
    V128Store(u32, u32),
    /// Create a 128-bit constant
    V128Const([u8; 16]),

    // SIMD - Basic operations
    /// Shuffle bytes from two 128-bit values into a new 128-bit value
    I8x16Shuffle([u8; 16]),
    /// Swizzle bytes within a single 128-bit value
    I8x16Swizzle,

    // SIMD - Lane-wise operations
    /// Extract a 8-bit lane as a signed value from a 128-bit vector
    I8x16ExtractLaneS(u8),
    /// Extract a 8-bit lane as an unsigned value from a 128-bit vector
    I8x16ExtractLaneU(u8),
    /// Replace a 8-bit lane in a 128-bit vector
    I8x16ReplaceLane(u8),
    /// Extract a 16-bit lane as a signed value from a 128-bit vector
    I16x8ExtractLaneS(u8),
    /// Extract a 16-bit lane as an unsigned value from a 128-bit vector
    I16x8ExtractLaneU(u8),
    /// Replace a 16-bit lane in a 128-bit vector
    I16x8ReplaceLane(u8),
    /// Extract a 32-bit lane from a 128-bit vector
    I32x4ExtractLane(u8),
    /// Replace a 32-bit lane in a 128-bit vector
    I32x4ReplaceLane(u8),
    /// Extract a 64-bit lane from a 128-bit vector
    I64x2ExtractLane(u8),
    /// Replace a 64-bit lane in a 128-bit vector
    I64x2ReplaceLane(u8),
    /// Extract a 32-bit float lane from a 128-bit vector
    F32x4ExtractLane(u8),
    /// Replace a 32-bit float lane in a 128-bit vector
    F32x4ReplaceLane(u8),
    /// Extract a 64-bit float lane from a 128-bit vector
    F64x2ExtractLane(u8),
    /// Replace a 64-bit float lane in a 128-bit vector
    F64x2ReplaceLane(u8),

    // SIMD - Splat operations
    /// Create a 128-bit vector by duplicating a 8-bit value to all lanes
    I8x16Splat,
    /// Create a 128-bit vector by duplicating a 16-bit value to all lanes
    I16x8Splat,
    /// Create a 128-bit vector by duplicating a 32-bit value to all lanes
    I32x4Splat,
    /// Create a 128-bit vector by duplicating a 64-bit value to all lanes
    I64x2Splat,
    /// Create a 128-bit vector by duplicating a 32-bit float value to all lanes
    F32x4Splat,
    /// Create a 128-bit vector by duplicating a 64-bit float value to all lanes
    F64x2Splat,

    // SIMD - Comparison operations
    /// Compare two 8-bit integers for equality (128-bit SIMD)
    I8x16Eq,
    /// Compare two 8-bit integers for inequality (128-bit SIMD)
    I8x16Ne,
    /// Signed less than comparison for 8-bit integers (128-bit SIMD)
    I8x16LtS,
    /// Unsigned less than comparison for 8-bit integers (128-bit SIMD)
    I8x16LtU,
    /// Signed greater than comparison for 8-bit integers (128-bit SIMD)
    I8x16GtS,
    /// Unsigned greater than comparison for 8-bit integers (128-bit SIMD)
    I8x16GtU,
    /// Signed less than or equal comparison for 8-bit integers (128-bit SIMD)
    I8x16LeS,
    /// Unsigned less than or equal comparison for 8-bit integers (128-bit SIMD)
    I8x16LeU,
    /// Signed greater than or equal comparison for 8-bit integers (128-bit SIMD)
    I8x16GeS,
    /// Unsigned greater than or equal comparison for 8-bit integers (128-bit SIMD)
    I8x16GeU,

    /// Compare two 16-bit integers for equality (128-bit SIMD)
    I16x8Eq,
    /// Compare two 16-bit integers for inequality (128-bit SIMD)
    I16x8Ne,
    /// Signed less than comparison for 16-bit integers (128-bit SIMD)
    I16x8LtS,
    /// Unsigned less than comparison for 16-bit integers (128-bit SIMD)
    I16x8LtU,
    /// Signed greater than comparison for 16-bit integers (128-bit SIMD)
    I16x8GtS,
    /// Unsigned greater than comparison for 16-bit integers (128-bit SIMD)
    I16x8GtU,
    /// Signed less than or equal comparison for 16-bit integers (128-bit SIMD)
    I16x8LeS,
    /// Unsigned less than or equal comparison for 16-bit integers (128-bit SIMD)
    I16x8LeU,
    /// Signed greater than or equal comparison for 16-bit integers (128-bit SIMD)
    I16x8GeS,
    /// Unsigned greater than or equal comparison for 16-bit integers (128-bit SIMD)
    I16x8GeU,

    /// Compare two 32-bit integers for equality (128-bit SIMD)
    I32x4Eq,
    /// Compare two 32-bit integers for inequality (128-bit SIMD)
    I32x4Ne,
    /// Signed less than comparison for 32-bit integers (128-bit SIMD)
    I32x4LtS,
    /// Unsigned less than comparison for 32-bit integers (128-bit SIMD)
    I32x4LtU,
    /// Signed greater than comparison for 32-bit integers (128-bit SIMD)
    I32x4GtS,
    /// Unsigned greater than comparison for 32-bit integers (128-bit SIMD)
    I32x4GtU,
    /// Signed less than or equal comparison for 32-bit integers (128-bit SIMD)
    I32x4LeS,
    /// Unsigned less than or equal comparison for 32-bit integers (128-bit SIMD)
    I32x4LeU,
    /// Signed greater than or equal comparison for 32-bit integers (128-bit SIMD)
    I32x4GeS,
    /// Unsigned greater than or equal comparison for 32-bit integers (128-bit SIMD)
    I32x4GeU,

    /// Compare two 64-bit integers for equality (128-bit SIMD)
    I64x2Eq,
    /// Compare two 64-bit integers for inequality (128-bit SIMD)
    I64x2Ne,
    /// Signed less than comparison for 64-bit integers (128-bit SIMD)
    I64x2LtS,
    /// Signed greater than comparison for 64-bit integers (128-bit SIMD)
    I64x2GtS,
    /// Signed less than or equal comparison for 64-bit integers (128-bit SIMD)
    I64x2LeS,
    /// Signed greater than or equal comparison for 64-bit integers (128-bit SIMD)
    I64x2GeS,

    /// Compare two 32-bit floats for equality (128-bit SIMD)
    F32x4Eq,
    /// Compare two 32-bit floats for inequality (128-bit SIMD)
    F32x4Ne,
    /// Less than comparison for 32-bit floats (128-bit SIMD)
    F32x4Lt,
    /// Greater than comparison for 32-bit floats (128-bit SIMD)
    F32x4Gt,
    /// Less than or equal comparison for 32-bit floats (128-bit SIMD)
    F32x4Le,
    /// Greater than or equal comparison for 32-bit floats (128-bit SIMD)
    F32x4Ge,

    /// Compare two 64-bit floats for equality (128-bit SIMD)
    F64x2Eq,
    /// Compare two 64-bit floats for inequality (128-bit SIMD)
    F64x2Ne,
    /// Less than comparison for 64-bit floats (128-bit SIMD)
    F64x2Lt,
    /// Greater than comparison for 64-bit floats (128-bit SIMD)
    F64x2Gt,
    /// Less than or equal comparison for 64-bit floats (128-bit SIMD)
    F64x2Le,
    /// Greater than or equal comparison for 64-bit floats (128-bit SIMD)
    F64x2Ge,

    // SIMD - Arithmetic operations
    /// Negate each 8-bit integer lane (128-bit SIMD)
    I8x16Neg,
    /// Add two 8-bit integer lanes (128-bit SIMD)
    I8x16Add,
    /// Add two 8-bit integer lanes with signed saturation (128-bit SIMD)
    I8x16AddSaturateS,
    /// Add two 8-bit integer lanes with unsigned saturation (128-bit SIMD)
    I8x16AddSaturateU,
    /// Subtract 8-bit integer lanes (128-bit SIMD)
    I8x16Sub,
    /// Subtract 8-bit integer lanes with signed saturation (128-bit SIMD)
    I8x16SubSaturateS,
    /// Subtract 8-bit integer lanes with unsigned saturation (128-bit SIMD)
    I8x16SubSaturateU,

    /// Negate each 16-bit integer lane (128-bit SIMD)
    I16x8Neg,
    /// Add 16-bit integer lanes (128-bit SIMD)
    I16x8Add,
    /// Add 16-bit integer lanes with signed saturation (128-bit SIMD)
    I16x8AddSaturateS,
    /// Add 16-bit integer lanes with unsigned saturation (128-bit SIMD)
    I16x8AddSaturateU,
    /// Subtract 16-bit integer lanes (128-bit SIMD)
    I16x8Sub,
    /// Subtract 16-bit integer lanes with signed saturation (128-bit SIMD)
    I16x8SubSaturateS,
    /// Subtract 16-bit integer lanes with unsigned saturation (128-bit SIMD)
    I16x8SubSaturateU,
    /// Multiply 16-bit integer lanes (128-bit SIMD)
    I16x8Mul,

    /// Negate each 32-bit integer lane (128-bit SIMD)
    I32x4Neg,
    /// Add 32-bit integer lanes (128-bit SIMD)
    I32x4Add,
    /// Subtract 32-bit integer lanes (128-bit SIMD)
    I32x4Sub,
    /// Multiply 32-bit integer lanes (128-bit SIMD)
    I32x4Mul,
    /// Dot product of 16-bit integer lanes with signed saturation (128-bit SIMD)
    I32x4DotI16x8S,

    /// Negate each 64-bit integer lane (128-bit SIMD)
    I64x2Neg,
    /// Add 64-bit integer lanes (128-bit SIMD)
    I64x2Add,
    /// Subtract 64-bit integer lanes (128-bit SIMD)
    I64x2Sub,
    /// Multiply 64-bit integer lanes (128-bit SIMD)
    I64x2Mul,

    /// Absolute value of 32-bit float lanes (128-bit SIMD)
    F32x4Abs,
    /// Negate 32-bit float lanes (128-bit SIMD)
    F32x4Neg,
    /// Square root of 32-bit float lanes (128-bit SIMD)
    F32x4Sqrt,
    /// Add 32-bit float lanes (128-bit SIMD)
    F32x4Add,
    /// Subtract 32-bit float lanes (128-bit SIMD)
    F32x4Sub,
    /// Multiply 32-bit float lanes (128-bit SIMD)
    F32x4Mul,
    /// Divide 32-bit float lanes (128-bit SIMD)
    F32x4Div,
    /// Minimum of 32-bit float lanes (128-bit SIMD)
    F32x4Min,
    /// Maximum of 32-bit float lanes (128-bit SIMD)
    F32x4Max,

    /// Absolute value of 64-bit float lanes (128-bit SIMD)
    F64x2Abs,
    /// Negate 64-bit float lanes (128-bit SIMD)
    F64x2Neg,
    /// Square root of 64-bit float lanes (128-bit SIMD)
    F64x2Sqrt,
    /// Add 64-bit float lanes (128-bit SIMD)
    F64x2Add,
    /// Subtract 64-bit float lanes (128-bit SIMD)
    F64x2Sub,
    /// Multiply 64-bit float lanes (128-bit SIMD)
    F64x2Mul,
    /// Divide 64-bit float lanes (128-bit SIMD)
    F64x2Div,
    /// Minimum of 64-bit float lanes (128-bit SIMD)
    F64x2Min,
    /// Maximum of 64-bit float lanes (128-bit SIMD)
    F64x2Max,

    // SIMD - Bitwise operations
    /// Bitwise NOT of 128-bit value
    V128Not,
    /// Bitwise AND of two 128-bit values
    V128And,
    /// Bitwise AND NOT of two 128-bit values
    V128AndNot,
    /// Bitwise OR of two 128-bit values
    V128Or,
    /// Bitwise XOR of two 128-bit values
    V128Xor,
    /// Bitwise select operation using three 128-bit values
    V128Bitselect,

    // SIMD - Conversion operations
    /// Truncate 32-bit float lanes to 32-bit integer lanes with signed saturation
    I32x4TruncSatF32x4S,
    /// Truncate 32-bit float lanes to 32-bit integer lanes with unsigned saturation
    I32x4TruncSatF32x4U,
    /// Convert 32-bit signed integer lanes to 32-bit float lanes
    F32x4ConvertI32x4S,
    /// Convert 32-bit unsigned integer lanes to 32-bit float lanes
    F32x4ConvertI32x4U,

    // Relaxed SIMD operations
    #[cfg(feature = "relaxed_simd")]
    F32x4RelaxedMin,
    #[cfg(feature = "relaxed_simd")]
    F32x4RelaxedMax,
    #[cfg(feature = "relaxed_simd")]
    F64x2RelaxedMin,
    #[cfg(feature = "relaxed_simd")]
    F64x2RelaxedMax,
    #[cfg(feature = "relaxed_simd")]
    I16x8RelaxedQ15MulrS,
    #[cfg(feature = "relaxed_simd")]
    I16x8RelaxedDotI8x16I7x16S,
    #[cfg(feature = "relaxed_simd")]
    I32x4RelaxedDotI8x16I7x16AddS,
    #[cfg(feature = "relaxed_simd")]
    I8x16RelaxedSwizzle,
    #[cfg(feature = "relaxed_simd")]
    I32x4RelaxedTruncSatF32x4S,
    #[cfg(feature = "relaxed_simd")]
    I32x4RelaxedTruncSatF32x4U,
    #[cfg(feature = "relaxed_simd")]
    I32x4RelaxedTruncSatF64x2SZero,
    #[cfg(feature = "relaxed_simd")]
    I32x4RelaxedTruncSatF64x2UZero,
}

/// Block type for control flow instructions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    /// Empty block type (no parameters or results)
    Empty,
    /// Single value type
    Type(ValueType),
    /// Type index into the module's type section
    TypeIndex(u32),
}

/// A trait for instructions that can be executed by the stackless engine
pub trait InstructionExecutor {
    /// Execute the instruction in the given context
    ///
    /// # Arguments
    /// * `stack` - The execution stack
    /// * `frame` - The current execution frame
    ///
    /// # Returns
    /// * `Ok(())` - If the instruction executed successfully
    /// * `Err(Error)` - If an error occurred
    fn execute(
        &self,
        stack: &mut crate::execution::Stack,
        frame: &mut crate::execution::Frame,
    ) -> std::result::Result<(), crate::error::Error>;
}

impl InstructionExecutor for Instruction {
    fn execute(
        &self,
        stack: &mut crate::execution::Stack,
        frame: &mut crate::execution::Frame,
    ) -> std::result::Result<(), crate::error::Error> {
        use crate::error::Error;

        // First try to handle with the specialized SIMD executor
        if let Ok(result) = simd::handle_simd_instruction(self, stack, frame) {
            return Ok(());
        }

        // Then handle other instruction types
        match self {
            // Comparison instructions
            Instruction::I32Eqz => comparison::i32_eqz(&mut stack.values),
            Instruction::I32Eq => comparison::i32_eq(&mut stack.values),
            Instruction::I32Ne => comparison::i32_ne(&mut stack.values),
            Instruction::I32LtS => comparison::i32_lt_s(&mut stack.values),
            Instruction::I32LtU => comparison::i32_lt_u(&mut stack.values),
            Instruction::I32GtS => comparison::i32_gt_s(&mut stack.values),
            Instruction::I32GtU => comparison::i32_gt_u(&mut stack.values),
            Instruction::I32LeS => comparison::i32_le_s(&mut stack.values),
            Instruction::I32LeU => comparison::i32_le_u(&mut stack.values),
            Instruction::I32GeS => comparison::i32_ge_s(&mut stack.values),
            Instruction::I32GeU => comparison::i32_ge_u(&mut stack.values),

            Instruction::I64Eqz => comparison::i64_eqz(&mut stack.values),
            Instruction::I64Eq => comparison::i64_eq(&mut stack.values),
            Instruction::I64Ne => comparison::i64_ne(&mut stack.values),
            Instruction::I64LtS => comparison::i64_lt_s(&mut stack.values),
            Instruction::I64LtU => comparison::i64_lt_u(&mut stack.values),
            Instruction::I64GtS => comparison::i64_gt_s(&mut stack.values),
            Instruction::I64GtU => comparison::i64_gt_u(&mut stack.values),
            Instruction::I64LeS => comparison::i64_le_s(&mut stack.values),
            Instruction::I64LeU => comparison::i64_le_u(&mut stack.values),
            Instruction::I64GeS => comparison::i64_ge_s(&mut stack.values),
            Instruction::I64GeU => comparison::i64_ge_u(&mut stack.values),

            // For other instructions, defer to other matchers or return not implemented
            _ => Err(Error::Execution(format!(
                "Instruction not implemented via trait: {:?}",
                self
            ))),
        }
    }
}
