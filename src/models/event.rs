use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    Camera,
    Network,
    Sensor,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventV1 {
    pub event_id: Uuid,
    pub site_id: Uuid,
    pub hub_id: Option<String>,

    pub source: EventSource,
    #[serde(rename = "type")]
    pub event_type: String,
    pub severity: Severity,

    // On accepte date-time en string pour Lot 0 (validation stricte plus tard)
    pub timestamp: String,

    pub payload: serde_json::Value,

    pub media_ref: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,

    pub schema_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestEventsRequest {
    pub events: Vec<EventV1>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngestEventsResponse {
    pub accepted: usize,
    pub rejected: usize,
}

/// Paramètres de query pour GET /v1/events
#[derive(Debug, Clone, Deserialize, Default)]
pub struct EventQueryParams {
    pub site_id: Option<Uuid>,
    pub source: Option<EventSource>,
    pub severity: Option<Severity>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Réponse paginée pour GET /v1/events
#[derive(Debug, Clone, Serialize)]
pub struct EventsListResponse {
    pub events: Vec<EventV1>,
    pub count: usize,
}

/// Réponse pour GET /v1/events/:id
#[derive(Debug, Clone, Serialize)]
pub struct EventResponse {
    pub event: EventV1,
}

/// Statistiques agrégées pour GET /v1/stats
#[derive(Debug, Clone, Serialize)]
pub struct EventStats {
    pub total_events: i64,
    pub last_24h: i64,
    /// Événements critiques des dernières 24h (utilisé comme "active alerts")
    pub active_alerts: i64,
    pub by_source: HashMap<String, i64>,
    pub by_severity: HashMap<String, i64>,
}

/// Corps de POST /v1/simulate
#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    /// Nombre d'événements à générer (1–100, défaut 5)
    #[serde(default = "default_count")]
    pub count: u32,
}

fn default_count() -> u32 {
    5
}

/// Réponse de POST /v1/simulate
#[derive(Debug, Serialize)]
pub struct SimulateResponse {
    pub generated: usize,
}
