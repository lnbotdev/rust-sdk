use thiserror::Error;

/// Errors returned by the LnBot SDK.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum LnBotError {
    /// The server rejected the request as invalid (HTTP 400).
    #[error("Bad Request (400): {body}")]
    BadRequest { body: String },

    /// The requested resource was not found (HTTP 404).
    #[error("Not Found (404): {body}")]
    NotFound { body: String },

    /// The request conflicted with existing state (HTTP 409).
    #[error("Conflict (409): {body}")]
    Conflict { body: String },

    /// An API error with a non-standard status code.
    #[error("API error (HTTP {status}): {body}")]
    Api { status: u16, body: String },

    /// An HTTP transport error from `reqwest`.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// A JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub(crate) fn from_status(status: u16, body: String) -> LnBotError {
    match status {
        400 => LnBotError::BadRequest { body },
        404 => LnBotError::NotFound { body },
        409 => LnBotError::Conflict { body },
        _ => LnBotError::Api { status, body },
    }
}
