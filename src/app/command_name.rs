use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::EnumIter;

use crate::persistence::Format;

use super::{chart::BarBreakdown, CommandParameterSpec, CommandParameters};

#[derive(Deserialize, Serialize, EnumIter, PartialEq, Debug)]
pub enum CommandName {
    RandomArtists,
    TopArtists,
    ArtistSongs,
    ArtistsOnDay,
    Summarize,
    ProcessListens,
    TopSongs,
    MostSkipped,
    Export,
    Chart,
}

impl ToString for CommandName {
    fn to_string(&self) -> String {
        match *self {
            Self::RandomArtists => "random artists".to_string(),
            Self::ArtistSongs => "artist songs".to_string(),
            Self::ArtistsOnDay => "artists on day".to_string(),
            Self::Summarize => "summarize".to_string(),
            Self::ProcessListens => "process".to_string(),
            Self::TopArtists => "top artists".to_string(),
            Self::TopSongs => "top songs".to_string(),
            Self::MostSkipped => "most skipped".to_string(),
            Self::Export => "export".to_string(),
            Self::Chart => "chart".to_string(),
        }
    }
}

impl FromStr for CommandName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "random artists" => Ok(Self::RandomArtists),
            "artist songs" => Ok(Self::ArtistSongs),
            "artists on day" => Ok(Self::ArtistsOnDay),
            "summarize" => Ok(Self::Summarize),
            "process" => Ok(Self::ProcessListens),
            "top artists" => Ok(Self::TopArtists),
            "top songs" => Ok(Self::TopSongs),
            "most skipped" => Ok(Self::MostSkipped),
            "export" => Ok(Self::Export),
            "chart" => Ok(Self::Chart),
            _ => Err("Unknown text".to_string()),
        }
    }
}

const DEFAULT_ARTIST_COUNT: usize = 5;
const DEFAULT_SONG_COUNT: usize = 20;
const DEFAULT_MIN_LISTENS: u64 = 5;
const DEFAULT_INPUT_FOLDER: &str = "./data/play_history";
const DEFAULT_OUTPUT_FOLDER: &str = "./output";
const DEFAULT_FILE_FORMAT: Format = Format::Yaml;

impl CommandName {
    pub fn description(&self) -> String {
        match *self {
            Self::RandomArtists => {
                "Select a number of random artists from some parameters".to_string()
            }
            Self::ArtistSongs => "List out the songs you've listened to from an artist".to_string(),
            Self::ArtistsOnDay => {
                "List all the songs you listened to on a specific day".to_string()
            }
            Self::Summarize => {
                "Print out a summary of your listens, either for a year or all time".to_string()
            }
            Self::ProcessListens => {
                "Process the listens in the data folder to fill the krustens database".to_string()
            }
            Self::TopArtists => "Return the most listened to artists".to_string(),
            Self::TopSongs => "Return the most listened to songs".to_string(),
            Self::MostSkipped => "Return the most skipped songs of all time".to_string(),
            Self::Export => "Export the current output to a file".to_string(),
            Self::Chart => "Create a chart of listens in a year by month".to_string(),
        }
    }

    pub fn default_parameters(&self) -> CommandParameters {
        match self {
            Self::RandomArtists => CommandParameters::RandomArtists {
                year: None,
                month: None,
                count: DEFAULT_ARTIST_COUNT,
                min_listens: DEFAULT_MIN_LISTENS,
            },
            Self::ArtistSongs => CommandParameters::ArtistSongs { name: None },
            Self::ArtistsOnDay => CommandParameters::ArtistsOnDay { date: None },
            Self::Summarize => CommandParameters::PrintStatistics { year: None },
            Self::ProcessListens => CommandParameters::GetFileNames {
                input_folder: DEFAULT_INPUT_FOLDER.to_string(),
            },
            Self::TopArtists => CommandParameters::TopArtists {
                count: DEFAULT_ARTIST_COUNT,
                year: None,
                month: None,
            },
            Self::TopSongs => CommandParameters::TopSongs {
                count: DEFAULT_SONG_COUNT,
                year: None,
            },
            Self::MostSkipped => CommandParameters::MostSkipped {
                count: DEFAULT_SONG_COUNT,
            },
            Self::Export => CommandParameters::Export {
                output_folder: DEFAULT_OUTPUT_FOLDER.to_string(),
                format: DEFAULT_FILE_FORMAT,
            },
            Self::Chart => {
                let default_year = chrono::Utc::now().year();
                CommandParameters::Chart {
                    year: default_year,
                    breakdown: BarBreakdown::default(),
                }
            }
        }
    }

