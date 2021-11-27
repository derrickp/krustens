use std::{cmp::Reverse, collections::HashMap};

use serde::{Deserialize, Serialize};

use crate::events::{event::Event, event_data::EventData};

#[derive(Debug, Deserialize, Serialize)]
pub struct Stats {
    pub stats: HashMap<String, PlayCount>,
    pub skipped: HashMap<String, SkippedTrack>,
}

impl Stats {
    pub fn count(events: Vec<&Event>) -> Self {
        let mut stats: HashMap<String, PlayCount> = HashMap::new();
        let mut skipped: HashMap<String, SkippedTrack> = HashMap::new();

        for event in events {
            match &event.data {
                EventData::TrackPlayAdded(listen) => stats
                    .entry(listen.artist_name.clone())
                    .or_insert_with(|| PlayCount::build(listen.artist_name.clone()))
                    .increment_song(&listen.track_name),
                EventData::TrackPlayIgnored(ignored) => skipped
                    .entry(ignored.artist_name.clone())
                    .or_insert_with(|| SkippedTrack::build(ignored.artist_name.clone()))
                    .increment_song(&ignored.track_name),
            }
        }

        Self { stats, skipped }
    }

    pub fn artist_count(&self) -> usize {
        self.stats.len()
    }

    pub fn top(&self, count: usize) -> Vec<PlayCount> {
        let mut counts: Vec<PlayCount> = self.stats.values().cloned().collect();
        counts.sort_by_key(|play| Reverse(play.total_plays()));
        counts.into_iter().take(count).collect()
    }

    pub fn top_songs(&self, count: usize) -> Vec<SongPlayCount> {
        let mut counts: Vec<SongPlayCount> = self
            .stats
            .values()
            .cloned()
            .flat_map(|play_count| play_count.all_song_plays())
            .collect();
        counts.sort_by_key(|song_play_count| Reverse(song_play_count.count));
        counts.into_iter().take(count).collect()
    }

    pub fn top_unique_artists(&self, count: usize) -> Vec<PlayCount> {
        let mut counts: Vec<PlayCount> = self.stats.values().cloned().collect();

        counts.sort_by_key(|play| Reverse(play.max_song_play().count));
        counts.into_iter().take(count).collect()
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct PlayCount {
    pub artist_name: String,
    pub song_counts: HashMap<String, u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

impl PlayCount {
    pub fn build(artist_name: String) -> Self {
        Self {
            artist_name,
            song_counts: HashMap::new(),
        }
    }

    pub fn total_plays(&self) -> u64 {
        self.song_counts.values().copied().sum()
    }

    pub fn increment_song(&mut self, song_name: &str) {
        *self.song_counts.entry(song_name.to_string()).or_insert(0) += 1;
    }

    pub fn all_song_plays(&self) -> Vec<SongPlayCount> {
        self.song_counts
            .iter()
            .map(|(song_name, count)| SongPlayCount {
                artist_name: self.artist_name.clone(),
                song_name: song_name.clone(),
                count: *count,
            })
            .collect()
    }

    pub fn max_song_play(&self) -> SongPlayCount {
        self.song_counts
            .iter()
            .map(|(song_name, count)| SongPlayCount {
                artist_name: self.artist_name.clone(),
                song_name: song_name.clone(),
                count: *count,
            })
            .max_by_key(|song_count| song_count.count)
            .unwrap_or_default()
    }
}

#[derive(Default)]
pub struct SongPlayCount {
    pub artist_name: String,
    pub song_name: String,
    pub count: u64,
}
