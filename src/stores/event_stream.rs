use crate::events::event::Event;

pub struct EventStream {
    pub events: Vec<Event>,
    pub version: u64,
}

impl Default for EventStream {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            version: 0,
        }
    }
}
