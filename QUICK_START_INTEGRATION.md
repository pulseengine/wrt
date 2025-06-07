# Quick Start: SCORE-Inspired Safety Integration

## Current Status Summary

✅ **What's Working:**
- SCORE-inspired safety verification frameworks implemented (`wrt-verification-tool/`)
- Requirements traceability, ASIL testing, safety verification, documentation verification
- Standalone demo successfully runs and shows comprehensive safety reporting
- Error API migration partially completed for `wrt-component`

❌ **Current Blockers:**
- Compilation dependencies due to complex trait bounds in foundation
- Platform panic handler issues in no_std builds
- Missing trait implementations for bounded collections

## Immediate Next Steps (This Week)

### Step 1: Create Simple Requirements File Format

```bash
# Create requirements.toml in project root
cat > requirements.toml << EOF
[meta]
project = "WRT WebAssembly Runtime"
version = "0.2.0"
safety_standard = "ISO26262"

[[requirement]]
id = "REQ_MEM_001"
title = "Memory Bounds Checking"
description = "All memory operations must be bounds-checked to prevent buffer overflows"
type = "Memory"
asil_level = "AsilC"
implementations = ["wrt-foundation/src/safe_memory.rs"]
tests = ["wrt-foundation/tests/memory_tests_moved.rs"]
documentation = ["docs/memory-safety.md"]

[[requirement]]
id = "REQ_SAFETY_001"
title = "ASIL Context Maintenance"
description = "Runtime must maintain safety context with ASIL level tracking"
type = "Safety"
asil_level = "AsilD"
implementations = ["wrt-foundation/src/safety_system.rs"]
tests = ["wrt-foundation/tests/"]
documentation = ["docs/safety-requirements.md"]
EOF
```

### Step 2: Add Verification Commands to Justfile

```bash
# Add to justfile
verify-safety:
    @echo "🔍 Running SCORE-inspired safety verification..."
    cd wrt-verification-tool && cargo run --example score_verification_demo

check-requirements:
    @echo "📋 Checking requirements traceability..."
    @if [ -f requirements.toml ]; then \
        echo "✅ Requirements file found"; \
        echo "📊 Requirements defined: $(grep -c '\\[\\[requirement\\]\\]' requirements.toml)"; \
    else \
        echo "❌ No requirements.toml found - create one with 'just init-requirements'"; \
    fi

init-requirements:
    @echo "📋 Creating sample requirements.toml..."
    @if [ ! -f requirements.toml ]; then \
        cp wrt-verification-tool/examples/requirements.toml .; \
        echo "✅ Created requirements.toml"; \
    else \
        echo "⚠️  requirements.toml already exists"; \
    fi

safety-report:
    @echo "📊 Generating safety verification report..."
    cd wrt-verification-tool && cargo run --example score_verification_demo > ../safety-report.txt
    @echo "✅ Safety report generated: safety-report.txt"
```

### Step 3: Create ASIL Test Attributes (Proc Macro)

```bash
# Create simple proc macro for ASIL test tagging
mkdir wrt-test-macros
cd wrt-test-macros

cat > Cargo.toml << EOF
[package]
name = "wrt-test-macros"
version = "0.2.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = "2.0"
EOF

cat > src/lib.rs << EOF
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Lit, Meta, MetaNameValue};

#[proc_macro_attribute]
pub fn asil_test(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    
    // Parse ASIL level from attribute
    let asil_level = if args.is_empty() {
        quote! { "QM" }
    } else {
        let args_str = args.to_string();
        quote! { #args_str }
    };
    
    let expanded = quote! {
        #[test]
        #input_fn
        
        // Register test metadata
        #[ctor::ctor]
        fn #fn_name_register() {
            wrt_test_registry::register_asil_test(
                stringify!(#fn_name),
                #asil_level,
                file!(),
                line!()
            );
        }
    };
    
    TokenStream::from(expanded)
}
EOF
```

