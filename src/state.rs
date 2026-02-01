use crate::db::{DbPool, EventRepository};
use crate::models::event::EventV1;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Capacité du channel broadcast (nombre d'événements en buffer)
const BROADCAST_CAPACITY: usize = 1024;

/// État partagé de l'application, accessible dans tous les handlers
#[derive(Clone)]
pub struct AppState {
    pub event_repo: EventRepository,
    /// Channel pour broadcaster les événements en temps réel
    pub event_tx: broadcast::Sender<EventV1>,
}

impl AppState {
    pub fn new(pool: DbPool) -> Self {
        let (event_tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            event_repo: EventRepository::new(pool),
            event_tx,
        }
    }

    /// Crée un nouveau receiver pour écouter les événements
    pub fn subscribe(&self) -> broadcast::Receiver<EventV1> {
        self.event_tx.subscribe()
    }
}

/// Type alias pour l'utilisation avec Axum State extractor
pub type SharedState = Arc<AppState>;
