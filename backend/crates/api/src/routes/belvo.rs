//! Belvo Open Banking routes.
//!
//! Endpoints:
//! - GET  /belvo/token      - Get widget access token
//! - POST /belvo/webhook    - Receive Belvo webhooks
//! - GET  /belvo/links      - List user's connected banks
//! - DELETE /belvo/links/{id} - Disconnect a bank
//! - GET  /belvo/accounts   - List all accounts
//! - POST /belvo/sync/{link_id} - Trigger manual sync

use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use belvo::{webhook, BelvoClient};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use shared::{
    error::AppError,
    models::{BelvoAccount, BelvoLink, BelvoLinkStatus, NewBelvoLink},
    schema::{belvo_accounts, belvo_links},
    DbPool,
};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::extractors::AuthUser;

/// Redis queue name for Belvo sync jobs.
const BELVO_SYNC_QUEUE: &str = "gasticos:jobs:belvo_sync";

// ============================================================================
// Response DTOs
// ============================================================================

/// Widget token response.
#[derive(Debug, Serialize)]
struct WidgetTokenResponse {
    access_token: String,
}

/// Connected bank response.
#[derive(Debug, Serialize)]
struct ConnectedBankResponse {
    id: Uuid,
    institution: String,
    institution_name: String,
    status: String,
    last_synced_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<BelvoLink> for ConnectedBankResponse {
    fn from(link: BelvoLink) -> Self {
        Self {
            id: link.id,
            institution: link.institution,
            institution_name: link.institution_name,
            status: format!("{:?}", link.status).to_lowercase(),
            last_synced_at: link.last_synced_at,
            created_at: link.created_at,
        }
    }
}

/// Account response.
#[derive(Debug, Serialize)]
struct AccountResponse {
    id: Uuid,
    link_id: Uuid,
    name: Option<String>,
    number_masked: Option<String>,
    account_type: String,
    currency: String,
    balance_current: Option<f64>,
    balance_available: Option<f64>,
}

impl From<BelvoAccount> for AccountResponse {
    fn from(account: BelvoAccount) -> Self {
        Self {
            id: account.id,
            link_id: account.link_id,
            name: account.name,
            number_masked: account.number_masked,
            account_type: format!("{:?}", account.account_type).to_lowercase(),
            currency: account.currency,
            balance_current: account
                .balance_current
                .map(|b| b.to_string().parse().unwrap_or(0.0)),
            balance_available: account
                .balance_available
                .map(|b| b.to_string().parse().unwrap_or(0.0)),
        }
    }
}

/// Belvo sync job payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct BelvoSyncJobPayload {
    pub link_id: Uuid,
    pub user_id: Uuid,
    pub belvo_link_id: Uuid,
}

// ============================================================================
// Widget Token
// ============================================================================

/// Get a Belvo widget access token.
///
/// GET /api/v1/belvo/token
///
/// Returns an access token for initializing the Belvo Connect widget.
#[get("/belvo/token")]
pub async fn get_widget_token(
    _auth_user: AuthUser,
    belvo_client: web::Data<BelvoClient>,
) -> Result<impl Responder, AppError> {
    // Note: Not filtering institutions for sandbox (mock banks aren't Colombian)
    // TODO: Re-enable institution filter for production: Some(institutions::COLOMBIA_INSTITUTIONS...)
    let token = belvo_client
        .create_widget_token_with_config(
            Some("Gasticos"),
            None, // Use default logo
            None, // No institution filter (for sandbox compatibility)
            None,
            None,
            None,
        )
        .await
        .map_err(|e| {
            error!("Failed to create widget token: {}", e);
            AppError::Internal("Failed to create widget token".to_string())
        })?;

    Ok(HttpResponse::Ok().json(WidgetTokenResponse {
        access_token: token.access,
    }))
}

// ============================================================================
// Webhook
// ============================================================================

