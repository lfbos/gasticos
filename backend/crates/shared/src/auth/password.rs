//! Password hashing and verification using Argon2id.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::error::AppError;

/// Hash a password using Argon2id with secure defaults.
///
/// Returns the hashed password as a PHC string.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))
}

/// Verify a password against a stored hash.
///
/// Returns `Ok(true)` if the password matches, `Ok(false)` if it doesn't.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash format: {}", e)))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Validate password strength requirements.
///
/// Password must be:
/// - At least 8 characters long
/// - Contain at least one uppercase letter
/// - Contain at least one lowercase letter
/// - Contain at least one number
pub fn validate_password_strength(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::PasswordTooWeak(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(AppError::PasswordTooWeak(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(AppError::PasswordTooWeak(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err(AppError::PasswordTooWeak(
            "Password must contain at least one number".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "TestPassword123";
        let hash = hash_password(password).expect("Failed to hash password");

        assert!(hash.starts_with("$argon2"));
        assert!(verify_password(password, &hash).expect("Failed to verify password"));
        assert!(!verify_password("WrongPassword123", &hash).expect("Failed to verify password"));
    }

    #[test]
    fn test_password_validation_too_short() {
        let result = validate_password_strength("Short1");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_validation_no_uppercase() {
        let result = validate_password_strength("password123");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_validation_no_lowercase() {
        let result = validate_password_strength("PASSWORD123");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_validation_no_number() {
        let result = validate_password_strength("PasswordOnly");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_validation_valid() {
        let result = validate_password_strength("ValidPassword123");
        assert!(result.is_ok());
    }
}
