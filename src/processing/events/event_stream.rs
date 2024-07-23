use super::Event;

#[derive(Default)]
pub struct EventStream {
    pub events: Vec<Event>,
}
