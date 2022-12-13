use crate::track_plays::{ArtistName, TrackName};

pub trait HasListen {
    fn has_listen(&self, artist_name: &ArtistName, track_name: &TrackName, end_time: &str) -> bool;
    fn version(&self) -> u32;
}
