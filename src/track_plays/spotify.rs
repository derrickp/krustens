use serde::{Deserialize, Serialize};
use std::u64;

#[derive(Serialize, Deserialize, Clone)]
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
