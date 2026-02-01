pub mod events;
pub mod health;
pub mod simulate;
pub mod stats;
pub mod ws;

use crate::state::SharedState;
use axum::Router;

pub fn router_v1() -> Router<SharedState> {
    Router::new()
        .merge(health::router())
        .merge(events::router())
        .merge(stats::router())
        .merge(simulate::router())
        .merge(ws::router())
}
