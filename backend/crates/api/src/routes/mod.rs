//! API route handlers.

pub mod auth;
pub mod belvo;
pub mod categories;
pub mod transactions;
pub mod upload;

pub use auth::auth_routes;
pub use belvo::belvo_routes;
pub use categories::category_routes;
pub use transactions::transaction_routes;
pub use upload::upload_routes;
