use std::fmt::Display;

use serde::Serialize;

use crate::track_plays::{ArtistName, TrackName};

#[derive(Default, Serialize, Clone, Debug)]
pub struct SongCount(pub TrackName, pub u64);

impl Display for SongCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{} - {}", &self.0, self.1))
    }
}

pub struct ArtistAndSongCount {
    pub artist_name: ArtistName,
    pub song_count: SongCount,
}

impl Display for ArtistAndSongCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{} - {} - {}",
            &self.artist_name, &self.song_count.0, self.song_count.1
        ))
    }
}
