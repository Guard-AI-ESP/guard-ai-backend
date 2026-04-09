use crate::state::SharedState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// Middleware qui vérifie le header `X-Api-Key` sur toutes les routes protégées.
///
/// La clé attendue est lue depuis `AppState.api_key`.
/// Si elle est `None`, l'authentification est désactivée (mode dev).
pub async fn require_api_key(
    State(state): State<SharedState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let expected = match state.api_key.as_deref() {
        Some(k) if !k.is_empty() => k,
        // Clé non configurée → auth désactivée
        _ => return Ok(next.run(req).await),
    };

    let provided = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided == expected {
        Ok(next.run(req).await)
    } else {
        tracing::warn!("unauthorized request — invalid or missing API key");
        Err(StatusCode::UNAUTHORIZED)
    }
}
