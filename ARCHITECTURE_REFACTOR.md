# WRT Comprehensive Refactor Architecture

## High-Level Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                           WRT COMPREHENSIVE REFACTOR ARCHITECTURE                    │
│                                                                                     │
│  External Sources → Platform Discovery → Type Unification → Component Integration  │
│                                         ↓                                           │
│                            Memory Constraints & Safety Validation                  │
│                                         ↓                                           │
│                               Unified Runtime Execution                            │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Phase 0: Critical Compilation Fix Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              PHASE 0: COMPILATION FIXES                            │
└─────────────────────────────────────────────────────────────────────────────────────┘

Current State (BROKEN):
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│wrt-runtime  │    │wrt-component│    │wrt-foundation│   │wrt-platform │
│             │    │             │    │             │    │             │
│BoundedVec<  │    │BoundedVec<  │    │BoundedVec<  │    │BoundedVec<  │
│ T,64,P1>    │ ❌ │ T,256,P2>   │ ❌ │ T,1024,P3>  │ ❌ │ T,128,P4>   │
│             │    │             │    │             │    │             │
│ComponentInst│ ❌ │ComponentInst│    │Value        │    │MemoryLimits │
│(duplicate)  │    │             │    │             │    │             │
│             │    │cfi_types::  │ ❌ │             │    │             │
│memory::Adapt│ ❌ │(missing)    │    │             │    │             │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
     ↑                    ↑                    ↑                    ↑
     └────────────────────┴── 421+ TYPE ERRORS ─┴────────────────────┘

Target State (FIXED):
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                           UNIFIED TYPE SYSTEM                                      │
│                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                        wrt-foundation                                       │   │
│  │  ┌─────────────────────────────────────────────────────────────────────┐   │   │
│  │  │                  Unified Memory System                              │   │   │
│  │  │                                                                     │   │   │
│  │  │  trait MemoryProvider { ... }                                      │   │   │
│  │  │  struct RuntimeProvider<const SIZE: usize>                         │   │   │
│  │  │  struct PlatformCapacities { small, medium, large }                │   │   │
│  │  │                                                                     │   │   │
│  │  └─────────────────────────────────────────────────────────────────────┘   │   │
│  │                                                                             │   │
│  │  ┌─────────────────────────────────────────────────────────────────────┐   │   │
│  │  │                 Unified Collection Types                           │   │   │
│  │  │                                                                     │   │   │
│  │  │  BoundedVec<T, CAP, Provider>                                     │   │   │
│  │  │  BoundedString<LEN, Provider>                                     │   │   │
│  │  │  BoundedMap<K, V, CAP, Provider>                                  │   │   │
│  │  │                                                                     │   │   │
│  │  └─────────────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
│                                         │                                           │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                        wrt-runtime                                          │   │
│  │                                                                             │   │
│  │  unified_types.rs ──┐                                                      │   │
│  │  │                 │  RuntimeTypes<SMALL, MEDIUM, LARGE>                  │   │
│  │  │                 │  type SmallVec<T> = BoundedVec<T, SMALL, Provider>   │   │
│  │  │                 │  type MediumVec<T> = BoundedVec<T, MEDIUM, Provider> │   │
│  │  │                 │  type LargeVec<T> = BoundedVec<T, LARGE, Provider>   │   │
│  │  └─────────────────┘                                                       │   │
│  │                                                                             │   │
│  │  component_unified.rs ──┐                                                  │   │
│  │  │                      │  UnifiedComponentInstance                        │   │
│  │  │                      │  UnifiedComponentRuntime                         │   │
│  │  │                      │  ComponentMemoryBudget                           │   │
│  │  └──────────────────────┘                                                  │   │
│  │                                                                             │   │
│  │  component_impl.rs (FIXED) ──┐                                             │   │
│  │  │                           │  ComponentRuntimeImpl                       │   │
│  │  │                           │  - No duplicate imports                     │   │
│  │  │                           │  - Uses wrt_instructions::cfi_control_ops   │   │
│  │  │                           │  - Unified memory adapter                   │   │
│  │  └───────────────────────────┘                                             │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
│                                         │                                           │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                      Other Crates (Updated)                                │   │
│  │                                                                             │   │
│  │  wrt-component: Uses RuntimeTypes::MediumVec for consistency               │   │
│  │  wrt-debug: Uses RuntimeTypes::SmallVec for debug metadata                 │   │
│  │  wrt-platform: Uses RuntimeTypes for platform limit storage               │   │
│  │  wrt-decoder: Uses RuntimeTypes for parser state                           │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Phase 1-2: Platform-Aware Memory Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                    PLATFORM-AWARE MEMORY CONSTRAINT SYSTEM                         │
└─────────────────────────────────────────────────────────────────────────────────────┘

