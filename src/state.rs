use crate::db::{
    CommandRepository, DbPool, DeviceRepository, EventRepository, HubRepository, PersonRepository,
    UserRepository,
};
use crate::models::command::Command;
use crate::models::event::EventV1;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Capacité du canal WebSocket — les clients lents perdent les messages anciens
pub const WS_CHANNEL_CAPACITY: usize = 256;

/// Durée de validité des JWT en secondes (24h)
pub const JWT_EXPIRY_SECS: u64 = 86_400;

/// État partagé de l'application, accessible dans tous les handlers
#[derive(Clone)]
pub struct AppState {
    pub event_repo: EventRepository,
    pub person_repo: PersonRepository,
    pub user_repo: UserRepository,
    pub hub_repo: HubRepository,
    pub device_repo: DeviceRepository,
    pub command_repo: CommandRepository,
    /// Canal de diffusion des nouveaux événements vers les clients WebSocket
    pub event_tx: broadcast::Sender<EventV1>,
    /// Canal de diffusion des nouvelles commandes vers les hubs connectés.
    /// Chaque hub-agent filtre sur son propre `hub_id` côté WS.
    pub command_tx: broadcast::Sender<Command>,
    /// Clé secrète pour signer/vérifier les JWT
    pub jwt_secret: String,
    /// Clé API pour l'accès machine-to-machine (IoT, services).
    /// `None` = auth machine désactivée (dev)
    pub api_key: Option<String>,
}

impl AppState {
    /// Construit depuis les variables d'environnement
    pub fn new(pool: DbPool) -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-insecure-secret-change-in-production".to_string());
        let api_key = std::env::var("API_KEY").ok();
        Self::with_config(pool, jwt_secret, api_key)
    }

    /// Constructeur explicite — utile dans les tests pour contrôler la config
    pub fn with_config(pool: DbPool, jwt_secret: String, api_key: Option<String>) -> Self {
        let (event_tx, _) = broadcast::channel(WS_CHANNEL_CAPACITY);
        let (command_tx, _) = broadcast::channel(WS_CHANNEL_CAPACITY);
        Self {
            event_repo: EventRepository::new(pool.clone()),
            person_repo: PersonRepository::new(pool.clone()),
            user_repo: UserRepository::new(pool.clone()),
            hub_repo: HubRepository::new(pool.clone()),
            device_repo: DeviceRepository::new(pool.clone()),
            command_repo: CommandRepository::new(pool),
            event_tx,
            command_tx,
            jwt_secret,
            api_key,
        }
    }
}

/// Type alias pour l'utilisation avec Axum State extractor
pub type SharedState = Arc<AppState>;
