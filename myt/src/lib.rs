use serde::{Deserialize, Serialize};
use spin_sdk::http::conversions::IntoBody;
use std::{convert::From, option::Option};

// New OAuth2 Authentication Structures

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticateRequest {
    pub callbacks: Vec<Callback>,
    #[serde(rename = "authId")]
    pub auth_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Callback {
    pub r#type: String,
    pub output: Option<Vec<CallbackOutput>>,
    pub input: Option<Vec<CallbackInput>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallbackOutput {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallbackInput {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticateResponse {
    #[serde(rename = "tokenId")]
    pub token_id: Option<String>,
    pub callbacks: Option<Vec<Callback>>,
    #[serde(rename = "authId")]
    pub auth_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenRequest {
    pub client_id: String,
    pub code: String,
    pub redirect_uri: String,
    pub grant_type: String,
    pub code_verifier: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub expires_in: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CachedToken {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub uuid: String,
    pub expires_at: i64, // Unix timestamp
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshTokenRequest {
    pub grant_type: String,
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JwtPayload {
    pub uuid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomerProfile {
    username: String,
    email: String,
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    #[serde(rename = "languageCode")]
    language_code: String,
    #[serde(rename = "countryCode")]
    country_code: String,
    title: Option<String>,
    pub uuid: String,
    #[serde(rename = "mobileNo")]
    mobile_no: Option<String>,
    dob: Option<String>,
    #[serde(rename = "commPref")]
    comm_pref: CommPref,
    addresses: Vec<Address>,
    #[serde(rename = "myToyotaId")]
    my_toyota_id: String,
    active: bool,
    extras: Extras,
    #[serde(rename = "hotspotActivationStatus")]
    hotspot_activation_status: Option<String>,
    groups: Vec<String>,
    #[serde(rename = "hasUnreadNotifications")]
    has_unread_notifications: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Extras {
    #[serde(rename = "hasPurchasedCars")]
    has_purchased_cars: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Address {
    #[serde(rename = "addressLine1")]
    address_line1: String,
    #[serde(rename = "addressLine2")]
    address_line2: Option<String>,
    country: String,
    city: String,
    postcode: String,
    favourite: bool,
    r#type: String,
    id: i32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CommPref {
    sms: bool,
    tel: bool,
    email: bool,
    post: bool,
    emails: Vec<Email>,
    phones: Vec<Phone>,
    language: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Email {
    email: String,
    preferred: bool,
    primary: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Phone {
    phone: String,
    preferred: bool,
    r#type: String,
    verified: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Authenticate {
    pub username: String,
    pub password: String,
}

// New API Electric Status Structures

#[derive(Serialize, Deserialize, Debug)]
pub struct ElectricStatusResponse {
    pub payload: ElectricStatusPayload,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ElectricStatusPayload {
    #[serde(rename = "vehicleInfo")]
    pub vehicle_info: ElectricVehicleInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ElectricVehicleInfo {
    #[serde(rename = "chargeInfo")]
    pub charge_info: NewChargeInfo,
    #[serde(rename = "lastUpdateTimestamp")]
    pub last_update_timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewChargeInfo {
    #[serde(rename = "chargeRemainingAmount")]
    pub charge_remaining_amount: Option<i32>,
    #[serde(rename = "chargingStatus")]
    pub charging_status: Option<String>,
    #[serde(rename = "evRange")]
    pub ev_range: Option<f32>,
    #[serde(rename = "evRangeWithAc")]
    pub ev_range_with_ac: Option<f32>,
    #[serde(rename = "remainingChargeTime")]
    pub remaining_charge_time: Option<i32>,
}

// Trait implementations for new structures

impl IntoBody for AuthenticateRequest {
    fn into_body(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

impl IntoBody for TokenRequest {
    fn into_body(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

impl IntoBody for RefreshTokenRequest {
    fn into_body(self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

impl AuthenticateRequest {
    pub fn new() -> Self {
        AuthenticateRequest {
            callbacks: vec![],
            auth_id: None,
        }
    }

    pub fn with_credentials(username: String, password: String, auth_id: Option<String>) -> Self {
        AuthenticateRequest {
            callbacks: vec![
                Callback {
                    r#type: "NameCallback".to_string(),
                    output: None,
                    input: Some(vec![CallbackInput {
                        name: "IDToken1".to_string(),
                        value: username,
                    }]),
                },
                Callback {
                    r#type: "PasswordCallback".to_string(),
                    output: None,
                    input: Some(vec![CallbackInput {
                        name: "IDToken2".to_string(),
                        value: password,
                    }]),
                },
            ],
            auth_id,
        }
    }
}

impl From<&[u8]> for AuthenticateResponse {
    fn from(item: &[u8]) -> Self {
        serde_json::from_slice(item).unwrap()
    }
}

impl From<&[u8]> for TokenResponse {
    fn from(item: &[u8]) -> Self {
        serde_json::from_slice(item).unwrap()
    }
}

impl From<&[u8]> for ElectricStatusResponse {
    fn from(item: &[u8]) -> Self {
        let result = String::from_utf8_lossy(item);
        let deserializer = &mut serde_json::Deserializer::from_str(&result);
        let result: Result<ElectricStatusResponse, _> =
            serde_path_to_error::deserialize(deserializer);
        match result {
            Ok(status) => status,
            Err(err) => {
                panic!("Failed to parse electric status: {}", err);
            }
        }
    }
}

impl TokenRequest {
    pub fn new(code: String) -> Self {
        TokenRequest {
            client_id: "oneapp".to_string(),
            code,
            redirect_uri: "com.toyota.oneapp:/oauth2Callback".to_string(),
            grant_type: "authorization_code".to_string(),
            code_verifier: "plain".to_string(),
        }
    }
}

impl RefreshTokenRequest {
    pub fn new(refresh_token: String) -> Self {
        RefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token,
        }
    }
}

impl CachedToken {
    pub fn from_token_response(token_response: TokenResponse, uuid: String, current_time: i64) -> Self {
        CachedToken {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            id_token: token_response.id_token,
            uuid,
            expires_at: current_time + (token_response.expires_in as i64) - 60, // Subtract 60s buffer
        }
    }

    pub fn is_expired(&self, current_time: i64) -> bool {
        current_time >= self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticate_request_new() {
        let req = AuthenticateRequest::new();
        assert!(req.callbacks.is_empty());
        assert!(req.auth_id.is_none());
    }

    #[test]
    fn test_authenticate_request_with_credentials() {
        let req = AuthenticateRequest::with_credentials(
            "test@example.com".to_string(),
            "password123".to_string(),
            Some("auth-id-123".to_string()),
        );

        assert_eq!(req.callbacks.len(), 2);
        assert_eq!(req.callbacks[0].r#type, "NameCallback");
        assert_eq!(req.callbacks[1].r#type, "PasswordCallback");
        assert_eq!(req.auth_id, Some("auth-id-123".to_string()));

        // Verify username callback
        let username_input = &req.callbacks[0].input.as_ref().unwrap()[0];
        assert_eq!(username_input.name, "IDToken1");
        assert_eq!(username_input.value, "test@example.com");

        // Verify password callback
        let password_input = &req.callbacks[1].input.as_ref().unwrap()[0];
        assert_eq!(password_input.name, "IDToken2");
        assert_eq!(password_input.value, "password123");
    }

    #[test]
    fn test_token_request_new() {
        let req = TokenRequest::new("auth-code-123".to_string());
        assert_eq!(req.client_id, "oneapp");
        assert_eq!(req.code, "auth-code-123");
        assert_eq!(req.redirect_uri, "com.toyota.oneapp:/oauth2Callback");
        assert_eq!(req.grant_type, "authorization_code");
        assert_eq!(req.code_verifier, "plain");
    }

    #[test]
    fn test_refresh_token_request_new() {
        let req = RefreshTokenRequest::new("refresh-token-123".to_string());
        assert_eq!(req.grant_type, "refresh_token");
        assert_eq!(req.refresh_token, "refresh-token-123");
    }

    #[test]
    fn test_cached_token_from_token_response() {
        let token_response = TokenResponse {
            access_token: "access-123".to_string(),
            refresh_token: "refresh-456".to_string(),
            id_token: "id-789".to_string(),
            expires_in: 3600,
        };

        let current_time = 1000;
        let uuid = "uuid-test-123".to_string();

        let cached = CachedToken::from_token_response(token_response, uuid.clone(), current_time);

        assert_eq!(cached.access_token, "access-123");
        assert_eq!(cached.refresh_token, "refresh-456");
        assert_eq!(cached.id_token, "id-789");
        assert_eq!(cached.uuid, uuid);
        assert_eq!(cached.expires_at, current_time + 3600 - 60); // With 60s buffer
    }

    #[test]
    fn test_cached_token_is_expired() {
        let token_response = TokenResponse {
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            id_token: "id".to_string(),
            expires_in: 3600,
        };

        let current_time = 1000;
        let cached = CachedToken::from_token_response(
            token_response,
            "uuid".to_string(),
            current_time,
        );

        // Not expired right away
        assert!(!cached.is_expired(current_time));
        assert!(!cached.is_expired(current_time + 100));
        assert!(!cached.is_expired(current_time + 3000));

        // Expired at expiry time (with buffer)
        assert!(cached.is_expired(current_time + 3540));
        assert!(cached.is_expired(current_time + 4000));
    }

    #[test]
    fn test_electric_status_response_structure() {
        let json = r#"{
            "payload": {
                "vehicleInfo": {
                    "chargeInfo": {
                        "chargeRemainingAmount": 85,
                        "chargingStatus": "CHARGING",
                        "evRange": 250.5,
                        "evRangeWithAc": 230.0,
                        "remainingChargeTime": 120
                    },
                    "lastUpdateTimestamp": "2025-01-01T12:00:00Z"
                }
            }
        }"#;

        let result: Result<ElectricStatusResponse, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.payload.vehicle_info.charge_info.charge_remaining_amount, Some(85));
        assert_eq!(response.payload.vehicle_info.charge_info.charging_status, Some("CHARGING".to_string()));
        assert_eq!(response.payload.vehicle_info.last_update_timestamp, "2025-01-01T12:00:00Z");
    }

    #[test]
    fn test_electric_status_optional_fields() {
        let json = r#"{
            "payload": {
                "vehicleInfo": {
                    "chargeInfo": {},
                    "lastUpdateTimestamp": "2025-01-01T12:00:00Z"
                }
            }
        }"#;

        let result: Result<ElectricStatusResponse, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.payload.vehicle_info.charge_info.charge_remaining_amount, None);
        assert_eq!(response.payload.vehicle_info.charge_info.charging_status, None);
    }

    #[test]
    fn test_token_response_serialization() {
        let token = TokenResponse {
            access_token: "access-123".to_string(),
            refresh_token: "refresh-456".to_string(),
            id_token: "id-789".to_string(),
            expires_in: 3600,
        };

        let json = serde_json::to_string(&token).unwrap();
        let deserialized: TokenResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.access_token, token.access_token);
        assert_eq!(deserialized.refresh_token, token.refresh_token);
        assert_eq!(deserialized.id_token, token.id_token);
        assert_eq!(deserialized.expires_in, token.expires_in);
    }
}
