use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Unable to deserialize data with error {message:?} {file_name:?}")]
    FailedToDeserializeJson { message: String, file_name: String },
    #[error("Unable to deserialize data with error {message:?} {file_name:?}")]
    FailedToDeserializeCsv { message: String, file_name: String },
    #[error("Unsupported file type {file_type:?}")]
    UnsupportedFileType { file_type: String },
    #[error("Not a file {file_name:?}")]
    NotAFile { file_name: String },
    #[error("Cannot read file contents {file_name:?} {message:?}")]
    CannotReadContents { file_name: String, message: String },
}
