use super::HTTPState;
use crate::*;
use axum::{
    extract::State,
    http::{HeaderName, StatusCode},
};

pub async fn serve_generated<S: HTTPState + Clone + Send + Sync + 'static>(
    State(state): State<S>,
    uri: axum::http::Uri,
) -> (StatusCode, [(HeaderName, &'static str); 1], Vec<u8>) {
    match state.cached(uri.path()) {
        Some(arc) => {
            let (mime, body) = arc;
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, mime)],
                body.to_vec(),
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, bmime::HTML)],
            S::ERROR_404.to_vec(),
        ),
    }
}
