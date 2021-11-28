mod commands;
mod config;
mod events;
mod history;
mod persistence;
mod projections;
mod spotify;
mod stores;

use std::{
    env,
    fs::{self, create_dir, create_dir_all},
    path::Path,
};

use config::Config;
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
    let config_content = fs::read_to_string("./resources/config.json").unwrap();
    let config: Config = serde_json::from_str(&config_content).unwrap();

    if !Path::new(app_files.folder).exists() {
        create_dir(app_files.folder).unwrap();
    }

    if !Path::new(&config.stats_folder()).exists() {
        create_dir_all(&config.stats_folder()).unwrap()
    }

    let existing_stream = match fs::read_to_string(app_files.streams_file()) {
        Ok(it) => it,
        _ => "".to_string(),
    };
    let stream_reader = JsonReader {
        contents: &existing_stream,
    };

    match run_command.as_str() {
        "process_listens" => {
            process_listens(app_files, config, Store::build(&stream_reader));
        }
        _ => {
            generate_stats(config, Store::build(&stream_reader));
        }
    }
}

fn generate_stats(config: Config, store: Store) {
    let event_stream = store.get_events("listens".to_string()).unwrap();
    let stats = Stats::generate(event_stream.events.iter().collect());

    FileWriter::yaml_writer(config.general_stats_file())
        .write(&stats.general_stats(config.count_general_stats_to_compile))
        .unwrap();
    FileWriter::from(config.complete_stats_file())
        .write(&stats)
        .unwrap();
    FileWriter::from(config.top_50_stats_file())
        .write(&stats.top(50))
        .unwrap();
    FileWriter::from(config.top_100_stats_file())
        .write(&stats.top(100))
        .unwrap();
}

fn process_listens(app_files: AppFiles, config: Config, mut store: Store) {
    let snapshot_contents = match fs::read_to_string(app_files.snapshot_file()) {
        Ok(it) => it,
        _ => "".to_string(),
    };
    let snapshot_reader = JsonReader {
        contents: &snapshot_contents,
    };
    let mut repository = Repository::build(1500, &snapshot_reader);
    let streaming_files = fs::read_dir(&config.history_folder)
        .expect(format!("Could not read {}", &config.history_folder).as_str());

    for entry in streaming_files.into_iter() {
        let path = entry.unwrap().path().clone();

        if !path.display().to_string().ends_with(".json") {
            continue;
        }

        let contents = fs::read_to_string(&path.display().to_string()).unwrap();
        let listens: Vec<TrackPlay> = serde_json::from_str(&contents).unwrap();
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
        }

        println!("processed {}", path.display().to_string());
    }
}
