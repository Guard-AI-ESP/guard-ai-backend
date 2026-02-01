use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use guard_ai_backend::{app, db};
use http_body_util::BodyExt;
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::StreamExt;
use tower::ServiceExt;

async fn setup_test_db() -> db::DbPool {
    let pool = db::pool::create_pool("sqlite::memory:")
        .await
        .expect("test pool");
    db::pool::run_migrations(&pool).await.expect("migrations");
    pool
}

#[tokio::test]
async fn test_health_check() {
    let pool = setup_test_db().await;
    let app = app::build_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ingest_and_retrieve_events() {
    let pool = setup_test_db().await;
    let app = app::build_router(pool);

    let event = json!({
        "events": [{
            "event_id": "550e8400-e29b-41d4-a716-446655440001",
            "site_id": "550e8400-e29b-41d4-a716-446655440002",
            "hub_id": null,
            "source": "camera",
            "type": "motion_detected",
            "severity": "warning",
            "timestamp": "2024-01-15T10:30:00Z",
            "payload": {"zone": "entrance"},
            "media_ref": null,
            "tags": ["outdoor"],
            "schema_version": "v1"
        }]
    });

    // 1. Ingest un événement
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("Content-Type", "application/json")
                .body(Body::from(event.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["accepted"], 1);
    assert_eq!(json["rejected"], 0);

    // 2. Récupère les événements
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 1);

    // 3. Récupère par ID
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/events/550e8400-e29b-41d4-a716-446655440001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_filter_by_site_id() {
    let pool = setup_test_db().await;
    let app = app::build_router(pool);

    let events = json!({
        "events": [
            {
                "event_id": "550e8400-e29b-41d4-a716-446655440010",
                "site_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "source": "camera",
                "type": "test",
                "severity": "info",
                "timestamp": "2024-01-15T10:00:00Z",
                "payload": {},
                "tags": [],
                "schema_version": "v1"
            },
            {
                "event_id": "550e8400-e29b-41d4-a716-446655440011",
                "site_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                "source": "sensor",
                "type": "test",
                "severity": "warning",
                "timestamp": "2024-01-15T11:00:00Z",
                "payload": {},
                "tags": [],
                "schema_version": "v1"
            }
        ]
    });

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("Content-Type", "application/json")
                .body(Body::from(events.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Filter by site_id
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/events?site_id=aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 1);
}

#[tokio::test]
async fn test_event_not_found() {
    let pool = setup_test_db().await;
    let app = app::build_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/events/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_reject_invalid_schema_version() {
    let pool = setup_test_db().await;
    let app = app::build_router(pool);

    let event = json!({
        "events": [{
            "event_id": "550e8400-e29b-41d4-a716-446655440099",
            "site_id": "550e8400-e29b-41d4-a716-446655440002",
            "source": "camera",
            "type": "test",
            "severity": "info",
            "timestamp": "2024-01-15T10:30:00Z",
            "payload": {},
            "tags": [],
            "schema_version": "v2"
        }]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("Content-Type", "application/json")
                .body(Body::from(event.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["accepted"], 0);
    assert_eq!(json["rejected"], 1);
}

#[tokio::test]
async fn test_stats_endpoint() {
    let pool = setup_test_db().await;
    let app = app::build_router(pool);

    // Insérer quelques événements
    let events = json!({
        "events": [
            {
                "event_id": "550e8400-e29b-41d4-a716-446655440020",
                "site_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "source": "camera",
                "type": "test",
                "severity": "info",
                "timestamp": "2024-01-15T10:00:00Z",
                "payload": {},
                "tags": [],
                "schema_version": "v1"
            },
            {
                "event_id": "550e8400-e29b-41d4-a716-446655440021",
                "site_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "source": "sensor",
                "type": "test",
                "severity": "warning",
                "timestamp": "2024-01-15T11:00:00Z",
                "payload": {},
                "tags": [],
                "schema_version": "v1"
            }
        ]
    });

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("Content-Type", "application/json")
                .body(Body::from(events.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Vérifier les stats
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["total_events"], 2);
    assert!(json["by_severity"].is_object());
    assert!(json["by_source"].is_object());
}

#[tokio::test]
async fn test_simulate_endpoint() {
    let pool = setup_test_db().await;
    let app = app::build_router(pool);

    let simulate_req = json!({
        "count": 5
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/simulate")
                .header("Content-Type", "application/json")
                .body(Body::from(simulate_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["generated"], 5);
    assert_eq!(json["persisted"], 5);

    // Vérifier que les événements ont été créés
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 5);
}

#[tokio::test]
async fn test_websocket_event_stream() {
    // Setup test database and app
    let pool = setup_test_db().await;
    let app = app::build_router(pool);

    // Start the server in a background task
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind");
    let addr = listener.local_addr().expect("failed to get local addr");
    
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("server failed");
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Connect to WebSocket endpoint
    let ws_url = format!("ws://{}/v1/events/stream", addr);
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("failed to connect to websocket");

    let (mut _write, mut read) = ws_stream.split();

    // Create a task to listen for messages
    let receive_task = tokio::spawn(async move {
        let mut received_messages = Vec::new();
        while let Some(msg_result) = read.next().await {
            if let Ok(Message::Text(text)) = msg_result {
                received_messages.push(text);
                // Stop after receiving one message
                if received_messages.len() >= 1 {
                    break;
                }
            }
        }
        received_messages
    });

    // Trigger event generation via simulate endpoint
    let client = reqwest::Client::new();
    let simulate_url = format!("http://{}/v1/simulate", addr);
    let simulate_body = json!({ "count": 3 });
    
    client
        .post(&simulate_url)
        .json(&simulate_body)
        .send()
        .await
        .expect("failed to send simulate request");

    // Wait for messages with timeout
    let messages = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        receive_task
    )
    .await
    .expect("timeout waiting for messages")
    .expect("receive task failed");

    // Verify we received at least one message
    assert!(
        !messages.is_empty(),
        "Expected to receive at least one WebSocket message"
    );

    // Verify the message is valid JSON and has expected structure
    let event: serde_json::Value = serde_json::from_str(&messages[0])
        .expect("received message is not valid JSON");
    
    // Verify event has expected fields
    assert!(event.get("event_id").is_some(), "event missing event_id");
    assert!(event.get("site_id").is_some(), "event missing site_id");
    assert!(event.get("source").is_some(), "event missing source");
    assert!(event.get("type").is_some(), "event missing type");
    assert!(event.get("severity").is_some(), "event missing severity");
    assert!(event.get("timestamp").is_some(), "event missing timestamp");
}
