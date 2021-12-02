mod app_files;
mod commands;
mod config;
mod events;
mod persistence;
mod projections;
mod spotify;
mod stores;

use std::{
    fs::{self, create_dir, create_dir_all},
    path::Path,
};

use app_files::AppFiles;
use clap::{App, Arg};
use config::Config;
use spotify::track_play::TrackPlay;

use crate::{
    commands::add_spotify_listen::AddSpotifyListen,
    persistence::{file_writer::FileWriter, json_reader::JsonReader, writer::Writer},
    projections::{repository::Repository, stats::Stats},
    stores::store::Store,
};

pub const MIN_LISTEN_LENGTH: u64 = 1000 * 60; // 1000ms in s, 60s in minute

fn main() {
    let app = App::new("krustens")
        .version("0.1")
        .author("derrickp")
        .about("Generate stats from spotify history")
        .arg(
            Arg::with_name("year")
                .long("year")
                .short("y")
                .required(false)
                .takes_value(true)
                .validator(|year| match str::parse::<i32>(&year) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                })
                .help("year to generate stats for"),
        )
        .arg(
            Arg::with_name("mode")
                .long("mode")
                .short("m")
                .takes_value(true)
                .possible_values(&["process", "generate"]),
        );

    let matches = app.get_matches();
    let mode = matches.value_of("mode").unwrap_or("generate");
    let year = match matches.value_of("year") {
        Some(it) => Some(str::parse::<i32>(&it).unwrap()),
        _ => None,
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

    println!("{}", mode.clone());

    match mode {
        "process" => {
            process_listens(app_files, config, Store::build(&stream_reader));
        }
        _ => {
            generate_stats(config, Store::build(&stream_reader), year);
        }
    }
}

fn generate_stats(config: Config, store: Store, year: Option<i32>) {
    let event_stream = store.get_events("listens".to_string()).unwrap();
    let stats = match year {
        Some(it) => Stats::generate_for_year(event_stream.events.iter().collect(), it),
        _ => Stats::generate(event_stream.events.iter().collect()),
    };

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

    repository.flush(&app_files.snapshot_writer());
    store.flush(&app_files.streams_writer());
}
