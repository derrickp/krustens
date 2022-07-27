use async_trait::async_trait;
use serde::Serialize;

use super::write_error::WriteError;

#[async_trait]
pub trait Writer<T: Serialize> {
    async fn write(&self, value: &T) -> Result<bool, WriteError>;
}
