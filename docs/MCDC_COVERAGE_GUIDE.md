# MC/DC Coverage Guide for WRT

## Overview

Modified Condition/Decision Coverage (MC/DC) is a critical requirement for safety-critical systems (ASIL-D, SIL-3). This guide explains how to achieve and measure MC/DC coverage in the WRT project.

## Requirements

1. **Rust Nightly**: MC/DC requires nightly Rust with specific features
2. **LLVM 18+**: MC/DC support requires LLVM 18 or later
3. **cargo-llvm-cov**: Version 0.5.0+ with MC/DC support

## Setup

### 1. Install Nightly Rust with Coverage Components

```bash
rustup install nightly
rustup component add llvm-tools-preview --toolchain nightly
```

### 2. Configure MC/DC Coverage

Create `.cargo/config.toml` for MC/DC-specific settings:

```toml
[build]
rustflags = ["-C", "instrument-coverage", "-C", "llvm-args=-enable-mcdc"]

[target.'cfg(all())']
rustflags = ["-C", "instrument-coverage", "-C", "llvm-args=-enable-mcdc"]
```

### 3. Enable MC/DC in Tests

For MC/DC to be effective, tests must exercise all condition combinations:

```rust
#[cfg(test)]
mod mcdc_tests {
    use super::*;

    #[test]
    fn test_complex_condition_mcdc() {
        // For condition: (a && b) || (c && d)
        // MC/DC requires testing all combinations where each condition
        // independently affects the outcome
        
        // Test cases for MC/DC coverage:
        assert!(evaluate(true, true, false, false));   // a && b = true
        assert!(evaluate(false, true, true, true));    // c && d = true
        assert!(!evaluate(false, false, false, false)); // all false
        assert!(!evaluate(true, false, false, true));   // partial true
        // ... more cases to achieve independence
    }
}
```

## Running MC/DC Coverage

### Basic MC/DC Coverage

```bash
cargo +nightly llvm-cov --package <crate-name> --mcdc
```

### With HTML Report

```bash
cargo +nightly llvm-cov --package <crate-name> --mcdc --html
```

### JSON Output for Analysis

```bash
cargo +nightly llvm-cov --package <crate-name> --mcdc --json --output-path mcdc_report.json
```

## Safety-Critical Crates Requiring MC/DC

The following crates require 100% MC/DC coverage:

1. **wrt-runtime**: Core runtime execution
2. **wrt-instructions**: Instruction execution logic
3. **wrt-sync**: Synchronization primitives
4. **wrt-foundation**: Core types and operations
5. **wrt-platform**: Platform-specific safety features

## MC/DC Testing Patterns

### 1. Simple Boolean Conditions

```rust
fn safety_check(speed: u32, brake_pressed: bool) -> bool {
    speed < 100 || brake_pressed
}

#[test]
fn test_safety_check_mcdc() {
    // Case 1: speed affects outcome (brake_pressed = false)
    assert!(safety_check(50, false));   // speed < 100
    assert!(!safety_check(150, false)); // speed >= 100
    
    // Case 2: brake_pressed affects outcome (speed >= 100)
    assert!(!safety_check(150, false)); // brake not pressed
    assert!(safety_check(150, true));   // brake pressed
}
```

### 2. Complex Conditions

```rust
fn validate_operation(
    authorized: bool,
    within_limits: bool,
    emergency_override: bool,
    system_ready: bool
) -> bool {
    (authorized && within_limits && system_ready) || emergency_override
}

#[test]
fn test_validate_operation_mcdc() {
    // Test emergency_override independence
    assert!(validate_operation(false, false, true, false));  // override = true
    assert!(!validate_operation(false, false, false, false)); // override = false
    
    // Test authorized independence (when other conditions true)
    assert!(validate_operation(true, true, false, true));   // authorized = true
    assert!(!validate_operation(false, true, false, true)); // authorized = false
    
    // Test within_limits independence
    assert!(validate_operation(true, true, false, true));   // within_limits = true
    assert!(!validate_operation(true, false, false, true)); // within_limits = false
    
    // Test system_ready independence
    assert!(validate_operation(true, true, false, true));   // system_ready = true
    assert!(!validate_operation(true, true, false, false)); // system_ready = false
}
```

### 3. Nested Conditions

```rust
fn complex_validation(a: bool, b: bool, c: bool, d: bool) -> bool {
    if a && b {
        c || d
    } else {
        false
    }
}
```

## Analyzing MC/DC Results

### Understanding Coverage Reports

MC/DC coverage reports show:
- **Condition Coverage**: Each boolean sub-expression
- **Decision Coverage**: Overall boolean expressions
- **MC/DC Pairs**: Condition combinations that demonstrate independence

### Example Report Analysis

```
Function: validate_operation
  Decision at line 10: (authorized && within_limits && system_ready) || emergency_override
    Conditions:
      1. authorized         - Covered: Yes, Independence shown: Yes
      2. within_limits      - Covered: Yes, Independence shown: Yes
      3. system_ready       - Covered: Yes, Independence shown: Yes
      4. emergency_override - Covered: Yes, Independence shown: Yes
    MC/DC Coverage: 100%
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
- name: Run MC/DC Coverage
  run: |
    rustup install nightly
    rustup component add llvm-tools-preview --toolchain nightly
    cargo install cargo-llvm-cov
    cargo +nightly llvm-cov --workspace --mcdc --lcov --output-path mcdc.info
    
- name: Validate MC/DC Thresholds
  run: |
    # Extract MC/DC percentage and validate
    MCDC_PERCENT=$(cargo +nightly llvm-cov --workspace --mcdc --summary-only | grep "MCDC" | awk '{print $3}')
    if (( $(echo "$MCDC_PERCENT < 100.0" | bc -l) )); then
      echo "MC/DC coverage is below 100% for safety-critical components!"
      exit 1
    fi
```

## Best Practices

1. **Write Tests First**: Design tests to achieve MC/DC before implementation
2. **Use Truth Tables**: Create truth tables for complex conditions
3. **Minimize Complexity**: Keep boolean expressions simple when possible
4. **Document Rationale**: Explain why certain conditions exist
5. **Regular Validation**: Run MC/DC analysis in CI for every change

## Troubleshooting

### Common Issues

1. **"MC/DC not supported"**: Ensure using nightly Rust with LLVM 18+
2. **Low Coverage**: Check if all condition combinations are tested
3. **Performance**: MC/DC instrumentation adds overhead; use separate test runs

### Debugging MC/DC Coverage

```bash
# Generate detailed MC/DC report
cargo +nightly llvm-cov --package <crate> --mcdc --show-mcdc

# Export raw profiling data
LLVM_PROFILE_FILE=mcdc-%p-%m.profraw cargo +nightly test
cargo +nightly profdata merge -sparse mcdc-*.profraw -o mcdc.profdata
cargo +nightly cov show --use-color --show-mcdc-summary target/debug/deps/<test-binary> -instr-profile=mcdc.profdata
```

## Compliance Documentation

For safety certification, maintain:

1. **MC/DC Test Plan**: Document which conditions require MC/DC
2. **Coverage Reports**: Archive MC/DC reports for each release
3. **Justification**: Document any exceptions or deviations
4. **Traceability**: Link MC/DC tests to safety requirements

## Tools and Resources

- [LLVM MC/DC Documentation](https://llvm.org/docs/CoverageMappingFormat.html#mcdc-coverage)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [MC/DC Tutorial](https://www.rapitasystems.com/mcdc-coverage)