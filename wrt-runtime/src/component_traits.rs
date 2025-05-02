use crate::func::FuncType;
use wrt_error::Result;
use wrt_types::{ComponentType, ExternType, Value, Vec};

/// Represents a runtime component instance
pub trait ComponentInstance {
    /// Execute a function by name with the given arguments
    fn execute_function(&self, name: &str, args: &[Value]) -> Result<Vec<Value>>;

    /// Read from exported memory
    fn read_memory(&self, name: &str, offset: u32, size: u32) -> Result<Vec<u8>>;

    /// Write to exported memory
    fn write_memory(&mut self, name: &str, offset: u32, bytes: &[u8]) -> Result<()>;

    /// Get the type of an export
    fn get_export_type(&self, name: &str) -> Result<ExternType>;
}

/// Represents a host function implementation
pub trait HostFunction {
    /// Call the host function with the given arguments
    fn call(&self, args: &[Value]) -> Result<Vec<Value>>;

    /// Get the function's type
    fn get_type(&self) -> FuncType;
}

/// Represents a host function factory
pub trait HostFunctionFactory {
    /// Create a host function implementation
    fn create_function(&self, name: &str, ty: &FuncType) -> Result<Box<dyn HostFunction>>;
}

/// Represents a component runtime environment
pub trait ComponentRuntime {
    /// Create a new runtime instance
    fn new() -> Self
    where
        Self: Sized;

    /// Register a host function factory
    fn register_host_factory(&mut self, factory: Box<dyn HostFunctionFactory>);

    /// Instantiate a component
    fn instantiate(&self, component_type: &ComponentType) -> Result<Box<dyn ComponentInstance>>;

    /// Register a specific host function
    fn register_host_function<F>(&mut self, name: &str, ty: FuncType, function: F) -> Result<()>
    where
        F: Fn(&[Value]) -> Result<Vec<Value>> + 'static + Send + Sync;
}
