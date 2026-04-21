use crate::models::command::Command;
use crate::state::SharedState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;

#[derive(Deserialize)]
pub struct HubWsParams {
    pub hub_id: Option<String>,
    pub api_key: Option<String>,
}

pub fn router() -> Router<SharedState> {
    Router::new().route("/hub", get(hub_handler))
}

/// GET /ws/hub?hub_id=<id>&api_key=<plain>
///
/// Authentifie le hub via la clé M2M (bcrypt check), met à jour
/// `last_seen_at`, puis pousse en JSON chaque `Command` créée dont
/// `command.hub_id == hub_id` demandé.
async fn hub_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HubWsParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let (Some(hub_id), Some(api_key)) = (params.hub_id.clone(), params.api_key) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    match state.hub_repo.verify_api_key(&hub_id, &api_key).await {
        Ok(true) => {}
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    }

    // Met à jour le last_seen_at du hub dès la connexion.
    if let Err(e) = state.hub_repo.touch(&hub_id).await {
        tracing::warn!(error = %e, hub_id = %hub_id, "failed to touch hub on WS connect");
    }

    tracing::info!(hub_id = %hub_id, "hub WS connected");
    ws.on_upgrade(move |socket| stream_commands(socket, state, hub_id))
}

async fn stream_commands(mut socket: WebSocket, state: SharedState, hub_id: String) {
    let mut rx = state.command_tx.subscribe();

    loop {
        tokio::select! {
            result = rx.recv() => match result {
                Ok(cmd) if cmd.hub_id == hub_id => {
                    if let Err(e) = deliver(&mut socket, &state, cmd).await {
                        tracing::debug!(hub_id = %hub_id, error = %e, "WS deliver failed, closing");
                        break;
                    }
                }
                Ok(_) => { /* commande pour un autre hub, ignore */ }
                Err(RecvError::Lagged(n)) => {
                    tracing::warn!(hub_id = %hub_id, skipped = n, "hub WS lagged, commands dropped");
                    continue;
                }
                Err(RecvError::Closed) => break,
            },
            msg = socket.recv() => match msg {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Err(_)) => break,
                Some(Ok(_)) => { /* ignore client-originated text/ping/pong */ }
            }
        }
    }

    tracing::info!(hub_id = %hub_id, "hub WS disconnected");
}

async fn deliver(
    socket: &mut WebSocket,
    state: &SharedState,
    cmd: Command,
) -> Result<(), String> {
    let json = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;
    socket
        .send(Message::Text(json))
        .await
        .map_err(|e| e.to_string())?;
    // Best-effort: mark the command dispatched once it left the socket.
    if let Err(e) = state.command_repo.mark_dispatched(cmd.id).await {
        tracing::warn!(
            hub_id = %cmd.hub_id,
            command_id = %cmd.id,
            error = %e,
            "failed to mark command dispatched"
        );
    }
    Ok(())
}
