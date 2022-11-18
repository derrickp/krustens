pub mod apple_music;
mod artist_name;
mod normalized;
mod parse;
mod spotify;
mod track_play;

pub use artist_name::ArtistName;
pub use normalized::Normalized;
pub use parse::read_track_plays;
pub use spotify::Spotify;
pub use track_play::TrackPlay;
