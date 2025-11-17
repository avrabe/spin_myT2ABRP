// Toyota MyT2ABRP Web UI Component
//
// This component serves as the primary web interface for the MyT2ABRP application.
// It provides:
// - Static file serving (HTML, CSS, JavaScript) for the web dashboard
// - RESTful JSON API endpoints for vehicle status, charging control, and analytics
// - Clean separation of concerns: Rust returns JSON, JavaScript handles rendering
// - Health check and metrics endpoints for monitoring
//
// Architecture:
// - Built on Fermyon Spin (WebAssembly serverless runtime)
// - Compiles to wasm32-wasip2 target
// - JSON API-first design (modern web architecture)
// - Client-side rendering with vanilla JavaScript
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

        // API Endpoints - Vehicle Status (JSON)
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

            Ok(json_response(&serde_json::to_value(&status)?))
        }

        // API Endpoints - Charging Status (JSON)
        (Method::Get, "/api/charging/status") => {
            let status = ChargingStatus {
                is_charging: true,
                current_level: 85,
                target_level: 100,
                power_kw: 50.0,
                time_remaining_minutes: Some(45),
                charge_rate_kwh: 48.5,
            };

            Ok(json_response(&serde_json::to_value(&status)?))
        }

        // API Endpoints - Range (JSON)
        (Method::Get, "/api/range") => Ok(json_response(&json!({
            "estimated_range_km": 320,
            "range_at_80_percent_km": 280,
            "range_at_100_percent_km": 400,
        }))),

        // API Endpoints - Battery Health (JSON)
        (Method::Get, "/api/battery/health") => {
            let health = BatteryHealth {
                capacity_percentage: 98,
                health_status: "Excellent".to_string(),
                cycles: 120,
                temperature_celsius: 22.5,
            };

            Ok(json_response(&serde_json::to_value(&health)?))
        }

        // API Endpoints - Charging History (JSON)
        (Method::Get, "/api/charging/history") => Ok(json_response(&json!([
            {
                "date": "Today, 08:30",
                "start_level": 45,
                "end_level": 100,
                "duration_minutes": 135,
                "energy_kwh": 35.5
            },
            {
                "date": "Yesterday, 19:45",
                "start_level": 20,
                "end_level": 80,
                "duration_minutes": 90,
                "energy_kwh": 42.0
            },
            {
                "date": "2 days ago, 10:15",
                "start_level": 55,
                "end_level": 100,
                "duration_minutes": 105,
                "energy_kwh": 32.5
            }
        ]))),

        // API Endpoints - Active Alerts (JSON)
        (Method::Get, "/api/alerts/active") => Ok(json_response(&json!([
            {
                "type": "success",
                "title": "Charging Complete",
                "message": "Your vehicle is fully charged and ready to go!",
                "time_ago": "5 minutes ago"
            },
            {
                "type": "info",
                "title": "Optimal Charge Level Reached",
                "message": "Battery at 80% - optimal for battery longevity",
                "time_ago": "2 hours ago"
            }
        ]))),

        // API Endpoints - Analytics - Weekly (JSON)
        (Method::Get, "/api/analytics/weekly") => Ok(json_response(&json!({
            "charging_sessions": 7,
            "total_energy_kwh": 245,
            "avg_duration_minutes": 155
        }))),

        // API Endpoints - Analytics - Costs (JSON)
        (Method::Get, "/api/analytics/costs") => Ok(json_response(&json!({
            "this_week_cost": 42.50,
            "per_session_avg": 6.08,
            "avg_price_per_kwh": 0.173,
            "currency": "EUR"
        }))),

        // API Endpoints - Analytics - Efficiency (JSON)
        (Method::Get, "/api/analytics/efficiency") => Ok(json_response(&json!({
            "charging_efficiency_percent": 92,
            "avg_consumption_kwh_per_100km": 18.5,
            "battery_health_percent": 98
        }))),

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
