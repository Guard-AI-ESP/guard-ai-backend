use crate::db::repository::EventFilter;
use crate::models::event::{
    EventQueryParams, EventResponse, EventsListResponse, IngestEventsRequest, IngestEventsResponse,
};
use crate::state::SharedState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/events", post(ingest_events).get(list_events))
        .route("/events/:id", get(get_event))
}

/// POST /v1/events - Ingestion par lots
async fn ingest_events(
    State(state): State<SharedState>,
    Json(req): Json<IngestEventsRequest>,
) -> Json<IngestEventsResponse> {
    let mut to_insert = Vec::new();
    let mut rejected = 0usize;

    // Validation
    for e in req.events {
        if e.schema_version != "v1" || e.event_type.trim().is_empty() {
            rejected += 1;
            continue;
        }
        to_insert.push(e);
    }

    // Insertion en batch
    let accepted = match state.event_repo.insert_batch(&to_insert).await {
        Ok(count) => {
            tracing::info!(accepted = count, rejected, "batch ingested");
            count
        }
        Err(e) => {
            tracing::error!(error = %e, "batch insert failed");
            rejected += to_insert.len();
            0
        }
    };

    Json(IngestEventsResponse { accepted, rejected })
}

/// GET /v1/events - Liste avec filtres
async fn list_events(
    State(state): State<SharedState>,
    Query(params): Query<EventQueryParams>,
) -> Result<Json<EventsListResponse>, StatusCode> {
    let filter = EventFilter {
        site_id: params.site_id,
        source: params.source,
        severity: params.severity,
        from_timestamp: params.from,
        to_timestamp: params.to,
        limit: params.limit,
        offset: params.offset,
    };

    match state.event_repo.find(&filter).await {
        Ok(events) => {
            let count = events.len();
            Ok(Json(EventsListResponse { events, count }))
        }
        Err(e) => {
            tracing::error!(error = %e, "failed to fetch events");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /v1/events/:id - Récupère un événement par ID
async fn get_event(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<EventResponse>, StatusCode> {
    match state.event_repo.find_by_id(id).await {
        Ok(Some(event)) => Ok(Json(EventResponse { event })),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!(error = %e, "failed to fetch event");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
