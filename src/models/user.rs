use serde::{Deserialize, Serialize};

/// Utilisateur stocké en base
#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
}

/// Corps de POST /v1/auth/register
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// Corps de POST /v1/auth/login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Réponse JWT renvoyée après login ou register
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: &'static str,
    /// Durée de validité en secondes
    pub expires_in: u64,
}

/// Claims encodés dans le JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject = email de l'utilisateur
    pub sub: String,
    /// Issued at (Unix timestamp)
    pub iat: u64,
    /// Expiration (Unix timestamp)
    pub exp: u64,
}
