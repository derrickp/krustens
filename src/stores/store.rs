use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};

use crate::{events::event::Event, stores::message::Message};

use super::{
    add_event_error::AddEventError, event_stream::EventStream, get_events_error::GetEventsError,
};

#[derive(Default)]
pub struct Store {
    collection: Vec<Message>,
    storage_path: String,
    flush_count: usize,
    current_not_flushed: usize,
}

impl Store {
    pub fn build(path: String) -> Self {
        let contents = fs::read_to_string(&path).unwrap_or_else(|_| "[]".to_string());
        let messages: Vec<Message> = serde_json::from_str(&contents).unwrap();

        Self {
            collection: messages,
            storage_path: path,
            flush_count: 100,
            current_not_flushed: 0,
        }
    }

    pub fn add_event(
        &mut self,
        stream: String,
        event: &Event,
        expected_version: u64,
    ) -> Result<bool, AddEventError> {
        let message_count = self
            .collection
            .iter()
            .filter(|message| message.stream.eq_ignore_ascii_case(&stream))
            .count();

        if expected_version < message_count as u64 {
            return Err(AddEventError);
        }

        let message = Message {
            stream,
            position: expected_version + 1,
            data: event.data.clone(),
        };
        self.collection.push(message);
        self.current_not_flushed += 1;

        if self.current_not_flushed > self.flush_count {
            let file = File::create(self.storage_path.clone()).unwrap();
            let mut writer = BufWriter::new(file);
            serde_json::to_writer(&mut writer, &self.collection).unwrap();
            writer.flush().unwrap();
            self.current_not_flushed = 0;
        }

        Ok(true)
    }

    pub fn get_events(&self, stream: String) -> Result<EventStream, GetEventsError> {
        let events: Vec<Event> = self
            .collection
            .iter()
            .filter_map(|message| {
                if message.stream.eq_ignore_ascii_case(&stream) {
                    Some(Event {
                        version: message.position,
                        data: message.data.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        let version = events.len() as u64;

        Ok(EventStream { events, version })
    }
}