External Limit Sources:
┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   CLI Args  │  │   Env Vars  │  │  Config     │  │  Container  │  │   Runtime   │
│             │  │             │  │  Files      │  │  Discovery  │  │   APIs      │
│--max-memory │  │WRTD_MAX_MEM │  │memory_limit │  │  cgroup     │  │update_limits│
│--asil-level │  │WRTD_ASIL_LV │  │asil_level   │  │  systemd    │  │set_asil     │
└─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘
       │                │                │                │                │
       └────────────────┴────────────────┴────────────────┴────────────────┘
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                           Platform Discovery Layer                                 │
│                                                                                     │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐       │
│  │     Linux     │  │      QNX      │  │   Embedded    │  │     macOS     │       │
│  │               │  │               │  │               │  │               │       │
│  │/proc/meminfo  │  │SYSPAGE query  │  │heap boundaries│  │sysctl hw.mem  │       │
│  │cgroup limits  │  │partition info │  │stack regions  │  │vm_stat query  │       │
│  │container env  │  │ASIL detection │  │device tree    │  │process limits │       │
│  └───────────────┘  └───────────────┘  └───────────────┘  └───────────────┘       │
│         │                    │                    │                    │           │
│         └────────────────────┴────────────────────┴────────────────────┘           │
│                                         │                                           │
│                                         ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                  Comprehensive Platform Limits                             │   │
│  │                                                                             │   │
│  │  struct ComprehensivePlatformLimits {                                      │   │
│  │      platform_id: PlatformId,                                              │   │
│  │      max_total_memory: usize,                                              │   │
│  │      max_wasm_linear_memory: usize,                                        │   │
│  │      max_components: usize,                                                │   │
│  │      max_debug_overhead: usize,                                            │   │
│  │      asil_level: AsilLevel,                                                │   │
│  │      safety_reserve_percentage: usize,                                     │   │
│  │  }                                                                          │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Phase 3-4: Component Model & Debug Integration

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                    COMPONENT MODEL & DEBUG INTEGRATION                             │
└─────────────────────────────────────────────────────────────────────────────────────┘

Streaming Multi-Constraint Validation:
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│WASM Binary  │───▶│  Component  │───▶│Debug Info   │───▶│Safety       │
│             │    │  Binary     │    │(DWARF)      │    │Validation   │
│Core module  │    │  (optional) │    │(optional)   │    │             │
│Functions    │    │WIT interfaces│    │Source maps  │    │ASIL checks  │
│Memory       │    │Resources    │    │Breakpoints  │    │Limits       │
│Imports      │    │Cross-calls  │    │Stack traces │    │Compliance   │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       │                    │                    │                    │
       ▼                    ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                    Comprehensive WASM Validator                                    │
│                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                      Single-Pass Processing                                 │   │
│  │                                                                             │   │
│  │  for section in wasm_sections {                                            │   │
│  │      match section {                                                       │   │
│  │          Memory => validate_memory_against_limits(),                       │   │
│  │          Code => estimate_stack_usage(),                                   │   │
│  │          Component => validate_component_complexity(),                     │   │
│  │          Debug => validate_debug_overhead(),                               │   │
│  │      }                                                                     │   │
│  │      if exceeds_limits() { return Err(FailFast); }                        │   │
│  │  }                                                                         │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
│                                         │                                           │
│                                         ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                    Memory Budget Calculator                                 │   │
│  │                                                                             │   │
│  │  Total Memory: 256MB (from platform limits)                               │   │
│  │  ├── WASM Linear: 180MB (70%)                                             │   │
│  │  ├── Component Overhead: 25MB (10%)                                       │   │
│  │  ├── Debug Overhead: 15MB (6%)                                            │   │
│  │  ├── Runtime Overhead: 20MB (8%)                                          │   │
│  │  └── Safety Reserve: 16MB (6%)                                            │   │
│  │                                                                             │   │
│  │  if (total_required > platform_limit) { FAIL_FAST }                       │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────────┘

