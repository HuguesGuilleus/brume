mod error;
mod fs_mem;
mod handler_json;
mod user_token;

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tokio;

pub use error::*;
pub use fs_mem::*;
pub use handler_json::json_handler;
pub use user_token::*;

/// Access to all information for the handler.
pub struct Server {
    pub counter: std::sync::atomic::AtomicU64,
}

/// Informations of the request for the handler.
pub struct JsonRequest<T: DTO> {
    pub user: UserToken,
    pub dto: T,
}

/// Check if the data is coerent, exist. It do not have acces to the dara.
pub trait DTO: Debug + for<'a> Deserialize<'a> {
    fn check(self: &Self) -> Result<()> {
        Ok(())
    }
    fn check_user(self: &Self, _user: &UserToken) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct JsonResponse<T: Serialize + Debug + PartialEq> {
    pub user: Option<UserToken>,
    pub dto: T,
}

pub type JsonResult<T> = Result<JsonResponse<T>>;

pub fn json_response_ok<T: Serialize + Debug + PartialEq>(data: T) -> JsonResult<T> {
    Ok(JsonResponse {
        user: None,
        dto: data,
    })
}

////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CounterAddDTO {
    pub nb: u64,
}

impl DTO for CounterAddDTO {
    fn check(self: &Self) -> Result<()> {
        match self.nb {
            0 => Err(WrapError::http(
                StatusCode::BAD_REQUEST,
                "nb: 0 is not accepted",
            )),
            _ => Ok(()),
        }
    }

    fn check_user(self: &Self, _user: &UserToken) -> Result<()> {
        Ok(())
    }
}

pub async fn service_counter_add(
    server: &Server,
    dto: &JsonRequest<CounterAddDTO>,
) -> JsonResult<CounterAddDTO> {
    json_response_ok(CounterAddDTO {
        nb: dto.dto.nb
            + server
                .counter
                .fetch_add(dto.dto.nb, std::sync::atomic::Ordering::AcqRel),
    })
}

#[tokio::test]
async fn feature() {
    let s = Server {
        counter: std::sync::atomic::AtomicU64::new(1),
    };
    let out = service_counter_add(
        &s,
        &JsonRequest {
            user: UserToken::default(),
            dto: CounterAddDTO { nb: 2 },
        },
    )
    .await;

    assert_eq!(
        out.unwrap(),
        JsonResponse {
            user: None,
            dto: CounterAddDTO { nb: 3 },
        }
    );

    assert_eq!(s.counter.load(std::sync::atomic::Ordering::Relaxed), 3,);
}
