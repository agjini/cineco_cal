use std::fmt;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum Error {
    // NotFound(String),
    Unreachable(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Error::NotFound(ref cause) => write!(f, "Not found: {}", cause),
            Error::Unreachable(ref cause) => {
                write!(f, "Unreachable: {}", cause)
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            // Error::NotFound(message) => (StatusCode::NOT_FOUND, message),
            Error::Unreachable(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
        };

        format!("status = {}, message = {}", status, error_message).into_response()
    }
}
