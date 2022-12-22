use serde::Deserialize;
use std::u64;

use crate::utils::{parse_end_time_rfc3339, parse_spotify_end_time};

use super::{AlbumName, ArtistName, Normalized, TrackName};

#[derive(Debug, Deserialize, Clone)]
pub struct Spotify {
    #[serde(rename = "endTime", alias = "ts")]
    pub end_time: Option<String>,
    #[serde(rename = "artistName", alias = "master_metadata_album_artist_name")]
    pub artist_name: Option<String>,
    #[serde(rename = "trackName", alias = "master_metadata_track_name")]
    pub track_name: Option<String>,
    #[serde(alias = "master_metadata_album_album_name")]
    pub album_name: Option<String>,
    #[serde(rename = "msPlayed", alias = "ms_played")]
    pub ms_played: u64,
    pub skipped: Option<bool>,
}

impl Spotify {
    pub fn is_valid(&self) -> bool {
        self.end_time.is_some() && self.artist_name.is_some() && self.track_name.is_some()
    }
}

impl TryInto<Normalized> for Spotify {
    type Error = ();

    fn try_into(self) -> Result<Normalized, Self::Error> {
        let end_time = match self
            .end_time
            .map(|text| parse_spotify_end_time(&text).or_else(|_| parse_end_time_rfc3339(&text)))
        {
            Some(Ok(it)) => it,
            _ => return Err(()),
        };

        let artist_name = if let Some(artist_name) = self.artist_name {
            ArtistName(artist_name)
        } else {
            return Err(());
        };

        let track_name = if let Some(track_name) = self.track_name {
            TrackName(track_name)
        } else {
            return Err(());
        };

        Ok(Normalized {
            end_time,
            album_name: self.album_name.map(AlbumName),
            artist_name,
            track_name,
            service_hint: "spotify".to_string(),
            ms_played: Some(self.ms_played),
            track_ms: None,
            skipped: self.skipped,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use crate::track_plays::Normalized;

    use super::Spotify;

    #[test]
    fn test_full_history_end_time() {
        let play = Spotify {
            end_time: Some("2019-10-30T21:13:23Z".to_string()),
            artist_name: Some("Whitechapel".to_string()),
            track_name: Some("Brimstone".to_string()),
            album_name: None,
            ms_played: 20222,
            skipped: None,
        };

        let normalized: Normalized = play.try_into().unwrap();
        assert_eq!(2019, normalized.end_time.year());
    }

    #[test]
    fn test_partial_end_time() {
        let play = Spotify {
            end_time: Some("2021-10-21 17:17".to_string()),
            artist_name: Some("Whitechapel".to_string()),
            track_name: Some("Brimstone".to_string()),
            album_name: None,
            ms_played: 20222,
            skipped: None,
        };

        let normalized: Normalized = play.try_into().unwrap();
        assert_eq!(2021, normalized.end_time.year());
    }
}
