//! Integration tests for authentication endpoints.

use actix_web::{http::StatusCode, test, web, App};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde_json::{json, Value};
use shared::{
    auth::JwtConfig,
    db::create_pool,
    models::{AuthResponse, UserResponse},
    schema::{refresh_tokens, users},
    DbPool,
};
use uuid::Uuid;

// Import the routes module from the api crate
use api::routes::auth_routes;

/// Test configuration
fn test_jwt_config() -> JwtConfig {
    JwtConfig {
        secret: "test_secret_key_that_is_at_least_32_characters_long".to_string(),
        access_expiry_hours: 1,
        refresh_expiry_days: 7,
    }
}

/// Create test app with database pool
async fn create_test_app(
    pool: DbPool,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    let jwt_config = test_jwt_config();

    test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .app_data(web::Data::new(jwt_config))
            .service(web::scope("/api/v1").configure(auth_routes)),
    )
    .await
}

/// Get database pool for tests
fn get_test_pool() -> DbPool {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    create_pool(&database_url)
}

/// Generate unique email for test isolation
fn unique_email() -> String {
    format!("test_{}@example.com", Uuid::new_v4())
}

/// Clean up test user by email
async fn cleanup_user(pool: &DbPool, email: &str) {
    let mut conn = pool.get().await.expect("Failed to get connection");

    // Delete refresh tokens first (foreign key constraint)
    let user_result: Option<shared::models::User> = users::table
        .filter(users::email.eq(email))
        .first(&mut conn)
        .await
        .optional()
        .expect("Failed to query user");

    if let Some(user) = user_result {
        diesel::delete(refresh_tokens::table.filter(refresh_tokens::user_id.eq(user.id)))
            .execute(&mut conn)
            .await
            .ok();

        diesel::delete(users::table.filter(users::id.eq(user.id)))
            .execute(&mut conn)
            .await
            .ok();
    }
}

// ============================================================================
// Register Tests
// ============================================================================

#[actix_web::test]
async fn test_register_success() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;
    let email = unique_email();

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": "ValidPassword123",
            "name": "Test User"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: AuthResponse = test::read_body_json(resp).await;
    assert!(!body.access_token.is_empty());
    assert!(!body.refresh_token.is_empty());
    assert_eq!(body.user.email, email.to_lowercase());
    assert_eq!(body.user.name, "Test User");

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_register_duplicate_email() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;
    let email = unique_email();

    // First registration
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": "ValidPassword123",
            "name": "Test User"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Second registration with same email
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": "AnotherPassword123",
            "name": "Another User"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "EMAIL_EXISTS");

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_register_invalid_email() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": "invalid-email",
            "password": "ValidPassword123",
            "name": "Test User"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
}

#[actix_web::test]
async fn test_register_weak_password() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;
    let email = unique_email();

    // Too short
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": "Short1",
            "name": "Test User"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "PASSWORD_WEAK");
}

#[actix_web::test]
async fn test_register_empty_name() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;
    let email = unique_email();

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": "ValidPassword123",
            "name": "   "
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
}

// ============================================================================
// Login Tests
// ============================================================================

#[actix_web::test]
async fn test_login_success() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;
    let email = unique_email();
    let password = "ValidPassword123";

    // Register first
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": password,
            "name": "Test User"
        }))
        .to_request();

    test::call_service(&app, req).await;

    // Login
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(json!({
            "email": email,
            "password": password
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: AuthResponse = test::read_body_json(resp).await;
    assert!(!body.access_token.is_empty());
    assert!(!body.refresh_token.is_empty());
    assert_eq!(body.user.email, email.to_lowercase());

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_login_wrong_password() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;
    let email = unique_email();

    // Register first
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": "ValidPassword123",
            "name": "Test User"
        }))
        .to_request();

    test::call_service(&app, req).await;

    // Login with wrong password
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(json!({
            "email": email,
            "password": "WrongPassword123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_login_nonexistent_user() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(json!({
            "email": "nonexistent@example.com",
            "password": "SomePassword123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");
}

// ============================================================================
// Refresh Tests
// ============================================================================

#[actix_web::test]
async fn test_refresh_success() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;
    let email = unique_email();

    // Register to get tokens
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": "ValidPassword123",
            "name": "Test User"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let auth: AuthResponse = test::read_body_json(resp).await;

    // Refresh token
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .set_json(json!({
            "refresh_token": auth.refresh_token
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let new_auth: AuthResponse = test::read_body_json(resp).await;
    assert!(!new_auth.access_token.is_empty());
    assert!(!new_auth.refresh_token.is_empty());
    // Refresh tokens should always be different (rotated)
    assert_ne!(new_auth.refresh_token, auth.refresh_token);
    // Note: access tokens might be identical if generated in the same second
    // due to the same `iat` timestamp

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_refresh_invalid_token() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .set_json(json!({
            "refresh_token": "invalid.token.here"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "TOKEN_INVALID");
}

#[actix_web::test]
async fn test_refresh_revoked_token() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;
    let email = unique_email();

    // Register to get tokens
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": "ValidPassword123",
            "name": "Test User"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let auth: AuthResponse = test::read_body_json(resp).await;

    // Use refresh token once (this revokes the old one)
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .set_json(json!({
            "refresh_token": auth.refresh_token
        }))
        .to_request();

    test::call_service(&app, req).await;

    // Try to use the same refresh token again (should fail)
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .set_json(json!({
            "refresh_token": auth.refresh_token
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "TOKEN_INVALID");

    // Cleanup
    cleanup_user(&pool, &email).await;
}

// ============================================================================
// Logout Tests
// ============================================================================

#[actix_web::test]
async fn test_logout_success() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;
    let email = unique_email();

    // Register to get tokens
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": "ValidPassword123",
            "name": "Test User"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let auth: AuthResponse = test::read_body_json(resp).await;

    // Logout
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/logout")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["message"], "Successfully logged out");

    // Verify refresh token is revoked
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .set_json(json!({
            "refresh_token": auth.refresh_token
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_logout_without_token() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/logout")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_logout_invalid_token() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/logout")
        .insert_header(("Authorization", "Bearer invalid.token.here"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Me Tests
// ============================================================================

#[actix_web::test]
async fn test_me_success() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;
    let email = unique_email();

    // Register to get tokens
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": "ValidPassword123",
            "name": "Test User"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let auth: AuthResponse = test::read_body_json(resp).await;

    // Get current user
    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let user: UserResponse = test::read_body_json(resp).await;
    assert_eq!(user.email, email.to_lowercase());
    assert_eq!(user.name, "Test User");

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_me_without_token() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;

    let req = test::TestRequest::get().uri("/api/v1/auth/me").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_me_invalid_token() {
    let pool = get_test_pool();
    let app = create_test_app(pool.clone()).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header(("Authorization", "Bearer invalid.token.here"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
