use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct SkippedTrack {
    pub artist_name: String,
    pub song_counts: HashMap<String, u64>,
}

impl SkippedTrack {
    pub fn build(artist_name: String) -> Self {
        Self {
            artist_name,
            song_counts: HashMap::new(),
        }
    }

    pub fn increment_song(&mut self, song_name: &str) {
        *self.song_counts.entry(song_name.to_string()).or_insert(0) += 1;
    }
}
