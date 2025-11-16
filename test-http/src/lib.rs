// Simple HTTP test component for Toyota MyT2ABRP
// Tests Spin HTTP trigger with comprehensive endpoint coverage

use spin_sdk::http::{IntoResponse, Method, Request, ResponseBuilder};
use spin_sdk::http_component;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct StatusResponse {
    message: String,
    version: String,
    status: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
}

#[derive(Serialize)]
struct MetricsResponse {
    requests: u64,
    uptime: u64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct ValidationResponse {
    valid: bool,
    message: String,
}

#[derive(Deserialize)]
struct ValidateRequest {
    #[allow(dead_code)]
    vin: Option<String>,
    #[allow(dead_code)]
    data: Option<serde_json::Value>,
}

/// HTTP request handler
#[http_component]
fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    let method = req.method();
    let path = req.path();

    let response = match (method, path) {
        (Method::Get, "/") => {
            let body = StatusResponse {
                message: "Toyota MyT2ABRP Test Component".to_string(),
                version: "0.1.0".to_string(),
                status: "ok".to_string(),
            };
            ResponseBuilder::new(200)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&body)?)
                .build()
        },

        (Method::Get, "/health") => {
            let body = HealthResponse {
                status: "healthy".to_string(),
                timestamp: "2025-11-16T00:00:00Z".to_string(),
            };
            ResponseBuilder::new(200)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&body)?)
                .build()
        },

        (Method::Get, "/status") => {
            ResponseBuilder::new(200)
                .header("content-type", "application/json")
                .body(r#"{"version":"0.1.0","components":["test-http"],"framework":"spin"}"#)
                .build()
        },

        (Method::Get, "/metrics") => {
            let body = MetricsResponse {
                requests: 1,
                uptime: 100,
            };
            ResponseBuilder::new(200)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&body)?)
                .build()
        },

        (Method::Post, "/validate") => {
            // Try to parse request body
            let _body: Result<ValidateRequest, _> = serde_json::from_slice(req.body());
            let response_body = ValidationResponse {
                valid: true,
                message: "Validation successful".to_string(),
            };
            ResponseBuilder::new(200)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&response_body)?)
                .build()
        },

        (Method::Post, "/transform") => {
            ResponseBuilder::new(200)
                .header("content-type", "application/json")
                .body(r#"{"transformed":true,"message":"Data transformed"}"#)
                .build()
        },

        (Method::Get, "/api/test-retry") => {
            ResponseBuilder::new(200)
                .header("content-type", "application/json")
                .body(r#"{"retry":"success","attempts":1}"#)
                .build()
        },

        (Method::Get, "/api/force-failure") => {
            let body = ErrorResponse {
                error: "Service unavailable".to_string(),
            };
            ResponseBuilder::new(503)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&body)?)
                .build()
        },

        (Method::Get, "/api/protected") => {
            // Check for Authorization header
            let mut has_auth = false;
            for (name, _value) in req.headers() {
                if name.eq_ignore_ascii_case("authorization") {
                    has_auth = true;
                    break;
                }
            }

            if has_auth {
                let body = ErrorResponse {
                    error: "Invalid token".to_string(),
                };
                ResponseBuilder::new(403)
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&body)?)
                    .build()
            } else {
                let body = ErrorResponse {
                    error: "Unauthorized".to_string(),
                };
                ResponseBuilder::new(401)
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&body)?)
                    .build()
            }
        },

        _ => {
            let body = ErrorResponse {
                error: "Not found".to_string(),
            };
            ResponseBuilder::new(404)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&body)?)
                .build()
        }
    };

    Ok(response)
}
