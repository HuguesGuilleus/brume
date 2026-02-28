use axum::http::StatusCode;
use std::fmt;

pub type Result<T> = std::result::Result<T, WrapError>;

#[derive(Debug)]
pub struct WrapError {
    /// A simple description string, without new line, returned to the client.
    desc: &'static str,
    /// Sub error
    pub source_error: Option<Box<dyn std::error::Error + 'static>>,
    /// Some argument, logged, but not returned to client.
    pub argument: Option<String>,
    // A status code returned to the HTTP client.
    pub status_http: Option<StatusCode>,
}

impl WrapError {
    pub fn new(description: &'static str) -> Self {
        Self {
            source_error: None,
            desc: description,
            argument: None,
            status_http: None,
        }
    }

    pub fn http(status_http: StatusCode, description: &'static str) -> Self {
        Self {
            desc: description,
            source_error: None,
            argument: None,
            status_http: Some(status_http),
        }
    }

    pub fn wrap(mut self, source_error: Box<dyn std::error::Error>) -> Self {
        self.source_error = Some(source_error);
        self
    }

    /// Add the argument of the failing operation.
    /// The previous argument are replaced.
    pub fn arg(mut self, argument: String) -> Self {
        self.argument = Some(argument);
        self
    }

    pub fn description(&self) -> &'static str {
        self.desc
    }
}

impl fmt::Display for WrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.desc)?;

        if let Some(arg) = &self.argument {
            write!(f, " {:?}", arg)?;
        }

        if let Some(src) = &self.source_error {
            write!(f, " {}", src)?;
        }

        Ok(())
    }
}

impl std::error::Error for WrapError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source_error.as_deref()
    }

    fn description(&self) -> &'static str {
        self.desc
    }
}
