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
pub fn hash_token(token: &str) -> String {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("{:x}", hasher.finish())
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
}
