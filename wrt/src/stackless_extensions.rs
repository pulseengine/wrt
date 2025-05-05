// Stackless Extensions Module
// This module extends the stackless execution engine functionality

use crate::execution::ExecutionStats;
use crate::instructions::instruction_type::Instruction;
use crate::stackless::{StacklessEngine, StacklessFrame};
use wrt_error::Error;

// Basic extension traits and functionality
pub trait StacklessExtension {
    fn execute(&self, engine: &mut StacklessEngine) -> Result<ExecutionResult, Error>;
}
