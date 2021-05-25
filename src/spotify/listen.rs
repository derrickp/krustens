use std::u64;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]

pub struct Listen {
  #[serde(rename = "endTime")]
  pub end_time: String,
  #[serde(rename = "artistName")]
  pub artist_name: String,
  #[serde(rename = "trackName")]
  pub track_name: String,
  #[serde(rename = "msPlayed")]
  pub ms_played: u64
}
