use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};

use crate::{events::event_data::EventData, history::listen::Listen, stores::store::Store};

use super::listen_tracker::{build_id, ListenTracker};

pub struct Repository {
    listen_tracker: ListenTracker,
    path: String,
    dirty: bool,
}

impl Repository {
    pub fn build(path: &str) -> Self {
        let contents = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
        let listen_tracker: ListenTracker =
            serde_json::from_str(&contents).unwrap_or(ListenTracker::default());

        Self {
            listen_tracker,
            path: path.to_string(),
            dirty: false,
        }
    }

    pub fn get_tracker(&mut self, store: &Store) -> &ListenTracker {
        let current_version = self.listen_tracker.version;
        let store_version = store.stream_version("listens".to_string());

        if current_version == store_version {
            return &self.listen_tracker;
        }

        let event_stream = store
            .get_events_after("listens".to_string(), self.listen_tracker.version)
            .unwrap();

        for event in event_stream.events.iter() {
            match &event.data {
                EventData::ListenAdded(listen) => {
                    let id = build_id(&listen.artist_name, &listen.track_name, &listen.end_time);
                    self.listen_tracker.version += 1;
                    self.listen_tracker
                        .listens
                        .insert(id.clone(), Listen::build(&id, &listen))
                }
            };
        }
        self.dirty = true;

        &self.listen_tracker
    }

    pub fn flush(&mut self) {
        if !self.dirty {
            return;
        }

        let file = File::create(&self.path).unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &self.listen_tracker).unwrap();
        writer.flush().unwrap();
        self.dirty = false
    }
}
