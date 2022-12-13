use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};

use crate::{
    processing::events::{Event, EventData},
    track_plays::ArtistName,
    utils::parse_formatted_end_time,
};

use super::{
    calendar_counts::YearCounts, counter::ArtistSongCounter, ArtistAndSongCount, ArtistsCounts,
    MonthCounts,
};

#[derive(Default)]
pub struct EventProcessor {
    pub years: HashMap<i32, YearCounts>,
    pub artists_counts: ArtistsCounts,
}

impl EventProcessor {
    pub fn artists_on_day(&self, date: NaiveDate) -> Vec<ArtistSongCounter> {
        self.years
            .get(&date.year())
            .map(|year_count| year_count.artists_on_day(&date))
            .unwrap_or_default()
    }

    pub fn month_counts(&self, month: u32) -> Vec<MonthCounts> {
        let mut month_counts: Vec<MonthCounts> = Vec::new();

        for year_count in self.years.values() {
            if let Some(month_count) = year_count.month_count(month) {
                month_counts.push(month_count.clone());
            }
        }

        month_counts
    }

    pub fn year_counts(&self) -> Vec<&YearCounts> {
        self.years.values().collect()
    }

    pub fn year_count(&self, year: i32) -> Option<&YearCounts> {
        self.years.get(&year)
    }

    pub fn artist_song_counter(&self, artist_name: &ArtistName) -> Option<ArtistSongCounter> {
        self.artists_counts.find_artist(artist_name)
    }

    pub fn process_event(&mut self, event: &Event) {
        match &event.data {
            EventData::TrackPlayAdded(listen) => {
                let calendar_day_result =
                    parse_formatted_end_time(listen.end_time.as_str()).map(|e| e.date());

                if let Ok(date) = calendar_day_result {
                    let year_counts = self
                        .years
                        .entry(date.year())
                        .or_insert_with(|| YearCounts::from(&date));
                    year_counts.add_song_play(
                        &date,
                        &listen.artist_name,
                        &listen.track_name,
                        listen.ms_played,
                    );
                }

                self.artists_counts.add_song_play(
                    &listen.artist_name,
                    &listen.track_name,
                    listen.ms_played,
                );
            }
            EventData::TrackPlayIgnored(ignored) => {
                self.artists_counts
                    .add_song_skip(&ignored.artist_name, &ignored.track_name);
            }
        };
    }

    pub fn top_skipped(&self, count: usize) -> Vec<ArtistAndSongCount> {
        self.artists_counts.top_skipped_songs(count)
    }
}
