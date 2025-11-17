// Toyota MyT2ABRP Web UI Component
//
// This component serves as the primary web interface for the MyT2ABRP application.
// It provides:
// - Static file serving (HTML, CSS, JavaScript) for the web dashboard
// - RESTful API endpoints for vehicle status, charging control, and analytics
// - HTMX-compatible HTML fragments for dynamic updates
// - Health check and metrics endpoints for monitoring
//
// Architecture:
// - Built on Fermyon Spin (WebAssembly serverless runtime)
// - Compiles to wasm32-wasip2 target
// - Uses minimal dependencies for optimal WASM binary size
// - Stateless design (all state managed externally)

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use spin_sdk::http::{IntoResponse, Method, Request, ResponseBuilder};
use spin_sdk::http_component;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ============================================================================
// Global Metrics Tracking
// ============================================================================
// These static atomics track application metrics across all requests.
// They are thread-safe and have minimal performance overhead.

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);
static SUCCESS_COUNTER: AtomicU64 = AtomicU64::new(0);
static ERROR_COUNTER: AtomicU64 = AtomicU64::new(0);
static START_TIME: once_cell::sync::Lazy<Instant> = once_cell::sync::Lazy::new(Instant::now);

// ============================================================================
// Error Types
// ============================================================================

/// Custom error type for web-ui operations
#[derive(Debug)]
enum WebUiError {
    /// File not found error
    NotFound(String),
    /// Internal server error
    InternalError(String),
    /// Invalid request error
    BadRequest(String),
}

impl std::fmt::Display for WebUiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebUiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            WebUiError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            WebUiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
        }
    }
}

impl std::error::Error for WebUiError {}

// ============================================================================
// Data Structures
// ============================================================================

/// Represents the current status of the vehicle
///
/// This structure contains real-time information about the vehicle's state,
/// including battery level, range, charging status, and location.
#[derive(Serialize)]
struct VehicleStatus {
    /// Vehicle Identification Number
    vin: String,
    /// Current battery level (0-100%)
    battery_level: u8,
    /// Estimated range in kilometers
    range_km: u16,
    /// Whether the vehicle is currently charging
    is_charging: bool,
    /// Whether the vehicle is connected to the cloud
    is_connected: bool,
    /// Optional GPS location of the vehicle
    location: Option<Location>,
}

/// GPS coordinates for vehicle location
#[derive(Serialize)]
struct Location {
    /// Latitude in decimal degrees
    lat: f64,
    /// Longitude in decimal degrees
    lon: f64,
}

/// Represents the current charging session status
///
/// Provides detailed information about an active or recent charging session,
/// including power delivery, timing, and target charge level.
#[derive(Serialize)]
struct ChargingStatus {
    /// Whether charging is currently active
    is_charging: bool,
    /// Current battery level (0-100%)
    current_level: u8,
    /// Target charge level (0-100%)
    target_level: u8,
    /// Charging power in kilowatts (kW)
    power_kw: f32,
    /// Estimated time to complete charging (in minutes)
    time_remaining_minutes: Option<u16>,
    /// Energy delivery rate in kWh
    charge_rate_kwh: f32,
}

/// Battery health metrics and diagnostics
///
/// Tracks the long-term health and condition of the vehicle's battery pack,
/// including degradation, cycle count, and operating temperature.
#[derive(Serialize)]
struct BatteryHealth {
    /// Current battery capacity as percentage of original (0-100%)
    capacity_percentage: u8,
    /// Human-readable health status (e.g., "Excellent", "Good", "Fair", "Poor")
    health_status: String,
    /// Number of complete charge/discharge cycles
    cycles: u32,
    /// Current battery pack temperature in Celsius
    temperature_celsius: f32,
}

/// Configuration for charging alerts and notifications
///
/// Allows users to customize when they receive notifications about their
/// vehicle's charging status.
///
/// Note: This structure is currently unused but reserved for future implementation
/// of the POST /api/alerts/save endpoint.
#[allow(dead_code)]
#[derive(Deserialize)]
struct AlertConfig {
    /// Send alert when charging is complete (100%)
    charge_complete: bool,
    /// Send alert at 80% (optimal for battery longevity)
    optimal_charge: bool,
    /// Custom alert level (50-100%)
    custom_level: u8,
    /// Alert when battery falls below 20%
    low_battery: bool,
    /// Alert when charging is slower than expected
    charging_slow: bool,
    /// Alert when vehicle is ready for a planned trip
    ready_for_trip: bool,
}

// ============================================================================
// Main HTTP Handler
// ============================================================================

