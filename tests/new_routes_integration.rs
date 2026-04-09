/// Tests d'intégration pour les nouvelles routes : stats, simulate, API key middleware
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use guard_ai_backend::{app, db, state::AppState};
use http_body_util::BodyExt;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_test_db() -> db::DbPool {
    let pool = db::pool::create_pool("sqlite::memory:")
        .await
        .expect("test pool");
    db::pool::run_migrations(&pool).await.expect("migrations");
    pool
}

/// App sans clé API (auth désactivée)
async fn build_app_no_auth() -> axum::Router {
    let pool = setup_test_db().await;
    app::build_router(pool)
}

/// App avec une clé API définie
async fn build_app_with_key(key: &str) -> axum::Router {
    let pool = setup_test_db().await;
    let state = Arc::new(AppState::with_config(pool, Some(key.to_string())));
    app::build_router_with_state(state)
}

// ─── /v1/stats ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_stats_empty_db() {
    let app = build_app_no_auth().await;

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

    assert_eq!(json["total_events"], 0);
    assert_eq!(json["last_24h"], 0);
    assert_eq!(json["active_alerts"], 0);
}

#[tokio::test]
async fn test_stats_after_ingest() {
    let app = build_app_no_auth().await;

    // Ingest 2 events : 1 warning camera, 1 critical sensor
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
                .body(Body::from(events.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

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
    // Timestamps en 2020 → hors de la fenêtre des 24 dernières heures
    assert_eq!(json["last_24h"], 0);
    assert_eq!(json["by_source"]["camera"], 1);
    assert_eq!(json["by_source"]["sensor"], 1);
    assert_eq!(json["by_severity"]["warning"], 1);
    assert_eq!(json["by_severity"]["critical"], 1);
}

// ─── /v1/simulate ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_simulate_generates_events() {
    let app = build_app_no_auth().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/simulate")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "count": 5 }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["generated"], 5);

    // Vérifie que les events sont bien en base
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
async fn test_simulate_default_count() {
    let app = build_app_no_auth().await;

    // Pas de count → défaut = 5
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/simulate")
                .header("Content-Type", "application/json")
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
    let app = build_app_no_auth().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/simulate")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "count": 999 }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Cappé à 100
    assert_eq!(json["generated"], 100);
}

// ─── API key middleware ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_api_key_required_when_configured() {
    let app = build_app_with_key("test-secret-key").await;

    // Sans clé → 401
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

    // Mauvaise clé → 401
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/stats")
                .header("x-api-key", "wrong-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Bonne clé → 200
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/stats")
                .header("x-api-key", "test-secret-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_is_public_even_with_api_key_configured() {
    let app = build_app_with_key("test-secret-key").await;

    // Health ne passe pas par le middleware protégé
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
