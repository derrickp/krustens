use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Unable to serialize data with error {message:?})")]
    FailedToDeserializeJson { message: String },
}
