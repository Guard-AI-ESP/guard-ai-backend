use crate::models::user::{AuthResponse, Claims, LoginRequest, RegisterRequest};
use crate::state::{SharedState, JWT_EXPIRY_SECS};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
}

/// POST /v1/auth/register — crée un compte et retourne un JWT
async fn register(
    State(state): State<SharedState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Validation minimale
    if req.email.trim().is_empty() || req.password.len() < 6 {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "email requis et mot de passe >= 6 caractères",
        ));
    }

    // Vérifier unicité de l'email
    match state.user_repo.email_exists(&req.email).await {
        Ok(true) => return Err(error(StatusCode::CONFLICT, "cet email est déjà utilisé")),
        Err(e) => {
            tracing::error!(error = %e, "db error checking email");
            return Err(error(StatusCode::INTERNAL_SERVER_ERROR, "erreur serveur"));
        }
        _ => {}
    }

    // Hashage du mot de passe — opération bloquante isolée dans un thread dédié
    let password = req.password.clone();
    let hash = tokio::task::spawn_blocking(move || bcrypt::hash(password, bcrypt::DEFAULT_COST))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "spawn_blocking join error");
            error(StatusCode::INTERNAL_SERVER_ERROR, "erreur serveur")
        })?
        .map_err(|e| {
            tracing::error!(error = %e, "bcrypt hash error");
            error(StatusCode::INTERNAL_SERVER_ERROR, "erreur serveur")
        })?;

    if let Err(e) = state.user_repo.create(&req.email, &hash).await {
        tracing::error!(error = %e, "failed to create user");
        return Err(error(StatusCode::INTERNAL_SERVER_ERROR, "erreur serveur"));
    }

    let token = build_jwt(&req.email, &state.jwt_secret)?;
    tracing::info!(email = %req.email, "user registered");

    Ok(Json(AuthResponse {
        token,
        token_type: "Bearer",
        expires_in: JWT_EXPIRY_SECS,
    }))
}

/// POST /v1/auth/login — vérifie les credentials et retourne un JWT
async fn login(
    State(state): State<SharedState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = state
        .user_repo
        .find_by_email(&req.email)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "db error finding user");
            error(StatusCode::INTERNAL_SERVER_ERROR, "erreur serveur")
        })?
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "identifiants invalides"))?;

    // Vérification du mot de passe — opération bloquante
    let password = req.password.clone();
    let hash = user.password_hash.clone();
    let valid = tokio::task::spawn_blocking(move || bcrypt::verify(password, &hash))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "spawn_blocking join error");
            error(StatusCode::INTERNAL_SERVER_ERROR, "erreur serveur")
        })?
        .map_err(|e| {
            tracing::error!(error = %e, "bcrypt verify error");
            error(StatusCode::INTERNAL_SERVER_ERROR, "erreur serveur")
        })?;

    if !valid {
        return Err(error(StatusCode::UNAUTHORIZED, "identifiants invalides"));
    }

    let token = build_jwt(&user.email, &state.jwt_secret)?;
    tracing::info!(email = %user.email, "user logged in");

    Ok(Json(AuthResponse {
        token,
        token_type: "Bearer",
        expires_in: JWT_EXPIRY_SECS,
    }))
}

/// Construit un JWT signé pour l'email donné
fn build_jwt(email: &str, secret: &str) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let claims = Claims {
        sub: email.to_string(),
        iat: now,
        exp: now + JWT_EXPIRY_SECS,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!(error = %e, "jwt encode error");
        error(StatusCode::INTERNAL_SERVER_ERROR, "erreur serveur")
    })
}

/// Construit une réponse d'erreur JSON uniforme
fn error(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message })))
}
