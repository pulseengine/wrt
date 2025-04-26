//! WebAssembly Component Model format.
//!
//! This module provides types and utilities for working with the WebAssembly
//! Component Model binary format.

#[cfg(feature = "std")]
use std::{boxed::Box, format, string::String, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::module::Module;
use crate::types::ValueType;

/// WebAssembly Component Model component definition
#[derive(Debug, Clone)]
pub struct Component {
    /// Component name (if available from name section)
    pub name: Option<String>,
    /// Core modules included in this component
    pub modules: Vec<Module>,
    /// Core instances defined in this component
    pub core_instances: Vec<CoreInstance>,
    /// Core types defined in this component
    pub core_types: Vec<CoreType>,
    /// Nested components
    pub components: Vec<Component>,
    /// Component instances
    pub instances: Vec<Instance>,
    /// Component aliases
    pub aliases: Vec<Alias>,
    /// Component types
    pub types: Vec<ComponentType>,
    /// Canonical function conversions
    pub canonicals: Vec<Canon>,
    /// Component start function
    pub start: Option<Start>,
    /// Component imports
    pub imports: Vec<Import>,
    /// Component exports
    pub exports: Vec<Export>,
    /// Component values
    pub values: Vec<Value>,
    /// Original binary (if available)
    pub binary: Option<Vec<u8>>,
}

impl Default for Component {
    fn default() -> Self {
        Self::new()
    }
}

impl Component {
    /// Create a new empty component
    pub fn new() -> Self {
        Self {
            name: None,
            modules: Vec::new(),
            core_instances: Vec::new(),
            core_types: Vec::new(),
            components: Vec::new(),
            instances: Vec::new(),
            aliases: Vec::new(),
            types: Vec::new(),
            canonicals: Vec::new(),
            start: None,
            imports: Vec::new(),
            exports: Vec::new(),
            values: Vec::new(),
            binary: None,
        }
    }
}

/// Core WebAssembly instance definition in a component
#[derive(Debug, Clone)]
pub struct CoreInstance {
    /// Instance expression
    pub instance_expr: CoreInstanceExpr,
}

/// Core WebAssembly instance expression
#[derive(Debug, Clone)]
pub enum CoreInstanceExpr {
    /// Instantiate a core module
    Instantiate {
        /// Module index
        module_idx: u32,
        /// Instantiation arguments
        args: Vec<CoreInstantiateArg>,
    },
    /// Collection of inlined exports
    InlineExports(Vec<CoreInlineExport>),
}

/// Core WebAssembly instantiation argument
#[derive(Debug, Clone)]
pub struct CoreInstantiateArg {
    /// Name of the argument
    pub name: String,
    /// Instance index that provides the value
    pub instance_idx: u32,
}

/// Core WebAssembly inlined export
#[derive(Debug, Clone)]
pub struct CoreInlineExport {
    /// Export name
    pub name: String,
    /// Export reference
    pub sort: CoreSort,
    /// Index within the sort
    pub idx: u32,
}

/// Core WebAssembly sort kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreSort {
    /// Function reference
    Function,
    /// Table reference
    Table,
    /// Memory reference
    Memory,
    /// Global reference
    Global,
    /// Type reference
    Type,
    /// Module reference
    Module,
    /// Instance reference
    Instance,
}

/// Core WebAssembly type definition in a component
#[derive(Debug, Clone)]
pub struct CoreType {
    /// Type definition
    pub definition: CoreTypeDefinition,
}

/// Core WebAssembly type definition
#[derive(Debug, Clone)]
pub enum CoreTypeDefinition {
    /// Function type
    Function {
        /// Parameter types
        params: Vec<ValueType>,
        /// Result types
        results: Vec<ValueType>,
    },
    /// Module type
    Module {
        /// Module imports
        imports: Vec<(String, String, CoreExternType)>,
        /// Module exports
        exports: Vec<(String, CoreExternType)>,
    },
}

