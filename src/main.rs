mod commands;
mod events;
mod history;
mod projections;
mod spotify;
mod stores;

use std::{env, fs::{self, File}, io::{BufWriter, Write}};

use spotify::track_play::TrackPlay;

use crate::{
    commands::add_spotify_listen::AddSpotifyListen,
    projections::{repository::Repository, stats::Stats},
    stores::store::Store,
};

pub const MIN_LISTEN_LENGTH: u64 = 1000 * 60; // 1000ms in s, 60s in minute

fn main() {
    let args: Vec<String> = env::args().collect();

    let run_command = match args.get(1) {
        Some(it) => it.clone(),
        _ => "".to_string(),
    };

    let file_name = "./derrick_garbage/streaming_history.json";
    let contents = fs::read_to_string(file_name).unwrap();
    let listens: Vec<TrackPlay> = serde_json::from_str(&contents).unwrap();
    let mut store = Store::build("./derrick_garbage/messages.json".to_string());

    match run_command.as_str() {
        "process_listens" => {
            let mut repository = Repository::build("./derrick_garbage/listen_tracker.json");

            let mut snapshot_count = 0;

            for listen in listens.iter() {
                let command = AddSpotifyListen {
                    listen: listen.clone(),
                    min_listen_length: MIN_LISTEN_LENGTH,
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
            let stats = Stats::count(event_stream.events.iter().collect());

            println!("=========== Artist Total Plays ==========");
            print!("\n");
            let top_10 = stats.top(10);

            for artist in top_10 {
                println!("{} - {}", artist.artist_name, artist.total_plays());
            }

            print!("\n\n");

            println!("=========== Most Played Songs ==========");
            print!("\n");
            let top_10_songs = stats.top_songs(10);

            for song_count in top_10_songs {
                println!("{} - {} - {}", song_count.artist_name, song_count.song_name, song_count.count);
            }

            print!("\n\n");
            println!("=========== Unique Artists Songs ==========");
            print!("\n");

            let top_unique_artists = stats.top_unique_artists(10);

            for top_artist in top_unique_artists {
                let max_song = &top_artist.max_song_play();
                println!("{} - {} - {}", top_artist.artist_name, max_song.song_name.clone(), max_song.count);
            }

            print!("\n\n");
            println!("Total artists listened to - {}", stats.artist_count());

            let file = File::create("./derrick_garbage/stats.json").unwrap();
            let mut writer = BufWriter::new(file);
            serde_json::to_writer(&mut writer, &stats).unwrap();
            writer.flush().unwrap();

            let top_50_artists = stats.top(50);
            let top_50_file = File::create("./derrick_garbage/top_50_stats.json").unwrap();
            let mut writer = BufWriter::new(top_50_file);
            serde_json::to_writer(&mut writer, &top_50_artists).unwrap();
            writer.flush().unwrap();

            let top_100_artists = stats.top(100);
            let top_100_file = File::create("./derrick_garbage/top_100_stats.json").unwrap();
            let mut writer = BufWriter::new(top_100_file);
            serde_json::to_writer(&mut writer, &top_100_artists).unwrap();
            writer.flush().unwrap();
        }
    }
}
