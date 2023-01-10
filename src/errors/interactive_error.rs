use thiserror::Error;

use super::GetEventsError;

#[derive(Debug, Error)]
pub enum InteractiveError {
    #[error("crossterm Error {message:?}")]
    Crossterm { message: String },
    #[error("tui Error {message:?}")]
    TuiError { message: String },
    #[error("parsing Error {message:?}")]
    ParsingIssue { message: String },
    #[error("error getting events {error:?}")]
    GetEventsError { error: GetEventsError },
    #[error("error with clipboard {message:?}")]
    ClipboardError { message: String },
}
