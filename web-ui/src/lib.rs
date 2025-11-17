// Toyota MyT2ABRP Web UI Component
// Serves static files and provides API endpoints with HTMX support

use spin_sdk::http::{IntoResponse, Method, Request, ResponseBuilder};
use spin_sdk::http_component;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Vehicle Status
#[derive(Serialize)]
struct VehicleStatus {
    vin: String,
    battery_level: u8,
    range_km: u16,
    is_charging: bool,
    is_connected: bool,
    location: Option<Location>,
}

#[derive(Serialize)]
struct Location {
    lat: f64,
    lon: f64,
}

// Charging Status
#[derive(Serialize)]
struct ChargingStatus {
    is_charging: bool,
    current_level: u8,
    target_level: u8,
    power_kw: f32,
    time_remaining_minutes: Option<u16>,
    charge_rate_kwh: f32,
}

// Battery Health
#[derive(Serialize)]
struct BatteryHealth {
    capacity_percentage: u8,
    health_status: String,
    cycles: u32,
    temperature_celsius: f32,
}

// Alert Configuration (reserved for future POST /api/alerts/save implementation)
#[allow(dead_code)]
#[derive(Deserialize)]
struct AlertConfig {
    charge_complete: bool,
    optimal_charge: bool,
    custom_level: u8,
    low_battery: bool,
    charging_slow: bool,
    ready_for_trip: bool,
}

#[http_component]
fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    let path = req.path();
    let method = req.method();

    match (method, path) {
        // Static files
        (Method::Get, "/") | (Method::Get, "/index.html") => {
            serve_static_file("index.html", "text/html")
        }
        (Method::Get, "/styles.css") => {
            serve_static_file("styles.css", "text/css")
        }
        (Method::Get, "/app.js") => {
            serve_static_file("app.js", "application/javascript")
        }

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
        (Method::Get, "/api/range") => {
            Ok(html_response(&format!(
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
            )))
        }

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
        (Method::Get, "/api/charging/history") => {
            Ok(html_response(&render_charging_history()))
        }

        // API Endpoints - Active Alerts
        (Method::Get, "/api/alerts/active") => {
            Ok(html_response(&render_active_alerts()))
        }

        // API Endpoints - Analytics
        (Method::Get, "/api/analytics/weekly") => {
            Ok(html_response(&render_weekly_analytics()))
        }

        (Method::Get, "/api/analytics/costs") => {
            Ok(html_response(&render_cost_analytics()))
        }

        (Method::Get, "/api/analytics/efficiency") => {
            Ok(html_response(&render_efficiency_analytics()))
        }

        // POST Endpoints - Actions
        (Method::Post, "/api/charging/start") => {
            Ok(json_response(&json!({
                "success": true,
                "message": "Charging started"
            })))
        }

        (Method::Post, "/api/charging/stop") => {
            Ok(json_response(&json!({
                "success": true,
                "message": "Charging stopped"
            })))
        }

        (Method::Post, "/api/precondition") => {
            Ok(json_response(&json!({
                "success": true,
                "message": "Pre-conditioning started"
            })))
        }

        (Method::Post, "/api/alerts/save") => {
            Ok(json_response(&json!({
                "success": true,
                "message": "Alert settings saved"
            })))
        }

        (Method::Post, "/api/settings/vehicle") |
        (Method::Post, "/api/settings/api") |
        (Method::Post, "/api/settings/notifications") => {
            Ok(json_response(&json!({
                "success": true,
                "message": "Settings updated"
            })))
        }

        // 404 for unknown routes
        _ => Ok(ResponseBuilder::new(404)
            .header("content-type", "text/html")
            .body("<h1>404 Not Found</h1>")
            .build()),
    }
}

// Helper: Serve static file
fn serve_static_file(filename: &str, content_type: &str) -> anyhow::Result<spin_sdk::http::Response> {
    let content = match filename {
        "index.html" => include_str!("../static/index.html"),
        "styles.css" => include_str!("../static/styles.css"),
        "app.js" => include_str!("../static/app.js"),
        _ => return Ok(ResponseBuilder::new(404).body("Not found").build()),
    };

    Ok(ResponseBuilder::new(200)
        .header("content-type", content_type)
        .header("cache-control", "public, max-age=3600")
        .body(content)
        .build())
}

// Helper: HTML response
fn html_response(html: &str) -> spin_sdk::http::Response {
    ResponseBuilder::new(200)
        .header("content-type", "text/html; charset=utf-8")
        .body(html)
        .build()
}

// Helper: JSON response
fn json_response(value: &serde_json::Value) -> spin_sdk::http::Response {
    ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(value.to_string())
        .build()
}

// Render Functions
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
        if status.is_charging { "⚡ Charging" } else { "🅿️ Parked" }
    )
}

fn render_charging_status(status: &ChargingStatus) -> String {
    let time_remaining = status.time_remaining_minutes
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
        health.capacity_percentage,
        health.health_status,
        health.cycles,
        health.temperature_celsius
    )
}

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
    </div>"#.to_string()
}

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
    </div>"#.to_string()
}

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
    </div>"#.to_string()
}
