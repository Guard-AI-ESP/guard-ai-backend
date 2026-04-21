mod helpers;
use helpers::{build_test_app, test_jwt};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

/// Embedding minimal valide (2 valeurs) — le handler accepte toute taille non vide
fn minimal_embedding() -> Vec<f64> {
    vec![0.1, 0.2, 0.3]
}

fn create_person_body(name: &str) -> String {
    json!({
        "name": name,
        "embedding": minimal_embedding(),
        "photo_url": null
    })
    .to_string()
}

// ── Helpers HTTP ──────────────────────────────────────────────────────────────

async fn post_person(app: axum::Router, token: &str, body: &str) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/v1/persons")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
    .unwrap()
}

async fn get_persons(app: axum::Router, token: &str) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .uri("/v1/persons")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_persons_returns_empty() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    let response = get_persons(app, &token).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["count"], 0);
    assert!(body["persons"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_person_happy_path() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    let response = post_person(app, &token, &create_person_body("Alice Martin")).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["person"]["name"], "Alice Martin");
    assert!(body["person"]["id"].is_string());
    assert!(body["person"]["embedding"].is_array());
    assert!(body["person"]["created_at"].is_string());
}

#[tokio::test]
async fn test_create_person_missing_name_returns_400() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    let body = json!({ "name": "  ", "embedding": minimal_embedding() }).to_string();
    let response = post_person(app, &token, &body).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_person_empty_embedding_returns_400() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    let body = json!({ "name": "Bob", "embedding": [] }).to_string();
    let response = post_person(app, &token, &body).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_persons_after_create() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    // Créer deux personnes
    post_person(app.clone(), &token, &create_person_body("Alice")).await;
    post_person(app.clone(), &token, &create_person_body("Bob")).await;

    let response = get_persons(app, &token).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["count"], 2);
    assert_eq!(body["persons"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_person_happy_path() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    // Créer une personne
    let create_resp = post_person(app.clone(), &token, &create_person_body("Claire")).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body: Value =
        serde_json::from_slice(&create_resp.into_body().collect().await.unwrap().to_bytes())
            .unwrap();
    let id = body["person"]["id"].as_str().unwrap().to_string();

    // Supprimer
    let delete_resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/v1/persons/{id}"))
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_person_not_found_returns_404() {
    let app = build_test_app().await;
    let token = test_jwt("test@guard-ai.com");

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/persons/00000000-0000-0000-0000-000000000000")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_persons_routes_require_jwt() {
    let app = build_test_app().await;

    // GET sans token → 401
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/persons")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // POST sans token → 401
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/persons")
                .header("Content-Type", "application/json")
                .body(Body::from(create_person_body("Test")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
