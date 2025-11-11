# Toyota MyT (Europe) to A Better Route Planner Gateway using Fermyon Spin

[![Rust](https://github.com/avrabe/spin_myT2ABRP/actions/workflows/rust.yml/badge.svg)](https://github.com/avrabe/spin_myT2ABRP/actions/workflows/rust.yml)

A WebAssembly-based gateway service that bridges Toyota Connected Services Europe (MyToyota) with A Better Route Planner (ABRP) for electric vehicle telemetry data.

## Overview

This application provides real-time vehicle battery status (State of Charge) from Toyota electric/hybrid vehicles in Europe to ABRP for accurate route planning. Built with Rust and Fermyon Spin, it runs as a serverless WebAssembly component with minimal resource usage and maximum performance.

**⚠️ Important**: This uses the unofficial Toyota Connected Services API. Toyota may change their API at any time, which could break this application.

## Features

### 🚀 Latest Updates (v4.0)

**Multi-User & Apple Watch Support:**
- **HTTP Basic Authentication**: Multi-user access via standard Basic Auth
- **Per-User Token Caching**: Isolated token caching with SHA256-hashed usernames
- **1-Hour Token TTL**: Automatic token expiration after 1 hour of inactivity
- **Privacy-First Design**: No credential storage, hashed cache keys, automatic cleanup
- **Backward Compatible**: Still supports single-user environment variable configuration
- **Apple Watch Ready**: Perfect for iOS/watchOS integration with Keychain credentials

**Phase 1-3 Features:**

**Phase 1 - ABRP Integration:**
- **ABRP Endpoint**: Dedicated `/abrp` endpoint with properly formatted telemetry for A Better Route Planner
- **Location Data**: GPS coordinates (lat/lon) from vehicle's last known position
- **Odometer Data**: Complete mileage tracking for accurate range calculations
- **Smart Charging Detection**: Automatic detection of charging state from vehicle status

**Phase 2 - Multi-Vehicle Support:**
- **Vehicle List**: `/vehicles` endpoint to discover all vehicles in your Toyota account
- **Per-Vehicle Data**: Each endpoint supports the configured VIN
- **Flexible Integration**: Easy to extend for multiple vehicle support

**Phase 3 - Enhanced Features:**
- **CORS Support**: Full CORS headers for web application integration
- **OPTIONS Support**: Proper preflight request handling
- **Additional Endpoints**: `/location`, `/telemetry`, `/vehicles` for granular data access
- **6 Total Endpoints**: Comprehensive API surface for all use cases

**Core Features (v2.0):**
- **Spin SDK 5.1.1**: Latest Fermyon Spin framework with HTTP/2 support
- **New Toyota API**: Migrated to `ctpa-oneapi` - the modern Toyota Connected Services Europe API
- **OAuth2 Authentication**: Secure multi-step authentication flow matching the official MyToyota app
- **Token Caching**: Intelligent token caching with automatic refresh (reduces latency from ~4s to ~200ms)
- **Better Error Handling**: Comprehensive error messages and graceful degradation
- **Proper UUID Generation**: Using standard UUID v4 for correlation IDs
- **Comprehensive Testing**: 16 unit tests covering all critical functionality
- **Zero Warnings**: Clean build with no compiler warnings

### 📊 Technical Details

- **Runtime**: Fermyon Spin 5.x (WebAssembly/WASI)
- **Language**: Rust 2021 Edition
- **Target**: `wasm32-wasip1`
- **Architecture**: Modular workspace with `myt` (API library) and `myt2abrp` (HTTP handler)
- **API Endpoints**: Main endpoint (vehicle telemetry) + Health endpoint (monitoring)
- **Response Format**: JSON with proper Content-Type headers and version info

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

### Available Endpoints

#### 1. Main Endpoint - Vehicle Status
**GET /**

Returns comprehensive vehicle telemetry including battery status, charging state, and range.

```sh
curl http://127.0.0.1:3000/
```

**Response:**
```json
{
  "soc": 85,
  "access_date": "2025-01-01T12:00:00Z",
  "charging_status": "CHARGING",
  "ev_range": 250.5,
  "ev_range_with_ac": 230.0,
  "remaining_charge_time": 120,
  "version": "0.1.0"
}
```

#### 2. ABRP Integration Endpoint
**GET /abrp**

Returns telemetry data formatted for A Better Route Planner integration. Includes location, odometer, and charging status.

```sh
curl http://127.0.0.1:3000/abrp
```

**Response:**
```json
{
  "utc": 1704110400,
  "soc": 85.0,
  "lat": 52.5200,
  "lon": 13.4050,
  "is_charging": true,
  "odometer": 15234.5,
  "est_battery_range": 250.5,
  "version": "0.1.0"
}
```

#### 3. Vehicle List
**GET /vehicles**

Lists all vehicles registered to your Toyota account.

```sh
curl http://127.0.0.1:3000/vehicles
```

#### 4. Vehicle Location
**GET /location**

Returns the last known GPS location of the vehicle.

```sh
curl http://127.0.0.1:3000/location
```

**Response:**
```json
{
  "payload": {
    "vehicleInfo": {
      "location": {
        "lat": 52.5200,
        "lon": 13.4050
      },
      "lastUpdateTimestamp": "2025-01-01T12:00:00Z"
    }
  }
}
```

#### 5. Vehicle Telemetry
**GET /telemetry**

Returns odometer reading and fuel information.

```sh
curl http://127.0.0.1:3000/telemetry
```

**Response:**
```json
{
  "payload": {
    "vehicleInfo": {
      "odometer": {
        "value": 15234.5,
        "unit": "km"
      }
    }
  }
}
```

#### 6. Health Check
**GET /health**

Returns service health status for monitoring.

```sh
curl http://127.0.0.1:3000/health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

### CORS Support

All endpoints support CORS with the following headers:
- `Access-Control-Allow-Origin: *`
- `Access-Control-Allow-Methods: GET, OPTIONS`
- `Access-Control-Allow-Headers: Content-Type`

This enables direct integration from web applications.

## Using with A Better Route Planner (ABRP)

This service provides a dedicated `/abrp` endpoint for seamless integration with ABRP:

1. **Set up your gateway**: Deploy this service and note your public URL
2. **Configure ABRP**: In ABRP settings, configure generic telemetry API
3. **Endpoint**: Point to `https://your-gateway.example.com/abrp`
4. **Polling**: ABRP will automatically poll for updates

The `/abrp` endpoint provides:
- **State of Charge (SoC)**: Battery percentage
- **Location**: GPS coordinates for accurate routing
- **Odometer**: Total mileage for range calculations
- **Charging Status**: Whether vehicle is charging
- **Battery Range**: Estimated range with current charge
- **Timestamp**: UTC timestamp for data freshness

All data is fetched in real-time from Toyota's API and formatted according to ABRP's telemetry specification.

## Apple Watch and Multi-User Support

### Overview

This service supports **multi-user access** via HTTP Basic Authentication, making it ideal for Apple Watch applications and other mobile clients. Each user's Toyota credentials are authenticated separately, and tokens are cached individually with automatic expiration.

### Key Features

- **Basic Authentication**: Standard HTTP Basic Auth for easy integration with Apple Watch and mobile apps
- **Per-User Token Caching**: Each user gets their own cached token with 1-hour TTL
- **Privacy-First Design**:
  - Usernames are hashed with SHA256 for cache keys
  - No credentials are stored permanently
  - Tokens automatically expire after 1 hour of inactivity
- **Backward Compatible**: Still supports environment variable configuration for single-user deployments

### Apple Watch Integration

To integrate with an Apple Watch app:

1. **Store Credentials**: Use the iOS Keychain to securely store the user's Toyota MyT credentials
2. **Make Authenticated Requests**: Include credentials in the HTTP Authorization header
3. **Access Vehicle Data**: Call any endpoint with Basic Auth

#### Example Swift Code

```swift
import Foundation

func fetchVehicleStatus(username: String, password: String) async throws -> VehicleStatus {
    let url = URL(string: "https://your-gateway.example.com/abrp")!
    var request = URLRequest(url: url)

    // Add Basic Auth header
    let credentials = "\(username):\(password)"
    let credentialsData = credentials.data(using: .utf8)!
    let base64Credentials = credentialsData.base64EncodedString()
    request.setValue("Basic \(base64Credentials)", forHTTPHeaderField: "Authorization")

    let (data, response) = try await URLSession.shared.data(for: request)

    guard let httpResponse = response as? HTTPURLResponse,
          httpResponse.statusCode == 200 else {
        throw VehicleError.authenticationFailed
    }

    return try JSONDecoder().decode(VehicleStatus.self, from: data)
}
```

### Authentication Methods

The gateway supports two authentication methods (checked in order):

1. **HTTP Basic Authentication** (Recommended for multi-user)
   ```bash
   curl -u "your.email@example.com:your_password" https://your-gateway.example.com/abrp
   ```

2. **Environment Variables** (For single-user deployments)
   ```bash
   export SPIN_VARIABLE_USERNAME=your.email@example.com
   export SPIN_VARIABLE_PASSWORD=your_password
   export SPIN_VARIABLE_VIN=YOUR_VEHICLE_VIN
   ```

### Security & Privacy Features

#### Token Caching
- **Per-User Isolation**: Each user's token is cached separately using a hashed username as the key
- **SHA256 Hashing**: Usernames are hashed before being used as cache keys, protecting user privacy
- **Automatic Expiration**: Tokens expire 1 hour after last use, minimizing exposure
- **Automatic Cleanup**: Expired tokens are automatically removed from the cache

#### Credential Handling
- **No Persistent Storage**: User credentials are never stored by the gateway
- **Credentials in Transit Only**: Passwords are only used during the OAuth2 flow
- **HTTPS Required**: All Toyota API communication uses HTTPS

#### Cache Key Format
```
toyota_auth_token_<SHA256(username)>
```

Example:
- Username: `user@example.com`
- Cache Key: `toyota_auth_token_e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`

### Multi-User Performance

The per-user token caching provides excellent performance for multi-user scenarios:

- **First Request**: ~4 seconds (full OAuth2 flow)
- **Cached Requests**: ~200ms (using cached token)
- **Token Refresh**: ~1 second (when token expires)
- **TTL Expiration**: Automatic cleanup after 1 hour of inactivity

### Production Deployment Considerations

When deploying for multi-user access:

1. **Use HTTPS**: Always deploy behind HTTPS to protect credentials in transit
2. **Configure CORS**: Restrict `access-control-allow-origin` to your app's domain (currently set to `*`)
3. **Monitor Cache Size**: Each user adds one cache entry; consider cleanup strategies for high-traffic deployments
4. **Set Environment Variables Optional**: Make credentials optional in `spin.toml` (already configured)
5. **Rate Limiting**: Consider adding rate limiting to prevent abuse

### Troubleshooting Multi-User Issues

**401 Authentication Required**
- Verify Basic Auth header is properly formatted: `Authorization: Basic <base64(username:password)>`
- Check credentials are correct for the MyToyota app

**Token Expired Messages**
- Normal behavior after 1 hour of inactivity
- Next request will automatically refresh the token

**VIN Required Error**
- VIN must still be configured via environment variable (future enhancement: per-request VIN parameter)
- Use `/vehicles` endpoint to discover available VINs

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

### Gateway Endpoints (This Service)

- **Main Status**: `GET /` - Complete vehicle telemetry
- **ABRP Integration**: `GET /abrp` - ABRP-formatted telemetry with location and odometer
- **Vehicle List**: `GET /vehicles` - List all registered vehicles
- **Location**: `GET /location` - GPS coordinates
- **Telemetry**: `GET /telemetry` - Odometer and fuel data
- **Health**: `GET /health` - Service health check

### Toyota Connected Services Europe (Upstream)

- **Authentication**: `https://b2c-login.toyota-europe.com/json/realms/root/realms/tme/authenticate`
- **Authorization**: `https://b2c-login.toyota-europe.com/oauth2/realms/root/realms/tme/authorize`
- **Token Exchange**: `https://b2c-login.toyota-europe.com/oauth2/realms/root/realms/tme/access_token`
- **Electric Status**: `https://ctpa-oneapi.tceu-ctp-prd.toyotaconnectedeurope.io/v1/global/remote/electric/status`
- **Location**: `https://ctpa-oneapi.tceu-ctp-prd.toyotaconnectedeurope.io/v1/global/remote/location`
- **Telemetry**: `https://ctpa-oneapi.tceu-ctp-prd.toyotaconnectedeurope.io/v1/global/remote/telemetry`
- **Vehicles**: `https://ctpa-oneapi.tceu-ctp-prd.toyotaconnectedeurope.io/v1/vehicles`

## Testing Coverage

- ✅ JWT decoding with valid tokens
- ✅ JWT decoding with invalid formats
- ✅ Token caching and expiry logic
- ✅ OAuth2 request structure validation
- ✅ Electric vehicle status parsing
- ✅ Timestamp generation
- ✅ Error handling paths
- ✅ Enhanced response structure with all telemetry fields
- ✅ Optional field handling in responses
- ✅ Username hashing (SHA256)
- ✅ Per-user token cache key generation
- ✅ Per-user token TTL expiration
- ✅ Token access time updates

**Total: 20 tests** (9 in `myt` library, 11 in `myt2abrp` handler)

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
