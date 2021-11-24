mod commands;
mod events;
mod history;
mod projections;
mod spotify;
mod stores;

use std::fs;

use spotify::listen::Listen;

use crate::{
    commands::add_spotify_listen::AddSpotifyListen, projections::stats::Stats, stores::store::Store,
};

fn main() {
    let file_name = "./derrick_garbage/streaming_history.json";
    let contents = fs::read_to_string(file_name).unwrap();
    let listens: Vec<Listen> = serde_json::from_str(&contents).unwrap();
    let mut store = Store::build("./derrick_garbage/messages.json".to_string());

    println!("{}", listens.len());

    for listen in listens {
        let command = AddSpotifyListen {
            listen: listen.clone(),
        };
        let handle_result = command.handle(&store).unwrap();

        if let Some(event) = handle_result {
            match store.add_event("listens".to_string(), &event, event.version) {
                Err(err) => println!("{:?}", err),
                _ => continue,
            }
        }
    }

    let event_stream = store.get_events("listens".to_string()).unwrap();
    let artist_stats = Stats::count_by_artist(event_stream.events.iter().collect());
    let top_10 = artist_stats.top(10);
    for artist_count in top_10.iter() {
        println!("{} - {}", artist_count.name, artist_count.play_count);
    }

    println!("=========== Songs ==========");

    let song_stats = Stats::count_by_track(event_stream.events.iter().collect());
    let top_10_songs = song_stats.top(10);
    for song_count in top_10_songs.iter() {
        println!("{} - {}", song_count.name, song_count.play_count);
    }
}
