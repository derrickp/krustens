use async_trait::async_trait;

use crate::stores::EventStore;

#[async_trait]
pub trait ProjectionRepository<T: std::marker::Sync> {
    async fn get(&mut self, store: &(impl EventStore + Send + std::marker::Sync)) -> T;
    async fn flush(&mut self);
}
