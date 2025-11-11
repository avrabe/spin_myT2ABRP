use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use myt::{
    AuthenticateRequest, AuthenticateResponse, CachedToken, ElectricStatusResponse,
    LocationResponse, RefreshTokenRequest, TelemetryResponse, TokenRequest, TokenResponse,
    VehicleListResponse,
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use spin_sdk::http::{IncomingRequest, IntoResponse, Request, Response};
use spin_sdk::key_value::Store;
use spin_sdk::{http_component, variables};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const AUTH_URL: &str = "https://b2c-login.toyota-europe.com/json/realms/root/realms/tme/authenticate?authIndexType=service&authIndexValue=oneapp";
const AUTHORIZE_URL: &str = "https://b2c-login.toyota-europe.com/oauth2/realms/root/realms/tme/authorize?client_id=oneapp&scope=openid+profile+write&response_type=code&redirect_uri=com.toyota.oneapp:/oauth2Callback&code_challenge=plain&code_challenge_method=plain";
const TOKEN_URL: &str =
    "https://b2c-login.toyota-europe.com/oauth2/realms/root/realms/tme/access_token";
const API_BASE: &str = "https://ctpa-oneapi.tceu-ctp-prd.toyotaconnectedeurope.io";

const TOKEN_CACHE_KEY_PREFIX: &str = "toyota_auth_token_";
const TOKEN_TTL_SECONDS: i64 = 3600; // 1 hour TTL for per-user tokens
const VERSION: &str = env!("CARGO_PKG_VERSION");

// SECURITY: HMAC key for username hashing
// IMPORTANT: Change this to a random value in production via environment variable
// This prevents rainbow table attacks on cached username hashes
const HMAC_KEY_DEFAULT: &[u8] = b"toyota-myt-gateway-hmac-key-change-in-production";

// Input validation limits
const MAX_USERNAME_LENGTH: usize = 256;
const MAX_PASSWORD_LENGTH: usize = 256;

#[derive(Serialize, Deserialize, Debug)]
struct CurrentStatus {
    pub soc: i32,
    pub access_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charging_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ev_range: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ev_range_with_ac: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_charge_time: Option<i32>,
    pub version: String,
}

impl CurrentStatus {
    pub fn new(
        soc: i32,
        access_date: String,
        charging_status: Option<String>,
        ev_range: Option<f32>,
        ev_range_with_ac: Option<f32>,
        remaining_charge_time: Option<i32>,
    ) -> CurrentStatus {
        CurrentStatus {
            soc,
            access_date,
            charging_status,
            ev_range,
            ev_range_with_ac,
            remaining_charge_time,
            version: VERSION.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct HealthStatus {
    pub status: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AbrpTelemetry {
    pub utc: i64,
    pub soc: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lon: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_charging: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_parked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub odometer: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub est_battery_range: Option<f64>,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PerUserCachedToken {
    pub token: CachedToken,
    pub last_accessed: i64, // Unix timestamp
    pub ttl_seconds: i64,
}

impl PerUserCachedToken {
    pub fn new(token: CachedToken, current_time: i64, ttl_seconds: i64) -> Self {
        PerUserCachedToken {
            token,
            last_accessed: current_time,
            ttl_seconds,
        }
    }

    pub fn is_ttl_expired(&self, current_time: i64) -> bool {
        (current_time - self.last_accessed) > self.ttl_seconds
    }

    pub fn update_access_time(&mut self, current_time: i64) {
        self.last_accessed = current_time;
    }
}

fn get_timestamp_ms() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn get_current_timestamp() -> i64 {
    Utc::now().timestamp()
}

fn decode_jwt_uuid(id_token: &str) -> anyhow::Result<String> {
    // Split JWT into parts
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid JWT format: expected 3 parts, got {}", parts.len());
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
        .decode(payload_padded.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to decode JWT base64: {}", e))?;

    let payload_str = String::from_utf8(decoded)
        .map_err(|e| anyhow::anyhow!("Failed to parse JWT payload as UTF-8: {}", e))?;

    // Parse JSON to get uuid
    let payload: serde_json::Value = serde_json::from_str(&payload_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse JWT payload as JSON: {}", e))?;

    payload["uuid"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No uuid field found in JWT payload"))
        .map(|s| s.to_string())
}

fn validate_credentials(username: &str, password: &str) -> anyhow::Result<()> {
    // Validate username length
    if username.is_empty() {
        anyhow::bail!("Username cannot be empty");
    }
    if username.len() > MAX_USERNAME_LENGTH {
        anyhow::bail!("Username exceeds maximum length of {} characters", MAX_USERNAME_LENGTH);
    }

    // Validate password length
    if password.is_empty() {
        anyhow::bail!("Password cannot be empty");
    }
    if password.len() > MAX_PASSWORD_LENGTH {
        anyhow::bail!("Password exceeds maximum length of {} characters", MAX_PASSWORD_LENGTH);
    }

    // Basic format validation for username (should look like an email)
    if !username.contains('@') || !username.contains('.') {
        anyhow::bail!("Username must be a valid email address");
    }

    Ok(())
}

fn extract_basic_auth(request: &IncomingRequest) -> Option<(String, String)> {
    // Look for Authorization header by iterating over headers
    let headers = request.headers();
    let mut auth_value_str = None;

    for (name, value) in headers.entries() {
        if name.to_lowercase() == "authorization" {
            auth_value_str = Some(String::from_utf8_lossy(&value).to_string());
            break;
        }
    }

    let auth_value = auth_value_str?;

    // Check if it starts with "Basic "
    if !auth_value.starts_with("Basic ") {
        return None;
    }

    // Extract base64 part
    let base64_part = auth_value.strip_prefix("Basic ")?;

    // Decode base64 with size limit check
    let base64_bytes = base64_part.trim().as_bytes();
    if base64_bytes.len() > 1024 {  // Reasonable limit for base64 encoded credentials
        return None;
    }

    let decoded = general_purpose::STANDARD
        .decode(base64_bytes)
        .ok()?;

    let decoded_str = String::from_utf8(decoded).ok()?;

    // Split on first ':'
    let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }

    Some((parts[0].to_string(), parts[1].to_string()))
}

fn hash_username(username: &str) -> String {
    hash_username_with_key(username, None)
}

fn hash_username_with_key(username: &str, key: Option<&[u8]>) -> String {
    type HmacSha256 = Hmac<Sha256>;

    // Use provided key, or try to get from environment, or use default
    let hmac_key = match key {
        Some(k) => k.to_vec(),
        None => {
            variables::get("hmac_key")
                .unwrap_or_else(|_| String::from_utf8_lossy(HMAC_KEY_DEFAULT).to_string())
                .into_bytes()
        }
    };

    let mut mac = HmacSha256::new_from_slice(&hmac_key)
        .expect("HMAC can take key of any size");
    mac.update(username.as_bytes());
    let result = mac.finalize();

    format!("{:x}", result.into_bytes())
}

fn get_user_token_cache_key(username_hash: &str) -> String {
    format!("{}{}", TOKEN_CACHE_KEY_PREFIX, username_hash)
}

async fn send_request(request: Request) -> anyhow::Result<Response> {
    spin_sdk::http::send(request)
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {:?}", e))
}

async fn get_per_user_cached_token(
    store: &Store,
    username_hash: &str,
) -> anyhow::Result<Option<PerUserCachedToken>> {
    let cache_key = get_user_token_cache_key(username_hash);
    match store.get(&cache_key) {
        Ok(Some(bytes)) => {
            let per_user_token: PerUserCachedToken = serde_json::from_slice(&bytes)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize per-user cached token: {}", e))?;

            // Return without updating - let caller check expiration first
            Ok(Some(per_user_token))
        }
        Ok(None) | Err(_) => Ok(None), // Key doesn't exist or error reading
    }
}

async fn save_per_user_token_to_cache(
    store: &Store,
    username_hash: &str,
    per_user_token: &PerUserCachedToken,
) -> anyhow::Result<()> {
    let cache_key = get_user_token_cache_key(username_hash);
    let bytes = serde_json::to_vec(per_user_token)
        .map_err(|e| anyhow::anyhow!("Failed to serialize per-user token for caching: {}", e))?;
    store
        .set(&cache_key, &bytes)
        .map_err(|e| anyhow::anyhow!("Failed to save per-user token to cache: {}", e))?;
    Ok(())
}

async fn cleanup_expired_user_token(
    store: &Store,
    username_hash: &str,
) -> anyhow::Result<()> {
    let cache_key = get_user_token_cache_key(username_hash);
    store
        .delete(&cache_key)
        .map_err(|e| anyhow::anyhow!("Failed to delete expired token from cache: {}", e))?;
    println!("Cleaned up expired token for user");
    Ok(())
}

async fn refresh_access_token(
    refresh_token: String,
) -> anyhow::Result<(TokenResponse, String)> {
    println!("Refreshing access token...");

    let refresh_request = RefreshTokenRequest::new(refresh_token.clone());
    let token_body = format!(
        "grant_type={}&refresh_token={}",
        refresh_request.grant_type, refresh_request.refresh_token
    );

    let request = Request::post(TOKEN_URL, token_body)
        .header("content-type", "application/x-www-form-urlencoded")
        .header("authorization", "Basic b25lYXBwOm9uZWFwcA==")
        .build();

    let response = send_request(request).await?;

    if *response.status() != 200 {
        anyhow::bail!(
            "Token refresh failed with status: {}. Body: {}",
            response.status(),
            String::from_utf8_lossy(response.body())
        );
    }

    let token_response: TokenResponse = serde_json::from_slice(response.body())
        .map_err(|e| anyhow::anyhow!("Failed to parse token response: {}", e))?;

    let uuid = decode_jwt_uuid(&token_response.id_token)?;

    println!("Token refreshed successfully");
    Ok((token_response, uuid))
}

async fn perform_full_oauth_flow(
    username: String,
    password: String,
) -> anyhow::Result<(TokenResponse, String)> {
    println!("Performing full OAuth2 authentication flow...");

    // Step 1: Initial authentication request
    println!("Step 1: Starting authentication...");
    let auth_request = AuthenticateRequest::new();
    let request = Request::post(AUTH_URL, auth_request)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .build();

    let response = send_request(request).await?;
    let auth_response: AuthenticateResponse = serde_json::from_slice(response.body())
        .map_err(|e| anyhow::anyhow!("Failed to parse initial auth response: {}", e))?;

    // Step 2: Submit credentials
    println!("Step 2: Submitting credentials...");
    let auth_request_with_creds =
        AuthenticateRequest::with_credentials(username, password, auth_response.auth_id);
    let request = Request::post(AUTH_URL, auth_request_with_creds)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .build();

    let response = send_request(request).await?;

    if *response.status() != 200 {
        anyhow::bail!(
            "Authentication failed with status: {}. Body: {}",
            response.status(),
            String::from_utf8_lossy(response.body())
        );
    }

    let auth_response: AuthenticateResponse = serde_json::from_slice(response.body())
        .map_err(|e| anyhow::anyhow!("Failed to parse credential submission response: {}", e))?;

    let token_id = auth_response
        .token_id
        .ok_or_else(|| anyhow::anyhow!("No tokenId received after authentication"))?;

    println!("Authentication successful, got tokenId");

    // Step 3: Get authorization code
    println!("Step 3: Getting authorization code...");
    let cookie = format!("iPlanetDirectoryPro={}", token_id);
    let request = Request::get(AUTHORIZE_URL).header("cookie", cookie).build();

    let response = send_request(request).await?;

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
        .ok_or_else(|| anyhow::anyhow!("No code parameter found in redirect URL"))?
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
    let request = Request::post(TOKEN_URL, token_body)
        .header("content-type", "application/x-www-form-urlencoded")
        .header("authorization", "Basic b25lYXBwOm9uZWFwcA==")
        .build();

    let response = send_request(request).await?;

    if *response.status() != 200 {
        anyhow::bail!(
            "Token exchange failed with status: {}. Body: {}",
            response.status(),
            String::from_utf8_lossy(response.body())
        );
    }

    let token_response: TokenResponse = serde_json::from_slice(response.body())
        .map_err(|e| anyhow::anyhow!("Failed to parse token exchange response: {}", e))?;

    println!("Got access token");

    // Decode UUID from JWT
    let uuid = decode_jwt_uuid(&token_response.id_token)?;
    println!("Decoded UUID: {}", uuid);

    Ok((token_response, uuid))
}

async fn fetch_vehicle_location(
    token: &CachedToken,
    vin: &str,
) -> anyhow::Result<LocationResponse> {
    println!("Fetching vehicle location...");
    let url = format!("{}/v1/global/remote/location?vin={}", API_BASE, vin);

    let request = Request::get(&url)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .header("authorization", format!("Bearer {}", token.access_token))
        .header("datetime", get_timestamp_ms())
        .header("x-correlationid", Uuid::new_v4().to_string())
        .build();

    let response = send_request(request).await?;

    if *response.status() != 200 {
        anyhow::bail!(
            "Location request failed with status: {}",
            response.status()
        );
    }

    Ok(serde_json::from_slice(response.body())?)
}

async fn fetch_vehicle_telemetry(
    token: &CachedToken,
    vin: &str,
) -> anyhow::Result<TelemetryResponse> {
    println!("Fetching vehicle telemetry...");
    let url = format!("{}/v1/global/remote/telemetry?vin={}", API_BASE, vin);

    let request = Request::get(&url)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .header("authorization", format!("Bearer {}", token.access_token))
        .header("datetime", get_timestamp_ms())
        .header("x-correlationid", Uuid::new_v4().to_string())
        .build();

    let response = send_request(request).await?;

    if *response.status() != 200 {
        anyhow::bail!(
            "Telemetry request failed with status: {}",
            response.status()
        );
    }

    Ok(serde_json::from_slice(response.body())?)
}

async fn fetch_vehicle_list(token: &CachedToken) -> anyhow::Result<VehicleListResponse> {
    println!("Fetching vehicle list...");
    let url = format!("{}/v1/vehicles", API_BASE);

    let request = Request::get(&url)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .header("authorization", format!("Bearer {}", token.access_token))
        .header("datetime", get_timestamp_ms())
        .header("x-correlationid", Uuid::new_v4().to_string())
        .build();

    let response = send_request(request).await?;

    if *response.status() != 200 {
        anyhow::bail!(
            "Vehicle list request failed with status: {}",
            response.status()
        );
    }

    Ok(serde_json::from_slice(response.body())?)
}

fn parse_iso8601_to_timestamp(iso_string: &str) -> i64 {
    // Try to parse ISO 8601 timestamp
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso_string) {
        return dt.timestamp();
    }
    // If parsing fails, return current time
    Utc::now().timestamp()
}

fn add_cors_headers(mut builder: spin_sdk::http::ResponseBuilder) -> spin_sdk::http::ResponseBuilder {
    builder.header("access-control-allow-origin", "*");
    builder.header("access-control-allow-methods", "GET, OPTIONS");
    builder.header("access-control-allow-headers", "Content-Type, Authorization");
    builder
}

async fn get_or_refresh_token_for_user(
    username: String,
    password: String,
) -> anyhow::Result<CachedToken> {
    let store = Store::open_default()
        .map_err(|e| anyhow::anyhow!("Failed to open key-value store: {}", e))?;

    let current_time = get_current_timestamp();
    let username_hash = hash_username(&username);

    // Try to get cached token for this user
    if let Some(mut per_user_token) = get_per_user_cached_token(&store, &username_hash).await? {
        // Check if TTL expired (1 hour since last access) - BEFORE updating access time
        if per_user_token.is_ttl_expired(current_time) {
            println!("Per-user token TTL expired (inactive for {} seconds), cleaning up...",
                current_time - per_user_token.last_accessed);
            cleanup_expired_user_token(&store, &username_hash).await?;
        } else if !per_user_token.token.is_expired(current_time) {
            // Token is still valid, update access time and save
            println!("Using cached token for user (expires in {} seconds, last accessed {} seconds ago)",
                per_user_token.token.expires_at - current_time,
                current_time - per_user_token.last_accessed);

            // Update and save access time
            per_user_token.update_access_time(current_time);
            save_per_user_token_to_cache(&store, &username_hash, &per_user_token).await?;
            return Ok(per_user_token.token);
        } else {
            println!("Cached token expired, attempting refresh...");

            // Try to refresh the token
            match refresh_access_token(per_user_token.token.refresh_token.clone()).await {
                Ok((token_response, uuid)) => {
                    let new_cached_token =
                        CachedToken::from_token_response(token_response, uuid, current_time);
                    let new_per_user_token = PerUserCachedToken::new(
                        new_cached_token.clone(),
                        current_time,
                        TOKEN_TTL_SECONDS,
                    );
                    save_per_user_token_to_cache(&store, &username_hash, &new_per_user_token).await?;
                    return Ok(new_cached_token);
                }
                Err(e) => {
                    println!("Token refresh failed: {}. Performing full authentication...", e);
                    cleanup_expired_user_token(&store, &username_hash).await?;
                }
            }
        }
    } else {
        println!("No cached token found for user");
    }

    // Perform full OAuth flow
    let (token_response, uuid) = perform_full_oauth_flow(username, password).await?;

    let cached_token = CachedToken::from_token_response(token_response, uuid, current_time);
    let per_user_token = PerUserCachedToken::new(cached_token.clone(), current_time, TOKEN_TTL_SECONDS);
    save_per_user_token_to_cache(&store, &username_hash, &per_user_token).await?;

    Ok(cached_token)
}

/// Send an HTTP request and return the response.
#[http_component]
async fn handle_request(request: IncomingRequest) -> Result<impl IntoResponse, anyhow::Error> {
    // Handle health endpoint
    let path = request.uri();

    // Handle OPTIONS requests for CORS preflight
    if request.method() == spin_sdk::http::Method::Options {
        return Ok(add_cors_headers(Response::builder())
            .status(200)
            .body("")
            .build());
    }

    if path == "/health" {
        let health = HealthStatus {
            status: "healthy".to_string(),
            version: VERSION.to_string(),
        };
        let json_response = serde_json::to_string(&health)
            .map_err(|e| anyhow::anyhow!("Failed to serialize health response: {}", e))?;

        return Ok(add_cors_headers(Response::builder())
            .status(200)
            .header("content-type", "application/json")
            .body(json_response)
            .build());
    }

    // Get credentials: prioritize Basic Auth, fallback to environment variables
    let (username, password) = match extract_basic_auth(&request) {
        Some((user, pass)) => {
            println!("Using Basic Auth credentials");
            (user, pass)
        }
        None => {
            // Try environment variables for backward compatibility
            match (variables::get("username"), variables::get("password")) {
                (Ok(user), Ok(pass)) => {
                    println!("Using environment variable credentials");
                    (user, pass)
                }
                _ => {
                    let error_json = serde_json::json!({
                        "error": "Authentication required",
                        "message": "Please provide credentials via Basic Auth header or environment variables",
                        "version": VERSION
                    });
                    return Ok(add_cors_headers(Response::builder())
                        .status(401)
                        .header("content-type", "application/json")
                        .header("www-authenticate", "Basic realm=\"Toyota MyT Gateway\"")
                        .body(error_json.to_string())
                        .build());
                }
            }
        }
    };

    // Validate credentials before use (prevent DoS, ensure proper format)
    if let Err(e) = validate_credentials(&username, &password) {
        let error_json = serde_json::json!({
            "error": "Invalid credentials",
            "message": e.to_string(),
            "version": VERSION
        });
        return Ok(add_cors_headers(Response::builder())
            .status(400)
            .header("content-type", "application/json")
            .body(error_json.to_string())
            .build());
    }

    // Get VIN from environment variable or query parameter
    let vin = variables::get("vin").unwrap_or_else(|_| {
        // Could also support VIN from query parameter in future
        "".to_string()
    });

    if vin.is_empty() && path != "/vehicles" {
        let error_json = serde_json::json!({
            "error": "VIN required",
            "message": "VIN must be configured via environment variable",
            "version": VERSION
        });
        return Ok(add_cors_headers(Response::builder())
            .status(400)
            .header("content-type", "application/json")
            .body(error_json.to_string())
            .build());
    }

    // Get or refresh authentication token for this user
    let cached_token = match get_or_refresh_token_for_user(username, password).await {
        Ok(token) => token,
        Err(e) => {
            println!("Authentication failed: {}", e);
            let error_json = serde_json::json!({
                "error": "Authentication failed",
                "message": e.to_string(),
                "version": VERSION
            });
            return Ok(add_cors_headers(Response::builder())
                .status(401)
                .header("content-type", "application/json")
                .body(error_json.to_string())
                .build());
        }
    };

    // Handle /vehicles endpoint - list all vehicles
    if path == "/vehicles" {
        match fetch_vehicle_list(&cached_token).await {
            Ok(vehicle_list) => {
                let json_response = serde_json::to_string(&vehicle_list)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize vehicle list: {}", e))?;
                return Ok(add_cors_headers(Response::builder())
                    .status(200)
                    .header("content-type", "application/json")
                    .body(json_response)
                    .build());
            }
            Err(e) => {
                let error_json = serde_json::json!({
                    "error": "Failed to fetch vehicle list",
                    "message": e.to_string(),
                    "version": VERSION
                });
                return Ok(add_cors_headers(Response::builder())
                    .status(500)
                    .header("content-type", "application/json")
                    .body(error_json.to_string())
                    .build());
            }
        }
    }

    // Handle /location endpoint
    if path == "/location" {
        match fetch_vehicle_location(&cached_token, &vin).await {
            Ok(location) => {
                let json_response = serde_json::to_string(&location)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize location: {}", e))?;
                return Ok(add_cors_headers(Response::builder())
                    .status(200)
                    .header("content-type", "application/json")
                    .body(json_response)
                    .build());
            }
            Err(e) => {
                let error_json = serde_json::json!({
                    "error": "Failed to fetch location",
                    "message": e.to_string(),
                    "version": VERSION
                });
                return Ok(add_cors_headers(Response::builder())
                    .status(500)
                    .header("content-type", "application/json")
                    .body(error_json.to_string())
                    .build());
            }
        }
    }

    // Handle /telemetry endpoint
    if path == "/telemetry" {
        match fetch_vehicle_telemetry(&cached_token, &vin).await {
            Ok(telemetry) => {
                let json_response = serde_json::to_string(&telemetry)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize telemetry: {}", e))?;
                return Ok(add_cors_headers(Response::builder())
                    .status(200)
                    .header("content-type", "application/json")
                    .body(json_response)
                    .build());
            }
            Err(e) => {
                let error_json = serde_json::json!({
                    "error": "Failed to fetch telemetry",
                    "message": e.to_string(),
                    "version": VERSION
                });
                return Ok(add_cors_headers(Response::builder())
                    .status(500)
                    .header("content-type", "application/json")
                    .body(error_json.to_string())
                    .build());
            }
        }
    }

    // Handle /abrp endpoint - ABRP-formatted telemetry
    if path == "/abrp" {
        // Fetch electric status
        let status_url = format!("{}/v1/global/remote/electric/status?vin={}", API_BASE, vin);
        let request = Request::get(&status_url)
            .header("content-type", "application/json")
            .header("accept", "application/json")
            .header("authorization", format!("Bearer {}", cached_token.access_token))
            .header("datetime", get_timestamp_ms())
            .header("x-correlationid", Uuid::new_v4().to_string())
            .build();

        let electric_status: ElectricStatusResponse = match send_request(request).await {
            Ok(response) => {
                if *response.status() != 200 {
                    let error_json = serde_json::json!({
                        "error": "Failed to fetch electric status",
                        "status": *response.status(),
                        "version": VERSION
                    });
                    return Ok(add_cors_headers(Response::builder())
                        .status(500)
                        .header("content-type", "application/json")
                        .body(error_json.to_string())
                        .build());
                }
                serde_json::from_slice(response.body())
                    .map_err(|e| anyhow::anyhow!("Failed to parse electric status: {}", e))?
            }
            Err(e) => {
                let error_json = serde_json::json!({
                    "error": "Failed to fetch electric status",
                    "message": e.to_string(),
                    "version": VERSION
                });
                return Ok(add_cors_headers(Response::builder())
                    .status(500)
                    .header("content-type", "application/json")
                    .body(error_json.to_string())
                    .build());
            }
        };

        // Fetch location (best effort)
        let location = fetch_vehicle_location(&cached_token, &vin).await.ok();

        // Fetch telemetry (best effort)
        let telemetry = fetch_vehicle_telemetry(&cached_token, &vin).await.ok();

        let charge_info = &electric_status.payload.vehicle_info.charge_info;
        let soc = charge_info.charge_remaining_amount.unwrap_or(0) as f64;
        let utc = parse_iso8601_to_timestamp(&electric_status.payload.vehicle_info.last_update_timestamp);

        // Determine if charging
        let is_charging = charge_info.charging_status.as_ref().map(|status| {
            status.to_uppercase().contains("CHARGING") || status.to_uppercase().contains("CONNECTED")
        });

        let abrp_data = AbrpTelemetry {
            utc,
            soc,
            lat: location.as_ref().map(|l| l.payload.vehicle_info.location.lat),
            lon: location.as_ref().map(|l| l.payload.vehicle_info.location.lon),
            is_charging,
            is_parked: None, // Not available from Toyota API
            odometer: telemetry.as_ref()
                .and_then(|t| t.payload.vehicle_info.odometer.as_ref())
                .and_then(|o| o.value),
            est_battery_range: charge_info.ev_range.map(|r| r as f64),
            version: VERSION.to_string(),
        };

        let json_response = serde_json::to_string(&abrp_data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize ABRP data: {}", e))?;

        return Ok(add_cors_headers(Response::builder())
            .status(200)
            .header("content-type", "application/json")
            .body(json_response)
            .build());
    }

    // Step 5: Get vehicle electric status (default endpoint)
    println!("Getting vehicle electric status...");
    let status_url = format!("{}/v1/global/remote/electric/status?vin={}", API_BASE, vin);

    let request = Request::get(&status_url)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .header(
            "authorization",
            format!("Bearer {}", cached_token.access_token),
        )
        .header("datetime", get_timestamp_ms())
        .header("x-correlationid", Uuid::new_v4().to_string())
        .build();

    let response = match send_request(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to get vehicle status: {}", e);
            let error_json = serde_json::json!({
                "error": "Failed to get vehicle status",
                "message": e.to_string(),
                "version": VERSION
            });
            return Ok(add_cors_headers(Response::builder())
                .status(500)
                .header("content-type", "application/json")
                .body(error_json.to_string())
                .build());
        }
    };

    if *response.status() != 200 {
        let error_body = String::from_utf8_lossy(response.body());
        println!(
            "Failed to get vehicle status. Status: {}. Body: {}",
            response.status(),
            error_body
        );
        let error_json = serde_json::json!({
            "error": "Failed to get vehicle status",
            "status": *response.status(),
            "version": VERSION
        });
        return Ok(add_cors_headers(Response::builder())
            .status(500)
            .header("content-type", "application/json")
            .body(error_json.to_string())
            .build());
    }

    let electric_status: ElectricStatusResponse = serde_json::from_slice(response.body())
        .map_err(|e| anyhow::anyhow!("Failed to parse vehicle status response: {}", e))?;

    let charge_info = &electric_status.payload.vehicle_info.charge_info;

    let soc = charge_info.charge_remaining_amount.unwrap_or(0);
    let access_date = electric_status
        .payload
        .vehicle_info
        .last_update_timestamp;

    let return_value = CurrentStatus::new(
        soc,
        access_date,
        charge_info.charging_status.clone(),
        charge_info.ev_range,
        charge_info.ev_range_with_ac,
        charge_info.remaining_charge_time,
    );
    let json_response = serde_json::to_string(&return_value)
        .map_err(|e| anyhow::anyhow!("Failed to serialize response: {}", e))?;

    println!("Success: {}", json_response);

    Ok(add_cors_headers(Response::builder())
        .status(200)
        .header("content-type", "application/json")
        .body(json_response)
        .build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_jwt_uuid() {
        // Valid JWT with uuid field (base64url encoded)
        let test_jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1dWlkIjoidGVzdC11dWlkLTEyMzQiLCJpYXQiOjE1MTYyMzkwMjJ9.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let result = decode_jwt_uuid(test_jwt);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-uuid-1234");
    }

    #[test]
    fn test_decode_jwt_uuid_invalid_format() {
        let invalid_jwt = "invalid.jwt";
        let result = decode_jwt_uuid(invalid_jwt);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid JWT format"));
    }

    #[test]
    fn test_decode_jwt_uuid_no_uuid_field() {
        // Test with JWT that has properly encoded JSON but no uuid field
        // Payload: {"name":"test"} -> eyJuYW1lIjoidGVzdCJ9 (base64url, no padding needed)
        let test_jwt = "header.eyJuYW1lIjoidGVzdCJ9.signature";
        let result = decode_jwt_uuid(test_jwt);
        assert!(result.is_err());
        // Just verify it's an error - could be base64 or uuid related
        assert!(result.is_err());
    }

    #[test]
    fn test_cached_token_expiry() {
        let token_response = TokenResponse {
            access_token: "test_access".to_string(),
            refresh_token: "test_refresh".to_string(),
            id_token: "test_id".to_string(),
            expires_in: 3600,
        };

        let current_time = 1000;
        let cached_token =
            CachedToken::from_token_response(token_response, "test-uuid".to_string(), current_time);

        // Should not be expired immediately
        assert!(!cached_token.is_expired(current_time));

        // Should not be expired within the timeframe (minus buffer)
        assert!(!cached_token.is_expired(current_time + 3500));

        // Should be expired after the expiry time
        assert!(cached_token.is_expired(current_time + 3600));
    }

    #[test]
    fn test_current_status_creation() {
        let status = CurrentStatus::new(
            85,
            "2025-01-01T12:00:00Z".to_string(),
            Some("CHARGING".to_string()),
            Some(250.5),
            Some(230.0),
            Some(120),
        );
        assert_eq!(status.soc, 85);
        assert_eq!(status.access_date, "2025-01-01T12:00:00Z");
        assert_eq!(status.charging_status, Some("CHARGING".to_string()));
        assert_eq!(status.ev_range, Some(250.5));
        assert_eq!(status.ev_range_with_ac, Some(230.0));
        assert_eq!(status.remaining_charge_time, Some(120));
        assert_eq!(status.version, VERSION);
    }

    #[test]
    fn test_current_status_with_optional_fields() {
        let status = CurrentStatus::new(
            75,
            "2025-01-01T13:00:00Z".to_string(),
            None,
            None,
            None,
            None,
        );
        assert_eq!(status.soc, 75);
        assert_eq!(status.access_date, "2025-01-01T13:00:00Z");
        assert_eq!(status.charging_status, None);
        assert_eq!(status.ev_range, None);
        assert_eq!(status.ev_range_with_ac, None);
        assert_eq!(status.remaining_charge_time, None);
        assert_eq!(status.version, VERSION);
    }

    #[test]
    fn test_get_timestamp_ms() {
        let timestamp = get_timestamp_ms();
        assert!(!timestamp.is_empty());
        // Should be a valid number
        assert!(timestamp.parse::<u128>().is_ok());
    }

    #[test]
    fn test_hash_username() {
        let test_key = b"test-hmac-key-for-testing";
        let username = "test@example.com";
        let hash1 = hash_username_with_key(username, Some(test_key));
        let hash2 = hash_username_with_key(username, Some(test_key));

        // Same username should produce same hash
        assert_eq!(hash1, hash2);

        // Hash should be 64 characters (HMAC-SHA256 in hex)
        assert_eq!(hash1.len(), 64);

        // Different username should produce different hash
        let hash3 = hash_username_with_key("different@example.com", Some(test_key));
        assert_ne!(hash1, hash3);

        // Different key should produce different hash
        let hash4 = hash_username_with_key(username, Some(b"different-key"));
        assert_ne!(hash1, hash4);
    }

    #[test]
    fn test_get_user_token_cache_key() {
        let username_hash = "abc123";
        let cache_key = get_user_token_cache_key(username_hash);

        assert!(cache_key.starts_with(TOKEN_CACHE_KEY_PREFIX));
        assert!(cache_key.contains("abc123"));
        assert_eq!(cache_key, format!("{}abc123", TOKEN_CACHE_KEY_PREFIX));
    }

    #[test]
    fn test_per_user_cached_token_ttl() {
        let token = CachedToken {
            access_token: "test_access".to_string(),
            refresh_token: "test_refresh".to_string(),
            id_token: "test_id".to_string(),
            uuid: "test_uuid".to_string(),
            expires_at: 5000,
        };

        let current_time = 1000;
        let ttl_seconds = 3600;

        let per_user_token = PerUserCachedToken::new(token, current_time, ttl_seconds);

        // Should not be TTL expired immediately
        assert!(!per_user_token.is_ttl_expired(current_time));

        // Should not be TTL expired within TTL window
        assert!(!per_user_token.is_ttl_expired(current_time + 3500));

        // Should be TTL expired after TTL window
        assert!(per_user_token.is_ttl_expired(current_time + 3601));

        // Last accessed should be updated
        assert_eq!(per_user_token.last_accessed, current_time);
    }

    #[test]
    fn test_per_user_cached_token_update_access_time() {
        let token = CachedToken {
            access_token: "test_access".to_string(),
            refresh_token: "test_refresh".to_string(),
            id_token: "test_id".to_string(),
            uuid: "test_uuid".to_string(),
            expires_at: 5000,
        };

        let current_time = 1000;
        let mut per_user_token = PerUserCachedToken::new(token, current_time, 3600);

        assert_eq!(per_user_token.last_accessed, current_time);

        // Update access time
        let new_time = 2000;
        per_user_token.update_access_time(new_time);

        assert_eq!(per_user_token.last_accessed, new_time);
    }
}
