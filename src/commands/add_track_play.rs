use crate::{
    events::{Event, EventData, TrackPlayAdded, TrackSkipped},
    projections::HasListen,
    track_plays::{self, TrackPlay},
};

pub struct AddTrackPlay {
    pub track_play: TrackPlay,
    pub min_listen_length: u64,
}

impl AddTrackPlay {
    pub fn handle(&self, tracker: &impl HasListen) -> Option<Event> {
        let event_data = match &self.track_play {
            TrackPlay::Spotify(it) => self.handle_spotify_listen(it, tracker),
        };

        event_data.map(|data| Event {
            data,
            version: tracker.version() + 1,
        })
    }

    pub fn handle_spotify_listen(
        &self,
        listen: &track_plays::Spotify,
        tracker: &impl HasListen,
    ) -> Option<EventData> {
        if tracker.has_listen(&listen.artist_name, &listen.track_name, &listen.end_time) {
            return None;
        }

        if listen.ms_played <= self.min_listen_length {
            Some(EventData::TrackPlayIgnored(TrackSkipped {
                artist_name: listen.artist_name.clone(),
                track_name: listen.track_name.clone(),
                end_time: listen.end_time.clone(),
                ms_played: listen.ms_played,
            }))
        } else {
            Some(EventData::TrackPlayAdded(TrackPlayAdded {
                artist_name: listen.artist_name.clone(),
                track_name: listen.track_name.clone(),
                end_time: listen.end_time.clone(),
                ms_played: listen.ms_played,
            }))
        }
    }
}
