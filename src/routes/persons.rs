use crate::models::person::{CreatePersonRequest, PersonResponse, PersonsListResponse};
use crate::state::SharedState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};
use uuid::Uuid;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/persons", get(list_persons).post(create_person))
        .route("/persons/:id", delete(delete_person))
}

/// GET /v1/persons — liste toutes les personnes connues (embedding inclus pour la reconnaissance)
async fn list_persons(
    State(state): State<SharedState>,
) -> Result<Json<PersonsListResponse>, StatusCode> {
    let persons = state
        .person_repo
        .find_all()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "list_persons db error");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let count = persons.len();
    Ok(Json(PersonsListResponse { persons, count }))
}

/// POST /v1/persons — enregistre une nouvelle personne avec son embedding
async fn create_person(
    State(state): State<SharedState>,
    Json(req): Json<CreatePersonRequest>,
) -> Result<(StatusCode, Json<PersonResponse>), StatusCode> {
    if req.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if req.embedding.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let person = state
        .person_repo
        .insert(
            req.name.trim(),
            &req.embedding,
            req.photo_url.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "create_person db error");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!(id = %person.id, name = %person.name, "person registered");
    Ok((StatusCode::CREATED, Json(PersonResponse { person })))
}

/// DELETE /v1/persons/:id — supprime une personne par son UUID
async fn delete_person(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    match state.person_repo.delete(id).await {
        Ok(true) => {
            tracing::info!(%id, "person deleted");
            StatusCode::NO_CONTENT
        }
        Ok(false) => StatusCode::NOT_FOUND,
        Err(e) => {
            tracing::error!(error = %e, "delete_person db error");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
