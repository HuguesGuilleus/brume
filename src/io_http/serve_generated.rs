use axum::{
    extract,
    http::{HeaderName, StatusCode},
};

use super::HTTPState;
use crate::base::bmime;

pub async fn serve_generated<S: HTTPState + Clone + Send + Sync + 'static>(
    extract::State(state): axum::extract::State<S>,
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
