use std::collections::HashMap;

use crate::history::listen::Listen;

use super::has_listen::HasListen;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ListenTracker {
    pub listens: HashMap<String, Listen>,
    pub version: u64,
}

impl HasListen for ListenTracker {
    fn has_listen(&self, artist_name: &str, track_name: &str, end_time: &str) -> bool {
        let id = build_id(artist_name, track_name, end_time);
        self.listens.contains_key(&id)
    }
}

pub fn build_id(artist_name: &str, track_name: &str, end_time: &str) -> String {
    format!("{}-{}-{}", artist_name, track_name, end_time)
}
