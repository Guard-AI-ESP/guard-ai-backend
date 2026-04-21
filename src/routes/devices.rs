use crate::models::device::{DeviceSnapshotRequest, DeviceSnapshotResponse, DevicesListResponse};
use crate::state::SharedState;
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};

pub fn router() -> Router<SharedState> {
    Router::new().route("/devices", get(list_devices).put(apply_snapshot))
}

async fn list_devices(
    State(state): State<SharedState>,
) -> Result<Json<DevicesListResponse>, StatusCode> {
    let devices = state.device_repo.find_all().await.map_err(db_err)?;
    let count = devices.len();
    Ok(Json(DevicesListResponse { devices, count }))
}

async fn apply_snapshot(
    State(state): State<SharedState>,
    Json(req): Json<DeviceSnapshotRequest>,
) -> Result<Json<DeviceSnapshotResponse>, StatusCode> {
    if req.hub_id.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    // Le hub doit exister — évite qu'un faux hub pollue la table.
    if state
        .hub_repo
        .find_by_id(&req.hub_id)
        .await
        .map_err(db_err)?
        .is_none()
    {
        return Err(StatusCode::NOT_FOUND);
    }

    let connected_count = state
        .device_repo
        .apply_snapshot(&req.hub_id, &req.devices)
        .await
        .map_err(db_err)?;

    tracing::info!(
        hub_id = %req.hub_id,
        snapshot_len = req.devices.len(),
        connected = connected_count,
        "device snapshot applied"
    );

    Ok(Json(DeviceSnapshotResponse { connected_count }))
}

fn db_err(e: sqlx::Error) -> StatusCode {
    tracing::error!(error = %e, "devices route db error");
    StatusCode::INTERNAL_SERVER_ERROR
}
