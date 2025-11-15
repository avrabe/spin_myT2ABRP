// Toyota to ABRP Data Transformation
//
// Pure business logic for converting Toyota Connected Services vehicle data
// to A Better Route Planner (ABRP) telemetry format. Zero dependencies on
// Spin SDK or any platform-specific APIs.

#[allow(warnings)]
mod bindings;

use bindings::exports::toyota::data_transform::converter::{AbrpTelemetry, Guest};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// =============================================================================
// Internal Data Structures (for JSON parsing)
// =============================================================================

#[derive(Serialize, Deserialize, Debug)]
struct ElectricStatusResponse {
    payload: ElectricStatusPayload,
}

#[derive(Serialize, Deserialize, Debug)]
struct ElectricStatusPayload {
    #[serde(rename = "vehicleInfo")]
    vehicle_info: ElectricVehicleInfo,
}

#[derive(Serialize, Deserialize, Debug)]
struct ElectricVehicleInfo {
    #[serde(rename = "chargeInfo")]
    charge_info: ChargeInfo,
    #[serde(rename = "lastUpdateTimestamp")]
    last_update_timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChargeInfo {
    #[serde(rename = "chargeRemainingAmount")]
    charge_remaining_amount: Option<i32>,
    #[serde(rename = "chargingStatus")]
    charging_status: Option<String>,
    #[serde(rename = "evRange")]
    ev_range: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LocationResponse {
    payload: LocationPayload,
}

#[derive(Serialize, Deserialize, Debug)]
struct LocationPayload {
    #[serde(rename = "vehicleInfo")]
    vehicle_info: LocationVehicleInfo,
}

#[derive(Serialize, Deserialize, Debug)]
struct LocationVehicleInfo {
    location: LocationData,
}

#[derive(Serialize, Deserialize, Debug)]
struct LocationData {
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TelemetryResponse {
    payload: TelemetryPayload,
}

#[derive(Serialize, Deserialize, Debug)]
struct TelemetryPayload {
    #[serde(rename = "vehicleInfo")]
    vehicle_info: TelemetryVehicleInfo,
}

#[derive(Serialize, Deserialize, Debug)]
struct TelemetryVehicleInfo {
    odometer: Option<TelemetryOdometer>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TelemetryOdometer {
    value: Option<f64>,
}

// =============================================================================
// Transformation Functions
// =============================================================================

/// Parse ISO 8601 timestamp to Unix seconds
fn parse_iso8601_timestamp_internal(iso_string: &str) -> i64 {
    // Try to parse ISO 8601 timestamp
    if let Ok(dt) = DateTime::parse_from_rfc3339(iso_string) {
        return dt.timestamp();
    }
    // If parsing fails, return current time
    Utc::now().timestamp()
}

/// Determine if vehicle is charging from status string
fn is_charging_status_internal(status: &str) -> bool {
    let status_upper = status.to_uppercase();
    // Explicitly handle negative statuses
    if status_upper.contains("NOT") || status_upper.contains("DISCONNECT") {
        return false;
    }
    // Positive statuses
    status_upper.contains("CHARGING") || status_upper.contains("CONNECTED")
}

/// Convert Toyota electric status to ABRP telemetry
fn toyota_to_abrp_internal(
    electric_status_json: &str,
    location_json: Option<&str>,
    telemetry_json: Option<&str>,
    version: &str,
) -> Result<AbrpTelemetryInternal, String> {
    // Parse electric status (required)
    let electric_status: ElectricStatusResponse = serde_json::from_str(electric_status_json)
        .map_err(|e| format!("Failed to parse electric status: {}", e))?;

    // Parse location (optional)
    let location: Option<LocationResponse> =
        location_json.and_then(|json| serde_json::from_str(json).ok());

    // Parse telemetry (optional)
    let telemetry: Option<TelemetryResponse> =
        telemetry_json.and_then(|json| serde_json::from_str(json).ok());

    // Extract charge info
    let charge_info = &electric_status.payload.vehicle_info.charge_info;
    let soc = charge_info.charge_remaining_amount.unwrap_or(0) as f64;
    let utc = parse_iso8601_timestamp_internal(
        &electric_status.payload.vehicle_info.last_update_timestamp,
    );

    // Determine if charging
    let is_charging = charge_info
        .charging_status
        .as_ref()
        .map(|status| is_charging_status_internal(status));

    // Build ABRP telemetry
    Ok(AbrpTelemetryInternal {
        utc,
        soc,
        lat: location
            .as_ref()
            .map(|l| l.payload.vehicle_info.location.lat),
        lon: location
            .as_ref()
            .map(|l| l.payload.vehicle_info.location.lon),
        is_charging,
        is_parked: None, // Not available from Toyota API
        odometer: telemetry
            .as_ref()
            .and_then(|t| t.payload.vehicle_info.odometer.as_ref())
            .and_then(|o| o.value),
        est_battery_range: charge_info.ev_range.map(|r| r as f64),
        version: version.to_string(),
    })
}

// =============================================================================
// Internal ABRP Telemetry (for serialization)
// =============================================================================

#[derive(Serialize, Deserialize, Debug)]
struct AbrpTelemetryInternal {
    utc: i64,
    soc: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lon: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_charging: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_parked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    odometer: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    est_battery_range: Option<f64>,
    version: String,
}

impl From<AbrpTelemetryInternal> for AbrpTelemetry {
    fn from(internal: AbrpTelemetryInternal) -> Self {
        AbrpTelemetry {
            utc: internal.utc,
            soc: internal.soc,
            lat: internal.lat,
            lon: internal.lon,
            is_charging: internal.is_charging,
            is_parked: internal.is_parked,
            odometer: internal.odometer,
            est_battery_range: internal.est_battery_range,
            version: internal.version,
        }
    }
}

impl From<AbrpTelemetry> for AbrpTelemetryInternal {
    fn from(wit: AbrpTelemetry) -> Self {
        AbrpTelemetryInternal {
            utc: wit.utc,
            soc: wit.soc,
            lat: wit.lat,
            lon: wit.lon,
            is_charging: wit.is_charging,
            is_parked: wit.is_parked,
            odometer: wit.odometer,
            est_battery_range: wit.est_battery_range,
            version: wit.version,
        }
    }
}

// =============================================================================
// WIT Interface Implementation
// =============================================================================

struct Component;

impl Guest for Component {
    fn parse_iso8601_timestamp(iso_string: String) -> i64 {
        parse_iso8601_timestamp_internal(&iso_string)
    }

    fn is_charging_status(status: String) -> bool {
        is_charging_status_internal(&status)
    }

    fn toyota_to_abrp(
        electric_status_json: String,
        location_json: Option<String>,
        telemetry_json: Option<String>,
        version: String,
    ) -> Result<AbrpTelemetry, String> {
        let internal = toyota_to_abrp_internal(
            &electric_status_json,
            location_json.as_deref(),
            telemetry_json.as_deref(),
            &version,
        )?;
        Ok(internal.into())
    }

    fn serialize_abrp_telemetry(telemetry: AbrpTelemetry) -> String {
        let internal: AbrpTelemetryInternal = telemetry.into();
        serde_json::to_string(&internal).unwrap_or_default()
    }

    fn deserialize_abrp_telemetry(json: String) -> Result<AbrpTelemetry, String> {
        let internal: AbrpTelemetryInternal = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize ABRP telemetry: {}", e))?;
        Ok(internal.into())
    }
}

bindings::export!(Component with_types_in bindings);

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iso8601_timestamp() {
        let timestamp = parse_iso8601_timestamp_internal("2025-01-15T12:00:00Z");
        assert!(timestamp > 0);
        assert_eq!(timestamp, 1736942400); // 2025-01-15 12:00:00 UTC

        let timestamp2 = parse_iso8601_timestamp_internal("2025-01-15T12:00:00+00:00");
        assert_eq!(timestamp, timestamp2);

        // Invalid timestamp should return current time
        let timestamp3 = parse_iso8601_timestamp_internal("invalid-date");
        assert!(timestamp3 > 0);
    }

    #[test]
    fn test_is_charging_status() {
        assert!(is_charging_status_internal("CHARGING"));
        assert!(is_charging_status_internal("charging"));
        assert!(is_charging_status_internal("CONNECTED"));
        assert!(is_charging_status_internal("connected"));
        assert!(!is_charging_status_internal("NOT_CHARGING"));
        assert!(!is_charging_status_internal("DISCONNECTED"));
    }

    #[test]
    fn test_toyota_to_abrp_basic() {
        let electric_status = r#"{
            "payload": {
                "vehicleInfo": {
                    "chargeInfo": {
                        "chargeRemainingAmount": 85,
                        "chargingStatus": "CHARGING",
                        "evRange": 250.5
                    },
                    "lastUpdateTimestamp": "2025-01-15T12:00:00Z"
                }
            }
        }"#;

        let result = toyota_to_abrp_internal(electric_status, None, None, "1.0.0");
        assert!(result.is_ok());

        let abrp = result.unwrap();
        assert_eq!(abrp.soc, 85.0);
        assert_eq!(abrp.utc, 1736942400);
        assert_eq!(abrp.is_charging, Some(true));
        assert_eq!(abrp.est_battery_range, Some(250.5));
        assert_eq!(abrp.version, "1.0.0");
        assert!(abrp.lat.is_none());
        assert!(abrp.lon.is_none());
        assert!(abrp.odometer.is_none());
    }

    #[test]
    fn test_toyota_to_abrp_with_location() {
        let electric_status = r#"{
            "payload": {
                "vehicleInfo": {
                    "chargeInfo": {
                        "chargeRemainingAmount": 75,
                        "chargingStatus": "NOT_CHARGING",
                        "evRange": 200.0
                    },
                    "lastUpdateTimestamp": "2025-01-15T14:30:00Z"
                }
            }
        }"#;

        let location = r#"{
            "payload": {
                "vehicleInfo": {
                    "location": {
                        "lat": 52.520008,
                        "lon": 13.404954
                    }
                }
            }
        }"#;

        let result = toyota_to_abrp_internal(electric_status, Some(location), None, "1.0.0");
        assert!(result.is_ok());

        let abrp = result.unwrap();
        assert_eq!(abrp.soc, 75.0);
        assert_eq!(abrp.is_charging, Some(false));
        assert_eq!(abrp.lat, Some(52.520008));
        assert_eq!(abrp.lon, Some(13.404954));
    }

