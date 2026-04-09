use crate::db::DbPool;
use crate::middleware::jwt::require_jwt;
use crate::routes;
use crate::state::{AppState, SharedState};
use axum::{middleware, Router};
use std::sync::Arc;

/// Construit le router complet depuis les variables d'environnement
pub fn build_router(pool: DbPool) -> Router {
    let state: SharedState = Arc::new(AppState::new(pool));
    build_router_with_state(state)
}

/// Construit le router avec un état explicite.
/// Utile dans les tests pour injecter jwt_secret et api_key sans toucher l'env.
pub fn build_router_with_state(state: SharedState) -> Router {
    // Routes protégées par JWT
    let protected = Router::new()
        .nest("/v1", routes::protected_router())
        .layer(middleware::from_fn_with_state(state.clone(), require_jwt));

    // Routes publiques (health, auth/register, auth/login)
    let public = Router::new().nest("/v1", routes::public_router());

    // WebSocket — auth gérée dans le handler
    let ws = Router::new().nest("/ws", routes::ws_router());

    Router::new()
        .merge(public)
        .merge(protected)
        .merge(ws)
        .with_state(state)
}
