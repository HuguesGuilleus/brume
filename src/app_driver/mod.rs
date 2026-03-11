mod error;
mod hand_home;

use crate::{bmime, io_http::*, *};
use std::sync::Arc;

/// Access to all informations for the handlers.
#[derive(Debug)]
pub struct State {
    /// Pre generated pages, ready to send to HTTP client.
    /// Indexed by absolute path.
    /// Value is a MIME type and the content.
    pub pages: std::sync::RwLock<std::collections::BTreeMap<String, (&'static str, Arc<Vec<u8>>)>>,

    /// The page data behind the root path `/`.
    /// Only administrator can edit it.
    pub home: std::sync::Mutex<hand_home::Page>,
}

impl State {
    pub fn new() -> Result<Self> {
        let server = State {
            pages: std::sync::RwLock::new(std::collections::BTreeMap::new()),
            home: hand_home::Page::default().into(),
        };

        hand_home::init(&server)?;

        Ok(server)
    }
}

#[async_trait::async_trait]
impl HTTPState for State {
    const ASSETS: &[(&str, &str, &[u8])] = &[
        (
            "/favicon.ico",
            bmime::ICO,
            include_bytes!("assets/favicon.ico"),
        ),
        (
            "/favicon.webp",
            bmime::WEBP,
            include_bytes!("assets/favicon.webp"),
        ),
    ];

    fn cached(&self, path: &str) -> Option<(&'static str, Arc<Vec<u8>>)> {
        if let Some(pages) = self.pages.read().ok() {
            pages.get(path).cloned()
        } else {
            None
        }
    }

    const ERROR_404: &[u8] = b"<!DOCTYPE html>404 Not Found\r\n";

    /// Key to sign user token.
    fn user_token_key<'a>(&'a self) -> &'a [u8] {
        &[0u8]
    }

    async fn api_json(
        &self,
        operation: &str,
        user: UserToken,
        data: &[u8],
    ) -> Result<(Option<UserToken>, Vec<u8>)> {
        match operation {
            "home.get" => api_data_call(self, user, data, hand_home::get).await,
            "home.set" => api_data_call(self, user, data, hand_home::set).await,
            _ => Ok((None, vec![])),
        }
    }
}
