# MyT2ABRP Architecture

Complete architecture documentation for the Toyota MyT to A Better Route Planner (MyT2ABRP) project.

## Overview

MyT2ABRP is a comprehensive multi-platform application that enhances the Toyota electric vehicle experience with smart charging management, battery health tracking, and intelligent route planning.

## Technology Stack

### Backend (Spin + WebAssembly)
- **Runtime**: Fermyon Spin (WebAssembly serverless)
- **Language**: Rust
- **Build System**: Bazel + Cargo
- **Components**:
  - `myt2abrp` - Main API (Toyota integration)
  - `web-ui` - HTMX dashboard (frontend)

### Frontend (Web)
- **Framework**: HTMX (hypermedia-driven)
- **Styling**: Custom CSS (Toyota brand colors)
- **JavaScript**: Minimal vanilla JS + PWA
- **Architecture**: Progressive Enhancement

### Mobile (iOS & watchOS)
- **Language**: Swift
- **Framework**: SwiftUI
- **Architecture**: MVVM (Model-View-ViewModel)
- **Platforms**: iOS 17+, watchOS 10+

### Testing
- **E2E**: Playwright
- **Browsers**: Chrome, Firefox, Safari, Mobile
- **Coverage**: Web UI, API, Integration

## Project Structure

```
spin_myT2ABRP/
├── components/                 # Bazel-built Rust components
│   └── myt2abrp/              # Main Toyota API integration
│       ├── src/
│       ├── wit/
│       └── BUILD.bazel
│
├── web-ui/                    # HTMX Web Dashboard (Cargo)
│   ├── src/
│   │   └── lib.rs             # Spin HTTP component
│   ├── static/
│   │   ├── index.html         # HTMX dashboard
│   │   ├── styles.css         # Toyota styling
│   │   └── app.js             # Minimal JS + PWA
│   ├── Cargo.toml
│   └── BUILD_FIX_NOTES.md
│
├── ios-app/                   # iOS Native App
│   └── MyT2ABRP/
│       ├── ViewModels/        # Business logic
│       │   ├── VehicleManager.swift
│       │   ├── ChargingManager.swift
│       │   └── NotificationManager.swift
│       ├── Views/             # SwiftUI views
│       │   ├── ContentView.swift
│       │   ├── ChargingView.swift
│       │   ├── RoutesView.swift
│       │   ├── AnalyticsView.swift
│       │   └── SettingsView.swift
│       └── MyT2ABRPApp.swift
│
├── watchos-app/               # Apple Watch Companion
│   └── MyT2ABRP Watch App/
│       ├── MyT2ABRPWatchApp.swift
│       └── Complications.swift
│
├── tests/                     # Playwright E2E Tests
│   ├── tests/
│   │   ├── web-ui/           # Web UI tests
│   │   ├── api/              # API tests
│   │   └── integration/      # Integration tests
│   ├── playwright.config.ts
│   ├── package.json
│   └── README.md
│
├── docs/                      # Documentation
│   └── STATIC_WEB_RESEARCH.md
│
├── spin.toml                  # Spin app configuration
├── BUILD.bazel                # Bazel workspace
├── build.sh                   # Build script
├── .gitignore
├── SESSION_2_SUMMARY.md       # Implementation summary
└── ARCHITECTURE.md            # This file
```

## Component Architecture

### 1. Spin Application (spin.toml)

The Spin configuration defines two HTTP components:

#### Web UI Component (`web-ui`)
- **Routes**: `/`, `/styles.css`, `/app.js`, `/api/...`
- **Source**: `web-ui/target/wasm32-wasip2/release/web_ui.wasm`
- **Purpose**: Serve HTMX dashboard and API endpoints
- **Build**: `cd web-ui && cargo build --target wasm32-wasip2 --release`

#### Main API Component (`myt2abrp`)
- **Routes**: `/...` (catch-all, matched last)
- **Source**: `bazel-bin/myt2abrp_app.wasm`
- **Purpose**: Toyota API integration
- **Build**: `./build.sh` (Bazel)
- **Outbound Access**: Toyota Europe API endpoints
- **Storage**: Key-value store for authentication

### 2. Web UI Architecture

#### HTMX Pattern

The web UI uses HTMX for dynamic updates without a JavaScript SPA:

```html
<!-- Auto-refreshing status card (every 5 seconds) -->
<div class="status-card"
     hx-get="/api/vehicle/status"
     hx-trigger="load, every 5s"
     hx-swap="innerHTML">
  Loading...
</div>

<!-- Action button with POST -->
<button hx-post="/api/charging/start"
        hx-swap="none">
  Start Charging
</button>
```

**Benefits**:
- Minimal JavaScript (< 1KB custom code)
- Progressive enhancement
- Server-side rendering
- Simple state management
- Fast page loads

#### API Endpoints (HTML Fragments)

All `/api/*` endpoints return HTML fragments (not JSON):

