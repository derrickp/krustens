use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Listen {
    pub id: String,
    pub artist_name: String,
    pub track_name: String,
    pub ms_played: u64,
    pub end_time: String,
}
