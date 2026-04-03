# AGENTS.md

## Build, Lint, and Test Commands

### Core Development
```bash
cargo build          # Build the FFI crate
cargo check          # Check for compilation errors without building artifacts
cargo test           # Run all tests in the crate
cargo test --test <name>  # Run specific test file
cargo test <pattern>      # Run tests matching pattern
```

### Code Quality
```bash
cargo clippy        # Lint for common mistakes and FFI-specific issues
cargo fmt            # Format code with rustfmt
cargo doc --open     # Generate and open documentation
cargo bench          # Run benchmarks (if any)
```

### Single Test Execution
```bash
# Run a single test function
cargo test test_function_name

# Run all tests in a specific test file
cargo test --test test_file_name

# Run tests with verbose output
cargo test -- --nocapture
```

## Code Style Guidelines

### File Organization
- Use clear section headers with `// ====` separators for logical grouping
- Organize into standard sections: Opaque Types, Enums, Structures, Callback Types, External Functions, Constants
- Group related functionality together in each section
- Keep sections vertically separated and clearly labeled

### Import Organization
```rust
use std::os::raw::{c_char, c_int, c_void};
```
- Keep imports minimal - only use raw C types needed for FFI bindings
- Organize imports alphabetically within each use statement
- No unnecessary imports

### Attribute Usage

**Allowlist Attributes for FFI**
```rust
#![doc = "FFI bindings for librist - Reliable Internet Stream Transport protocol library"]
#![allow(non_upper_case_globals)]   // Allow RIST_* constants instead of RIST_*
#![allow(non_camel_case_types)]     // Allow rist_* types instead of RistProfile
#![allow(non_snake_case)]           // Allow rist_* function names instead of rist_sender_create_

// Add FFI representation
#[repr(C)]
pub struct rist_ctx {
    _private: [u8; 0],
}

// Add traits to enums for debugging
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum rist_profile {
    RIST_PROFILE_SIMPLE = 0,
    RIST_PROFILE_MAIN = 1,
    RIST_PROFILE_ADVANCED = 2,
}
```

**Rep C Attributes**
- Always use `#[repr(C)]` for types that cross FFI boundary
- Ensures memory layout matches C implementation
- Opaque types use zero-length arrays: `#[repr(C)] pub struct name { _private: [u8; 0] }`

### Documentation Style
- Crate-level doc using `#![doc = "..."]` for high-level description
- Minimal inline documentation - keep it simple and descriptive
- Use descriptive type/enum names instead of verbose rustdoc
- Group logical sections with section headers
- Include purpose of each section in comments

### Naming Conventions

**Type Naming**
- Prefix all types with `rist_*` (e.g., `rist_ctx`, `rist_peer`, `rist_stats`)
- Enums follow `rist_*` + `upper_snake_case` (e.g., `rist_profile`, `rist_log_level`)
- Opaque types use `rist_*` prefix (e.g., `rist_ctx`, `rist_peer`)

**Function Naming**
- Prefix all exported functions with `rist_*` (e.g., `rist_start`, `rist_sender_create`)
- Use version suffixes for newer APIs: `receiver_data_read2`, `rist_peer_config_free2`
- Verb prefixes for actions: `rist_*_create`, `rist_*_destroy`, `rist_*_start`, `rist_*_stop`

**Constant Naming**
- Prefix all constants with `RIST_*` (e.g., `RIST_PEER_CONFIG_VERSION`, `RIST_DEFAULT_VIRT_DST_PORT`)
- Use `RIST_DEFAULT_*` pattern for configuration values
- Use `RIST_` prefix for enum values (already follows pattern in enum definitions)

**Callback Types**
- Suffix with `*_callback_t` or `*_t` (e.g., `receiver_data_callback2_t`, `connection_status_callback_t`)
- Use Option wrapper for safety: `Option<unsafe extern "C" fn(...) -> c_int>`

### FFI-Specific Patterns

**Opaque Type Definition**
```rust
#[repr(C)]
pub struct rist_ctx {
    _private: [u8; 0],  // Zero-length array prevents direct access
}
```

**Callback Type Definition**
```rust
pub type receiver_data_callback2_t =
    Option<unsafe extern "C" fn(*mut c_void, *mut rist_data_block) -> c_int>;
```

**Double Pointers for Creation**
- Use `*mut *mut` for create functions that allocate and return context
- Pattern: `fn rist_sender_create(ctx: *mut *mut rist_ctx, ...) -> c_int`

**Memory Management**
- Always pair create functions with free functions
- Use `free2` suffix for manual allocation cleanup (e.g., `rist_peer_config_free2`)
- Handle double pointers appropriately in wrappers

## Error Handling

### FFI Return Values
- All FFI functions return `c_int` for success or error
- Zero indicates success, non-zero (often negative) indicates error
- No custom error types used - interpret raw codes directly
- Negative values follow C convention

### Error Check Pattern
```rust
let result = rist_start(ctx);
if result != 0 {
    // Handle error - interpret the code (usually negative means error)
    eprintln!("Error starting rist: {}", result);
    return;
}
```

## Underlying Library Context

### Dependencies
- **pkg-config**: Required to find and link librist library
- **librist C library**: Written in C99 (see librist/CONTRIBUTING.md)

### Feature-Based Linking
```toml
[features]
default = []
static = []  # Enables static linking when cargo build --features static
```

### C99 Standards Reference
- See `librist/CONTRIBUTING.md` for complete guidelines
- C99 without VLA or Complex features
- No compiler extensions allowed
- Use modern POSIX functions with fallbacks

## Testing

### Test Structure
- Place tests in dedicated files within `src/` directory
- Name test files ending with `_test.rs`
- Tests must be compatible with FFI patterns (raw pointers, c_int)
- Keep tests focused and independent

### Test Pattern Example
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rist_start_and_destroy() {
        let mut ctx: *mut rist_ctx = std::ptr::null_mut();
        let result = rist_start(&mut ctx);
        assert_eq!(result, 0, "Failed to start rist context");
        
        // Clean up
        let cleanup_result = rist_destroy(ctx);
        assert_eq!(cleanup_result, 0, "Failed to destroy rist context");
    }
}
```

### Test Requirements
- Always check return values from FFI functions
- Verify zero return value for success
- Clean up allocated resources properly after tests
- Handle edge cases and error conditions
- Tests should demonstrate proper FFI usage patterns
- Use assertions for expected behavior and error codes

### Running Tests
```bash
cargo test                    # Run all tests
cargo test --test <name>      # Run tests in specific file
cargo test <pattern>          # Run tests matching pattern
cargo test -- --nocapture     # Run tests with output
cargo test -- --show-output   # Show all output from test runs
cargo test -- --test-threads=1  # Run tests serially
```

## Additional Notes

### Working with Raw Pointers
- Be cautious with raw pointers to C memory
- Always validate pointer values before dereferencing
- Use `std::ptr::null_mut()` for uninitialized pointers
- Consider wrapper types for safer access to C structures

Always use Context7 when I need library/API documentation, code generation, setup or configuration steps without me having to explicitly ask.