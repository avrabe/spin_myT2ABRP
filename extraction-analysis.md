# Additional Component Extraction Analysis

**Current State**: 2,548 lines in `myt2abrp/src/lib.rs` + 603 lines in `myt/src/lib.rs` = **3,151 lines remaining**

---

## ✅ Already Extracted (842 lines)

| Component | Lines | Spin Deps | Status |
|-----------|-------|-----------|--------|
| business-logic | 400 | ❌ Zero | ✅ Extracted |
| circuit-breaker | 184 | ❌ Zero | ✅ Extracted |
| metrics | 228 | ❌ Zero | ✅ Extracted |
| gateway | 30 | ✅ HTTP only | ✅ Minimal |

---

## 🎯 Can Be Extracted (Pure WASI)

### 1. **toyota-api-types** (~600 lines) - HIGH PRIORITY

**Location**: `myt/src/lib.rs`

**Content**:
- Toyota API data structures (AuthenticateRequest, TokenRequest, TokenResponse, etc.)
- Vehicle data models (VehicleListResponse, ElectricStatusResponse, LocationResponse)
- ABRP telemetry models
- Pure Serde types

**Dependencies**:
- ❌ Zero Spin SDK (has one `IntoBody` trait that can be made optional)
- Just `serde` + `serde_json`

**Effort**: 1-2 hours
**Value**: HIGH - Reusable types across projects

**WIT Interface**:
```wit
package toyota:api-types@0.1.0;

interface models {
    record token-response {
        access-token: string,
        refresh-token: string,
        id-token: string,
        expires-in: u32,
    }

    record vehicle-status {
        battery-soc: f32,
        range-km: f32,
        charging: bool,
        // ... more fields
    }

    // Serialization/deserialization helpers
    serialize-token-response: func(resp: token-response) -> string;
    deserialize-token-response: func(json: string) -> result<token-response, string>;
}
```

---

### 2. **data-transform** (~200 lines) - HIGH PRIORITY

**Location**: Scattered in `myt2abrp/src/lib.rs`

**Content**:
- Toyota → ABRP data conversion
- Unit conversions (km → miles, etc.)
- Data normalization
- Timestamp formatting

**Current Functions**:
```rust
// Example from codebase:
fn toyota_to_abrp(electric_status: ElectricStatusResponse, location: LocationResponse)
    -> AbrpTelemetry {
    AbrpTelemetry {
        utc: format_timestamp(location.timestamp),
        soc: electric_status.battery_soc as f32,
        latitude: location.latitude,
        longitude: location.longitude,
        // ... more conversions
    }
}
```

**Dependencies**: ❌ Zero Spin SDK

**Effort**: 2-3 hours
**Value**: HIGH - Pure business logic, easily testable

**WIT Interface**:
```wit
package toyota:data-transform@0.1.0;

interface converter {
    record electric-status {
        battery-soc: f32,
        range-km: f32,
        charging: bool,
    }

    record location {
        latitude: f64,
        longitude: f64,
        timestamp: s64,
    }

    record abrp-telemetry {
        utc: s64,
        soc: f32,
        latitude: f64,
        longitude: f64,
        is-charging: s32,
        // ... more fields
    }

    transform-to-abrp: func(
        status: electric-status,
        location: location
    ) -> abrp-telemetry;
}
```

---

### 3. **validation** (~150 lines) - MEDIUM PRIORITY

**Location**: `myt2abrp/src/lib.rs`

**Content**:
- Credential validation
- Email format validation
- Input length checks
- Password strength checks (if needed)

**Current Functions**:
```rust
fn validate_credentials(username: &str, password: &str) -> Result<()> {
    if username.is_empty() || password.is_empty() {
        return Err("Empty credentials");
    }
    if username.len() > 256 {
        return Err("Username too long");
    }
    // Email validation
    Ok(())
}
```

**Dependencies**: ❌ Zero Spin SDK

**Effort**: 1-2 hours
**Value**: MEDIUM - Reusable, but simple logic

**WIT Interface**:
```wit
package toyota:validation@0.1.0;

interface validator {
    validate-email: func(email: string) -> result<_, string>;
    validate-credentials: func(username: string, password: string) -> result<_, string>;
    validate-vin: func(vin: string) -> result<_, string>;
}
```

---

### 4. **retry-logic** (~100 lines) - MEDIUM PRIORITY

**Location**: `myt2abrp/src/lib.rs`

**Content**:
- Exponential backoff algorithm
- Retry decision logic
- Timeout calculation

