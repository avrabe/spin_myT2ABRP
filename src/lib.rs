use anyhow::Result;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use spin_sdk::{
    config,
    http::{Request, Response},
    http_component,
};
use tracing::{debug, info};

#[derive(Serialize, Deserialize, Debug)]

struct AuthenticateResult {
    token: String,
    #[serde(rename = "customerProfile")]
    customer_profile: CustomerProfile,
}

#[derive(Serialize, Deserialize, Debug)]
struct CustomerProfile {
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
    uuid: String,
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
struct Extras {
    #[serde(rename = "hasPurchasedCars")]
    has_purchased_cars: bool,
}
#[derive(Serialize, Deserialize, Debug)]
struct Address {
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
struct CommPref {
    sms: bool,
    tel: bool,
    email: bool,
    post: bool,
    emails: Vec<Email>,
    phones: Vec<Phone>,
    language: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct Email {
    email: String,
    preferred: bool,
    primary: bool,
}
#[derive(Serialize, Deserialize, Debug)]
struct Phone {
    phone: String,
    preferred: bool,
    r#type: String,
    verified: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Authenticate {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]

struct VehicleInfo {
    #[serde(rename = "AcquisitionDatetime")]
    acquisition_datetime: String,
    #[serde(rename = "RemoteHvacInfo")]
    remote_hvac_info: RemoteHvacInfo,
    #[serde(rename = "ChargeInfo")]
    charge_info: ChargeInfo,
}
#[derive(Serialize, Deserialize, Debug)]

struct RemoteHvacInfo {
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

struct ChargeInfo {
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
    charge_remaining_amount: i32,
    #[serde(rename = "SettingChangeAcceptanceStatus")]
    setting_change_acceptance_status: i32,
    #[serde(rename = "ChargeType")]
    charge_type: i32,
}
#[derive(Serialize, Deserialize, Debug)]
struct RemoteControlStatus {
    #[serde(rename = "VehicleInfo")]
    vehicle_info: VehicleInfo,
    #[serde(rename = "ReturnCode")]
    return_code: String,
}

/// Send an HTTP request and return the response.
#[http_component]
fn send_outbound(_req: Request) -> Result<Response> {
    let address = Authenticate {
        username: config::get("username").expect("could not get variable username"),
        password: config::get("password").expect("could not get variable password"),
    };
    let string_address = serde_json::to_string(&address).unwrap();
    let bytes_address = Some(Bytes::from(string_address));
    let request = http::Request::builder()
        .method("POST")
        .uri("https://ssoms.toyota-europe.com/authenticate")
        .header("content-type", "application/json")
        .header("X-TME-BRAND", "TOYOTA")
        .header("X-TME-LC", "de-de")
        .header("Sec-Fetch-Dest", "empty")
        .header("Accept", "application/json, text/plain, */*")
        .body(bytes_address)?;
    //println!("{:?}", request);
    let result = spin_sdk::outbound_http::send_request(request).unwrap();
    if let Some(body) = &result.body().clone() {
        let body = String::from_utf8(body.to_vec()).unwrap();
        debug!("Body: {:?}", body);
        let result: AuthenticateResult = serde_json::from_str(&body).unwrap();
        info!("Struct: {:?}", result);

        let status = format!(
            "https://myt-agg.toyota-europe.com/cma/api/vehicles/{}/remoteControl/status",
            config::get("vin").expect("could not get variable vin")
        );
        let cookie: String = format!("iPlanetDirectoryPro={}", result.token);
        let request: Request = http::Request::builder()
            .method("GET")
            .uri(status)
            .header("content-type", "application/json")
            .header("X-TME-APP-VERSION", "4.10.0")
            .header("X-TME-BRAND", "TOYOTA")
            .header("X-TME-LOCALE", "de-de")
            .header("Accept", "application/json, text/plain, */*")
            .header("Cookie", cookie)
            .header("UUID", result.customer_profile.uuid)
            .body(None)?;
        let result = spin_sdk::outbound_http::send_request(request).unwrap();
        if let Some(body) = &result.body().clone() {
            let body = String::from_utf8(body.to_vec()).unwrap();
            debug!("Body: {:?}", body);
            let remote_control_status: RemoteControlStatus = serde_json::from_str(&body).unwrap();
            println!("SOC: {:?}", remote_control_status.vehicle_info.charge_info.charge_remaining_amount);
        }
        //println!("{:?}", result);
        let mut res = result;
        res.headers_mut()
            .insert("spin-component", "rust-outbound-http".try_into()?);
        debug!("{:?}", res);
        Ok(res)
    } else {
        let res = Response::new(None);
        Ok(res)
    }
}
