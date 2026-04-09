mod helpers;
use helpers::{build_test_app, test_jwt};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use guard_ai_backend::{app, db, state::AppState};
use http_body_util::BodyExt;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

// ─── /v1/stats ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_stats_empty_db() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/stats")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["total_events"], 0);
    assert_eq!(json["last_24h"], 0);
    assert_eq!(json["active_alerts"], 0);
}

#[tokio::test]
async fn test_stats_after_ingest() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    let events = json!({
        "events": [
            {
                "event_id": "aaaaaaaa-0000-0000-0000-000000000001",
                "site_id": "aaaaaaaa-0000-0000-0000-000000000000",
                "source": "camera",
                "type": "motion_detected",
                "severity": "warning",
                "timestamp": "2020-01-01T12:00:00Z",
                "payload": {},
                "tags": [],
                "schema_version": "v1"
            },
            {
                "event_id": "aaaaaaaa-0000-0000-0000-000000000002",
                "site_id": "aaaaaaaa-0000-0000-0000-000000000000",
                "source": "sensor",
                "type": "door_forced",
                "severity": "critical",
                "timestamp": "2020-01-01T12:01:00Z",
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
                .uri("/v1/stats")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["total_events"], 2);
    assert_eq!(json["last_24h"], 0);
    assert_eq!(json["by_source"]["camera"], 1);
    assert_eq!(json["by_source"]["sensor"], 1);
    assert_eq!(json["by_severity"]["warning"], 1);
    assert_eq!(json["by_severity"]["critical"], 1);
}

// ─── /v1/simulate ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_simulate_generates_events() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/simulate")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(json!({ "count": 5 }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["generated"], 5);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/events")
                .header("Authorization", format!("Bearer {token}"))
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
async fn test_simulate_default_count() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/simulate")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["generated"], 5);
}

#[tokio::test]
async fn test_simulate_count_capped_at_100() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/simulate")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(json!({ "count": 999 }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["generated"], 100);
}

// ─── JWT middleware ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_jwt_required_on_protected_routes() {
    let pool = db::pool::create_pool("sqlite::memory:").await.unwrap();
    db::pool::run_migrations(&pool).await.unwrap();
    let state = Arc::new(AppState::with_config(pool, "secret".to_string(), None));
    let app = app::build_router_with_state(state);

    // Sans token → 401
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Mauvais token → 401
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/stats")
                .header("Authorization", "Bearer invalid.token.here")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_health_is_public() {
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
