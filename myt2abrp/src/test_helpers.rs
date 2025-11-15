//! Test Helper Functions and Mocks
//!
//! This module provides utilities for testing the myt2abrp gateway application.
//! It includes mock data, helper functions, and test utilities.

#![cfg(test)]

use base64::{engine::general_purpose, Engine as _};
use serde_json::json;

/// Generate a valid test JWT token with uuid claim
pub fn generate_test_id_token(uuid: &str) -> String {
    // Header: {"alg":"HS256","typ":"JWT"}
    let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";

    // Payload with uuid
    let payload_json = json!({
        "uuid": uuid,
        "iat": 1516239022,
        "sub": "test-user"
    });
    let payload_str = payload_json.to_string();
    let payload_b64 = general_purpose::URL_SAFE_NO_PAD.encode(payload_str.as_bytes());

    // Signature (mock - not validated in tests)
    let signature = "mock-signature";

    format!("{}.{}.{}", header, payload_b64, signature)
}

/// Mock Toyota API authentication response (Step 1)
pub fn mock_auth_step1_response() -> String {
    json!({
        "authId": "test-auth-id-12345",
        "stage": "DataStore1",
        "callbacks": []
    })
    .to_string()
}

/// Mock Toyota API authentication response with tokenId (Step 2)
pub fn mock_auth_step2_response() -> String {
    json!({
        "tokenId": "test-token-id-67890",
        "successUrl": "/auth/success",
        "realm": "/tme"
    })
    .to_string()
}

/// Mock Toyota API token exchange response
pub fn mock_token_response(uuid: &str) -> String {
    json!({
        "access_token": "mock-toyota-access-token",
        "refresh_token": "mock-toyota-refresh-token",
        "id_token": generate_test_id_token(uuid),
        "token_type": "Bearer",
        "expires_in": 3600
    })
    .to_string()
}

/// Mock electric status response
pub fn mock_electric_status_response(soc: i32, charging: bool) -> String {
    let mut charge_info = json!({
        "chargeRemainingAmount": soc,
        "chargingStatus": if charging { "CHARGING" } else { "NOT_CHARGING" },
        "evRange": 250.5,
        "evRangeWithAc": 230.0
    });

    if charging {
        charge_info["remainingChargeTime"] = json!(120);
    }

    json!({
        "payload": {
            "vehicleInfo": {
                "chargeInfo": charge_info,
                "lastUpdateTimestamp": "2025-01-15T12:00:00Z"
            }
        }
    })
    .to_string()
}

/// Mock location response
pub fn mock_location_response(lat: f64, lon: f64) -> String {
    json!({
        "payload": {
            "vehicleInfo": {
                "location": {
                    "lat": lat,
                    "lon": lon
                },
                "lastUpdateTimestamp": "2025-01-15T12:00:00Z"
            }
        }
    })
    .to_string()
}

/// Mock telemetry response
pub fn mock_telemetry_response(odometer: f64) -> String {
    json!({
        "payload": {
            "vehicleInfo": {
                "odometer": {
                    "value": odometer,
                    "unit": "km"
                },
                "lastUpdateTimestamp": "2025-01-15T12:00:00Z"
            }
        }
    })
    .to_string()
}

/// Mock vehicle list response
pub fn mock_vehicle_list_response() -> String {
    json!({
        "data": [
            {
                "vin": "TEST1234567890ABC",
                "modelName": "bZ4X",
                "modelYear": 2023,
                "nickname": "My EV"
            }
        ]
    })
    .to_string()
}

/// Create test user credentials
pub fn test_credentials() -> (&'static str, &'static str) {
    ("test@example.com", "test-password-123")
}

/// Create invalid credentials for testing
pub fn invalid_credentials() -> (&'static str, &'static str) {
    ("", "") // Empty credentials should fail validation
}

/// Generate a test VIN
pub fn test_vin() -> &'static str {
    "TEST1234567890ABC"
}

/// Create a test timestamp (2025-01-15 12:00:00 UTC)
pub fn test_timestamp() -> i64 {
    1736942400
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_id_token() {
        let token = generate_test_id_token("test-uuid-123");
        assert!(token.contains('.'));
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_mock_responses_are_valid_json() {
        // Verify all mock responses are valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&mock_auth_step1_response()).is_ok());
        assert!(serde_json::from_str::<serde_json::Value>(&mock_auth_step2_response()).is_ok());
        assert!(serde_json::from_str::<serde_json::Value>(&mock_token_response("test")).is_ok());
        assert!(
            serde_json::from_str::<serde_json::Value>(&mock_electric_status_response(80, true))
                .is_ok()
        );
        assert!(
            serde_json::from_str::<serde_json::Value>(&mock_location_response(52.5, 13.4)).is_ok()
        );
        assert!(
            serde_json::from_str::<serde_json::Value>(&mock_telemetry_response(15000.0)).is_ok()
        );
        assert!(serde_json::from_str::<serde_json::Value>(&mock_vehicle_list_response()).is_ok());
    }
}
