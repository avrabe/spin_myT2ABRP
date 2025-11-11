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

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub expires_in: u32,
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
