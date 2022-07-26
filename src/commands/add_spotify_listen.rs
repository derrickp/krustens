use crate::{
    events::{
        Event, EventData, TrackPlayAdded,
        TrackSkipped,
    },
    projections::HasListen,
    spotify::TrackPlay,
};

pub struct AddSpotifyListen {
    pub listen: TrackPlay,
    pub min_listen_length: u64,
}

impl AddSpotifyListen {
    pub fn handle(&self, tracker: &impl HasListen) -> Option<Event> {
        if tracker.has_listen(
            &self.listen.artist_name,
            &self.listen.track_name,
            &self.listen.end_time,
        ) {
            return None;
        }

        let data = if self.listen.ms_played <= self.min_listen_length {
            EventData::TrackPlayIgnored(TrackSkipped {
                artist_name: self.listen.artist_name.clone(),
                track_name: self.listen.track_name.clone(),
                end_time: self.listen.end_time.clone(),
                ms_played: self.listen.ms_played,
            })
        } else {
            EventData::TrackPlayAdded(TrackPlayAdded {
                artist_name: self.listen.artist_name.clone(),
                track_name: self.listen.track_name.clone(),
                end_time: self.listen.end_time.clone(),
                ms_played: self.listen.ms_played,
            })
        };

        Some(Event {
            data,
            version: tracker.version() + 1,
        })
    }
}
