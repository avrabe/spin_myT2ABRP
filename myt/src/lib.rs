use bytes::Bytes;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{convert::From, option::Option};
#[derive(Serialize, Deserialize, Debug)]

pub struct AuthenticateResult {
    pub token: String,
    #[serde(rename = "customerProfile")]
    pub customer_profile: CustomerProfile,
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
    title: String,
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
    address_line2: String,
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
    verified: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Authenticate {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct VehicleInfo {
    #[serde(rename = "AcquisitionDatetime")]
    pub acquisition_datetime: String,
    #[serde(rename = "RemoteHvacInfo")]
    remote_hvac_info: RemoteHvacInfo,
    #[serde(rename = "ChargeInfo")]
    pub charge_info: ChargeInfo,
}
#[derive(Serialize, Deserialize, Debug)]

pub struct RemoteHvacInfo {
    #[serde(rename = "Temperaturelevel")]
    temperature_level: i32,
    #[serde(rename = "SettingTemperature")]
    setting_temperature: f32,
    #[serde(rename = "BlowerStatus")]
    blower_status: i32,
    #[serde(rename = "FrontDefoggerStatus")]
    front_defogger_status: i32,
    #[serde(rename = "RearDefoggerStatus")]
    rear_defogger_status: i32,
    #[serde(rename = "RemoteHvacMode")]
    remote_hvac_mode: i32,
    #[serde(rename = "RemoteHvacProhibitionSignal")]
    remote_hvac_prohibition_signal: i32,
    #[serde(rename = "InsideTemperature")]
    inside_temperature: i32,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct ChargeInfo {
    #[serde(rename = "EvDistanceInKm")]
    ev_distance_in_km: f32,
    #[serde(rename = "GasolineTravelableDistanceUnit")]
    gasoline_travelable_distance_unit: i32,
    #[serde(rename = "GasolineTravelableDistance")]
    gasoline_travelable_distance: i32,
    #[serde(rename = "ChargeWeek")]
    charge_week: i32,
    #[serde(rename = "ChargeStartTime")]
    charge_start_time: String,
    #[serde(rename = "ChargeEndTime")]
    charge_end_time: String,
    #[serde(rename = "ConnectorStatus")]
    connector_status: i32,
    #[serde(rename = "BatteryPowerSupplyPossibleTime")]
    battery_power_supply_possible_time: i32,
    #[serde(rename = "ChargingStatus")]
    charging_status: String,
    #[serde(rename = "EvDistanceWithAirCoInKm")]
    ev_distance_with_air_co_in_km: f32,
    #[serde(rename = "PlugStatus")]
    plug_status: i32,
    #[serde(rename = "PlugInHistory")]
    plug_in_history: i32,
    #[serde(rename = "RemainingChargeTime")]
    remaining_charge_time: i32,
    #[serde(rename = "EvTravelableDistance")]
    ev_travelable_distance: f32,
    #[serde(rename = "EvTravelableDistanceSubtractionRate")]
    ev_travelable_distance_subtraction_rate: i32,
    #[serde(rename = "ChargeRemainingAmount")]
    pub charge_remaining_amount: i32,
    #[serde(rename = "SettingChangeAcceptanceStatus")]
    setting_change_acceptance_status: i32,
    #[serde(rename = "ChargeType")]
    charge_type: i32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteControlStatus {
    #[serde(rename = "VehicleInfo")]
    pub vehicle_info: Option<VehicleInfo>,
    #[serde(rename = "ReturnCode")]
    return_code: String,
}

impl Authenticate {
    pub fn new(username: String, password: String) -> Authenticate {
        Authenticate { username, password }
    }
}

impl From<Authenticate> for Option<Bytes> {
    fn from(item: Authenticate) -> Self {
        let string_address = serde_json::to_string(&item).unwrap();
        Some(Bytes::from(string_address))
    }
}

impl From<&Option<Bytes>> for AuthenticateResult {
    fn from(item: &Option<Bytes>) -> Self {
        from::<AuthenticateResult>(item)
    }
}

impl From<&Option<Bytes>> for RemoteControlStatus {
    fn from(item: &Option<Bytes>) -> Self {
        from::<RemoteControlStatus>(item)
    }
}

fn from<D>(item: &Option<Bytes>) -> D
where
    D: DeserializeOwned,
{
    let result = item.clone().unwrap();
    let result = String::from_utf8(result.to_vec()).unwrap();
    let result: D = serde_json::from_str(&result).unwrap();
    result
}

#[test]
fn test_remote_control_status_from() {
    let json = r#"{"ReturnCode": "OK"}"#;
    let bytes = Bytes::from(json);

    let status: RemoteControlStatus = RemoteControlStatus::from(&Some(bytes));
    print!("{:?}", status);
    assert_eq!("OK", status.return_code);
    // Add assertions to verify VehicleInfo fields...
}
