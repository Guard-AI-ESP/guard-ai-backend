use guard_ai_backend::db;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Configuration DB depuis variable d'environnement ou default
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://guard-ai.db".to_string());

    // Initialisation du pool de connexions
    let pool = db::pool::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    tracing::info!(%database_url, "Database pool created");

    // Exécution des migrations
    db::pool::run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database migrations completed");

    // Démarrage du serveur
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let app = guard_ai_backend::app::build_router(pool);

    tracing::info!(%addr, "GuardAI Hub Event Manager starting");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
