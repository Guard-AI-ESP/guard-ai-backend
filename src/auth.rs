use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use subtle::ConstantTimeEq;

const API_KEY_HEADER: &str = "X-API-Key";

/// Middleware pour valider l'API key
pub async fn require_api_key(req: Request, next: Next) -> Result<Response, StatusCode> {
    let api_key = std::env::var("API_KEY").ok();

    // Si aucune API_KEY n'est configurée, on laisse passer (mode développement)
    let Some(expected_key) = api_key else {
        tracing::warn!("API_KEY not configured - authentication disabled");
        return Ok(next.run(req).await);
    };

    // Vérifier le header X-API-Key
    let provided_key = req
        .headers()
        .get(API_KEY_HEADER)
        .and_then(|v| v.to_str().ok());

    match provided_key {
        Some(key) if constant_time_eq(key.as_bytes(), expected_key.as_bytes()) => {
            Ok(next.run(req).await)
        }
        Some(_) => {
            tracing::warn!("Invalid API key provided");
            Err(StatusCode::UNAUTHORIZED)
        }
        None => {
            tracing::warn!("Missing API key header");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Constant-time comparison to prevent timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.ct_eq(b).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "ok"
    }

    #[tokio::test]
    async fn test_missing_api_key() {
        std::env::set_var("API_KEY", "test-key-123");

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(require_api_key));

        let req = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        std::env::remove_var("API_KEY");
    }

    #[tokio::test]
    async fn test_invalid_api_key() {
        std::env::set_var("API_KEY", "test-key-123");

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(require_api_key));

        let req = Request::builder()
            .uri("/test")
            .header("X-API-Key", "wrong-key")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        std::env::remove_var("API_KEY");
    }

    #[tokio::test]
    async fn test_valid_api_key() {
        std::env::set_var("API_KEY", "test-key-123");

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(require_api_key));

        let req = Request::builder()
            .uri("/test")
            .header("X-API-Key", "test-key-123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        std::env::remove_var("API_KEY");
    }

    #[tokio::test]
    #[ignore] // This test has environment variable isolation issues when run in parallel
    async fn test_no_api_key_configured() {
        std::env::remove_var("API_KEY");

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(require_api_key));

        let req = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        // Should pass through when no API_KEY is configured
        assert_eq!(response.status(), StatusCode::OK);
    }
}
