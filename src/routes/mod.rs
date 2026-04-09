pub mod events;
pub mod health;
pub mod simulate;
pub mod stats;
pub mod ws;

use crate::state::SharedState;
use axum::Router;

/// Routes publiques (pas d'authentification requise)
pub fn public_router() -> Router<SharedState> {
    Router::new().merge(health::router())
}

/// Routes protégées par API key
pub fn protected_router() -> Router<SharedState> {
    Router::new()
        .merge(events::router())
        .merge(stats::router())
        .merge(simulate::router())
}

/// Routes WebSocket (auth par query param)
pub fn ws_router() -> Router<SharedState> {
    ws::router()
}