```rust
// Rust handler
fn render_vehicle_status(status: &VehicleStatus) -> String {
    format!(r#"
        <div>
            <h2>Vehicle Status</h2>
            <div class="battery-percentage">{}</div>
            <div class="range">{} km</div>
        </div>
    "#, status.battery_level, status.range_km)
}
```

This enables HTMX to directly inject responses into the DOM.

#### Static Files (Embedded)

Static files are embedded at compile time:

```rust
fn serve_static_file(filename: &str) -> Result<Response> {
    let content = match filename {
        "index.html" => include_str!("../static/index.html"),
        "styles.css" => include_str!("../static/styles.css"),
        "app.js" => include_str!("../static/app.js"),
        _ => return Ok(404_response()),
    };

    Ok(Response::new(200)
        .header("content-type", content_type)
        .header("cache-control", "public, max-age=3600")
        .body(content))
}
```

**Advantages**:
- Single WASM binary
- No external file dependencies
- Fast serving
- Cacheable responses

### 3. iOS App Architecture (MVVM)

#### ViewModels (Observable Business Logic)

**VehicleManager**:
```swift
class VehicleManager: ObservableObject {
    @Published var batteryLevel: Int = 0
    @Published var rangeKm: Int = 0
    @Published var batteryHealth: Int = 100

    // Auto-refresh every 5 minutes
    func refresh() async { /* ... */ }

    // Calculate optimal range at 80%
    private func calculateRangeAt80Percent() -> Int { /* ... */ }
}
```

**ChargingManager**:
```swift
class ChargingManager: ObservableObject {
    @Published var isCharging: Bool = false
    @Published var currentLevel: Int = 0
    @Published var targetLevel: Int = 100

    // Smart charging alerts (INNOVATIVE)
    @Published var alertAt80Percent: Bool = true
    @Published var customAlertLevel: Int = 90

    // Check and trigger alerts
    func checkChargingAlerts() { /* ... */ }
}
```

**NotificationManager**:
```swift
class NotificationManager: ObservableObject {
    // Handle all notifications
    func sendChargingNotification(_ alert: ChargingAlert) { /* ... */ }

    // Quiet hours support
    private func isInQuietHours() -> Bool { /* ... */ }
}
```

#### Views (SwiftUI Declarative UI)

**ContentView** (Dashboard):
- Tab navigation
- Battery status with animated ring
- Charging progress
- Quick actions
- Recent alerts

**ChargingView** (Detailed Controls):
- Charge target slider (50-100%)
- Smart alert configuration
- Charging history
- Session statistics

**AnalyticsView** (Insights):
- Battery health tracking
- Cost analysis
- Efficiency metrics
- Environmental impact

**RoutesView** (Planning):
- Map with nearby chargers
- Route planning with charging stops
- Saved routes
- Trip preparation

**SettingsView** (Configuration):
- Vehicle details
- API configuration
- Notification preferences
- App settings

#### Data Flow

```
User Action → View
       ↓
   ViewModel (@Published)
       ↓
   API Call (URLSession)
       ↓
   Spin Server (/api/*)
       ↓
   Response → Update @Published
       ↓
   SwiftUI Auto-Updates View
```

#### Persistence

- **UserDefaults**: Cached data, preferences
- **Keychain**: Sensitive tokens (future)
- **Background Fetch**: 15-minute updates

### 4. watchOS App Architecture

#### Main App

Three-page Tab View:
1. **Battery View**: Large percentage, range, health
2. **Charging Control**: Start/stop, progress ring
3. **Quick Actions**: Charge target, pre-condition

#### Complications (Watch Face Widgets)

```swift
struct MyT2ABRPComplicationProvider: TimelineProvider {
    func getTimeline() -> Timeline<Entry> {
        let entry = Entry(
            date: Date(),
            batteryLevel: UserDefaults.batteryLevel,
            isCharging: UserDefaults.isCharging,
            range: UserDefaults.range
        )

        // Refresh every 15 minutes
        Timeline(entries: [entry], policy: .after(Date() + 900))
    }
}
```

**Supported Families**:
- Circular (large round)
- Rectangular
- Inline (text)
- Corner

#### Haptic Feedback

```swift
// Haptic confirmation on actions
WKInterfaceDevice.current().play(.start)  // Charging started
WKInterfaceDevice.current().play(.stop)   // Charging stopped
WKInterfaceDevice.current().play(.click)  // Button pressed
```

### 5. Testing Architecture

#### Playwright E2E Tests

**Test Structure**:
```
tests/
├── web-ui/          # UI behavior tests
├── api/             # Endpoint tests
└── integration/     # Component integration
```

**Test Execution**:
1. `playwright.config.ts` starts Spin server
2. Tests run against `http://localhost:3000`
3. Results captured with screenshots/videos
4. Reports generated in `test-results/`

**Coverage**:
- All API endpoints
- HTMX functionality
- Cross-browser compatibility
- Mobile responsiveness
- Performance benchmarks

## Data Models

