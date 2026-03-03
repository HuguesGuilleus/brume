use axum::http::StatusCode;
use io_http::*;

pub fn err_sync_fail(_: impl std::error::Error) -> WrapError {
    WrapError::http(StatusCode::INTERNAL_SERVER_ERROR, "internal sync fail")
}

pub fn err_empty_values(entries: &'static str) -> WrapError {
    WrapError::http(
        StatusCode::BAD_REQUEST,
        "Some values are empty in the data body",
    )
    .add_err(WrapError::new(entries))
}
