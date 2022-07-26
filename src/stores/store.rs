use std::collections::HashMap;

use crate::{
    events::Event,
    persistence::{Reader, Writer},
    stores::message::Message,
};

use super::{
    add_event_error::AddEventError, event_stream::EventStream, get_events_error::GetEventsError,
};

pub type MessageCollection = HashMap<String, Vec<Message>>;

#[derive(Default)]
pub struct Store {
    collection: MessageCollection,
    flush_count: usize,
    current_not_flushed: usize,
}

impl<'a> Store {
    pub fn build(reader: &impl Reader<'a, MessageCollection>) -> Self {
        let messages = reader.read().unwrap_or_default();

        Self {
            collection: messages,
            flush_count: 1000,
            current_not_flushed: 0,
        }
    }

    pub fn stream_version(&self, stream: String) -> u64 {
        match self.collection.get(&stream) {
            Some(it) => it.len() as u64,
            _ => 0,
        }
    }

    pub fn add_event(
        &mut self,
        stream: String,
        event: &Event,
        expected_version: u64,
        writer: &impl Writer<MessageCollection>,
    ) -> Result<bool, AddEventError> {
        let message_count = self
            .collection
            .entry(stream.clone())
            .or_insert_with(Vec::new)
            .len();

        if expected_version < message_count as u64 {
            return Err(AddEventError);
        }

        let message = Message {
            stream: stream.clone(),
            position: expected_version + 1,
            data: event.data.clone(),
        };

        self.collection
            .entry(stream)
            .or_insert_with(Vec::new)
            .push(message);
        self.current_not_flushed += 1;

        if self.current_not_flushed > self.flush_count {
            writer.write(&self.collection).unwrap();
            self.current_not_flushed = 0;
        }

        Ok(true)
    }

    pub fn flush(&mut self, writer: &impl Writer<MessageCollection>) {
        writer.write(&self.collection).unwrap();
        self.current_not_flushed = 0;
    }

    pub fn get_events(&self, stream: String) -> Result<EventStream, GetEventsError> {
        let message_stream = match self.collection.get(&stream) {
            Some(it) => it,
            _ => return Ok(EventStream::default()),
        };

        let events: Vec<Event> = message_stream
            .iter()
            .map(|message| Event {
                version: message.position,
                data: message.data.clone(),
            })
            .collect();

        let version = events.len() as u64;

        Ok(EventStream { events, version })
    }

    pub fn get_events_after(
        &self,
        stream: String,
        version: u64,
    ) -> Result<EventStream, GetEventsError> {
        let message_stream = match self.collection.get(&stream) {
            Some(it) => it,
            _ => return Ok(EventStream::default()),
        };

        let events: Vec<Event> = message_stream
            .iter()
            .filter(|message| message.position > version)
            .map(|message| Event {
                version: message.position,
                data: message.data.clone(),
            })
            .collect();

        let version = events.len() as u64;

        Ok(EventStream { events, version })
    }
}
