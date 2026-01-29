pub mod events;
pub mod health;

use crate::state::SharedState;
use axum::Router;

pub fn router_v1() -> Router<SharedState> {
    Router::new()
        .merge(health::router())
        .merge(events::router())
}
