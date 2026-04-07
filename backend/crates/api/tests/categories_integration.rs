//! Integration tests for category endpoints.

use actix_web::{http::StatusCode, test, web, App};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde_json::{json, Value};
use shared::{
    auth::JwtConfig,
    db::create_pool,
    models::AuthResponse,
    schema::{categories, refresh_tokens, users},
    DbPool,
};
use uuid::Uuid;

// Import the routes from the api crate
use api::routes::{auth_routes, category_routes};

/// Test configuration
fn test_jwt_config() -> JwtConfig {
    JwtConfig {
        secret: "test_secret_key_that_is_at_least_32_characters_long".to_string(),
        access_expiry_hours: 1,
        refresh_expiry_days: 7,
    }
}

/// Get database pool for tests
fn get_test_pool() -> DbPool {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    create_pool(&database_url)
}

/// Get Redis client for tests
fn get_test_redis() -> redis::Client {
    dotenvy::dotenv().ok();
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    redis::Client::open(redis_url).expect("Failed to create Redis client for tests")
}

/// Create test app with database pool and redis
async fn create_test_app(
    pool: DbPool,
    redis: redis::Client,
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
            .app_data(web::Data::new(redis))
            .service(
                web::scope("/api/v1")
                    .configure(auth_routes)
                    .configure(category_routes),
            ),
    )
    .await
}

/// Generate unique email for test isolation
fn unique_email() -> String {
    format!("test_cat_{}@example.com", Uuid::new_v4())
}

/// Register a user and return auth tokens
async fn register_user(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    email: &str,
) -> AuthResponse {
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(json!({
            "email": email,
            "password": "ValidPassword123",
            "name": "Test User"
        }))
        .to_request();

    let resp = test::call_service(app, req).await;
    test::read_body_json(resp).await
}

/// Clean up test user by email
async fn cleanup_user(pool: &DbPool, email: &str) {
    let mut conn = pool.get().await.expect("Failed to get connection");

    // Find user
    let user_result: Option<shared::models::User> = users::table
        .filter(users::email.eq(email.to_lowercase()))
        .first(&mut conn)
        .await
        .optional()
        .expect("Failed to query user");

    if let Some(user) = user_result {
        // Delete user's custom categories
        diesel::delete(categories::table.filter(categories::user_id.eq(user.id)))
            .execute(&mut conn)
            .await
            .ok();

        // Delete refresh tokens
        diesel::delete(refresh_tokens::table.filter(refresh_tokens::user_id.eq(user.id)))
            .execute(&mut conn)
            .await
            .ok();

        // Delete user
        diesel::delete(users::table.filter(users::id.eq(user.id)))
            .execute(&mut conn)
            .await
            .ok();
    }
}

// ============================================================================
// List Categories Tests
// ============================================================================

#[actix_web::test]
async fn test_list_categories_success() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // List categories
    let req = test::TestRequest::get()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let categories: Vec<Value> = test::read_body_json(resp).await;
    // Should have at least the 12 system categories
    assert!(categories.len() >= 12);

    // Verify system categories are present
    let has_groceries = categories.iter().any(|c| c["key"] == "groceries");
    let has_housing = categories.iter().any(|c| c["key"] == "housing");
    assert!(has_groceries);
    assert!(has_housing);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_list_categories_unauthorized() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool, redis).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/categories")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Get Single Category Tests
// ============================================================================

#[actix_web::test]
async fn test_get_category_success() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // First get the list to find a category ID
    let req = test::TestRequest::get()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let categories: Vec<Value> = test::read_body_json(resp).await;
    let category_id = categories[0]["id"].as_str().unwrap();

    // Get single category
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/categories/{}", category_id))
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let category: Value = test::read_body_json(resp).await;
    assert_eq!(category["id"], category_id);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_get_category_not_found() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Try to get non-existent category
    let fake_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/categories/{}", fake_id))
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

// ============================================================================
// Create Category Tests
// ============================================================================