/// Main HTTP request handler
///
/// This function is the entry point for all HTTP requests to the web-ui component.
/// It implements routing based on HTTP method and path, serving static files and
/// API endpoints as appropriate.
///
/// ## Request Logging
/// All requests are logged with method, path, and status code for observability.
///
/// ## Metrics Tracking
/// The handler increments request counters for monitoring and alerting.
///
/// ## Error Handling
/// Errors are logged to stderr and returned as appropriate HTTP error responses.
#[http_component]
fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    let path = req.path();
    let method = req.method();

    // Increment request counter
    REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Log incoming request
    eprintln!("[INFO] {} {}", method, path);

    // Route request and track result
    let result = match (method, path) {
        // Static files
        (Method::Get, "/") | (Method::Get, "/index.html") => {
            serve_static_file("index.html", "text/html")
        }
        (Method::Get, "/styles.css") => serve_static_file("styles.css", "text/css"),
        (Method::Get, "/app.js") => serve_static_file("app.js", "application/javascript"),

        // API Endpoints - Vehicle Status
        (Method::Get, "/api/vehicle/status") => {
            let status = VehicleStatus {
                vin: "TESTVIN123456789".to_string(),
                battery_level: 85,
                range_km: 320,
                is_charging: true,
                is_connected: true,
                location: Some(Location {
                    lat: 52.52,
                    lon: 13.405,
                }),
            };

            Ok(html_response(&render_vehicle_status(&status)))
        }

        // API Endpoints - Charging Status
        (Method::Get, "/api/charging/status") => {
            let status = ChargingStatus {
                is_charging: true,
                current_level: 85,
                target_level: 100,
                power_kw: 50.0,
                time_remaining_minutes: Some(45),
                charge_rate_kwh: 48.5,
            };

            Ok(html_response(&render_charging_status(&status)))
        }

        // API Endpoints - Range
        (Method::Get, "/api/range") => Ok(html_response(&format!(
            r#"<div class="range-info">
                    <div>
                        <div class="stat-label">Estimated Range</div>
                        <div class="range-value">320 km</div>
                    </div>
                    <div>
                        <div class="stat-label">Range @ 80%</div>
                        <div class="stat-value">280 km</div>
                    </div>
                </div>"#
        ))),

        // API Endpoints - Battery Health
        (Method::Get, "/api/battery/health") => {
            let health = BatteryHealth {
                capacity_percentage: 98,
                health_status: "Excellent".to_string(),
                cycles: 120,
                temperature_celsius: 22.5,
            };

            Ok(html_response(&render_battery_health(&health)))
        }

        // API Endpoints - Charging History
        (Method::Get, "/api/charging/history") => Ok(html_response(&render_charging_history())),

        // API Endpoints - Active Alerts
        (Method::Get, "/api/alerts/active") => Ok(html_response(&render_active_alerts())),

        // API Endpoints - Analytics
        (Method::Get, "/api/analytics/weekly") => Ok(html_response(&render_weekly_analytics())),

        (Method::Get, "/api/analytics/costs") => Ok(html_response(&render_cost_analytics())),

        (Method::Get, "/api/analytics/efficiency") => {
            Ok(html_response(&render_efficiency_analytics()))
        }

        // POST Endpoints - Actions
        (Method::Post, "/api/charging/start") => Ok(json_response(&json!({
            "success": true,
            "message": "Charging started"
        }))),

        (Method::Post, "/api/charging/stop") => Ok(json_response(&json!({
            "success": true,
            "message": "Charging stopped"
        }))),

        (Method::Post, "/api/precondition") => Ok(json_response(&json!({
            "success": true,
            "message": "Pre-conditioning started"
        }))),

        (Method::Post, "/api/alerts/save") => Ok(json_response(&json!({
            "success": true,
            "message": "Alert settings saved"
        }))),

        (Method::Post, "/api/settings/vehicle")
        | (Method::Post, "/api/settings/api")
        | (Method::Post, "/api/settings/notifications") => Ok(json_response(&json!({
            "success": true,
            "message": "Settings updated"
        }))),

        // Health Check & Monitoring Endpoints
        (Method::Get, "/health") | (Method::Get, "/api/health") => {
            Ok(json_response_with_security(&json!({
                "status": "healthy",
                "timestamp": Utc::now().to_rfc3339(),
                "version": env!("CARGO_PKG_VERSION"),
                "component": "web-ui"
            })))
        }

        (Method::Get, "/api/metrics") => {
            // Calculate real uptime
            let uptime = START_TIME.elapsed().as_secs();

            // Load atomic counters
            let total = REQUEST_COUNTER.load(Ordering::Relaxed);
            let success = SUCCESS_COUNTER.load(Ordering::Relaxed);
            let errors = ERROR_COUNTER.load(Ordering::Relaxed);

            // Calculate cache hit rate (placeholder - would need actual cache tracking)
            let cache_hit_rate = 0.0;

            Ok(json_response_with_security(&json!({
                "uptime_seconds": uptime,
                "requests_total": total,
                "requests_success": success,
                "requests_error": errors,
                "cache_hit_rate": cache_hit_rate
            })))
        }

        // 404 for unknown routes
        _ => {
            ERROR_COUNTER.fetch_add(1, Ordering::Relaxed);
            eprintln!("[WARN] 404 Not Found: {} {}", method, path);
            Ok(ResponseBuilder::new(404)
                .header("content-type", "text/html")
                .header("X-Content-Type-Options", "nosniff")
                .body("<h1>404 Not Found</h1>")
                .build())
        }
    };

    // Log result and update metrics
    match &result {
        Ok(_) => {
            SUCCESS_COUNTER.fetch_add(1, Ordering::Relaxed);
            eprintln!("[SUCCESS] {} {} - 200 OK", method, path);
        }
        Err(e) => {
            ERROR_COUNTER.fetch_add(1, Ordering::Relaxed);
            eprintln!("[ERROR] {} {} - Error: {}", method, path, e);
        }
    }

    result
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Serves a static file from the compiled binary
///
/// Static files are embedded into the WASM binary at compile time using
/// include_str! macro, eliminating the need for a separate file system.
///
/// ## Caching
/// Static files are cached for 1 hour (3600 seconds) via Cache-Control header
/// to reduce network traffic and improve performance.
///
/// ## Arguments
/// * `filename` - Name of the file to serve ("index.html", "styles.css", "app.js")
/// * `content_type` - MIME type for the Content-Type header
///
/// ## Returns
/// HTTP response with the file contents or 404 if file not found
fn serve_static_file(
    filename: &str,
    content_type: &str,
) -> anyhow::Result<spin_sdk::http::Response> {
    let content = match filename {
        "index.html" => include_str!("../static/index.html"),
        "styles.css" => include_str!("../static/styles.css"),
        "app.js" => include_str!("../static/app.js"),
        _ => {
            eprintln!("[ERROR] Static file not found: {}", filename);
            return Ok(ResponseBuilder::new(404).body("Not found").build());
        }
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", content_type)
        .header("cache-control", "public, max-age=3600")
        .header("X-Content-Type-Options", "nosniff")
        .body(content)
        .build())
}

