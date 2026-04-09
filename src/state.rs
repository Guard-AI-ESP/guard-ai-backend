use crate::db::{DbPool, EventRepository};
use crate::models::event::EventV1;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Capacité du canal WebSocket — les clients lents perdent les messages anciens
pub const WS_CHANNEL_CAPACITY: usize = 256;

/// État partagé de l'application, accessible dans tous les handlers
#[derive(Clone)]
pub struct AppState {
    pub event_repo: EventRepository,
    /// Canal de diffusion des nouveaux événements vers les clients WebSocket
    pub event_tx: broadcast::Sender<EventV1>,
    /// Clé API attendue. `None` = auth désactivée (dev sans variable d'env)
    pub api_key: Option<String>,
}

impl AppState {
    /// Construit depuis les variables d'environnement
    pub fn new(pool: DbPool) -> Self {
        Self::with_config(pool, std::env::var("API_KEY").ok())
    }

    /// Constructeur explicite — utile dans les tests pour contrôler la clé sans toucher l'env
    pub fn with_config(pool: DbPool, api_key: Option<String>) -> Self {
        let (event_tx, _) = broadcast::channel(WS_CHANNEL_CAPACITY);
        Self {
            event_repo: EventRepository::new(pool),
            event_tx,
            api_key,
        }
    }
}

/// Type alias pour l'utilisation avec Axum State extractor
pub type SharedState = Arc<AppState>;