/// Core WebAssembly external type
#[derive(Debug, Clone)]
pub enum CoreExternType {
    /// Function type
    Function {
        /// Parameter types
        params: Vec<ValueType>,
        /// Result types
        results: Vec<ValueType>,
    },
    /// Table type
    Table {
        /// Element type
        element_type: ValueType,
        /// Minimum size
        min: u32,
        /// Maximum size (optional)
        max: Option<u32>,
    },
    /// Memory type
    Memory {
        /// Minimum size in pages
        min: u32,
        /// Maximum size in pages (optional)
        max: Option<u32>,
        /// Whether the memory is shared
        shared: bool,
    },
    /// Global type
    Global {
        /// Value type
        value_type: ValueType,
        /// Whether the global is mutable
        mutable: bool,
    },
}

/// Component instance definition
#[derive(Debug, Clone)]
pub struct Instance {
    /// Instance expression
    pub instance_expr: InstanceExpr,
}

/// Component instance expression
#[derive(Debug, Clone)]
pub enum InstanceExpr {
    /// Instantiate a component
    Instantiate {
        /// Component index
        component_idx: u32,
        /// Instantiation arguments
        args: Vec<InstantiateArg>,
    },
    /// Collection of inlined exports
    InlineExports(Vec<InlineExport>),
}

/// Component instantiation argument
#[derive(Debug, Clone)]
pub struct InstantiateArg {
    /// Name of the argument
    pub name: String,
    /// Sort of the referenced item
    pub sort: Sort,
    /// Index within the sort
    pub idx: u32,
}

/// Component inlined export
#[derive(Debug, Clone)]
pub struct InlineExport {
    /// Export name
    pub name: String,
    /// Export reference
    pub sort: Sort,
    /// Index within the sort
    pub idx: u32,
}

/// Component sort kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    /// Core reference
    Core(CoreSort),
    /// Function reference
    Function,
    /// Value reference
    Value,
    /// Type reference
    Type,
    /// Component reference
    Component,
    /// Instance reference
    Instance,
}

/// Component alias definition
#[derive(Debug, Clone)]
pub struct Alias {
    /// Alias target
    pub target: AliasTarget,
}

/// Component alias target
#[derive(Debug, Clone)]
pub enum AliasTarget {
    /// Core WebAssembly export from an instance
    CoreInstanceExport {
        /// Instance index
        instance_idx: u32,
        /// Export name
        name: String,
        /// Kind of the target
        kind: CoreSort,
    },
    /// Export from a component instance
    InstanceExport {
        /// Instance index
        instance_idx: u32,
        /// Export name
        name: String,
        /// Kind of the target
        kind: Sort,
    },
    /// Outer definition from an enclosing component (forwarding from parent)
    Outer {
        /// Count of components to traverse outward
        count: u32,
        /// Kind of the target
        kind: Sort,
        /// Index within the kind
        idx: u32,
    },
}

/// Component type definition
#[derive(Debug, Clone)]
pub struct ComponentType {
    /// Type definition
    pub definition: ComponentTypeDefinition,
}

/// Component type definition
#[derive(Debug, Clone)]
pub enum ComponentTypeDefinition {
    /// Component type
    Component {
        /// Component imports
        imports: Vec<(String, String, ExternType)>,
        /// Component exports
        exports: Vec<(String, ExternType)>,
    },
    /// Instance type
    Instance {
        /// Instance exports
        exports: Vec<(String, ExternType)>,
    },
    /// Function type
    Function {
        /// Parameter types
        params: Vec<(String, ValType)>,
        /// Result types
        results: Vec<ValType>,
    },
    /// Value type
    Value(ValType),
    /// Resource type
    Resource {
        /// Resource representation type
        representation: ResourceRepresentation,
        /// Whether the resource is nullable
        nullable: bool,
    },
}

/// Component external type
#[derive(Debug, Clone)]
pub enum ExternType {
    /// Function type
    Function {
        /// Parameter types
        params: Vec<(String, ValType)>,
        /// Result types
        results: Vec<ValType>,
    },
    /// Value type
    Value(ValType),
    /// Type reference
    Type(u32),
    /// Instance type
    Instance {
        /// Instance exports
        exports: Vec<(String, ExternType)>,
    },
    /// Component type
    Component {
        /// Component imports
        imports: Vec<(String, String, ExternType)>,
        /// Component exports
        exports: Vec<(String, ExternType)>,
    },
}

