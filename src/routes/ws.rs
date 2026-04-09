use crate::models::user::Claims;
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
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;

#[derive(Deserialize)]
pub struct WsParams {
    /// JWT passé en query param (les headers custom ne sont pas supportés par l'API WebSocket browser)
    token: Option<String>,
}

pub fn router() -> Router<SharedState> {
    Router::new().route("/events/stream", get(ws_handler))
}

/// GET /ws/events/stream?token=<jwt>
///
/// Upgrade HTTP → WebSocket. Diffuse chaque nouvel EventV1 (JSON) aux clients connectés.
async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    if !is_authorized(&params.token, &state) {
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
                            break;
                        }
                    }
                    Err(RecvError::Lagged(skipped)) => {
                        tracing::warn!(skipped, "WS client lagged, events dropped");
                        continue;
                    }
                    Err(RecvError::Closed) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(_)) => {}
                    _ => break,
                }
            }
        }
    }

    tracing::debug!("WebSocket client disconnected");
}

/// Valide le JWT passé en query param
fn is_authorized(token: &Option<String>, state: &crate::state::AppState) -> bool {
    let Some(token) = token else {
        // Pas de token → autorisé uniquement si JWT_SECRET non configuré (dev)
        return state.jwt_secret == "dev-insecure-secret-change-in-production";
    };

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .is_ok()
}
