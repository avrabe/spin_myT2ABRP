# Toyota MyT (Europe) to A Better Route Planner Gateway using Fermyon Spin

[![Rust](https://github.com/avrabe/spin_myT2ABRP/actions/workflows/rust.yml/badge.svg)](https://github.com/avrabe/spin_myT2ABRP/actions/workflows/rust.yml)

A WebAssembly-based gateway service that bridges Toyota Connected Services Europe (MyToyota) with A Better Route Planner (ABRP) for electric vehicle telemetry data.

## Overview

This application provides real-time vehicle battery status (State of Charge) from Toyota electric/hybrid vehicles in Europe to ABRP for accurate route planning. Built with Rust and Fermyon Spin, it runs as a serverless WebAssembly component with minimal resource usage and maximum performance.

**⚠️ Important**: This uses the unofficial Toyota Connected Services API. Toyota may change their API at any time, which could break this application.

## Features

### 🚀 Latest Updates (v5.0) - COMPLETE SECURITY OVERHAUL

**⚠️ BREAKING CHANGE: JWT Bearer Token Authentication**

This release implements a **complete architectural redesign** from Basic Authentication to JWT Bearer tokens, following OAuth2 industry best practices for 2024/2025:

- **🔐 JWT Bearer Tokens**: Industry-standard authentication with HS256 signing
  - **Access Tokens**: Short-lived (15 minutes) for API access
  - **Refresh Tokens**: Long-lived (7 days) for token renewal
  - **Token Revocation**: Logout invalidates tokens immediately

- **🛡️ Enhanced Security Architecture**:
  - **Credentials Sent Once**: Login only - not on every request (Basic Auth sent on EVERY request)
  - **Session Management**: Track active sessions with timestamps, IP, user agent
  - **Rate Limiting**: 100 req/user/hour, login lockout after 5 failed attempts (15 min)
  - **Automatic Token Expiration**: Access tokens expire after 15 minutes

- **🔑 Authentication Endpoints**:
  - `POST /auth/login` - Authenticate and receive JWT tokens
  - `POST /auth/refresh` - Renew access token using refresh token
  - `POST /auth/logout` - Revoke tokens and end session

- **📊 Multi-User Support**:
  - **Per-User Token Caching**: Each user gets isolated Toyota OAuth token cache
  - **Privacy-First**: Usernames hashed with HMAC-SHA256 for cache keys
  - **Session Tracking**: Full audit trail of user sessions

- **🍎 Apple Watch Integration**: Updated flow with token-based auth (see below)

**Previous Updates (v4.0):**
- Basic Authentication (now removed)
- Multi-user support via Basic Auth
- Per-user token caching

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

## Authentication

### JWT Bearer Token Flow

This service uses **JWT Bearer tokens** for authentication, following OAuth2 best practices:

#### 1. Login (POST /auth/login)

Authenticate with your Toyota MyT credentials to receive JWT tokens:

```bash
curl -X POST http://127.0.0.1:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "your.email@example.com",
    "password": "your_password"
  }'
```

**Response:**
```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "eyJhbGc...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**Security Features:**
- Toyota credentials validated against official API
- JWT tokens generated with HS256 signing
- Session created with tracking info
- Failed login attempts tracked (5 attempts = 15 min lockout)
- Rate limited: 100 requests/hour per user

#### 2. Access Protected Endpoints

Use the access token in the `Authorization` header:

```bash
curl http://127.0.0.1:3000/abrp \
  -H "Authorization: Bearer eyJhbGc..."
```

**Access Token Details:**
- **Lifetime**: 15 minutes
- **Purpose**: Access all data endpoints
- **Format**: JWT with HS256 signature
- **Claims**: sub (username), exp (expiration), jti (token ID)

#### 3. Refresh Token (POST /auth/refresh)

When your access token expires, use the refresh token to get a new one:

```bash
curl -X POST http://127.0.0.1:3000/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "eyJhbGc..."
  }'
```

**Response:**
```json
{
  "access_token": "eyJhbGc...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**Refresh Token Details:**
- **Lifetime**: 7 days
- **Purpose**: Obtain new access tokens without re-login
- **Security**: Refresh tokens are also checked against revocation list

#### 4. Logout (POST /auth/logout)

Revoke your tokens when done:

```bash
curl -X POST http://127.0.0.1:3000/auth/logout \
  -H "Authorization: Bearer eyJhbGc..."
```

**Response:**
```json
{
  "message": "Successfully logged out"
}
```

**What Happens:**
- Access token immediately revoked (added to revocation list)
- Future requests with this token will fail with 401
- Refresh token for this session also invalidated

### Why JWT Instead of Basic Auth?

**Security Improvements:**
1. **Credentials Sent Once**: With Basic Auth, credentials were sent on EVERY request. With JWT, credentials are only sent during login.
2. **Token Revocation**: Can immediately invalidate compromised tokens without changing password
3. **Session Management**: Track all active sessions, see last access time, IP, user agent
4. **Rate Limiting**: Prevent brute force attacks with login attempt limits
5. **Audit Trail**: Know exactly when and from where users authenticated

**Performance:**
- Same as before: Toyota OAuth tokens are still cached (1 hour TTL)
- JWT verification is fast (cryptographic signature check)
- No database lookup required for every request

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

#### Production Deployment (Required)

**⚠️ CRITICAL: Set JWT Secret**

The JWT secret is used to sign authentication tokens. You **MUST** set a random secret in production:

```bash
# Generate a random 256-bit secret
openssl rand -hex 32

# Set as environment variable
export SPIN_VARIABLE_JWT_SECRET=<your-random-secret-here>
```

**⚠️ CRITICAL: Set HMAC Key**

The HMAC key is used to hash usernames for privacy. You **MUST** set a random key in production:

```bash
# Generate a random 256-bit key
openssl rand -hex 32

# Set as environment variable
export SPIN_VARIABLE_HMAC_KEY=<your-random-key-here>
```

#### Optional: Single-Vehicle VIN

If you only have one vehicle, set the VIN (otherwise use `/vehicles` endpoint):

```bash
export SPIN_VARIABLE_VIN=YOUR_VEHICLE_VIN
```

**Security Configuration in spin.toml:**
```toml
[variables]
jwt_secret = { required = false, secret = true }
hmac_key = { required = false, secret = true }
vin = { required = false }
```

**Default Values (INSECURE - for development only):**
- If `jwt_secret` not set: Uses hardcoded default (defined in code)
- If `hmac_key` not set: Uses hardcoded default (defined in code)
- **WARNING**: Never use default secrets in production!

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

This service supports **multi-user access** via **JWT Bearer tokens**, making it ideal for Apple Watch applications and other mobile clients. Each user authenticates once to receive tokens, which are then used for all subsequent requests.

### Key Features

- **JWT Bearer Tokens**: Industry-standard OAuth2-style authentication
- **Token Storage**: Store JWT tokens in iOS Keychain (more secure than storing passwords)
- **Per-User Token Caching**: Each user gets their own cached Toyota OAuth token (1-hour TTL)
- **Privacy-First Design**:
  - Usernames are hashed with HMAC-SHA256 for cache keys
  - No credentials are stored by the gateway
  - Tokens automatically expire (access: 15 min, refresh: 7 days)
- **Session Management**: Track all active sessions with timestamps and metadata

### Apple Watch Integration

To integrate with an Apple Watch app:

1. **Login Flow**:
   - Prompt user for Toyota MyT credentials (one-time)
   - Send credentials to `/auth/login`
   - Receive access token and refresh token
   - Store tokens in iOS Keychain

2. **Access Vehicle Data**:
   - Include access token in Authorization header
   - When access token expires, use refresh token to get new one

3. **Token Refresh**:
   - Access tokens expire after 15 minutes
   - Refresh tokens valid for 7 days
   - Automatically refresh when needed

#### Example Swift Code

```swift
import Foundation

// MARK: - Data Models

struct LoginRequest: Codable {
    let username: String
    let password: String
}

struct LoginResponse: Codable {
    let access_token: String
    let refresh_token: String
    let token_type: String
    let expires_in: Int
}

struct RefreshRequest: Codable {
    let refresh_token: String
}

struct RefreshResponse: Codable {
    let access_token: String
    let token_type: String
    let expires_in: Int
}

// MARK: - Authentication Service

class ToyotaGatewayAuth {
    static let shared = ToyotaGatewayAuth()
    private let baseURL = "https://your-gateway.example.com"

    // Login and store tokens
    func login(username: String, password: String) async throws -> LoginResponse {
        let url = URL(string: "\(baseURL)/auth/login")!
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let loginReq = LoginRequest(username: username, password: password)
        request.httpBody = try JSONEncoder().encode(loginReq)

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw AuthError.loginFailed
        }

        let loginResponse = try JSONDecoder().decode(LoginResponse.self, from: data)

        // Store tokens in Keychain
        try KeychainManager.save(token: loginResponse.access_token, key: "access_token")
        try KeychainManager.save(token: loginResponse.refresh_token, key: "refresh_token")

        return loginResponse
    }

    // Refresh access token
    func refreshToken() async throws -> String {
        guard let refreshToken = try KeychainManager.load(key: "refresh_token") else {
            throw AuthError.noRefreshToken
        }

        let url = URL(string: "\(baseURL)/auth/refresh")!
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let refreshReq = RefreshRequest(refresh_token: refreshToken)
        request.httpBody = try JSONEncoder().encode(refreshReq)

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse,
              httpResponse.statusCode == 200 else {
            throw AuthError.refreshFailed
        }

        let refreshResponse = try JSONDecoder().decode(RefreshResponse.self, from: data)

        // Store new access token
        try KeychainManager.save(token: refreshResponse.access_token, key: "access_token")

        return refreshResponse.access_token
    }

    // Fetch vehicle data with auto-refresh
    func fetchVehicleStatus() async throws -> VehicleStatus {
        var accessToken = try KeychainManager.load(key: "access_token")

        // Try with current token
        do {
            return try await makeRequest(token: accessToken)
        } catch AuthError.unauthorized {
            // Token expired, refresh and retry
            accessToken = try await refreshToken()
            return try await makeRequest(token: accessToken)
        }
    }

    private func makeRequest(token: String?) async throws -> VehicleStatus {
        guard let token = token else {
            throw AuthError.noAccessToken
        }

        let url = URL(string: "\(baseURL)/abrp")!
        var request = URLRequest(url: url)
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse else {
            throw AuthError.invalidResponse
        }

        if httpResponse.statusCode == 401 {
            throw AuthError.unauthorized
        }

        guard httpResponse.statusCode == 200 else {
            throw AuthError.requestFailed
        }

        return try JSONDecoder().decode(VehicleStatus.self, from: data)
    }

    // Logout
    func logout() async throws {
        guard let accessToken = try KeychainManager.load(key: "access_token") else {
            return
        }

        let url = URL(string: "\(baseURL)/auth/logout")!
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("Bearer \(accessToken)", forHTTPHeaderField: "Authorization")

        _ = try await URLSession.shared.data(for: request)

        // Clear keychain
        try KeychainManager.delete(key: "access_token")
        try KeychainManager.delete(key: "refresh_token")
    }
}

enum AuthError: Error {
    case loginFailed
    case refreshFailed
    case unauthorized
    case noAccessToken
    case noRefreshToken
    case invalidResponse
    case requestFailed
}
```

### Authentication Flow (Updated for JWT)

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
2. **⚠️ CRITICAL: Configure CORS**: Restrict `access-control-allow-origin` to your app's domain (currently set to `*` - **MUST CHANGE**)
3. **⚠️ CRITICAL: Set HMAC Key**: Configure `SPIN_VARIABLE_HMAC_KEY` with a random secret (see Security Configuration below)
4. **Monitor Cache Size**: Each user adds one cache entry; consider cleanup strategies for high-traffic deployments
5. **Set Environment Variables Optional**: Make credentials optional in `spin.toml` (already configured)
6. **Rate Limiting**: Consider adding rate limiting to prevent abuse

### Security Configuration

#### ⚠️ CRITICAL: HMAC Key

The service uses HMAC-SHA256 to hash usernames before using them as cache keys. This prevents rainbow table attacks if the key-value store is compromised.

**You MUST set a random HMAC key in production:**

```bash
# Generate a random key (32+ bytes recommended)
openssl rand -hex 32

# Set as environment variable
export SPIN_VARIABLE_HMAC_KEY=<your-random-key-here>
```

**Default value**: If not set, uses a hardcoded default key (defined in code). This is **INSECURE** for production use.

**Add to spin.toml:**
```toml
[variables]
hmac_key = { required = false, secret = true }
```

#### ⚠️ CRITICAL: CORS Configuration

Currently set to `access-control-allow-origin: *` which allows **ANY** website to make requests.

**For production**, edit `myt2abrp/src/lib.rs` and change:
```rust
builder.header("access-control-allow-origin", "*");
// To:
builder.header("access-control-allow-origin", "https://your-app-domain.com");
```

Or make it configurable via environment variable.

#### Input Validation

The service validates:
- Username must be a valid email address
- Username maximum length: 256 characters
- Password maximum length: 256 characters
- Base64 auth header maximum size: 1024 bytes

These limits prevent DoS attacks via extremely large inputs.

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
