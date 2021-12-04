pub mod day_stat;
pub mod general;
pub mod play_count;
pub mod skipped_track;
pub mod song_play_count;

use std::{cmp::Reverse, collections::HashMap};

use chrono::{Datelike, NaiveDateTime, ParseResult};
use serde::Serialize;

use crate::events::{event::Event, event_data::EventData};

use self::{
    general::GeneralStats, play_count::PlayCount, skipped_track::SkippedTrack,
    song_play_count::SongPlayCount,
};

#[derive(Debug, Serialize, Default)]
pub struct Stats {
    pub stats: HashMap<String, PlayCount>,
    pub skipped: HashMap<String, SkippedTrack>,
}

fn parse_end_time(end_time: &str) -> ParseResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(end_time, "%Y-%m-%d %H:%M")
}

impl Stats {
    pub fn generate_for_year(events: Vec<&Event>, year: i32) -> Self {
        let matching_events = events
            .iter()
            .filter(|event| {
                let end_time = match &event.data {
                    EventData::TrackPlayAdded(added) => parse_end_time(added.end_time.as_str()),
                    EventData::TrackPlayIgnored(ignored) => {
                        parse_end_time(ignored.end_time.as_str())
                    }
                };

                match end_time {
                    Ok(it) => it.year().eq(&year),
                    _ => false
                }
            })
            .cloned()
            .collect();

        Self::generate(matching_events)
    }

    pub fn generate_month_year(events: Vec<&Event>, year: i32, month: u32) -> Self {
        let matching_events: Vec<&Event> = events
            .iter()
            .filter(|event| {
                let end_time = match &event.data {
                    EventData::TrackPlayAdded(added) => parse_end_time(added.end_time.as_str()),
                    EventData::TrackPlayIgnored(ignored) => {
                        parse_end_time(ignored.end_time.as_str())
                    }
                };

                match end_time {
                    Ok(it) => it.year() == year && it.month() == month,
                    _ => false,
                }
            })
            .cloned()
            .collect();

        Self::generate(matching_events)
    }

    pub fn generate(events: Vec<&Event>) -> Self {
        let mut stats = events.iter().fold(Self::default(), |mut acc, event| {
            match &event.data {
                EventData::TrackPlayAdded(listen) => acc
                    .stats
                    .entry(listen.artist_name.clone())
                    .or_insert_with(|| PlayCount::build(listen.artist_name.clone()))
                    .increment_song(&listen.track_name, listen.ms_played),
                EventData::TrackPlayIgnored(ignored) => acc
                    .skipped
                    .entry(ignored.artist_name.clone())
                    .or_insert_with(|| SkippedTrack::build(ignored.artist_name.clone()))
                    .increment_song(&ignored.track_name),
            };

            acc
        });

        for play_count in stats.stats.values_mut() {
            play_count.sort_by_song_count();
        }

        stats
    }

    pub fn general_stats(&self, count: usize) -> GeneralStats {
        let artist_total_plays: Vec<String> = self
            .top(count)
            .iter()
            .map(PlayCount::total_plays_display)
            .collect();

        let most_played_songs: Vec<String> = self
            .top_songs(count)
            .iter()
            .map(ArtistAndSongPlays::display)
            .collect();

        let unique_artists_most_played_songs: Vec<String> = self
            .top_unique_artists(count)
            .iter()
            .map(PlayCount::max_song_display)
            .collect();

        GeneralStats {
            artist_total_plays,
            most_played_songs,
            artist_most_played_songs: unique_artists_most_played_songs,
            count_artists_listened_to: self.artist_count(),
        }
    }

    pub fn artist_count(&self) -> usize {
        self.stats.len()
    }

    pub fn top(&self, count: usize) -> Vec<PlayCount> {
        let mut counts: Vec<PlayCount> = self.stats.values().cloned().collect();
        counts.sort_by_key(|play| Reverse(play.total_plays()));
        counts.into_iter().take(count).collect()
    }

    pub fn top_songs(&self, count: usize) -> Vec<ArtistAndSongPlays> {
        let mut counts: Vec<ArtistAndSongPlays> = self
            .stats
            .values()
            .cloned()
            .flat_map(|play_count| {
                play_count
                    .all_song_plays()
                    .iter()
                    .map(|song_count| ArtistAndSongPlays {
                        artist_name: play_count.artist_name.clone(),
                        song_count: song_count.clone(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        counts.sort_by_key(|song_play_count| Reverse(song_play_count.song_count.1));
        counts.into_iter().take(count).collect()
    }

    pub fn top_unique_artists(&self, count: usize) -> Vec<PlayCount> {
        let mut counts: Vec<PlayCount> = self.stats.values().cloned().collect();

        counts.sort_by_key(|play| Reverse(play.max_song_play().1));
        counts.into_iter().take(count).collect()
    }
}

pub struct ArtistAndSongPlays {
    pub artist_name: String,
    pub song_count: SongPlayCount,
}

impl ArtistAndSongPlays {
    pub fn display(&self) -> String {
        format!(
            "{} - {} - {}",
            self.artist_name.clone(),
            self.song_count.0.clone(),
            self.song_count.1
        )
    }
}
