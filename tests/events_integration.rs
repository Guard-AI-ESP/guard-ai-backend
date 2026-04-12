mod helpers;
use helpers::{build_test_app, test_jwt};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let app = build_test_app().await;

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
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

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

    // 1. Ingest
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
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

    // 2. List
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/events")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 1);

    // 3. Get by ID
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/events/550e8400-e29b-41d4-a716-446655440001")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_filter_by_site_id() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

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
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(events.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/events?site_id=aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
                .header("Authorization", format!("Bearer {token}"))
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
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/events/00000000-0000-0000-0000-000000000000")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_reject_invalid_schema_version() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

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
                .header("Authorization", format!("Bearer {token}"))
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
async fn test_protected_routes_require_jwt() {
    let app = build_test_app().await;

    // Sans token → 401
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
