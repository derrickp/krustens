use std::{cmp::Reverse, collections::HashMap};

use serde::Serialize;

use crate::track_plays::ArtistName;

use super::{
    song_counter::{ArtistSongCounter, SongCounter},
    ArtistAndSongCount, General, TimePlayed,
};

#[derive(Default, Serialize)]
pub struct ArtistsCounts {
    pub artists: HashMap<ArtistName, SongCounter>,
    pub time_played: TimePlayed,
    pub skipped_artists: HashMap<ArtistName, SongCounter>,
}

impl ArtistsCounts {
    pub fn total_count(&self) -> u64 {
        self.artists
            .values()
            .map(|song_counter| song_counter.total_song_plays())
            .sum()
    }

    pub fn find_artist(&self, name: &ArtistName) -> Option<ArtistSongCounter> {
        self.artists.iter().find_map(|(n, v)| {
            if name.0.to_lowercase() == n.0.to_lowercase() {
                Some(ArtistSongCounter {
                    artist_name: n.clone(),
                    play_details: v.clone(),
                })
            } else {
                None
            }
        })
    }

    pub fn add_song_skip(&mut self, artist_name: &ArtistName, song_name: &str) {
        let artist_counts = self
            .skipped_artists
            .entry(artist_name.clone())
            .or_insert_with(SongCounter::default);
        artist_counts.increment_song(song_name, 0);
    }

    pub fn add_song_play(&mut self, artist_name: &ArtistName, song_name: &str, time_played: u64) {
        self.time_played.add_ms(time_played);
        let song_counter = self
            .artists
            .entry(artist_name.clone())
            .or_insert_with(SongCounter::default);

        song_counter.increment_song(song_name, time_played);
    }

    pub fn over_min_plays(&self, min_plays: u64) -> Vec<ArtistSongCounter> {
        self.artists
            .iter()
            .filter_map(|(name, counter)| {
                if counter.total_song_plays() < min_plays {
                    None
                } else {
                    Some(ArtistSongCounter {
                        artist_name: name.clone(),
                        play_details: counter.clone(),
                    })
                }
            })
            .collect()
    }

    pub fn all(&self) -> Vec<ArtistSongCounter> {
        let mut counts: Vec<ArtistSongCounter> = self
            .artists
            .iter()
            .map(|(name, counter)| ArtistSongCounter {
                artist_name: name.clone(),
                play_details: counter.clone(),
            })
            .collect();
        counts.sort_by_key(|play| Reverse(play.total_song_plays()));
        counts.into_iter().collect()
    }

    pub fn top(&self, count: usize) -> Vec<ArtistSongCounter> {
        let mut counts: Vec<ArtistSongCounter> = self
            .artists
            .iter()
            .map(|(name, counter)| ArtistSongCounter {
                artist_name: name.clone(),
                play_details: counter.clone(),
            })
            .collect();
        counts.sort_by_key(|play| Reverse(play.total_song_plays()));
        counts.into_iter().take(count).collect()
    }

    pub fn general_stats(&self, count: usize) -> General {
        let artist_total_plays: Vec<String> = self
            .top(count)
            .iter()
            .map(ArtistSongCounter::total_plays_display)
            .collect();

        let most_played_songs: Vec<String> = self
            .top_songs(count)
            .iter()
            .map(|artist_count| format!("{}", &artist_count))
            .collect();

        let unique_artists_most_played_songs: Vec<String> = self
            .top_unique_artists(count)
            .into_iter()
            .map(|counter| counter.max_song_display())
            .collect();

        General {
            artist_total_plays,
            most_played_songs,
            artist_most_played_songs: unique_artists_most_played_songs,
            count_artists_listened_to: self.artist_count(),
        }
    }

    pub fn artist_count(&self) -> usize {
        self.artists.len()
    }

    pub fn top_unique_artists(&self, count: usize) -> Vec<ArtistSongCounter> {
        let mut counts: Vec<ArtistSongCounter> = self
            .artists
            .iter()
            .map(|(name, counter)| ArtistSongCounter {
                artist_name: name.clone(),
                play_details: counter.clone(),
            })
            .collect();

        counts.sort_by_key(|counter| Reverse(counter.max_song_play().1));
        counts.into_iter().take(count).collect()
    }

    pub fn top_songs(&self, count: usize) -> Vec<ArtistAndSongCount> {
        let mut counts: Vec<ArtistAndSongCount> = self
            .artists
            .clone()
            .into_iter()
            .flat_map(|(artist_name, play_count)| {
                play_count
                    .all_song_plays()
                    .iter()
                    .map(|song_count| ArtistAndSongCount {
                        artist_name: artist_name.clone(),
                        song_count: song_count.clone(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        counts.sort_by_key(|song_play_count| Reverse(song_play_count.song_count.1));
        counts.into_iter().take(count).collect()
    }

    pub fn top_skipped_songs(&self, count: usize) -> Vec<ArtistAndSongCount> {
        let mut counts: Vec<ArtistAndSongCount> = self
            .skipped_artists
            .clone()
            .into_iter()
            .flat_map(|(artist_name, play_count)| {
                play_count
                    .all_song_plays()
                    .iter()
                    .map(|song_count| ArtistAndSongCount {
                        artist_name: artist_name.clone(),
                        song_count: song_count.clone(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        counts.sort_by_key(|song_play_count| Reverse(song_play_count.song_count.1));
        counts.into_iter().take(count).collect()
    }
}
