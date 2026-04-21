use crate::models::command::{
    Command, CommandResponse, CommandStatus, CommandsListResponse, CreateCommandRequest,
    UpdateCommandRequest,
};
use crate::models::hub::{CreateHubRequest, HeartbeatRequest, Hub, HubResponse, HubsListResponse};
use crate::state::SharedState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/hubs", get(list_hubs).post(create_hub))
        .route("/hubs/:id/heartbeat", post(heartbeat))
        .route(
            "/hubs/:id/commands",
            get(list_commands).post(create_command),
        )
        .route(
            "/hubs/:id/commands/:cmd_id",
            get(get_command).patch(update_command),
        )
}

// ── Hubs CRUD ────────────────────────────────────────────────────────────────

async fn list_hubs(State(state): State<SharedState>) -> Result<Json<HubsListResponse>, StatusCode> {
    let hubs = state.hub_repo.find_all().await.map_err(db_err)?;
    let count = hubs.len();
    Ok(Json(HubsListResponse { hubs, count }))
}

async fn create_hub(
    State(state): State<SharedState>,
    Json(req): Json<CreateHubRequest>,
) -> Result<(StatusCode, Json<HubResponse>), StatusCode> {
    if req.id.trim().is_empty() || req.name.trim().is_empty() || req.api_key.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    // Empêche les doublons — on lit d'abord
    if state
        .hub_repo
        .find_by_id(&req.id)
        .await
        .map_err(db_err)?
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    let hub = state
        .hub_repo
        .insert(&req.id, &req.site_id, &req.name, &req.api_key)
        .await
        .map_err(db_err)?;

    tracing::info!(hub_id = %hub.id, site = %hub.site_id, "hub registered");
    Ok((StatusCode::CREATED, Json(HubResponse { hub })))
}

async fn heartbeat(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(_req): Json<HeartbeatRequest>,
) -> Result<Json<HubResponse>, StatusCode> {
    let hub = ensure_hub(&state, &id).await?;
    state.hub_repo.touch(&id).await.map_err(db_err)?;
    let refreshed = state
        .hub_repo
        .find_by_id(&id)
        .await
        .map_err(db_err)?
        .unwrap_or(hub);
    Ok(Json(HubResponse { hub: refreshed }))
}

// ── Commands ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListCommandsParams {
    status: Option<String>,
}

async fn list_commands(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Query(params): Query<ListCommandsParams>,
) -> Result<Json<CommandsListResponse>, StatusCode> {
    ensure_hub(&state, &id).await?;

    let status_filter = params.status.as_deref().and_then(CommandStatus::parse);

    let commands = state
        .command_repo
        .find_by_hub(&id, status_filter)
        .await
        .map_err(db_err)?;

    let count = commands.len();
    Ok(Json(CommandsListResponse { commands, count }))
}

async fn create_command(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<CreateCommandRequest>,
) -> Result<(StatusCode, Json<CommandResponse>), StatusCode> {
    ensure_hub(&state, &id).await?;

    let command = state
        .command_repo
        .insert(&id, req.command_type, &req.payload, req.ttl_seconds)
        .await
        .map_err(db_err)?;

    // Best-effort broadcast aux hubs WS connectés. Les récepteurs filtrent sur hub_id.
    let _ = state.command_tx.send(command.clone());

    tracing::info!(
        hub_id = %command.hub_id,
        command_id = %command.id,
        cmd_type = command.command_type.as_str(),
        "command enqueued"
    );

    Ok((StatusCode::CREATED, Json(CommandResponse { command })))
}

async fn get_command(
    State(state): State<SharedState>,
    Path((hub_id, cmd_id)): Path<(String, Uuid)>,
) -> Result<Json<CommandResponse>, StatusCode> {
    ensure_hub(&state, &hub_id).await?;

    let command = state
        .command_repo
        .find_by_id(cmd_id)
        .await
        .map_err(db_err)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if command.hub_id != hub_id {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(CommandResponse { command }))
}

async fn update_command(
    State(state): State<SharedState>,
    Path((hub_id, cmd_id)): Path<(String, Uuid)>,
    Json(req): Json<UpdateCommandRequest>,
) -> Result<Json<CommandResponse>, StatusCode> {
    let existing = ensure_command(&state, &hub_id, cmd_id).await?;
    // Empêche les transitions arbitraires — on valide le passage
    if !is_valid_transition(existing.status, req.status) {
        return Err(StatusCode::CONFLICT);
    }

    let updated = state
        .command_repo
        .update_status(
            cmd_id,
            req.status,
            req.result.as_ref(),
            req.error.as_deref(),
        )
        .await
        .map_err(db_err)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(CommandResponse { command: updated }))
}

// ── Helpers ──────────────────────────────────────────────────────────────────

async fn ensure_hub(state: &SharedState, id: &str) -> Result<Hub, StatusCode> {
    state
        .hub_repo
        .find_by_id(id)
        .await
        .map_err(db_err)?
        .ok_or(StatusCode::NOT_FOUND)
}

async fn ensure_command(
    state: &SharedState,
    hub_id: &str,
    cmd_id: Uuid,
) -> Result<Command, StatusCode> {
    let cmd = state
        .command_repo
        .find_by_id(cmd_id)
        .await
        .map_err(db_err)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if cmd.hub_id != hub_id {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(cmd)
}

fn is_valid_transition(from: CommandStatus, to: CommandStatus) -> bool {
    use CommandStatus::*;
    matches!(
        (from, to),
        (Pending, Dispatched)
            | (Pending, Running)
            | (Pending, Failed)
            | (Dispatched, Running)
            | (Dispatched, Failed)
            | (Running, Succeeded)
            | (Running, Failed)
    )
}

fn db_err(e: sqlx::Error) -> StatusCode {
    tracing::error!(error = %e, "hubs route db error");
    StatusCode::INTERNAL_SERVER_ERROR
}
