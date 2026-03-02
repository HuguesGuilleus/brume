use super::WrapError;
use axum::http::StatusCode;

pub const TOKEN_PREFIX: WrapError = WrapError {
    desc: "Invalid token prefix, expected prefix 'T0.'",
    status_http: Some(StatusCode::BAD_REQUEST),
    source_error: None,
    argument: None,
};

pub const TOKEN_BASE64: WrapError = WrapError {
    desc: "Invalid base64 data",
    status_http: Some(StatusCode::BAD_REQUEST),
    source_error: None,
    argument: None,
};

pub const TOKEN_TO_SHORT: WrapError = WrapError {
    desc: "The token is too short",
    status_http: Some(StatusCode::BAD_REQUEST),
    source_error: None,
    argument: None,
};

pub const TOKEN_EXPIRED: WrapError = WrapError {
    desc: "The token is expired",
    status_http: Some(StatusCode::UNAUTHORIZED),
    source_error: None,
    argument: None,
};

pub const TOKEN_WRONG_SIGNATURE: WrapError = WrapError {
    desc: "The token signature is invalid",
    status_http: Some(StatusCode::UNAUTHORIZED),
    source_error: None,
    argument: None,
};

pub const TOKEN_WRONG_VALUE: WrapError = WrapError {
    desc: "The token contain value unknown or wrong syntax",
    status_http: Some(StatusCode::UNAUTHORIZED),
    source_error: None,
    argument: None,
};

pub fn sync_fail(_: impl std::error::Error) -> WrapError {
    WrapError {
        desc: "internal sync fail",
        source_error: None,
        argument: None,
        status_http: Some(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub fn enpty_values(entries: &'static str) -> WrapError {
    WrapError {
        desc: "Some values are empty in the data body",
        source_error: Some(Box::new(WrapError {
            desc: entries,
            source_error: None,
            argument: None,
            status_http: None,
        })),
        argument: None,
        status_http: Some(StatusCode::BAD_REQUEST),
    }
}
