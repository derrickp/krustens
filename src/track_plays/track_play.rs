use serde::Deserialize;

use super::{apple_music, Normalized, Spotify};

#[derive(Debug, Deserialize, Clone)]
pub enum TrackPlay {
    Spotify(Spotify),
    AppleMusicPlayActivity(apple_music::PlayActivity),
}

impl TryInto<Normalized> for TrackPlay {
    type Error = ();

    fn try_into(self) -> Result<Normalized, Self::Error> {
        match self {
            TrackPlay::Spotify(it) => it.try_into(),
            TrackPlay::AppleMusicPlayActivity(it) => it.try_into(),
        }
    }
}
