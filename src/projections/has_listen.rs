pub trait HasListen {
    fn has_listen(&self, artist_name: &str, track_name: &str, end_time: &str) -> bool;
}
