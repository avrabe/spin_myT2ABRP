# validation

Pure input validation utilities for user credentials and data with **zero dependencies** on Spin SDK.

## Overview

This component provides stateless validation functions for common input types including email validation, length checks, and credential validation. All functions are pure - no side effects, no I/O, fully deterministic.

## Features

- **Zero Spin SDK dependencies** - Pure WASI component
- **Pure functions** - No async, no HTTP, no state
- **Configurable limits** - Adjustable validation rules
- **Well-tested** - 13 unit tests, 100% passing
- **Minimal size** - 71KB release binary (smallest component!)
- **Comprehensive coverage** - Email, length, emptiness checks

## WIT Interface

```wit
interface validator {
    /// Validation configuration
    record validation-config {
        max-username-length: u32,
        max-password-length: u32,
    }

    /// Get default validation configuration
    get-default-config: func() -> validation-config;

    /// Validate user credentials
    validate-credentials: func(
        username: string,
        password: string,
        config: validation-config
    ) -> result<_, string>;

    /// Validate email format (basic check)
    is-valid-email: func(email: string) -> bool;

    /// Validate string length
    validate-string-length: func(
        value: string,
        field-name: string,
        min-length: u32,
        max-length: u32
    ) -> result<_, string>;

    /// Check if string is empty
    is-empty: func(value: string) -> bool;
}
```

## Validation Rules

### Credentials Validation

Checks performed by `validate-credentials`:

1. **Username (email)**:
   - ✅ Not empty
   - ✅ Length ≤ `max-username-length` (default: 256)
   - ✅ Contains '@' character
   - ✅ Contains '.' character

2. **Password**:
   - ✅ Not empty
   - ✅ Length ≤ `max-password-length` (default: 256)

### Email Validation

Basic email format validation (used for username):
- Must contain '@' character
- Must contain '.' character

**Note**: This is a simple format check, not RFC 5322 compliant validation. Suitable for basic filtering but not cryptographic verification.

### String Length Validation

Generic length validation with configurable min/max:
- Validates string length is within bounds
- Provides descriptive error messages
- Field name included in error for context

## Usage Example

```rust
use toyota_validation::*;

// Get default configuration
let config = get_default_config();
// config.max_username_length == 256
// config.max_password_length == 256

// Validate credentials
let result = validate_credentials(
    "user@example.com",
    "password123",
    config
);
assert!(result.is_ok());

// Email validation
assert!(is_valid_email("test@example.com"));
assert!(!is_valid_email("not-an-email"));

// Length validation
let result = validate_string_length(
    "hello",
    "field_name",
    1,   // min length
    10   // max length
);
assert!(result.is_ok());

// Custom configuration
let custom_config = ValidationConfig {
    max_username_length: 50,
    max_password_length: 32,
};

let result = validate_credentials(
    "short@example.com",
    "pass",
    custom_config
);
assert!(result.is_ok());
```

## Building

```bash
# Build component
cargo component build --release

# Run tests (native target)
cargo test --package toyota-validation --target x86_64-unknown-linux-gnu

# Size
ls -lh target/wasm32-wasip1/release/toyota_validation.wasm
# 71K (smallest component!)
```

## Dependencies

- `wit-bindgen-rt` - WIT bindings (only dependency!)

**No other dependencies** - Pure Rust standard library code.

## Test Coverage

All 13 tests passing:
- ✅ `test_validate_credentials_success`
- ✅ `test_validate_credentials_empty_username`
- ✅ `test_validate_credentials_empty_password`
- ✅ `test_validate_credentials_invalid_email`
- ✅ `test_validate_credentials_too_long_username`
- ✅ `test_validate_credentials_too_long_password`
- ✅ `test_is_valid_email_valid`
- ✅ `test_is_valid_email_invalid`
- ✅ `test_validate_string_length_success`
- ✅ `test_validate_string_length_too_short`
- ✅ `test_validate_string_length_too_long`
- ✅ `test_is_empty`
- ✅ `test_custom_config`

## Architecture

Part of the component-based Toyota integration system:

```
┌────────────────────────────────────┐
│  Gateway (imports via WAC)         │
│    toyota:validation/validator     │
└───────────────┬────────────────────┘
                │
                ↓
┌────────────────────────────────────┐
│  validation (71KB)                 │
│  - Credential validation           │
│  - Email format checks             │
│  - Length validation               │
│  - Zero Spin dependencies          │
│  - Pure functions                  │
└────────────────────────────────────┘
```

## Design Decisions

### Why basic email validation?

The component uses simple `contains('@') && contains('.')` validation because:
- **Good enough** - Catches obvious typos and formatting errors
- **Fast** - No regex, no complex parsing
- **Portable** - Works everywhere, no dependencies
- **Defensive** - Better to validate with backend/SMTP anyway

For production, always verify emails via confirmation link or SMTP check.

### Why configurable limits?

Different applications have different requirements:
- **APIs** - Might need shorter limits (reduce payload size)
- **Enterprise** - Might need longer limits (complex SSO usernames)
- **Testing** - Might need custom limits for specific scenarios

The default 256 characters is a reasonable balance.

### Why pure functions?

No async, no HTTP, no state:
- **Testable** - Deterministic, no mocking needed
- **Composable** - Easy to chain with other validation
- **Portable** - Runs anywhere (WASI, native, browser)
- **Fast** - Pure CPU, no I/O overhead

### Why minimal dependencies?

Only `wit-bindgen-rt` (required for WASM components):
- **Small** - Results in 71KB binary (smallest component!)
- **Secure** - Less surface area for vulnerabilities
- **Stable** - Fewer breaking changes
- **Fast** - Quick compile times

## Error Messages

All validation functions return descriptive error messages:

```rust
// Empty username
"Username cannot be empty"

// Empty password
"Password cannot be empty"

// Username too long
"Username exceeds maximum length of 256 characters"

// Password too long
"Password exceeds maximum length of 256 characters"

// Invalid email format
"Username must be a valid email address"

// String too short
"field_name must be at least 5 characters long (got 2)"

// String too long
"field_name must be at most 5 characters long (got 11)"
```

## Future Enhancements

Potential additions (not yet implemented):
- VIN validation (17-character alphanumeric)
- Phone number validation
- Strong password rules (complexity requirements)
- Internationalized email (RFC 6531)
- Custom validation rules (via callbacks)

These can be added without breaking the existing interface.

## License

Apache 2.0
