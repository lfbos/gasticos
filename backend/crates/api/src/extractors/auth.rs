//! Authentication extractor for protected routes.

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use shared::{
    auth::{validate_access_token, JwtConfig},
    error::AppError,
};
use std::future::{ready, Ready};
use uuid::Uuid;

/// Authenticated user information extracted from JWT token.
///
/// This extractor validates the JWT token from the Authorization header
/// and extracts the user ID. Use this in route handlers that require
/// authentication.
///
/// # Example
/// ```ignore
/// #[get("/protected")]
/// async fn protected_route(auth_user: AuthUser) -> impl Responder {
///     format!("Hello user {}", auth_user.user_id)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

impl FromRequest for AuthUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Get JWT config from app data
        let jwt_config = match req.app_data::<web::Data<JwtConfig>>() {
            Some(config) => config,
            None => {
                return ready(Err(AppError::Internal(
                    "JWT configuration not found".to_string(),
                )))
            }
        };

        // Extract Authorization header
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header,
            None => return ready(Err(AppError::Unauthorized)),
        };

        // Parse header value
        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => return ready(Err(AppError::Unauthorized)),
        };

        // Check for Bearer prefix
        if !auth_str.starts_with("Bearer ") {
            return ready(Err(AppError::Unauthorized));
        }

        let token = &auth_str[7..]; // Skip "Bearer "

        // Validate token
        match validate_access_token(token, jwt_config.get_ref()) {
            Ok(claims) => ready(Ok(AuthUser {
                user_id: claims.sub,
            })),
            Err(e) => ready(Err(e)),
        }
    }
}
