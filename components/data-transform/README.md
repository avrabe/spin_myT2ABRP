# data-transform

Pure business logic for transforming Toyota Connected Services vehicle data to A Better Route Planner (ABRP) telemetry format with **zero dependencies** on Spin SDK.

## Overview

This component provides stateless transformation functions that convert Toyota API responses (electric status, location, telemetry) into the ABRP telemetry format. All functions are pure - no async, no HTTP, no platform dependencies.

## Features

- **Zero Spin SDK dependencies** - Pure WASI component
- **Pure transformation logic** - No side effects, easily testable
- **JSON-based interface** - Works with serialized data from toyota-api-types
- **Timestamp parsing** - ISO 8601 to Unix timestamp conversion
- **Charging detection** - Smart status string parsing
- **Well-tested** - 8 unit tests, 100% passing
- **Optimized** - 263KB release binary

## WIT Interface

```wit
interface converter {
    /// ABRP telemetry record
    record abrp-telemetry {
        utc: s64,
        soc: f64,
        lat: option<f64>,
        lon: option<f64>,
        is-charging: option<bool>,
        is-parked: option<bool>,
        odometer: option<f64>,
        est-battery-range: option<f64>,
        version: string,
    }

    /// Parse ISO 8601 timestamp to Unix seconds
    parse-iso8601-timestamp: func(iso-string: string) -> s64;

    /// Determine if vehicle is charging from status string
    is-charging-status: func(status: string) -> bool;

    /// Convert Toyota electric status to ABRP telemetry
    toyota-to-abrp: func(
        electric-status-json: string,
        location-json: option<string>,
        telemetry-json: option<string>,
        version: string
    ) -> result<abrp-telemetry, string>;

    /// Serialize/deserialize ABRP telemetry
    serialize-abrp-telemetry: func(telemetry: abrp-telemetry) -> string;
    deserialize-abrp-telemetry: func(json: string) -> result<abrp-telemetry, string>;
}
```

## Transformation Logic

### Input Data (Toyota API Format)

The component accepts JSON strings from three Toyota API endpoints:

1. **Electric Status** (required):
   - Battery state of charge (SOC)
   - Charging status string
   - EV range estimates
   - Timestamp

2. **Location** (optional):
   - GPS coordinates (lat/lon)
   - Timestamp

3. **Telemetry** (optional):
   - Odometer reading
   - Fuel level (for hybrids)

### Output Data (ABRP Format)

Returns standardized telemetry record with:
- **utc**: Unix timestamp (seconds since epoch)
- **soc**: State of charge (0-100)
- **lat/lon**: GPS coordinates (optional)
- **is_charging**: Boolean derived from status string
- **odometer**: Kilometers traveled (optional)
- **est_battery_range**: Estimated range in km (optional)
- **version**: API version string

### Charging Status Detection

Smart parsing of Toyota's charging status strings:

| Status String | is_charging |
|--------------|-------------|
| "CHARGING" | true |
| "CONNECTED" | true |
| "NOT_CHARGING" | false |
| "DISCONNECTED" | false |
| "NOT_CONNECTED" | false |

Detection logic:
1. Check for negative indicators ("NOT", "DISCONNECT") → false
2. Check for positive indicators ("CHARGING", "CONNECTED") → true
3. Case-insensitive matching

## Usage Example

```rust
use toyota_data_transform::*;

// Parse timestamp
let timestamp = parse_iso8601_timestamp("2025-01-15T12:00:00Z");
// → 1736942400

// Check charging status
let is_charging = is_charging_status("CHARGING");
// → true

// Transform Toyota data to ABRP
let electric_status_json = r#"{
    "payload": {
        "vehicleInfo": {
            "chargeInfo": {
                "chargeRemainingAmount": 85,
                "chargingStatus": "CHARGING",
                "evRange": 250.5
            },
            "lastUpdateTimestamp": "2025-01-15T12:00:00Z"
        }
    }
}"#;

let location_json = r#"{
    "payload": {
        "vehicleInfo": {
            "location": {
                "lat": 52.520008,
                "lon": 13.404954
            }
        }
    }
}"#;

let abrp = toyota_to_abrp(
    electric_status_json,
    Some(location_json),
    None,
    "1.0.0"
)?;

// abrp.soc == 85.0
// abrp.lat == Some(52.520008)
// abrp.is_charging == Some(true)
```

## Building

```bash
# Build component
cargo component build --release

# Run tests (native target)
cargo test --package toyota-data-transform --target x86_64-unknown-linux-gnu

# Size
ls -lh target/wasm32-wasip1/release/toyota_data_transform.wasm
# 263K
```

## Dependencies

- `chrono` - ISO 8601 timestamp parsing
- `serde` / `serde_json` - JSON serialization
- `wit-bindgen-rt` - WIT bindings

## Test Coverage

All 8 tests passing:
- ✅ `test_parse_iso8601_timestamp` - Timestamp parsing with valid/invalid inputs
- ✅ `test_is_charging_status` - Charging status detection edge cases
- ✅ `test_toyota_to_abrp_basic` - Basic transformation with electric status only
- ✅ `test_toyota_to_abrp_with_location` - Transformation with GPS coordinates
- ✅ `test_toyota_to_abrp_with_telemetry` - Transformation with odometer data
- ✅ `test_toyota_to_abrp_complete` - Full transformation with all inputs
- ✅ `test_abrp_telemetry_serialization` - JSON serialization/deserialization
- ✅ `test_toyota_to_abrp_invalid_json` - Error handling for malformed input

## Architecture

Part of the component-based Toyota integration system:

```
┌────────────────────────────────────┐
│  Gateway (imports via WAC)         │
│    toyota:data-transform/converter │
└───────────────┬────────────────────┘
                │
                ↓
┌────────────────────────────────────┐
│  data-transform (263KB)            │
│  - Toyota → ABRP conversion        │
│  - Timestamp parsing               │
│  - Charging status detection       │
│  - Zero Spin dependencies          │
└────────────────────────────────────┘
```

Can be composed with `toyota-api-types` component for full data pipeline:

```
toyota-api-types → data-transform → ABRP telemetry
(deserialize JSON)   (transform)     (serialize JSON)
```

## Design Decisions

### Why JSON-based interface?

Using JSON strings instead of direct struct imports provides:
- **Loose coupling** - No dependency on toyota-api-types at compile time
- **Flexibility** - Works with any source that produces Toyota API JSON
- **Simplicity** - Clear input/output boundaries
- **Testability** - Easy to test with string literals

### Why pure functions?

No async, no HTTP, no state:
- **Testable** - Deterministic, no mocking needed
- **Composable** - Easy to chain with other components
- **Portable** - Runs anywhere (WASI, native, browser)
- **Fast** - No I/O overhead, pure CPU

### Why internal data structures?

Duplicating Toyota API types internally (instead of importing):
- **Independence** - Component doesn't depend on toyota-api-types
- **Minimal** - Only includes fields actually used in transformation
- **Stable** - Internal API won't break if toyota-api-types changes

## License

Apache 2.0
