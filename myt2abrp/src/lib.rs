use bytes::Bytes;
use futures::SinkExt;
use myt::{Authenticate, AuthenticateResult, RemoteControlStatus};
use serde::{Deserialize, Serialize};
use spin_sdk::{
    http::{
        send, Fields, IncomingRequest, Method, OutgoingResponse, Request, Response,
        ResponseOutparam,
    },
    http_component, variables,
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
async fn handle_request(_request: IncomingRequest, response_out: ResponseOutparam) {
    let username = variables::get("username").expect("could not get variable username");
    let password = variables::get("password").expect("could not get variable password");
    let address = Authenticate::new(username, password);
    // Send the request and await the response
    let request = Request::builder()
        .method(Method::Post)
        .header("content-type", "application/json")
        .header("X-TME-BRAND", "TOYOTA")
        .header("X-TME-LC", "de-de")
        .header("Sec-Fetch-Dest", "empty")
        .header("Accept", "application/json, text/plain, */*")
        .body(address)
        .uri("https://ssoms.toyota-europe.com/authenticate")
        .build();

    //println!("{:?}", request);
    let result: Result<Response, spin_sdk::http::SendError> = send(request).await;
    match result {
        Ok(ref _result) => {
            //println!("{:?}", &result);
            //println!("{:?}", String::from_utf8_lossy(&result.body()));
        }
        Err(ref error) => {
            println!("{:?}", &error);
        }
    }
    let result: Response = result.unwrap();
    let status = result.status();
    if *status != 200 {
        println!("Authentication failed");
        let response = OutgoingResponse::new(Fields::new());
        response.set_status_code(405).unwrap();

        response_out.set(response);
        return;
    } else {
        println!("Authentication successful");
    }
    let result: AuthenticateResult = result.body().into();

    let status = format!(
        "https://myt-agg.toyota-europe.com/cma/api/vehicles/{}/remoteControl/status",
        variables::get("vin").expect("could not get variable vin")
    );
    let cookie: String = format!("iPlanetDirectoryPro={}", result.token);
    let request = Request::builder()
        .method(Method::Get)
        .header("content-type", "application/json")
        .header("X-TME-APP-VERSION", "4.10.0")
        .header("X-TME-BRAND", "TOYOTA")
        .header("X-TME-LOCALE", "de-de")
        .header("Accept", "application/json, text/plain, */*")
        .header("Cookie", cookie)
        .header("UUID", result.customer_profile.uuid)
        .uri(status)
        .build();

    let result: Response = send(request).await.unwrap();
    let remote_control_status: RemoteControlStatus = result.body().into();
    let return_value = CurrentStatus::new(
        remote_control_status
            .vehicle_info
            .charge_info
            .charge_remaining_amount,
        remote_control_status.vehicle_info.acquisition_datetime,
    );
    let return_value = serde_json::to_string(&return_value).unwrap();
    println!("{}", return_value);
    //println!("{:?}", result);
    //let mut res: http::Response<Option<Bytes>> = result;
    let fields =
        Fields::from_list(&[("content-type".to_owned(), b"application/json".to_vec())]).unwrap();

    let response = OutgoingResponse::new(fields);
    let _ = response.set_status_code(200);

    let mut body = response.take_body();

    response_out.set(response);
    let b = Bytes::from(return_value).to_vec();

    if let Err(e) = body.send(b).await {
        eprintln!("Error sending payload: {e}");
    }
}