**Current Functions**:
```rust
async fn send_request_with_retry<F>(request_builder: F) -> Result<Response>
where F: Fn() -> Request {
    let mut attempts = 0;
    loop {
        match send_request_once(request_builder()).await {
            Ok(response) => return Ok(response),
            Err(e) if attempts < MAX_RETRIES => {
                attempts += 1;
                sleep(backoff_duration(attempts)).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

**Problem**: Uses async HTTP calls which need Spin SDK
**Solution**: Extract just the **retry logic** (backoff calculation, decision making)

**Dependencies**: ❌ Zero Spin SDK (if we extract just the logic)

**Effort**: 2 hours
**Value**: MEDIUM - Generic retry logic

**WIT Interface**:
```wit
package toyota:retry@0.1.0;

interface strategy {
    record retry-config {
        max-attempts: u32,
        initial-delay-ms: u64,
        max-delay-ms: u64,
        multiplier: f64,
    }

    // Calculate backoff duration for attempt N
    calculate-backoff: func(attempt: u32, config: retry-config) -> u64;

    // Should we retry this error?
    should-retry: func(error-code: u32, attempt: u32, max-attempts: u32) -> bool;
}
```

---

## ❌ Cannot Be Extracted (Requires Spin SDK)

### 1. **KV Store Operations** (~400 lines) - **MUST STAY**

**Reason**: Uses `spin_sdk::key_value::Store` directly

**Content**:
- Token caching (`get_cached_vehicle_data`, `set_cached_vehicle_data`)
- Session management (`create_session`, `get_session`, `delete_session`)
- Rate limiting (`check_rate_limit`, `record_failed_login`)
- Token revocation (`is_token_revoked`, `revoke_token`)

**Cannot extract because**:
- KV Store is a Spin-specific resource
- No WASI standard for KV stores yet
- Gateway needs to orchestrate caching

**Stay in**: Gateway component

---

### 2. **HTTP Client** (~300 lines) - **MUST STAY**

**Reason**: Uses `spin_sdk::http::Request` and `spin_sdk::http::send()`

**Content**:
- `send_request()` - HTTP client wrapper
- `send_request_with_retry()` - Retry wrapper
- `send_request_once()` - Single request

**Cannot extract because**:
- Uses Spin's HTTP client
- Integrates with circuit breaker (which needs metrics/state)
- Could potentially use `wasi:http/outgoing-handler` but complex

**Stay in**: Gateway component

---

### 3. **Endpoint Handlers** (~800 lines) - **MUST STAY**

**Reason**: Uses `spin_sdk::http::IncomingRequest`

**Content**:
- `handle_login()` - Login endpoint
- `handle_refresh()` - Token refresh
- `handle_logout()` - Logout
- `handle_abrp()` - ABRP telemetry endpoint
- `handle_health()` - Health check
- `handle_metrics()` - Metrics endpoint
- Route dispatcher

**Cannot extract because**:
- Tightly coupled to Spin HTTP trigger
- Orchestrates all other components
- This IS the gateway

**Stay in**: Gateway component

---

### 4. **Configuration** (~100 lines) - **MUST STAY**

**Reason**: Uses `spin_sdk::variables`

**Content**:
- `get_jwt_secret()` - Read from env vars
- `get_hmac_key()` - Read from env vars
- `get_cors_origin()` - Read from env vars
- `validate_production_config()` - Startup validation

**Cannot extract because**:
- Uses Spin's variable system
- Environment-specific configuration

**Stay in**: Gateway component

---

## 📊 Extraction Summary

### Extractable Components (4 new)

| Component | Lines | Spin Deps | Effort | Value | Priority |
|-----------|-------|-----------|--------|-------|----------|
| **toyota-api-types** | ~600 | ❌ Zero | 1-2h | HIGH | 🔥 #1 |
| **data-transform** | ~200 | ❌ Zero | 2-3h | HIGH | 🔥 #2 |
| **validation** | ~150 | ❌ Zero | 1-2h | MEDIUM | #3 |
| **retry-logic** | ~100 | ❌ Zero | 2h | MEDIUM | #4 |
| **Total Extractable** | **~1,050** | - | **6-9h** | - | - |

### Must Remain in Gateway

| Category | Lines | Reason |
|----------|-------|--------|
| KV Store Operations | ~400 | Uses `spin_sdk::key_value::Store` |
| HTTP Client | ~300 | Uses `spin_sdk::http::Request` |
| Endpoint Handlers | ~800 | Uses `spin_sdk::http::IncomingRequest` |
| Configuration | ~100 | Uses `spin_sdk::variables` |
| Routing/Dispatch | ~200 | Spin HTTP component macro |
| **Total Gateway** | **~1,800** | Core orchestration |

### Already Extracted

| Component | Lines | Status |
|-----------|-------|--------|
| business-logic | 400 | ✅ Done |
| circuit-breaker | 184 | ✅ Done |
| metrics | 228 | ✅ Done |
| gateway-shell | 30 | ✅ Done |
| **Total Extracted** | **842** | ✅ Complete |

---

## 🎯 Recommended Extraction Order

### Phase 2 (Next Steps)

1. **toyota-api-types** (1-2 hours) - ⭐ HIGHEST VALUE
   - Pure data models
   - Immediately reusable
   - Zero dependencies
   - Easy to test

2. **data-transform** (2-3 hours) - ⭐ HIGH VALUE
   - Pure business logic
   - Toyota → ABRP conversion
   - Easy to test with fixtures
   - Clear boundaries

3. **validation** (1-2 hours)
   - Simple input validation
   - Reusable across projects
   - Easy to test

4. **retry-logic** (2 hours)
   - Generic retry strategy
   - Decoupled from HTTP
   - Reusable pattern

**Total**: 6-9 hours to extract 4 more components (~1,050 lines)

---

## 📈 Final Architecture (After Phase 2)

```
┌─────────────────────────────────────────────┐
│  Gateway (~1,800 lines)                     │
│  - HTTP routing & endpoints                 │
│  - KV store orchestration                   │
│  - HTTP client with retry                   │
│  - Configuration management                 │
│  - Session/cache/rate-limit                 │
│                                             │
│  imports (via WAC):                         │
│    toyota:business-logic/jwt                │
│    toyota:circuit-breaker/breaker           │
│    toyota:metrics/collector                 │
│    toyota:api-types/models           🆕    │
│    toyota:data-transform/converter   🆕    │
│    toyota:validation/validator       🆕    │
│    toyota:retry/strategy             🆕    │
└────────┬────────────────────────────────────┘
         │ WAC composition
         ↓
