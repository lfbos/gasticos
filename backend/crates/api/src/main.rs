use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use belvo::BelvoClient;
use diesel_async::RunQueryDsl;
use shared::{auth::JwtConfig, db::create_pool, DbPool};
use tracing::info;
use tracing_subscriber::EnvFilter;
use utoipa_swagger_ui::{Config, SwaggerUi};

mod extractors;
mod middleware;
mod routes;

const OPENAPI_SPEC: &str = include_str!("../../../../shared/api-spec/openapi.yml");

#[get("/api-docs/openapi.yml")]
async fn openapi_spec() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/yaml")
        .body(OPENAPI_SPEC)
}

#[get("/docs")]
async fn docs_redirect() -> impl Responder {
    HttpResponse::Found()
        .insert_header(("Location", "/swagger-ui/index.html"))
        .finish()
}

#[get("/health")]
async fn health(pool: web::Data<DbPool>) -> impl Responder {
    // Check database connectivity
    let db_status = match pool.get().await {
        Ok(mut conn) => match diesel::sql_query("SELECT 1").execute(&mut conn).await {
            Ok(_) => "connected",
            Err(_) => "error",
        },
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
        "name": "Gasticos API",
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
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    info!("Connecting to database...");
    let pool = create_pool(&database_url);
    info!("Database pool created");

    info!("Loading JWT configuration...");
    let jwt_config = JwtConfig::from_env().expect("Failed to load JWT configuration");
    info!("JWT configuration loaded");

    info!("Connecting to Redis at {}...", redis_url);
    let redis_client =
        redis::Client::open(redis_url.as_str()).expect("Failed to create Redis client");
    info!("Redis client created");

    info!("Configuring Belvo client...");
    let belvo_client = BelvoClient::from_env().expect("Failed to configure Belvo client");
    info!("Belvo client configured for {}", belvo_client.base_url());

    info!("Starting Gasticos API server at {}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(jwt_config.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            .app_data(web::Data::new(belvo_client.clone()))
            .service(health)
            .service(index)
            .service(openapi_spec)
            .service(docs_redirect)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").config(Config::new(["/api-docs/openapi.yml"])),
            )
            .service(
                web::scope("/api/v1")
                    .configure(routes::auth_routes)
                    .configure(routes::belvo_routes),
            )
    })
    .bind((host, port))?
    .run()
    .await
}
