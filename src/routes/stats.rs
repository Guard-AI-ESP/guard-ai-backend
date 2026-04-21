use crate::models::event::EventStats;
use crate::state::SharedState;
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};

pub fn router() -> Router<SharedState> {
    Router::new().route("/stats", get(get_stats))
}

/// GET /v1/stats — statistiques agrégées sur les événements
async fn get_stats(State(state): State<SharedState>) -> Result<Json<EventStats>, StatusCode> {
    state.event_repo.get_stats().await.map(Json).map_err(|e| {
        tracing::error!(error = %e, "failed to compute stats");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
