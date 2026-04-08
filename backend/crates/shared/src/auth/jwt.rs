//! JWT token generation and validation.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

/// JWT configuration loaded from environment variables.
#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_expiry_hours: i64,
    pub refresh_expiry_days: i64,
}

impl JwtConfig {
    /// Create a new JwtConfig from environment variables.
    pub fn from_env() -> Result<Self, AppError> {
        let secret = std::env::var("JWT_SECRET")
            .map_err(|_| AppError::Internal("JWT_SECRET must be set".to_string()))?;

        if secret.len() < 32 {
            return Err(AppError::Internal(
                "JWT_SECRET must be at least 32 characters".to_string(),
            ));
        }

        let access_expiry_hours: i64 = std::env::var("JWT_ACCESS_EXPIRY_HOURS")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .map_err(|_| AppError::Internal("Invalid JWT_ACCESS_EXPIRY_HOURS".to_string()))?;

        let refresh_expiry_days: i64 = std::env::var("JWT_REFRESH_EXPIRY_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse()
            .map_err(|_| AppError::Internal("Invalid JWT_REFRESH_EXPIRY_DAYS".to_string()))?;

        Ok(Self {
            secret,
            access_expiry_hours,
            refresh_expiry_days,
        })
    }
}

/// Token type to distinguish access vs refresh tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

/// JWT claims structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Token type (access or refresh)
    pub token_type: TokenType,
    /// JWT ID (unique identifier for this token)
    pub jti: String,
}

impl Claims {
    /// Create new claims for an access token.
    pub fn new_access(user_id: Uuid, config: &JwtConfig) -> Self {
        let now = chrono::Utc::now().timestamp();
        let exp = now + (config.access_expiry_hours * 3600);

        Self {
            sub: user_id,
            exp,
            iat: now,
            token_type: TokenType::Access,
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Create new claims for a refresh token.
    pub fn new_refresh(user_id: Uuid, config: &JwtConfig) -> Self {
        let now = chrono::Utc::now().timestamp();
        let exp = now + (config.refresh_expiry_days * 24 * 3600);

        Self {
            sub: user_id,
            exp,
            iat: now,
            token_type: TokenType::Refresh,
            jti: Uuid::new_v4().to_string(),
        }
    }
}

/// Generate an access token for the given user.
pub fn generate_access_token(user_id: Uuid, config: &JwtConfig) -> Result<String, AppError> {
    let claims = Claims::new_access(user_id, config);
    encode_token(&claims, config)
}

/// Generate a refresh token for the given user.
pub fn generate_refresh_token(user_id: Uuid, config: &JwtConfig) -> Result<String, AppError> {
    let claims = Claims::new_refresh(user_id, config);
    encode_token(&claims, config)
}

/// Encode claims into a JWT string.
fn encode_token(claims: &Claims, config: &JwtConfig) -> Result<String, AppError> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to encode token: {}", e)))
}

/// Validate and decode a JWT token.
///
/// Returns the claims if valid, or an appropriate error.
pub fn validate_token(token: &str, config: &JwtConfig) -> Result<Claims, AppError> {
    let validation = Validation::default();

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => Ok(token_data.claims),
        Err(e) => {
            use jsonwebtoken::errors::ErrorKind;
            match e.kind() {
                ErrorKind::ExpiredSignature => Err(AppError::TokenExpired),
                _ => Err(AppError::TokenInvalid),
            }
        }
    }
}

/// Validate that a token is an access token.
pub fn validate_access_token(token: &str, config: &JwtConfig) -> Result<Claims, AppError> {
    let claims = validate_token(token, config)?;

    if claims.token_type != TokenType::Access {
        return Err(AppError::TokenInvalid);
    }

    Ok(claims)
}

/// Validate that a token is a refresh token.
pub fn validate_refresh_token(token: &str, config: &JwtConfig) -> Result<Claims, AppError> {
    let claims = validate_token(token, config)?;

    if claims.token_type != TokenType::Refresh {
        return Err(AppError::TokenInvalid);
    }

    Ok(claims)
}

/// Generate a random token string for refresh token storage.
pub fn generate_random_token() -> String {
    let bytes: [u8; 32] = rand::random();
    hex::encode(bytes)
}

