mod serve_api_data;
mod serve_generated;
mod usertoken;

use crate::*;
use axum::http::header::{CONTENT_TYPE, SET_COOKIE};
use axum::routing;
use axum::{
    Router,
    http::{HeaderValue, StatusCode},
};
pub use serve_api_data::{
    DTO, DataRequest, DataResponse, DataResponseResult, EmptyDTO, api_data_call, data_response_ok,
};
use std::sync::Arc;
use usertoken::encode_user_token;

const USER_COOKIE: &str = "user=";

/// Create a router.
pub fn router<S: HTTPState + Clone + 'static>() -> Router<S> {
    let mut router = Router::new().route(
        "/_api.json/{service}",
        routing::post(serve_api_data::json_handler::<S>).fallback(method_not_allowed),
    );

    for (path, mime, data) in S::ASSETS {
        router = router.route(
            path,
            routing::get(async || ([(CONTENT_TYPE, HeaderValue::from_static(mime))], *data))
                .fallback(method_not_allowed),
        )
    }

    router = router
        .fallback(routing::get(serve_generated::serve_generated::<S>).fallback(method_not_allowed));

    if cfg!(debug_assertions) {
        router = router.route(
            "/!user-token-editor",
            routing::get(
                async |axum::extract::State(state): axum::extract::State<S>| {
                    let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
                    let token =
                        encode_user_token(&UserToken::DEV_EDITOR, state.user_token_key(), now);
                    ([(SET_COOKIE, token.clone())], token)
                },
            ),
        )
    }

    router
}

#[async_trait::async_trait]
impl<S: HTTPState> HTTPState for Arc<S> {
    const ASSETS: &[(&str, &str, &[u8])] = S::ASSETS;

    fn cached(&self, path: &str) -> Option<(&'static str, Arc<Vec<u8>>)> {
        let s: &S = &self;
        s.cached(path)
    }

    const ERROR_404: &[u8] = S::ERROR_404;

    fn user_token_key<'a>(&'a self) -> &'a [u8] {
        let s: &S = &self;
        s.user_token_key()
    }

    async fn api_json(
        &self,
        operation: &str,
        user: UserToken,
        data: &[u8],
    ) -> Result<(Option<UserToken>, Vec<u8>)> {
        let s: &S = &self;
        s.api_json(operation, user, data).await
    }
}

pub async fn method_not_allowed() -> impl axum::response::IntoResponse {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        [(axum::http::header::CONTENT_TYPE, bmime::TEXT)],
        "405 Method Not Allowed\r\n",
    )
}
