use crate::events::event::Event;

pub struct EventStream {
    pub events: Vec<Event>,
    pub version: u64,
}