/// Creates an HTML response for HTMX fragments
///
/// Used for endpoints that return HTML fragments to be swapped into the page
/// by HTMX. The response includes UTF-8 charset specification.
///
/// ## Arguments
/// * `html` - The HTML content to return
///
/// ## Returns
/// HTTP 200 response with HTML content type
fn html_response(html: &str) -> spin_sdk::http::Response {
    ResponseBuilder::new(200)
        .header("content-type", "text/html; charset=utf-8")
        .body(html)
        .build()
}

/// Creates a JSON response
///
/// Used for API endpoints that return structured data (e.g., action confirmations,
/// error messages).
///
/// ## Arguments
/// * `value` - The JSON value to serialize and return
///
/// ## Returns
/// HTTP 200 response with JSON content type
fn json_response(value: &serde_json::Value) -> spin_sdk::http::Response {
    ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(value.to_string())
        .build()
}

/// Creates a JSON response with comprehensive security headers
///
/// Used for sensitive endpoints (health checks, metrics) that should not be
/// cached or embedded in other sites.
///
/// ## Security Headers
/// - `X-Content-Type-Options: nosniff` - Prevents MIME type sniffing
/// - `X-Frame-Options: DENY` - Prevents clickjacking attacks
/// - `X-XSS-Protection: 1; mode=block` - Enables XSS filter
/// - `Referrer-Policy: strict-origin-when-cross-origin` - Controls referrer info
/// - `Cache-Control: no-store, no-cache, must-revalidate` - Prevents caching
///
/// ## Arguments
/// * `value` - The JSON value to serialize and return
///
/// ## Returns
/// HTTP 200 response with JSON content type and security headers
fn json_response_with_security(value: &serde_json::Value) -> spin_sdk::http::Response {
    ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "DENY")
        .header("X-XSS-Protection", "1; mode=block")
        .header("Referrer-Policy", "strict-origin-when-cross-origin")
        .header("Cache-Control", "no-store, no-cache, must-revalidate")
        .body(value.to_string())
        .build()
}

