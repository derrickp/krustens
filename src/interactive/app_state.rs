use std::{collections::HashSet, str::FromStr};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    errors::InteractiveError,
    projections::statistics::{EventProcessor, General},
    track_plays::ArtistName,
};

use super::AppMessageSet;

#[derive(Default)]
pub struct AppState {
    pub input: String,
    pub mode: AppMode,
    pub command_name: Option<AppCommandName>,
    pub error_message: Option<String>,
    pub message_sets: Vec<AppMessageSet>,
    pub command_steps: Vec<CommandParameterSpec>,
    pub command_parameters: Option<AppCommandParameters>,
}

impl AppState {
    pub fn display_sets(&self) -> Vec<AppMessageSet> {
        match self.mode {
            AppMode::CommandParameters => Vec::new(),
            AppMode::EnterCommand => {
                vec![AppMessageSet {
                    title: "Commands".to_string(),
                    messages: AppCommandName::iter()
                        .map(|command| {
                            format!("{} - {}", command.to_string(), command.description())
                        })
                        .collect(),
                }]
            }
            AppMode::Normal => self.message_sets.to_vec(),
        }
    }

    pub fn run_command(&mut self, processor: &EventProcessor) {
        if let Some(name) = &self.command_name {
            let parameters = self
                .command_parameters
                .to_owned()
                .unwrap_or_else(|| name.default_parameters());
            match parameters {
                AppCommandParameters::RandomArtists {
                    year,
                    month,
                    artist_count,
                    min_listens,
                } => self.run_random_artists(
                    year,
                    month,
                    artist_count.unwrap_or_default(),
                    min_listens.unwrap_or_default(),
                    processor,
                ),
                AppCommandParameters::ArtistSongs { name } => {
                    self.run_artist_songs(&name.unwrap_or_else(|| "".to_string()), processor);
                }
                AppCommandParameters::ArtistsOnDay { date } => {
                    self.run_artists_on_day(date.unwrap_or_default(), processor);
                }
                AppCommandParameters::PrintStatistics { year } => {
                    self.run_print_statistics(year, processor)
                }
            }
        }
    }

    fn run_random_artists(
        &mut self,
        year: Option<i32>,
        month: Option<u32>,
        artist_count: usize,
        min_listens: u64,
        processor: &EventProcessor,
    ) {
        let mut artist_names: HashSet<String> = HashSet::new();

        let year_counts = if let Some(y) = year {
            processor
                .year_count(y)
                .map(|year_count| vec![year_count])
                .unwrap_or_default()
        } else {
            processor.year_counts()
        };

        for year_count in year_counts.iter() {
            let artist_counts = if let Some(m) = month {
                year_count
                    .month_count(m)
                    .map(|month_counts| month_counts.over_min_plays(min_listens))
            } else {
                Some(year_count.over_min_plays(min_listens))
            }
            .unwrap_or_default();

            for artist_count in artist_counts.iter() {
                if !artist_names.contains(&artist_count.artist_name.0) {
                    artist_names.insert(artist_count.artist_name.0.clone());
                }
            }
        }

        let year_text = year
            .map(|y| format!("{}", y))
            .unwrap_or_else(|| "None".to_string());
        let month_text = month
            .map(|m| format!("{}", m))
            .unwrap_or_else(|| "None".to_string());
        let title = format!(
            "Artist search (year: {}, month: {}, min listens: {}, count: {})",
            year_text, month_text, min_listens, artist_count
        );

        let messages = if artist_names.is_empty() {
            vec!["No artists found".to_string()]
        } else {
            artist_names.iter().take(artist_count).cloned().collect()
        };

        self.message_sets
            .insert(0, AppMessageSet { title, messages });
    }

    fn run_artist_songs(&mut self, name: &str, processor: &EventProcessor) {
        let mut songs: Vec<String> = processor
            .artist_song_counter(&ArtistName(name.to_string()))
            .map(|artist_counter| {
                artist_counter
                    .play_details
                    .all_song_plays()
                    .iter()
                    .map(|song_play| song_play.0.clone())
                    .collect()
            })
            .unwrap_or_default();

        songs.sort();
        songs.dedup();

        self.message_sets.insert(
            0,
            AppMessageSet {
                title: format!("Songs for {}", name),
                messages: songs,
            },
        );
    }

