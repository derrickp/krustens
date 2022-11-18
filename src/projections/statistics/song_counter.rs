use std::cmp::Reverse;

use serde::Serialize;

use crate::track_plays::ArtistName;

use super::{SongCount, TimePlayed};

#[derive(Serialize, Clone, Debug)]
pub struct ArtistSongCounter {
    pub artist_name: ArtistName,
    pub play_details: SongCounter,
}

impl ArtistSongCounter {
    pub fn total_song_plays(&self) -> u64 {
        self.play_details.total_song_plays()
    }

    pub fn total_plays_display(&self) -> String {
        format!("{} - {}", &self.artist_name, self.total_song_plays())
    }

    pub fn max_song_display(&self) -> String {
        format!(
            "{} - {}",
            &self.artist_name,
            self.play_details.max_song_play()
        )
    }

    pub fn max_song_play(&self) -> SongCount {
        self.play_details.max_song_play()
    }
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct SongCounter {
    total_song_plays: u64,
    total_time_played: TimePlayed,
    song_counts: Vec<SongCount>,
}

impl SongCounter {
    pub fn sort_by_song_count(&mut self) {
        self.song_counts
            .sort_by_key(|song_count| Reverse(song_count.1))
    }

    pub fn total_song_plays(&self) -> u64 {
        self.total_song_plays
    }

    pub fn increment_song(&mut self, song_name: &str, time_played: u64) {
        match self
            .song_counts
            .iter_mut()
            .find(|song_play_count| song_play_count.0.eq_ignore_ascii_case(song_name))
        {
            Some(it) => it.1 += 1,
            _ => {
                let song_count = SongCount(song_name.to_string(), 1);
                self.song_counts.push(song_count);
            }
        }
        self.total_song_plays += 1;
        self.total_time_played.add_ms(time_played);
    }

    pub fn all_song_plays(&self) -> Vec<SongCount> {
        self.song_counts.to_vec()
    }

    pub fn max_song_play(&self) -> SongCount {
        self.song_counts
            .iter()
            .cloned()
            .max_by_key(|song_count| song_count.1)
            .unwrap_or_default()
    }
}
