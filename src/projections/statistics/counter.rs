use serde::{Deserialize, Serialize};

use crate::track_plays::{AlbumName, ArtistName, TrackName};

use super::{count::AlbumCount, SongCount, TimePlayed};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ArtistSongCounter {
    pub artist_name: ArtistName,
    pub play_details: SongCounter,
}

impl ArtistSongCounter {
    pub fn total_song_plays(&self) -> u64 {
        self.play_details.total_song_plays()
    }

    pub fn total_plays_display(&self) -> String {
        format!("{} - {}", &self.artist_name, self.total_song_plays())
    }

    pub fn max_song_display(&self) -> String {
        format!(
            "{} - {}",
            &self.artist_name,
            self.play_details.max_song_play()
        )
    }

    pub fn max_song_play(&self) -> SongCount {
        self.play_details.max_song_play()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ArtistAlbumCounter {
    pub artist_name: ArtistName,
    pub play_details: AlbumCounter,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct AlbumCounter {
    total_plays: u64,
    album_counts: Vec<AlbumCount>,
}

impl AlbumCounter {
    pub fn increment_album(&mut self, album_name: &AlbumName) {
        match self
            .album_counts
            .iter_mut()
            .find(|album_count| album_count.0.eq_ignore_ascii_case(album_name))
        {
            Some(it) => it.1 += 1,
            _ => {
                let album_count = AlbumCount(album_name.clone(), 1);
                self.album_counts.push(album_count);
            }
        }
        self.total_plays += 1;
    }

    pub fn all_album_plays(&self) -> Vec<AlbumCount> {
        self.album_counts.to_vec()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct SongCounter {
    total_song_plays: u64,
    total_time_played: TimePlayed,
    song_counts: Vec<SongCount>,
}

impl SongCounter {
    pub fn add(&mut self, other: &SongCounter) {
        self.total_song_plays += other.total_song_plays;
        self.total_time_played
            .add_ms(other.total_time_played.time_ms);
        for song_count in other.song_counts.iter() {
            self.increment_song(&song_count.0, song_count.1);
        }
    }

    pub fn total_song_plays(&self) -> u64 {
        self.total_song_plays
    }

    pub fn increment_song(&mut self, track_name: &TrackName, time_played: u64) {
        match self
            .song_counts
            .iter_mut()
            .find(|song_play_count| song_play_count.0.eq_ignore_ascii_case(track_name))
        {
            Some(it) => it.1 += 1,
            _ => {
                let song_count = SongCount(track_name.clone(), 1);
                self.song_counts.push(song_count);
            }
        }
        self.total_song_plays += 1;
        self.total_time_played.add_ms(time_played);
    }

    pub fn all_song_plays(&self) -> Vec<SongCount> {
        self.song_counts.to_vec()
    }

    pub fn max_song_play(&self) -> SongCount {
        self.song_counts
            .iter()
            .max_by_key(|song_count| song_count.1)
            .cloned()
            .unwrap_or_default()
    }
}
