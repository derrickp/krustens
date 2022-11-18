use std::collections::HashMap;

use chrono::Datelike;

use crate::{
    events::{Event, EventData},
    track_plays::ArtistName,
    utils::parse_formatted_end_time,
};

use super::{
    calendar_counts::{CalendarDay, YearCounts},
    ArtistsCounts,
};

#[derive(Default)]
pub struct EventProcessor {
    pub years: HashMap<i32, YearCounts>,
    pub artists_counts: ArtistsCounts,
}

impl EventProcessor {
    pub fn year_counts(&self) -> Vec<&YearCounts> {
        self.years.values().collect()
    }

    pub fn process_event(&mut self, event: &Event) {
        match &event.data {
            EventData::TrackPlayAdded(listen) => {
                let calendar_day_result =
                    parse_formatted_end_time(listen.end_time.as_str()).map(|e| CalendarDay {
                        year: e.year(),
                        month: e.month(),
                        day: e.day(),
                        weekday: e.weekday(),
                    });

                if let Ok(calendar_day) = calendar_day_result {
                    let year_counts = self
                        .years
                        .entry(calendar_day.year)
                        .or_insert_with(|| YearCounts::from(&calendar_day));
                    year_counts.add_song_play(
                        &calendar_day,
                        &ArtistName(listen.artist_name.clone()),
                        &listen.track_name,
                        listen.ms_played,
                    );
                }

                self.artists_counts.add_song_play(
                    &ArtistName(listen.artist_name.clone()),
                    &listen.track_name,
                    listen.ms_played,
                );
            }
            EventData::TrackPlayIgnored(_) => {}
        };
    }

    pub fn sort_by_song_count(&mut self) {
        self.artists_counts.sort_by_song_count();

        // TODO: Sort everything.
    }
}
