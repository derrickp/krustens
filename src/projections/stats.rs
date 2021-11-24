use std::{cmp::Reverse, collections::HashMap};

use crate::events::{event::Event, event_data::EventData};

pub struct Stats {
    pub stats: HashMap<String, u64>,
}

impl Stats {
    pub fn count_by_artist(events: Vec<&Event>) -> Self {
        let mut stats: HashMap<String, u64> = HashMap::new();

        for event in events {
            match &event.data {
                EventData::ListenAdded(listen) => stats.insert(
                    listen.artist_name.clone(),
                    stats.get(&listen.artist_name).unwrap_or(&0) + 1,
                ),
            };
        }

        Self { stats }
    }

    pub fn count_by_track(events: Vec<&Event>) -> Self {
        let mut stats: HashMap<String, u64> = HashMap::new();

        for event in events {
            match &event.data {
                EventData::ListenAdded(listen) => stats.insert(
                    listen.track_name.clone(),
                    stats.get(&listen.track_name).unwrap_or(&0) + 1,
                ),
            };
        }

        Self { stats }
    }

    pub fn top(&self, count: usize) -> Vec<PlayCount> {
        let mut artist_counts: Vec<PlayCount> = self
            .stats
            .iter()
            .map(|(name, count)| PlayCount {
                name: name.clone(),
                play_count: *count,
            })
            .collect();
        artist_counts.sort_by_key(|play| Reverse(play.play_count));
        artist_counts.into_iter().take(count).collect()
    }
}

pub struct PlayCount {
    pub name: String,
    pub play_count: u64,
}
