// Stackless Extensions Module
// This module extends the stackless execution engine functionality

use wrt_runtime::stackless::{StacklessEngine, StacklessExecutionState};
use wrt_error::{Error, Result};

/// Result of executing a stackless extension
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Extension execution completed successfully
    Completed,
    /// Extension execution is paused and can be resumed
    Paused,
    /// Extension execution returned a value
    Value(Vec<wrt_foundation::values::Value>),
    /// Extension execution encountered an error
    Error(String),
}

/// Trait for extensions to the stackless execution engine
pub trait StacklessExtension {
    /// Execute the extension with the given engine
    fn execute(&self, engine: &mut StacklessEngine) -> Result<ExecutionResult>;
}