/// Handle Belvo webhook callbacks.
///
/// POST /api/v1/belvo/webhook
///
/// Receives and processes webhook events from Belvo.
#[post("/belvo/webhook")]
pub async fn handle_webhook(
    req: HttpRequest,
    body: web::Bytes,
    pool: web::Data<DbPool>,
    redis_client: web::Data<redis::Client>,
) -> Result<impl Responder, AppError> {
    // Get webhook secret from environment
    let webhook_secret = std::env::var("BELVO_WEBHOOK_SECRET").unwrap_or_default();

    // Verify signature if secret is configured
    if !webhook_secret.is_empty() {
        let signature = req
            .headers()
            .get("belvo-signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();

        if let Err(e) = webhook::verify_webhook_signature(&webhook_secret, &body, signature) {
            warn!("Invalid webhook signature: {}", e);
            return Err(AppError::Unauthorized);
        }
    }

    // Parse webhook payload
    let payload = webhook::parse_webhook_payload(&body).map_err(|e| {
        error!("Failed to parse webhook payload: {}", e);
        AppError::BadRequest("Invalid webhook payload".to_string())
    })?;

    info!("Received Belvo webhook: {:?}", payload.webhook_code);

    // Handle different webhook types
    match payload.webhook_code {
        belvo::models::WebhookEventType::LinkCreated => {
            // Link created via widget - we need to store it
            // The frontend should call our API after successful widget flow
            info!("Link created webhook received: {:?}", payload.link_id);
        }
        belvo::models::WebhookEventType::AccountsCreated => {
            // New accounts available - trigger sync
            if let Some(link_id) = payload.link_id {
                info!("Accounts created for link {}, queueing sync", link_id);
                // Queue a sync job (we'll need to look up the internal link first)
                if let Ok(mut conn) = pool.get().await {
                    if let Ok(link) = belvo_links::table
                        .filter(belvo_links::belvo_link_id.eq(link_id))
                        .first::<BelvoLink>(&mut conn)
                        .await
                    {
                        let job_payload = BelvoSyncJobPayload {
                            link_id: link.id,
                            user_id: link.user_id,
                            belvo_link_id: link_id,
                        };

                        if let Ok(job_json) = serde_json::to_string(&job_payload) {
                            if let Ok(mut redis_conn) =
                                redis_client.get_multiplexed_async_connection().await
                            {
                                let _: Result<(), _> =
                                    redis_conn.rpush(BELVO_SYNC_QUEUE, &job_json).await;
                            }
                        }
                    }
                }
            }
        }
        belvo::models::WebhookEventType::TransactionsCreated => {
            // New transactions available
            info!(
                "Transactions created webhook for link: {:?}",
                payload.link_id
            );
        }
        belvo::models::WebhookEventType::LinkDeleted => {
            // Link was deleted
            if let Some(belvo_link_id) = payload.link_id {
                info!("Link deleted: {}", belvo_link_id);
                // Update our database
                if let Ok(mut conn) = pool.get().await {
                    let _ = diesel::update(
                        belvo_links::table.filter(belvo_links::belvo_link_id.eq(belvo_link_id)),
                    )
                    .set(belvo_links::status.eq(BelvoLinkStatus::Invalid))
                    .execute(&mut conn)
                    .await;
                }
            }
        }
        _ => {
            info!("Unhandled webhook type: {:?}", payload.webhook_code);
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "received"})))
}

// ============================================================================
// Links Management
// ============================================================================

/// Register a new bank link from the widget.
///
/// POST /api/v1/belvo/links
///
/// Called by frontend after successful Belvo widget flow.
#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub link_id: Uuid,
    pub institution: String,
}

