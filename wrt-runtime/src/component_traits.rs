use crate::{func::FuncType, prelude::*};
use wrt_foundation::{
    safe_memory::{SafeStack, SafeSlice},
    Value, VerificationLevel,
};

// Type aliases with proper memory provider
pub type ComponentType = wrt_foundation::component::ComponentType<wrt_foundation::safe_memory::NoStdProvider<1024>>;
pub type ExternType = wrt_foundation::component::ExternType<wrt_foundation::safe_memory::NoStdProvider<1024>>;
pub type SafeStackValue = wrt_foundation::safe_memory::SafeStack<Value, 64, wrt_foundation::safe_memory::NoStdProvider<1024>>;

/// Represents a runtime component instance
pub trait ComponentInstance {
    /// Execute a function by name with the given arguments
    fn execute_function(&self, name: &str, args: &[Value]) -> Result<SafeStackValue>;

    /// Read from exported memory
    fn read_memory(&self, name: &str, offset: u32, size: u32) -> Result<SafeSlice<'_>>;

    /// Write to exported memory
    fn write_memory(&mut self, name: &str, offset: u32, bytes: &[u8]) -> Result<()>;

    /// Get the type of an export
    fn get_export_type(&self, name: &str) -> Result<ExternType>;

    /// Execute a function by name with the given arguments (legacy `Vec` API)
    #[deprecated(since = "0.2.0", note = "Use execute_function with SafeStack instead")]
    fn execute_function_vec(&self, name: &str, args: &[Value]) -> Result<Vec<Value>> {
        // Convert from the new SafeStack API to the legacy Vec API
        let safe_stack = self.execute_function(name, args)?;
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let mut vec = Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap();
            while let Ok(Some(value)) = safe_stack.pop() {
                vec.push(value);
            }
            vec.reverse(); // SafeStack pops in reverse order
            Ok(vec)
        }
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        {
            // In no_std mode without alloc, we can't create Vec
            Err(Error::new(ErrorCategory::Runtime, codes::UNSUPPORTED_OPERATION, "Vector operations not supported in no_std mode without alloc"))
        }
    }

    /// Read from exported memory (legacy `Vec` API)
    #[cfg(any(feature = "std", feature = "alloc"))]
    #[deprecated(since = "0.2.0", note = "Use read_memory with SafeSlice instead")]
    fn read_memory_vec(&self, name: &str, offset: u32, size: u32) -> Result<Vec<u8>> {
        // Convert from the new SafeSlice API to the legacy Vec API
        let safe_slice = self.read_memory(name, offset, size)?;
        let data = safe_slice.data()?;
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            Ok(data.to_vec())
        }
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        {
            // In no_std mode without alloc, we can't create Vec
            Err(Error::new(ErrorCategory::Runtime, codes::UNSUPPORTED_OPERATION, "Vector operations not supported in no_std mode without alloc"))
        }
    }
}

/// Represents a host function implementation
pub trait HostFunction {
    /// Call the host function with the given arguments
    fn call(&self, args: &[Value]) -> Result<SafeStackValue>;

    /// Get the function's type
    fn get_type(&self) -> FuncType;

    /// Call the host function with the given arguments (legacy `Vec` API)
    #[deprecated(since = "0.2.0", note = "Use call with SafeStack instead")]
    fn call_vec(&self, args: &[Value]) -> Result<Vec<Value>> {
        // Convert from the new SafeStack API to the legacy Vec API
        let safe_stack = self.call(args)?;
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let mut vec = Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap();
            // Convert SafeStack to Vec by popping all values
            let mut stack_copy = safe_stack;
            while let Ok(Some(value)) = stack_copy.pop() {
                vec.push(value);
            }
            vec.reverse(); // SafeStack pops in reverse order
            Ok(vec)
        }
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        {
            // In no_std mode without alloc, we can't create Vec
            Err(Error::new(ErrorCategory::Runtime, codes::UNSUPPORTED_OPERATION, "Vector operations not supported in no_std mode without alloc"))
        }
    }
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
        F: Fn(&[Value]) -> Result<wrt_foundation::bounded::BoundedVec<Value, 16, wrt_foundation::safe_memory::NoStdProvider<1024>>> + 'static + Send + Sync;

    /// Set the verification level for memory operations
    fn set_verification_level(&mut self, level: VerificationLevel) -> Result<()>;

    /// Get the current verification level
    fn verification_level(&self) -> VerificationLevel;
}
