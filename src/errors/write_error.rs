use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("Unable to create file (path {path:?}, with message {message:?})")]
    CannotCreateFile { path: String, message: String },
    #[error("Unable to serialize data with error {message:?})")]
    FailedToSerializeJson { message: String },
    #[error("Unable to serialize data with error {message:?})")]
    FailedToSerializeYaml { message: String },
    #[error("Unable to write to file (path {path:?}, with message {message:?})")]
    CannotWriteToFile { path: String, message: String },
}
