# Crate Documentation Standards

This document provides templates and guidelines for standardizing documentation across all crates in the WRT project.

## Cargo.toml Metadata

Every crate should include the following metadata fields:

```toml
[package]
name = "wrt-example"
version.workspace = true
edition.workspace = true
description = "Clear description of the crate's purpose and functionality"
license.workspace = true
repository = "https://github.com/avrabe/wrt"
documentation = "https://docs.rs/wrt-example"
keywords = ["wasm", "webassembly", "runtime", "component-model"]
categories = ["wasm", "no-std"]  # Add other relevant categories

# Docs.rs configuration should be included for all crates
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

## README.md Template

Every crate should have a README.md file with the following structure:

```markdown
# WRT Example

Brief one-line description of the crate.

Detailed description (2-3 sentences) explaining the purpose and context of the crate within the WRT ecosystem.

## Features

- **Feature One** - Description of first feature
- **Feature Two** - Description of second feature
- **Feature Three** - Description of third feature

## Usage

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
wrt-example = "0.2.0"
```

### Example

```rust
use wrt_example::SomeType;

// Example code showing basic usage
let example = SomeType::new();
example.do_something();
```

## Feature Flags

- `std` (default): Use the standard library
- `alloc`: Enable allocation support without std
- `no_std`: Enable complete no_std support
- Additional feature flags specific to this crate...

## No-std Usage

To use this crate in a `no_std` environment:

```toml
[dependencies]
wrt-example = { version = "0.2.0", default-features = false, features = ["no_std", "alloc"] }
```

## License

This project is licensed under the MIT License.
```

## lib.rs Documentation Template

The `lib.rs` file should have comprehensive crate-level documentation:

```rust
//! # WRT Example
//!
//! Brief description of the crate.
//!
//! Detailed description (2-3 paragraphs) explaining the purpose, context, and
//! functionality of the crate within the WRT ecosystem.
//!
//! ## Features
//!
//! - **Feature One**: Description of first feature
//! - **Feature Two**: Description of second feature
//! - **Feature Three**: Description of third feature
//!
//! ## Usage Example
//!
//! ```rust
//! use wrt_example::SomeType;
//!
//! // Example code showing basic usage
//! let example = SomeType::new();
//! example.do_something();
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::missing_panics_doc)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Rest of the file...

/// Brief description of the module.
///
/// Detailed description explaining the purpose and functionality of the module.
/// Include examples where appropriate.
pub mod example_module;
```

## Module-Level Documentation

Each module should have descriptive documentation:

```rust
/// Provides functionality for X.
///
/// This module contains types and functions related to X, which are used to...
///
/// # Examples
///
/// ```
/// use wrt_example::module_name::{Type, function};
///
/// let value = Type::new();
/// function(value);
/// ```
pub mod module_name {
    // Module contents...
}
```

## Type and Function Documentation

All public types and functions should be documented:

```rust
/// A brief description of what this type represents.
///
/// A more detailed explanation of the type, its purpose, and how it fits into 
/// the broader architecture.
///
/// # Examples
///
/// ```
/// use wrt_example::Type;
///
/// let instance = Type::new(42);
/// assert_eq!(instance.value(), 42);
/// ```
pub struct Type {
    // Fields...
}

impl Type {
    /// Creates a new instance of `Type`.
    ///
    /// # Arguments
    ///
    /// * `value` - The initial value to store.
    ///
    /// # Returns
    ///
    /// A new `Type` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use wrt_example::Type;
    ///
    /// let instance = Type::new(42);
    /// ```
    pub fn new(value: i32) -> Self {
        // Implementation...
    }
}
```

## Standardization Checklist

For each crate:

1. [ ] Update Cargo.toml metadata
2. [ ] Create or update README.md
3. [ ] Improve lib.rs crate-level documentation
4. [ ] Add module-level documentation
5. [ ] Document all public types and functions
6. [ ] Add usage examples
7. [ ] Document feature flags
8. [ ] Add no_std usage information if applicable 