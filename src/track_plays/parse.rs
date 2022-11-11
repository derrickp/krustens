use std::{fs, path::PathBuf};

use crate::errors::ReadError;

use super::{apple_music::PlayActivity, Spotify, TrackPlay};

enum FileType {
    Json,
    Csv,
    Unsupported(String),
}

pub fn read_track_plays(path: &PathBuf) -> Result<Vec<TrackPlay>, ReadError> {
    if !path.is_file() {
        return Err(ReadError::NotAFile {
            file_name: path.display().to_string(),
        });
    }

    let file_type = path.extension().map(|extension| {
        if extension.eq_ignore_ascii_case("json") {
            FileType::Json
        } else if extension.eq_ignore_ascii_case("csv") {
            FileType::Csv
        } else {
            FileType::Unsupported(format!("{:?}", extension))
        }
    });

    match file_type {
        Some(FileType::Json) => parse_json(path),
        Some(FileType::Csv) => parse_csv(path),
        Some(FileType::Unsupported(extension)) => Err(ReadError::UnsupportedFileType {
            file_type: extension,
        }),
        None => Err(ReadError::UnsupportedFileType {
            file_type: format!("{:?}", &path),
        }),
    }
}

fn parse_json(path: &PathBuf) -> Result<Vec<TrackPlay>, ReadError> {
    let contents = fs::read_to_string(format!("{}", &path.display())).map_err(|err| {
        ReadError::CannotReadContents {
            file_name: format!("{:?}", &path),
            message: err.to_string(),
        }
    })?;

    serde_json::from_str(&contents)
        .map(|spotify_listens: Vec<Spotify>| {
            spotify_listens
                .into_iter()
                .map(TrackPlay::Spotify)
                .collect()
        })
        .map_err(|err| ReadError::FailedToDeserializeJson {
            message: err.to_string(),
            file_name: format!("{:?}", &path),
        })
}

fn parse_csv(path: &PathBuf) -> Result<Vec<TrackPlay>, ReadError> {
    let mut reader = csv::Reader::from_path(path).map_err(|err| ReadError::CannotReadContents {
        file_name: format!("{:?}", &path),
        message: err.to_string(),
    })?;

    let headers = reader
        .headers()
        .map_err(|err| ReadError::FailedToDeserializeCsv {
            message: err.to_string(),
            file_name: format!("{:?}", &path),
        })
        .cloned()?;

    let activities: Vec<TrackPlay> = reader
        .records()
        .into_iter()
        .filter_map(|record| record.ok())
        .filter_map(|record| {
            if headers
                .iter()
                .any(|header| header.eq_ignore_ascii_case(&PlayActivity::identifying_header()))
            {
                record
                    .deserialize::<PlayActivity>(Some(&headers))
                    .ok()
                    .filter(|activity| activity.is_end_event())
                    .map(|activity| vec![TrackPlay::AppleMusicPlayActivity(activity)])
            } else {
                None
            }
        })
        .flatten()
        .collect();

    if activities.is_empty() {
        return Err(ReadError::FailedToDeserializeCsv {
            message: "No records successfully deserialized".to_string(),
            file_name: format!("{:?}", &path),
        });
    }

    Ok(activities)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::read_track_plays;

    #[test]
    fn parse_json_test() {
        let mut path = PathBuf::new();
        path.push("./fixtures");
        path.push("spotify_listens");
        path.set_extension("json");

        let plays = read_track_plays(&path).unwrap();
        println!("{:?}", &plays);
        assert_eq!(1, plays.len());
    }

    #[test]
    fn parse_csv_test() {
        let mut path = PathBuf::new();
        path.push("./fixtures");
        path.push("apple_music_play_activity");
        path.set_extension("csv");

        let plays = read_track_plays(&path).unwrap();
        println!("{:?}", &plays);
        assert_eq!(1, plays.len());
    }
}
