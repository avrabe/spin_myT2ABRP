// Standalone JWT business logic component - NO Spin dependencies!
// This can be tested with wasmtime independently

#[allow(warnings)]
mod bindings;

use bindings::exports::toyota::business_logic::jwt::{Claims, Guest};
use hmac::{Hmac, Mac};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

// JWT Token Settings (same as main app)
const JWT_ACCESS_TOKEN_EXPIRY: i64 = 900; // 15 minutes
const JWT_REFRESH_TOKEN_EXPIRY: i64 = 604800; // 7 days
const JWT_ALGORITHM: Algorithm = Algorithm::HS256;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct JwtClaims {
    sub: String,
    exp: i64,
    iat: i64,
    jti: String,
    #[serde(rename = "type")]
    token_type: String,
}

struct Component;

impl Guest for Component {
    fn generate_access_token(username: String, jwt_secret: Vec<u8>) -> Result<String, String> {
        generate_token(&username, &jwt_secret, "access", JWT_ACCESS_TOKEN_EXPIRY)
    }

    fn generate_refresh_token(username: String, jwt_secret: Vec<u8>) -> Result<String, String> {
        generate_token(&username, &jwt_secret, "refresh", JWT_REFRESH_TOKEN_EXPIRY)
    }

    fn verify_token(token: String, jwt_secret: Vec<u8>) -> Result<Claims, String> {
        let mut validation = Validation::new(JWT_ALGORITHM);
        validation.validate_exp = true;
        validation.validate_nbf = false;
        validation.required_spec_claims.clear();

        let token_data = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(&jwt_secret),
            &validation,
        )
        .map_err(|e| format!("Failed to decode token: {}", e))?;

        Ok(Claims {
            sub: token_data.claims.sub,
            exp: token_data.claims.exp,
            iat: token_data.claims.iat,
            jti: token_data.claims.jti,
            token_type: token_data.claims.token_type,
        })
    }

    fn hash_username(username: String, hmac_key: Vec<u8>) -> String {
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(&hmac_key)
            .expect("HMAC can take key of any size");
        mac.update(username.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }
}

bindings::export!(Component with_types_in bindings);

// Helper function to generate tokens
fn generate_token(
    username: &str,
    jwt_secret: &[u8],
    token_type: &str,
    expiry_seconds: i64,
) -> Result<String, String> {
    let now = get_current_timestamp();
    let exp = now + expiry_seconds;
    let jti = Uuid::new_v4().to_string();

    let claims = JwtClaims {
        sub: username.to_string(),
        exp,
        iat: now,
        jti,
        token_type: token_type.to_string(),
    };

    let header = Header::new(JWT_ALGORITHM);
    let encoding_key = EncodingKey::from_secret(jwt_secret);

    encode(&header, &claims, &encoding_key)
        .map_err(|e| format!("Failed to encode JWT: {}", e))
}

fn get_current_timestamp() -> i64 {
    // For WASI, we use a simple timestamp
    // In production, this would use wasi:clocks
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &[u8] = b"test-jwt-secret-key";
    const TEST_HMAC: &[u8] = b"test-hmac-key";

    #[test]
    fn test_generate_access_token() {
        let token = Component::generate_access_token(
            "testuser".to_string(),
            TEST_SECRET.to_vec(),
        )
        .expect("Should generate token");

        assert!(!token.is_empty());
        assert!(token.starts_with("eyJ")); // JWT header base64
    }

    #[test]
    fn test_generate_refresh_token() {
        let token = Component::generate_refresh_token(
            "testuser".to_string(),
            TEST_SECRET.to_vec(),
        )
        .expect("Should generate token");

        assert!(!token.is_empty());
        assert!(token.starts_with("eyJ"));
    }

    #[test]
    fn test_verify_valid_token() {
        let token = Component::generate_access_token(
            "testuser".to_string(),
            TEST_SECRET.to_vec(),
        )
        .expect("Should generate token");

        let claims = Component::verify_token(token, TEST_SECRET.to_vec())
            .expect("Should verify token");

        assert_eq!(claims.sub, "testuser");
        assert_eq!(claims.token_type, "access");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_verify_invalid_token() {
        let result = Component::verify_token(
            "invalid.token.here".to_string(),
            TEST_SECRET.to_vec(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_verify_token_wrong_secret() {
        let token = Component::generate_access_token(
            "testuser".to_string(),
            TEST_SECRET.to_vec(),
        )
        .expect("Should generate token");

        let wrong_secret = b"wrong-secret";
        let result = Component::verify_token(token, wrong_secret.to_vec());

        assert!(result.is_err());
    }

    #[test]
    fn test_hash_username() {
        let hash1 = Component::hash_username("testuser".to_string(), TEST_HMAC.to_vec());
        let hash2 = Component::hash_username("testuser".to_string(), TEST_HMAC.to_vec());
        let hash3 = Component::hash_username("otheruser".to_string(), TEST_HMAC.to_vec());

        // Same username should produce same hash
        assert_eq!(hash1, hash2);
        // Different username should produce different hash
        assert_ne!(hash1, hash3);
        // Hash should be hex encoded (SHA256 = 64 hex chars)
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_access_and_refresh_tokens_different() {
        let access = Component::generate_access_token(
            "testuser".to_string(),
            TEST_SECRET.to_vec(),
        )
        .expect("Should generate access token");

        let refresh = Component::generate_refresh_token(
            "testuser".to_string(),
            TEST_SECRET.to_vec(),
        )
        .expect("Should generate refresh token");

        // Tokens should be different
        assert_ne!(access, refresh);

        // Verify they have different types
        let access_claims = Component::verify_token(access, TEST_SECRET.to_vec())
            .expect("Should verify access token");
        let refresh_claims = Component::verify_token(refresh, TEST_SECRET.to_vec())
            .expect("Should verify refresh token");

        assert_eq!(access_claims.token_type, "access");
        assert_eq!(refresh_claims.token_type, "refresh");
    }
}
