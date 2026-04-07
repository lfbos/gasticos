//! Category management routes.

use actix_web::{delete, get, post, put, web, HttpResponse};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use shared::{error::AppError, models::Category, schema::categories, DbPool};
use uuid::Uuid;

use crate::extractors::AuthUser;

/// Redis queue name for categorization jobs
const CATEGORIZE_QUEUE: &str = "gasticos:jobs:categorize";

/// Configure category routes
pub fn category_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_categories)
        .service(get_category)
        .service(create_category)
        .service(update_category)
        .service(delete_category)
        .service(trigger_categorization);
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Response for a single category
#[derive(Debug, Clone, Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub key: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_system: bool,
}

impl From<Category> for CategoryResponse {
    fn from(cat: Category) -> Self {
        Self {
            id: cat.id,
            name: cat.name,
            key: cat.key,
            icon: cat.icon,
            color: cat.color,
            is_system: cat.is_system,
        }
    }
}

/// Request to create a new category
#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// Request to update a category
#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// Request to trigger categorization
#[derive(Debug, Deserialize)]
pub struct TriggerCategorizationRequest {
    /// Optional: specific transaction IDs to categorize.
    /// If empty, will categorize all uncategorized transactions.
    #[serde(default)]
    pub transaction_ids: Vec<Uuid>,
    /// Maximum number of transactions to process in one batch.
    #[serde(default = "default_batch_size")]
    pub batch_size: i64,
}

fn default_batch_size() -> i64 {
    500
}

/// Response for categorization trigger
#[derive(Debug, Serialize)]
pub struct TriggerCategorizationResponse {
    pub message: String,
    pub job_queued: bool,
}

// ============================================================================
// Route Handlers
// ============================================================================

/// List all categories for the current user (system + user-specific)
#[get("/categories")]
async fn list_categories(
    pool: web::Data<DbPool>,
    auth_user: AuthUser,
) -> Result<HttpResponse, AppError> {
    let mut conn = pool.get().await?;

    let cats: Vec<Category> = categories::table
        .filter(
            categories::is_system
                .eq(true)
                .or(categories::user_id.eq(auth_user.user_id)),
        )
        .order(categories::is_system.desc())
        .then_order_by(categories::name.asc())
        .select(Category::as_select())
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response: Vec<CategoryResponse> = cats.into_iter().map(CategoryResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

/// Get a single category by ID
#[get("/categories/{id}")]
async fn get_category(
    pool: web::Data<DbPool>,
    auth_user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let category_id = path.into_inner();
    let mut conn = pool.get().await?;

    let cat: Category = categories::table
        .filter(categories::id.eq(category_id))
        .filter(
            categories::is_system
                .eq(true)
                .or(categories::user_id.eq(auth_user.user_id)),
        )
        .select(Category::as_select())
        .first(&mut conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("Category not found".to_string()),
            _ => AppError::Database(e.to_string()),
        })?;

    Ok(HttpResponse::Ok().json(CategoryResponse::from(cat)))
}

/// Create a new custom category
#[post("/categories")]
async fn create_category(
    pool: web::Data<DbPool>,
    auth_user: AuthUser,
    body: web::Json<CreateCategoryRequest>,
) -> Result<HttpResponse, AppError> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest(
            "Category name is required".to_string(),
        ));
    }

    if name.len() > 100 {
        return Err(AppError::BadRequest(
            "Category name must be 100 characters or less".to_string(),
        ));
    }

    // Validate color format if provided
    if let Some(ref color) = body.color {
        if !is_valid_color(color) {
            return Err(AppError::BadRequest(
                "Invalid color format. Use hex format like #FF0000".to_string(),
            ));
        }
    }

    let mut conn = pool.get().await?;

    // Check if user already has a category with this name (case-insensitive)
    let user_categories: Vec<Category> = categories::table
        .filter(categories::user_id.eq(auth_user.user_id))
        .select(Category::as_select())
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let name_lower = name.to_lowercase();
    let existing = user_categories
        .iter()
        .find(|c| c.name.to_lowercase() == name_lower);

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "A category with this name already exists".to_string(),
        ));
    }

    // Generate a key from the name
    let key = name.to_lowercase().replace(' ', "_");

    let new_category = shared::models::NewCategory {
        user_id: Some(auth_user.user_id),
        name: name.to_string(),
        key: Some(key),
        icon: body.icon.clone(),
        color: body.color.clone(),
        is_system: false,
    };

    let cat: Category = diesel::insert_into(categories::table)
        .values(&new_category)
        .get_result(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Created().json(CategoryResponse::from(cat)))
}