// ============================================================================
// HTML Rendering Functions
// ============================================================================
// These functions generate HTML fragments for HTMX to swap into the page.
// They use inline CSS for styling where needed and follow the Toyota design system.

/// Renders vehicle status as an HTML fragment
///
/// Creates a comprehensive vehicle status display including battery level indicator,
/// VIN, range, and current charging/parking status.
///
/// ## Arguments
/// * `status` - Current vehicle status data
///
/// ## Returns
/// HTML string with vehicle status markup
fn render_vehicle_status(status: &VehicleStatus) -> String {
    format!(
        r#"<div>
            <h2>Vehicle Status</h2>
            <div class="battery-indicator">
                <div class="battery-icon">
                    <div class="battery-level" style="width: {}%"></div>
                </div>
                <div class="battery-percentage">{}%</div>
            </div>
            <div class="status-grid">
                <div class="stat">
                    <div class="stat-label">VIN</div>
                    <div class="stat-value">{}</div>
                </div>
                <div class="stat">
                    <div class="stat-label">Range</div>
                    <div class="stat-value">{} km</div>
                </div>
                <div class="stat">
                    <div class="stat-label">Status</div>
                    <div class="stat-value">{}</div>
                </div>
            </div>
        </div>"#,
        status.battery_level,
        status.battery_level,
        status.vin,
        status.range_km,
        if status.is_charging {
            "⚡ Charging"
        } else {
            "🅿️ Parked"
        }
    )
}

/// Renders charging status as an HTML fragment
///
/// Displays current charging session information including progress bar,
/// power delivery, time remaining, and charge rate.
///
/// ## Arguments
/// * `status` - Current charging status data
///
/// ## Returns
/// HTML string with charging status markup including progress visualization
fn render_charging_status(status: &ChargingStatus) -> String {
    let time_remaining = status
        .time_remaining_minutes
        .map(|mins| format!("{} min", mins))
        .unwrap_or_else(|| "N/A".to_string());

    format!(
        r#"<div>
            <h2>Charging Status</h2>
            <div class="charging-progress">
                <div class="progress-bar">
                    <div class="progress-fill" style="width: {}%"></div>
                </div>
                <div style="margin-top: 10px; font-size: 14px;">
                    {}% → {}% target
                </div>
            </div>
            <div class="charging-stats">
                <div class="stat">
                    <div class="stat-value">{:.1} kW</div>
                    <div class="stat-label">Power</div>
                </div>
                <div class="stat">
                    <div class="stat-value">{}</div>
                    <div class="stat-label">Time Left</div>
                </div>
                <div class="stat">
                    <div class="stat-value">{:.1} kWh</div>
                    <div class="stat-label">Rate</div>
                </div>
            </div>
        </div>"#,
        status.current_level,
        status.current_level,
        status.target_level,
        status.power_kw,
        time_remaining,
        status.charge_rate_kwh
    )
}

/// Renders battery health metrics as an HTML fragment
///
/// Shows battery capacity, health status, charge cycles, and temperature
/// in an easy-to-read format with visual emphasis on the health percentage.
///
/// ## Arguments
/// * `health` - Battery health data
///
/// ## Returns
/// HTML string with battery health metrics
fn render_battery_health(health: &BatteryHealth) -> String {
    format!(
        r#"<div>
            <div class="health-status" style="text-align: center; margin: 20px 0;">
                <div style="font-size: 48px; font-weight: 700; color: var(--success-color);">
                    {}%
                </div>
                <div style="font-size: 18px; margin-top: 10px;">
                    {} Health
                </div>
            </div>
            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 15px;">
                <div class="stat">
                    <div class="stat-label">Charge Cycles</div>
                    <div class="stat-value">{}</div>
                </div>
                <div class="stat">
                    <div class="stat-label">Temperature</div>
                    <div class="stat-value">{:.1}°C</div>
                </div>
            </div>
        </div>"#,
        health.capacity_percentage, health.health_status, health.cycles, health.temperature_celsius
    )
}

