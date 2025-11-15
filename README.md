# Toyota MyT to ABRP Gateway

WebAssembly-based gateway service that bridges Toyota Connected Services Europe (MyToyota) with A Better Route Planner (ABRP) for electric vehicle telemetry data.

## Overview

This project provides a component-based architecture built with WebAssembly Component Model, enabling:

- **Toyota API Integration**: Authenticate and fetch vehicle data from Toyota Connected Services
- **ABRP Telemetry**: Transform Toyota data into ABRP-compatible telemetry format
- **Component Architecture**: 8 independent WASM components (7 pure WASI, 1 Spin gateway)
- **Production Ready**: JWT authentication, circuit breaker, metrics, retry logic, validation

## Architecture

```
┌─────────────────────────────────────────────┐
│  Gateway (Spin HTTP Component)              │
│  - Authentication & session management      │
│  - Toyota API integration                   │
│  - ABRP telemetry endpoint                  │
│  - KV store for caching                     │
└────────────┬────────────────────────────────┘
             │ imports (via WAC)
             ↓
┌─────────────────────────────────────────────┐
│  Pure WASI Components (7 components)        │
├─────────────────────────────────────────────┤
│  business-logic   │ JWT operations          │
│  circuit-breaker  │ Resilience pattern      │
│  data-transform   │ Toyota → ABRP           │
│  metrics          │ Prometheus metrics      │
│  retry-logic      │ Exponential backoff     │
│  toyota-api-types │ API data models         │
│  validation       │ Input validation        │
└─────────────────────────────────────────────┘
```

## Components

See [components/README.md](components/README.md) for detailed component documentation.

**Statistics**:
- 8 total components
- 1,892 lines of code
- 54/54 tests passing
- 1.2 MB total size

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [cargo-component](https://github.com/bytecodealliance/cargo-component)
- [Spin CLI](https://developer.fermyon.com/spin/install)

```bash
# Install cargo-component
cargo install cargo-component

# Install Spin
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
```

### Building

```bash
# Build all components
cargo component build --release

# Or build with Bazel (hermetic builds)
bazel build //components/...
```

### Running

```bash
# Start the gateway
cd components/gateway
spin up

# The gateway will be available at http://localhost:3000
```

## API Endpoints

### Authentication

```bash
# Login
POST /api/login
Content-Type: application/json
{
  "username": "user@example.com",
  "password": "your-password"
}

# Response
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Telemetry

```bash
# Get ABRP telemetry
GET /abrp?token=<your_token>&vin=<vehicle_vin>
Authorization: Bearer <access_token>

# Response
{
  "utc": 1736942400,
  "soc": 85.0,
  "lat": 52.520008,
  "lon": 13.404954,
  "is_charging": true,
  "odometer": 15000.0,
  "est_battery_range": 250.5,
  "version": "1.0.0"
}
```

### Health Check

```bash
# Check gateway health
GET /health

# Response
{
  "status": "healthy",
  "message": "Gateway component builds successfully"
}
```

## Development

### Project Structure

```
.
├── components/           # WebAssembly components
│   ├── business-logic/   # JWT operations
│   ├── circuit-breaker/  # Resilience pattern
│   ├── data-transform/   # Toyota → ABRP transformation
│   ├── gateway/          # Spin HTTP gateway
│   ├── metrics/          # Prometheus metrics
│   ├── retry-logic/      # Retry strategy
│   ├── toyota-api-types/ # Toyota API data models
│   └── validation/       # Input validation
├── myt2abrp/             # Main gateway application
├── myt/                  # Toyota API client library
├── bazel-build.md        # Bazel build instructions
└── README.md             # This file
```

### Testing

```bash
# Run all tests (native target - faster)
cargo test --workspace --target x86_64-unknown-linux-gnu

# Run specific component tests
cargo test --package toyota-validation --target x86_64-unknown-linux-gnu

# Run with Bazel
bazel test //components/...
```

### Building with Bazel

See [bazel-build.md](bazel-build.md) for detailed Bazel instructions.

```bash
# Install Bazel (via Bazelisk)
npm install -g @bazel/bazelisk

# Build all components
bazel build //components/...

# Run tests
bazel test //components/...
```

## Configuration

Gateway configuration via Spin variables:

```toml
# spin.toml
[component.gateway.variables]
jwt_secret = { required = true }
cors_origin = { default = "*" }
```

Set variables:

```bash
# Set JWT secret
spin variables set jwt_secret "your-secret-key"
```

## Deployment

### Deploy to Fermyon Cloud

```bash
# Login to Fermyon Cloud
spin login

# Deploy
spin deploy
```

### Deploy to Kubernetes (with SpinKube)

```bash
# Build and push
spin build
spin registry push ghcr.io/your-org/toyota-gateway:latest

# Deploy with SpinKube
kubectl apply -f k8s/deployment.yaml
```

## Monitoring

Prometheus metrics available at `/metrics`:

- `total_requests` - Total HTTP requests
- `total_errors` - Total errors
- `cache_hits` / `cache_misses` - Cache statistics
- `jwt_generations` / `jwt_verifications` - JWT operations
- `circuit_breaker_opens` / `circuit_breaker_closes` - Circuit breaker state changes
- `retry_attempts` / `retry_success` / `retry_exhausted` - Retry statistics

## License

Apache 2.0 - See [LICENSE](LICENSE) file for details.

## Contributing

This is an internal project. For questions or issues, please contact the development team.

## Technical Details

### WebAssembly Component Model

All components use the [WebAssembly Component Model](https://github.com/WebAssembly/component-model) with WIT (WebAssembly Interface Types) for strong typing and composition.

### Spin Framework

The gateway runs on [Spin](https://www.fermyon.com/spin), a serverless WebAssembly application framework.

### Zero Dependencies

7 out of 8 components have **zero dependencies** on Spin SDK, making them:
- Reusable across any WASM runtime
- Testable on native targets (faster development)
- Portable to other platforms

### Build Systems

- **Cargo Component**: Primary development tool
- **Bazel**: Hermetic, reproducible CI/CD builds
- Both build the same artifacts with identical WIT interfaces
