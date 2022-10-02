use thiserror::Error;

#[derive(Debug, Error)]
pub enum AddEventError {
    #[error("Expected version ({expected_version:?}) less than current ({current_version:?})")]
    VersionOutOfDate {
        expected_version: u32,
        current_version: u32,
    },
}

#[derive(Debug, Error)]
pub enum GetEventsError {
    #[error("Unable to read stream for source {event_source:?}) with error ({message:?})")]
    UnableToReadStream {
        message: String,
        event_source: String,
    },
}
