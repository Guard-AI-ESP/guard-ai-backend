mod helpers;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use futures_util::{SinkExt, StreamExt};
use helpers::{build_test_app, test_jwt};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tower::ServiceExt;

async fn register_hub(app: axum::Router, id: &str, api_key: &str) {
    let req = Request::builder()
        .method("POST")
        .uri("/v1/hubs")
        .header(
            "Authorization",
            format!("Bearer {}", test_jwt("admin@test.io")),
        )
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "id": id,
                "site_id": "00000000-0000-0000-0000-000000000042",
                "name": "test hub",
                "api_key": api_key
            })
            .to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn ws_hub_rejects_missing_creds() {
    let app = build_test_app().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let _h = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/ws/hub");
    let result = connect_async(url).await;
    assert!(result.is_err(), "WS handshake should fail without creds");
}

#[tokio::test]
async fn ws_hub_rejects_wrong_api_key() {
    let app = build_test_app().await;
    register_hub(app.clone(), "pi-hub-01", "the-right-key").await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let _h = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/ws/hub?hub_id=pi-hub-01&api_key=wrong");
    let result = connect_async(url).await;
    assert!(
        result.is_err(),
        "WS handshake should fail with wrong api_key"
    );
}

#[tokio::test]
async fn ws_hub_receives_command_posted_over_http() {
    let app = build_test_app().await;
    register_hub(app.clone(), "pi-hub-01", "secret-hub-key").await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let server_app = app.clone();
    let _h = tokio::spawn(async move { axum::serve(listener, server_app).await.unwrap() });

    // Connect the hub WS
    let url = format!("ws://{addr}/ws/hub?hub_id=pi-hub-01&api_key=secret-hub-key");
    let (mut ws, _resp) = connect_async(url).await.expect("WS should connect");

    // Give the handler a moment to subscribe to the broadcast channel before we post
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Post a command over HTTP
    let http_req = Request::builder()
        .method("POST")
        .uri("/v1/hubs/pi-hub-01/commands")
        .header(
            "Authorization",
            format!("Bearer {}", test_jwt("admin@test.io")),
        )
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "type": "scan_network",
                "payload": {"subnet": "192.168.50.0/24", "mode": "host_discovery"}
            })
            .to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(http_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let created: Value = serde_json::from_slice(&body).unwrap();
    let expected_id = created["command"]["id"].as_str().unwrap().to_string();

    // Receive the command on the WS (with timeout)
    let msg = tokio::time::timeout(Duration::from_secs(2), ws.next())
        .await
        .expect("WS message should arrive within 2s")
        .expect("WS stream still open")
        .expect("WS message without error");

    let text = msg.into_text().unwrap();
    let got: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(got["id"], expected_id);
    assert_eq!(got["type"], "scan_network");
    assert_eq!(got["hub_id"], "pi-hub-01");

    // Close cleanly
    let _ = ws.close(None).await;
}
