use serde::{Deserialize, Serialize};

use crate::track_plays::{ArtistName, TrackName};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackPlayAdded {
    pub artist_name: ArtistName,
    pub track_name: TrackName,
    pub ms_played: u64,
    pub end_time: String,
    pub service_hint: String,
}
