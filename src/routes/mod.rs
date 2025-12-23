pub mod events;
pub mod health;

use axum::Router;

pub fn router_v1() -> Router {
    Router::new()
        .merge(health::router())
        .merge(events::router())
}
