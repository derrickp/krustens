use serde::Serialize;

#[derive(Serialize)]
pub struct General {
    pub count_artists_listened_to: usize,
    pub artist_total_plays: Vec<String>,
    pub most_played_songs: Vec<String>,
    pub artist_most_played_songs: Vec<String>,
}
