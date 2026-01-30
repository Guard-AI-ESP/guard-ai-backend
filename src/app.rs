use crate::db::DbPool;
use crate::state::{AppState, SharedState};
use axum::Router;
use std::sync::Arc;

/// Construit le router avec l'état de l'application
pub fn build_router(pool: DbPool) -> Router {
    let state: SharedState = Arc::new(AppState::new(pool));

    Router::new()
        .nest("/v1", crate::routes::router_v1())
        .with_state(state)
}