    fn run_artists_on_day(&mut self, date: NaiveDate, processor: &EventProcessor) {
        let names = processor
            .artists_on_day(date)
            .into_iter()
            .map(|artist_counter| artist_counter.total_plays_display())
            .collect::<Vec<String>>();

        let title = format!("Artists listened to on {}", date.format("%Y-%m-%d"));

        let messages = if names.is_empty() {
            vec!["No artists found".to_string()]
        } else {
            names
        };

        self.message_sets
            .insert(0, AppMessageSet { title, messages });
    }

    fn run_print_statistics(&mut self, year: Option<i32>, processor: &EventProcessor) {
        if let Some(y) = year {
            let title = format!("Statistics for {}", y);
            if let Some(year_counts) = processor.year_count(y) {
                let general = year_counts.artists_counts.general_stats(5);
                self.add_general_stats_to_messages(&general, &title);
            } else {
                self.message_sets.insert(
                    0,
                    AppMessageSet {
                        title: format!("Statistics for {}", y),
                        messages: vec!["No statistics gathered".to_string()],
                    },
                );
            }
        } else {
            let general = processor.artists_counts.general_stats(5);
            self.add_general_stats_to_messages(&general, "Statistics");
        }
    }

    fn add_general_stats_to_messages(&mut self, general: &General, title: &str) {
        self.message_sets.insert(
            0,
            AppMessageSet {
                title: title.to_string(),
                messages: vec![format!(
                    "You've listened to {} artists",
                    general.count_artists_listened_to
                )],
            },
        );

        self.message_sets.insert(
            1,
            AppMessageSet {
                title: "Most listened to artists".to_string(),
                messages: general.artist_total_plays.to_vec(),
            },
        );
    }

    pub fn insert_command_parameter(
        &mut self,
        text: &str,
        spec: &CommandParameterSpec,
    ) -> Result<(), InteractiveError> {
        match spec {
            CommandParameterSpec::Year {
                optional,
                description: _,
            } => {
                if let Some(year) = text.parse::<i32>().ok() {
                    self.add_year_parameter(year);
                    Ok(())
                } else {
                    if !optional {
                        Err(InteractiveError::RequiredParameterNotSet {
                            name: "year".to_string(),
                        })
                    } else {
                        Ok(())
                    }
                }
            }
            CommandParameterSpec::Month {
                optional,
                description: _,
            } => {
                if let Some(month) = text.parse::<u32>().ok().filter(|m| (&1..=&12).contains(&m)) {
                    self.add_month_parameter(month);
                    Ok(())
                } else {
                    if !optional {
                        Err(InteractiveError::RequiredParameterNotSet {
                            name: "month".to_string(),
                        })
                    } else {
                        Ok(())
                    }
                }
            }
            CommandParameterSpec::MinListens {
                default,
                description: _,
            } => {
                let min_listens = text.parse::<u64>().unwrap_or(*default);
                self.add_min_listens_parameter(min_listens);
                Ok(())
            }
            CommandParameterSpec::ArtistCount {
                default,
                description: _,
            } => {
                let artist_count = text.parse::<usize>().unwrap_or(*default);
                self.add_artist_count_parameter(artist_count);
                Ok(())
            }
            CommandParameterSpec::Date {
                optional: _,
                description: _,
            } => match NaiveDate::parse_from_str(text, "%Y-%m-%d") {
                Ok(date) => {
                    self.add_date_parameter(date);
                    Ok(())
                }
                Err(e) => Err(InteractiveError::ParsingIssue {
                    message: e.to_string(),
                }),
            },
            CommandParameterSpec::ArtistName {
                optional,
                description: _,
            } => {
                if text.is_empty() && !optional {
                    Err(InteractiveError::RequiredParameterNotSet {
                        name: "name".to_string(),
                    })
                } else {
                    self.add_name_parameter(text);
                    Ok(())
                }
            }
        }
    }

    fn add_year_parameter(&mut self, year: i32) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_year_parameter(year));
        }
    }

    fn add_month_parameter(&mut self, month: u32) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_month_parameter(month));
        }
    }

    fn add_min_listens_parameter(&mut self, min_listens: u64) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_min_listens_parameter(min_listens));
        }
    }

    fn add_artist_count_parameter(&mut self, count: usize) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_artist_count_parameter(count));
        }
    }

    fn add_date_parameter(&mut self, date: NaiveDate) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_date_parameter(date));
        }
    }

    fn add_name_parameter(&mut self, name: &str) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_name_parameter(name));
        }
    }

    fn set_default_command_parameters(&mut self) {
        if self.command_parameters.is_some() {
            return;
        }

        match &self.command_name {
            Some(name) => {
                self.command_parameters = Some(name.default_parameters());
            }
            None => {}
        }
    }
}

