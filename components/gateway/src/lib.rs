//! Toyota MyT2ABRP Gateway - Component Model P2 HTTP Handler
//!
//! This gateway is a pure WebAssembly Component Model P2 component that:
//! - Exports wasi:http/incoming-handler@0.2.0 for standard HTTP handling
//! - Imports 7 business logic components for orchestration
//! - Can be composed with WAC (WebAssembly Composition)
//! - Compatible with Spin runtime and other Component Model hosts
//!
//! Components orchestrated via WAC composition:
//! - validation: Input validation for credentials
//! - retry: Retry strategy with exponential backoff
//! - business-logic: JWT token operations
//! - circuit-breaker: Resilient API calls
//! - data-transform: Toyota API ↔ ABRP conversion
//! - api-types: Type serialization/deserialization
//! - metrics: Prometheus metrics collection

// Generate bindings for the gateway world
wit_bindgen::generate!({
    world: "gateway",
    path: "wit",
    exports: {
        "wasi:http/incoming-handler@0.2.0": GatewayComponent,
    },
});

use exports::wasi::http::incoming_handler::Guest;
use wasi::http::types::{
    Fields, IncomingRequest, OutgoingResponse, ResponseOutparam, StatusCode,
};

/// Gateway component implementation
struct GatewayComponent;

impl Guest for GatewayComponent {
    /// Handle incoming HTTP requests (WASI HTTP 0.2.0)
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        let response = match handle_request_internal(&request) {
            Ok(resp) => resp,
            Err(e) => create_error_response(500, &format!("Internal error: {}", e)),
        };

        ResponseOutparam::set(response_out, Ok(response));
    }
}

/// Internal request handler with error handling
fn handle_request_internal(request: &IncomingRequest) -> Result<OutgoingResponse, String> {
    // Extract request path
    let path = request
        .path_with_query()
        .unwrap_or_else(|| "/".to_string());

    match path.as_str() {
        "/health" => Ok(handle_health()),
        "/validate" => Ok(handle_validate()),
        "/jwt/generate" => Ok(handle_jwt_generate()),
        "/transform" => Ok(handle_transform()),
        "/metrics" => Ok(handle_metrics()),
        _ => Ok(create_error_response(
            404,
            &format!(
                "{{\"error\":\"Not found\",\"path\":\"{}\",\"available_endpoints\":[\"/health\",\"/validate\",\"/jwt/generate\",\"/transform\",\"/metrics\"]}}",
                path
            ),
        )),
    }
}

/// Health check endpoint - demonstrates basic Component Model functionality
fn handle_health() -> OutgoingResponse {
    create_json_response(
        200,
        r#"{"status":"healthy","message":"Component Model P2 Gateway","components":{"validation":"ready","retry":"ready","business-logic":"ready","circuit-breaker":"ready","data-transform":"ready","api-types":"ready","metrics":"ready"},"wasi_version":"0.2.0"}"#,
    )
}

/// Validation endpoint - demonstrates using imported validation component
fn handle_validate() -> OutgoingResponse {
    // Example: Use the validation component imported via WAC
    use toyota::validation::validator;

    let config = validator::get_default_config();
    let test_email = "test@example.com";

    match validator::is_valid_email(test_email) {
        Ok(is_valid) => {
            let body = format!(
                r#"{{"endpoint":"validate","email":"{}","is_valid":{},"config":{{"min_length":{},"max_length":{}}}}}"#,
                test_email, is_valid, config.min_length, config.max_length
            );
            create_json_response(200, &body)
        }
        Err(e) => create_error_response(400, &format!("Validation error: {}", e)),
    }
}

/// JWT generation endpoint - demonstrates using imported business-logic component
fn handle_jwt_generate() -> OutgoingResponse {
    // Example: Use the JWT component imported via WAC
    use toyota::business_logic::jwt;

    let username = "demo_user";
    let jwt_secret = b"demo_secret_key_for_testing_only";

    match jwt::generate_access_token(username, jwt_secret) {
        Ok(token) => {
            let body = format!(
                r#"{{"endpoint":"jwt/generate","username":"{}","token_preview":"{}...","note":"This is a demo token"}}"#,
                username,
                &token.chars().take(20).collect::<String>()
            );
            create_json_response(200, &body)
        }
        Err(e) => create_error_response(400, &format!("JWT error: {}", e)),
    }
}

/// Transform endpoint - demonstrates using imported data-transform component
fn handle_transform() -> OutgoingResponse {
    // Example: Use the data-transform component imported via WAC
    use toyota::data_transform::converter;

    let sample_electric_status = r#"{"batteryLevel":80}"#;
    let version = "1.0.0";

    match converter::toyota_to_abrp(sample_electric_status, None, None, version) {
        Ok(telemetry) => {
            let body = format!(
                r#"{{"endpoint":"transform","soc":{},"utc":{},"version":"{}"}}"#,
                telemetry.soc, telemetry.utc, telemetry.version
            );
            create_json_response(200, &body)
        }
        Err(e) => create_error_response(400, &format!("Transform error: {}", e)),
    }
}

/// Metrics endpoint - demonstrates using imported metrics component
fn handle_metrics() -> OutgoingResponse {
    // Example: Use the metrics component imported via WAC
    use toyota::metrics::collector;

    collector::increment_counter("gateway_requests", 1);
    collector::record_histogram("request_duration_ms", 42.5);

    let metrics = collector::get_all_metrics();
    let body = format!(
        r#"{{"endpoint":"metrics","total_metrics":{},"sample":"{:?}"}}"#,
        metrics.len(),
        metrics.first()
    );
    create_json_response(200, &body)
}

/// Helper: Create a JSON response with status code and body
fn create_json_response(status: u16, body: &str) -> OutgoingResponse {
    let headers = Fields::new();
    let _ = headers.set(
        &"content-type".to_string(),
        &[b"application/json".to_vec()],
    );

    let response = OutgoingResponse::new(headers);
    response.set_status_code(status).ok();

    let response_body = response.body().expect("Failed to get response body");
    let output_stream = response_body
        .write()
        .expect("Failed to get output stream");

    output_stream
        .blocking_write_and_flush(body.as_bytes())
        .expect("Failed to write response body");

    drop(output_stream);
    OutgoingResponse::finish(response_body, None).ok();

    response
}

/// Helper: Create an error response
fn create_error_response(status: u16, message: &str) -> OutgoingResponse {
    let body = format!(r#"{{"error":true,"status":{},"message":"{}"}}"#, status, message);
    create_json_response(status, &body)
}
