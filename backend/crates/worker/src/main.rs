use shared::db::create_pool;
use tracing::info;
use tracing_subscriber::EnvFilter;

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
    let _pool = create_pool(&database_url);
    info!("Database pool created");

    info!("Connecting to Redis at {}", redis_url);

    // Placeholder: In future phases, this will:
    // 1. Connect to Redis
    // 2. Listen for jobs on queues (parsing, categorization)
    // 3. Process jobs and write results to PostgreSQL

    info!("Worker ready, waiting for jobs...");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        info!("Worker heartbeat");
    }
}
