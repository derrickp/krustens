use serde::Deserialize;

use crate::{
    track_plays::{AlbumName, ArtistName, Normalized, TrackName},
    utils::parse_end_time_rfc3339,
};

#[derive(Clone, Debug, Deserialize)]
pub struct PlayActivity {
    #[serde(alias = "Album Name")]
    pub album_name: Option<String>,
    #[serde(alias = "Artist Name")]
    pub artist_name: String,
    #[serde(alias = "End Reason Type")]
    pub end_reason_type: String,
    #[serde(alias = "Event Type")]
    pub event_type: String,
    #[serde(alias = "Song Name")]
    pub song_name: String,
    #[serde(alias = "Event End Timestamp")]
    pub event_end_timestamp: String,
    #[serde(alias = "Feature Name")]
    pub feature_name: String,
    #[serde(alias = "Media Duration In Milliseconds")]
    pub media_duration_ms: Option<u64>,
    #[serde(alias = "Play Duration Milliseconds")]
    pub play_duration_ms: Option<i64>,
}

impl PlayActivity {
    pub fn identifying_header() -> String {
        "Event Type".to_string()
    }

    pub fn is_end_event(&self) -> bool {
        self.event_type.eq_ignore_ascii_case("play_end")
    }

    pub fn is_skipped_by_percent(&self) -> bool {
        self.play_duration_ms
            .zip(self.media_duration_ms)
            .map(|(ms_played, track_ms)| (ms_played as f64 / track_ms as f64) < 0.1)
            .unwrap_or(false)
    }
}

impl TryInto<Normalized> for PlayActivity {
    type Error = ();

    fn try_into(self) -> Result<Normalized, Self::Error> {
        let ms_played: Option<u64> = match self.play_duration_ms {
            Some(ms_played) => {
                if ms_played < 0 {
                    return Err(());
                }

                ms_played.try_into().ok()
            }
            None => None,
        };

        let end_time = match parse_end_time_rfc3339(&self.event_end_timestamp) {
            Ok(it) => it,
            Err(_) => return Err(()),
        };

        Ok(Normalized {
            end_time,
            ms_played,
            skipped: Some(self.is_skipped_by_percent()),
            album_name: self.album_name.map(AlbumName),
            artist_name: ArtistName(self.artist_name),
            track_name: TrackName(self.song_name),
            track_ms: self.media_duration_ms,
            service_hint: "apple_music".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use crate::track_plays::Normalized;

    use super::PlayActivity;

    #[test]
    fn deserialization() {
        let mut reader =
            csv::Reader::from_path("./fixtures/apple_music_play_activity.csv").unwrap();
        assert!(reader
            .deserialize::<PlayActivity>()
            .map(|r| r.unwrap())
            .next()
            .is_some());
    }

    #[test]
    fn into_normalized() {
        let play_activity = PlayActivity {
            album_name: None,
            artist_name: "Goatwhore".to_string(),
            end_reason_type: "NATURAL_END_OF_TRACK".to_string(),
            event_type: "PLAY_END".to_string(),
            song_name: "Nihil".to_string(),
            event_end_timestamp: "2022-10-07T18:39:52.592Z".to_string(),
            feature_name: "search / artist_detail / album_detail".to_string(),
            media_duration_ms: Some(192000),
            play_duration_ms: Some(192000),
        };

        let normalized: Normalized = play_activity.try_into().unwrap();

        assert_eq!(normalized.end_time.date().year(), 2022);
    }
}
