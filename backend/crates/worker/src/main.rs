use belvo::BelvoClient;
use redis::AsyncCommands;
use shared::db::create_pool;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod jobs;

use jobs::{
    process_belvo_sync_job, process_categorize_job, BelvoSyncJobPayload, CategorizeJobPayload,
};

/// Redis queue name for Belvo sync jobs
const BELVO_SYNC_QUEUE: &str = "gasticos:jobs:belvo_sync";

/// Redis queue name for categorization jobs
const CATEGORIZE_QUEUE: &str = "gasticos:jobs:categorize";

/// Job processing timeout in seconds
const JOB_TIMEOUT: u64 = 300;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting Gasticos worker");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    info!("Connecting to database...");
    let pool = create_pool(&database_url);
    info!("Database pool created");

    info!("Connecting to Redis at {}", redis_url);
    let redis_client = redis::Client::open(redis_url.as_str())?;
    info!("Redis client created");

    // Configure Belvo client
    info!("Configuring Belvo client...");
    let belvo_client = BelvoClient::from_env().expect("Failed to configure Belvo client");
    info!("Belvo client configured for {}", belvo_client.base_url());

    info!("Worker ready, waiting for jobs...");

    // Start the job consumer loop
    run_consumer_loop(pool, redis_client, belvo_client).await
}

/// Main consumer loop that processes jobs from Redis queues.
async fn run_consumer_loop(
    pool: shared::DbPool,
    redis_client: redis::Client,
    belvo_client: BelvoClient,
) -> anyhow::Result<()> {
    let mut redis_conn = redis_client.get_multiplexed_async_connection().await?;

    // List of queues to monitor
    let queues = vec![BELVO_SYNC_QUEUE, CATEGORIZE_QUEUE];

    loop {
        // Use BLPOP for blocking pop with timeout on multiple queues
        // This will wait up to 5 seconds for a job from any queue
        let result: Option<(String, String)> = redis_conn
            .blpop(&queues, 5.0)
            .await
            .map_err(|e| anyhow::anyhow!("Redis error: {}", e))?;

        if let Some((queue, job_data)) = result {
            info!("Received job from queue: {}", queue);

            // Get a database connection
            let mut conn = match pool.get().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get database connection: {}", e);
                    // Re-queue the job on connection failure
                    if let Err(e) = redis_conn.rpush::<_, _, ()>(&queue, &job_data).await {
                        error!("Failed to re-queue job: {}", e);
                    }
                    continue;
                }
            };

            // Route to appropriate handler based on queue
            match queue.as_str() {
                q if q == BELVO_SYNC_QUEUE => {
                    process_belvo_sync_queue_job(&job_data, &mut conn, &belvo_client).await;
                }
                q if q == CATEGORIZE_QUEUE => {
                    process_categorize_queue_job(&job_data, &mut conn).await;
                }
                _ => {
                    error!("Unknown queue: {}", queue);
                }
            }
        }
    }
}

/// Process a job from the Belvo sync queue.
async fn process_belvo_sync_queue_job(
    job_data: &str,
    conn: &mut diesel_async::AsyncPgConnection,
    belvo_client: &BelvoClient,
) {
    match serde_json::from_str::<BelvoSyncJobPayload>(job_data) {
        Ok(payload) => {
            let job_future = process_belvo_sync_job(payload.clone(), conn, belvo_client);

            match tokio::time::timeout(std::time::Duration::from_secs(JOB_TIMEOUT), job_future)
                .await
            {
                Ok(Ok(())) => {
                    info!(
                        "Successfully processed Belvo sync job for link {}",
                        payload.link_id
                    );
                }
                Ok(Err(e)) => {
                    error!(
                        "Failed to process Belvo sync job for link {}: {}",
                        payload.link_id, e
                    );
                }
                Err(_) => {
                    error!(
                        "Belvo sync job timed out for link {} after {} seconds",
                        payload.link_id, JOB_TIMEOUT
                    );
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to parse Belvo sync job payload: {}. Data: {}",
                e, job_data
            );
        }
    }
}

/// Process a job from the categorization queue.
async fn process_categorize_queue_job(job_data: &str, conn: &mut diesel_async::AsyncPgConnection) {
    match serde_json::from_str::<CategorizeJobPayload>(job_data) {
        Ok(payload) => {
            let user_id = payload.user_id;
            let job_future = process_categorize_job(payload, conn);

            match tokio::time::timeout(std::time::Duration::from_secs(JOB_TIMEOUT), job_future)
                .await
            {
                Ok(Ok(result)) => {
                    info!(
                        "Successfully processed categorization job for user {}: {}/{} categorized",
                        user_id, result.categorized, result.total_processed
                    );
                }
                Ok(Err(e)) => {
                    error!(
                        "Failed to process categorization job for user {}: {}",
                        user_id, e
                    );
                }
                Err(_) => {
                    error!(
                        "Categorization job timed out for user {} after {} seconds",
                        user_id, JOB_TIMEOUT
                    );
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to parse categorization job payload: {}. Data: {}",
                e, job_data
            );
        }
    }
}
