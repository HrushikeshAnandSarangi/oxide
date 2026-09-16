use axum::http::header;
use axum::response::IntoResponse;

pub async fn metrics() -> impl IntoResponse {
    let buffer = controller::metrics::gather();
    (
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        buffer,
    )
}
