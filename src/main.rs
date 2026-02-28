use axum::http::StatusCode;
use axum::{Router, routing};
use brume::{self};
use tokio;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let server = std::sync::Arc::new(brume::Server {
        counter: std::sync::atomic::AtomicU64::new(0),
    });

    let app = Router::new()
        .route(
            "/_api.json/{service}",
            routing::post(brume::json_handler).fallback(async || {
                (
                    StatusCode::METHOD_NOT_ALLOWED,
                    "\"405 Method not allowed\"\r\n",
                )
            }),
        )
        .with_state(server.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
