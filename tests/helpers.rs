/// Helpers partagés entre les tests d'intégration
use guard_ai_backend::{app, db, models::user::Claims, state::AppState};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub const TEST_JWT_SECRET: &str = "test-secret";

pub async fn setup_test_db() -> db::DbPool {
    let pool = db::pool::create_pool("sqlite::memory:")
        .await
        .expect("test pool");
    db::pool::run_migrations(&pool).await.expect("migrations");
    pool
}

/// App de test avec JWT secret connu et auth activée
pub async fn build_test_app() -> axum::Router {
    let pool = setup_test_db().await;
    let state = Arc::new(AppState::with_config(
        pool,
        TEST_JWT_SECRET.to_string(),
        None,
    ));
    app::build_router_with_state(state)
}

/// Génère un JWT valide signé avec le secret de test
pub fn test_jwt(email: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let claims = Claims {
        sub: email.to_string(),
        iat: now,
        exp: now + 86_400,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("test jwt encode")
}
