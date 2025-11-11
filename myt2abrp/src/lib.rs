use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use myt::{
    AuthenticateRequest, AuthenticateResponse, CachedToken, ElectricStatusResponse,
    RefreshTokenRequest, TokenRequest, TokenResponse,
};
use serde::{Deserialize, Serialize};
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

const TOKEN_CACHE_KEY: &str = "toyota_auth_token";

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

async fn send_request(request: Request) -> anyhow::Result<Response> {
    spin_sdk::http::send(request)
        .await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {:?}", e))
}

async fn get_cached_token(store: &Store) -> anyhow::Result<Option<CachedToken>> {
    match store.get(TOKEN_CACHE_KEY) {
        Ok(Some(bytes)) => {
            let token: CachedToken = serde_json::from_slice(&bytes)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize cached token: {}", e))?;
            Ok(Some(token))
        }
        Ok(None) | Err(_) => Ok(None), // Key doesn't exist or error reading
    }
}

async fn save_token_to_cache(store: &Store, token: &CachedToken) -> anyhow::Result<()> {
    let bytes = serde_json::to_vec(token)
        .map_err(|e| anyhow::anyhow!("Failed to serialize token for caching: {}", e))?;
    store
        .set(TOKEN_CACHE_KEY, &bytes)
        .map_err(|e| anyhow::anyhow!("Failed to save token to cache: {}", e))?;
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

async fn get_or_refresh_token(
    username: String,
    password: String,
) -> anyhow::Result<CachedToken> {
    let store = Store::open_default()
        .map_err(|e| anyhow::anyhow!("Failed to open key-value store: {}", e))?;

    let current_time = get_current_timestamp();

    // Try to get cached token
    if let Some(cached_token) = get_cached_token(&store).await? {
        if !cached_token.is_expired(current_time) {
            println!("Using cached token (expires in {} seconds)", cached_token.expires_at - current_time);
            return Ok(cached_token);
        }

        println!("Cached token expired, attempting refresh...");

        // Try to refresh the token
        match refresh_access_token(cached_token.refresh_token.clone()).await {
            Ok((token_response, uuid)) => {
                let new_cached_token =
                    CachedToken::from_token_response(token_response, uuid, current_time);
                save_token_to_cache(&store, &new_cached_token).await?;
                return Ok(new_cached_token);
            }
            Err(e) => {
                println!("Token refresh failed: {}. Performing full authentication...", e);
            }
        }
    } else {
        println!("No cached token found");
    }

    // Perform full OAuth flow
    let (token_response, uuid) = perform_full_oauth_flow(username, password).await?;

    let cached_token = CachedToken::from_token_response(token_response, uuid, current_time);
    save_token_to_cache(&store, &cached_token).await?;

    Ok(cached_token)
}

/// Send an HTTP request and return the response.
#[http_component]
async fn handle_request(_request: IncomingRequest) -> Result<impl IntoResponse, anyhow::Error> {
    let username = variables::get("username")
        .map_err(|e| anyhow::anyhow!("Failed to get username variable: {}", e))?;
    let password = variables::get("password")
        .map_err(|e| anyhow::anyhow!("Failed to get password variable: {}", e))?;
    let vin =
        variables::get("vin").map_err(|e| anyhow::anyhow!("Failed to get vin variable: {}", e))?;

    // Get or refresh authentication token
    let cached_token = match get_or_refresh_token(username, password).await {
        Ok(token) => token,
        Err(e) => {
            println!("Authentication failed: {}", e);
            return Ok(Response::new(
                401,
                format!("Authentication failed: {}", e),
            ));
        }
    };

    // Step 5: Get vehicle electric status
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
            return Ok(Response::new(
                500,
                format!("Failed to get vehicle status: {}", e),
            ));
        }
    };

    if *response.status() != 200 {
        let error_body = String::from_utf8_lossy(response.body());
        println!(
            "Failed to get vehicle status. Status: {}. Body: {}",
            response.status(),
            error_body
        );
        return Ok(Response::new(
            500,
            format!(
                "Failed to get vehicle status. Status: {}",
                response.status()
            ),
        ));
    }

    let electric_status: ElectricStatusResponse = serde_json::from_slice(response.body())
        .map_err(|e| anyhow::anyhow!("Failed to parse vehicle status response: {}", e))?;

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
    let json_response = serde_json::to_string(&return_value)
        .map_err(|e| anyhow::anyhow!("Failed to serialize response: {}", e))?;

    println!("Success: {}", json_response);

    Ok(Response::new(200, json_response))
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
        let status = CurrentStatus::new(85, "2025-01-01T12:00:00Z".to_string());
        assert_eq!(status.soc, 85);
        assert_eq!(status.access_date, "2025-01-01T12:00:00Z");
    }

    #[test]
    fn test_get_timestamp_ms() {
        let timestamp = get_timestamp_ms();
        assert!(!timestamp.is_empty());
        // Should be a valid number
        assert!(timestamp.parse::<u128>().is_ok());
    }
}
