use crate::state::SharedState;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};

pub fn router() -> Router<SharedState> {
    Router::new().route("/events/stream", get(ws_handler))
}

/// Handler pour l'upgrade WebSocket
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<SharedState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Gère la connexion WebSocket
async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let mut event_rx = state.subscribe();

    tracing::info!("WebSocket client connected");

    // Task pour envoyer les événements au client
    let send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match serde_json::to_string(&event) {
                Ok(json) => {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "failed to serialize event for WebSocket");
                }
            }
        }
    });

    // Task pour recevoir les messages du client (ping/pong, close)
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => break,
                Ok(Message::Ping(data)) => {
                    tracing::debug!("Received ping");
                    // Le pong est géré automatiquement par axum
                    drop(data);
                }
                Err(e) => {
                    tracing::debug!(error = %e, "WebSocket receive error");
                    break;
                }
                _ => {}
            }
        }
    });

    // Attendre qu'une des tasks se termine
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    tracing::info!("WebSocket client disconnected");
}
