use crate::models::event::{IngestEventsRequest, IngestEventsResponse};
use axum::{routing::post, Json, Router};

pub fn router() -> Router {
    Router::new().route("/events", post(ingest_events))
}

async fn ingest_events(Json(req): Json<IngestEventsRequest>) -> Json<IngestEventsResponse> {
    // Lot 0: validation "structurelle" via serde + checks simples
    let mut accepted = 0usize;
    let mut rejected = 0usize;

    for e in req.events {
        if e.schema_version != "v1" || e.event_type.trim().is_empty() {
            rejected += 1;
            continue;
        }
        accepted += 1;
        // Lot 0: on log juste (DB viendra Lot 1)
        tracing::info!(
            event_id=%e.event_id,
            site_id=%e.site_id,
            source=?e.source,
            severity=?e.severity,
            event_type=%e.event_type,
            "event accepted"
        );
    }

    Json(IngestEventsResponse { accepted, rejected })
}
