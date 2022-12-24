use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

use crate::track_plays::{AlbumName, ArtistName, TrackName};

use super::{artists_counts::ArtistsCounts, counter::ArtistSongCounter};

#[derive(Clone, Deserialize, Serialize)]
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

    pub fn month_counts(&self) -> Vec<&MonthCounts> {
        self.months.values().collect()
    }

    pub fn add_album_play(
        &mut self,
        date: &NaiveDate,
        artist_name: &ArtistName,
        album_name: &AlbumName,
    ) {
        let month_count = self
            .months
            .entry(date.month())
            .or_insert_with(|| MonthCounts::from(date));

        month_count.add_album_play(date, artist_name, album_name);
        self.artists_counts.add_album_play(artist_name, album_name);
    }

    pub fn add_song_play(
        &mut self,
        date: &NaiveDate,
        artist_name: &ArtistName,
        track_name: &TrackName,
        time_played: u64,
    ) {
        let month_count = self
            .months
            .entry(date.month())
            .or_insert_with(|| MonthCounts::from(date));

        month_count.add_song_play(date, artist_name, track_name, time_played);
        self.artists_counts
            .add_song_play(artist_name, track_name, time_played);
    }

    pub fn over_min_plays(&self, min: u64) -> Vec<ArtistSongCounter> {
        self.artists_counts.over_min_plays(min)
    }

    pub fn month_count(&self, month: u32) -> Option<&MonthCounts> {
        self.months.get(&month)
    }
}

#[derive(Clone, Deserialize, Serialize)]
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
    pub fn merge(month_counts: Vec<MonthCounts>) -> ArtistsCounts {
        let mut starting_counts = ArtistsCounts::default();

        for month_count in month_counts.iter() {
            starting_counts.add(&month_count.artists_counts);
        }

        starting_counts
    }

    pub fn day_counts(&self) -> Vec<&DayCounts> {
        self.days.values().collect()
    }

    pub fn add_album_play(
        &mut self,
        date: &NaiveDate,
        artist_name: &ArtistName,
        album_name: &AlbumName,
    ) {
        let day_counts = self
            .days
            .entry(date.day())
            .or_insert_with(|| DayCounts::from(date));

        day_counts.add_album_play(artist_name, album_name);
        self.artists_counts.add_album_play(artist_name, album_name);
    }

    pub fn add_song_play(
        &mut self,
        date: &NaiveDate,
        artist_name: &ArtistName,
        track_name: &TrackName,
        time_played: u64,
    ) {
        let day_counts = self
            .days
            .entry(date.day())
            .or_insert_with(|| DayCounts::from(date));

        day_counts.add_song_play(artist_name, track_name, time_played);
        self.artists_counts
            .add_song_play(artist_name, track_name, time_played);
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

#[derive(Clone, Deserialize, Serialize)]
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
    pub fn add_album_play(&mut self, artist_name: &ArtistName, album_name: &AlbumName) {
        self.artists_counts.add_album_play(artist_name, album_name);
    }

    pub fn add_song_play(
        &mut self,
        artist_name: &ArtistName,
        track_name: &TrackName,
        time_played: u64,
    ) {
        self.artists_counts
            .add_song_play(artist_name, track_name, time_played);
    }

    pub fn artists(&self) -> Vec<ArtistSongCounter> {
        self.artists_counts.all()
    }
}

pub fn order_in_week(weekday: &Weekday) -> u8 {
    match weekday {
        Weekday::Sun => 1,
        Weekday::Mon => 2,
        Weekday::Tue => 3,
        Weekday::Wed => 4,
        Weekday::Thu => 5,
        Weekday::Fri => 6,
        Weekday::Sat => 7,
    }
}
