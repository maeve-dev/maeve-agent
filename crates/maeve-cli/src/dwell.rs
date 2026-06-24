use axum::response::Html;

/// Serves the quiet dwelling page — nothing is required of you here.
pub async fn page() -> Html<&'static str> {
    Html(include_str!("../assets/dwell.html"))
}

/// Serves the landing / index page.
pub async fn index() -> Html<&'static str> {
    Html(include_str!("../assets/index.html"))
}

/// Serves the dashboard page.
pub async fn dashboard() -> Html<&'static str> {
    Html(include_str!("../assets/dashboard.html"))
}
