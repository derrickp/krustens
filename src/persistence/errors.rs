use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("persistence failed `{0}`")]
    General(String),
}
