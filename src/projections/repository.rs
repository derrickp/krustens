use crate::events::Event;

use super::ListenTracker;

#[async_trait::async_trait]
pub trait ListenTrackerRepository {
    fn get(&self) -> &ListenTracker;
    async fn flush(&mut self);
    async fn project_event(&mut self, event: &Event);
}
