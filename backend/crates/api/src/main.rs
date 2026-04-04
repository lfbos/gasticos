use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use diesel_async::RunQueryDsl;
use shared::{db::create_pool, DbPool};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[get("/health")]
async fn health(pool: web::Data<DbPool>) -> impl Responder {
    // Check database connectivity
    let db_status = match pool.get().await {
        Ok(mut conn) => {
            match diesel::sql_query("SELECT 1").execute(&mut conn).await {
                Ok(_) => "connected",
                Err(_) => "error",
            }
        }
        Err(_) => "disconnected",
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "gastico-api",
        "database": db_status
    }))
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "name": "Gastico API",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    info!("Connecting to database...");
    let pool = create_pool(&database_url);
    info!("Database pool created");

    info!("Starting Gastico API server at {}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .service(health)
            .service(index)
            .service(web::scope("/api/v1"))
    })
    .bind((host, port))?
    .run()
    .await
}