Component Instance Management:
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                      Unified Component Runtime                                     │
│                                                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │
│  │ Component A │  │ Component B │  │ Component C │  │   Debug     │               │
│  │             │  │             │  │             │  │   Info      │               │
│  │Memory: 50MB │  │Memory: 30MB │  │Memory: 25MB │  │Memory: 15MB │               │
│  │Resources:128│  │Resources: 64│  │Resources: 32│  │Symbols:2048 │               │
│  │Imports: 16  │  │Imports: 8   │  │Imports: 4   │  │Breakpts: 64 │               │
│  │Exports: 8   │  │Exports: 12  │  │Exports: 6   │  │Files: 32    │               │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘               │
│         │                │                │                │                       │
│         └────────────────┴────────────────┴────────────────┘                       │
│                                         │                                           │
│                                         ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                Platform Memory Adapter                                      │   │
│  │                                                                             │   │
│  │  Available: 256MB → Allocated: 120MB → Remaining: 136MB                    │   │
│  │  Safety Reserve: 16MB → Usable: 120MB                                      │   │
│  │                                                                             │   │
│  │  ✓ All components fit within platform limits                              │   │
│  │  ✓ Debug overhead within acceptable bounds                                 │   │
│  │  ✓ ASIL-B compliance maintained                                            │   │
│  └─────────────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Platform-Specific Configurations

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                      PLATFORM-SPECIFIC CONFIGURATIONS                              │
└─────────────────────────────────────────────────────────────────────────────────────┘

Linux Container (Kubernetes):
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  Container Memory: 512MB (from cgroup)                                             │
│  ├── WASM Memory: 350MB (68%)                                                      │
│  ├── Components: 50MB (10%) → 256 components, 1024 instances                      │
│  ├── Debug: 50MB (10%) → Full debugging, 10K breakpoints                          │
│  ├── Runtime: 40MB (8%)                                                            │
│  └── Safety: 22MB (4%) → ASIL-A level                                             │
│                                                                                     │
│  Platform Features: cgroup detection, container limits, K8s env vars              │
└─────────────────────────────────────────────────────────────────────────────────────┘

QNX Safety-Critical:
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  Partition Memory: 64MB (from SYSPAGE)                                             │
│  ├── WASM Memory: 32MB (50%)                                                       │
│  ├── Components: 8MB (12%) → 16 components, 64 instances                          │
│  ├── Debug: 2MB (3%) → Minimal debugging, 50 breakpoints                          │
│  ├── Runtime: 8MB (12%)                                                            │
│  └── Safety: 14MB (23%) → ASIL-C level                                            │
│                                                                                     │
│  Platform Features: SYSPAGE queries, partition detection, ASIL enforcement        │
└─────────────────────────────────────────────────────────────────────────────────────┘

Embedded (Zephyr):
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  Total RAM: 32KB (from device tree)                                                │
│  ├── WASM Memory: 20KB (62%)                                                       │
│  ├── Components: 4KB (12%) → 4 components, 8 instances                            │
│  ├── Debug: 1KB (3%) → Basic debugging, 10 breakpoints                            │
│  ├── Runtime: 4KB (12%)                                                            │
│  └── Safety: 3KB (11%) → ASIL-B level                                             │
│                                                                                     │
│  Platform Features: heap boundaries, memory domains, minimal overhead             │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Implementation Flow

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                          IMPLEMENTATION FLOW                                       │
└─────────────────────────────────────────────────────────────────────────────────────┘

Phase 0 (Days 1-3): CRITICAL COMPILATION FIXES
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  Day 1: wrt-runtime                                                                │
│  ├── unified_types.rs implementation                                               │
│  ├── component_impl.rs duplicate import fixes                                      │
│  ├── component_unified.rs creation                                                 │
│  └── prelude.rs unified type integration                                           │
│                                                                                     │
│  Day 2: wrt-runtime continued                                                      │
│  ├── cfi_engine.rs missing module fixes                                            │
│  ├── UnifiedMemoryAdapter integration                                              │
│  ├── All runtime modules updated                                                   │
│  └── std/no_std compilation verification                                           │
│                                                                                     │
│  Day 3: wrt-foundation                                                             │
│  ├── Bounded collections runtime configuration                                     │
│  ├── Platform-aware memory provider factory                                        │
│  ├── Memory budget calculation utilities                                           │
│  └── Cross-crate type compatibility verification                                   │
└─────────────────────────────────────────────────────────────────────────────────────┘

