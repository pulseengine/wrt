# Architectural Issues Analysis
Date: 2025-06-20 13:44:59

## Analyzing failure for: WRT ASIL-B (no_std + alloc)
Features: alloc, safety-asil-b

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
   Compiling wrt v0.2.0 (/Users/r/git/wrt2/wrt)
warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:575:34
    |
575 | impl<const N: usize> Default for NoStdProviderBuilder<N> {
    |                                  ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:581:22
    |
581 | impl<const N: usize> NoStdProviderBuilder<N> {
    |                      ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:643:18
    |
643 | impl Default for NoStdProviderBuilder1 {
    |                  ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:649:6
    |
649 | impl NoStdProviderBuilder1 {
    |      ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated module `safe_allocation::safe_factories`: Use capability_factories for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:260:9
    |
260 | pub use safe_factories::*;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:195:24
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:205:24
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:577:16
    |
577 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder1::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:645:16
    |
645 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:45:17
   |
45 |         context.create_provider::<N>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:78:17
   |
78 |         context.create_provider::<SIZE>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:96:17
   |
96 |         context.create_provider::<SIZE>(self.crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:195:45
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:205:45
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_memory::NoStdProvider::<N>::new`: Use BudgetProvider::new() or create_provider! macro for budget-aware allocation
   --> wrt-foundation/src/capabilities/dynamic.rs:210:57
    |
210 |             crate::safe_memory::NoStdProvider::<65536>::new()
    |                                                         ^^^

warning: `wrt-foundation` (lib) generated 15 warnings
warning: unused import: `wrt_error::Error`
 --> wrt-intercept/src/builtins.rs:8:5
  |
8 | use wrt_error::Error;
  |     ^^^^^^^^^^^^^^^^
  |
```

## Analyzing failure for: WRT Development (std)
Features: std

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
   Compiling wrt v0.2.0 (/Users/r/git/wrt2/wrt)
warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:575:34
    |
575 | impl<const N: usize> Default for NoStdProviderBuilder<N> {
    |                                  ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:581:22
    |
581 | impl<const N: usize> NoStdProviderBuilder<N> {
    |                      ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:643:18
    |
643 | impl Default for NoStdProviderBuilder1 {
    |                  ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:649:6
    |
649 | impl NoStdProviderBuilder1 {
    |      ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated module `safe_allocation::safe_factories`: Use capability_factories for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:260:9
    |
260 | pub use safe_factories::*;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:195:24
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:205:24
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:577:16
    |
577 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder1::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:645:16
    |
645 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:45:17
   |
45 |         context.create_provider::<N>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:78:17
   |
78 |         context.create_provider::<SIZE>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:96:17
   |
96 |         context.create_provider::<SIZE>(self.crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:195:45
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:205:45
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_memory::NoStdProvider::<N>::new`: Use BudgetProvider::new() or create_provider! macro for budget-aware allocation
   --> wrt-foundation/src/capabilities/dynamic.rs:210:57
    |
210 |             crate::safe_memory::NoStdProvider::<65536>::new()
    |                                                         ^^^

warning: `wrt-foundation` (lib) generated 15 warnings
   Compiling wrt-decoder v0.2.0 (/Users/r/git/wrt2/wrt-decoder)
warning: unused import: `BoundedVec`
  --> wrt-instructions/src/const_expr.rs:13:40
   |
13 | use crate::prelude::{Debug, PartialEq, BoundedVec};
   |                                        ^^^^^^^^^^
```

## Analyzing failure for: WRT Development with Optimization
Features: std, optimize

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
   Compiling wrt v0.2.0 (/Users/r/git/wrt2/wrt)
warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:575:34
    |
575 | impl<const N: usize> Default for NoStdProviderBuilder<N> {
    |                                  ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:581:22
    |
581 | impl<const N: usize> NoStdProviderBuilder<N> {
    |                      ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:643:18
    |
643 | impl Default for NoStdProviderBuilder1 {
    |                  ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:649:6
    |
649 | impl NoStdProviderBuilder1 {
    |      ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated module `safe_allocation::safe_factories`: Use capability_factories for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:260:9
    |
260 | pub use safe_factories::*;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:195:24
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:205:24
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:577:16
    |
577 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder1::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:645:16
    |
645 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:45:17
   |
45 |         context.create_provider::<N>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:78:17
   |
78 |         context.create_provider::<SIZE>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:96:17
   |
96 |         context.create_provider::<SIZE>(self.crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:195:45
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:205:45
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_memory::NoStdProvider::<N>::new`: Use BudgetProvider::new() or create_provider! macro for budget-aware allocation
   --> wrt-foundation/src/capabilities/dynamic.rs:210:57
    |
210 |             crate::safe_memory::NoStdProvider::<65536>::new()
    |                                                         ^^^

warning: `wrt-foundation` (lib) generated 15 warnings
   Compiling wrt-decoder v0.2.0 (/Users/r/git/wrt2/wrt-decoder)
warning: unused import: `BoundedVec`
  --> wrt-instructions/src/const_expr.rs:13:40
   |
13 | use crate::prelude::{Debug, PartialEq, BoundedVec};
   |                                        ^^^^^^^^^^
```

## Analyzing failure for: WRT Server
Features: std, optimize, platform

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
   Compiling wrt v0.2.0 (/Users/r/git/wrt2/wrt)
warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:575:34
    |
575 | impl<const N: usize> Default for NoStdProviderBuilder<N> {
    |                                  ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:581:22
    |
581 | impl<const N: usize> NoStdProviderBuilder<N> {
    |                      ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:643:18
    |
643 | impl Default for NoStdProviderBuilder1 {
    |                  ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:649:6
    |
649 | impl NoStdProviderBuilder1 {
    |      ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated module `safe_allocation::safe_factories`: Use capability_factories for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:260:9
    |
260 | pub use safe_factories::*;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:195:24
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:205:24
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:577:16
    |
577 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder1::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:645:16
    |
645 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:45:17
   |
45 |         context.create_provider::<N>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:78:17
   |
78 |         context.create_provider::<SIZE>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:96:17
   |
96 |         context.create_provider::<SIZE>(self.crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:195:45
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:205:45
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_memory::NoStdProvider::<N>::new`: Use BudgetProvider::new() or create_provider! macro for budget-aware allocation
   --> wrt-foundation/src/capabilities/dynamic.rs:210:57
    |
210 |             crate::safe_memory::NoStdProvider::<65536>::new()
    |                                                         ^^^

warning: `wrt-foundation` (lib) generated 15 warnings
warning: unused import: `BoundedVec`
  --> wrt-instructions/src/const_expr.rs:13:40
   |
13 | use crate::prelude::{Debug, PartialEq, BoundedVec};
   |                                        ^^^^^^^^^^
   |
```

## Analyzing failure for: WRTD ASIL-B Runtime
Features: safety-asil-b, wrt-execution, asil-b-panic

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
warning: wrt@0.2.0: Setting WASM_TESTSUITE to /Users/r/git/wrt2/target/debug/build/wrt-c3722ec280d4681b/out/testsuite
warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:575:34
    |
575 | impl<const N: usize> Default for NoStdProviderBuilder<N> {
    |                                  ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:581:22
    |
581 | impl<const N: usize> NoStdProviderBuilder<N> {
    |                      ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:643:18
    |
643 | impl Default for NoStdProviderBuilder1 {
    |                  ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:649:6
    |
649 | impl NoStdProviderBuilder1 {
    |      ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated module `safe_allocation::safe_factories`: Use capability_factories for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:260:9
    |
260 | pub use safe_factories::*;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:195:24
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:205:24
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:577:16
    |
577 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder1::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:645:16
    |
645 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:45:17
   |
45 |         context.create_provider::<N>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:78:17
   |
78 |         context.create_provider::<SIZE>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:96:17
   |
96 |         context.create_provider::<SIZE>(self.crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:195:45
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:205:45
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_memory::NoStdProvider::<N>::new`: Use BudgetProvider::new() or create_provider! macro for budget-aware allocation
   --> wrt-foundation/src/capabilities/dynamic.rs:210:57
    |
210 |             crate::safe_memory::NoStdProvider::<65536>::new()
    |                                                         ^^^

warning: `wrt-foundation` (lib) generated 15 warnings
   Compiling wrt-decoder v0.2.0 (/Users/r/git/wrt2/wrt-decoder)
warning: use of deprecated struct `wrt_foundation::safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
  --> wrt-logging/src/bounded_log_infra.rs:47:20
   |
47 |     let provider = safe_managed_alloc!(8192, CrateId::Logging)?;
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## Analyzing failure for: WRTD Development Runtime
Features: std, wrt-execution, dev-panic

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
warning: wrt@0.2.0: Setting WASM_TESTSUITE to /Users/r/git/wrt2/target/debug/build/wrt-c3722ec280d4681b/out/testsuite
warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:575:34
    |
575 | impl<const N: usize> Default for NoStdProviderBuilder<N> {
    |                                  ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:581:22
    |
581 | impl<const N: usize> NoStdProviderBuilder<N> {
    |                      ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:643:18
    |
643 | impl Default for NoStdProviderBuilder1 {
    |                  ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:649:6
    |
649 | impl NoStdProviderBuilder1 {
    |      ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated module `safe_allocation::safe_factories`: Use capability_factories for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:260:9
    |
260 | pub use safe_factories::*;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:195:24
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:205:24
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:577:16
    |
577 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder1::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:645:16
    |
645 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:45:17
   |
45 |         context.create_provider::<N>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:78:17
   |
78 |         context.create_provider::<SIZE>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:96:17
   |
96 |         context.create_provider::<SIZE>(self.crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:195:45
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:205:45
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_memory::NoStdProvider::<N>::new`: Use BudgetProvider::new() or create_provider! macro for budget-aware allocation
   --> wrt-foundation/src/capabilities/dynamic.rs:210:57
    |
210 |             crate::safe_memory::NoStdProvider::<65536>::new()
    |                                                         ^^^

warning: `wrt-foundation` (lib) generated 15 warnings
   Compiling wrt-decoder v0.2.0 (/Users/r/git/wrt2/wrt-decoder)
warning: use of deprecated struct `wrt_foundation::safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
  --> wrt-logging/src/bounded_log_infra.rs:47:20
   |
47 |     let provider = safe_managed_alloc!(8192, CrateId::Logging)?;
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## Analyzing failure for: WRTD Server Runtime
Features: std, wrt-execution

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
warning: wrt@0.2.0: Setting WASM_TESTSUITE to /Users/r/git/wrt2/target/debug/build/wrt-c3722ec280d4681b/out/testsuite
warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:575:34
    |
575 | impl<const N: usize> Default for NoStdProviderBuilder<N> {
    |                                  ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:581:22
    |
581 | impl<const N: usize> NoStdProviderBuilder<N> {
    |                      ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:643:18
    |
643 | impl Default for NoStdProviderBuilder1 {
    |                  ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:649:6
    |
649 | impl NoStdProviderBuilder1 {
    |      ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated module `safe_allocation::safe_factories`: Use capability_factories for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:260:9
    |
260 | pub use safe_factories::*;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:195:24
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:205:24
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:577:16
    |
577 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder1::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:645:16
    |
645 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:45:17
   |
45 |         context.create_provider::<N>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:78:17
   |
78 |         context.create_provider::<SIZE>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:96:17
   |
96 |         context.create_provider::<SIZE>(self.crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:195:45
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:205:45
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_memory::NoStdProvider::<N>::new`: Use BudgetProvider::new() or create_provider! macro for budget-aware allocation
   --> wrt-foundation/src/capabilities/dynamic.rs:210:57
    |
210 |             crate::safe_memory::NoStdProvider::<65536>::new()
    |                                                         ^^^

warning: `wrt-foundation` (lib) generated 15 warnings
   Compiling wrt-decoder v0.2.0 (/Users/r/git/wrt2/wrt-decoder)
warning: use of deprecated struct `wrt_foundation::safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
  --> wrt-logging/src/bounded_log_infra.rs:47:20
   |
47 |     let provider = safe_managed_alloc!(8192, CrateId::Logging)?;
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## Analyzing failure for: Component Model Core
Features: no_std, alloc, component-model-core

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:575:34
    |
575 | impl<const N: usize> Default for NoStdProviderBuilder<N> {
    |                                  ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:581:22
    |
581 | impl<const N: usize> NoStdProviderBuilder<N> {
    |                      ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:643:18
    |
643 | impl Default for NoStdProviderBuilder1 {
    |                  ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:649:6
    |
649 | impl NoStdProviderBuilder1 {
    |      ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated module `safe_allocation::safe_factories`: Use capability_factories for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:260:9
    |
260 | pub use safe_factories::*;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:195:24
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:205:24
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:577:16
    |
577 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder1::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:645:16
    |
645 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:45:17
   |
45 |         context.create_provider::<N>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:78:17
   |
78 |         context.create_provider::<SIZE>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:96:17
   |
96 |         context.create_provider::<SIZE>(self.crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:195:45
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:205:45
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_memory::NoStdProvider::<N>::new`: Use BudgetProvider::new() or create_provider! macro for budget-aware allocation
   --> wrt-foundation/src/capabilities/dynamic.rs:210:57
    |
210 |             crate::safe_memory::NoStdProvider::<65536>::new()
    |                                                         ^^^

warning: `wrt-foundation` (lib) generated 15 warnings
warning: unused import: `wrt_error::Error`
 --> wrt-intercept/src/builtins.rs:8:5
  |
8 | use wrt_error::Error;
  |     ^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` on by default
```

## Analyzing failure for: Component Model Full
Features: std, component-model-all

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:575:34
    |
575 | impl<const N: usize> Default for NoStdProviderBuilder<N> {
    |                                  ^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `builder::NoStdProviderBuilder`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:581:22
    |
581 | impl<const N: usize> NoStdProviderBuilder<N> {
    |                      ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:643:18
    |
643 | impl Default for NoStdProviderBuilder1 {
    |                  ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `builder::NoStdProviderBuilder1`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:649:6
    |
649 | impl NoStdProviderBuilder1 {
    |      ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated module `safe_allocation::safe_factories`: Use capability_factories for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:260:9
    |
260 | pub use safe_factories::*;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:195:24
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `safe_allocation::SafeProviderFactory`: Use CapabilityProviderFactory for capability-driven design
   --> wrt-foundation/src/safe_allocation.rs:205:24
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                        ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation. See type documentation for migration guide.
   --> wrt-foundation/src/builder.rs:577:16
    |
577 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `builder::NoStdProviderBuilder1::_migration_marker`: Use safe_managed_alloc!() for budget-aware allocation.
   --> wrt-foundation/src/builder.rs:645:16
    |
645 |         Self { _migration_marker: core::marker::PhantomData }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:45:17
   |
45 |         context.create_provider::<N>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:78:17
   |
78 |         context.create_provider::<SIZE>(crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated method `capabilities::context::MemoryCapabilityContext::create_provider`: Use CapabilityMemoryFactory::create_provider() for new code
  --> wrt-foundation/src/enforcement.rs:96:17
   |
96 |         context.create_provider::<SIZE>(self.crate_id)
   |                 ^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:195:45
    |
195 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_allocation::SafeProviderFactory::create_managed_provider`: Use CapabilityProviderFactory::create_context_managed_provider instead
   --> wrt-foundation/src/safe_allocation.rs:205:45
    |
205 |         let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `safe_memory::NoStdProvider::<N>::new`: Use BudgetProvider::new() or create_provider! macro for budget-aware allocation
   --> wrt-foundation/src/capabilities/dynamic.rs:210:57
    |
210 |             crate::safe_memory::NoStdProvider::<65536>::new()
    |                                                         ^^^

warning: `wrt-foundation` (lib) generated 15 warnings
warning: unused import: `BoundedVec`
  --> wrt-instructions/src/const_expr.rs:13:40
   |
13 | use crate::prelude::{Debug, PartialEq, BoundedVec};
   |                                        ^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default
```

# Architectural Issues Summary

- Trait bound violations suggesting improper abstractions

## Recommended Actions
1. Review module boundaries and dependencies
2. Ensure feature flags properly isolate platform-specific code
3. Verify all ASIL configurations can build without std library
4. Remove or properly abstract unsafe code in safety-critical paths
