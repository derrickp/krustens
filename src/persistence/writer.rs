use async_trait::async_trait;
use serde::Serialize;

use crate::errors::WriteError;

use super::Format;

#[async_trait]
pub trait Writer {
    async fn write<T: Serialize + Sync>(
        &self,
        value: &T,
        name: &str,
        format: Format,
    ) -> Result<bool, WriteError>;
}
