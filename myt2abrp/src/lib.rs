use anyhow::Result;
use bytes::Bytes;
use myt::{Authenticate, AuthenticateResult, RemoteControlStatus};
use spin_sdk::{
    config,
    http::{Request, Response},
    http_component,
};
use tracing::{debug, info};

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
            println!(
                "SOC: {:?}",
                remote_control_status
                    .vehicle_info
                    .charge_info
                    .charge_remaining_amount
            );
        }
        //println!("{:?}", result);
        //let mut res: http::Response<Option<Bytes>> = result;
        let res = Response::new(Some(Bytes::from("foo")));
        debug!("{:?}", res);
        Ok(res)
    } else {
        let res = Response::new(Some(Bytes::from("bar")));
        Ok(res)
    }
}
