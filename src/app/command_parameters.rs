use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::persistence::Format;

use super::chart::BarBreakdown;

#[derive(Deserialize, Serialize)]
pub enum CommandParameterSpec {
    Year { description: String },
    Month { description: String },
    MinListens { description: String },
    Count { description: String },
    Date { description: String },
    ArtistName { description: String },
    InputFolder { description: String },
    OutputFolder { description: String },
    FileFormat { description: String },
    BarBreakdown { description: String },
}

impl CommandParameterSpec {
    pub fn description(&self) -> String {
        match self {
            CommandParameterSpec::Year { description }
            | CommandParameterSpec::Month { description }
            | CommandParameterSpec::MinListens { description }
            | CommandParameterSpec::Date { description }
            | CommandParameterSpec::ArtistName { description }
            | CommandParameterSpec::Count { description }
            | CommandParameterSpec::InputFolder { description }
            | CommandParameterSpec::OutputFolder { description }
            | CommandParameterSpec::FileFormat { description }
            | CommandParameterSpec::BarBreakdown { description } => description.clone(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum CommandParameters {
    RandomArtists {
        year: Option<i32>,
        month: Option<u32>,
        count: usize,
        min_listens: u64,
    },
    TopArtists {
        count: usize,
        year: Option<i32>,
        month: Option<u32>,
    },
    TopAlbums {
        count: usize,
        year: Option<i32>,
    },
    TopSongs {
        count: usize,
        year: Option<i32>,
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
    MostSkipped {
        count: usize,
    },
    Export {
        output_folder: String,
        format: Format,
    },
    Chart {
        year: i32,
        breakdown: BarBreakdown,
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

    pub fn with_output_folder_parameter(&self, output_folder: &str) -> Self {
        match self {
            Self::Export {
                output_folder: _,
                format,
            } => Self::Export {
                output_folder: output_folder.to_string(),
                format: format.to_owned(),
            },
            _ => self.to_owned(),
        }
    }

    pub fn with_format_parameter(&self, format: Format) -> Self {
        match self {
            Self::Export {
                output_folder,
                format: _,
            } => Self::Export {
                output_folder: output_folder.to_owned(),
                format,
            },
            _ => self.to_owned(),
        }
    }

    pub fn with_year_parameter(&self, year: i32) -> Self {
        match self {
            Self::RandomArtists {
                year: _,
                month,
                count,
                min_listens,
            } => Self::RandomArtists {
                year: Some(year),
                month: month.to_owned(),
                count: count.to_owned(),
                min_listens: min_listens.to_owned(),
            },
            Self::TopArtists {
                count,
                year: _,
                month,
            } => Self::TopArtists {
                count: count.to_owned(),
                year: Some(year),
                month: month.to_owned(),
            },
            Self::TopAlbums { count, year: _ } => Self::TopAlbums {
                count: count.to_owned(),
                year: Some(year),
            },
            Self::TopSongs { count, year: _ } => Self::TopSongs {
                count: count.to_owned(),
                year: Some(year),
            },
            Self::PrintStatistics { year: _ } => Self::PrintStatistics { year: Some(year) },
            Self::Chart { year: _, breakdown } => Self::Chart {
                year,
                breakdown: breakdown.to_owned(),
            },
            _ => self.to_owned(),
        }
    }

    pub fn with_bar_breakdown_parameter(&self, breakdown: BarBreakdown) -> Self {
        match self {
            Self::Chart { year, breakdown: _ } => Self::Chart {
                year: year.to_owned(),
                breakdown,
            },
            _ => self.to_owned(),
        }
    }

    pub fn with_month_parameter(&self, month: u32) -> Self {
        match self {
            Self::RandomArtists {
                year,
                month: _,
                count: artist_count,
                min_listens,
            } => Self::RandomArtists {
                year: year.to_owned(),
                month: Some(month),
                count: artist_count.to_owned(),
                min_listens: min_listens.to_owned(),
            },
            Self::TopArtists {
                count,
                year,
                month: _,
            } => Self::TopArtists {
                year: year.to_owned(),
                count: count.to_owned(),
                month: Some(month),
            },
            _ => self.to_owned(),
        }
    }

    pub fn with_count_parameter(&self, count: usize) -> Self {
        match self {
            Self::RandomArtists {
                year,
                month,
                count: _,
                min_listens,
            } => Self::RandomArtists {
                year: year.to_owned(),
                month: month.to_owned(),
                count,
                min_listens: min_listens.to_owned(),
            },
            Self::TopArtists {
                count: _,
                year,
                month,
            } => Self::TopArtists {
                count,
                year: year.to_owned(),
                month: month.to_owned(),
            },
            Self::TopAlbums { count: _, year } => Self::TopAlbums {
                count,
                year: year.to_owned(),
            },
            Self::TopSongs { count: _, year } => Self::TopSongs {
                count,
                year: year.to_owned(),
            },
            Self::MostSkipped { count: _ } => Self::MostSkipped {
                count: count.to_owned(),
            },
            _ => self.to_owned(),
        }
    }

    pub fn with_min_listens_parameter(&self, min_listens: u64) -> Self {
        match self {
            Self::RandomArtists {
                year,
                month,
                count: artist_count,
                min_listens: _,
            } => Self::RandomArtists {
                year: year.to_owned(),
                month: month.to_owned(),
                count: artist_count.to_owned(),
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
                count: _,
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
                count: _,
                min_listens: _,
            } => self.to_owned(),
            Self::ArtistSongs { name: _ } => Self::ArtistSongs {
                name: Some(name.to_string()),
            },
            _ => self.to_owned(),
        }
    }
}
