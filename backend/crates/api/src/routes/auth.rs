//! Authentication routes.

use actix_web::{get, post, web, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use shared::{
    auth::{
        generate_access_token, generate_refresh_token, hash_password, hash_token,
        validate_password_strength, validate_refresh_token, verify_password, JwtConfig,
    },
    error::AppError,
    models::{
        AuthResponse, LoginRequest, NewRefreshToken, NewUser, RefreshRequest, RegisterRequest,
        User, UserResponse,
    },
    schema::{refresh_tokens, users},
    DbPool,
};

use crate::extractors::AuthUser;

/// Configure auth routes
pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(refresh)
        .service(logout)
        .service(me);
}

/// Register a new user
#[post("/auth/register")]
async fn register(
    pool: web::Data<DbPool>,
    jwt_config: web::Data<JwtConfig>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate email format
    if !is_valid_email(&body.email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    // Validate password strength
    validate_password_strength(&body.password)?;

    // Validate name is not empty
    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("Name is required".to_string()));
    }

    let mut conn = pool.get().await?;

    // Check if email already exists
    let existing_user: Option<User> = users::table
        .filter(users::email.eq(&body.email.to_lowercase()))
        .first(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::Database(e.to_string()))?;

    if existing_user.is_some() {
        return Err(AppError::EmailAlreadyExists);
    }

    // Hash password
    let password_hash = hash_password(&body.password)?;

    // Create user
    let new_user = NewUser {
        email: body.email.to_lowercase(),
        password_hash,
        name: name.to_string(),
    };

    let user: User = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Generate tokens
    let access_token = generate_access_token(user.id, &jwt_config)?;
    let refresh_token = generate_refresh_token(user.id, &jwt_config)?;

    // Store refresh token hash in database
    let token_hash = hash_token(&refresh_token);
    let expires_at = Utc::now() + chrono::Duration::days(jwt_config.refresh_expiry_days);

    let new_refresh_token = NewRefreshToken {
        user_id: user.id,
        token_hash,
        expires_at,
    };

    diesel::insert_into(refresh_tokens::table)
        .values(&new_refresh_token)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Created().json(AuthResponse {
        access_token,
        refresh_token,
        user: UserResponse::from(user),
    }))
}

/// Login with email and password
#[post("/auth/login")]
async fn login(
    pool: web::Data<DbPool>,
    jwt_config: web::Data<JwtConfig>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().await?;

    // Find user by email
    let user: User = users::table
        .filter(users::email.eq(&body.email.to_lowercase()))
        .first(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or(AppError::InvalidCredentials)?;

    // Verify password
    if !verify_password(&body.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    // Generate tokens
    let access_token = generate_access_token(user.id, &jwt_config)?;
    let refresh_token = generate_refresh_token(user.id, &jwt_config)?;

    // Store refresh token hash in database
    let token_hash = hash_token(&refresh_token);
    let expires_at = Utc::now() + chrono::Duration::days(jwt_config.refresh_expiry_days);

    let new_refresh_token = NewRefreshToken {
        user_id: user.id,
        token_hash,
        expires_at,
    };

    diesel::insert_into(refresh_tokens::table)
        .values(&new_refresh_token)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token,
        refresh_token,
        user: UserResponse::from(user),
    }))
}

/// Refresh access token
#[post("/auth/refresh")]
async fn refresh(
    pool: web::Data<DbPool>,
    jwt_config: web::Data<JwtConfig>,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate refresh token
    let claims = validate_refresh_token(&body.refresh_token, &jwt_config)?;

    let mut conn = pool.get().await?;

    // Check if token is revoked
    let token_hash = hash_token(&body.refresh_token);
    let stored_token: Option<shared::models::RefreshToken> = refresh_tokens::table
        .filter(refresh_tokens::token_hash.eq(&token_hash))
        .filter(refresh_tokens::revoked_at.is_null())
        .first(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::Database(e.to_string()))?;

    if stored_token.is_none() {
        return Err(AppError::TokenInvalid);
    }

    // Get user
    let user: User = users::table
        .find(claims.sub)
        .first(&mut conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::TokenInvalid,
            _ => AppError::Database(e.to_string()),
        })?;

    // Revoke old refresh token
    diesel::update(refresh_tokens::table.filter(refresh_tokens::token_hash.eq(&token_hash)))
        .set(refresh_tokens::revoked_at.eq(Some(Utc::now())))
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Generate new tokens
    let access_token = generate_access_token(user.id, &jwt_config)?;
    let new_refresh_token = generate_refresh_token(user.id, &jwt_config)?;

    // Store new refresh token hash
    let new_token_hash = hash_token(&new_refresh_token);
    let expires_at = Utc::now() + chrono::Duration::days(jwt_config.refresh_expiry_days);

    let new_refresh_token_record = NewRefreshToken {
        user_id: user.id,
        token_hash: new_token_hash,
        expires_at,
    };

    diesel::insert_into(refresh_tokens::table)
        .values(&new_refresh_token_record)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        user: UserResponse::from(user),
    }))
}

/// Logout (revoke refresh token)
#[post("/auth/logout")]
async fn logout(pool: web::Data<DbPool>, auth_user: AuthUser) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().await?;

    // Revoke all refresh tokens for this user
    diesel::update(
        refresh_tokens::table
            .filter(refresh_tokens::user_id.eq(auth_user.user_id))
            .filter(refresh_tokens::revoked_at.is_null()),
    )
    .set(refresh_tokens::revoked_at.eq(Some(Utc::now())))
    .execute(&mut conn)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Successfully logged out"
    })))
}

/// Get current user info
#[get("/auth/me")]
async fn me(pool: web::Data<DbPool>, auth_user: AuthUser) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().await?;

    let user: User = users::table
        .find(auth_user.user_id)
        .first(&mut conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("User not found".to_string()),
            _ => AppError::Database(e.to_string()),
        })?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

/// Basic email validation
fn is_valid_email(email: &str) -> bool {
    // Simple email validation - contains @ with content before and after,
    // and at least one . in the domain part
    let email = email.trim().to_lowercase();
    if let Some(at_pos) = email.find('@') {
        let local = &email[..at_pos];
        let domain = &email[at_pos + 1..];
        !local.is_empty()
            && !domain.is_empty()
            && domain.contains('.')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.co"));
        assert!(is_valid_email("user+tag@example.org"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user@domain"));
        assert!(!is_valid_email("user@.com"));
        assert!(!is_valid_email("user@domain."));
    }
}
