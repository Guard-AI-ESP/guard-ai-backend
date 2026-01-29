use crate::models::event::{EventSource, EventV1, Severity};
use rand::Rng;
use uuid::Uuid;

/// Types d'événements simulés
const EVENT_TYPES: &[&str] = &[
    "motion_detected",
    "intrusion_alert",
    "door_opened",
    "temperature_high",
    "network_anomaly",
    "camera_offline",
    "sensor_triggered",
    "access_denied",
];

/// Zones simulées
const ZONES: &[&str] = &[
    "entrance",
    "parking",
    "warehouse",
    "office",
    "server_room",
    "reception",
    "corridor",
    "emergency_exit",
];

/// Génère un événement aléatoire
pub fn generate_random_event(site_id: Option<Uuid>) -> EventV1 {
    let mut rng = rand::thread_rng();

    let source = match rng.gen_range(0..4) {
        0 => EventSource::Camera,
        1 => EventSource::Sensor,
        2 => EventSource::Network,
        _ => EventSource::System,
    };

    let severity = match rng.gen_range(0..10) {
        0..=5 => Severity::Info,
        6..=8 => Severity::Warning,
        _ => Severity::Critical,
    };

    let event_type = EVENT_TYPES[rng.gen_range(0..EVENT_TYPES.len())].to_string();
    let zone = ZONES[rng.gen_range(0..ZONES.len())];

    let payload = serde_json::json!({
        "zone": zone,
        "confidence": rng.gen_range(70..100),
        "simulated": true
    });

    EventV1 {
        event_id: Uuid::new_v4(),
        site_id: site_id.unwrap_or_else(Uuid::new_v4),
        hub_id: Some(format!("hub-{}", rng.gen_range(1..5))),
        source,
        event_type,
        severity,
        timestamp: chrono_now_iso(),
        payload,
        media_ref: None,
        tags: vec!["simulated".to_string()],
        schema_version: "v1".to_string(),
    }
}

/// Génère un timestamp ISO 8601 actuel
fn chrono_now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}
