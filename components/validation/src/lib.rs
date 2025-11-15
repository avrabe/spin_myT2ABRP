// Input Validation Utilities
//
// Pure validation logic for user credentials and data inputs.
// Zero dependencies on Spin SDK or any platform-specific APIs.

#[allow(warnings)]
mod bindings;

use bindings::exports::toyota::validation::validator::{Guest, ValidationConfig};

// =============================================================================
// Constants
// =============================================================================

const DEFAULT_MAX_USERNAME_LENGTH: u32 = 256;
const DEFAULT_MAX_PASSWORD_LENGTH: u32 = 256;

// =============================================================================
// Validation Functions
// =============================================================================

/// Validate user credentials (username and password)
fn validate_credentials_internal(
    username: &str,
    password: &str,
    config: &ValidationConfig,
) -> Result<(), String> {
    // Validate username length
    if username.is_empty() {
        return Err("Username cannot be empty".to_string());
    }
    if username.len() > config.max_username_length as usize {
        return Err(format!(
            "Username exceeds maximum length of {} characters",
            config.max_username_length
        ));
    }

    // Validate password length
    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }
    if password.len() > config.max_password_length as usize {
        return Err(format!(
            "Password exceeds maximum length of {} characters",
            config.max_password_length
        ));
    }

    // Basic format validation for username (should look like an email)
    if !is_valid_email_internal(username) {
        return Err("Username must be a valid email address".to_string());
    }

    Ok(())
}

/// Validate email format (basic check)
fn is_valid_email_internal(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

/// Validate string length is within bounds
fn validate_string_length_internal(
    value: &str,
    field_name: &str,
    min_length: u32,
    max_length: u32,
) -> Result<(), String> {
    let len = value.len() as u32;

    if len < min_length {
        return Err(format!(
            "{} must be at least {} characters long (got {})",
            field_name, min_length, len
        ));
    }

    if len > max_length {
        return Err(format!(
            "{} must be at most {} characters long (got {})",
            field_name, max_length, len
        ));
    }

    Ok(())
}

/// Check if string is empty
fn is_empty_internal(value: &str) -> bool {
    value.is_empty()
}

// =============================================================================
// WIT Interface Implementation
// =============================================================================

struct Component;

impl Guest for Component {
    fn get_default_config() -> ValidationConfig {
        ValidationConfig {
            max_username_length: DEFAULT_MAX_USERNAME_LENGTH,
            max_password_length: DEFAULT_MAX_PASSWORD_LENGTH,
        }
    }

    fn validate_credentials(
        username: String,
        password: String,
        config: ValidationConfig,
    ) -> Result<(), String> {
        validate_credentials_internal(&username, &password, &config)
    }

    fn is_valid_email(email: String) -> bool {
        is_valid_email_internal(&email)
    }

    fn validate_string_length(
        value: String,
        field_name: String,
        min_length: u32,
        max_length: u32,
    ) -> Result<(), String> {
        validate_string_length_internal(&value, &field_name, min_length, max_length)
    }

    fn is_empty(value: String) -> bool {
        is_empty_internal(&value)
    }
}

bindings::export!(Component with_types_in bindings);

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_config() -> ValidationConfig {
        ValidationConfig {
            max_username_length: DEFAULT_MAX_USERNAME_LENGTH,
            max_password_length: DEFAULT_MAX_PASSWORD_LENGTH,
        }
    }

    #[test]
    fn test_validate_credentials_success() {
        let config = get_test_config();
        let result = validate_credentials_internal("test@example.com", "password123", &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_credentials_empty_username() {
        let config = get_test_config();
        let result = validate_credentials_internal("", "password123", &config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_lowercase().contains("empty"));
        assert!(err.to_lowercase().contains("username"));
    }

    #[test]
    fn test_validate_credentials_empty_password() {
        let config = get_test_config();
        let result = validate_credentials_internal("test@example.com", "", &config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_lowercase().contains("empty"));
        assert!(err.to_lowercase().contains("password"));
    }

    #[test]
    fn test_validate_credentials_invalid_email() {
        let config = get_test_config();
        let result = validate_credentials_internal("not-an-email", "password123", &config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_lowercase().contains("email"));
    }

    #[test]
    fn test_validate_credentials_too_long_username() {
        let config = get_test_config();
        let long_username = "a".repeat(300) + "@test.com";
        let result = validate_credentials_internal(&long_username, "password123", &config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("maximum length"));
        assert!(err.contains("Username"));
    }

    #[test]
    fn test_validate_credentials_too_long_password() {
        let config = get_test_config();
        let long_password = "a".repeat(300);
        let result = validate_credentials_internal("test@example.com", &long_password, &config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("maximum length"));
        assert!(err.contains("Password"));
    }

    #[test]
    fn test_is_valid_email_valid() {
        assert!(is_valid_email_internal("test@example.com"));
        assert!(is_valid_email_internal("user.name@domain.co.uk"));
        assert!(is_valid_email_internal("a@b.c"));
    }

    #[test]
    fn test_is_valid_email_invalid() {
        assert!(!is_valid_email_internal("not-an-email"));
        assert!(!is_valid_email_internal("no-at-sign.com"));
        assert!(!is_valid_email_internal("no-dot@domain"));
        assert!(!is_valid_email_internal(""));
    }

    #[test]
    fn test_validate_string_length_success() {
        let result = validate_string_length_internal("hello", "field", 1, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_string_length_too_short() {
        let result = validate_string_length_internal("hi", "field", 5, 10);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("at least 5"));
    }

    #[test]
    fn test_validate_string_length_too_long() {
        let result = validate_string_length_internal("hello world", "field", 1, 5);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("at most 5"));
    }

    #[test]
    fn test_is_empty() {
        assert!(is_empty_internal(""));
        assert!(!is_empty_internal("not empty"));
        assert!(!is_empty_internal(" "));
    }

    #[test]
    fn test_custom_config() {
        let config = ValidationConfig {
            max_username_length: 10,
            max_password_length: 8,
        };

        // Valid within custom limits
        let result = validate_credentials_internal("a@b.c", "pass", &config);
        assert!(result.is_ok());

        // Too long for custom limits
        let result =
            validate_credentials_internal("verylongemail@example.com", "password", &config);
        assert!(result.is_err());
    }
}
