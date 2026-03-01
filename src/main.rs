use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderValue, StatusCode};
use axum::{Router, routing};
use brume::{self};
use tokio;

const INDEX_HTML: &[u8] = include_bytes!("index.html");

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let server = std::sync::Arc::new(brume::Server::new());
    {
        let mut pages = server.pages.write().unwrap();
        pages.insert(
            "/".to_string(),
            Arc::new((brume::handler_generated::MIME_HTML, INDEX_HTML.to_vec())),
        );
        pages.insert(
            "/a".to_string(),
            Arc::new((
                brume::handler_generated::MIME_TEXT,
                b"Hello from a".to_vec(),
            )),
        );
    }

    let app = brume::handler_generated::router().with_state(server);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
