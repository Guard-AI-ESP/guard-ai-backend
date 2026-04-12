use crate::models::user::Claims;
use crate::state::SharedState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

/// Middleware JWT — vérifie le header `Authorization: Bearer <token>`
///
/// Si la variable d'environnement `JWT_SECRET` n'est pas définie, un secret de
/// développement est utilisé. En production, `JWT_SECRET` doit toujours être défini.
pub async fn require_jwt(
    State(state): State<SharedState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = extract_bearer_token(req.headers())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        tracing::warn!(error = %e, "jwt validation failed");
        StatusCode::UNAUTHORIZED
    })?;

    Ok(next.run(req).await)
}

/// Extrait le token depuis `Authorization: Bearer <token>`
fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}
