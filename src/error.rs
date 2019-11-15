use crate::resource::ErrorObject;
use failure::{self, format_err, Fail};
use reqwest::StatusCode;
use std::fmt;

/// An alias to `Result` with `Err` of `onedrive_api::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// The error may occur when processing requests.
#[derive(Debug)]
pub struct Error {
    inner: Box<ErrorKind>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

#[derive(Debug, Fail)]
enum ErrorKind {
    #[fail(display = "Deserialize error: {}", 0)]
    DeserializeError(failure::Error),
    #[fail(display = "Request error: {} (response: {:?})", source, response)]
    RequestError {
        source: reqwest::Error,
        response: Option<ErrorObject>,
    },
}

impl Error {
    pub(crate) fn from_response(source: reqwest::Error, response: Option<ErrorObject>) -> Self {
        Self {
            inner: Box::new(ErrorKind::RequestError { source, response }),
        }
    }

    pub(crate) fn unexpected_response(reason: &'static str) -> Self {
        Self {
            inner: Box::new(ErrorKind::DeserializeError(format_err!("{}", reason))),
        }
    }

    /// Check whether the error may be recovered by retrying.
    pub fn should_retry(&self) -> bool {
        match &*self.inner {
            ErrorKind::DeserializeError(_) => false,
            ErrorKind::RequestError { source, .. } => {
                !source.is_client_error() && !source.is_serialization()
            }
        }
    }

    /// Get the url related to the error.
    pub fn url(&self) -> Option<&reqwest::Url> {
        match &*self.inner {
            ErrorKind::DeserializeError(_) => None,
            ErrorKind::RequestError { source, .. } => source.url(),
        }
    }

    /// Get the error response from API if caused by error status code.
    pub fn error_response(&self) -> Option<&ErrorObject> {
        match &*self.inner {
            ErrorKind::DeserializeError(_) => None,
            ErrorKind::RequestError { response, .. } => response.as_ref(),
        }
    }

    /// Get the HTTP status code if caused by error status code.
    pub fn status_code(&self) -> Option<StatusCode> {
        match &*self.inner {
            ErrorKind::DeserializeError(_) => None,
            ErrorKind::RequestError { source, .. } => source.status(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(source: reqwest::Error) -> Self {
        Self::from_response(source, None)
    }
}
