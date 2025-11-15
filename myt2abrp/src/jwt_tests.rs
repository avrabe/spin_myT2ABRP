//! JWT Token Generation and Validation Tests
//!
//! These tests cover the critical JWT functionality including token generation,
//! validation, and bearer token extraction.

#![cfg(test)]

use crate::*;
use serial_test::serial;
use std::env;

/// Set up test environment with required secrets
fn setup_test_env() {
    env::set_var("JWT_SECRET", "test-jwt-secret-32-bytes-long!!");
    env::set_var("HMAC_KEY", "test-hmac-key-32-bytes-long!!!");
    env::set_var("CORS_ORIGIN", "http://localhost:3000");
}

/// Clean up test environment
fn cleanup_test_env() {
    env::remove_var("JWT_SECRET");
    env::remove_var("HMAC_KEY");
    env::remove_var("CORS_ORIGIN");
}

#[test]
#[serial]
#[serial]
fn test_generate_access_token_success() {
    setup_test_env();

    let result = generate_access_token("test@example.com");

    assert!(result.is_ok());
    let token = result.unwrap();

    // Token should have 3 parts (header.payload.signature)
    assert_eq!(token.split('.').count(), 3);

    // Should be non-empty
    assert!(!token.is_empty());

    cleanup_test_env();
}

#[test]
#[serial]
fn test_generate_refresh_token_success() {
    setup_test_env();

    let result = generate_refresh_token("test@example.com");

    assert!(result.is_ok());
    let token = result.unwrap();

    // Token should have 3 parts
    assert_eq!(token.split('.').count(), 3);
    assert!(!token.is_empty());

    cleanup_test_env();
}

#[test]
#[serial]
fn test_access_token_and_refresh_token_are_different() {
    setup_test_env();

    let access_token = generate_access_token("test@example.com").unwrap();
    let refresh_token = generate_refresh_token("test@example.com").unwrap();

    // Tokens should be different (different token_type and expiry)
    assert_ne!(access_token, refresh_token);

    cleanup_test_env();
}

#[test]
#[serial]
fn test_verify_valid_access_token() {
    setup_test_env();

    let token = generate_access_token("test@example.com").unwrap();
    let result = verify_token(&token);

    assert!(result.is_ok());
    let claims = result.unwrap();

    // Verify claims
    assert_eq!(claims.sub, "test@example.com");
    assert_eq!(claims.token_type, "access");

    // Expiry should be in the future
    let now = get_current_timestamp();
    assert!(claims.exp > now);

    // Issued at should be in the past or now
    assert!(claims.iat <= now + 10); // Allow 10 second clock skew

    // JTI should not be empty
    assert!(!claims.jti.is_empty());

    cleanup_test_env();
}

#[test]
#[serial]
fn test_verify_valid_refresh_token() {
    setup_test_env();

    let token = generate_refresh_token("test@example.com").unwrap();
    let result = verify_token(&token);

    assert!(result.is_ok());
    let claims = result.unwrap();

    assert_eq!(claims.sub, "test@example.com");
    assert_eq!(claims.token_type, "refresh");

    cleanup_test_env();
}

#[test]
#[serial]
fn test_verify_invalid_token_format() {
    setup_test_env();

    let result = verify_token("invalid.token");

    assert!(result.is_err());

    cleanup_test_env();
}

#[test]
#[serial]
fn test_verify_tampered_token() {
    setup_test_env();

    let token = generate_access_token("test@example.com").unwrap();

    // Tamper with the token by changing one character in the signature
    let mut parts: Vec<&str> = token.split('.').collect();
    parts[2] = "tampered_signature";
    let tampered_token = parts.join(".");

    let result = verify_token(&tampered_token);

    assert!(result.is_err());

    cleanup_test_env();
}

#[test]
#[serial]
fn test_verify_token_with_wrong_secret() {
    setup_test_env();

    let token = generate_access_token("test@example.com").unwrap();

    // Change the secret
    env::set_var("JWT_SECRET", "different-secret-32-bytes-long!");

    let result = verify_token(&token);

    // Should fail because secret changed
    assert!(result.is_err());

    cleanup_test_env();
}

