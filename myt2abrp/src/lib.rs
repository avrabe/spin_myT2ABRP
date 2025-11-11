use base64::{engine::general_purpose, Engine as _};
use myt::{
    AuthenticateRequest, AuthenticateResponse, ElectricStatusResponse, TokenRequest,
    TokenResponse,
};
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IncomingRequest, IntoResponse, Request, Response};
use spin_sdk::{http_component, variables};
use std::time::{SystemTime, UNIX_EPOCH};

const AUTH_URL: &str = "https://b2c-login.toyota-europe.com/json/realms/root/realms/tme/authenticate?authIndexType=service&authIndexValue=oneapp";
const AUTHORIZE_URL: &str = "https://b2c-login.toyota-europe.com/oauth2/realms/root/realms/tme/authorize?client_id=oneapp&scope=openid+profile+write&response_type=code&redirect_uri=com.toyota.oneapp:/oauth2Callback&code_challenge=plain&code_challenge_method=plain";
const TOKEN_URL: &str =
    "https://b2c-login.toyota-europe.com/oauth2/realms/root/realms/tme/access_token";
const API_BASE: &str = "https://ctpa-oneapi.tceu-ctp-prd.toyotaconnectedeurope.io";

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

fn get_timestamp_ms() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

fn generate_uuid() -> String {
    use std::fmt::Write;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut s = String::with_capacity(36);
    let _ = write!(
        &mut s,
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (timestamp >> 64) as u32,
        ((timestamp >> 48) & 0xffff) as u16,
        ((timestamp >> 32) & 0xffff) as u16,
        ((timestamp >> 16) & 0xffff) as u16,
        (timestamp & 0xffffffffffff) as u64
    );
    s
}

fn decode_jwt_uuid(id_token: &str) -> anyhow::Result<String> {
    // Split JWT into parts
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid JWT format");
    }

    // Decode base64 payload (second part)
    let payload_b64 = parts[1];
    // Add padding if needed
    let padding = match payload_b64.len() % 4 {
        2 => "==",
        3 => "=",
        _ => "",
    };
    let payload_padded = format!("{}{}", payload_b64, padding);

    // Decode base64
    let decoded = general_purpose::URL_SAFE_NO_PAD
        .decode(payload_padded.as_bytes())?;

    let payload_str = String::from_utf8(decoded)?;

    // Parse JSON to get uuid
    let payload: serde_json::Value = serde_json::from_str(&payload_str)?;

    payload["uuid"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No uuid in JWT"))
        .map(|s| s.to_string())
}

/// Send an HTTP request and return the response.
#[http_component]
async fn handle_request(_request: IncomingRequest) -> Result<impl IntoResponse, anyhow::Error> {
    let username = variables::get("username").expect("could not get variable username");
    let password = variables::get("password").expect("could not get variable password");
    let vin = variables::get("vin").expect("could not get variable vin");

    // Step 1: Initial authentication request
    println!("Step 1: Starting authentication...");
    let auth_request = AuthenticateRequest::new();
    let mut request = Request::post(AUTH_URL, auth_request)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .build();

    let response: Response = spin_sdk::http::send(request).await?;
    let auth_response: AuthenticateResponse = response.body().as_ref().into();

    // Step 2: Submit credentials
    println!("Step 2: Submitting credentials...");
    let auth_request_with_creds =
        AuthenticateRequest::with_credentials(username, password, auth_response.auth_id);
    request = Request::post(AUTH_URL, auth_request_with_creds)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .build();

    let response: Response = spin_sdk::http::send(request).await?;

    if *response.status() != 200 {
        println!("Authentication failed with status: {}", response.status());
        return Ok(Response::new(401, "Authentication failed"));
    }

    let auth_response: AuthenticateResponse = response.body().as_ref().into();
    let token_id = auth_response
        .token_id
        .ok_or_else(|| anyhow::anyhow!("No tokenId received"))?;

    println!("Authentication successful, got tokenId");

    // Step 3: Get authorization code
    println!("Step 3: Getting authorization code...");
    let cookie = format!("iPlanetDirectoryPro={}", token_id);
    request = Request::get(AUTHORIZE_URL)
        .header("cookie", cookie)
        .build();

    let response: Response = spin_sdk::http::send(request).await?;

    // Extract code from Location header redirect
    let location = response
        .headers()
        .find(|(k, _)| k.to_lowercase() == "location")
        .map(|(_, v)| v)
        .ok_or_else(|| anyhow::anyhow!("No location header in authorization response"))?;

    let location_str = String::from_utf8_lossy(location.as_bytes());
    let code = location_str
        .split("code=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .ok_or_else(|| anyhow::anyhow!("No code in redirect URL"))?
        .to_string();

    println!("Got authorization code");

    // Step 4: Exchange code for tokens
    println!("Step 4: Exchanging code for access token...");
    let token_request = TokenRequest::new(code);
    let token_body = format!(
        "client_id={}&code={}&redirect_uri={}&grant_type={}&code_verifier={}",
        token_request.client_id,
        token_request.code,
        token_request.redirect_uri,
        token_request.grant_type,
        token_request.code_verifier
    );
    request = Request::post(TOKEN_URL, token_body)
        .header("content-type", "application/x-www-form-urlencoded")
        .header("authorization", "Basic b25lYXBwOm9uZWFwcA==")
        .build();

    let response: Response = spin_sdk::http::send(request).await?;

    if *response.status() != 200 {
        println!("Token exchange failed with status: {}", response.status());
        return Ok(Response::new(401, "Token exchange failed"));
    }

    let token_response: TokenResponse = response.body().as_ref().into();
    println!("Got access token");

    // Decode UUID from JWT
    let uuid = decode_jwt_uuid(&token_response.id_token)?;
    println!("Decoded UUID: {}", uuid);

    // Step 5: Get vehicle electric status
    println!("Step 5: Getting vehicle electric status...");
    let status_url = format!(
        "{}/v1/global/remote/electric/status?vin={}",
        API_BASE, vin
    );

    request = Request::get(&status_url)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .header("authorization", format!("Bearer {}", token_response.access_token))
        .header("datetime", get_timestamp_ms())
        .header("x-correlationid", generate_uuid())
        .build();

    let response: Response = spin_sdk::http::send(request).await?;

    if *response.status() != 200 {
        println!(
            "Failed to get vehicle status. Status: {}",
            response.status()
        );
        let body = String::from_utf8_lossy(response.body());
        println!("Response body: {}", body);
        return Ok(Response::new(500, "Failed to get vehicle status"));
    }

    let electric_status: ElectricStatusResponse = response.body().as_ref().into();

    let soc = electric_status
        .payload
        .vehicle_info
        .charge_info
        .charge_remaining_amount
        .unwrap_or(0);

    let access_date = electric_status
        .payload
        .vehicle_info
        .last_update_timestamp;

    let return_value = CurrentStatus::new(soc, access_date);
    let json_response = serde_json::to_string(&return_value)?;

    println!("Success: {}", json_response);

    Ok(Response::new(200, json_response))
}
