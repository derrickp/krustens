pub mod apple_music;
mod normalized;
mod parse;
mod spotify;
mod track_play;

pub use normalized::Normalized;
pub use parse::read_track_plays;
pub use spotify::Spotify;
pub use track_play::TrackPlay;
