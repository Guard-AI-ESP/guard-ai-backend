pub mod auth;
pub mod devices;
pub mod events;
pub mod health;
pub mod hubs;
pub mod persons;
pub mod simulate;
pub mod stats;
pub mod ws;
pub mod ws_hub;

use crate::state::SharedState;
use axum::Router;

/// Routes publiques — pas d'authentification requise
pub fn public_router() -> Router<SharedState> {
    Router::new()
        .merge(health::router())
        .merge(auth::router())
}

/// Routes protégées par JWT
pub fn protected_router() -> Router<SharedState> {
    Router::new()
        .merge(events::router())
        .merge(persons::router())
        .merge(stats::router())
        .merge(simulate::router())
        .merge(hubs::router())
        .merge(devices::router())
}

/// Routes WebSocket — auth par query param (`?token=` côté front, `?api_key=` côté hub).
pub fn ws_router() -> Router<SharedState> {
    Router::new().merge(ws::router()).merge(ws_hub::router())
}
