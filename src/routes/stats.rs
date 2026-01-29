use crate::db::EventStats;
use crate::state::SharedState;
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};

pub fn router() -> Router<SharedState> {
    Router::new().route("/stats", get(get_stats))
}

/// GET /v1/stats - Statistiques agrégées
async fn get_stats(State(state): State<SharedState>) -> Result<Json<EventStats>, StatusCode> {
    match state.event_repo.get_stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            tracing::error!(error = %e, "failed to fetch stats");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