    pub fn parameters(&self) -> Vec<CommandParameterSpec> {
        match *self {
            CommandName::RandomArtists => vec![
                CommandParameterSpec::Count {
                    description: format!(
                        "Number of artists to return (default: {DEFAULT_ARTIST_COUNT})"
                    ),
                },
                CommandParameterSpec::MinListens {
                    description: format!(
                        "Minimum number of listens to filter artists by (default: {DEFAULT_MIN_LISTENS})"
                    ),
                },
                CommandParameterSpec::Year {
                    description: "Year to search in (optional, e.g 2022)".to_string(),
                },
                CommandParameterSpec::Month {
                    description: "Month to search in (optional, 1-12)".to_string(),
                },
            ],
            CommandName::ArtistSongs => vec![CommandParameterSpec::ArtistName {
                description: "The name of the artist to get songs of".to_string(),
            }],
            CommandName::ArtistsOnDay => vec![CommandParameterSpec::Date {
                description: "Date to search on (required, format YYYY-MM-DD)".to_string(),
            }],
            CommandName::Summarize => vec![CommandParameterSpec::Year {
                description: "Year to get statistics of (optional, e.g 2022)".to_string(),
            }],
            CommandName::ProcessListens => vec![CommandParameterSpec::InputFolder {
                description: format!(
                    "What folder to parse the files containing listens from (default: {DEFAULT_INPUT_FOLDER})"
                ),
            }],
            CommandName::TopArtists => vec![
                CommandParameterSpec::Count {
                    description: format!(
                        "Number of artists to return (default: {DEFAULT_ARTIST_COUNT})"
                    ),
                },
                CommandParameterSpec::Year {
                    description: "Year to search in (optional, e.g 2022)".to_string(),
                },
                CommandParameterSpec::Month {
                    description: "Month to search in(optional, 1-12)".to_string(),
                }
            ],
            CommandName::TopSongs => vec![
                CommandParameterSpec::Count {
                    description: format!(
                        "Number of songs to return (default: {DEFAULT_SONG_COUNT})"
                    ),
                },
                CommandParameterSpec::Year {
                    description: "Year to search in (optional, e.g 2022)".to_string(),
                },
            ],
            CommandName::MostSkipped => vec![CommandParameterSpec::Count {
                description: format!(
                    "Number of songs to return (default: {DEFAULT_SONG_COUNT})"
                ),
            }],
            CommandName::Export => vec![
                CommandParameterSpec::OutputFolder {
                    description: format!("Where to put the file (default: {DEFAULT_OUTPUT_FOLDER})")
                },
                CommandParameterSpec::FileFormat {
                    description: format!(
                        "What file format to use ({} or {}, default: {})",
                        Format::Json.extension_display(),
                        Format::Yaml.extension_display(),
                        Format::Yaml.extension_display()
                    )
                }
            ],
            CommandName::Chart => vec![
                CommandParameterSpec::Year {
                    description: "What year for the chart (defaults to current year)".to_string()
                },
                CommandParameterSpec::BarBreakdown {
                    description: "How do you want to break down the data (weekday or month, defaults to month)".to_string()
                }
            ]
        }
    }
}

#[cfg(test)]
mod command_name_tests {
    use std::str::FromStr;

    use super::CommandName;

    #[test]
    fn deserialize() {
        let text = "random artists";

        let command: CommandName = CommandName::from_str(text).unwrap();
        assert_eq!(CommandName::RandomArtists, command);
    }
}