/// Renders recent charging history as an HTML fragment
///
/// Shows a list of recent charging sessions with date, time, duration,
/// and energy consumed. Currently uses demo data - would be replaced
/// with actual historical data from a database.
///
/// ## Returns
/// HTML string with charging history list and embedded styles
fn render_charging_history() -> String {
    r#"<div class="history-list">
        <div class="history-item">
            <div><strong>Today, 08:30</strong></div>
            <div>45% → 100% | 2h 15min | 35.5 kWh</div>
        </div>
        <div class="history-item">
            <div><strong>Yesterday, 19:45</strong></div>
            <div>20% → 80% | 1h 30min | 42.0 kWh</div>
        </div>
        <div class="history-item">
            <div><strong>2 days ago, 10:15</strong></div>
            <div>55% → 100% | 1h 45min | 32.5 kWh</div>
        </div>
    </div>
    <style>
        .history-list { display: flex; flex-direction: column; gap: 10px; }
        .history-item { padding: 12px; background: var(--background); border-radius: 8px; }
        .history-item div:last-child { font-size: 13px; color: var(--text-secondary); margin-top: 5px; }
    </style>"#.to_string()
}

/// Renders active charging alerts as an HTML fragment
///
/// Displays current active alerts and notifications related to charging,
/// battery status, and vehicle readiness. Alerts are styled with different
/// colors based on their type (success, warning, info).
///
/// ## Returns
/// HTML string with alerts list and embedded styles
fn render_active_alerts() -> String {
    r#"<div class="alerts-list">
        <div class="alert-item success">
            <div><strong>Charging Complete</strong></div>
            <div>Your vehicle is fully charged and ready to go!</div>
            <div style="font-size: 12px; color: var(--text-secondary); margin-top: 5px;">5 minutes ago</div>
        </div>
        <div class="alert-item">
            <div><strong>Optimal Charge Level Reached</strong></div>
            <div>Battery at 80% - optimal for battery longevity</div>
            <div style="font-size: 12px; color: var(--text-secondary); margin-top: 5px;">2 hours ago</div>
        </div>
    </div>
    <style>
        .alerts-list { display: flex; flex-direction: column; gap: 10px; }
        .alert-item.success { border-left-color: var(--success-color); }
    </style>"#.to_string()
}

/// Renders weekly charging statistics as an HTML fragment
///
/// Shows aggregated charging data for the past week including number of
/// sessions, total energy consumed, and average session duration.
///
/// ## Returns
/// HTML string with weekly statistics
fn render_weekly_analytics() -> String {
    r#"<div>
        <h3>Weekly Charging Stats</h3>
        <div style="margin: 20px 0;">
            <div class="stat">
                <div class="stat-value">7</div>
                <div class="stat-label">Charging Sessions</div>
            </div>
            <div class="stat" style="margin-top: 15px;">
                <div class="stat-value">245 kWh</div>
                <div class="stat-label">Total Energy</div>
            </div>
            <div class="stat" style="margin-top: 15px;">
                <div class="stat-value">2h 35min</div>
                <div class="stat-label">Avg. Duration</div>
            </div>
        </div>
    </div>"#
        .to_string()
}

/// Renders charging cost analysis as an HTML fragment
///
/// Displays cost breakdown for charging sessions including weekly/monthly totals,
/// average cost per session, and average price per kWh.
///
/// ## Returns
/// HTML string with cost analysis metrics
fn render_cost_analytics() -> String {
    r#"<div>
        <h3>Cost Analysis</h3>
        <div style="margin: 20px 0;">
            <div class="stat">
                <div class="stat-value">€42.50</div>
                <div class="stat-label">This Week</div>
            </div>
            <div class="stat" style="margin-top: 15px;">
                <div class="stat-value">€6.08</div>
                <div class="stat-label">Per Session Avg.</div>
            </div>
            <div class="stat" style="margin-top: 15px;">
                <div class="stat-value">€0.173/kWh</div>
                <div class="stat-label">Avg. Price</div>
            </div>
        </div>
    </div>"#
        .to_string()
}

/// Renders energy efficiency metrics as an HTML fragment
///
/// Shows vehicle efficiency data including charging efficiency, average
/// consumption per 100km, and battery health percentage.
///
/// ## Returns
/// HTML string with efficiency metrics
fn render_efficiency_analytics() -> String {
    r#"<div>
        <h3>Efficiency Metrics</h3>
        <div style="margin: 20px 0;">
            <div class="stat">
                <div class="stat-value">92%</div>
                <div class="stat-label">Charging Efficiency</div>
            </div>
            <div class="stat" style="margin-top: 15px;">
                <div class="stat-value">18.5 kWh/100km</div>
                <div class="stat-label">Avg. Consumption</div>
            </div>
            <div class="stat" style="margin-top: 15px;">
                <div class="stat-value">98%</div>
                <div class="stat-label">Battery Health</div>
            </div>
        </div>
    </div>"#
        .to_string()
}
