use crate::db::{DbPool, EventRepository};
use std::sync::Arc;

/// État partagé de l'application, accessible dans tous les handlers
#[derive(Clone)]
pub struct AppState {
    pub event_repo: EventRepository,
}

impl AppState {
    pub fn new(pool: DbPool) -> Self {
        Self {
            event_repo: EventRepository::new(pool),
        }
    }
}

/// Type alias pour l'utilisation avec Axum State extractor
pub type SharedState = Arc<AppState>;
