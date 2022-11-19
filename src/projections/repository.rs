use super::ListenTracker;

#[async_trait::async_trait]
pub trait ListenTrackerRepository {
    async fn get(&mut self) -> &ListenTracker;
    async fn flush(&mut self);
}
