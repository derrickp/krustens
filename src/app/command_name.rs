use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use strum_macros::EnumIter;

use crate::persistence::Format;

use super::{chart::BarBreakdown, CommandParameterSpec, CommandParameters};

#[derive(Clone, Deserialize, Serialize, EnumIter, PartialEq, Debug)]
pub enum CommandName {
    RandomArtists,
    TopArtists,
    ArtistSongs,
    ArtistsOnDay,
    Summarize,
    ProcessListens,
    TopAlbums,
    TopSongs,
    MostSkipped,
    Export,
    Chart,
    ClearOutput,
}

impl Display for CommandName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match *self {
            Self::RandomArtists => "random artists",
            Self::ArtistSongs => "artist songs",
            Self::ArtistsOnDay => "artists on day",
            Self::Summarize => "summarize",
            Self::ProcessListens => "process",
            Self::TopArtists => "top artists",
            Self::TopSongs => "top songs",
            Self::MostSkipped => "most skipped",
            Self::Export => "export",
            Self::Chart => "chart",
            Self::TopAlbums => "top albums",
            Self::ClearOutput => "clear output",
        };

        f.write_str(value)
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
            "top albums" => Ok(Self::TopAlbums),
            "clear output" => Ok(Self::ClearOutput),
            _ => Err("Unknown text".to_string()),
        }
    }
}

const DEFAULT_ARTIST_COUNT: usize = 5;
const DEFAULT_SONG_COUNT: usize = 20;
const DEFAULT_ALBUM_COUNT: usize = 10;
const DEFAULT_MIN_LISTENS: u64 = 5;
const DEFAULT_INPUT_FOLDER: &str = "./data/play_history";
const DEFAULT_OUTPUT_FOLDER: &str = "./output";
const DEFAULT_FILE_FORMAT: Format = Format::Yaml;

impl CommandName {
    pub fn description(&self) -> &str {
        match *self {
            Self::RandomArtists => "Select a number of random artists from some parameters",
            Self::ArtistSongs => "List out the songs you've listened to from an artist",
            Self::ArtistsOnDay => "List all the songs you listened to on a specific day",
            Self::Summarize => "Print out a summary of your listens, either for a year or all time",
            Self::ProcessListens => {
                "Process the listens in the data folder to fill the krustens database"
            }
            Self::TopArtists => "Return the most listened to artists",
            Self::TopSongs => "Return the most listened to songs",
            Self::MostSkipped => "Return the most skipped songs of all time",
            Self::Export => "Export the current output to a file",
            Self::Chart => "Create a chart of listens in a year by month",
            Self::TopAlbums => "Return the most listened to albums",
            Self::ClearOutput => "Clear all of the output",
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
            Self::TopAlbums => CommandParameters::TopAlbums {
                count: DEFAULT_ALBUM_COUNT,
                year: None,
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
            Self::Chart => CommandParameters::Chart {
                year: None,
                breakdown: BarBreakdown::default(),
                artist_name: None,
            },
            Self::ClearOutput => CommandParameters::ClearOutput,
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
            CommandName::TopAlbums => vec![
                CommandParameterSpec::Count {
                    description: format!(
                        "Number of albums to return (default: {DEFAULT_ALBUM_COUNT})"
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
                    description: "What year for the chart (e.g. 2022)".to_string()
                },
                CommandParameterSpec::ArtistName {
                    description: "If you'd like to filter by artist, enter the name.".to_string()
                },
                CommandParameterSpec::BarBreakdown {
                    description: "How do you want to break down the data (weekday or month, defaults to month)".to_string()
                }
            ],
            CommandName::ClearOutput => Vec::new(),
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
