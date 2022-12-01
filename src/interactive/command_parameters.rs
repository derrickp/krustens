use std::path::PathBuf;

use chrono::NaiveDate;

pub enum CommandParameterSpec {
    Year { optional: bool, description: String },
    Month { optional: bool, description: String },
    MinListens { description: String },
    ArtistCount { description: String },
    Date { optional: bool, description: String },
    ArtistName { optional: bool, description: String },
    InputFolder { description: String },
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
            | CommandParameterSpec::MinListens { description }
            | CommandParameterSpec::Date {
                optional: _,
                description,
            }
            | CommandParameterSpec::ArtistName {
                optional: _,
                description,
            }
            | CommandParameterSpec::ArtistCount { description }
            | CommandParameterSpec::InputFolder { description } => description.clone(),
        }
    }
}

#[derive(Clone)]
pub enum CommandParameters {
    RandomArtists {
        year: Option<i32>,
        month: Option<u32>,
        artist_count: usize,
        min_listens: u64,
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
    GetFileNames {
        input_folder: String,
    },
    ProcessListens {
        files: Vec<PathBuf>,
    },
}

impl CommandParameters {
    pub fn with_input_folder_parameter(&self, input_folder: &str) -> Self {
        match self {
            Self::GetFileNames { input_folder: _ } => Self::GetFileNames {
                input_folder: input_folder.to_string(),
            },
            _ => self.to_owned(),
        }
    }

    pub fn with_year_parameter(&self, year: i32) -> Self {
        match self {
            Self::RandomArtists {
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
            Self::PrintStatistics { year: _ } => Self::PrintStatistics { year: Some(year) },
            _ => self.to_owned(),
        }
    }

    pub fn with_month_parameter(&self, month: u32) -> Self {
        match self {
            Self::RandomArtists {
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
            _ => self.to_owned(),
        }
    }

    pub fn with_artist_count_parameter(&self, artist_count: usize) -> Self {
        match self {
            Self::RandomArtists {
                year,
                month,
                artist_count: _,
                min_listens,
            } => Self::RandomArtists {
                year: year.to_owned(),
                month: month.to_owned(),
                artist_count,
                min_listens: min_listens.to_owned(),
            },
            _ => self.to_owned(),
        }
    }

    pub fn with_min_listens_parameter(&self, min_listens: u64) -> Self {
        match self {
            Self::RandomArtists {
                year,
                month,
                artist_count,
                min_listens: _,
            } => Self::RandomArtists {
                year: year.to_owned(),
                month: month.to_owned(),
                artist_count: artist_count.to_owned(),
                min_listens,
            },
            _ => self.to_owned(),
        }
    }

    pub fn with_date_parameter(&self, date: NaiveDate) -> Self {
        match self {
            Self::RandomArtists {
                year: _,
                month: _,
                artist_count: _,
                min_listens: _,
            } => self.to_owned(),
            Self::ArtistsOnDay { date: _ } => Self::ArtistsOnDay { date: Some(date) },
            _ => self.to_owned(),
        }
    }

    pub fn with_name_parameter(&self, name: &str) -> Self {
        match self {
            Self::RandomArtists {
                year: _,
                month: _,
                artist_count: _,
                min_listens: _,
            } => self.to_owned(),
            Self::ArtistSongs { name: _ } => Self::ArtistSongs {
                name: Some(name.to_string()),
            },
            _ => self.to_owned(),
        }
    }
}
