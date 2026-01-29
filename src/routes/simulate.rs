use crate::simulator::generate_random_event;
use crate::state::SharedState;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<SharedState> {
    Router::new().route("/simulate", post(simulate_events))
}

#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    /// Nombre d'événements à générer (default: 1, max: 100)
    #[serde(default = "default_count")]
    pub count: usize,
    /// Site ID optionnel (si non fourni, généré aléatoirement)
    pub site_id: Option<Uuid>,
    /// Intervalle entre les événements en ms (default: 0 = instantané)
    #[serde(default)]
    pub interval_ms: u64,
}

fn default_count() -> usize {
    1
}

#[derive(Debug, Serialize)]
pub struct SimulateResponse {
    pub generated: usize,
    pub persisted: usize,
    pub broadcast: usize,
}

/// POST /v1/simulate - Génère des événements de test
async fn simulate_events(
    State(state): State<SharedState>,
    Json(req): Json<SimulateRequest>,
) -> Result<Json<SimulateResponse>, StatusCode> {
    let count = req.count.min(100); // Cap à 100 max

    let mut events = Vec::with_capacity(count);
    for _ in 0..count {
        events.push(generate_random_event(req.site_id));
    }

    // Persister en DB
    let persisted = match state.event_repo.insert_batch(&events).await {
        Ok(n) => n,
        Err(e) => {
            tracing::error!(error = %e, "failed to persist simulated events");
            0
        }
    };

    // Broadcast aux clients WebSocket
    let mut broadcast = 0;
    for event in &events {
        if state.event_tx.send(event.clone()).is_ok() {
            broadcast += 1;
        }
    }

    tracing::info!(
        generated = count,
        persisted,
        broadcast,
        "simulated events generated"
    );

    Ok(Json(SimulateResponse {
        generated: count,
        persisted,
        broadcast,
    }))
}
