use async_trait::async_trait;

use crate::events::Event;

use super::{AddEventError, EventStream, GetEventsError};

#[async_trait]
pub trait EventStore {
    async fn stream_version(&self, stream: String) -> u32;
    async fn add_event(
        &self,
        stream: String,
        event: &Event,
        expected_version: u32,
    ) -> Result<(), AddEventError>;
    async fn get_events(&self, stream: String) -> Result<EventStream, GetEventsError>;
    async fn get_events_after(
        &self,
        stream: String,
        version: u32,
    ) -> Result<EventStream, GetEventsError>;
}
