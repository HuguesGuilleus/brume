use axum::{
    body::Bytes,
    extract,
    http::{StatusCode, header::CONTENT_TYPE},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{HTTPState, Result, UserToken, WrapError};

/* HANDLER INPUT / OUTPUT TYPES */

/// Informations of the request for the handler.
#[derive(Debug, Clone)]
pub struct DataRequest<T: DTO> {
    pub user: UserToken,
    pub dto: T,
}

#[derive(Debug, PartialEq)]
pub struct DataResponse<T: Serialize> {
    pub user: Option<UserToken>,
    pub dto: T,
}

pub type DataResponseResult<T> = Result<DataResponse<T>>;

pub fn data_response_ok<T: Serialize>(data: T) -> DataResponseResult<T> {
    Ok(DataResponse {
        dto: data,
        user: None,
    })
}

/// Check if the data is coerent, exist. It do not have acces to the dara.
pub trait DTO: std::fmt::Debug + for<'a> Deserialize<'a> + Default {
    const IS_EMPTY: bool = false;

    fn check(self: &Self) -> Result<()> {
        Ok(())
    }
    fn check_user(self: &Self, _user: &UserToken) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct EmptyDTO();

impl DTO for EmptyDTO {
    const IS_EMPTY: bool = true;
}

/* HANDLER WRAPER */

pub async fn json_handler<S: HTTPState>(
    extract::State(state): extract::State<S>,
    extract::Path(handler): extract::Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    let user = UserToken::default();

    match state.api_json(handler.as_str(), user, &body).await {
        Ok((_user, output)) => (StatusCode::OK, [(CONTENT_TYPE, bmime::JSON)], output),
        Err(err) => {
            let status = err.status_http.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let mut output = format!("{} {}\r\n", status, status.canonical_reason().unwrap_or(""));
            print_err(&mut output, &err);
            (status, [(CONTENT_TYPE, bmime::TEXT)], output.into_bytes())
        }
    }
}

fn print_err(output: &mut String, err: &WrapError) {
    output.push_str(err.description());
    output.push_str("\r\n");

    if let Some(ref b) = err.source_error {
        if let Some(err) = b.downcast_ref::<WrapError>() {
            print_err(output, err);
        }
    }
}

pub async fn api_data_call<S, F, T, U>(
    state: S,
    user: UserToken,
    data: &[u8],
    service: F,
) -> Result<(Option<UserToken>, Vec<u8>)>
where
    F: AsyncFn(S, DataRequest<T>) -> DataResponseResult<U>,
    T: DTO,
    U: Serialize + std::fmt::Debug,
{
    let dto: T = if T::IS_EMPTY {
        T::default()
    } else {
        serde_json::from_slice(&data).map_err(|err| {
            WrapError::http(StatusCode::BAD_REQUEST, "Decoding request JSON body fail").add_err(err)
        })?
    };

    dto.check()?;
    dto.check_user(&user)?;

    let response = service(state, DataRequest { user, dto }).await?;

    let output = serde_json::to_vec(&response.dto).map_err(|err| {
        WrapError::http(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Encoding body JSON response fail",
        )
        .add_err(err)
    })?;

    Ok((response.user, output))
}
