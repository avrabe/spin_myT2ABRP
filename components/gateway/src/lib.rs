/// Minimal Spin gateway for component composition proof-of-concept
///
/// This gateway component will import business-logic and circuit-breaker
/// via WAC composition at build time.
///
/// For now, it's a simple HTTP handler to prove the gateway builds correctly.

use spin_sdk::http::{IncomingRequest, Response, ResponseBuilder};
use spin_sdk::http_component;

/// HTTP component entry point
#[http_component]
async fn handle_request(req: IncomingRequest) -> anyhow::Result<Response> {
    let path = req.path_with_query().unwrap_or("/".to_string());

    let response = match path.as_str() {
        "/health" => handle_health(),
        _ => ResponseBuilder::new(404)
            .header("content-type", "application/json")
            .body(r#"{"error":"Not found","hint":"Try /health"}"#)
            .build(),
    };

    Ok(response)
}

/// Health check endpoint
fn handle_health() -> Response {
    ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(r#"{"status":"healthy","message":"Gateway component builds successfully"}"#)
        .build()
}
