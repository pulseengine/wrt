//! Test no_std compatibility for wrt-intercept
//!
//! This file validates that the wrt-intercept crate works correctly in no_std environments.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(test)]
mod tests {
    // Import necessary types for no_std environment
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{format, string::String, vec, vec::Vec};

    #[cfg(feature = "std")]
    use std::{string::String, vec, vec::Vec};

    // Import from wrt-intercept
    use wrt_intercept::{
        builtins::{
            BeforeBuiltinResult, BuiltinInterceptor, BuiltinSerialization, InterceptContext,
        },
        strategies::{FirewallConfig, FirewallRule, FirewallStrategy, LoggingStrategy},
        InterceptionResult, LinkInterceptor, LinkInterceptorStrategy, Modification,
    };

    // Import from wrt-types
    use wrt_types::{builtin::BuiltinType, component_value::ComponentValue, values::Value};

    #[test]
    fn test_firewall_strategy() {
        // Create a firewall rule
        let rule = FirewallRule::new("test_function".to_string(), true);

        // Check rule properties
        assert_eq!(rule.name(), "test_function");
        assert_eq!(rule.allow(), true);

        // Create a firewall config
        let mut config = FirewallConfig::new();
        config.add_rule(rule);

        // Check config
        assert_eq!(config.rules().len(), 1);

        // Create a firewall strategy
        let strategy = FirewallStrategy::new(config);

        // The strategy should implement LinkInterceptorStrategy
        assert!(strategy.name().contains("Firewall"));
    }

    #[test]
    fn test_logging_strategy() {
        // Create a logging strategy
        let strategy = LoggingStrategy::new();

        // The strategy should implement LinkInterceptorStrategy
        assert!(strategy.name().contains("Logging"));
    }

    #[test]
    fn test_intercept_context() {
        // Create an intercept context
        let context = InterceptContext::new(
            "test_function".to_string(),
            BuiltinType::Memory,
            vec![Value::I32(42)],
        );

        // Check context properties
        assert_eq!(context.function_name(), "test_function");
        assert_eq!(context.builtin_type(), BuiltinType::Memory);
        assert_eq!(context.parameters().len(), 1);
        assert_eq!(context.parameters()[0], Value::I32(42));
    }

    #[test]
    fn test_builtin_serialization() {
        // Test serialization trait methods

        // Test component value conversion to/from Value
        let i32_value = Value::I32(42);
        let component_value = ComponentValue::I32(42);

        // Convert from Value to ComponentValue
        assert_eq!(
            BuiltinSerialization::value_to_component_value(&i32_value),
            Ok(component_value.clone())
        );

        // Convert from ComponentValue to Value
        assert_eq!(
            BuiltinSerialization::component_value_to_value(&component_value),
            Ok(i32_value)
        );
    }

    #[test]
    fn test_modification() {
        // Test modification enum
        let allow = Modification::Allow;
        let deny = Modification::Deny("Access denied".to_string());
        let replace = Modification::Replace(Value::I32(42));

        // Check they're different
        assert_ne!(format!("{:?}", allow), format!("{:?}", deny));
        assert_ne!(format!("{:?}", deny), format!("{:?}", replace));
    }

    #[test]
    fn test_before_builtin_result() {
        // Test BeforeBuiltinResult enum
        let continue_result = BeforeBuiltinResult::Continue;
        let replace_result = BeforeBuiltinResult::Replace(Value::I32(42));

        // Check they're different
        assert_ne!(
            format!("{:?}", continue_result),
            format!("{:?}", replace_result)
        );
    }
}
