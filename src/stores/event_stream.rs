use crate::events::Event;

#[derive(Default)]
pub struct EventStream {
    pub events: Vec<Event>,
    pub version: u64,
}
