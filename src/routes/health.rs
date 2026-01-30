use crate::state::SharedState;
use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResp {
    status: &'static str,
}

pub fn router() -> Router<SharedState> {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<HealthResp> {
    Json(HealthResp { status: "ok" })
}
