//! Authentication utilities for Gasticos.
//!
//! This module provides:
//! - Password hashing and verification using Argon2id
//! - JWT token generation and validation
//! - Token type definitions

pub mod jwt;
pub mod password;

pub use jwt::{
    generate_access_token, generate_random_token, generate_refresh_token, hash_token,
    validate_access_token, validate_refresh_token, validate_token, Claims, JwtConfig, TokenType,
};
pub use password::{hash_password, validate_password_strength, verify_password};
