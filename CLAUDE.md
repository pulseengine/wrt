# WRT (WebAssembly Runtime) Project Guidelines

## Project Status
This is a work-in-progress WebAssembly runtime implementation. The codebase needs fixes before it will build:
1. Fix missing `format!` and `vec!` macro imports in no_std context
2. Add PartialEq derives for MemoryType and GlobalType
3. Resolve recursive Value type with Box<T>
4. Fix type mismatches between u32 and usize in memory operations
5. Add the missing get_label method to Stack or update call sites

## Build Commands
- Build all: `just build` or `cargo build`
- Build specific crate: `cargo build -p wrt|wrtd|example`
- Clean: `just clean` or `cargo clean`
- Run tests: `just test` or `cargo test`
- Run single test: `cargo test -p wrt -- test_name --nocapture`
- Format code: `just fmt` or `cargo fmt`
- Lint: `just check` or `cargo clippy`

## Code Style Guidelines
- Use 4-space indentation
- Follow Rust naming conventions: snake_case for functions/variables, CamelCase for types
- Organize imports in the following order:
  1. Standard library imports (std, core, alloc)
  2. External crates/third-party dependencies
  3. Internal modules (crate:: imports)
  4. Each group should be separated by a blank line
- Always derive Debug, Clone, PartialEq for data structures
- Use thiserror for error definitions
- Handle no_std environments with appropriate macro imports
- Write tests for new functionality
- Fix type consistency:
  - WebAssembly spec uses u32 for sizes
  - Convert between u32 and usize explicitly when working with Rust memory
- Break cyclic references with Box<T> for recursive types

## Commit Message Guidelines

Use the Conventional Commits format for all commit messages:

```
<type>(<scope>): <short summary>

<detailed description>

<footer>
```

Types:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, indentation)
- `refactor`: Code refactoring without functionality changes
- `perf`: Performance improvements
- `test`: Adding or modifying tests
- `chore`: Routine tasks, maintenance, tooling changes
- `ci`: Changes to CI configuration and scripts
- `build`: Changes to build system or dependencies

Examples:
- `feat(execution): implement fuel-bounded execution`
- `fix(memory): correct address boundary checking`
- `docs(api): add examples for module instantiation`
- `test(component): add test cases for component linking`

## Documentation Guidelines

All public API must be documented thoroughly using doc comments (`///`). Documentation should follow this pattern:

```rust
/// Brief description of what the function/type does.
///
/// More detailed explanation if needed. Explain the purpose and context.
/// Include code examples where appropriate.
///
/// # Examples
///
/// ```
/// use wrt::YourType;
/// let instance = YourType::new();
/// assert!(instance.is_valid());
/// ```
///
/// # Parameters
///
/// * `param1` - Description of the first parameter
/// * `param2` - Description of the second parameter
///
/// # Returns
///
/// Description of the return value and what it represents.
///
/// # Errors
///
/// This function will return an error if:
/// * The input is invalid (with Error::Validation)
/// * The operation cannot be completed (with Error::Execution)
///
/// # Panics
///
/// This function will panic if a required invariant is violated,
/// specifically if the internal state is inconsistent.
```

Prioritize documenting:
1. Public functions and methods
2. Public types and their fields
3. Error conditions for functions that return Results
4. Panic conditions for functions that may panic
5. Examples for non-trivial functions

## Reference Implementations
- Wasmtime: https://github.com/bytecodealliance/wasmtime
- Wasmer: https://github.com/wasmerio/wasmer
- WebAssembly spec: https://webassembly.github.io/spec/