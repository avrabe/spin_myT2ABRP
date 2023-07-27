use anyhow::Result;
use bytes::Bytes;
use myt::{Authenticate, AuthenticateResult, RemoteControlStatus};
use serde::{Deserialize, Serialize};
use spin_sdk::{
    config,
    http::{Request, Response},
    http_component,
};

#[derive(Serialize, Deserialize, Debug)]
struct CurrentStatus {
    pub soc: i32,
    pub access_date: String,
}

impl CurrentStatus {
    pub fn new(soc: i32, access_date: String) -> CurrentStatus {
        CurrentStatus { soc, access_date }
    }
}

/// Send an HTTP request and return the response.
#[http_component]
fn send_outbound(_req: Request) -> Result<Response> {
    let username = config::get("username").expect("could not get variable username");
    let password = config::get("password").expect("could not get variable password");
    let address = Authenticate::new(username, password);
    let request = http::Request::builder()
        .method("POST")
        .uri("https://ssoms.toyota-europe.com/authenticate")
        .header("content-type", "application/json")
        .header("X-TME-BRAND", "TOYOTA")
        .header("X-TME-LC", "de-de")
        .header("Sec-Fetch-Dest", "empty")
        .header("Accept", "application/json, text/plain, */*")
        .body(address.into())?;
    //println!("{:?}", request);
    let result = spin_sdk::outbound_http::send_request(request).unwrap();
    if result.status().as_u16() != 200 {
        println!("Authentication failed");
        return Ok(Response::new(None));
    } else {
        println!("Authentication successful");
    }
    let result: AuthenticateResult = result.body().into();

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
    let remote_control_status: RemoteControlStatus = result.body().into();
    if let Some(vehicle_info) = remote_control_status.vehicle_info {
        let return_value = CurrentStatus::new(
            vehicle_info.charge_info.charge_remaining_amount,
            vehicle_info.acquisition_datetime,
        );
        let return_value = serde_json::to_string(&return_value).unwrap();
        println!("{}", return_value);
        //println!("{:?}", result);
        //let mut res: http::Response<Option<Bytes>> = result;
        let res = Response::new(Some(Bytes::from(return_value)));
        Ok(res)
    } else {
        Err(anyhow::anyhow!("No vehicle info found"))
    }
}
