use std::collections::HashMap;

use chrono::Weekday;
use serde::Serialize;

use crate::track_plays::ArtistName;

use super::{artists_counts::ArtistsCounts, song_counter::ArtistSongCounter};

pub struct CalendarDay {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub weekday: Weekday,
}

pub struct YearCounts {
    pub year: i32,
    pub months: HashMap<u32, MonthCounts>,
    pub artists_counts: ArtistsCounts,
}

impl From<&CalendarDay> for YearCounts {
    fn from(calendar_day: &CalendarDay) -> Self {
        Self {
            year: calendar_day.year,
            months: HashMap::new(),
            artists_counts: ArtistsCounts::default(),
        }
    }
}

impl YearCounts {
    pub fn add_song_play(
        &mut self,
        calendar_day: &CalendarDay,
        artist_name: &ArtistName,
        song_name: &str,
        time_played: u64,
    ) {
        let month_count = self
            .months
            .entry(calendar_day.month)
            .or_insert_with(|| MonthCounts::from(calendar_day));

        month_count.add_song_play(calendar_day, artist_name, song_name, time_played);
        self.artists_counts
            .add_song_play(artist_name, song_name, time_played);
    }

    pub fn over_min_plays(&self, min: u64) -> Vec<ArtistSongCounter> {
        self.artists_counts.over_min_plays(min)
    }

    pub fn month_counts(&self) -> Vec<&MonthCounts> {
        self.months.values().collect()
    }

    pub fn month_count(&self, month: u32) -> Option<&MonthCounts> {
        self.months.get(&month)
    }
}

pub struct MonthCounts {
    pub month: u32,
    pub days: HashMap<u32, DayCounts>,
    pub artists_counts: ArtistsCounts,
}

impl From<&CalendarDay> for MonthCounts {
    fn from(calendar_day: &CalendarDay) -> Self {
        Self {
            month: calendar_day.month,
            days: HashMap::new(),
            artists_counts: ArtistsCounts::default(),
        }
    }
}

impl MonthCounts {
    pub fn add_song_play(
        &mut self,
        calendar_day: &CalendarDay,
        artist_name: &ArtistName,
        song_name: &str,
        time_played: u64,
    ) {
        let day_counts = self
            .days
            .entry(calendar_day.day)
            .or_insert_with(|| DayCounts::from(calendar_day));

        day_counts.add_song_play(artist_name, song_name, time_played);
        self.artists_counts
            .add_song_play(artist_name, song_name, time_played);
    }

    pub fn over_min_plays(&self, min: u64) -> Vec<ArtistSongCounter> {
        self.artists_counts.over_min_plays(min)
    }
}

#[derive(Serialize)]
pub struct DayCounts {
    pub day_of_month: u32,
    pub weekday: Weekday,
    pub artists_counts: ArtistsCounts,
}

impl From<&CalendarDay> for DayCounts {
    fn from(calendar_day: &CalendarDay) -> Self {
        Self {
            day_of_month: calendar_day.day,
            weekday: calendar_day.weekday,
            artists_counts: ArtistsCounts::default(),
        }
    }
}

impl DayCounts {
    pub fn add_song_play(&mut self, artist_name: &ArtistName, song_name: &str, time_played: u64) {
        self.artists_counts
            .add_song_play(artist_name, song_name, time_played);
    }
}
