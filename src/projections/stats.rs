use std::{cmp::Reverse, collections::HashMap};

use crate::events::{event::Event, event_data::EventData};

pub struct Stats {
    pub stats: HashMap<String, PlayCount>,
}

impl Stats {
    pub fn count_by_artist(events: Vec<&Event>) -> Self {
        let mut stats: HashMap<String, PlayCount> = HashMap::new();

        for event in events {
            match &event.data {
                EventData::ListenAdded(listen) => stats
                    .entry(listen.artist_name.clone())
                    .or_insert(PlayCount::build(
                        listen.artist_name.clone(),
                        listen.artist_name.clone(),
                    ))
                    .increment(),
            };
        }

        Self { stats }
    }

    pub fn count_by_track(events: Vec<&Event>) -> Self {
        let mut stats: HashMap<String, PlayCount> = HashMap::new();

        for event in events {
            match &event.data {
                EventData::ListenAdded(listen) => stats
                    .entry(listen.track_name.clone())
                    .or_insert(PlayCount::build(
                        listen.track_name.clone(),
                        listen.artist_name.clone(),
                    ))
                    .increment(),
            };
        }

        Self { stats }
    }

    pub fn top(&self, count: usize) -> Vec<PlayCount> {
        let mut counts: Vec<PlayCount> = self.stats.values().cloned().collect();
        counts.sort_by_key(|play| Reverse(play.play_count));
        counts.into_iter().take(count).collect()
    }

    pub fn top_unique_artists(&self, count: usize) -> Vec<PlayCount> {
        let counts: Vec<PlayCount> = self.stats.values().cloned().collect();
        let mut tracker: HashMap<String, PlayCount> = HashMap::new();

        for count in counts.iter() {
            if count.play_count > tracker.entry(count.artist_name.clone()).or_default().play_count {
                tracker.insert(count.artist_name.clone(), count.clone());
            }
        }

        let mut unique_counts: Vec<PlayCount> = tracker.values().cloned().collect();
        unique_counts.sort_by_key(|play| Reverse(play.play_count));
        unique_counts.into_iter().take(count).collect()
    }
}

#[derive(Clone, Default)]
pub struct PlayCount {
    pub name: String,
    pub artist_name: String,
    pub play_count: u64,
}

impl PlayCount {
    pub fn build(name: String, artist_name: String) -> Self {
        Self {
            name,
            artist_name,
            play_count: 0,
        }
    }

    pub fn increment(&mut self) {
        self.play_count += 1
    }
}
