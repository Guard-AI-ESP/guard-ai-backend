mod helpers;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use helpers::{build_test_app, test_jwt};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

fn auth_header() -> (String, String) {
    ("Authorization".to_string(), format!("Bearer {}", test_jwt("admin@test.io")))
}

async fn send(app: axum::Router, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let (h, v) = auth_header();
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header(h, v)
        .header("content-type", "application/json");
    let req = match body {
        Some(b) => req.body(Body::from(b.to_string())).unwrap(),
        None => req.body(Body::empty()).unwrap(),
    };
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

#[tokio::test]
async fn hubs_crud_and_heartbeat() {
    let app = build_test_app().await;

    // Register a hub
    let (status, body) = send(
        app.clone(),
        "POST",
        "/v1/hubs",
        Some(json!({
            "id": "pi-hub-01",
            "site_id": "00000000-0000-0000-0000-000000000042",
            "name": "Demo Jury Hub",
            "api_key": "super-secret-hub-key"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["hub"]["id"], "pi-hub-01");
    assert!(body["hub"]["last_seen_at"].is_null());

    // Duplicate id → 409
    let (status, _) = send(
        app.clone(),
        "POST",
        "/v1/hubs",
        Some(json!({
            "id": "pi-hub-01",
            "site_id": "00000000-0000-0000-0000-000000000042",
            "name": "dup",
            "api_key": "x"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Heartbeat → last_seen_at gets populated
    let (status, body) = send(
        app.clone(),
        "POST",
        "/v1/hubs/pi-hub-01/heartbeat",
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["hub"]["last_seen_at"].is_string());

    // List
    let (status, body) = send(app, "GET", "/v1/hubs", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 1);
}

#[tokio::test]
async fn commands_enqueue_dispatch_update_lifecycle() {
    let app = build_test_app().await;

    send(
        app.clone(),
        "POST",
        "/v1/hubs",
        Some(json!({
            "id": "pi-hub-01",
            "site_id": "00000000-0000-0000-0000-000000000042",
            "name": "hub",
            "api_key": "k"
        })),
    )
    .await;

    // Enqueue a scan_network command with TTL
    let (status, body) = send(
        app.clone(),
        "POST",
        "/v1/hubs/pi-hub-01/commands",
        Some(json!({
            "type": "scan_network",
            "payload": {"subnet": "192.168.50.0/24", "mode": "host_discovery"},
            "ttl_seconds": 300
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["command"]["type"], "scan_network");
    assert_eq!(body["command"]["status"], "pending");
    assert!(body["command"]["expires_at"].is_string());
    let cmd_id = body["command"]["id"].as_str().unwrap().to_string();

    // List pending → finds it
    let (status, body) = send(
        app.clone(),
        "GET",
        "/v1/hubs/pi-hub-01/commands?status=pending",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"], 1);

    // Hub moves to running
    let url = format!("/v1/hubs/pi-hub-01/commands/{cmd_id}");
    let (status, body) = send(
        app.clone(),
        "PATCH",
        &url,
        Some(json!({"status": "running"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["command"]["status"], "running");

    // Invalid transition: running → pending should 409
    let (status, _) = send(
        app.clone(),
        "PATCH",
        &url,
        Some(json!({"status": "pending"})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Complete with result
    let (status, body) = send(
        app.clone(),
        "PATCH",
        &url,
        Some(json!({
            "status": "succeeded",
            "result": {"hosts_found": 4, "scan_duration_ms": 812}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["command"]["status"], "succeeded");
    assert_eq!(body["command"]["result"]["hosts_found"], 4);
    assert!(body["command"]["completed_at"].is_string());
}

#[tokio::test]
async fn commands_scope_to_their_hub() {
    let app = build_test_app().await;

    // Register two hubs
    for (id, name) in [("pi-hub-01", "first"), ("pi-hub-02", "second")] {
        send(
            app.clone(),
            "POST",
            "/v1/hubs",
            Some(json!({
                "id": id, "site_id": "00000000-0000-0000-0000-000000000042",
                "name": name, "api_key": "k"
            })),
        )
        .await;
    }

    // Enqueue command on hub-01
    let (_, body) = send(
        app.clone(),
        "POST",
        "/v1/hubs/pi-hub-01/commands",
        Some(json!({"type": "scan_network", "payload": {}})),
    )
    .await;
    let cmd_id = body["command"]["id"].as_str().unwrap().to_string();

    // Try to read it through hub-02 → must 404
    let url = format!("/v1/hubs/pi-hub-02/commands/{cmd_id}");
    let (status, _) = send(app, "GET", &url, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
