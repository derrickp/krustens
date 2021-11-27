mod commands;
mod events;
mod history;
mod persistence;
mod projections;
mod spotify;
mod stores;

use std::{
    env,
    fs::{self, create_dir},
    path::Path,
};

use spotify::track_play::TrackPlay;

use crate::{
    commands::add_spotify_listen::AddSpotifyListen,
    persistence::{file_writer::FileWriter, json_reader::JsonReader, writer::Writer},
    projections::{repository::Repository, stats::Stats},
    stores::store::Store,
};

pub const MIN_LISTEN_LENGTH: u64 = 1000 * 60; // 1000ms in s, 60s in minute

struct AppFiles {
    folder: &'static str,
}

impl AppFiles {
    fn streams_file(&self) -> String {
        format!("{}/streams.json", self.folder)
    }

    fn snapshot_file(&self) -> String {
        format!("{}/snapshot.json", self.folder)
    }

    fn streams_writer(&self) -> FileWriter {
        FileWriter::from(self.streams_file())
    }

    fn snapshot_writer(&self) -> FileWriter {
        FileWriter::from(self.snapshot_file())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let run_command = match args.get(1) {
        Some(it) => it.clone(),
        _ => "".to_string(),
    };

    let app_files = AppFiles {
        folder: "./app_data",
    };

    if !Path::new(app_files.folder).exists() {
        create_dir(app_files.folder).unwrap();
    }

    let history_folder = "./derrick_garbage/spotify_play_history";
    let stats_folder = "./derrick_garbage/output/stats";

    let existing_stream = match fs::read_to_string(app_files.streams_file()) {
        Ok(it) => it,
        _ => "".to_string(),
    };
    let stream_reader = JsonReader {
        contents: &existing_stream,
    };
    let mut store = Store::build(&stream_reader);

    match run_command.as_str() {
        "process_listens" => {
            let snapshot_contents = match fs::read_to_string(app_files.snapshot_file()) {
                Ok(it) => it,
                _ => "".to_string(),
            };
            let snapshot_reader = JsonReader {
                contents: &snapshot_contents,
            };
            let mut repository = Repository::build(1500, &snapshot_reader);
            let streaming_files = fs::read_dir(history_folder).unwrap();

            for entry in streaming_files.into_iter() {
                let path = entry.unwrap().path().clone();

                if !path.display().to_string().ends_with(".json") {
                    continue;
                }

                let contents = fs::read_to_string(path.display().to_string()).unwrap();
                let listens: Vec<TrackPlay> = serde_json::from_str(&contents).unwrap();
                let mut listen_count = 0;
                for listen in listens.iter() {
                    let command = AddSpotifyListen {
                        listen: listen.clone(),
                        min_listen_length: MIN_LISTEN_LENGTH,
                    };
                    let tracker = repository.get_tracker(&store, &app_files.snapshot_writer());
                    let handle_result = command.handle(tracker);

                    if let Some(event) = handle_result {
                        if let Err(err) = store.add_event(
                            "listens".to_string(),
                            &event,
                            event.version,
                            &app_files.streams_writer(),
                        ) {
                            println!("{:?}", err);
                        }
                    }

                    listen_count += 1;
                }

                println!("read {} listens", listen_count);
            }
        }
        _ => {
            let event_stream = store.get_events("listens".to_string()).unwrap();
            let stats = Stats::count(event_stream.events.iter().collect());

            println!("=========== Artist Total Plays ==========");
            println!();
            let top_10 = stats.top(10);

            for artist in top_10 {
                println!("{} - {}", artist.artist_name, artist.total_plays());
            }

            println!();
            println!();

            println!("=========== Most Played Songs ==========");
            println!();
            let top_10_songs = stats.top_songs(10);

            for song_count in top_10_songs {
                println!(
                    "{} - {} - {}",
                    song_count.artist_name, song_count.song_name, song_count.count
                );
            }

            println!();
            println!();
            println!("=========== Unique Artists Songs ==========");
            println!();

            let top_unique_artists = stats.top_unique_artists(10);

            for top_artist in top_unique_artists {
                let max_song = &top_artist.max_song_play();
                println!(
                    "{} - {} - {}",
                    top_artist.artist_name,
                    max_song.song_name.clone(),
                    max_song.count
                );
            }

            println!();
            println!("Total artists listened to - {}", stats.artist_count());

            let complete_stats_writer = FileWriter::from(format!("{}/complete.json", stats_folder));
            complete_stats_writer.write(&stats).unwrap();

            let top_50_artists = stats.top(50);
            let top_50_writer = FileWriter::from(format!("{}/top_50.json", stats_folder));
            top_50_writer.write(&top_50_artists).unwrap();

            let top_100_artists = stats.top(100);
            let top_100_writer = FileWriter::from(format!("{}/top_100.json", stats_folder));
            top_100_writer.write(&top_100_artists).unwrap();
        }
    }
}
