use std::collections::HashMap;

use crate::{
    events::{event::Event, event_data::EventData, listen_added::ListenAdded},
    history::listen::Listen,
};

use super::has_listen::HasListen;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListenTracker {
    pub listens: HashMap<String, Listen>,
    pub version: i32,
}

impl HasListen for ListenTracker {
    fn has_listen(&self, artist_name: &str, track_name: &str, end_time: &str) -> bool {
        let id = build_id(artist_name, track_name, end_time);
        self.listens.contains_key(&id)
    }
}

pub fn build_tracker(events: Vec<Event>) -> Box<dyn HasListen> {
    let mut all_listens: HashMap<String, Listen> = HashMap::new();
    let version = events.len() as i32;

    for event in events {
        match event.data {
            EventData::ListenAdded(listen) => handle_listen_added(&mut all_listens, listen),
        }
    }

    Box::new(ListenTracker {
        version,
        listens: all_listens,
    })
}

fn handle_listen_added(all_listens: &mut HashMap<String, Listen>, listen_added: ListenAdded) {
    let id = build_id(
        &listen_added.artist_name,
        &listen_added.track_name,
        &listen_added.end_time,
    );
    let listen = Listen {
        id: id.clone(),
        artist_name: listen_added.artist_name,
        track_name: listen_added.track_name,
        end_time: listen_added.end_time,
        ms_played: listen_added.ms_played,
    };
    all_listens.insert(id, listen);
}

fn build_id(artist_name: &str, track_name: &str, end_time: &str) -> String {
    format!("{}-{}-{}", artist_name, track_name, end_time)
}
