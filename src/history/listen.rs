use serde::{Deserialize, Serialize};

use crate::events::listen_added::ListenAdded;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Listen {
    pub id: String,
    pub artist_name: String,
    pub track_name: String,
    pub ms_played: u64,
    pub end_time: String,
}

impl Listen {
    pub fn build(id: &str, listen_added: &ListenAdded) -> Self {
        Self {
            id: id.to_string(),
            artist_name: listen_added.artist_name.clone(),
            track_name: listen_added.track_name.clone(),
            ms_played: listen_added.ms_played,
            end_time: listen_added.end_time.clone(),
        }
    }
}