┌──────────────────────────────────────────────┐
│  Component Layer (Pure WASI)                 │
├──────────────────────────────────────────────┤
│  ✅ business-logic (400 lines)               │
│  ✅ circuit-breaker (184 lines)              │
│  ✅ metrics (228 lines)                      │
│  🆕 toyota-api-types (600 lines)            │
│  🆕 data-transform (200 lines)              │
│  🆕 validation (150 lines)                  │
│  🆕 retry-logic (100 lines)                 │
└──────────────────────────────────────────────┘
```

---

## 📊 Code Distribution (Final State)

| Layer | Lines | % of Total | Components |
|-------|-------|------------|------------|
| **Components** | 1,892 | 60% | 7 pure WASI |
| **Gateway** | 1,800 | 40% | 1 Spin-dependent |
| **Original** | 3,692 | - | Monolith |

**Reduction**: Gateway shrinks from 2,548 → 1,800 lines (29% reduction)
**Total components**: 1,892 lines across 7 independent modules

---

## 🎯 ROI Analysis

### High-Value Extractions (Do These)

1. **toyota-api-types** - ⭐⭐⭐⭐⭐
   - Reusable in ANY Toyota integration
   - Zero dependencies
   - Easy to maintain
   - Clear versioning
   - **ROI**: Extremely High

2. **data-transform** - ⭐⭐⭐⭐
   - Core business logic
   - Easy to test independently
   - Multiple output formats possible (not just ABRP)
   - **ROI**: Very High

### Medium-Value Extractions (Optional)

3. **validation** - ⭐⭐⭐
   - Generic validators
   - Reusable but simple
   - **ROI**: Medium

4. **retry-logic** - ⭐⭐⭐
   - Generic pattern
   - But HTTP client is still in gateway
   - **ROI**: Medium

---

## 🚫 What NOT to Extract

### 1. **HTTP Client Wrapper**
- Too tightly coupled to Spin SDK
- Uses `spin_sdk::http::send()`
- Integrates with circuit breaker
- **Keep in**: Gateway

### 2. **KV Store Operations**
- Uses Spin-specific KV store
- No WASI standard yet
- Gateway's core responsibility
- **Keep in**: Gateway

### 3. **Endpoint Handlers**
- This IS the gateway
- Orchestrates everything
- **Keep in**: Gateway

---

## ✅ Recommendation

**Extract in Phase 2**:
1. ✅ **toyota-api-types** (600 lines) - Do this first!
2. ✅ **data-transform** (200 lines) - Do this second!
3. ⚠️ **validation** (150 lines) - Optional
4. ⚠️ **retry-logic** (100 lines) - Optional

**Total extractable**: ~1,050 lines (33% of remaining code)
**Total effort**: 6-9 hours
**Value**: High - Reusable components with clear boundaries

**Leave in Gateway**: ~1,800 lines (57% of remaining code)
- KV store operations
- HTTP client
- Endpoint handlers
- Configuration
- Routing/dispatch

**Final Result**:
- 7-8 pure WASI components (1,892 lines)
- 1 thin gateway (1,800 lines)
- 60/40 split (components vs gateway)

---

**Bottom Line**: We can cleanly extract **~1,050 more lines** (33%) into 4 additional components. The remaining **1,800 lines** must stay in the gateway because they're inherently tied to Spin SDK (HTTP, KV store, variables).

This would give us **8 total components** with **60% of code** in reusable, independently testable pure WASI modules! 🎯