### Step 4: Simple Requirements Integration

```rust
// Create wrt-verification-tool/src/requirements_file.rs
use std::fs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RequirementsFile {
    pub meta: ProjectMeta,
    pub requirement: Vec<RequirementDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectMeta {
    pub project: String,
    pub version: String,
    pub safety_standard: String,
}

#[derive(Debug, Deserialize)]
pub struct RequirementDefinition {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub req_type: String,
    pub asil_level: String,
    pub implementations: Vec<String>,
    pub tests: Vec<String>,
    pub documentation: Vec<String>,
}

impl RequirementsFile {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let req_file: RequirementsFile = toml::from_str(&content)?;
        Ok(req_file)
    }
    
    pub fn verify_files_exist(&self) -> Vec<String> {
        let mut missing = Vec::new();
        
        for req in &self.requirement {
            for impl_file in &req.implementations {
                if !std::path::Path::new(impl_file).exists() {
                    missing.push(format!("Implementation: {}", impl_file));
                }
            }
            for test_file in &req.tests {
                if !std::path::Path::new(test_file).exists() {
                    missing.push(format!("Test: {}", test_file));
                }
            }
        }
        
        missing
    }
}
```

## Integration Workflow

### Phase A: Basic Requirements Tracking (Week 1)

1. **Create requirements.toml** with existing WRT safety requirements
2. **Add justfile commands** for basic verification
3. **Test workflow**: `just check-requirements && just verify-safety`

### Phase B: Test Integration (Week 2)

1. **Add ASIL test macros** to critical test files
2. **Create test categorization** by ASIL level
3. **Test filtering**: Run only ASIL-D tests for critical validation

### Phase C: CI Integration (Week 3)

1. **Add safety verification to CI** as non-blocking check
2. **Generate safety reports** on every build
3. **Track compliance trends** over time

### Phase D: Real File Verification (Week 4)

1. **Implement real documentation checking**
2. **Add cross-reference validation**
3. **Generate certification artifacts**

## Quick Demo Commands

```bash
# Test current implementation
just verify-safety

# Initialize requirements tracking
just init-requirements
just check-requirements

# Generate safety report
just safety-report
cat safety-report.txt
```

## Expected Output

```
🔍 SCORE-Inspired Safety Verification Framework Demo
════════════════════════════════════════════════════

📋 1. Requirements Traceability Framework
─────────────────────────────────────────
  REQ_MEM_001 [ASIL-C] - Memory Bounds Checking
    Status: Verified
    Coverage: Comprehensive
    Implementations: 1 files
    Tests: 1 files

🛡️  3. Safety Verification Framework
────────────────────────────────────
  ASIL Compliance Analysis:
  ┌─────────┬────────────┬──────────┬────────────┐
  │ ASIL    │ Current    │ Required │ Status     │
  ├─────────┼────────────┼──────────┼────────────┤
  │ QM      │    100.0% │   70.0% │ ✅ PASS     │
  │ ASIL-C  │     85.0% │   90.0% │ ❌ FAIL     │
  │ ASIL-D  │     75.0% │   95.0% │ ❌ FAIL     │
  └─────────┴────────────┴──────────┴────────────┘

📊 5. Comprehensive Safety Report
─────────────────────────────────
  🎯 Certification Readiness:
    • Requirements coverage: 90% ✅
    • ASIL-D compliance: 75% ❌
```

This shows working SCORE-inspired safety verification without waiting for all compilation issues to be resolved.

## Benefits

1. **Immediate Value**: Safety verification works today
2. **Incremental Integration**: Add features as compilation issues are resolved
3. **Process Improvement**: Establish safety-critical development practices
4. **Certification Readiness**: Build foundation for automotive/aerospace certification

## Next Developer Session

Focus on:
1. Fixing foundation trait bound issues (biggest blocker)
2. Adding real requirements.toml parsing
3. Creating simple test macros for ASIL tagging
4. CI integration for automated safety checking