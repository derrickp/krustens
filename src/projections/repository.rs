use crate::{
    events::event_data::EventData,
    persistence::{reader::Reader, writer::Writer},
    stores::store::Store,
};

use super::{
    has_listen::HasListen,
    listen_tracker::{build_id, ListenTracker},
};

pub struct Repository {
    listen_tracker: ListenTracker,
    dirty: bool,
    buffer_count: usize,
    not_persisted_count: usize,
}

impl<'a> Repository {
    pub fn build(buffer_count: usize, reader: &impl Reader<'a, ListenTracker>) -> Self {
        let listen_tracker = reader.read().unwrap_or_default();

        Self {
            listen_tracker,
            buffer_count,
            dirty: false,
            not_persisted_count: 0,
        }
    }

    pub fn get_tracker(
        &mut self,
        store: &Store,
        writer: &impl Writer<ListenTracker>,
    ) -> &impl HasListen {
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
                EventData::TrackPlayAdded(listen) => {
                    let id = build_id(&listen.artist_name, &listen.track_name, &listen.end_time);
                    self.listen_tracker.version += 1;
                    self.listen_tracker.listens.insert(id.clone())
                }
                EventData::TrackPlayIgnored(ignored) => {
                    let id = build_id(&ignored.artist_name, &ignored.track_name, &ignored.end_time);
                    self.listen_tracker.version += 1;
                    self.listen_tracker.listens.insert(id.clone())
                }
            };
            self.not_persisted_count += 1;
        }
        self.dirty = true;

        if self.not_persisted_count >= self.buffer_count {
            writer.write(&self.listen_tracker).unwrap();
            self.reset_persistence();
        }

        &self.listen_tracker
    }

    pub fn reset_persistence(&mut self) {
        self.dirty = false;
        self.not_persisted_count = 0;
    }
}