#[derive(Deserialize, Serialize, EnumIter, PartialEq, Debug)]
pub enum AppCommandName {
    RandomArtists,
    ArtistSongs,
    ArtistsOnDay,
    PrintStatistics,
}

#[derive(Clone)]
pub enum AppCommandParameters {
    RandomArtists {
        year: Option<i32>,
        month: Option<u32>,
        artist_count: Option<usize>,
        min_listens: Option<u64>,
    },
    ArtistSongs {
        name: Option<String>,
    },
    ArtistsOnDay {
        date: Option<NaiveDate>,
    },
    PrintStatistics {
        year: Option<i32>,
    },
}

impl AppCommandParameters {
    pub fn with_year_parameter(&self, year: i32) -> Self {
        match self {
            AppCommandParameters::RandomArtists {
                year: _,
                month,
                artist_count,
                min_listens,
            } => Self::RandomArtists {
                year: Some(year),
                month: month.to_owned(),
                artist_count: artist_count.to_owned(),
                min_listens: min_listens.to_owned(),
            },
            AppCommandParameters::ArtistSongs { name: _ } => self.to_owned(),
            AppCommandParameters::ArtistsOnDay { date: _ } => self.to_owned(),
            AppCommandParameters::PrintStatistics { year: _ } => {
                Self::PrintStatistics { year: Some(year) }
            }
        }
    }

    pub fn with_month_parameter(&self, month: u32) -> Self {
        match self {
            AppCommandParameters::RandomArtists {
                year,
                month: _,
                artist_count,
                min_listens,
            } => Self::RandomArtists {
                year: year.to_owned(),
                month: Some(month),
                artist_count: artist_count.to_owned(),
                min_listens: min_listens.to_owned(),
            },
            AppCommandParameters::ArtistSongs { name: _ } => self.to_owned(),
            AppCommandParameters::ArtistsOnDay { date: _ } => self.to_owned(),
            AppCommandParameters::PrintStatistics { year: _ } => self.to_owned(),
        }
    }

    pub fn with_artist_count_parameter(&self, artist_count: usize) -> Self {
        match self {
            AppCommandParameters::RandomArtists {
                year,
                month,
                artist_count: _,
                min_listens,
            } => Self::RandomArtists {
                year: year.to_owned(),
                month: month.to_owned(),
                artist_count: Some(artist_count),
                min_listens: min_listens.to_owned(),
            },
            AppCommandParameters::ArtistSongs { name: _ } => self.to_owned(),
            AppCommandParameters::ArtistsOnDay { date: _ } => self.to_owned(),
            AppCommandParameters::PrintStatistics { year: _ } => self.to_owned(),
        }
    }

    pub fn with_min_listens_parameter(&self, min_listens: u64) -> Self {
        match self {
            AppCommandParameters::RandomArtists {
                year,
                month,
                artist_count,
                min_listens: _,
            } => Self::RandomArtists {
                year: year.to_owned(),
                month: month.to_owned(),
                artist_count: artist_count.to_owned(),
                min_listens: Some(min_listens),
            },
            AppCommandParameters::ArtistSongs { name: _ } => self.to_owned(),
            AppCommandParameters::ArtistsOnDay { date: _ } => self.to_owned(),
            AppCommandParameters::PrintStatistics { year: _ } => self.to_owned(),
        }
    }

    pub fn with_date_parameter(&self, date: NaiveDate) -> Self {
        match self {
            AppCommandParameters::RandomArtists {
                year: _,
                month: _,
                artist_count: _,
                min_listens: _,
            } => self.to_owned(),
            AppCommandParameters::ArtistSongs { name: _ } => self.to_owned(),
            AppCommandParameters::ArtistsOnDay { date: _ } => {
                Self::ArtistsOnDay { date: Some(date) }
            }
            AppCommandParameters::PrintStatistics { year: _ } => self.to_owned(),
        }
    }

    pub fn with_name_parameter(&self, name: &str) -> Self {
        match self {
            AppCommandParameters::RandomArtists {
                year: _,
                month: _,
                artist_count: _,
                min_listens: _,
            } => self.to_owned(),
            AppCommandParameters::ArtistSongs { name: _ } => Self::ArtistSongs {
                name: Some(name.to_string()),
            },
            AppCommandParameters::ArtistsOnDay { date: _ } => self.to_owned(),
            AppCommandParameters::PrintStatistics { year: _ } => self.to_owned(),
        }
    }
}

