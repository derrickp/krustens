use std::cmp::Reverse;

use serde::Serialize;

use super::song_play_count::SongPlayCount;

#[derive(Clone, Default, Debug, Serialize)]
pub struct TimePlayed {
    pub time_ms: u64,
    pub time_sec: f32,
    pub time_min: f32,
    pub time_hr: f32,
}

impl TimePlayed {
    pub fn add_ms(&mut self, time: u64) {
        self.time_ms += time;
        self.time_sec += time as f32 / 1000.0;
        self.time_min += time as f32 / 60000.0;
        self.time_hr += time as f32 / (60000.0 * 60.0);
    }
}

#[derive(Clone, Default, Debug, Serialize)]
pub struct PlayCount {
    pub artist_name: String,
    pub total_plays: u64,
    pub total_time_played: TimePlayed,
    pub song_counts: Vec<SongPlayCount>,
}

impl PlayCount {
    pub fn build(artist_name: String) -> Self {
        Self {
            artist_name,
            total_plays: 0,
            total_time_played: TimePlayed::default(),
            song_counts: Vec::new(),
        }
    }

    pub fn sort_by_song_count(&mut self) {
        self.song_counts.sort_by_key(|song_count| Reverse(song_count.1))
    }

    pub fn total_plays(&self) -> u64 {
        self.total_plays
    }

    pub fn increment_song(&mut self, song_name: &str, time_played: u64) {
        match self
            .song_counts
            .iter_mut()
            .find(|song_play_count| song_play_count.0.eq_ignore_ascii_case(song_name))
        {
            Some(it) => it.1 += 1,
            _ => {
                let song_count = SongPlayCount(song_name.to_string(), 1);
                self.song_counts.push(song_count);
            }
        }
        self.total_plays += 1;
        self.total_time_played.add_ms(time_played);
    }

    pub fn all_song_plays(&self) -> Vec<SongPlayCount> {
        self.song_counts.iter().cloned().collect()
    }

    pub fn max_song_play(&self) -> SongPlayCount {
        self.song_counts
            .iter()
            .cloned()
            .max_by_key(|song_count| song_count.1)
            .unwrap_or_default()
    }

    pub fn max_song_display(&self) -> String {
        let max_song = self.max_song_play();

        format!("{} - {}", self.artist_name.clone(), max_song.display())
    }

    pub fn total_plays_display(&self) -> String {
        format!("{} - {}", self.artist_name, self.total_plays())
    }
}
