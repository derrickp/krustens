use serde::{Deserialize, Serialize};

use super::Spotify;

#[derive(Serialize, Deserialize, Clone)]
pub enum TrackPlay {
    Spotify(Spotify),
}
