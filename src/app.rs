use crate::db::DbPool;
use crate::middleware::api_key::require_api_key;
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
/// Utile dans les tests pour injecter une clé API spécifique sans toucher l'environnement.
pub fn build_router_with_state(state: SharedState) -> Router {
    // Routes protégées par le middleware API key (lit la clé depuis state)
    let protected = Router::new()
        .nest("/v1", routes::protected_router())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_key,
        ));

    // Routes publiques (health) — pas de middleware
    let public = Router::new().nest("/v1", routes::public_router());

    // WebSocket — auth gérée dans le handler via query param
    let ws = Router::new().nest("/ws", routes::ws_router());

    Router::new()
        .merge(public)
        .merge(protected)
        .merge(ws)
        .with_state(state)
}