#[post("/belvo/links")]
pub async fn create_link(
    auth_user: AuthUser,
    body: web::Json<CreateLinkRequest>,
    pool: web::Data<DbPool>,
    belvo_client: web::Data<BelvoClient>,
    redis_client: web::Data<redis::Client>,
) -> Result<impl Responder, AppError> {
    // Verify the link exists in Belvo
    let belvo_link = belvo_client.get_link(body.link_id).await.map_err(|e| {
        error!("Failed to verify link with Belvo: {}", e);
        AppError::BadRequest("Invalid link ID".to_string())
    })?;

    // Get institution display name from Belvo API
    let institution_name = match belvo_client.get_institution(&body.institution).await {
        Ok(inst) => inst.display_name.unwrap_or(inst.name),
        Err(_) => body.institution.clone(),
    };

    // Check if link already exists
    let mut conn = pool.get().await?;

    let existing = belvo_links::table
        .filter(belvo_links::belvo_link_id.eq(body.link_id))
        .first::<BelvoLink>(&mut conn)
        .await
        .ok();

    if existing.is_some() {
        return Err(AppError::BadRequest("Link already exists".to_string()));
    }

    // Create the link in our database
    let new_link = NewBelvoLink {
        user_id: auth_user.user_id,
        belvo_link_id: body.link_id,
        institution: body.institution.clone(),
        institution_name,
        access_mode: format!("{:?}", belvo_link.access_mode).to_lowercase(),
        status: match belvo_link.status {
            belvo::models::LinkStatus::Valid => BelvoLinkStatus::Valid,
            belvo::models::LinkStatus::Invalid => BelvoLinkStatus::Invalid,
            belvo::models::LinkStatus::TokenRequired => BelvoLinkStatus::TokenRequired,
            belvo::models::LinkStatus::Unconfirmed => BelvoLinkStatus::Unconfirmed,
        },
    };

    let link: BelvoLink = diesel::insert_into(belvo_links::table)
        .values(&new_link)
        .returning(BelvoLink::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to create link: {}", e);
            AppError::Database(e.to_string())
        })?;

    // Queue initial sync job
    let job_payload = BelvoSyncJobPayload {
        link_id: link.id,
        user_id: auth_user.user_id,
        belvo_link_id: body.link_id,
    };

    let job_json = serde_json::to_string(&job_payload).map_err(|e| {
        error!("Failed to serialize sync job: {}", e);
        AppError::Internal("Failed to queue sync".to_string())
    })?;

    let mut redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| {
            error!("Failed to connect to Redis: {}", e);
            AppError::Internal("Failed to queue sync".to_string())
        })?;

    redis_conn
        .rpush::<_, _, ()>(BELVO_SYNC_QUEUE, &job_json)
        .await
        .map_err(|e| {
            error!("Failed to queue sync job: {}", e);
            AppError::Internal("Failed to queue sync".to_string())
        })?;

    info!(
        "Created link {} for user {} and queued initial sync",
        link.id, auth_user.user_id
    );

    Ok(HttpResponse::Created().json(ConnectedBankResponse::from(link)))
}

/// List user's connected banks.
///
/// GET /api/v1/belvo/links
#[get("/belvo/links")]
pub async fn list_links(
    auth_user: AuthUser,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let mut conn = pool.get().await?;

    let links: Vec<BelvoLink> = belvo_links::table
        .filter(belvo_links::user_id.eq(auth_user.user_id))
        .filter(belvo_links::status.ne(BelvoLinkStatus::Invalid))
        .order(belvo_links::created_at.desc())
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response: Vec<ConnectedBankResponse> = links.into_iter().map(Into::into).collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "data": response
    })))
}

