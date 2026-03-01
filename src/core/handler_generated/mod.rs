use axum::body::Body;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::{Router, routing};
use std::sync::Arc;

use super::{Server, json_handler};

pub fn router() -> Router<Arc<Server>> {
    Router::new()
        .route(
            "/_api.json/{service}",
            routing::post(json_handler).fallback(method_not_allowed),
        )
        .fallback(method_not_allowed)
        .fallback(routing::get(serve_generated))
}

async fn method_not_allowed() -> impl axum::response::IntoResponse {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        [(axum::http::header::CONTENT_TYPE, MIME_TEXT)],
        "405 Method Not Allowed\r\n",
    )
}

pub const MIME_CSS: HeaderValue = HeaderValue::from_static("text/css");
pub const MIME_HTML: HeaderValue = HeaderValue::from_static("text/html");
pub const MIME_JSON: HeaderValue = HeaderValue::from_static("application/json");
pub const MIME_TEXT: HeaderValue = HeaderValue::from_static("text/plain; charset=UTF-8");

const PAGE404: [u8; 30] = *b"<!DOCTYPE html>404 Not Found\r\n";

async fn serve_generated(
    axum::extract::State(server): axum::extract::State<Arc<Server>>,
    uri: axum::http::Uri,
) -> axum::http::response::Response<Body> {
    let pages = server.pages.read().unwrap();

    let (status, mime, body) = match pages.get(uri.path()) {
        Some(arc) => {
            let (mime, body) = &**arc;
            let body = axum::body::Body::from(body.to_vec());
            (StatusCode::OK, mime.clone(), body)
        }
        None => {
            let body = axum::body::Body::from(PAGE404.to_vec());
            (StatusCode::NOT_FOUND, MIME_HTML, body)
        }
    };

    let mut response = Response::new(body);
    *response.status_mut() = status;

    response
        .headers_mut()
        .append(axum::http::header::CONTENT_TYPE, mime);

    response
}
