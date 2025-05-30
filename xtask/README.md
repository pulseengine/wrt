# Xtask - WRT Development Automation

Xtask is the cross-platform task automation tool for the WRT project, replacing shell scripts with Rust-based commands.

## Overview

Xtask provides a consistent interface for development tasks across all platforms (Linux, macOS, Windows), eliminating the need for platform-specific shell scripts.

## Installation

No installation needed! Xtask is part of the workspace:

```bash
# Show all available commands
cargo xtask --help
```

## Available Commands

### Testing Commands

#### `run-tests`
Run the unified test suite.
```bash
cargo xtask run-tests
```

#### `verify-no-std`
Verify no_std compatibility across all WRT crates.
```bash
# Full verification
cargo xtask verify-no-std

# Quick partial verification (tests half the crates)
cargo xtask verify-no-std --partial

# Continue on errors to see all failures
cargo xtask verify-no-std --continue-on-error

# Verbose output with detailed summary
cargo xtask verify-no-std --verbose --detailed
```

### Documentation Commands

#### `publish-docs-dagger`
Build and publish documentation using Dagger.
```bash
cargo xtask publish-docs-dagger --output-dir docs_output
```

#### `preview-docs`
Start a local HTTP server to preview documentation.
```bash
# Start server on default port 8000
cargo xtask preview-docs

# Custom port and auto-open browser
cargo xtask preview-docs --port 8080 --open-browser

# Serve from custom directory
cargo xtask preview-docs --docs-dir path/to/docs
```

#### `validate-docs`
Validate that required documentation files exist.
```bash
cargo xtask validate-docs
```

#### `validate-docs-comprehensive`
Run comprehensive documentation validation including structure and link checks.
```bash
cargo xtask validate-docs-comprehensive
```

#### `generate-changelog`
Generate project changelog using git-cliff.
```bash
# Generate full changelog
cargo xtask generate-changelog

# Generate only unreleased changes
cargo xtask generate-changelog --unreleased

# Specify custom output file
cargo xtask generate-changelog --output CHANGELOG.md

# Install git-cliff automatically if missing
cargo xtask generate-changelog --install-if-missing
```

#### `deploy-docs-sftp`
Deploy documentation to SFTP hosting (e.g., shared hosting providers).
```bash
# Deploy with environment variables
export SFTP_HOST="your-server-ip"
export SFTP_USERNAME="your-username"
export SFTP_SSH_KEY_PATH="~/.ssh/id_rsa"
cargo xtask deploy-docs-sftp

# Deploy with command line options
cargo xtask deploy-docs-sftp \
  --host your-server-ip \
  --username your-username \
  --ssh-key-path ~/.ssh/id_rsa \
  --target-dir /htdocs \
  --build-docs

# Dry run to see what would be deployed
cargo xtask deploy-docs-sftp --dry-run

# Deploy to custom directory
cargo xtask deploy-docs-sftp --target-dir /public_html/docs
```

### Code Quality Commands

#### `fmt-check`
Check code formatting without making changes.
```bash
cargo xtask fmt-check
```

#### `coverage`
Generate code coverage reports.
```bash
cargo xtask coverage
```

#### `coverage-simple`
Generate simple coverage without Dagger (faster).
```bash
cargo xtask coverage-simple
```

#### `coverage-comprehensive`
Generate comprehensive coverage analysis.
```bash
cargo xtask coverage-comprehensive
```

### CI Commands

#### `ci-static-analysis`
Run static analysis checks (clippy, formatting, etc.).
```bash
cargo xtask ci-static-analysis
```

#### `ci-advanced-tests`
Run advanced test suite including integration tests.
```bash
cargo xtask ci-advanced-tests
```

#### `ci-integrity-checks`
Run integrity checks on the codebase.
```bash
cargo xtask ci-integrity-checks
```

### Utility Commands

#### `fs`
File system operations (cross-platform).
```bash
# Remove directory recursively
cargo xtask fs rm-rf path/to/dir

# Create directory with parents
cargo xtask fs mkdir-p path/to/dir

# Find and delete files matching pattern
cargo xtask fs find-delete . "*.tmp"

# Count files matching pattern
cargo xtask fs count-files . "*.rs"

# Copy files
cargo xtask fs cp source.txt dest.txt
```

#### `wasm`
WebAssembly utilities.
```bash
# Build all WAT files in directory
cargo xtask wasm build path/to/wat/files

# Check WAT files for errors
cargo xtask wasm check path/to/wat/files

# Convert WAT to WASM
cargo xtask wasm convert file.wat
```

#### `generate-source-needs`
Generate source needs documentation for qualification.
```bash
cargo xtask generate-source-needs
```

#### `generate-coverage-summary`
Generate coverage summary for documentation.
```bash
cargo xtask generate-coverage-summary
```

## Command Options

### Global Options
- `--workspace-root <PATH>` - Path to workspace root (default: ./)
- `--log-level <LEVEL>` - Logging level: trace, debug, info, warn, error (default: info)

## Script Migration

The following shell scripts have been migrated to xtask commands:

| Old Script | New Command |
|------------|-------------|
| `scripts/verify_no_std.sh` | `cargo xtask verify-no-std` |
| `scripts/preview_docs.sh` | `cargo xtask preview-docs` |
| `scripts/test_wrt_logging.sh` | `cargo xtask verify-no-std` (includes logging tests) |
| `scripts/generate_changelog.sh` | `cargo xtask generate-changelog` |
| Manual SFTP deployment | `cargo xtask deploy-docs-sftp` |
| Direct `cargo test` commands | `cargo xtask run-tests` |

## Development

To add new commands:

1. Create a new module in `xtask/src/`
2. Add the command to the `Command` enum in `main.rs`
3. Handle the command in the match statement
4. Update this README

## Benefits

- **Cross-platform**: Works on Linux, macOS, and Windows
- **Type-safe**: Rust's type system prevents many errors
- **Integrated**: Direct access to Cargo and workspace metadata
- **Fast**: Compiled Rust code runs faster than shell scripts
- **Maintainable**: Easier to debug and extend than shell scripts