//! Verification module for the Kani model checker
//!
//! This module contains verification functions and proofs
//! for the Kani model checker to verify the correctness of
//! the interceptor implementation.

#[cfg(feature = "kani")]
mod proofs {
    use crate::{LinkInterceptor, LinkInterceptorStrategy};
    use std::sync::Arc;
    use wrt_error::Result;
    use wrt_types::values::Value;

    /// A simple strategy for verification
    struct TestStrategy {
        modify_args: bool,
    }

    impl LinkInterceptorStrategy for TestStrategy {
        fn before_call(
            &self,
            _source: &str,
            _target: &str,
            _function: &str,
            args: &[Value],
        ) -> Result<Vec<Value>> {
            if self.modify_args {
                Ok(vec![Value::I32(42)])
            } else {
                Ok(args.to_vec())
            }
        }

        fn after_call(
            &self,
            _source: &str,
            _target: &str,
            _function: &str,
            _args: &[Value],
            result: Result<Vec<Value>>,
        ) -> Result<Vec<Value>> {
            result
        }

        fn should_bypass(&self) -> bool {
            false
        }

        fn clone_strategy(&self) -> Arc<dyn LinkInterceptorStrategy> {
            Arc::new(Self {
                modify_args: self.modify_args,
            })
        }
    }

    /// Verify that the interceptor properly modifies arguments
    #[kani::proof]
    fn verify_interceptor_modifies_args() {
        let strategy = Arc::new(TestStrategy { modify_args: true });
        let mut interceptor = LinkInterceptor::new("test");
        interceptor.add_strategy(strategy);

        let args = vec![Value::I32(10)];
        let result = interceptor.intercept_call("target", "func", args.clone(), |modified_args| {
            // The strategy should have modified the args
            kani::assert(modified_args.len() == 1);
            kani::assert(matches!(modified_args[0], Value::I32(42)));
            Ok(vec![Value::I64(20)])
        });

        kani::assert(result.is_ok());
    }

    /// Verify that the interceptor passes through arguments when not modified
    #[kani::proof]
    fn verify_interceptor_passthrough() {
        let strategy = Arc::new(TestStrategy { modify_args: false });
        let mut interceptor = LinkInterceptor::new("test");
        interceptor.add_strategy(strategy);

        let args = vec![Value::I32(10)];
        let result = interceptor.intercept_call("target", "func", args.clone(), |modified_args| {
            // The strategy should not have modified the args
            kani::assert(modified_args.len() == args.len());
            kani::assert(matches!(modified_args[0], Value::I32(10)));
            Ok(vec![Value::I64(20)])
        });

        kani::assert(result.is_ok());
    }

    /// Verify that multiple strategies are applied in order
    #[kani::proof]
    fn verify_multiple_strategies() {
        let strategy1 = Arc::new(TestStrategy { modify_args: true });
        let strategy2 = Arc::new(TestStrategy { modify_args: false });

        let mut interceptor = LinkInterceptor::new("test");
        interceptor.add_strategy(strategy1);
        interceptor.add_strategy(strategy2);

        let args = vec![Value::I32(10)];
        let result = interceptor.intercept_call("target", "func", args.clone(), |modified_args| {
            // The first strategy should have modified the args
            // The second strategy should have passed them through
            kani::assert(modified_args.len() == 1);
            kani::assert(matches!(modified_args[0], Value::I32(42)));
            Ok(vec![Value::I64(20)])
        });

        kani::assert(result.is_ok());
    }

    /// Verify that the interceptor passes errors through
    #[kani::proof]
    fn verify_error_passthrough() {
        let strategy = Arc::new(TestStrategy { modify_args: false });
        let mut interceptor = LinkInterceptor::new("test");
        interceptor.add_strategy(strategy);

        let args = vec![Value::I32(10)];
        let result = interceptor.intercept_call("target", "func", args.clone(), |_| {
            Err(wrt_error::Error::new(wrt_error::kinds::ExecutionError(
                "Test error".to_string(),
            )))
        });

        kani::assert(result.is_err());
    }
}