/// Component value type including fixed-length lists
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValType {
    /// Boolean type
    Bool,
    /// 8-bit signed integer
    S8,
    /// 8-bit unsigned integer
    U8,
    /// 16-bit signed integer
    S16,
    /// 16-bit unsigned integer
    U16,
    /// 32-bit signed integer
    S32,
    /// 32-bit unsigned integer
    U32,
    /// 64-bit signed integer
    S64,
    /// 64-bit unsigned integer
    U64,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// Unicode character
    Char,
    /// String type
    String,
    /// Reference type
    Ref(u32),
    /// Record type with named fields
    Record(Vec<(String, ValType)>),
    /// Variant type
    Variant(Vec<(String, Option<ValType>)>),
    /// List type
    List(Box<ValType>),
    /// Fixed-length list type with element type and length
    FixedList(Box<ValType>, u32),
    /// Tuple type
    Tuple(Vec<ValType>),
    /// Flags type
    Flags(Vec<String>),
    /// Enum type
    Enum(Vec<String>),
    /// Option type
    Option(Box<ValType>),
    /// Result type (ok only)
    Result(Box<ValType>),
    /// Result type (error only)
    ResultErr(Box<ValType>),
    /// Result type (ok and error)
    ResultBoth(Box<ValType>, Box<ValType>),
    /// Own a resource
    Own(u32),
    /// Borrow a resource
    Borrow(u32),
    /// Error context type
    ErrorContext,
}

/// Resource representation type
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceRepresentation {
    /// 32-bit integer handle
    Handle32,
    /// 64-bit integer handle
    Handle64,
    /// Record representation
    Record(Vec<String>),
    /// Aggregate representation
    Aggregate(Vec<u32>),
}

/// Canonical function conversion
#[derive(Debug, Clone)]
pub struct Canon {
    /// Canonical operation
    pub operation: CanonOperation,
}

/// Canonical operation
#[derive(Debug, Clone)]
pub enum CanonOperation {
    /// Lift a core function to the component ABI
    Lift {
        /// Core function index
        func_idx: u32,
        /// Type index for the lifted function
        type_idx: u32,
        /// Options for lifting
        options: LiftOptions,
    },
    /// Lower a component function to the core ABI
    Lower {
        /// Component function index
        func_idx: u32,
        /// Options for lowering
        options: LowerOptions,
    },
    /// Resource operations
    Resource(ResourceOperation),
    /// Reallocation operation
    Realloc {
        /// Function index for memory allocation
        alloc_func_idx: u32,
        /// Memory index to use
        memory_idx: u32,
    },
    /// Post-return operation (cleanup)
    PostReturn {
        /// Function index for post-return cleanup
        func_idx: u32,
    },
    /// Memory copy operation (optional optimization)
    MemoryCopy {
        /// Source memory index
        src_memory_idx: u32,
        /// Destination memory index
        dst_memory_idx: u32,
        /// Function index for the copy operation
        func_idx: u32,
    },
    /// Async operation (stackful lift)
    Async {
        /// Function index for the async operation
        func_idx: u32,
        /// Type index for the async function
        type_idx: u32,
        /// Options for async operations
        options: AsyncOptions,
    },
}

/// Options for lifting operations
#[derive(Debug, Clone)]
pub struct LiftOptions {
    /// Memory index to use for string/list conversions
    pub memory_idx: Option<u32>,
    /// String encoding to use
    pub string_encoding: Option<StringEncoding>,
    /// Realloc function index (optional)
    pub realloc_func_idx: Option<u32>,
    /// Post-return function index (optional)
    pub post_return_func_idx: Option<u32>,
    /// Whether this is an async lift
    pub is_async: bool,
}

/// Options for lowering operations
#[derive(Debug, Clone)]
pub struct LowerOptions {
    /// Memory index to use for string/list conversions
    pub memory_idx: Option<u32>,
    /// String encoding to use
    pub string_encoding: Option<StringEncoding>,
    /// Realloc function index (optional)
    pub realloc_func_idx: Option<u32>,
    /// Whether this is an async lower
    pub is_async: bool,
    /// Error handling mode
    pub error_mode: Option<ErrorMode>,
}

