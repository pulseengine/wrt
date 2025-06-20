//! WebAssembly Component Model parser
//!
//! This module provides parsing support for the WebAssembly Component Model,
//! which extends Core WebAssembly with higher-level composition features.

use wrt_foundation::bounded::BoundedVec;
use wrt_foundation::safe_memory::MemoryProvider;
use wrt_error::Result;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// A WebAssembly Component
#[derive(Debug, Clone)]
pub struct Component<P: MemoryProvider> {
    pub core_modules: BoundedVec<ComponentCoreModule<P>, 128, P>,
    pub component_types: BoundedVec<ComponentType<P>, 512, P>,
    pub imports: BoundedVec<ComponentImport<P>, 512, P>,
    pub exports: BoundedVec<ComponentExport<P>, 512, P>,
    pub instances: BoundedVec<ComponentInstance<P>, 256, P>,
    pub funcs: BoundedVec<ComponentFunc<P>, 1024, P>,
}

impl<P: MemoryProvider> Component<P> {
    /// Create a new empty component
    pub fn new(provider: P) -> Result<Self> {
        Ok(Component {
            core_modules: BoundedVec::new(provider.clone())?,
            component_types: BoundedVec::new(provider.clone())?,
            imports: BoundedVec::new(provider.clone())?,
            exports: BoundedVec::new(provider.clone())?,
            instances: BoundedVec::new(provider.clone())?,
            funcs: BoundedVec::new(provider)?,
        })
    }
}

/// A core WebAssembly module within a component
#[derive(Debug, Clone)]
pub struct ComponentCoreModule<P: MemoryProvider> {
    pub module: crate::module_builder::Module<P>,
}

/// Component type definition
#[derive(Debug, Clone)]
pub struct ComponentType<P: MemoryProvider> {
    pub name: BoundedVec<u8, 256, P>,
    pub type_def: ComponentTypeDef,
}

/// Component type definition variants
#[derive(Debug, Clone)]
pub enum ComponentTypeDef {
    /// Core type (from core WebAssembly)
    Core(CoreType),
    /// Component function type
    Func(ComponentFuncType),
    /// Component interface type
    Interface(InterfaceType),
    /// Component instance type
    Instance(InstanceType),
}

/// Core WebAssembly type
#[derive(Debug, Clone)]
pub enum CoreType {
    Func(crate::types::FuncType),
    Module(ModuleType),
}

/// Component function type
#[derive(Debug, Clone)]
pub struct ComponentFuncType {
    pub params: BoundedVec<(BoundedVec<u8, 64, crate::ParserProvider>, ComponentValType), 32, crate::ParserProvider>,
    pub results: BoundedVec<ComponentValType, 32, crate::ParserProvider>,
}

/// Component value type
#[derive(Debug, Clone)]
pub enum ComponentValType {
    /// Primitive types
    Bool,
    S8,
    U8,
    S16,
    U16,
    S32,
    U32,
    S64,
    U64,
    Float32,
    Float64,
    Char,
    String,
    /// List type - uses index instead of Box for no_std compatibility
    List(u32), // Type index
    /// Record type
    Record(BoundedVec<RecordField, 32, crate::ParserProvider>),
    /// Variant type
    Variant(BoundedVec<VariantCase, 32, crate::ParserProvider>),
    /// Tuple type
    Tuple(BoundedVec<ComponentValType, 32, crate::ParserProvider>),
    /// Flags type
    Flags(BoundedVec<BoundedVec<u8, 64, crate::ParserProvider>, 32, crate::ParserProvider>),
    /// Enum type
    Enum(BoundedVec<BoundedVec<u8, 64, crate::ParserProvider>, 32, crate::ParserProvider>),
    /// Option type - uses index instead of Box
    Option(u32), // Type index
    /// Result type - uses indices instead of Box
    Result { ok: Option<u32>, err: Option<u32> }, // Type indices
}

/// Record field
#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: BoundedVec<u8, 64, crate::ParserProvider>,
    pub ty: ComponentValType,
}

/// Variant case
#[derive(Debug, Clone)]
pub struct VariantCase {
    pub name: BoundedVec<u8, 64, crate::ParserProvider>,
    pub ty: Option<ComponentValType>,
}

/// Interface type
#[derive(Debug, Clone)]
pub struct InterfaceType {
    // TODO: Define interface type structure
}

/// Instance type
#[derive(Debug, Clone)]
pub struct InstanceType {
    // TODO: Define instance type structure
}

/// Module type
#[derive(Debug, Clone)]
pub struct ModuleType {
    // TODO: Define module type structure
}

/// Component import
#[derive(Debug, Clone)]
pub struct ComponentImport<P: MemoryProvider> {
    pub name: BoundedVec<u8, 256, P>,
    pub url: BoundedVec<u8, 512, P>,
    pub desc: ComponentImportDesc,
}

/// Component import description
#[derive(Debug, Clone)]
pub enum ComponentImportDesc {
    Func(u32),
    Value(ComponentValType),
    Type(u32),
    Instance(u32),
    Component(u32),
}

/// Component export
#[derive(Debug, Clone)]
pub struct ComponentExport<P: MemoryProvider> {
    pub name: BoundedVec<u8, 256, P>,
    pub desc: ComponentExportDesc,
}

/// Component export description
#[derive(Debug, Clone)]
pub enum ComponentExportDesc {
    Func(u32),
    Value(u32),
    Type(u32),
    Instance(u32),
    Component(u32),
}

/// Component instance
#[derive(Debug, Clone)]
pub struct ComponentInstance<P: MemoryProvider> {
    pub instantiate_args: BoundedVec<ComponentArg<P>, 64, P>,
}

/// Component instantiation argument
#[derive(Debug, Clone)]
pub struct ComponentArg<P: MemoryProvider> {
    pub name: BoundedVec<u8, 64, P>,
    pub kind: ComponentArgKind,
}

/// Component argument kind
#[derive(Debug, Clone)]
pub enum ComponentArgKind {
    Instance(u32),
    Component(u32),
}

/// Component function
#[derive(Debug, Clone)]
pub struct ComponentFunc<P: MemoryProvider> {
    pub type_idx: u32,
    pub canonical_options: BoundedVec<CanonicalOption, 16, P>,
}

/// Canonical function options
#[derive(Debug, Clone)]
pub enum CanonicalOption {
    UTF8,
    UTF16,
    CompactUTF16,
    Memory(u32),
    Realloc(u32),
    PostReturn(u32),
}