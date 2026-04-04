//! Database connection pool and utilities.

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;

/// Type alias for the async connection pool
pub type DbPool = Pool<AsyncPgConnection>;

/// Creates a new database connection pool
pub fn create_pool(database_url: &str) -> DbPool {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder(config)
        .max_size(10)
        .build()
        .expect("Failed to create database pool")
}
