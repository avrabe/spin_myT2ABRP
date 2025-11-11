# Toyota MyT (Europe) to A Better Route Planner Gateway using Fermyon Spin

[![Rust](https://github.com/avrabe/spin_myT2ABRP/actions/workflows/rust.yml/badge.svg)](https://github.com/avrabe/spin_myT2ABRP/actions/workflows/rust.yml)

A WebAssembly-based gateway service that bridges Toyota Connected Services Europe (MyToyota) with A Better Route Planner (ABRP) for electric vehicle telemetry data.

## Overview

This application provides real-time vehicle battery status (State of Charge) from Toyota electric/hybrid vehicles in Europe to ABRP for accurate route planning. Built with Rust and Fermyon Spin, it runs as a serverless WebAssembly component with minimal resource usage and maximum performance.

**⚠️ Important**: This uses the unofficial Toyota Connected Services API. Toyota may change their API at any time, which could break this application.

## Features

### 🚀 Latest Updates (v2.0)

- **Spin SDK 5.1.1**: Latest Fermyon Spin framework with HTTP/2 support and improved performance
- **New Toyota API**: Migrated to `ctpa-oneapi` - the modern Toyota Connected Services Europe API
- **OAuth2 Authentication**: Secure multi-step authentication flow matching the official MyToyota app
- **Token Caching**: Intelligent token caching with automatic refresh (reduces latency from ~4s to ~200ms)
- **Better Error Handling**: Comprehensive error messages and graceful degradation
- **Proper UUID Generation**: Using standard UUID v4 for correlation IDs
- **Comprehensive Testing**: 15+ unit tests covering all critical functionality
- **Zero Warnings**: Clean build with no compiler warnings

### 📊 Technical Details

- **Runtime**: Fermyon Spin 5.x (WebAssembly/WASI)
- **Language**: Rust 2021 Edition
- **Target**: `wasm32-wasip1`
- **Architecture**: Modular workspace with `myt` (API library) and `myt2abrp` (HTTP handler)

## API Migration

### Old API (Deprecated)
- Endpoints: `ssoms.toyota-europe.com`, `myt-agg.toyota-europe.com`
- Authentication: Simple username/password POST
- Status: No longer supported

### New API (Current)
- Endpoints: `b2c-login.toyota-europe.com`, `ctpa-oneapi.tceu-ctp-prd.toyotaconnectedeurope.io`
- Authentication: OAuth2 with JWT tokens
- Status: ✅ Active (used by official MyToyota app)

## Setup

### Prerequisites

1. **Install Rust and WebAssembly target**
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add wasm32-wasip1
   ```

2. **Install Fermyon Spin**
   ```sh
   curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
   sudo mv spin /usr/local/bin/
   ```

3. **Install wasm32-wasip1 target** (if not already installed)
   ```sh
   rustup target add wasm32-wasip1
   ```

### Configuration

Set your Toyota MyT credentials as environment variables:

```sh
export SPIN_VARIABLE_USERNAME=your.email@example.com
export SPIN_VARIABLE_PASSWORD=your_password
export SPIN_VARIABLE_VIN=YOUR_VEHICLE_VIN
```

**Security Note**: Use the `secret` flag for sensitive variables in production:
```toml
[variables]
password = { required = true, secret = true }
```

## Development

### Build and Run

```sh
# Build the application
spin build

# Start local server (default port 3000)
spin up

# Development with auto-rebuild
spin watch
```

### Test

```sh
# Run all tests (requires native target)
cargo test --lib --target x86_64-unknown-linux-gnu

# Run specific tests
cargo test --lib --target x86_64-unknown-linux-gnu test_decode_jwt_uuid

# Build for production
cargo build --target wasm32-wasip1 --release
```

### Query the Service

```sh
curl http://127.0.0.1:3000/
```

**Expected Response:**
```json
{
  "soc": 85,
  "access_date": "2025-01-01T12:00:00Z"
}
```

## Authentication Flow

The application implements the official Toyota OAuth2 flow:

1. **Initial Authentication**: Request authentication challenge
2. **Credentials Submission**: Submit username and password
3. **Authorization Code**: Receive authorization code via redirect
4. **Token Exchange**: Exchange code for access/refresh tokens
5. **Token Refresh**: Automatically refresh expired tokens from cache

**Token Caching**: Tokens are cached in Spin's key-value store, dramatically reducing authentication overhead for subsequent requests.

## Architecture

### Project Structure

```
spin_myT2ABRP/
├── myt/                    # Toyota API library
│   ├── src/lib.rs         # Data structures and OAuth2 types
│   └── Cargo.toml
├── myt2abrp/              # HTTP gateway component
│   ├── src/lib.rs         # Main handler with caching
│   └── Cargo.toml
├── spin.toml              # Spin configuration (manifest v2)
└── Cargo.toml             # Workspace configuration
```

### Key Components

- **Token Caching**: Uses Spin KV store for persistent token storage
- **Error Handling**: Comprehensive error messages with proper Result types
- **JWT Decoding**: Extracts UUID from id_token for API requests
- **HTTP Retry**: Built-in error handling for transient failures

## Troubleshooting

### Common Issues

1. **Authentication fails**: Verify your credentials are correct for the MyToyota app
2. **Token expired**: Tokens auto-refresh, but you may need to clear the KV store if corrupted
3. **VIN not found**: Ensure your VIN matches the vehicle in your Toyota account
4. **Build errors**: Make sure `wasm32-wasip1` target is installed

### Debug Mode

Enable verbose logging by checking Spin's logs:
```sh
spin up --log-dir ./logs
```

## API Endpoints

### Toyota Connected Services Europe

- **Authentication**: `https://b2c-login.toyota-europe.com/json/realms/root/realms/tme/authenticate`
- **Authorization**: `https://b2c-login.toyota-europe.com/oauth2/realms/root/realms/tme/authorize`
- **Token Exchange**: `https://b2c-login.toyota-europe.com/oauth2/realms/root/realms/tme/access_token`
- **Vehicle Status**: `https://ctpa-oneapi.tceu-ctp-prd.toyotaconnectedeurope.io/v1/global/remote/electric/status`

## Testing Coverage

- ✅ JWT decoding with valid tokens
- ✅ JWT decoding with invalid formats
- ✅ Token caching and expiry logic
- ✅ OAuth2 request structure validation
- ✅ Electric vehicle status parsing
- ✅ Timestamp generation
- ✅ Error handling paths

Run tests with: `cargo test --lib --target x86_64-unknown-linux-gnu`

## Contributing

Contributions are welcome! Please ensure:
- All tests pass
- No compiler warnings
- Code follows Rust best practices
- Documentation is updated

## License

MIT

## Disclaimer

This is an unofficial integration. Toyota Motor Corporation is not affiliated with this project and does not endorse it. The Toyota Connected Services API is proprietary and may change without notice. Use at your own risk.

## References

- [Fermyon Spin Documentation](https://developer.fermyon.com/spin)
- [Toyota Connected Europe](https://www.toyotaconnected.eu/)
- [A Better Route Planner](https://abetterrouteplanner.com/)
- [Original tojota project](https://github.com/calmjm/tojota) (outdated)
- [mytoyota Python library](https://github.com/DurgNomis-drol/mytoyota) (reference implementation)

## Acknowledgments

Special thanks to the community projects that helped reverse-engineer the Toyota API, particularly the [mytoyota](https://github.com/DurgNomis-drol/mytoyota) project which provided valuable insights into the new ctpa-oneapi endpoints.