/// Options for async operations
#[derive(Debug, Clone)]
pub struct AsyncOptions {
    /// Memory index to use
    pub memory_idx: u32,
    /// Realloc function index
    pub realloc_func_idx: Option<u32>,
    /// String encoding to use
    pub string_encoding: Option<StringEncoding>,
}

/// String encoding options
#[derive(Debug, Clone)]
pub enum StringEncoding {
    /// UTF-8 encoding
    UTF8,
    /// UTF-16 encoding
    UTF16,
    /// Latin-1 encoding
    Latin1,
    /// ASCII encoding
    ASCII,
}

/// Error handling modes
#[derive(Debug, Clone)]
pub enum ErrorMode {
    /// Convert errors to exceptions
    ThrowOnError,
    /// Convert exceptions to errors
    CatchExceptions,
    /// Pass through errors/exceptions
    Passthrough,
}

/// Resource operation for canonical ABI
#[derive(Debug, Clone)]
pub enum ResourceOperation {
    /// Resource new: converts a constructor to an implementation
    New(ResourceNew),
    /// Resource drop: drops a resource without destroying it
    Drop(ResourceDrop),
    /// Resource rep: converts a resource to its representation
    Rep(ResourceRep),
    /// Resource destroy: destroys a resource
    Destroy(ResourceDestroy),
    /// Resource transfer: transfers ownership of a resource
    Transfer(ResourceTransfer),
    /// Resource borrow: temporarily borrows a resource
    Borrow(ResourceBorrow),
}

/// Resource new operation data
#[derive(Debug, Clone)]
pub struct ResourceNew {
    /// Type index of the resource
    pub type_idx: u32,
    /// Memory index for resource representation (if any)
    pub memory_idx: Option<u32>,
}

/// Resource drop operation data
#[derive(Debug, Clone)]
pub struct ResourceDrop {
    /// Type index of the resource
    pub type_idx: u32,
}

/// Resource rep operation data
#[derive(Debug, Clone)]
pub struct ResourceRep {
    /// Type index of the resource
    pub type_idx: u32,
    /// Memory index for resource representation (if any)
    pub memory_idx: Option<u32>,
}

/// Resource destroy operation data
#[derive(Debug, Clone)]
pub struct ResourceDestroy {
    /// Type index of the resource
    pub type_idx: u32,
}

/// Resource transfer operation data
#[derive(Debug, Clone)]
pub struct ResourceTransfer {
    /// Type index of the resource
    pub type_idx: u32,
}

/// Resource borrow operation data
#[derive(Debug, Clone)]
pub struct ResourceBorrow {
    /// Type index of the resource
    pub type_idx: u32,
    /// Borrow duration (in ticks)
    pub duration: Option<u32>,
}

/// Component start function
#[derive(Debug, Clone)]
pub struct Start {
    /// Function index
    pub func_idx: u32,
    /// Value arguments
    pub args: Vec<u32>,
    /// Number of results
    pub results: u32,
}

/// Import definition in a component
#[derive(Debug, Clone)]
pub struct Import {
    /// Import name in namespace.name format
    pub name: ImportName,
    /// Type of the import
    pub ty: ExternType,
}

/// Import name with support for nested namespaces
#[derive(Debug, Clone)]
pub struct ImportName {
    /// Namespace of the import
    pub namespace: String,
    /// Name of the import
    pub name: String,
    /// Nested namespaces (if any)
    pub nested: Vec<String>,
    /// Package reference (if any)
    pub package: Option<PackageReference>,
}

/// Package reference for imports
#[derive(Debug, Clone)]
pub struct PackageReference {
    /// Package name
    pub name: String,
    /// Package version
    pub version: Option<String>,
    /// Package hash (for content verification)
    pub hash: Option<String>,
}

/// Export definition in a component
#[derive(Debug, Clone)]
pub struct Export {
    /// Export name in "name" format
    pub name: ExportName,
    /// Sort of the exported item
    pub sort: Sort,
    /// Index within the sort
    pub idx: u32,
    /// Declared type (optional)
    pub ty: Option<ExternType>,
}

