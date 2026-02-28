use super::*;
use axum::{self, extract, response::IntoResponse};
use bytes::Bytes;
use std::sync::Arc;

pub async fn json_handler(
    extract::State(server): extract::State<Arc<Server>>,
    extract::Path(service): extract::Path<String>,
    body: Bytes,
) -> std::result::Result<impl IntoResponse, (StatusCode, String)> {
    match json_mux(&server, service.as_str(), body).await {
        Ok(data) => Ok((
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            data,
        )),
        Err(ref err) => {
            let mut buf = String::new();
            if let Some(status_http) = err.status_http {
                buf.push_str(status_http.as_str());
                buf.push_str(" ");
                buf.push_str(status_http.canonical_reason().unwrap_or(""));
            } else {
                buf.push_str("500");
            }
            buf.push_str("\r\n");
            print_err(&mut buf, err);

            Err((
                err.status_http.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                buf,
            ))
        }
    }
}

fn print_err(buf: &mut String, err: &WrapError) {
    buf.push_str(err.description());
    buf.push_str("\r\n");

    if let Some(ref b) = err.source_error {
        if let Some(err) = b.downcast_ref::<WrapError>() {
            print_err(buf, err);
        }
    }
}

async fn json_mux(server: &Server, service: &str, body: Bytes) -> Result<Vec<u8>> {
    match service {
        "counter" => caller(&server, body, service_counter_add).await,
        _ => return Err(WrapError::http(StatusCode::NOT_FOUND, "Service not found")),
    }
}

async fn caller<F, T, U>(server: &Server, body: Bytes, service: F) -> Result<Vec<u8>>
where
    F: AsyncFnOnce(&Server, &JsonRequest<T>) -> JsonResult<U>,
    T: DTO,
    U: Serialize + Debug + PartialEq,
{
    let user = UserToken::default();

    let dto: T = serde_json::from_slice(&body)
        .map_err(|_| WrapError::http(StatusCode::BAD_REQUEST, "Decoding request JSON body fail"))?;

    dto.check()?;
    dto.check_user(&user)?;

    let response = service(server, &JsonRequest { user, dto }).await?;

    let output = serde_json::to_vec(&response.dto).map_err(|_| {
        WrapError::http(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Encoding body JSON response fail",
        )
    })?;

    Ok(output)
}
