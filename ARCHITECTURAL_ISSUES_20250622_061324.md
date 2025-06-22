# Architectural Issues Analysis
Date: 2025-06-22 06:13:24

## Analyzing failure for: WRT no_std + alloc
Features: alloc

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
   Compiling wrt-error v0.2.0 (/Users/r/git/wrt2/wrt-error)
   Compiling wrt v0.2.0 (/Users/r/git/wrt2/wrt)
   Compiling wrt-sync v0.2.0 (/Users/r/git/wrt2/wrt-sync)
   Compiling wrt-math v0.2.0 (/Users/r/git/wrt2/wrt-math)
   Compiling wrt-foundation v0.2.0 (/Users/r/git/wrt2/wrt-foundation)
warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/budget_provider.rs:18:30
   |
18 |     CapabilityAwareProvider, CapabilityFactoryBuilder, DynamicMemoryCapability, ProviderCapabilityExt
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:44:6
   |
44 | impl CapabilityMemoryFactory {
   |      ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:276:6
    |
276 | impl CapabilityFactoryBuilder {
    |      ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:320:18
    |
320 | impl Default for CapabilityFactoryBuilder {
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:314:27
    |
314 |     pub fn build(self) -> CapabilityMemoryFactory {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:315:9
    |
315 |         CapabilityMemoryFactory::new(self.context)
    |         ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:47:16
   |
47 |         Self { context }
   |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:59:26
   |
59 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:78:26
   |
78 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:101:26
    |
101 |         let capability = self.context.get_capability(crate_id)?;
    |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:121:10
    |
121 |         &self.context
    |          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:131:9
    |
131 |         self.context.register_capability(crate_id, capability)
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:279:16
    |
279 |         Self { context: MemoryCapabilityContext::default() }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:284:16
    |
284 |         Self { context }
    |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:293:9
    |
293 |         self.context.register_dynamic_capability(crate_id, max_allocation)?;
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:299:9
    |
```

## Analyzing failure for: WRT ASIL-B (no_std + alloc)
Features: alloc, safety-asil-b

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
   Compiling wrt v0.2.0 (/Users/r/git/wrt2/wrt)
   Compiling wrt-foundation v0.2.0 (/Users/r/git/wrt2/wrt-foundation)
warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/budget_provider.rs:18:30
   |
18 |     CapabilityAwareProvider, CapabilityFactoryBuilder, DynamicMemoryCapability, ProviderCapabilityExt
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:44:6
   |
44 | impl CapabilityMemoryFactory {
   |      ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:276:6
    |
276 | impl CapabilityFactoryBuilder {
    |      ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:320:18
    |
320 | impl Default for CapabilityFactoryBuilder {
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:314:27
    |
314 |     pub fn build(self) -> CapabilityMemoryFactory {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:315:9
    |
315 |         CapabilityMemoryFactory::new(self.context)
    |         ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:47:16
   |
47 |         Self { context }
   |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:59:26
   |
59 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:78:26
   |
78 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:101:26
    |
101 |         let capability = self.context.get_capability(crate_id)?;
    |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:121:10
    |
121 |         &self.context
    |          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:131:9
    |
131 |         self.context.register_capability(crate_id, capability)
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:279:16
    |
279 |         Self { context: MemoryCapabilityContext::default() }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:284:16
    |
284 |         Self { context }
    |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:293:9
    |
293 |         self.context.register_dynamic_capability(crate_id, max_allocation)?;
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:299:9
    |
299 |         self.context.register_static_capability::<N>(crate_id)?;
    |         ^^^^^^^^^^^^

```

## Analyzing failure for: WRT Development (std)
Features: std

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### ⚠️ ARCHITECTURAL ISSUE: Missing Imports/Modules
Feature flags are not properly managing code visibility.
This violates ASIL requirement for deterministic compilation.

### Raw Error Output
```
   Compiling wrt-error v0.2.0 (/Users/r/git/wrt2/wrt-error)
   Compiling wrt v0.2.0 (/Users/r/git/wrt2/wrt)
   Compiling wrt-sync v0.2.0 (/Users/r/git/wrt2/wrt-sync)
   Compiling wrt-foundation v0.2.0 (/Users/r/git/wrt2/wrt-foundation)
   Compiling wrt-platform v0.2.0 (/Users/r/git/wrt2/wrt-platform)
warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/budget_provider.rs:18:30
   |
18 |     CapabilityAwareProvider, CapabilityFactoryBuilder, DynamicMemoryCapability, ProviderCapabilityExt
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:44:6
   |
44 | impl CapabilityMemoryFactory {
   |      ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:276:6
    |
276 | impl CapabilityFactoryBuilder {
    |      ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:320:18
    |
320 | impl Default for CapabilityFactoryBuilder {
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:314:27
    |
314 |     pub fn build(self) -> CapabilityMemoryFactory {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:315:9
    |
315 |         CapabilityMemoryFactory::new(self.context)
    |         ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:47:16
   |
47 |         Self { context }
   |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:59:26
   |
59 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:78:26
   |
78 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:101:26
    |
101 |         let capability = self.context.get_capability(crate_id)?;
    |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:121:10
    |
121 |         &self.context
    |          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:131:9
    |
131 |         self.context.register_capability(crate_id, capability)
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:279:16
    |
279 |         Self { context: MemoryCapabilityContext::default() }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:284:16
    |
284 |         Self { context }
    |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:293:9
    |
293 |         self.context.register_dynamic_capability(crate_id, max_allocation)?;
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:299:9
    |
```

## Analyzing failure for: WRT Development with Optimization
Features: std, optimize

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### ⚠️ ARCHITECTURAL ISSUE: Missing Imports/Modules
Feature flags are not properly managing code visibility.
This violates ASIL requirement for deterministic compilation.

### Raw Error Output
```
   Compiling wrt v0.2.0 (/Users/r/git/wrt2/wrt)
   Compiling wrt-foundation v0.2.0 (/Users/r/git/wrt2/wrt-foundation)
warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/budget_provider.rs:18:30
   |
18 |     CapabilityAwareProvider, CapabilityFactoryBuilder, DynamicMemoryCapability, ProviderCapabilityExt
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:44:6
   |
44 | impl CapabilityMemoryFactory {
   |      ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:276:6
    |
276 | impl CapabilityFactoryBuilder {
    |      ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:320:18
    |
320 | impl Default for CapabilityFactoryBuilder {
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:314:27
    |
314 |     pub fn build(self) -> CapabilityMemoryFactory {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:315:9
    |
315 |         CapabilityMemoryFactory::new(self.context)
    |         ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:47:16
   |
47 |         Self { context }
   |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:59:26
   |
59 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:78:26
   |
78 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:101:26
    |
101 |         let capability = self.context.get_capability(crate_id)?;
    |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:121:10
    |
121 |         &self.context
    |          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:131:9
    |
131 |         self.context.register_capability(crate_id, capability)
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:279:16
    |
279 |         Self { context: MemoryCapabilityContext::default() }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:284:16
    |
284 |         Self { context }
    |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:293:9
    |
293 |         self.context.register_dynamic_capability(crate_id, max_allocation)?;
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:299:9
    |
299 |         self.context.register_static_capability::<N>(crate_id)?;
    |         ^^^^^^^^^^^^

```

## Analyzing failure for: WRT Server
Features: std, optimize, platform

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### ⚠️ ARCHITECTURAL ISSUE: Missing Imports/Modules
Feature flags are not properly managing code visibility.
This violates ASIL requirement for deterministic compilation.

### Raw Error Output
```
   Compiling wrt v0.2.0 (/Users/r/git/wrt2/wrt)
warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/budget_provider.rs:18:30
   |
18 |     CapabilityAwareProvider, CapabilityFactoryBuilder, DynamicMemoryCapability, ProviderCapabilityExt
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:44:6
   |
44 | impl CapabilityMemoryFactory {
   |      ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:276:6
    |
276 | impl CapabilityFactoryBuilder {
    |      ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:320:18
    |
320 | impl Default for CapabilityFactoryBuilder {
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:314:27
    |
314 |     pub fn build(self) -> CapabilityMemoryFactory {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:315:9
    |
315 |         CapabilityMemoryFactory::new(self.context)
    |         ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:47:16
   |
47 |         Self { context }
   |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:59:26
   |
59 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:78:26
   |
78 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:101:26
    |
101 |         let capability = self.context.get_capability(crate_id)?;
    |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:121:10
    |
121 |         &self.context
    |          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:131:9
    |
131 |         self.context.register_capability(crate_id, capability)
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:279:16
    |
279 |         Self { context: MemoryCapabilityContext::default() }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:284:16
    |
284 |         Self { context }
    |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:293:9
    |
293 |         self.context.register_dynamic_capability(crate_id, max_allocation)?;
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:299:9
    |
299 |         self.context.register_static_capability::<N>(crate_id)?;
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
```

## Analyzing failure for: WRTD Development Runtime
Features: std, wrt-execution, dev-panic

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### ⚠️ ARCHITECTURAL ISSUE: Missing Imports/Modules
Feature flags are not properly managing code visibility.
This violates ASIL requirement for deterministic compilation.

### Raw Error Output
```
   Compiling wrt v0.2.0 (/Users/r/git/wrt2/wrt)
   Compiling wrt-panic v0.2.0 (/Users/r/git/wrt2/wrt-panic)
   Compiling spinning_top v0.2.5
   Compiling wrt-math v0.2.0 (/Users/r/git/wrt2/wrt-math)
   Compiling wrt-sync v0.2.0 (/Users/r/git/wrt2/wrt-sync)
   Compiling linked_list_allocator v0.10.5
   Compiling wrt-foundation v0.2.0 (/Users/r/git/wrt2/wrt-foundation)
   Compiling wrt-platform v0.2.0 (/Users/r/git/wrt2/wrt-platform)
warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/budget_provider.rs:18:30
   |
18 |     CapabilityAwareProvider, CapabilityFactoryBuilder, DynamicMemoryCapability, ProviderCapabilityExt
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:44:6
   |
44 | impl CapabilityMemoryFactory {
   |      ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:276:6
    |
276 | impl CapabilityFactoryBuilder {
    |      ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:320:18
    |
320 | impl Default for CapabilityFactoryBuilder {
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:314:27
    |
314 |     pub fn build(self) -> CapabilityMemoryFactory {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:315:9
    |
315 |         CapabilityMemoryFactory::new(self.context)
    |         ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:47:16
   |
47 |         Self { context }
   |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:59:26
   |
59 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:78:26
   |
78 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:101:26
    |
101 |         let capability = self.context.get_capability(crate_id)?;
    |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:121:10
    |
121 |         &self.context
    |          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:131:9
    |
131 |         self.context.register_capability(crate_id, capability)
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:279:16
    |
279 |         Self { context: MemoryCapabilityContext::default() }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:284:16
    |
284 |         Self { context }
    |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:293:9
    |
293 |         self.context.register_dynamic_capability(crate_id, max_allocation)?;
    |         ^^^^^^^^^^^^

```

## Analyzing failure for: WRTD Server Runtime
Features: std, wrt-execution

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### ⚠️ ARCHITECTURAL ISSUE: Missing Imports/Modules
Feature flags are not properly managing code visibility.
This violates ASIL requirement for deterministic compilation.

### Raw Error Output
```
   Compiling wrt-panic v0.2.0 (/Users/r/git/wrt2/wrt-panic)
warning: wrt@0.2.0: Setting WASM_TESTSUITE to /Users/r/git/wrt2/target/debug/build/wrt-c3722ec280d4681b/out/testsuite
warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/budget_provider.rs:18:30
   |
18 |     CapabilityAwareProvider, CapabilityFactoryBuilder, DynamicMemoryCapability, ProviderCapabilityExt
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:44:6
   |
44 | impl CapabilityMemoryFactory {
   |      ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:276:6
    |
276 | impl CapabilityFactoryBuilder {
    |      ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:320:18
    |
320 | impl Default for CapabilityFactoryBuilder {
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:314:27
    |
314 |     pub fn build(self) -> CapabilityMemoryFactory {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:315:9
    |
315 |         CapabilityMemoryFactory::new(self.context)
    |         ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:47:16
   |
47 |         Self { context }
   |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:59:26
   |
59 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:78:26
   |
78 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:101:26
    |
101 |         let capability = self.context.get_capability(crate_id)?;
    |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:121:10
    |
121 |         &self.context
    |          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:131:9
    |
131 |         self.context.register_capability(crate_id, capability)
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:279:16
    |
279 |         Self { context: MemoryCapabilityContext::default() }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:284:16
    |
284 |         Self { context }
    |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:293:9
    |
293 |         self.context.register_dynamic_capability(crate_id, max_allocation)?;
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:299:9
    |
299 |         self.context.register_static_capability::<N>(crate_id)?;
    |         ^^^^^^^^^^^^

```

## Analyzing failure for: Component Model Core
Features: no_std, alloc, component-model-core

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### Raw Error Output
```
   Compiling wrt-math v0.2.0 (/Users/r/git/wrt2/wrt-math)
   Compiling wrt-foundation v0.2.0 (/Users/r/git/wrt2/wrt-foundation)
warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/budget_provider.rs:18:30
   |
18 |     CapabilityAwareProvider, CapabilityFactoryBuilder, DynamicMemoryCapability, ProviderCapabilityExt
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:44:6
   |
44 | impl CapabilityMemoryFactory {
   |      ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:276:6
    |
276 | impl CapabilityFactoryBuilder {
    |      ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:320:18
    |
320 | impl Default for CapabilityFactoryBuilder {
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:314:27
    |
314 |     pub fn build(self) -> CapabilityMemoryFactory {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:315:9
    |
315 |         CapabilityMemoryFactory::new(self.context)
    |         ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:47:16
   |
47 |         Self { context }
   |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:59:26
   |
59 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:78:26
   |
78 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:101:26
    |
101 |         let capability = self.context.get_capability(crate_id)?;
    |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:121:10
    |
121 |         &self.context
    |          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:131:9
    |
131 |         self.context.register_capability(crate_id, capability)
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:279:16
    |
279 |         Self { context: MemoryCapabilityContext::default() }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:284:16
    |
284 |         Self { context }
    |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:293:9
    |
293 |         self.context.register_dynamic_capability(crate_id, max_allocation)?;
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:299:9
    |
299 |         self.context.register_static_capability::<N>(crate_id)?;
    |         ^^^^^^^^^^^^

```

## Analyzing failure for: Component Model Full
Features: std, component-model-all

### ⚠️ ARCHITECTURAL ISSUE: Trait Bound Violations
Feature combinations are creating incompatible trait requirements.
This suggests improper abstraction boundaries for ASIL compliance.

### ⚠️ ARCHITECTURAL ISSUE: Missing Imports/Modules
Feature flags are not properly managing code visibility.
This violates ASIL requirement for deterministic compilation.

### Raw Error Output
```
   Compiling wrt-foundation v0.2.0 (/Users/r/git/wrt2/wrt-foundation)
   Compiling wrt-platform v0.2.0 (/Users/r/git/wrt2/wrt-platform)
warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/budget_provider.rs:18:30
   |
18 |     CapabilityAwareProvider, CapabilityFactoryBuilder, DynamicMemoryCapability, ProviderCapabilityExt
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:44:6
   |
44 | impl CapabilityMemoryFactory {
   |      ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:276:6
    |
276 | impl CapabilityFactoryBuilder {
    |      ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityFactoryBuilder`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:320:18
    |
320 | impl Default for CapabilityFactoryBuilder {
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:314:27
    |
314 |     pub fn build(self) -> CapabilityMemoryFactory {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `capabilities::factory::CapabilityMemoryFactory`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:315:9
    |
315 |         CapabilityMemoryFactory::new(self.context)
    |         ^^^^^^^^^^^^^^^^^^^^^^^

   Compiling wrt-math v0.2.0 (/Users/r/git/wrt2/wrt-math)
warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:47:16
   |
47 |         Self { context }
   |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:59:26
   |
59 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
  --> wrt-foundation/src/capabilities/factory.rs:78:26
   |
78 |         let capability = self.context.get_capability(crate_id)?;
   |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:101:26
    |
101 |         let capability = self.context.get_capability(crate_id)?;
    |                          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:121:10
    |
121 |         &self.context
    |          ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityMemoryFactory::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:131:9
    |
131 |         self.context.register_capability(crate_id, capability)
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:279:16
    |
279 |         Self { context: MemoryCapabilityContext::default() }
    |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:284:16
    |
284 |         Self { context }
    |                ^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:293:9
    |
293 |         self.context.register_dynamic_capability(crate_id, max_allocation)?;
    |         ^^^^^^^^^^^^

warning: use of deprecated field `capabilities::factory::CapabilityFactoryBuilder::context`: Use MemoryFactory for simpler memory provider creation
   --> wrt-foundation/src/capabilities/factory.rs:299:9
    |
299 |         self.context.register_static_capability::<N>(crate_id)?;
    |         ^^^^^^^^^^^^
```

# Architectural Issues Summary

- Trait bound violations suggesting improper abstractions
- Missing imports/modules breaking deterministic compilation

## Recommended Actions
1. Review module boundaries and dependencies
2. Ensure feature flags properly isolate platform-specific code
3. Verify all ASIL configurations can build without std library
4. Remove or properly abstract unsafe code in safety-critical paths
