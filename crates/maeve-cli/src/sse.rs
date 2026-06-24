use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;

/// Build the router with all routes registered.
pub fn router() -> Router {
    Router::new()
        // Landing / index page
        .route("/", get(crate::dwell::index))
        // Dwelling page
        .route("/dwell", get(crate::dwell::page))
        // Dashboard page
        .route("/dashboard", get(crate::dwell::dashboard))
        // SSE endpoint (placeholder for future)
        // .route("/events", get(sse_handler))
        .layer(CorsLayer::permissive())
}
