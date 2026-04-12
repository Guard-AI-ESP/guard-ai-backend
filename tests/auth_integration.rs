mod helpers;
use helpers::build_test_app;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

// ─── POST /v1/auth/register ───────────────────────────────────────────────────

#[tokio::test]
async fn test_register_returns_token() {
    let app = build_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": "alice@guard-ai.com", "password": "secret123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["token"].as_str().is_some());
    assert_eq!(json["token_type"], "Bearer");
}

#[tokio::test]
async fn test_register_duplicate_email_returns_conflict() {
    let app = build_test_app().await;
    let payload = json!({ "email": "alice@guard-ai.com", "password": "secret123" }).to_string();

    // Premier register → OK
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.clone()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Second register avec le même email → 409
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_register_invalid_payload_returns_422() {
    let app = build_test_app().await;

    // Mot de passe trop court
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": "alice@guard-ai.com", "password": "abc" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ─── POST /v1/auth/login ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_login_returns_token() {
    let app = build_test_app().await;

    // Register d'abord
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": "bob@guard-ai.com", "password": "mypassword" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Login
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": "bob@guard-ai.com", "password": "mypassword" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["token"].as_str().is_some());
}

#[tokio::test]
async fn test_login_wrong_password_returns_401() {
    let app = build_test_app().await;

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": "carol@guard-ai.com", "password": "correct" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": "carol@guard-ai.com", "password": "wrong" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_unknown_email_returns_401() {
    let app = build_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": "nobody@guard-ai.com", "password": "whatever" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ─── Token used to access protected route ─────────────────────────────────────

#[tokio::test]
async fn test_jwt_from_login_grants_access_to_protected_routes() {
    let app = build_test_app().await;

    // Register
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": "dave@guard-ai.com", "password": "mypassword" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Login → get token
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "email": "dave@guard-ai.com", "password": "mypassword" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let login_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = login_json["token"].as_str().unwrap();

    // Use token to access protected route
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
}