/// Disconnect a bank (delete link).
///
/// DELETE /api/v1/belvo/links/{id}
#[delete("/belvo/links/{id}")]
pub async fn delete_link(
    auth_user: AuthUser,
    path: web::Path<Uuid>,
    pool: web::Data<DbPool>,
    belvo_client: web::Data<BelvoClient>,
) -> Result<impl Responder, AppError> {
    let link_id = path.into_inner();
    let mut conn = pool.get().await?;

    // Get the link to verify ownership and get Belvo link ID
    let link: BelvoLink = belvo_links::table
        .filter(belvo_links::id.eq(link_id))
        .filter(belvo_links::user_id.eq(auth_user.user_id))
        .first(&mut conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("Link not found".to_string()),
            _ => AppError::Database(e.to_string()),
        })?;

    // Delete from Belvo
    if let Err(e) = belvo_client.delete_link(link.belvo_link_id).await {
        warn!(
            "Failed to delete link from Belvo (may already be deleted): {}",
            e
        );
    }

    // Delete from our database (cascade will delete accounts)
    diesel::delete(belvo_links::table.filter(belvo_links::id.eq(link_id)))
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    info!("Deleted link {} for user {}", link_id, auth_user.user_id);

    Ok(HttpResponse::NoContent().finish())
}

// ============================================================================
// Accounts
// ============================================================================

/// List all accounts for user.
///
/// GET /api/v1/belvo/accounts
#[get("/belvo/accounts")]
pub async fn list_accounts(
    auth_user: AuthUser,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let mut conn = pool.get().await?;

    // Get user's link IDs first
    let user_link_ids: Vec<Uuid> = belvo_links::table
        .filter(belvo_links::user_id.eq(auth_user.user_id))
        .select(belvo_links::id)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Get accounts for those links
    let accounts: Vec<BelvoAccount> = belvo_accounts::table
        .filter(belvo_accounts::link_id.eq_any(&user_link_ids))
        .order(belvo_accounts::created_at.desc())
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response: Vec<AccountResponse> = accounts.into_iter().map(Into::into).collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "data": response
    })))
}

// ============================================================================
// Manual Sync
// ============================================================================

/// Trigger a manual sync for a link.
///
/// POST /api/v1/belvo/sync/{link_id}
#[post("/belvo/sync/{link_id}")]
pub async fn trigger_sync(
    auth_user: AuthUser,
    path: web::Path<Uuid>,
    pool: web::Data<DbPool>,
    redis_client: web::Data<redis::Client>,
) -> Result<impl Responder, AppError> {
    let link_id = path.into_inner();
    let mut conn = pool.get().await?;

    // Verify link ownership
    let link: BelvoLink = belvo_links::table
        .filter(belvo_links::id.eq(link_id))
        .filter(belvo_links::user_id.eq(auth_user.user_id))
        .first(&mut conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::NotFound("Link not found".to_string()),
            _ => AppError::Database(e.to_string()),
        })?;

    // Check link status
    if link.status != BelvoLinkStatus::Valid {
        return Err(AppError::BadRequest(
            "Link is not in valid state. Please reconnect the bank.".to_string(),
        ));
    }

    // Queue sync job
    let job_payload = BelvoSyncJobPayload {
        link_id: link.id,
        user_id: auth_user.user_id,
        belvo_link_id: link.belvo_link_id,
    };

    let job_json = serde_json::to_string(&job_payload).map_err(|e| {
        error!("Failed to serialize sync job: {}", e);
        AppError::Internal("Failed to queue sync".to_string())
    })?;

    let mut redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| {
            error!("Failed to connect to Redis: {}", e);
            AppError::Internal("Failed to queue sync".to_string())
        })?;

    redis_conn
        .rpush::<_, _, ()>(BELVO_SYNC_QUEUE, &job_json)
        .await
        .map_err(|e| {
            error!("Failed to queue sync job: {}", e);
            AppError::Internal("Failed to queue sync".to_string())
        })?;

    info!("Queued manual sync for link {}", link_id);

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "status": "sync_queued",
        "link_id": link_id
    })))
}

/// Configure Belvo routes.
pub fn belvo_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_widget_token)
        .service(handle_webhook)
        .service(create_link)
        .service(list_links)
        .service(delete_link)
        .service(list_accounts)
        .service(trigger_sync);
}