    #[test]
    fn test_toyota_to_abrp_with_telemetry() {
        let electric_status = r#"{
            "payload": {
                "vehicleInfo": {
                    "chargeInfo": {
                        "chargeRemainingAmount": 90,
                        "chargingStatus": "CONNECTED",
                        "evRange": 280.0
                    },
                    "lastUpdateTimestamp": "2025-01-15T16:00:00Z"
                }
            }
        }"#;

        let telemetry = r#"{
            "payload": {
                "vehicleInfo": {
                    "odometer": {
                        "value": 15432.5
                    }
                }
            }
        }"#;

        let result = toyota_to_abrp_internal(electric_status, None, Some(telemetry), "1.0.0");
        assert!(result.is_ok());

        let abrp = result.unwrap();
        assert_eq!(abrp.soc, 90.0);
        assert_eq!(abrp.is_charging, Some(true)); // CONNECTED is treated as charging
        assert_eq!(abrp.odometer, Some(15432.5));
        assert_eq!(abrp.est_battery_range, Some(280.0));
    }

    #[test]
    fn test_toyota_to_abrp_complete() {
        let electric_status = r#"{
            "payload": {
                "vehicleInfo": {
                    "chargeInfo": {
                        "chargeRemainingAmount": 80,
                        "chargingStatus": "CHARGING",
                        "evRange": 240.0
                    },
                    "lastUpdateTimestamp": "2025-01-15T18:00:00Z"
                }
            }
        }"#;

        let location = r#"{
            "payload": {
                "vehicleInfo": {
                    "location": {
                        "lat": 48.8566,
                        "lon": 2.3522
                    }
                }
            }
        }"#;

        let telemetry = r#"{
            "payload": {
                "vehicleInfo": {
                    "odometer": {
                        "value": 25000.0
                    }
                }
            }
        }"#;

        let result =
            toyota_to_abrp_internal(electric_status, Some(location), Some(telemetry), "1.0.0");
        assert!(result.is_ok());

        let abrp = result.unwrap();
        assert_eq!(abrp.soc, 80.0);
        assert_eq!(abrp.is_charging, Some(true));
        assert_eq!(abrp.lat, Some(48.8566));
        assert_eq!(abrp.lon, Some(2.3522));
        assert_eq!(abrp.odometer, Some(25000.0));
        assert_eq!(abrp.est_battery_range, Some(240.0));
        assert_eq!(abrp.version, "1.0.0");
    }

    #[test]
    fn test_abrp_telemetry_serialization() {
        let abrp = AbrpTelemetryInternal {
            utc: 1736942400,
            soc: 85.0,
            lat: Some(52.520008),
            lon: Some(13.404954),
            is_charging: Some(true),
            is_parked: None,
            odometer: Some(15000.0),
            est_battery_range: Some(250.5),
            version: "1.0.0".to_string(),
        };

        let json = serde_json::to_string(&abrp).unwrap();
        assert!(json.contains("\"utc\":1736942400"));
        assert!(json.contains("\"soc\":85"));
        assert!(json.contains("\"lat\":52.520008"));
        assert!(json.contains("\"is_charging\":true"));
        assert!(!json.contains("is_parked")); // Should be skipped

        let deserialized: AbrpTelemetryInternal = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.utc, abrp.utc);
        assert_eq!(deserialized.soc, abrp.soc);
        assert_eq!(deserialized.lat, abrp.lat);
        assert_eq!(deserialized.is_charging, abrp.is_charging);
    }

    #[test]
    fn test_toyota_to_abrp_invalid_json() {
        let result = toyota_to_abrp_internal("invalid json", None, None, "1.0.0");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Failed to parse electric status"));
    }
}