impl ToString for AppCommandName {
    fn to_string(&self) -> String {
        match *self {
            AppCommandName::RandomArtists => "random artists".to_string(),
            AppCommandName::ArtistSongs => "artist songs".to_string(),
            AppCommandName::ArtistsOnDay => "artists on day".to_string(),
            AppCommandName::PrintStatistics => "print statistics".to_string(),
        }
    }
}

impl FromStr for AppCommandName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "random artists" => Ok(AppCommandName::RandomArtists),
            "artist songs" => Ok(AppCommandName::ArtistSongs),
            "artists on day" => Ok(AppCommandName::ArtistsOnDay),
            "print statistics" => Ok(AppCommandName::PrintStatistics),
            _ => Err("Unknown text".to_string()),
        }
    }
}

impl AppCommandName {
    pub fn description(&self) -> String {
        match *self {
            AppCommandName::RandomArtists => {
                "Select a number of random artists from some parameters".to_string()
            }
            AppCommandName::ArtistSongs => {
                "List out the songs you've listened to from an artist".to_string()
            }
            AppCommandName::ArtistsOnDay => {
                "List all the songs you listened to on a specific day".to_string()
            }
            AppCommandName::PrintStatistics => {
                "Print out some of the statistics, either for a year or all time".to_string()
            }
        }
    }

    pub fn default_parameters(&self) -> AppCommandParameters {
        match self {
            AppCommandName::RandomArtists => AppCommandParameters::RandomArtists {
                year: None,
                month: None,
                artist_count: None,
                min_listens: None,
            },
            AppCommandName::ArtistSongs => AppCommandParameters::ArtistSongs { name: None },
            AppCommandName::ArtistsOnDay => AppCommandParameters::ArtistsOnDay { date: None },
            AppCommandName::PrintStatistics => AppCommandParameters::PrintStatistics { year: None },
        }
    }

    pub fn parameters(&self) -> Vec<CommandParameterSpec> {
        match *self {
            AppCommandName::RandomArtists => vec![
                CommandParameterSpec::ArtistCount {
                    default: 5,
                    description: "Number of artists to return (default: 5)".to_string(),
                },
                CommandParameterSpec::MinListens {
                    default: 5,
                    description: "Minimum number of listens to filter artists by (default: 5)"
                        .to_string(),
                },
                CommandParameterSpec::Year {
                    optional: true,
                    description: "Year to search in (optional, e.g 2022)".to_string(),
                },
                CommandParameterSpec::Month {
                    optional: true,
                    description: "Month to search in (optional, 1-12)".to_string(),
                },
            ],
            AppCommandName::ArtistSongs => vec![CommandParameterSpec::ArtistName {
                optional: false,
                description: "The name of the artist to get songs of".to_string(),
            }],
            AppCommandName::ArtistsOnDay => vec![CommandParameterSpec::Date {
                optional: false,
                description: "Date to search on (required, format YYYY-MM-DD)".to_string(),
            }],
            AppCommandName::PrintStatistics => vec![CommandParameterSpec::Year {
                optional: true,
                description: "Year to get statistics of (optional, e.g 2022)".to_string(),
            }],
        }
    }
}

pub enum AppMode {
    CommandParameters,
    EnterCommand,
    Normal,
}

pub enum CommandParameterSpec {
    Year { optional: bool, description: String },
    Month { optional: bool, description: String },
    MinListens { default: u64, description: String },
    ArtistCount { default: usize, description: String },
    Date { optional: bool, description: String },
    ArtistName { optional: bool, description: String },
}

impl CommandParameterSpec {
    pub fn description(&self) -> String {
        match self {
            CommandParameterSpec::Year {
                optional: _,
                description,
            }
            | CommandParameterSpec::Month {
                optional: _,
                description,
            }
            | CommandParameterSpec::MinListens {
                default: _,
                description,
            }
            | CommandParameterSpec::Date {
                optional: _,
                description,
            }
            | CommandParameterSpec::ArtistName {
                optional: _,
                description,
            }
            | CommandParameterSpec::ArtistCount {
                default: _,
                description,
            } => description.clone(),
        }
    }
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(test)]
mod command_name_tests {
    use std::str::FromStr;

    use super::AppCommandName;

    #[test]
    fn deserialize() {
        let text = "random artists";

        let command: AppCommandName = AppCommandName::from_str(text).unwrap();
        assert_eq!(AppCommandName::RandomArtists, command);
    }
}