Phase 1 (Weeks 1-2): PLATFORM INFRASTRUCTURE
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  Week 1: wrt-platform                                                              │
│  ├── ComprehensiveLimitProvider trait                                              │
│  ├── Platform detection and factory                                                │
│  ├── Linux comprehensive limit discovery                                           │
│  └── macOS comprehensive limit discovery                                           │
│                                                                                     │
│  Week 2: Extended platform support                                                 │
│  ├── QNX limit discovery with ASIL                                                 │
│  ├── Embedded platform support                                                     │
│  ├── External platform templates                                                   │
│  └── Error handling and fallbacks                                                  │
└─────────────────────────────────────────────────────────────────────────────────────┘

Phase 2 (Weeks 3-4): COMPONENT MODEL
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  Week 3: wrt-component                                                             │
│  ├── Platform-aware component configuration                                        │
│  ├── Component memory overhead calculation                                         │
│  ├── Resource management with platform limits                                      │
│  └── WIT parser with bounded collections                                           │
│                                                                                     │
│  Week 4: Component integration                                                     │
│  ├── Cross-component call limit enforcement                                        │
│  ├── Component instance management                                                 │
│  ├── Component metadata with memory budgets                                        │
│  └── Canonical ABI with platform-aware allocation                                  │
└─────────────────────────────────────────────────────────────────────────────────────┘

Phase 3 (Weeks 5-6): DEBUG & VALIDATION
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  Week 5: wrt-debug                                                                 │
│  ├── Platform-aware debug configuration                                            │
│  ├── DWARF parsing with memory limits                                              │
│  ├── Runtime debugging with bounded overhead                                       │
│  └── WIT-aware debugging with constraints                                          │
│                                                                                     │
│  Week 6: Streaming validation                                                      │
│  ├── Comprehensive streaming WASM validator                                        │
│  ├── Immediate limit validation during parsing                                     │
│  ├── Multi-constraint validation                                                   │
│  └── Platform-specific parsing optimizations                                       │
└─────────────────────────────────────────────────────────────────────────────────────┘

Phase 4 (Weeks 7-8): RUNTIME INTEGRATION
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  Week 7: wrt-runtime integration                                                   │
│  ├── Platform limits with execution engine                                         │
│  ├── ASIL-aware runtime behavior                                                   │
│  ├── Memory adapter with platform constraints                                      │
│  └── Component runtime with bounded resources                                      │
│                                                                                     │
│  Week 8: Final integration                                                         │
│  ├── All crate enhancements integration                                            │
│  ├── Comprehensive configuration API                                               │
│  ├── External limit discovery integration                                          │
│  └── Unified runtime factory with platform detection                               │
└─────────────────────────────────────────────────────────────────────────────────────┘

Phase 5 (Weeks 9-10): CLI & TESTING
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  Week 9: wrtd CLI                                                                  │
│  ├── Comprehensive command-line interface                                          │
│  ├── External limit specification                                                  │
│  ├── Platform-aware runtime mode selection                                         │
│  └── Debug level and ASIL configuration                                            │
│                                                                                     │
│  Week 10: Testing & deployment                                                     │
│  ├── Cross-platform integration tests                                              │
│  ├── Performance regression testing                                                │
│  ├── Production deployment simulation                                              │
│  └── Documentation and examples                                                    │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Success Criteria

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              SUCCESS CRITERIA                                      │
└─────────────────────────────────────────────────────────────────────────────────────┘

Compilation Success:
✓ wrt-runtime compiles with 0 errors (std and no_std)
✓ All 421+ type errors resolved through unified type system
✓ All crates compile individually and as workspace
✓ No duplicate imports or missing modules

Performance Targets:
✓ Single-pass validation ≤ 10ms for typical WASM modules
✓ Memory accuracy ≤ 5% difference between predicted and actual
✓ Platform detection ≤ 1ms for limit discovery
✓ Component instantiation overhead ≤ 2% of WASM execution

Functional Requirements:
✓ 100% platform coverage across all 7 defined platforms
✓ Full Component Model specification support with platform limits
✓ Production-appropriate debug capabilities per platform
✓ ASIL-D compliance with formal verification support

Operational Excellence:
✓ Container-native automatic limit discovery (Docker/Kubernetes)
✓ Safety-critical QNX/automotive deployment with RT guarantees
✓ Embedded deployment on 32KB+ systems
✓ Development-friendly debugging with minimal production overhead
```

This architecture shows the complete transformation from the current broken state to a unified, platform-aware system that solves both the immediate compilation issues and provides the comprehensive external memory constraint system.
