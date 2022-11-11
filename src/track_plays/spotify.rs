use serde::Deserialize;
use std::u64;

use crate::utils::parse_spotify_end_time;

use super::Normalized;

#[derive(Debug, Deserialize, Clone)]
pub struct Spotify {
    #[serde(rename = "endTime")]
    pub end_time: String,
    #[serde(rename = "artistName")]
    pub artist_name: String,
    #[serde(rename = "trackName")]
    pub track_name: String,
    #[serde(rename = "msPlayed")]
    pub ms_played: u64,
}

impl TryInto<Normalized> for Spotify {
    type Error = ();

    fn try_into(self) -> Result<Normalized, Self::Error> {
        let end_time = match parse_spotify_end_time(&self.end_time) {
            Ok(it) => it,
            Err(_) => return Err(()),
        };

        Ok(Normalized {
            end_time,
            artist_name: self.artist_name,
            track_name: self.track_name,
            service_hint: "spotify".to_string(),
            ms_played: Some(self.ms_played),
            track_ms: None,
        })
    }
}
