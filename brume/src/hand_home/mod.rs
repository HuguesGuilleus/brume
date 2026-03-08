use crate::*;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Page {
    pub title: String,
    pub description: String,
    pub body: String,
}

impl DTO for Page {
    fn check(self: &Self) -> crate::Result<()> {
        if self.title.len() == 0 || self.description.len() == 0 || self.body.len() == 0 {
            Err(err_empty_values("need: title, description, body"))
        } else {
            Ok(())
        }
    }
    fn check_user(self: &Self, user: &UserToken) -> Result<()> {
        match user.allow(42, UserLevel::EditData) {
            true => Ok(()),
            false => Err(WrapError::http(
                StatusCode::FORBIDDEN,
                "You can not access to this resources",
            )),
        }
    }
}

pub fn init(server: &State) -> crate::Result<()> {
    let mut page = server.home.lock().map_err(err_sync_fail)?;

    *page = Page {
        title: "Brume server".to_string(),
        description: "The brume server home page.".to_string(),
        body: "Yolo".to_string(),
    };
    render(server, &page)?;

    Ok(())
}

pub async fn set(
    server: &State,
    DataRequest { dto, .. }: DataRequest<Page>,
) -> Result<DataResponse<Page>> {
    let mut home = server.home.lock().map_err(err_sync_fail)?;
    *home = dto.clone();
    data_response_ok(dto)
}

pub async fn get(server: &State, _: DataRequest<EmptyDTO>) -> DataResponseResult<Page> {
    let home = server.home.lock().map_err(err_sync_fail)?;
    data_response_ok(home.clone())
}

fn render(server: &State, page: &Page) -> crate::Result<()> {
    let content = format!(
        "{}\r\n=====\r\n\r\n*{}*\r\n\r\n{}",
        page.title, page.description, page.body,
    );

    let mut pages = server.pages.write().map_err(err_sync_fail)?;
    pages.insert(String::from("/"), (bmime::TEXT, Arc::new(content.into())));

    Ok(())
}
