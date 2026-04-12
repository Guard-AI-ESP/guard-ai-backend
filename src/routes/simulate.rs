use crate::models::event::{EventSource, EventV1, Severity, SimulateRequest, SimulateResponse};
use crate::state::SharedState;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use rand::Rng;
use uuid::Uuid;

pub fn router() -> Router<SharedState> {
    Router::new().route("/simulate", post(simulate_events))
}

/// POST /v1/simulate — génère des événements fictifs et les insère en base
///
/// Utile en l'absence d'IoT réel pour alimenter le dashboard et tester le WebSocket.
/// `count` est limité à 100 pour éviter les abus.
async fn simulate_events(
    State(state): State<SharedState>,
    Json(req): Json<SimulateRequest>,
) -> Result<Json<SimulateResponse>, StatusCode> {
    let count = req.count.min(100) as usize;
    let events = build_fake_events(count);

    let generated = state
        .event_repo
        .insert_batch(&events)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "simulate insert failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Diffusion vers les clients WebSocket connectés (best-effort)
    for event in &events {
        // send() échoue silencieusement s'il n'y a aucun abonné
        let _ = state.event_tx.send(event.clone());
    }

    tracing::info!(generated, "simulated events inserted and broadcast");
    Ok(Json(SimulateResponse { generated }))
}

/// Génère `count` événements avec des données réalistes mais aléatoires
fn build_fake_events(count: usize) -> Vec<EventV1> {
    let mut rng = rand::thread_rng();
    let demo_site_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("static UUID is valid");

    (0..count)
        .map(|_| {
            let source = random_source(&mut rng);
            let severity = random_severity(&mut rng);
            let event_type = random_event_type(&source, &mut rng);
            let detection = build_detection_fields(&source, &event_type, &mut rng);

            EventV1 {
                event_id: Uuid::new_v4(),
                site_id: demo_site_id,
                hub_id: Some("hub-demo-01".to_string()),
                source,
                event_type,
                severity,
                timestamp: chrono_now_iso(),
                payload: serde_json::json!({ "simulated": true }),
                media_ref: None,
                tags: vec!["simulated".to_string()],
                schema_version: "v1".to_string(),
                camera_id: detection.camera_id,
                face_id: detection.face_id,
                person_name: detection.person_name,
                confidence: detection.confidence,
                is_known: detection.is_known,
                bounding_box: detection.bounding_box,
            }
        })
        .collect()
}

struct DetectionFields {
    camera_id: Option<String>,
    face_id: Option<String>,
    person_name: Option<String>,
    confidence: Option<f64>,
    is_known: Option<bool>,
    bounding_box: Option<serde_json::Value>,
}

/// Remplit les champs de détection uniquement pour les events caméra pertinents
fn build_detection_fields(
    source: &EventSource,
    event_type: &str,
    rng: &mut impl Rng,
) -> DetectionFields {
    let is_face_event = matches!(source, EventSource::Camera)
        && matches!(event_type, "face_recognized" | "face_unknown");

    if !is_face_event {
        return DetectionFields {
            camera_id: match source {
                EventSource::Camera => Some(random_camera_id(rng)),
                _ => None,
            },
            face_id: None,
            person_name: None,
            confidence: None,
            is_known: None,
            bounding_box: None,
        };
    }

    let is_known = event_type == "face_recognized";
    let confidence: f64 = if is_known {
        // Visage connu : confiance élevée (85–99%)
        rng.gen_range(0.85..0.99)
    } else {
        // Inconnu : confiance faible (40–75%)
        rng.gen_range(0.40..0.75)
    };

    // Coordonnées réalistes du visage dans le frame (normalisées 0.0–1.0)
    let x: f64 = rng.gen_range(0.1..0.7);
    let y: f64 = rng.gen_range(0.05..0.5);
    let w: f64 = rng.gen_range(0.1..0.3);
    let h: f64 = w * rng.gen_range(1.1..1.4); // visage légèrement plus haut que large

    let (face_id, person_name) = if is_known {
        random_known_person(rng)
    } else {
        (None, None)
    };

    DetectionFields {
        camera_id: Some(random_camera_id(rng)),
        face_id,
        person_name,
        confidence: Some((confidence * 1000.0).round() / 1000.0),
        is_known: Some(is_known),
        bounding_box: Some(serde_json::json!({
            "x": (x * 1000.0).round() / 1000.0,
            "y": (y * 1000.0).round() / 1000.0,
            "w": (w * 1000.0).round() / 1000.0,
            "h": (h * 1000.0).round() / 1000.0,
        })),
    }
}

fn random_camera_id(rng: &mut impl Rng) -> String {
    let cameras = ["cam-entree-01", "cam-garage-01", "cam-portail-01", "cam-couloir-01"];
    cameras[rng.gen_range(0..cameras.len())].to_string()
}

/// Retourne une personne connue fictive (face_id + nom)
fn random_known_person(rng: &mut impl Rng) -> (Option<String>, Option<String>) {
    let persons = [
        ("550e8400-e29b-41d4-a716-446655440001", "Alice Martin"),
        ("550e8400-e29b-41d4-a716-446655440002", "Bob Dupont"),
        ("550e8400-e29b-41d4-a716-446655440003", "Claire Bernard"),
    ];
    let (id, name) = persons[rng.gen_range(0..persons.len())];
    (Some(id.to_string()), Some(name.to_string()))
}

fn random_source(rng: &mut impl Rng) -> EventSource {
    match rng.gen_range(0..4) {
        0 => EventSource::Camera,
        1 => EventSource::Network,
        2 => EventSource::Sensor,
        _ => EventSource::System,
    }
}

/// Pondéré : 65% info, 25% warning, 10% critical
fn random_severity(rng: &mut impl Rng) -> Severity {
    let n = rng.gen_range(0..100);
    if n < 65 {
        Severity::Info
    } else if n < 90 {
        Severity::Warning
    } else {
        Severity::Critical
    }
}

fn random_event_type(source: &EventSource, rng: &mut impl Rng) -> String {
    let options: &[&str] = match source {
        EventSource::Camera => &[
            "motion_detected",
            "face_recognized",
            "face_unknown",
            "camera_offline",
            "zone_breach",
        ],
        EventSource::Network => &[
            "new_device_connected",
            "device_disconnected",
            "bandwidth_spike",
            "unauthorized_access",
        ],
        EventSource::Sensor => &[
            "door_opened",
            "door_forced",
            "window_opened",
            "temperature_alert",
            "smoke_detected",
        ],
        EventSource::System => &[
            "system_startup",
            "system_shutdown",
            "config_changed",
            "update_available",
            "disk_full",
        ],
    };
    options[rng.gen_range(0..options.len())].to_string()
}

/// Retourne le timestamp courant en ISO-8601 sans dépendance externe
fn chrono_now_iso() -> String {
    // time crate est déjà dans Cargo.toml
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Format minimal ISO-8601 UTC
    let (y, mo, d, h, mi, s) = epoch_to_parts(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

fn epoch_to_parts(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s = secs % 60;
    let total_min = secs / 60;
    let mi = total_min % 60;
    let total_hours = total_min / 60;
    let h = total_hours % 24;
    let total_days = total_hours / 24;

    // Algorithme de conversion jours epoch → date (civil date from Julian day)
    let z = total_days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };

    (y, mo, d, h, mi, s)
}