/// Export name with support for nested namespaces
#[derive(Debug, Clone)]
pub struct ExportName {
    /// Basic name
    pub name: String,
    /// Whether this export is a resource
    pub is_resource: bool,
    /// Semver compatibility string
    pub semver: Option<String>,
    /// Integrity hash for content verification
    pub integrity: Option<String>,
    /// Nested namespaces (if any)
    pub nested: Vec<String>,
}

impl ImportName {
    /// Create a new import name with just namespace and name
    pub fn new(namespace: String, name: String) -> Self {
        Self {
            namespace,
            name,
            nested: Vec::new(),
            package: None,
        }
    }

    /// Create a new import name with nested namespaces
    pub fn with_nested(namespace: String, name: String, nested: Vec<String>) -> Self {
        Self {
            namespace,
            name,
            nested,
            package: None,
        }
    }

    /// Add package reference to an import name
    pub fn with_package(mut self, package: PackageReference) -> Self {
        self.package = Some(package);
        self
    }

    /// Get the full import path as a string
    pub fn full_path(&self) -> String {
        let mut path = format!("{}.{}", self.namespace, self.name);
        for nested in &self.nested {
            path.push_str(&format!(".{}", nested));
        }
        path
    }
}

impl ExportName {
    /// Create a new export name
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_resource: false,
            semver: None,
            integrity: None,
            nested: Vec::new(),
        }
    }

    /// Create a new export name with nested namespaces
    pub fn with_nested(name: String, nested: Vec<String>) -> Self {
        Self {
            name,
            is_resource: false,
            semver: None,
            integrity: None,
            nested,
        }
    }

    /// Mark as a resource export
    pub fn as_resource(mut self) -> Self {
        self.is_resource = true;
        self
    }

    /// Add semver compatibility information
    pub fn with_semver(mut self, semver: String) -> Self {
        self.semver = Some(semver);
        self
    }

    /// Add integrity hash for content verification
    pub fn with_integrity(mut self, integrity: String) -> Self {
        self.integrity = Some(integrity);
        self
    }

    /// Get the full export path as a string
    pub fn full_path(&self) -> String {
        let mut path = self.name.clone();
        for nested in &self.nested {
            path.push_str(&format!(".{}", nested));
        }
        path
    }
}

/// Component value definition
#[derive(Debug, Clone)]
pub struct Value {
    /// Type of the value
    pub ty: ValType,
    /// Encoded value data
    pub data: Vec<u8>,
    /// Value expression (if available)
    pub expression: Option<ValueExpression>,
    /// Value name (if available from custom sections)
    pub name: Option<String>,
}

/// Value expression types
#[derive(Debug, Clone)]
pub enum ValueExpression {
    /// Reference to an item in component
    ItemRef {
        /// Sort of the referenced item
        sort: Sort,
        /// Index within the sort
        idx: u32,
    },
    /// Global initialization expression
    GlobalInit {
        /// Global index
        global_idx: u32,
    },
    /// Function call expression
    FunctionCall {
        /// Function index
        func_idx: u32,
        /// Arguments (indices to other values)
        args: Vec<u32>,
    },
    /// Direct constant value
    Const(ConstValue),
}

/// Constant value types
#[derive(Debug, Clone)]
pub enum ConstValue {
    /// Boolean constant
    Bool(bool),
    /// 8-bit signed integer
    S8(i8),
    /// 8-bit unsigned integer
    U8(u8),
    /// 16-bit signed integer
    S16(i16),
    /// 16-bit unsigned integer
    U16(u16),
    /// 32-bit signed integer
    S32(i32),
    /// 32-bit unsigned integer
    U32(u32),
    /// 64-bit signed integer
    S64(i64),
    /// 64-bit unsigned integer
    U64(u64),
    /// 32-bit float
    F32(f32),
    /// 64-bit float
    F64(f64),
    /// Character constant
    Char(char),
    /// String constant
    String(String),
    /// Empty null constant
    Null,
}
