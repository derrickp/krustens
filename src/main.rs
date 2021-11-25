mod commands;
mod events;
mod history;
mod projections;
mod spotify;
mod stores;

use std::{env, fs};

use spotify::listen::Listen;

use crate::{
    commands::add_spotify_listen::AddSpotifyListen,
    projections::{repository::Repository, stats::Stats},
    stores::store::Store,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    let run_command = match args.get(1) {
        Some(it) => it.clone(),
        _ => "".to_string(),
    };

    let file_name = "./derrick_garbage/streaming_history.json";
    let contents = fs::read_to_string(file_name).unwrap();
    let listens: Vec<Listen> = serde_json::from_str(&contents).unwrap();
    let mut store = Store::build("./derrick_garbage/messages.json".to_string());

    match run_command.as_str() {
        "process_listens" => {
            let mut repository = Repository::build("./derrick_garbage/listen_tracker.json");

            let mut snapshot_count = 0;

            for listen in listens.iter() {
                let command = AddSpotifyListen {
                    listen: listen.clone(),
                };
                let tracker = repository.get_tracker(&store);

                snapshot_count += 1;

                let handle_result = command.handle(tracker);

                if let Some(event) = handle_result {
                    match store.add_event("listens".to_string(), &event, event.version) {
                        Err(err) => println!("{:?}", err),
                        _ => continue,
                    }
                }

                if snapshot_count >= 100 {
                    repository.flush();
                    snapshot_count = 0;
                }
            }
        }
        _ => {
            let event_stream = store.get_events("listens".to_string()).unwrap();
            let artist_stats = Stats::count_by_artist(event_stream.events.iter().collect());
            let top_10 = artist_stats.top(10);

            println!("=========== Artists ==========");
            println!("");

            for artist_count in top_10.iter() {
                println!("{} - {}", artist_count.name, artist_count.play_count);
            }

            println!("");
            println!("=========== Songs ==========");
            println!("");

            let song_stats = Stats::count_by_track(event_stream.events.iter().collect());

            let top_10_songs = song_stats.top(25);
            for song_count in top_10_songs.iter() {
                println!("{} - {} - {}", song_count.artist_name, song_count.name, song_count.play_count);
            }

            println!("");
            println!("=========== Unique Artists Songs ==========");
            println!("");

            let top_10_unique_artists = song_stats.top_unique_artists(25);
            for song_count in top_10_unique_artists.iter() {
                println!("{} - {} - {}", song_count.artist_name, song_count.name, song_count.play_count);
            }
            println!("");
        }
    }
}
