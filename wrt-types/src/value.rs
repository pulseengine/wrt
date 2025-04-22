/// Represents a WebAssembly value type
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    /// 32-bit integer
    I32,
    /// 64-bit integer
    I64,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
    /// 128-bit vector
    V128,
    /// Function reference
    FuncRef,
    /// External reference
    ExternRef,
}

/// Represents a WebAssembly value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 32-bit integer
    I32(i32),
    /// 64-bit integer
    I64(i64),
    /// 32-bit floating point
    F32(f32),
    /// 64-bit floating point
    F64(f64),
    /// 128-bit vector
    V128(u128),
    /// Function reference
    FuncRef(Option<u32>),
    /// External reference
    ExternRef(Option<u32>),
}

impl Value {
    /// Creates a default value for the given type
    #[must_use]
    pub fn default_for_type(ty: &ValueType) -> Self {
        match ty {
            ValueType::I32 => Self::I32(0),
            ValueType::I64 => Self::I64(0),
            ValueType::F32 => Self::F32(0.0),
            ValueType::F64 => Self::F64(0.0),
            ValueType::V128 => Self::V128(0),
            ValueType::FuncRef => Self::FuncRef(None),
            ValueType::ExternRef => Self::ExternRef(None),
        }
    }

    /// Checks if this value matches the given type
    #[must_use]
    pub fn matches_type(&self, ty: &ValueType) -> bool {
        match (self, ty) {
            (Self::I32(_), ValueType::I32) => true,
            (Self::I64(_), ValueType::I64) => true,
            (Self::F32(_), ValueType::F32) => true,
            (Self::F64(_), ValueType::F64) => true,
            (Self::V128(_), ValueType::V128) => true,
            (Self::FuncRef(_), ValueType::FuncRef) => true,
            (Self::ExternRef(_), ValueType::ExternRef) => true,
            _ => false,
        }
    }
}
