# AGENTS.md - Coding Guidelines for pjrt-rs

This file provides guidance for AI agents working in the pjrt-rs repository.

## Build, Lint, and Test Commands

### Building
```bash
# Build the entire workspace
cargo build --verbose

# Build a specific package
cargo build -p pjrt
cargo build -p pjrt-sys

# Build for release
cargo build --release
```

### Code Formatting
```bash
# Format all code (requires nightly toolchain)
rustup component add rustfmt --toolchain nightly
cargo +nightly fmt --all

# Check formatting without modifying
cargo +nightly fmt --all -- --check
```

### Linting
```bash
# Run Clippy on workspace with tests and examples
cargo clippy --workspace --tests --examples -- -D warnings

# Install required components
rustup component add clippy rustfmt
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for a specific package
cargo test -p pjrt
cargo test -p pjrt-sys

# Run a single test by name
cargo test test_error_code_values
cargo test -p pjrt test_client_debug_impl

# Run tests with output
cargo test -- --nocapture

# Note: Integration tests and examples require PJRT_PLUGIN_PATH to be set
cargo test -- --nocapture 2>&1 | head -20  # See which tests run/fail

# Run tests in release mode (faster for heavy tests)
cargo test --release
```

### Prerequisites
- Install protoc: Required for building (uses prost for protobuf)
- Rust nightly toolchain: Required for rustfmt unstable features
- PJRT Plugin: Required for integration tests and examples, assume it is configurated by user.
  - Set `PJRT_PLUGIN_PATH` environment variable to the path of the PJRT plugin (e.g., `pjrt_c_api_cpu_plugin.so`)
  - Example: `export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so`

## Code Style Guidelines

### Rust Edition and Toolchain
- **Edition**: 2021
- **Toolchain**: Nightly (required for rustfmt unstable features)

### Formatting (rustfmt.toml)
```toml
newline_style = "Unix"
use_field_init_shorthand = true
style_edition = "2021"
imports_granularity = "Module"
group_imports = "StdExternalCrate"
format_code_in_doc_comments = true
format_macro_bodies = true
format_macro_matchers = true
```

### Import Organization
Imports must be grouped in this order (enforced by rustfmt):
1. **Standard library**: `use std::...`
2. **External crates**: `use bon::...`, `use pjrt_sys::...`
3. **Internal (crate)**: `use crate::...`

Example:
```rust
use std::borrow::Cow;
use std::ffi::c_void;
use std::rc::Rc;
use std::slice;

use bon::bon;
use pjrt_sys::{PJRT_Client, PJRT_Error_Code};

use crate::{Api, Buffer, Device, Result};
```

### Naming Conventions
- **Types**: PascalCase (e.g., `Client`, `HostBuffer`, `LoadedExecutable`)
- **Functions/Methods**: snake_case (e.g., `create_client`, `platform_name`)
- **Constants**: SCREAMING_SNAKE_CASE for true constants
- **Modules**: snake_case (e.g., `mod host_buffer;`)
- **Generic parameters**: Single uppercase letters or descriptive names (e.g., `T`, `Item`)

### Error Handling
- Use `thiserror` derive macro for error types
- Define a crate-wide `Result<T>` type alias
- Use `?` operator for error propagation
- Provide detailed error messages with context

Example:
```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("pjrt error {msg}\n{backtrace}")]
    PjrtError { msg: String, code: ErrorCode, backtrace: String },
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Types and Type Safety
- Prefer strong types over primitive types (e.g., `GlobalDeviceId`, `LocalHardwareId`)
- Use `#[repr(i32)]` for C-compatible enums
- Implement `Debug` for all public types
- Use `Cow<'_, str>` for string data that may be borrowed or owned

### Documentation
- All public items must have doc comments (`///`)
- Module-level documentation uses `//!`
- Document panics, safety requirements, and examples
- Use markdown in doc comments

Example:
```rust
//! PJRT Client
//!
//! This module provides the `Client` struct which represents a PJRT runtime instance.

/// Creates a buffer that carries an error future without allocating memory.
///
/// If this buffer is passed to an Execute call, the execution will fail
/// with the given error code and message.
pub fn create_error_buffer(...)
```

### Unsafe Code
- Minimize unsafe code blocks
- Always document safety requirements with `/// # Safety` section
- Mark unsafe functions with `unsafe fn`

### Builder Pattern
- Use `bon` crate for builder macros
- Use `#[builder(finish_fn = build)]` for custom finish function names
- Use `#[builder(start_fn)]` for required parameters in builder

Example:
```rust
#[bon]
impl Client {
    #[builder(finish_fn = build)]
    pub fn builder(
        #[builder(start_fn)] api: &Api,
        #[builder(default = bon::vec![], into)] options: Vec<NamedValue>,
    ) -> Result<Self> {
        // ...
    }
}
```

### Testing
- Write inline tests in `#[cfg(test)]` modules at the bottom of files
- Test both success and error cases
- Use descriptive test names
- Mock external dependencies when possible

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_from_pjrt_error_code() {
        let code: ErrorCode = PJRT_Error_Code_PJRT_Error_Code_CANCELLED
            .try_into()
            .unwrap();
        assert_eq!(code, ErrorCode::Cancel);
    }
}
```

### Workspace Structure
- `pjrt/`: High-level safe API
- `pjrt-sys/`: Low-level FFI bindings (generated via bindgen)
- Both crates use workspace dependencies defined in root `Cargo.toml`

### Common Patterns
- Resource management: Use RAII with `Drop` implementations
- FFI wrappers: Raw pointers wrapped in safe structs with lifetimes
- Extension traits: Use traits to add functionality to existing types
- Type conversions: Implement `From`/`TryFrom` for type safety

## IDE Configuration

For VSCode with rust-analyzer, use these settings:
```json
{
    "rust-analyzer.checkOnSave": false,
    "rust-analyzer.imports.prefix": "crate",
    "rust-analyzer.imports.granularity.group": "module",
    "rust-analyzer.rustfmt.extraArgs": ["+nightly"],
    "editor.formatOnSave": true
}
```

## CI/CD (GitHub Actions)

The repository uses GitHub Actions with the following jobs:
- **build**: Builds on Ubuntu, Windows, and macOS using nightly Rust
- **fmt**: Checks code formatting with nightly rustfmt
- **clippy**: Runs clippy with `-D warnings` (denies all warnings)

All PRs must pass these checks before merging.

## Rules
- Don't leave placeholder implmentations, implment it when it is required
- Don't crete simply versions for the requirement, implment well designed and tested version