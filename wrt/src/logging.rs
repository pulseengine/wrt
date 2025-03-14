use crate::{Box, String};

#[cfg(not(feature = "std"))]
use core::str::FromStr;
#[cfg(feature = "std")]
use std::str::FromStr;

#[cfg(not(feature = "std"))]
use core::result::Result;
#[cfg(feature = "std")]
use std::result::Result;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

/// Log levels for WebAssembly component logging
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    /// Trace level (most verbose)
    Trace,
    /// Debug level
    Debug,
    /// Info level (standard)
    Info,
    /// Warning level
    Warn,
    /// Error level
    Error,
    /// Critical level (most severe)
    Critical,
}

/// Error type for log level parsing errors
#[derive(Debug, Clone, PartialEq)]
pub struct ParseLogLevelError {
    /// The invalid level string
    invalid_level: String,
}

#[cfg(feature = "std")]
impl std::error::Error for ParseLogLevelError {}

#[cfg(feature = "std")]
impl std::fmt::Display for ParseLogLevelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid log level: {}", self.invalid_level)
    }
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for ParseLogLevelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Invalid log level: {}", self.invalid_level)
    }
}

impl FromStr for LogLevel {
    type Err = ParseLogLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            "critical" => Ok(LogLevel::Critical),
            _ => Err(ParseLogLevelError {
                invalid_level: s.to_string(),
            }),
        }
    }
}

impl LogLevel {
    /// Creates a LogLevel from a string, defaulting to Info for invalid levels
    pub fn from_string_or_default(s: &str) -> Self {
        Self::from_str(s).unwrap_or(LogLevel::Info)
    }

    /// Convert LogLevel to a string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Critical => "critical",
        }
    }
}

/// Log operation from a WebAssembly component
#[derive(Debug, Clone)]
pub struct LogOperation {
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Component ID (optional)
    pub component_id: Option<String>,
}

impl LogOperation {
    /// Create a new log operation
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            level,
            message,
            component_id: None,
        }
    }

    /// Create a new log operation with a component ID
    pub fn with_component(level: LogLevel, message: String, component_id: String) -> Self {
        Self {
            level,
            message,
            component_id: Some(component_id),
        }
    }
}

/// Log handler type for processing WebAssembly log operations
pub type LogHandler = Box<dyn Fn(LogOperation) + Send + Sync>;

/// A callback registry for handling WebAssembly component operations
#[derive(Default)]
pub struct CallbackRegistry {
    /// Log handler (if registered)
    #[allow(clippy::type_complexity)]
    log_handler: Option<LogHandler>,
}

#[cfg(feature = "std")]
// Manual Debug implementation since function pointers don't implement Debug
impl std::fmt::Debug for CallbackRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackRegistry")
            .field("has_log_handler", &self.has_log_handler())
            .finish()
    }
}

#[cfg(not(feature = "std"))]
// Manual Debug implementation since function pointers don't implement Debug
impl core::fmt::Debug for CallbackRegistry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CallbackRegistry")
            .field("has_log_handler", &self.has_log_handler())
            .finish()
    }
}

impl CallbackRegistry {
    /// Create a new callback registry
    pub fn new() -> Self {
        Self { log_handler: None }
    }

    /// Register a log handler
    pub fn register_log_handler<F>(&mut self, handler: F)
    where
        F: Fn(LogOperation) + Send + Sync + 'static,
    {
        self.log_handler = Some(Box::new(handler));
    }

    /// Handle a log operation
    pub fn handle_log(&self, operation: LogOperation) {
        if let Some(handler) = &self.log_handler {
            handler(operation);
        }
    }

    /// Check if a log handler is registered
    pub fn has_log_handler(&self) -> bool {
        self.log_handler.is_some()
    }
}
