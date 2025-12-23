use serde::{Deserialize, Serialize};
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
