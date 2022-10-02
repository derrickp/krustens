use async_trait::async_trait;

use crate::errors::ReadError;

#[async_trait]
pub trait Reader<T> {
    async fn read(&self) -> Result<T, ReadError>;
}