#[actix_web::test]
async fn test_create_category_success() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Create category
    let req = test::TestRequest::post()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({
            "name": "My Custom Category",
            "color": "#FF5733",
            "icon": "star"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let category: Value = test::read_body_json(resp).await;
    assert_eq!(category["name"], "My Custom Category");
    assert_eq!(category["color"], "#FF5733");
    assert_eq!(category["icon"], "star");
    assert_eq!(category["is_system"], false);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_create_category_empty_name() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Create category with empty name
    let req = test::TestRequest::post()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({
            "name": "   "
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_create_category_name_too_long() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Create category with name > 100 chars
    let long_name = "a".repeat(101);
    let req = test::TestRequest::post()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({
            "name": long_name
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_create_category_invalid_color() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Create category with invalid color
    let req = test::TestRequest::post()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({
            "name": "Test Category",
            "color": "not-a-color"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_create_category_duplicate_name() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Create first category
    let req = test::TestRequest::post()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({
            "name": "Unique Category Name"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Try to create duplicate
    let req = test::TestRequest::post()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({
            "name": "unique category name"  // Case insensitive check
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

// ============================================================================
// Update Category Tests
// ============================================================================

#[actix_web::test]
async fn test_update_category_success() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Create category first
    let req = test::TestRequest::post()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({
            "name": "Original Name"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let created: Value = test::read_body_json(resp).await;
    let category_id = created["id"].as_str().unwrap();

    // Update category
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/categories/{}", category_id))
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({
            "name": "Updated Name",
            "color": "#00FF00"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let updated: Value = test::read_body_json(resp).await;
    assert_eq!(updated["name"], "Updated Name");
    assert_eq!(updated["color"], "#00FF00");

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_update_system_category_fails() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Get a system category ID
    let req = test::TestRequest::get()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let categories: Vec<Value> = test::read_body_json(resp).await;
    let system_cat = categories.iter().find(|c| c["is_system"] == true).unwrap();
    let system_id = system_cat["id"].as_str().unwrap();

    // Try to update system category
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/categories/{}", system_id))
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({
            "name": "Hacked System Category"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

// ============================================================================
// Delete Category Tests
// ============================================================================

#[actix_web::test]
async fn test_delete_category_success() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Create category first
    let req = test::TestRequest::post()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({
            "name": "To Be Deleted"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let created: Value = test::read_body_json(resp).await;
    let category_id = created["id"].as_str().unwrap();

    // Delete category
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/categories/{}", category_id))
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify it's deleted
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/categories/{}", category_id))
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_delete_system_category_fails() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Get a system category ID
    let req = test::TestRequest::get()
        .uri("/api/v1/categories")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let categories: Vec<Value> = test::read_body_json(resp).await;
    let system_cat = categories.iter().find(|c| c["is_system"] == true).unwrap();
    let system_id = system_cat["id"].as_str().unwrap();

    // Try to delete system category
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/categories/{}", system_id))
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

// ============================================================================
// Trigger Categorization Tests
// ============================================================================

#[actix_web::test]
async fn test_trigger_categorization_success() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Trigger categorization
    let req = test::TestRequest::post()
        .uri("/api/v1/categories/categorize")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["job_queued"], true);
    assert!(body["message"].as_str().unwrap().contains("queued"));

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_trigger_categorization_with_ids() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool.clone(), redis).await;
    let email = unique_email();

    // Register user
    let auth = register_user(&app, &email).await;

    // Trigger categorization with specific transaction IDs
    let fake_txn_id = Uuid::new_v4();
    let req = test::TestRequest::post()
        .uri("/api/v1/categories/categorize")
        .insert_header(("Authorization", format!("Bearer {}", auth.access_token)))
        .set_json(json!({
            "transaction_ids": [fake_txn_id],
            "batch_size": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["job_queued"], true);

    // Cleanup
    cleanup_user(&pool, &email).await;
}

#[actix_web::test]
async fn test_trigger_categorization_unauthorized() {
    let pool = get_test_pool();
    let redis = get_test_redis();
    let app = create_test_app(pool, redis).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/categories/categorize")
        .set_json(json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