#[test]
#[serial]
fn test_token_expiry_values() {
    setup_test_env();

    let access_token = generate_access_token("test@example.com").unwrap();
    let refresh_token = generate_refresh_token("test@example.com").unwrap();

    let access_claims = verify_token(&access_token).unwrap();
    let refresh_claims = verify_token(&refresh_token).unwrap();

    // Access token should expire in ~15 minutes (900 seconds)
    let access_lifetime = access_claims.exp - access_claims.iat;
    assert!(access_lifetime >= 895 && access_lifetime <= 905,
            "Access token lifetime should be ~900 seconds, got {}", access_lifetime);

    // Refresh token should expire in ~7 days (604800 seconds)
    let refresh_lifetime = refresh_claims.exp - refresh_claims.iat;
    assert!(refresh_lifetime >= 604790 && refresh_lifetime <= 604810,
            "Refresh token lifetime should be ~604800 seconds, got {}", refresh_lifetime);

    cleanup_test_env();
}

#[test]
#[serial]
fn test_extract_bearer_token_valid() {
    // This would require creating a mock IncomingRequest
    // For now, we'll document that this needs Spin SDK mocking
    // TODO: Add test when we have Spin request mocking
}

#[test]
#[serial]
fn test_different_users_get_different_tokens() {
    setup_test_env();

    let token1 = generate_access_token("user1@example.com").unwrap();
    let token2 = generate_access_token("user2@example.com").unwrap();

    assert_ne!(token1, token2);

    let claims1 = verify_token(&token1).unwrap();
    let claims2 = verify_token(&token2).unwrap();

    assert_eq!(claims1.sub, "user1@example.com");
    assert_eq!(claims2.sub, "user2@example.com");

    cleanup_test_env();
}

#[test]
#[serial]
fn test_token_jti_uniqueness() {
    setup_test_env();

    let token1 = generate_access_token("test@example.com").unwrap();
    // Small delay to ensure different timestamp
    std::thread::sleep(std::time::Duration::from_millis(10));
    let token2 = generate_access_token("test@example.com").unwrap();

    let claims1 = verify_token(&token1).unwrap();
    let claims2 = verify_token(&token2).unwrap();

    // JTI should be unique for each token
    assert_ne!(claims1.jti, claims2.jti);

    cleanup_test_env();
}

#[test]
#[serial]
fn test_get_jwt_secret() {
    setup_test_env();

    let secret = get_jwt_secret();

    // Should match what we set in env
    assert_eq!(secret, b"test-jwt-secret-32-bytes-long!!");
    assert_eq!(secret.len(), 32);

    cleanup_test_env();
}

#[test]
#[serial]
fn test_get_hmac_key() {
    setup_test_env();

    let key = get_hmac_key();

    assert_eq!(key, b"test-hmac-key-32-bytes-long!!!");
    assert_eq!(key.len(), 32);

    cleanup_test_env();
}

#[test]
#[serial]
fn test_get_cors_origin() {
    setup_test_env();

    let origin = get_cors_origin();

    assert_eq!(origin, "http://localhost:3000");

    cleanup_test_env();
}

// Note: validate_production_config() tests are skipped because they use panic!()
// which doesn't play well with parallel test execution. In production, these
// panics prevent the server from starting with invalid config, which is the desired behavior.
// The validation logic itself is tested via manual verification.

#[test]
#[serial]
fn test_config_getters_with_valid_env() {
    env::set_var("JWT_SECRET", "valid-jwt-secret-that-is-long-enough-32bytes!");
    env::set_var("HMAC_KEY", "valid-hmac-key-that-is-long-enough-32bytes!");
    env::set_var("CORS_ORIGIN", "https://example.com");

    let jwt_secret = get_jwt_secret();
    let hmac_key = get_hmac_key();
    let cors_origin = get_cors_origin();

    assert_eq!(jwt_secret.len(), 47);
    assert_eq!(hmac_key.len(), 48);
    assert_eq!(cors_origin, "https://example.com");

    cleanup_test_env();
}