### Vehicle Status

```typescript
interface VehicleStatus {
    vin: string;
    battery_level: number;      // 0-100%
    range_km: number;
    is_charging: boolean;
    is_connected: boolean;
    location?: {
        lat: number;
        lon: number;
    };
}
```

### Charging Status

```typescript
interface ChargingStatus {
    is_charging: boolean;
    current_level: number;      // 0-100%
    target_level: number;       // 50-100%
    power_kw: number;
    time_remaining_minutes?: number;
    charge_rate_kwh: number;
}
```

### Battery Health

```typescript
interface BatteryHealth {
    capacity_percentage: number;  // 0-100%
    health_status: string;        // "Excellent", "Good", "Fair"
    cycles: number;
    temperature_celsius: number;
}
```

## Communication Patterns

### Web UI → Spin API

**HTMX Polling** (Real-time updates):
```
Browser                 Spin Server
   │                         │
   ├─ GET /api/vehicle/status (every 5s)
   │                         │
   │  ◀──── HTML Fragment ───┤
   │                         │
   └─ Swap into DOM
```

**User Actions**:
```
Browser                 Spin Server
   │                         │
   ├─ POST /api/charging/start
   │                         │
   │  ◀──── { success: true }
   │                         │
   └─ Update UI
```

### iOS App → Spin API

**Periodic Refresh**:
```
iOS App                 Spin Server
   │                         │
   ├─ Timer (5 min) ─────────┤
   │                         │
   ├─ URLSession GET /api/vehicle/status
   │                         │
   │  ◀──── JSON Response ───┤
   │                         │
   ├─ Update @Published vars
   │                         │
   └─ SwiftUI re-renders
```

**Background Fetch**:
```
iOS (Background)        Spin Server
   │                         │
   ├─ BGTaskScheduler (15 min)
   │                         │
   ├─ Fetch vehicle status ──┤
   │                         │
   │  ◀──── Response ────────┤
   │                         │
   ├─ Check for alerts
   │                         │
   └─ Send notifications (if needed)
```

### watchOS ↔ iOS

**Shared Data**:
- UserDefaults with app group
- WatchConnectivity framework
- Shared ViewModels

## Deployment

### Local Development

```bash
# Build all components
spin build

# Run server
spin up

# Access:
# - Web UI: http://localhost:3000
# - API: http://localhost:3000/api/*

# Run tests
cd tests
npm test
```

### Production Deployment

```bash
# Build optimized
cargo build --release --target wasm32-wasip2

# Deploy to Fermyon Cloud
spin deploy

# Or self-host with Docker
docker run -p 3000:3000 -v $(pwd):/app fermyon/spin:latest
```

### iOS/watchOS Deployment

```bash
# Open in Xcode
open ios-app/MyT2ABRP.xcodeproj

# Build and run
# Cmd+R (Simulator)
# Cmd+R (Device - requires Apple Developer account)
```

## Security Considerations

### Web UI
- **CSP**: Content Security Policy headers
- **CORS**: Configured origin restrictions
- **HTTPS**: TLS for production
- **Input Validation**: All user inputs sanitized

### API
- **Authentication**: JWT tokens (stored securely)
- **Rate Limiting**: Prevent abuse
- **HMAC**: Username hashing
- **Secrets**: Environment variables, not hardcoded

### iOS App
- **Keychain**: Sensitive data storage
- **Certificate Pinning**: Toyota API
- **Sandboxing**: iOS sandbox model
- **Permissions**: Only necessary permissions requested

## Performance Optimizations

### Web UI
- **Static file embedding**: No file I/O
- **HTTP caching**: 1-hour cache for static files
- **HTMX polling**: Only changed content transfers
- **Minimal JavaScript**: < 1KB custom code

### Spin Components
- **WASM**: Near-native performance
- **Component isolation**: Separate web-ui and API
- **Connection pooling**: HTTP client reuse
- **Async I/O**: Non-blocking operations

### iOS App
- **SwiftUI**: Optimized rendering
- **Lazy loading**: Charts and history
- **Background fetch**: Minimal battery impact
- **Cached data**: Reduce API calls

## Monitoring & Observability

### Metrics (Future)
- API response times
- Error rates
- User engagement
- Battery health trends
- Charging patterns

### Logging
- Structured logging in Rust
- Console logging in Swift
- Playwright test traces

## Future Enhancements

### Short Term
- [ ] Real Toyota API integration
- [ ] User authentication
- [ ] Data persistence (database)
- [ ] Push notifications (iOS)
- [ ] Offline support (PWA)

### Long Term
- [ ] Android app
- [ ] CarPlay integration
- [ ] Machine learning (charge predictions)
- [ ] Social features (route sharing)
- [ ] Integration with home charging systems
- [ ] Multi-vehicle support

## License

MIT

## Contributors

- Ralf Anton Beier <ralf_beier@me.com>

---

**Last Updated**: 2025-11-17
**Version**: 1.0.0
