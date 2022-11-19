use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Weekday};
use serde::Serialize;

use crate::track_plays::ArtistName;

use super::{artists_counts::ArtistsCounts, song_counter::ArtistSongCounter};

pub struct YearCounts {
    pub year: i32,
    pub months: HashMap<u32, MonthCounts>,
    pub artists_counts: ArtistsCounts,
}

impl From<&NaiveDate> for YearCounts {
    fn from(date: &NaiveDate) -> Self {
        Self {
            year: date.year(),
            months: HashMap::new(),
            artists_counts: ArtistsCounts::default(),
        }
    }
}

impl YearCounts {
    pub fn artists_on_day(&self, date: &NaiveDate) -> Vec<ArtistSongCounter> {
        self.months
            .get(&date.month())
            .map(|month_counts| month_counts.artists_on_day(date))
            .unwrap_or_default()
    }

    pub fn add_song_play(
        &mut self,
        date: &NaiveDate,
        artist_name: &ArtistName,
        song_name: &str,
        time_played: u64,
    ) {
        let month_count = self
            .months
            .entry(date.month())
            .or_insert_with(|| MonthCounts::from(date));

        month_count.add_song_play(date, artist_name, song_name, time_played);
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

impl From<&NaiveDate> for MonthCounts {
    fn from(date: &NaiveDate) -> Self {
        Self {
            month: date.month(),
            days: HashMap::new(),
            artists_counts: ArtistsCounts::default(),
        }
    }
}

impl MonthCounts {
    pub fn add_song_play(
        &mut self,
        date: &NaiveDate,
        artist_name: &ArtistName,
        song_name: &str,
        time_played: u64,
    ) {
        let day_counts = self
            .days
            .entry(date.day())
            .or_insert_with(|| DayCounts::from(date));

        day_counts.add_song_play(artist_name, song_name, time_played);
        self.artists_counts
            .add_song_play(artist_name, song_name, time_played);
    }

    pub fn over_min_plays(&self, min: u64) -> Vec<ArtistSongCounter> {
        self.artists_counts.over_min_plays(min)
    }

    pub fn artists_on_day(&self, date: &NaiveDate) -> Vec<ArtistSongCounter> {
        self.days
            .get(&date.day())
            .map(|day_counts| day_counts.artists())
            .unwrap_or_default()
    }
}

#[derive(Serialize)]
pub struct DayCounts {
    pub day_of_month: u32,
    pub weekday: Weekday,
    pub artists_counts: ArtistsCounts,
}

impl From<&NaiveDate> for DayCounts {
    fn from(calendar_day: &NaiveDate) -> Self {
        Self {
            day_of_month: calendar_day.day(),
            weekday: calendar_day.weekday(),
            artists_counts: ArtistsCounts::default(),
        }
    }
}

impl DayCounts {
    pub fn add_song_play(&mut self, artist_name: &ArtistName, song_name: &str, time_played: u64) {
        self.artists_counts
            .add_song_play(artist_name, song_name, time_played);
    }

    pub fn artists(&self) -> Vec<ArtistSongCounter> {
        self.artists_counts.all()
    }
}