/// Hash a token for secure storage in the database.
/// Uses SHA256 for deterministic and secure hashing.
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test_secret_key_that_is_at_least_32_chars_long".to_string(),
            access_expiry_hours: 1,
            refresh_expiry_days: 7,
        }
    }

    #[test]
    fn test_generate_and_validate_access_token() {
        let config = test_config();
        let user_id = Uuid::new_v4();

        let token = generate_access_token(user_id, &config).expect("Failed to generate token");
        let claims = validate_access_token(&token, &config).expect("Failed to validate token");

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_generate_and_validate_refresh_token() {
        let config = test_config();
        let user_id = Uuid::new_v4();

        let token = generate_refresh_token(user_id, &config).expect("Failed to generate token");
        let claims = validate_refresh_token(&token, &config).expect("Failed to validate token");

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_access_token_rejected_as_refresh() {
        let config = test_config();
        let user_id = Uuid::new_v4();

        let token = generate_access_token(user_id, &config).expect("Failed to generate token");
        let result = validate_refresh_token(&token, &config);

        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_token_rejected_as_access() {
        let config = test_config();
        let user_id = Uuid::new_v4();

        let token = generate_refresh_token(user_id, &config).expect("Failed to generate token");
        let result = validate_access_token(&token, &config);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_token_rejected() {
        let config = test_config();
        let result = validate_token("invalid.token.here", &config);

        assert!(result.is_err());
    }

    #[test]
    fn test_random_token_generation() {
        let token1 = generate_random_token();
        let token2 = generate_random_token();

        assert_ne!(token1, token2);
        assert_eq!(token1.len(), 64); // 32 bytes * 2 (hex encoding)
    }

    #[test]
    fn test_hash_token() {
        let token = "my_secret_token";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        // Same token should produce same hash (deterministic)
        assert_eq!(hash1, hash2);
        // Hash should be SHA256 hex (64 characters)
        assert_eq!(hash1.len(), 64);

        // Different tokens should produce different hashes
        let different_hash = hash_token("different_token");
        assert_ne!(hash1, different_hash);
    }

    #[test]
    fn test_claims_new_access() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let claims = Claims::new_access(user_id, &config);

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.token_type, TokenType::Access);
        assert!(claims.exp > claims.iat);
        // Access token should expire in 1 hour (3600 seconds)
        assert_eq!(claims.exp - claims.iat, 3600);
    }

    #[test]
    fn test_claims_new_refresh() {
        let config = test_config();
        let user_id = Uuid::new_v4();
        let claims = Claims::new_refresh(user_id, &config);

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.token_type, TokenType::Refresh);
        assert!(claims.exp > claims.iat);
        // Refresh token should expire in 7 days (7 * 24 * 3600 seconds)
        assert_eq!(claims.exp - claims.iat, 7 * 24 * 3600);
    }

    #[test]
    fn test_validate_token_directly() {
        let config = test_config();
        let user_id = Uuid::new_v4();

        let token = generate_access_token(user_id, &config).expect("Failed to generate token");
        let claims = validate_token(&token, &config).expect("Failed to validate token");

        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn test_token_type_serialization() {
        let access = TokenType::Access;
        let refresh = TokenType::Refresh;

        let access_json = serde_json::to_string(&access).unwrap();
        let refresh_json = serde_json::to_string(&refresh).unwrap();

        assert_eq!(access_json, "\"access\"");
        assert_eq!(refresh_json, "\"refresh\"");
    }

    // Combined into a single test to avoid race conditions with parallel test execution.
    // Environment variables are global state, so tests that modify them must run sequentially.
    #[test]
    fn test_from_env_scenarios() {
        // Scenario 1: All env vars set with custom values
        std::env::set_var(
            "JWT_SECRET",
            "test_secret_that_is_at_least_32_characters_for_security",
        );
        std::env::set_var("JWT_ACCESS_EXPIRY_HOURS", "2");
        std::env::set_var("JWT_REFRESH_EXPIRY_DAYS", "14");

        let result = JwtConfig::from_env();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(
            config.secret,
            "test_secret_that_is_at_least_32_characters_for_security"
        );
        assert_eq!(config.access_expiry_hours, 2);
        assert_eq!(config.refresh_expiry_days, 14);

        // Scenario 2: Only required env var set (uses defaults for expiry)
        std::env::set_var(
            "JWT_SECRET",
            "another_secret_that_is_at_least_32_characters_long",
        );
        std::env::remove_var("JWT_ACCESS_EXPIRY_HOURS");
        std::env::remove_var("JWT_REFRESH_EXPIRY_DAYS");

        let result = JwtConfig::from_env();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.access_expiry_hours, 1); // default
        assert_eq!(config.refresh_expiry_days, 7); // default

        // Scenario 3: Secret too short
        std::env::set_var("JWT_SECRET", "short");
        let result = JwtConfig::from_env();
        assert!(result.is_err());

        // Clean up
        std::env::remove_var("JWT_SECRET");
    }
}
