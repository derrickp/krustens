use crate::{
    events::{event::Event, event_data::EventData, listen_added::ListenAdded},
    projections::{has_listen::HasListen, listen_tracker::ListenTracker},
    spotify::listen::Listen,
};

pub struct AddSpotifyListen {
    pub listen: Listen,
}

impl AddSpotifyListen {
    pub fn handle(&self, tracker: &ListenTracker) -> Option<Event> {
        if tracker.has_listen(
            &self.listen.artist_name,
            &self.listen.track_name,
            &self.listen.end_time,
        ) {
            return None;
        }

        let event = Event {
            version: tracker.version + 1,
            data: EventData::ListenAdded(ListenAdded {
                artist_name: self.listen.artist_name.clone(),
                track_name: self.listen.track_name.clone(),
                end_time: self.listen.end_time.clone(),
                ms_played: self.listen.ms_played,
            }),
        };
        Some(event)
    }
}
