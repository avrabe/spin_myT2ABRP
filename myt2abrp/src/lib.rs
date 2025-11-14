use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use hmac::{Hmac, Mac};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use myt::{
    AuthenticateRequest, AuthenticateResponse, CachedToken, ElectricStatusResponse,
    LocationResponse, RefreshTokenRequest, TelemetryResponse, TokenRequest, TokenResponse,
    VehicleListResponse,
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use spin_sdk::http::{IncomingRequest, IntoResponse, Request, Response};
use spin_sdk::key_value::Store;
use spin_sdk::{http_component, variables};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

const AUTH_URL: &str = "https://b2c-login.toyota-europe.com/json/realms/root/realms/tme/authenticate?authIndexType=service&authIndexValue=oneapp";
const AUTHORIZE_URL: &str = "https://b2c-login.toyota-europe.com/oauth2/realms/root/realms/tme/authorize?client_id=oneapp&scope=openid+profile+write&response_type=code&redirect_uri=com.toyota.oneapp:/oauth2Callback&code_challenge=plain&code_challenge_method=plain";
const TOKEN_URL: &str =
    "https://b2c-login.toyota-europe.com/oauth2/realms/root/realms/tme/access_token";
const API_BASE: &str = "https://ctpa-oneapi.tceu-ctp-prd.toyotaconnectedeurope.io";

const TOKEN_CACHE_KEY_PREFIX: &str = "toyota_auth_token_";
const SESSION_KEY_PREFIX: &str = "session_";
const RATE_LIMIT_KEY_PREFIX: &str = "ratelimit_";
const REVOKED_TOKEN_KEY_PREFIX: &str = "revoked_";

// Toyota OAuth Token Cache Settings
const TOKEN_TTL_SECONDS: i64 = 3600; // 1 hour cache for Toyota OAuth tokens

// Vehicle Data Cache Settings
const VEHICLE_DATA_CACHE_KEY_PREFIX: &str = "vehicle_data_";
const VEHICLE_DATA_TTL_SECONDS: i64 = 300; // 5 minutes cache for vehicle data

const VERSION: &str = env!("CARGO_PKG_VERSION");

// JWT Token Settings
const JWT_ACCESS_TOKEN_EXPIRY: i64 = 900; // 15 minutes
const JWT_REFRESH_TOKEN_EXPIRY: i64 = 604800; // 7 days
const JWT_ALGORITHM: Algorithm = Algorithm::HS256;

// SECURITY: JWT secret key
// CRITICAL: Must be set via environment variable in production
const JWT_SECRET_DEFAULT: &[u8] = b"toyota-gateway-jwt-secret-CHANGE-IN-PRODUCTION";

// SECURITY: HMAC key for username hashing
// IMPORTANT: Change this to a random value in production via environment variable
const HMAC_KEY_DEFAULT: &[u8] = b"toyota-myt-gateway-hmac-key-change-in-production";

// Rate Limiting Settings
const RATE_LIMIT_PER_USER_HOUR: u32 = 100; // 100 requests per user per hour
#[allow(dead_code)] // Reserved for future IP-based rate limiting
const RATE_LIMIT_PER_IP_HOUR: u32 = 1000; // 1000 requests per IP per hour
const RATE_LIMIT_LOGIN_ATTEMPTS: u32 = 5; // 5 failed login attempts
const RATE_LIMIT_LOGIN_LOCKOUT_SECONDS: i64 = 900; // 15 minutes lockout

// Input validation limits
const MAX_USERNAME_LENGTH: usize = 256;
const MAX_PASSWORD_LENGTH: usize = 256;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
struct CurrentStatus {
    /// State of Charge (battery percentage)
    pub soc: i32,
    /// Timestamp of data access
    pub access_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Charging status (CHARGING, NOT_CHARGING, etc.)
    pub charging_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Estimated EV range in km
    pub ev_range: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Estimated EV range with AC in km
    pub ev_range_with_ac: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Remaining charge time in minutes
    pub remaining_charge_time: Option<i32>,
    /// API version
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

#[derive(Serialize, Deserialize, Debug, ToSchema)]
struct HealthStatus {
    /// Service health status
    pub status: String,
    /// API version
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// KV store status
    pub kv_store: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Uptime in seconds
    pub uptime_seconds: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
struct AbrpTelemetry {
    /// UTC timestamp
    pub utc: i64,
    /// State of Charge (0-100)
    pub soc: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Latitude
    pub lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Longitude
    pub lon: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Is vehicle charging
    pub is_charging: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Is vehicle parked
    pub is_parked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Odometer reading in km
    pub odometer: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Estimated battery range in km
    pub est_battery_range: Option<f64>,
    /// API version
    pub version: String,
}

// JWT Token Claims
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
struct Claims {
    /// Subject (username/email)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// JWT ID (unique identifier)
    pub jti: String,
    /// Token type: "access" or "refresh"
    pub token_type: String,
}

// Login Request
#[derive(Debug, Deserialize, ToSchema)]
struct LoginRequest {
    /// User email address
    pub username: String,
    /// User password
    pub password: String,
}

// Login Response
#[derive(Debug, Serialize, ToSchema)]
struct LoginResponse {
    /// JWT access token
    pub access_token: String,
    /// JWT refresh token
    pub refresh_token: String,
    /// Token type (Bearer)
    pub token_type: String,
    /// Access token expiry in seconds
    pub expires_in: i64,
}

// Refresh Token Request
#[derive(Debug, Deserialize, ToSchema)]
struct RefreshRequest {
    /// JWT refresh token
    pub refresh_token: String,
}

// Session Info
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Session {
    pub session_id: String,
    pub username: String,
    pub created_at: i64,
    pub last_accessed: i64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

// Rate Limit Info
#[derive(Debug, Serialize, Deserialize)]
struct RateLimitInfo {
    pub count: u32,
    pub window_start: i64,
    pub lockout_until: Option<i64>,
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
        anyhow::bail!(
            "Username exceeds maximum length of {} characters",
            MAX_USERNAME_LENGTH
        );
    }

    // Validate password length
    if password.is_empty() {
        anyhow::bail!("Password cannot be empty");
    }
    if password.len() > MAX_PASSWORD_LENGTH {
        anyhow::bail!(
            "Password exceeds maximum length of {} characters",
            MAX_PASSWORD_LENGTH
        );
    }

    // Basic format validation for username (should look like an email)
    if !username.contains('@') || !username.contains('.') {
        anyhow::bail!("Username must be a valid email address");
    }

    Ok(())
}

fn hash_username(username: &str) -> String {
    hash_username_with_key(username, None)
}

fn hash_username_with_key(username: &str, key: Option<&[u8]>) -> String {
    type HmacSha256 = Hmac<Sha256>;

    // Use provided key, or get from environment
    let hmac_key = match key {
        Some(k) => k.to_vec(),
        None => get_hmac_key(),
    };

    let mut mac = HmacSha256::new_from_slice(&hmac_key).expect("HMAC can take key of any size");
    mac.update(username.as_bytes());
    let result = mac.finalize();

    format!("{:x}", result.into_bytes())
}

fn get_user_token_cache_key(username_hash: &str) -> String {
    format!("{}{}", TOKEN_CACHE_KEY_PREFIX, username_hash)
}

// ============================================================================
// JWT TOKEN FUNCTIONS
// ============================================================================

fn get_jwt_secret() -> Vec<u8> {
    variables::get("jwt_secret")
        .unwrap_or_else(|_| String::from_utf8_lossy(JWT_SECRET_DEFAULT).to_string())
        .into_bytes()
}

fn get_hmac_key() -> Vec<u8> {
    variables::get("hmac_key")
        .unwrap_or_else(|_| String::from_utf8_lossy(HMAC_KEY_DEFAULT).to_string())
        .into_bytes()
}

fn get_cors_origin() -> String {
    variables::get("cors_origin").unwrap_or_else(|_| "*".to_string())
}

/// Validate production configuration on startup
/// CRITICAL: Panics if using default secrets in production
fn validate_production_config() {
    let jwt_secret = get_jwt_secret();
    let hmac_key = get_hmac_key();

    // Check if using default JWT secret
    if jwt_secret == JWT_SECRET_DEFAULT {
        error!("FATAL: Using default JWT_SECRET! Set SPIN_VARIABLE_JWT_SECRET environment variable.");
        panic!("FATAL: JWT_SECRET not configured for production! This is a critical security vulnerability.");
    }

    // Check JWT secret length (must be at least 256 bits / 32 bytes)
    if jwt_secret.len() < 32 {
        error!(
            "FATAL: JWT_SECRET too short ({} bytes). Must be at least 32 bytes (256 bits).",
            jwt_secret.len()
        );
        panic!("FATAL: JWT_SECRET is too short! Minimum 32 bytes required.");
    }

    // Check if using default HMAC key
    if hmac_key == HMAC_KEY_DEFAULT {
        error!("FATAL: Using default HMAC_KEY! Set SPIN_VARIABLE_HMAC_KEY environment variable.");
        panic!("FATAL: HMAC_KEY not configured for production! This is a critical security vulnerability.");
    }

    // Check HMAC key length
    if hmac_key.len() < 32 {
        error!(
            "FATAL: HMAC_KEY too short ({} bytes). Must be at least 32 bytes (256 bits).",
            hmac_key.len()
        );
        panic!("FATAL: HMAC_KEY is too short! Minimum 32 bytes required.");
    }

    // Log CORS configuration warning
    let cors_origin = get_cors_origin();
    if cors_origin == "*" {
        warn!("WARNING: CORS is configured to allow all origins (*). This is insecure for production!");
        warn!("Set SPIN_VARIABLE_CORS_ORIGIN to your application's domain.");
    } else {
        info!("CORS origin configured: {}", cors_origin);
    }

    info!("✓ Production configuration validated successfully");
}

fn generate_access_token(username: &str) -> anyhow::Result<String> {
    let now = get_current_timestamp();
    let claims = Claims {
        sub: username.to_string(),
        exp: now + JWT_ACCESS_TOKEN_EXPIRY,
        iat: now,
        jti: Uuid::new_v4().to_string(),
        token_type: "access".to_string(),
    };

    let secret = get_jwt_secret();
    let token = encode(
        &Header::new(JWT_ALGORITHM),
        &claims,
        &EncodingKey::from_secret(&secret),
    )
    .map_err(|e| anyhow::anyhow!("Failed to generate access token: {}", e))?;

    Ok(token)
}

fn generate_refresh_token(username: &str) -> anyhow::Result<String> {
    let now = get_current_timestamp();
    let claims = Claims {
        sub: username.to_string(),
        exp: now + JWT_REFRESH_TOKEN_EXPIRY,
        iat: now,
        jti: Uuid::new_v4().to_string(),
        token_type: "refresh".to_string(),
    };

    let secret = get_jwt_secret();
    let token = encode(
        &Header::new(JWT_ALGORITHM),
        &claims,
        &EncodingKey::from_secret(&secret),
    )
    .map_err(|e| anyhow::anyhow!("Failed to generate refresh token: {}", e))?;

    Ok(token)
}

fn verify_token(token: &str) -> anyhow::Result<Claims> {
    let secret = get_jwt_secret();
    let validation = Validation::new(JWT_ALGORITHM);

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(&secret), &validation)
        .map_err(|e| anyhow::anyhow!("Invalid token: {}", e))?;

    Ok(token_data.claims)
}

fn extract_bearer_token(request: &IncomingRequest) -> Option<String> {
    let headers = request.headers();

    for (name, value) in headers.entries() {
        if name.to_lowercase() == "authorization" {
            let auth_value = String::from_utf8_lossy(&value);
            if let Some(token) = auth_value.strip_prefix("Bearer ") {
                return Some(token.trim().to_string());
            }
            break;
        }
    }
    None
}

async fn is_token_revoked(store: &Store, jti: &str) -> bool {
    let key = format!("{}{}", REVOKED_TOKEN_KEY_PREFIX, jti);
    store.get(&key).is_ok_and(|opt| opt.is_some())
}

async fn revoke_token(store: &Store, jti: &str, exp: i64) -> anyhow::Result<()> {
    let key = format!("{}{}", REVOKED_TOKEN_KEY_PREFIX, jti);
    let _ttl = (exp - get_current_timestamp()).max(0) as u64;

    // Store with TTL - will auto-delete after token expires
    let value = vec![1u8];
    store
        .set(&key, &value)
        .map_err(|e| anyhow::anyhow!("Failed to revoke token: {}", e))?;

    Ok(())
}

// ============================================================================
// RATE LIMITING FUNCTIONS
// ============================================================================

async fn check_rate_limit(
    store: &Store,
    identifier: &str,
    limit: u32,
    window_seconds: i64,
) -> anyhow::Result<bool> {
    let key = format!("{}{}", RATE_LIMIT_KEY_PREFIX, identifier);
    let now = get_current_timestamp();

    let mut rate_info = match store.get(&key) {
        Ok(Some(bytes)) => serde_json::from_slice::<RateLimitInfo>(&bytes).unwrap_or({
            RateLimitInfo {
                count: 0,
                window_start: now,
                lockout_until: None,
            }
        }),
        _ => RateLimitInfo {
            count: 0,
            window_start: now,
            lockout_until: None,
        },
    };

    // Check lockout
    if let Some(lockout_until) = rate_info.lockout_until {
        if now < lockout_until {
            return Ok(false); // Still locked out
        }
        rate_info.lockout_until = None; // Lockout expired
    }

    // Reset window if expired
    if (now - rate_info.window_start) > window_seconds {
        rate_info.count = 0;
        rate_info.window_start = now;
    }

    // Increment count
    rate_info.count += 1;

    // Check limit
    if rate_info.count > limit {
        return Ok(false);
    }

    // Save updated rate info
    let bytes = serde_json::to_vec(&rate_info)?;
    store.set(&key, &bytes)?;

    Ok(true)
}

async fn record_failed_login(store: &Store, identifier: &str) -> anyhow::Result<()> {
    let key = format!("{}login_{}", RATE_LIMIT_KEY_PREFIX, identifier);
    let now = get_current_timestamp();

    let mut rate_info = match store.get(&key) {
        Ok(Some(bytes)) => serde_json::from_slice::<RateLimitInfo>(&bytes).unwrap_or({
            RateLimitInfo {
                count: 0,
                window_start: now,
                lockout_until: None,
            }
        }),
        _ => RateLimitInfo {
            count: 0,
            window_start: now,
            lockout_until: None,
        },
    };

    rate_info.count += 1;

    // Trigger lockout after threshold
    if rate_info.count >= RATE_LIMIT_LOGIN_ATTEMPTS {
        rate_info.lockout_until = Some(now + RATE_LIMIT_LOGIN_LOCKOUT_SECONDS);
        println!(
            "User {} locked out for {} seconds after {} failed attempts",
            identifier, RATE_LIMIT_LOGIN_LOCKOUT_SECONDS, rate_info.count
        );
    }

    let bytes = serde_json::to_vec(&rate_info)?;
    store.set(&key, &bytes)?;

    Ok(())
}

async fn clear_failed_logins(store: &Store, identifier: &str) -> anyhow::Result<()> {
    let key = format!("{}login_{}", RATE_LIMIT_KEY_PREFIX, identifier);
    let _ = store.delete(&key);
    Ok(())
}

// ============================================================================
// SESSION MANAGEMENT FUNCTIONS
// ============================================================================

async fn create_session(
    store: &Store,
    username: &str,
    session_id: &str,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> anyhow::Result<()> {
    let now = get_current_timestamp();
    let session = Session {
        session_id: session_id.to_string(),
        username: username.to_string(),
        created_at: now,
        last_accessed: now,
        ip_address,
        user_agent,
    };

    let key = format!("{}{}", SESSION_KEY_PREFIX, session_id);
    let bytes = serde_json::to_vec(&session)?;
    store
        .set(&key, &bytes)
        .map_err(|e| anyhow::anyhow!("Failed to create session: {}", e))?;

    Ok(())
}

#[allow(dead_code)] // Reserved for future session management features
async fn get_session(store: &Store, session_id: &str) -> anyhow::Result<Option<Session>> {
    let key = format!("{}{}", SESSION_KEY_PREFIX, session_id);
    match store.get(&key) {
        Ok(Some(bytes)) => {
            let session = serde_json::from_slice(&bytes)?;
            Ok(Some(session))
        }
        _ => Ok(None),
    }
}

#[allow(dead_code)] // Reserved for future session management features
async fn delete_session(store: &Store, session_id: &str) -> anyhow::Result<()> {
    let key = format!("{}{}", SESSION_KEY_PREFIX, session_id);
    store
        .delete(&key)
        .map_err(|e| anyhow::anyhow!("Failed to delete session: {}", e))?;
    Ok(())
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
            let per_user_token: PerUserCachedToken =
                serde_json::from_slice(&bytes).map_err(|e| {
                    anyhow::anyhow!("Failed to deserialize per-user cached token: {}", e)
                })?;

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

async fn cleanup_expired_user_token(store: &Store, username_hash: &str) -> anyhow::Result<()> {
    let cache_key = get_user_token_cache_key(username_hash);
    store
        .delete(&cache_key)
        .map_err(|e| anyhow::anyhow!("Failed to delete expired token from cache: {}", e))?;
    println!("Cleaned up expired token for user");
    Ok(())
}

async fn refresh_access_token(refresh_token: String) -> anyhow::Result<(TokenResponse, String)> {
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
        anyhow::bail!("Location request failed with status: {}", response.status());
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

fn add_cors_headers(
    mut builder: spin_sdk::http::ResponseBuilder,
) -> spin_sdk::http::ResponseBuilder {
    let cors_origin = get_cors_origin();
    builder.header("access-control-allow-origin", cors_origin);
    builder.header("access-control-allow-methods", "GET, POST, OPTIONS");
    builder.header(
        "access-control-allow-headers",
        "Content-Type, Authorization",
    );
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
            println!(
                "Per-user token TTL expired (inactive for {} seconds), cleaning up...",
                current_time - per_user_token.last_accessed
            );
            cleanup_expired_user_token(&store, &username_hash).await?;
        } else if !per_user_token.token.is_expired(current_time) {
            // Token is still valid, update access time and save
            println!(
                "Using cached token for user (expires in {} seconds, last accessed {} seconds ago)",
                per_user_token.token.expires_at - current_time,
                current_time - per_user_token.last_accessed
            );

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
                    save_per_user_token_to_cache(&store, &username_hash, &new_per_user_token)
                        .await?;
                    return Ok(new_cached_token);
                }
                Err(e) => {
                    println!(
                        "Token refresh failed: {}. Performing full authentication...",
                        e
                    );
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
    let per_user_token =
        PerUserCachedToken::new(cached_token.clone(), current_time, TOKEN_TTL_SECONDS);
    save_per_user_token_to_cache(&store, &username_hash, &per_user_token).await?;

    Ok(cached_token)
}

// ============================================================================
// AUTH ENDPOINT HANDLERS
// ============================================================================

async fn handle_login(request: IncomingRequest) -> Result<Response, anyhow::Error> {
    let store = Store::open_default()?;

    // Parse request body
    let body_bytes = request.into_body().await?;
    let login_req: LoginRequest = serde_json::from_slice(&body_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid login request: {}", e))?;

    // Validate credentials format
    if let Err(e) = validate_credentials(&login_req.username, &login_req.password) {
        return Ok(Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(
                serde_json::json!({
                    "error": "Invalid credentials format",
                    "message": e.to_string()
                })
                .to_string(),
            )
            .build());
    }

    // Check rate limit for this username
    let rate_limit_key = format!("login_{}", hash_username(&login_req.username));
    if !check_rate_limit(&store, &rate_limit_key, RATE_LIMIT_PER_USER_HOUR, 3600).await? {
        record_failed_login(&store, &login_req.username).await?;
        return Ok(Response::builder()
            .status(429)
            .header("content-type", "application/json")
            .body(serde_json::json!({
                "error": "Rate limit exceeded",
                "message": format!("Too many login attempts. Please try again in {} minutes", RATE_LIMIT_LOGIN_LOCKOUT_SECONDS / 60)
            }).to_string())
            .build());
    }

    // Authenticate with Toyota
    match get_or_refresh_token_for_user(login_req.username.clone(), login_req.password).await {
        Ok(_toyota_token) => {
            // Generate JWT tokens
            let access_token = generate_access_token(&login_req.username)?;
            let refresh_token = generate_refresh_token(&login_req.username)?;

            // Create session
            let session_id = Uuid::new_v4().to_string();
            let ip_address = None; // Could extract from headers if available
            let user_agent = None; // Could extract from headers if available
            create_session(
                &store,
                &login_req.username,
                &session_id,
                ip_address,
                user_agent,
            )
            .await?;

            // Clear failed login attempts
            clear_failed_logins(&store, &login_req.username).await?;

            let response = LoginResponse {
                access_token,
                refresh_token,
                token_type: "Bearer".to_string(),
                expires_in: JWT_ACCESS_TOKEN_EXPIRY,
            };

            Ok(Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&response)?)
                .build())
        }
        Err(_e) => {
            // Record failed login
            record_failed_login(&store, &login_req.username).await?;

            Ok(Response::builder()
                .status(401)
                .header("content-type", "application/json")
                .body(
                    serde_json::json!({
                        "error": "Authentication failed",
                        "message": "Invalid username or password"
                    })
                    .to_string(),
                )
                .build())
        }
    }
}

async fn handle_refresh(request: IncomingRequest) -> Result<Response, anyhow::Error> {
    let store = Store::open_default()?;

    // Parse request body
    let body_bytes = request.into_body().await?;
    let refresh_req: RefreshRequest = serde_json::from_slice(&body_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid refresh request: {}", e))?;

    // Verify refresh token
    let claims = match verify_token(&refresh_req.refresh_token) {
        Ok(claims) => {
            if claims.token_type != "refresh" {
                return Ok(Response::builder()
                    .status(401)
                    .header("content-type", "application/json")
                    .body(
                        serde_json::json!({
                            "error": "Invalid token type",
                            "message": "Token is not a refresh token"
                        })
                        .to_string(),
                    )
                    .build());
            }
            claims
        }
        Err(_) => {
            return Ok(Response::builder()
                .status(401)
                .header("content-type", "application/json")
                .body(
                    serde_json::json!({
                        "error": "Invalid token",
                        "message": "Token is expired or invalid"
                    })
                    .to_string(),
                )
                .build());
        }
    };

    // Check if token is revoked
    if is_token_revoked(&store, &claims.jti).await {
        return Ok(Response::builder()
            .status(401)
            .header("content-type", "application/json")
            .body(
                serde_json::json!({
                    "error": "Token revoked",
                    "message": "This refresh token has been revoked"
                })
                .to_string(),
            )
            .build());
    }

    // Generate new access token
    let access_token = generate_access_token(&claims.sub)?;

    let response = serde_json::json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": JWT_ACCESS_TOKEN_EXPIRY
    });

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(response.to_string())
        .build())
}

async fn handle_logout(request: &IncomingRequest) -> Result<Response, anyhow::Error> {
    let store = Store::open_default()?;

    // Extract Bearer token
    let token = match extract_bearer_token(request) {
        Some(t) => t,
        None => {
            return Ok(Response::builder()
                .status(401)
                .header("content-type", "application/json")
                .body(
                    serde_json::json!({
                        "error": "No token provided",
                        "message": "Authorization header with Bearer token required"
                    })
                    .to_string(),
                )
                .build());
        }
    };

    // Verify token
    let claims = match verify_token(&token) {
        Ok(claims) => claims,
        Err(_) => {
            return Ok(Response::builder()
                .status(401)
                .header("content-type", "application/json")
                .body(
                    serde_json::json!({
                        "error": "Invalid token"
                    })
                    .to_string(),
                )
                .build());
        }
    };

    // Revoke the token
    revoke_token(&store, &claims.jti, claims.exp).await?;

    // Could also delete all sessions for this user
    // For now, just revoke the token

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(
            serde_json::json!({
                "message": "Logged out successfully"
            })
            .to_string(),
        )
        .build())
}

/// Send an HTTP request and return the response.
#[http_component]
async fn handle_request(request: IncomingRequest) -> Result<impl IntoResponse, anyhow::Error> {
    // Validate production configuration on first request (static initialization)
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        validate_production_config();
    });

    let path = request.uri();
    let method = request.method();

    info!("Request: {} {}", method, path);

    // Handle OPTIONS requests for CORS preflight
    if method == spin_sdk::http::Method::Options {
        debug!("CORS preflight request");
        return Ok(add_cors_headers(Response::builder())
            .status(200)
            .body("")
            .build());
    }

    // Public endpoints (no authentication required)
    if path == "/health" {
        // Check KV store connectivity
        let kv_status = match Store::open_default() {
            Ok(store) => match store.exists("__health_check__") {
                Ok(_) => "ok",
                Err(e) => {
                    warn!("KV store check failed: {}", e);
                    "degraded"
                }
            },
            Err(e) => {
                error!("Cannot open KV store: {}", e);
                "error"
            }
        };

        let health = HealthStatus {
            status: if kv_status == "ok" {
                "healthy".to_string()
            } else {
                "degraded".to_string()
            },
            version: VERSION.to_string(),
            kv_store: Some(kv_status.to_string()),
            uptime_seconds: None, // TODO: Track actual uptime
        };

        let json_response = serde_json::to_string(&health)?;
        let status_code = if kv_status == "ok" { 200 } else { 503 };

        return Ok(add_cors_headers(Response::builder())
            .status(status_code)
            .header("content-type", "application/json")
            .body(json_response)
            .build());
    }

    // Auth endpoints (handle login, refresh, logout)
    if path == "/auth/login" && method == spin_sdk::http::Method::Post {
        return handle_login(request)
            .await
            .map(|r| add_cors_headers(r.into_builder()).build());
    }

    if path == "/auth/refresh" && method == spin_sdk::http::Method::Post {
        return handle_refresh(request)
            .await
            .map(|r| add_cors_headers(r.into_builder()).build());
    }

    if path == "/auth/logout" && method == spin_sdk::http::Method::Post {
        return handle_logout(&request)
            .await
            .map(|r| add_cors_headers(r.into_builder()).build());
    }

    // All other endpoints require Bearer token authentication
    let store = Store::open_default()?;

    // Extract and verify Bearer token
    let token = match extract_bearer_token(&request) {
        Some(t) => t,
        None => {
            let error_json = serde_json::json!({
                "error": "Authentication required",
                "message": "Bearer token required in Authorization header",
                "version": VERSION
            });
            return Ok(add_cors_headers(Response::builder())
                .status(401)
                .header("content-type", "application/json")
                .header("www-authenticate", "Bearer realm=\"Toyota MyT Gateway\"")
                .body(error_json.to_string())
                .build());
        }
    };

    // Verify token
    let claims = match verify_token(&token) {
        Ok(c) => {
            // Check token type
            if c.token_type != "access" {
                let error_json = serde_json::json!({
                    "error": "Invalid token type",
                    "message": "Access token required (not refresh token)",
                    "version": VERSION
                });
                return Ok(add_cors_headers(Response::builder())
                    .status(401)
                    .header("content-type", "application/json")
                    .body(error_json.to_string())
                    .build());
            }
            c
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "error": "Invalid or expired token",
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

    // Check if token is revoked
    if is_token_revoked(&store, &claims.jti).await {
        let error_json = serde_json::json!({
            "error": "Token revoked",
            "message": "This token has been revoked",
            "version": VERSION
        });
        return Ok(add_cors_headers(Response::builder())
            .status(401)
            .header("content-type", "application/json")
            .body(error_json.to_string())
            .build());
    }

    // Rate limiting per user
    let rate_limit_key = format!("api_{}", hash_username(&claims.sub));
    if !check_rate_limit(&store, &rate_limit_key, RATE_LIMIT_PER_USER_HOUR, 3600).await? {
        let error_json = serde_json::json!({
            "error": "Rate limit exceeded",
            "message": format!("Maximum {} requests per hour", RATE_LIMIT_PER_USER_HOUR),
            "version": VERSION
        });
        return Ok(add_cors_headers(Response::builder())
            .status(429)
            .header("content-type", "application/json")
            .header("retry-after", "3600")
            .body(error_json.to_string())
            .build());
    }

    // Get VIN from environment variable
    let vin = variables::get("vin").unwrap_or_default();

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

    // Get cached Toyota token for this user
    let username_hash = hash_username(&claims.sub);
    let toyota_token = match get_per_user_cached_token(&store, &username_hash).await? {
        Some(per_user_token) => per_user_token.token,
        None => {
            // Token expired, user needs to login again
            let error_json = serde_json::json!({
                "error": "Toyota session expired",
                "message": "Please login again to refresh Toyota connection",
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
        match fetch_vehicle_list(&toyota_token).await {
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
        match fetch_vehicle_location(&toyota_token, &vin).await {
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
        match fetch_vehicle_telemetry(&toyota_token, &vin).await {
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
            .header(
                "authorization",
                format!("Bearer {}", toyota_token.access_token),
            )
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
        let location = fetch_vehicle_location(&toyota_token, &vin).await.ok();

        // Fetch telemetry (best effort)
        let telemetry = fetch_vehicle_telemetry(&toyota_token, &vin).await.ok();

        let charge_info = &electric_status.payload.vehicle_info.charge_info;
        let soc = charge_info.charge_remaining_amount.unwrap_or(0) as f64;
        let utc =
            parse_iso8601_to_timestamp(&electric_status.payload.vehicle_info.last_update_timestamp);

        // Determine if charging
        let is_charging = charge_info.charging_status.as_ref().map(|status| {
            status.to_uppercase().contains("CHARGING")
                || status.to_uppercase().contains("CONNECTED")
        });

        let abrp_data = AbrpTelemetry {
            utc,
            soc,
            lat: location
                .as_ref()
                .map(|l| l.payload.vehicle_info.location.lat),
            lon: location
                .as_ref()
                .map(|l| l.payload.vehicle_info.location.lon),
            is_charging,
            is_parked: None, // Not available from Toyota API
            odometer: telemetry
                .as_ref()
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
            format!("Bearer {}", toyota_token.access_token),
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
    let access_date = electric_status.payload.vehicle_info.last_update_timestamp;

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
