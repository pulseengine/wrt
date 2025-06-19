# Architectural Issues Analysis
Date: Do 19 Jun 2025 05:14:13 CEST

## Analyzing failure for: WRT no_std + alloc
Features: alloc

### Raw Error Output
```
   Compiling wrt-error v0.2.0 (/Users/r/git/wrt2/wrt-error)
   Compiling wrt v0.2.0 (/Users/r/git/wrt2/wrt)
   Compiling wrt-sync v0.2.0 (/Users/r/git/wrt2/wrt-sync)
   Compiling wrt-math v0.2.0 (/Users/r/git/wrt2/wrt-math)
   Compiling wrt-foundation v0.2.0 (/Users/r/git/wrt2/wrt-foundation)
warning: use of deprecated struct `wrt_memory_system::WrtProviderFactory`: Use MemoryCapabilityContext::create_provider() for capability-driven design
   --> wrt-foundation/src/lib.rs:332:29
    |
332 | pub use wrt_memory_system::{WrtProviderFactory, WRT_MEMORY_COORDINATOR};
    |                             ^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated unit struct `wrt_memory_system::WrtProviderFactory`: Use MemoryCapabilityContext::create_provider() for capability-driven design
   --> wrt-foundation/src/lib.rs:332:29
    |
332 | pub use wrt_memory_system::{WrtProviderFactory, WRT_MEMORY_COORDINATOR};
    |                             ^^^^^^^^^^^^^^^^^^

warning: use of deprecated static `wrt_memory_system::WRT_MEMORY_COORDINATOR`: Use MemoryCapabilityContext for capability-driven design
   --> wrt-foundation/src/lib.rs:332:49
    |
332 | pub use wrt_memory_system::{WrtProviderFactory, WRT_MEMORY_COORDINATOR};
    |                                                 ^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `budget_provider::BudgetProvider`: Use CapabilityFactoryBuilder and safe_capability_alloc! macro instead
   --> wrt-foundation/src/lib.rs:343:26
    |
343 | pub use budget_provider::BudgetProvider;
    |                          ^^^^^^^^^^^^^^

warning: use of deprecated unit struct `budget_provider::BudgetProvider`: Use CapabilityFactoryBuilder and safe_capability_alloc! macro instead
   --> wrt-foundation/src/lib.rs:343:26
    |
343 | pub use budget_provider::BudgetProvider;
    |                          ^^^^^^^^^^^^^^

warning: use of deprecated struct `wrt_memory_system::WrtProviderFactory`: Use MemoryCapabilityContext::create_provider() for capability-driven design
   --> wrt-foundation/src/prelude.rs:144:25
    |
144 |     wrt_memory_system::{WrtProviderFactory, WRT_MEMORY_COORDINATOR},
    |                         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated unit struct `wrt_memory_system::WrtProviderFactory`: Use MemoryCapabilityContext::create_provider() for capability-driven design
   --> wrt-foundation/src/prelude.rs:144:25
    |
144 |     wrt_memory_system::{WrtProviderFactory, WRT_MEMORY_COORDINATOR},
    |                         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated static `wrt_memory_system::WRT_MEMORY_COORDINATOR`: Use MemoryCapabilityContext for capability-driven design
   --> wrt-foundation/src/prelude.rs:144:45
    |
144 |     wrt_memory_system::{WrtProviderFactory, WRT_MEMORY_COORDINATOR},
    |                                             ^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `wrt_memory_system::WrtProviderFactory`: Use MemoryCapabilityContext::create_provider() for capability-driven design
   --> wrt-foundation/src/wrt_memory_system.rs:106:6
    |
106 | impl WrtProviderFactory {
    |      ^^^^^^^^^^^^^^^^^^

warning: use of deprecated static `wrt_memory_system::WRT_MEMORY_COORDINATOR`: Use MemoryCapabilityContext for capability-driven design
   --> wrt-foundation/src/wrt_memory_system.rs:155:9
    |
155 |         WRT_MEMORY_COORDINATOR.initialize(budgets.iter().copied(), total)
    |         ^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `wrt_memory_system::WrtProviderFactory`: Use MemoryCapabilityContext::create_provider() for capability-driven design
  --> wrt-foundation/src/enforcement.rs:12:41
   |
12 |     wrt_memory_system::{WrtMemoryGuard, WrtProviderFactory},
   |                                         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated unit struct `wrt_memory_system::WrtProviderFactory`: Use MemoryCapabilityContext::create_provider() for capability-driven design
  --> wrt-foundation/src/enforcement.rs:12:41
   |
12 |     wrt_memory_system::{WrtMemoryGuard, WrtProviderFactory},
   |                                         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `wrt_memory_system::WrtProviderFactory`: Use MemoryCapabilityContext::create_provider() for capability-driven design
  --> wrt-foundation/src/enforcement.rs:30:25
   |
30 | impl sealed::Sealed for WrtProviderFactory {}
   |                         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `wrt_memory_system::WrtProviderFactory`: Use MemoryCapabilityContext::create_provider() for capability-driven design
  --> wrt-foundation/src/enforcement.rs:32:24
   |
32 | impl MemoryManaged for WrtProviderFactory {
   |                        ^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `wrt_memory_system::WrtProviderFactory`: Use MemoryCapabilityContext::create_provider() for capability-driven design
  --> wrt-foundation/src/enforcement.rs:65:9
   |
65 |         WrtProviderFactory::create_provider::<SIZE>(crate_id)
   |         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `wrt_memory_system::WrtProviderFactory`: Use MemoryCapabilityContext::create_provider() for capability-driven design
  --> wrt-foundation/src/enforcement.rs:83:9
   |
```

