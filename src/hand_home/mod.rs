use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    DTO, EmptyDTO, JsonRequest, JsonResult, MIME_TEXT, Page, Server,
    errw::{self, sync_fail},
    json_response_ok,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Data {
    pub title: String,
    pub description: String,
    pub body: String,
}

impl DTO for Data {
    fn check(self: &Self) -> crate::Result<()> {
        if self.title.len() == 0 || self.description.len() == 0 || self.body.len() == 0 {
            Err(errw::enpty_values("need: title, description, body"))
        } else {
            Ok(())
        }
    }
}

pub fn init(server: &Server) -> crate::Result<()> {
    let mut page = server.home.lock().map_err(sync_fail)?;

    *page = Page {
        title: "Brume server".to_string(),
        description: "The brume server home page.".to_string(),
        body: "Yolo".to_string(),
    };
    render(server, &page)?;

    Ok(())
}

pub async fn set(server: &Server, JsonRequest { dto, .. }: &JsonRequest<Data>) -> JsonResult<Data> {
    println!("home.set");

    let mut home = server.home.lock().map_err(sync_fail)?;
    home.title = dto.title.clone();
    home.description = dto.description.clone();
    home.body = dto.body.clone();
    json_response_ok(Data {
        title: dto.title.clone(),
        description: dto.description.clone(),
        body: dto.body.clone(),
    })
}

pub async fn get(server: &Server, _: &JsonRequest<EmptyDTO>) -> JsonResult<Data> {
    let home = server.home.lock().map_err(sync_fail)?;
    json_response_ok(Data {
        title: home.title.clone(),
        description: home.description.clone(),
        body: home.body.clone(),
    })
}

fn render(server: &Server, page: &Page) -> crate::Result<()> {
    let content = format!(
        "{}\r\n=====\r\n\r\n*{}*\r\n\r\n{}",
        page.title, page.description, page.body,
    );

    let mut pages = server.pages.write().map_err(sync_fail)?;
    pages.insert(String::from("/"), Arc::new((MIME_TEXT, content.into())));

    Ok(())
}
