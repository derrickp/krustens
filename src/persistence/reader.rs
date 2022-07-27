use async_trait::async_trait;

use super::read_error::ReadError;

#[async_trait]
pub trait Reader<T> {
    async fn read(&self) -> Result<T, ReadError>;
}
