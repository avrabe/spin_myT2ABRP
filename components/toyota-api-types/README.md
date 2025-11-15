# toyota-api-types

Pure data models for Toyota Connected Services APIs with **zero dependencies** on Spin SDK.

## Overview

This component provides type-safe Serde-compatible data structures for integrating with Toyota Connected Services APIs. All types are pure Rust with no platform-specific dependencies, making them reusable across any WASM runtime or native application.

## Features

- **Zero Spin SDK dependencies** - Pure WASI component
- **Comprehensive type coverage** - Authentication, vehicles, telemetry, location, charging status
- **JSON serialization/deserialization** - Full Serde support
- **WIT interface** - Helper functions for common operations
- **Well-tested** - 9 unit tests, 100% passing
- **Optimized** - 240KB release binary

## API Coverage

### Authentication & OAuth2
- `AuthenticateRequest` / `AuthenticateResponse`
- `TokenRequest` / `TokenResponse`
- `RefreshTokenRequest`
- `CachedToken` with expiration handling
- `JwtPayload`
- `CustomerProfile`

### Vehicle Data
- `VehicleListResponse` - List of user's vehicles
- `VehicleInfo` - Vehicle metadata (VIN, model, year, alias)

### Electric Vehicle Status
- `ElectricStatusResponse` - Battery and charging data
- `NewChargeInfo` - SOC, range, charging status, time remaining

### Location
- `LocationResponse` - GPS coordinates
- `LocationData` - Latitude/longitude with timestamp

### Telemetry
- `TelemetryResponse` - Odometer and fuel data
- `TelemetryOdometer` / `TelemetryFuel`

## WIT Interface

The component exports helper functions via WIT:

```wit
interface models {
    // Serialization helpers
    serialize-token-response: func(...) -> string;
    deserialize-token-response: func(json: string) -> result<...>;

    // Constructors
    create-auth-request: func(username: string, password: string, ...) -> string;
    create-token-request: func(code: string) -> string;
    create-refresh-request: func(refresh-token: string) -> string;

    // Utilities
    is-token-expired: func(expires-at: s64, current-time: s64) -> bool;
}
```

## Usage Example

```rust
use toyota_api_types::{TokenRequest, TokenResponse};

// Create a token request
let req = TokenRequest::new("auth-code-123".to_string());

// Serialize to JSON
let json = serde_json::to_string(&req)?;

// Deserialize response
let response: TokenResponse = serde_json::from_str(&response_json)?;

// Check token expiration
let cached = CachedToken::from_token_response(response, uuid, current_time);
if cached.is_expired(current_time) {
    // Refresh token
}
```

## Building

```bash
# Build component
cargo component build --release

# Run tests (native target)
cargo test --package toyota-api-types --target x86_64-unknown-linux-gnu

# Size
ls -lh target/wasm32-wasip1/release/toyota_api_types.wasm
# 240K
```

## Dependencies

- `serde` - Serialization framework
- `serde_json` - JSON support
- `serde_path_to_error` - Better error messages
- `wit-bindgen-rt` - WIT bindings

## Test Coverage

All 9 tests passing:
- ✅ `test_authenticate_request_new`
- ✅ `test_authenticate_request_with_credentials`
- ✅ `test_token_request_new`
- ✅ `test_refresh_token_request_new`
- ✅ `test_cached_token_from_token_response`
- ✅ `test_cached_token_is_expired`
- ✅ `test_electric_status_response_structure`
- ✅ `test_electric_status_optional_fields`
- ✅ `test_token_response_serialization`

## Architecture

Part of the component-based Toyota integration system:

```
┌────────────────────────────────────┐
│  Gateway (imports via WAC)         │
│    toyota:api-types/models         │
└───────────────┬────────────────────┘
                │
                ↓
┌────────────────────────────────────┐
│  toyota-api-types (240KB)          │
│  - Pure data models                │
│  - Zero Spin dependencies          │
│  - Reusable across projects        │
└────────────────────────────────────┘
```

## License

Apache 2.0
