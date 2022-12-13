use std::collections::HashSet;

use crate::track_plays::{ArtistName, TrackName};

use super::has_listen::HasListen;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ListenTracker {
    pub listens: HashSet<String>,
    pub version: u32,
}

impl HasListen for ListenTracker {
    fn has_listen(&self, artist_name: &ArtistName, track_name: &TrackName, end_time: &str) -> bool {
        let id = build_id(artist_name, track_name, end_time);
        self.listens.contains(&id)
    }

    fn version(&self) -> u32 {
        self.version
    }
}

pub fn build_id(artist_name: &ArtistName, track_name: &TrackName, end_time: &str) -> String {
    format!("{artist_name}-{track_name}-{end_time}")
}
