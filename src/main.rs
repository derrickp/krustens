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
            Arg::with_name("input")
                .long("input")
                .short("i")
                .required(false)
                .takes_value(true)
                .help("folder that contains the spotify streaming history")
                .default_value("./data/spotify_play_history"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("Folder to place the generated stats in")
                .default_value("./output"),
        )
        .arg(
            Arg::with_name("count")
                .long("count")
                .short("c")
                .takes_value(true)
                .validator(|count| match str::parse::<usize>(&count) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                })
                .help("how many top artists/songs to include in the generated general statistics")
                .default_value("25"),
        )
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
            Arg::with_name("split_monthly")
                .long("split_monthly")
                .short("s")
                .takes_value(true)
                .validator(|include| match str::parse::<bool>(&include) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                })
                .help("split year stats down by months")
                .default_value("false"),
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
    let split_monthly = match matches.value_of("split_monthly") {
        Some(it) => str::parse::<bool>(&it).unwrap(),
        _ => false,
    };
    let output_folder = matches.value_of("output").unwrap();
    let input_folder = matches.value_of("input").unwrap();
    let stats_count = str::parse::<usize>(matches.value_of("count").unwrap()).unwrap();

    let app_files = AppFiles {
        folder: "./app_data",
    };

    if !Path::new(app_files.folder).exists() {
        create_dir(app_files.folder).unwrap();
    }

    if !Path::new(Config::stats_folder(&output_folder).as_str()).exists() {
        create_dir_all(Config::stats_folder(&output_folder).as_str()).unwrap()
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
            process_listens(app_files, &input_folder, Store::build(&stream_reader));
        }
        _ => {
            generate_stats(
                &output_folder,
                stats_count,
                Store::build(&stream_reader),
                year,
                split_monthly,
            );
        }
    }
}

fn generate_stats(
    stats_folder: &str,
    count: usize,
    store: Store,
    year: Option<i32>,
    split_monthly: bool,
) {
    match year {
        Some(it) => generate_stats_for_single_year(stats_folder, count, store, it, split_monthly),
        _ => generate_all_stats(stats_folder, count, store),
    }
}

fn generate_all_stats(stats_folder: &str, count: usize, store: Store) {
    let event_stream = store.get_events("listens".to_string()).unwrap();

    let stats = Stats::generate(event_stream.events.iter().collect());
    write_stats(&stats_folder, &stats, count);
}

fn generate_stats_for_single_year(
    stats_folder: &str,
    count: usize,
    store: Store,
    year: i32,
    split_monthly: bool,
) {
    let event_stream = store.get_events("listens".to_string()).unwrap();

    if split_monthly {
        for month in 1..=12 {
            let stats = Stats::generate_month_year(
                event_stream.events.iter().collect(),
                year,
                month as u32,
            );

            let output_folder = format!("{}/{}_{}", &stats_folder, year, month).clone();

            if !Path::new(Config::stats_folder(&output_folder).as_str()).exists() {
                create_dir_all(Config::stats_folder(&output_folder).as_str()).unwrap()
            }

            write_stats(&output_folder, &stats, count);
        }
    } else {
        let stats = Stats::generate_for_year(event_stream.events.iter().collect(), year);
        write_stats(&stats_folder, &stats, count);
    }
}

fn write_stats(stats_folder: &str, stats: &Stats, count: usize) {
    FileWriter::yaml_writer(Config::general_stats_file(stats_folder))
            .write(&stats.general_stats(count))
            .unwrap();
        FileWriter::from(Config::complete_stats_file(stats_folder))
            .write(stats)
            .unwrap();
        FileWriter::from(Config::top_50_stats_file(stats_folder))
            .write(&stats.top(50))
            .unwrap();
        FileWriter::from(Config::top_100_stats_file(stats_folder))
            .write(&stats.top(100))
            .unwrap();
}

fn process_listens(app_files: AppFiles, input_folder: &str, mut store: Store) {
    let snapshot_contents = match fs::read_to_string(app_files.snapshot_file()) {
        Ok(it) => it,
        _ => "".to_string(),
    };
    let snapshot_reader = JsonReader {
        contents: &snapshot_contents,
    };
    let mut repository = Repository::build(1500, &snapshot_reader);
    let streaming_files =
        fs::read_dir(&input_folder).expect(format!("Could not read {}", &input_folder).as_str());

    for entry in streaming_files.into_iter() {
        let path = entry.unwrap().path().clone();

        if !path.display().to_string().ends_with(".json") {
            continue;
        }

        let contents = fs::read_to_string(&path.display().to_string()).unwrap();
        let listens: Vec<TrackPlay> = match serde_json::from_str(&contents) {
            Ok(it) => it,
            _ => continue,
        };
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