/// Update a custom category
#[put("/categories/{id}")]
async fn update_category(
    pool: web::Data<DbPool>,
    auth_user: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateCategoryRequest>,
) -> Result<HttpResponse, AppError> {
    let category_id = path.into_inner();
    let mut conn = pool.get().await?;

    // Find the category
    let cat: Category = categories::table
        .filter(categories::id.eq(category_id))
        .filter(categories::user_id.eq(auth_user.user_id))
        .filter(categories::is_system.eq(false))
        .select(Category::as_select())
        .first(&mut conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                AppError::NotFound("Category not found or is a system category".to_string())
            }
            _ => AppError::Database(e.to_string()),
        })?;

    // Validate updates
    if let Some(ref name) = body.name {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Category name cannot be empty".to_string(),
            ));
        }
        if name.len() > 100 {
            return Err(AppError::BadRequest(
                "Category name must be 100 characters or less".to_string(),
            ));
        }
    }

    if let Some(ref color) = body.color {
        if !is_valid_color(color) {
            return Err(AppError::BadRequest(
                "Invalid color format. Use hex format like #FF0000".to_string(),
            ));
        }
    }

    // Build the update
    let new_name = body.name.as_ref().map(|n| n.trim().to_string());
    let new_key = new_name
        .as_ref()
        .map(|n| n.to_lowercase().replace(' ', "_"));

    let updated_cat: Category = diesel::update(categories::table.filter(categories::id.eq(cat.id)))
        .set((
            new_name.map(|n| categories::name.eq(n)),
            new_key.map(|k| categories::key.eq(Some(k))),
            body.icon.clone().map(|i| categories::icon.eq(Some(i))),
            body.color.clone().map(|c| categories::color.eq(Some(c))),
        ))
        .get_result(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(CategoryResponse::from(updated_cat)))
}

/// Delete a custom category
#[delete("/categories/{id}")]
async fn delete_category(
    pool: web::Data<DbPool>,
    auth_user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let category_id = path.into_inner();
    let mut conn = pool.get().await?;

    // Ensure the category belongs to the user and is not a system category
    let cat: Category = categories::table
        .filter(categories::id.eq(category_id))
        .filter(categories::user_id.eq(auth_user.user_id))
        .filter(categories::is_system.eq(false))
        .select(Category::as_select())
        .first(&mut conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                AppError::NotFound("Category not found or is a system category".to_string())
            }
            _ => AppError::Database(e.to_string()),
        })?;

    // Delete the category
    diesel::delete(categories::table.filter(categories::id.eq(cat.id)))
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Category deleted successfully"
    })))
}

/// Trigger categorization of uncategorized transactions
#[post("/categories/categorize")]
async fn trigger_categorization(
    redis: web::Data<redis::Client>,
    auth_user: AuthUser,
    body: web::Json<TriggerCategorizationRequest>,
) -> Result<HttpResponse, AppError> {
    let mut redis_conn = redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AppError::Internal(format!("Redis connection error: {}", e)))?;

    // Create job payload
    let job_payload = serde_json::json!({
        "user_id": auth_user.user_id,
        "transaction_ids": body.transaction_ids,
        "batch_size": body.batch_size
    });

    // Queue the job
    redis_conn
        .rpush::<_, _, ()>(CATEGORIZE_QUEUE, job_payload.to_string())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to queue categorization job: {}", e)))?;

    Ok(HttpResponse::Ok().json(TriggerCategorizationResponse {
        message: "Categorization job queued successfully".to_string(),
        job_queued: true,
    }))
}

/// Validate hex color format
fn is_valid_color(color: &str) -> bool {
    if !color.starts_with('#') {
        return false;
    }
    let hex = &color[1..];
    hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_colors() {
        assert!(is_valid_color("#FF0000"));
        assert!(is_valid_color("#00ff00"));
        assert!(is_valid_color("#123ABC"));
    }

    #[test]
    fn test_invalid_colors() {
        assert!(!is_valid_color("FF0000")); // Missing #
        assert!(!is_valid_color("#FFF")); // Too short
        assert!(!is_valid_color("#GGGGGG")); // Invalid hex
        assert!(!is_valid_color("#FF00000")); // Too long
    }

    #[test]
    fn test_category_response_from() {
        let cat = Category {
            id: Uuid::new_v4(),
            user_id: None,
            name: "Test".to_string(),
            key: Some("test".to_string()),
            icon: Some("icon".to_string()),
            color: Some("#FF0000".to_string()),
            is_system: true,
            created_at: chrono::Utc::now(),
        };

        let response = CategoryResponse::from(cat.clone());
        assert_eq!(response.id, cat.id);
        assert_eq!(response.name, "Test");
        assert!(response.is_system);
    }

    #[test]
    fn test_create_category_request() {
        let json = r##"{"name": "Test Category", "color": "#FF0000"}"##;
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Test Category");
        assert_eq!(req.color, Some("#FF0000".to_string()));
        assert!(req.icon.is_none());
    }

    #[test]
    fn test_trigger_categorization_request_defaults() {
        let json = r#"{}"#;
        let req: TriggerCategorizationRequest = serde_json::from_str(json).unwrap();
        assert!(req.transaction_ids.is_empty());
        assert_eq!(req.batch_size, 500);
    }
}
