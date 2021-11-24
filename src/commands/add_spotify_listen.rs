use crate::{
    events::{event::Event, event_data::EventData, listen_added::ListenAdded},
    projections::listen_tracker::build_tracker,
    spotify::listen::Listen,
    stores::{error::Error, store::Store},
};

pub struct AddSpotifyListen {
    pub listen: Listen,
}

impl AddSpotifyListen {
    pub fn handle(&self, store: &Store) -> Result<Option<Event>, Error> {
        let event_stream = match store.get_events("listens".to_string()) {
            Ok(it) => it,
            Err(e) => return Err(Error::GetEventsError(e)),
        };

        let tracker = build_tracker(event_stream.events);
        if tracker.has_listen(
            &self.listen.artist_name,
            &self.listen.track_name,
            &self.listen.end_time,
        ) {
            return Ok(None);
        }

        let event = Event {
            version: event_stream.version,
            data: EventData::ListenAdded(ListenAdded {
                artist_name: self.listen.artist_name.clone(),
                track_name: self.listen.track_name.clone(),
                end_time: self.listen.end_time.clone(),
                ms_played: self.listen.ms_played,
            }),
        };
        Ok(Some(event))
    }
}
