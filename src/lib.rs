pub mod app_driver;
pub mod bmime;
mod error;
pub mod io_http;
mod usertoken;

pub use error::*;
use std::sync::Arc;
pub use usertoken::*;

#[async_trait::async_trait]
pub trait HTTPState: Send + Sync {
    /// Constant static assets.
    /// `(absolute_path, mime_type, content)`
    const ASSETS: &[(&str, &str, &[u8])];

    /// Already generated page
    fn cached(&self, path: &str) -> Option<(&'static str, Arc<Vec<u8>>)>;
    /// Page returned when not found ressource.
    const ERROR_404: &[u8];

    /// Key to sign user token.
    fn user_token_key<'a>(&'a self) -> &'a [u8];

    async fn api_json(
        &self,
        operation: &str,
        user: UserToken,
        data: &[u8],
    ) -> Result<(Option<UserToken>, Vec<u8>)>;
}
