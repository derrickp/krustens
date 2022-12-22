use crate::{
    processing::events::{Event, EventData, TrackPlayAdded, TrackSkipped},
    projections::HasListen,
    track_plays::{self, Normalized, TrackPlay},
};

pub struct AddTrackPlay {
    pub track_play: TrackPlay,
    pub min_listen_length: u64,
}

impl AddTrackPlay {
    pub fn handle(&self, tracker: &impl HasListen) -> Option<Event> {
        let normalized =
            match <TrackPlay as TryInto<Normalized>>::try_into(self.track_play.to_owned()) {
                Ok(it) => it,
                Err(_) => return None,
            };

        self.handle_normalized(&normalized, tracker)
            .map(|data| Event {
                data,
                version: tracker.version() + 1,
            })
    }

    pub fn handle_normalized(
        &self,
        listen: &track_plays::Normalized,
        tracker: &impl HasListen,
    ) -> Option<EventData> {
        if tracker.has_listen(
            &listen.artist_name,
            &listen.track_name,
            &listen.formatted_end_time(),
        ) {
            return None;
        }

        if listen.is_skipped() || listen.play_time() <= self.min_listen_length {
            Some(EventData::TrackPlayIgnored(TrackSkipped {
                artist_name: listen.artist_name.clone(),
                track_name: listen.track_name.clone(),
                album_name: listen.album_name.clone(),
                end_time: listen.formatted_end_time(),
                ms_played: listen.play_time(),
                service_hint: listen.service_hint.clone(),
            }))
        } else {
            Some(EventData::TrackPlayAdded(TrackPlayAdded {
                artist_name: listen.artist_name.clone(),
                track_name: listen.track_name.clone(),
                album_name: listen.album_name.clone(),
                end_time: listen.formatted_end_time(),
                ms_played: listen.play_time(),
                service_hint: listen.service_hint.clone(),
            }))
        }
    }
}
