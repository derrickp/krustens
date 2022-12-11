use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackPlayAdded {
    pub artist_name: String,
    pub track_name: String,
    pub ms_played: u64,
    pub end_time: String,
    pub service_hint: String,
}
