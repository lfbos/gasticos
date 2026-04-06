//! Middleware for the API.
//!
//! Note: Authentication is handled via the `AuthUser` extractor in the extractors module,
//! which validates JWT tokens from the Authorization header. This approach is preferred
//! over middleware for request-level authentication in Actix Web.
