pub mod events;
pub mod health;
pub mod simulate;
pub mod stats;
pub mod ws;

use crate::{auth, state::SharedState};
use axum::{middleware, Router};

pub fn router_v1() -> Router<SharedState> {
    // Routes publiques (sans authentification)
    let public_routes = Router::new().merge(health::router());

    // Routes protégées (avec authentification)
    let protected_routes = Router::new()
        .merge(events::router())
        .merge(stats::router())
        .merge(simulate::router())
        .merge(ws::router())
        .layer(middleware::from_fn(auth::require_api_key));

    // Combiner les routes
    Router::new().merge(public_routes).merge(protected_routes)
}
