use axum::Router;

pub fn build_router() -> Router {
    Router::new().nest("/v1", crate::routes::router_v1())
}
