//! Test no_std compatibility for wrt-logging
//!
//! This file validates that the wrt-logging crate works correctly in all
//! environments: std, no_std with alloc, and pure no_std.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Core imports for all configurations
#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(feature = "std")]
use std::fmt;

/// All tests that work in all three configurations (pure no_std, alloc, std)
#[cfg(test)]
mod universal_tests {
    use wrt_logging::{
        level::LogLevel,
        minimal_handler::{MinimalLogHandler, MinimalLogMessage},
        Result,
    };

    #[test]
    fn test_log_level_ordering() {
        // Test log level ordering
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Critical > LogLevel::Error);
        assert!(LogLevel::Trace < LogLevel::Debug);
    }

    #[test]
    fn test_log_level_as_str() {
        // Test log level string conversion
        assert_eq!(LogLevel::Trace.as_str(), "trace");
        assert_eq!(LogLevel::Debug.as_str(), "debug");
        assert_eq!(LogLevel::Info.as_str(), "info");
        assert_eq!(LogLevel::Warn.as_str(), "warn");
        assert_eq!(LogLevel::Error.as_str(), "error");
        assert_eq!(LogLevel::Critical.as_str(), "critical");
    }

    #[test]
    fn test_log_level_copy_safety() {
        // In all environments, we should be able to safely copy LogLevel
        let original = LogLevel::Debug;
        let copy = original;

        // Both should be valid and equal
        assert_eq!(original, copy);
        assert_eq!(original.as_str(), "debug");
        assert_eq!(copy.as_str(), "debug");
    }

    #[test]
    fn test_minimal_log_message() {
        // Test minimal log message that works in pure no_std
        let msg = MinimalLogMessage::new(LogLevel::Info, "static message");
        assert_eq!(msg.level, LogLevel::Info);
        assert_eq!(msg.message, "static message");
    }

    #[test]
    fn test_minimal_log_handler() {
        // Create a minimal log handler implementation
        struct TestMinimalHandler {
            last_level: Option<LogLevel>,
            last_message: Option<&'static str>,
        }

        impl MinimalLogHandler for TestMinimalHandler {
            fn handle_minimal_log(&self, level: LogLevel, message: &'static str) -> Result<()> {
                // Mutate through interior mutability in a real implementation
                // Here we're just testing the trait interface
                let this = unsafe { &mut *(self as *const Self as *mut Self) };
                this.last_level = Some(level);
                this.last_message = Some(message);
                Ok(())
            }
        }

        let handler = TestMinimalHandler { last_level: None, last_message: None };

        // Log a message
        let _ = handler.handle_minimal_log(LogLevel::Error, "error message");

        // Since we can't use interior mutability properly in this test,
        // we're using unsafe to verify the trait works as expected
        unsafe {
            let handler_mut =
                &mut *((&handler) as *const TestMinimalHandler as *mut TestMinimalHandler);
            assert_eq!(handler_mut.last_level, Some(LogLevel::Error));
            assert_eq!(handler_mut.last_message, Some("error message"));
        }
    }
}

/// Tests that require alloc or std
#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod alloc_tests {
    // Import necessary types for no_std environment
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{boxed::Box, format, string::String, vec::Vec};
    #[cfg(not(feature = "std"))]
    use core::str::FromStr;
    #[cfg(feature = "std")]
    use std::str::FromStr;
    #[cfg(feature = "std")]
    use std::{boxed::Box, format, string::String, vec::Vec};

    // Import from wrt-host
    use wrt_host::CallbackRegistry;
    // Import from wrt-logging
    use wrt_logging::{LogHandler, LogLevel, LogOperation, LoggingExt};

    #[test]
    fn test_log_level_string_operations() {
        // Test string operations which require alloc
        let level_str = LogLevel::Warn.as_str();
        let formatted = format!("Level: {}", level_str);
        assert_eq!(formatted, "Level: warn");

        // Test FromStr implementation
        let parsed = LogLevel::from_str("warning").unwrap();
        assert_eq!(parsed, LogLevel::Warn);

        let error = LogLevel::from_str("invalid").unwrap_err();
        assert_eq!(error.invalid_level, "invalid");
    }

    #[test]
    fn test_log_level_from_string_or_default() {
        assert_eq!(LogLevel::from_string_or_default("debug"), LogLevel::Debug);
        assert_eq!(LogLevel::from_string_or_default("invalid"), LogLevel::Info);
    }

    // Test LogOperation with alloc feature
    #[test]
    fn test_log_operation() {
        // Create a log operation
        let op = LogOperation::new(LogLevel::Info, "test message".to_string());
        assert_eq!(op.level, LogLevel::Info);
        assert_eq!(op.message, "test message");
        assert!(op.component_id.is_none());
    }

    // Test with component ID
    #[test]
    fn test_log_operation_with_component() {
        let op = LogOperation::with_component(LogLevel::Debug, "test message", "component-1");
        assert_eq!(op.level, LogLevel::Debug);
        assert_eq!(op.message, "test message");
        assert_eq!(op.component_id, Some("component-1".to_string()));
    }

    // Test registry creation and operations
    #[test]
    fn test_callback_registry() {
        let mut registry = CallbackRegistry::new();
        assert!(!registry.has_log_handler());

        // Different synchronization for different environments
        #[cfg(feature = "std")]
        let log_messages = {
            use std::sync::{Arc, Mutex};
            Arc::new(Mutex::new(Vec::new()))
        };

        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        let log_messages = {
            use core::cell::RefCell;
            RefCell::new(Vec::new())
        };

        // Register logging handler
        #[cfg(feature = "std")]
        {
            let messages = log_messages.clone();
            registry.register_log_handler(move |log_op| {
                messages.lock().unwrap().push((log_op.level, log_op.message.clone()));
            });
        }

        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        {
            let messages = &log_messages;
            registry.register_log_handler(move |log_op| {
                messages.borrow_mut().push((log_op.level, log_op.message.clone()));
            });
        }

        assert!(registry.has_log_handler());

        // Send log messages
        registry.handle_log(LogOperation::new(LogLevel::Info, "test info".to_string()));
        registry.handle_log(LogOperation::new(LogLevel::Error, "test error".to_string()));

        // Verify messages were logged
        #[cfg(feature = "std")]
        {
            let messages = log_messages.lock().unwrap();
            assert_eq!(messages.len(), 2);
            assert_eq!(messages[0].0, LogLevel::Info);
            assert_eq!(messages[0].1, "test info");
            assert_eq!(messages[1].0, LogLevel::Error);
            assert_eq!(messages[1].1, "test error");
        }

        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        {
            let messages = log_messages.borrow();
            assert_eq!(messages.len(), 2);
            assert_eq!(messages[0].0, LogLevel::Info);
            assert_eq!(messages[0].1, "test info");
            assert_eq!(messages[1].0, LogLevel::Error);
            assert_eq!(messages[1].1, "test error");
        }
    }
}

/// Tests that are only run in std configuration
#[cfg(test)]
#[cfg(feature = "std")]
mod std_tests {
    use std::error::Error;

    use wrt_logging::level::ParseLogLevelError;

    #[test]
    fn test_error_trait_implementation() {
        // Test std::error::Error implementation (std only)
        let error = ParseLogLevelError { invalid_level: "invalid".to_string() };
        let error_ref: &dyn Error = &error;
        assert!(error_ref.source().is_none());
    }
}
