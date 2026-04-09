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
pub struct WsParams {
    api_key: Option<String>,
}

pub fn router() -> Router<SharedState> {
    Router::new().route("/events/stream", get(ws_handler))
}

/// GET /ws/events/stream?api_key=<key>
///
/// Upgrade HTTP → WebSocket. Envoie chaque nouvel EventV1 (JSON) aux clients connectés
/// dès qu'il est inséré (via ingest ou simulate).
async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    // Validation de la clé API passée en query param
    if !is_authorized(&params.api_key, &state) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    ws.on_upgrade(move |socket| stream_events(socket, state))
}

/// Écoute le canal broadcast et transmet chaque événement au client WebSocket
async fn stream_events(mut socket: WebSocket, state: SharedState) {
    let mut rx = state.event_tx.subscribe();

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        let json = match serde_json::to_string(&event) {
                            Ok(s) => s,
                            Err(e) => {
                                tracing::warn!(error = %e, "failed to serialize WS event");
                                continue;
                            }
                        };
                        if socket.send(Message::Text(json)).await.is_err() {
                            // Client déconnecté
                            break;
                        }
                    }
                    // Le canal a débordé — on continue sans bloquer
                    Err(RecvError::Lagged(skipped)) => {
                        tracing::warn!(skipped, "WS client lagged, events dropped");
                        continue;
                    }
                    Err(RecvError::Closed) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    // Ping/pong géré par axum automatiquement — on ignore les autres messages
                    Some(Ok(_)) => {}
                    // None = client fermé proprement, Err = erreur réseau
                    _ => break,
                }
            }
        }
    }

    tracing::debug!("WebSocket client disconnected");
}

/// Valide la clé API transmise en query param contre celle stockée dans AppState
fn is_authorized(provided: &Option<String>, state: &crate::state::AppState) -> bool {
    match state.api_key.as_deref() {
        Some(expected) if !expected.is_empty() => {
            provided.as_deref().unwrap_or("") == expected
        }
        // Aucune clé configurée → accès libre
        _ => true,
    }
}
