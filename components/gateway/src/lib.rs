/// Toyota MyT2ABRP Gateway - Composed from 7 components via WAC
///
/// This gateway is designed to orchestrate all sub-components via WAC composition:
/// - validation: Input validation for credentials
/// - retry: Retry strategy with exponential backoff
/// - business-logic: JWT token operations
/// - circuit-breaker: Resilient API calls
/// - data-transform: Toyota API ↔ ABRP conversion
/// - api-types: Type serialization/deserialization
/// - metrics: Prometheus metrics collection
///
/// IMPORTANT: Full WAC composition with all imports requires Bazel build.
/// Standalone cargo-component builds create a minimal gateway stub for testing.

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
            .body(r#"{"error":"Not found","message":"Gateway stub. Build with Bazel for full WAC composition","hint":"Try /health"}"#)
            .build(),
    };

    Ok(response)
}

/// Health check endpoint
fn handle_health() -> Response {
    ResponseBuilder::new(200)
        .header("content-type", "application/json")
        .body(r#"{"status":"healthy","message":"Gateway stub ready for WAC composition","note":"Build with Bazel to enable all 7 components","components_planned":["validation","retry","business-logic","circuit-breaker","data-transform","api-types","metrics"]}"#)
        .build()
}

// Full implementation with WAC-composed components would go here
// when built with Bazel. The imports from gateway.wit are wired at
// build time by wac_plug in BUILD.bazel.
//
// Example of how components would be used (requires Bazel + WAC):
//
// wit_bindgen::generate!({
//     world: "gateway",
//     path: "wit",
// });
//
// fn handle_validation_demo() -> Response {
//     use toyota::validation::validator;
//     let config = validator::get_default_config();
//     // ... use validation component
// }
