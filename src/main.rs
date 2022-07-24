mod app_files;
mod commands;
mod events;
mod persistence;
mod projections;
mod spotify;
mod stores;

use std::{
    fs::{self, create_dir},
    path::Path,
};

use app_files::AppFiles;
use chrono::Weekday;
use clap::{Arg, Command};
use projections::stats::folder::{FileName, Folder};
use spotify::track_play::TrackPlay;

use crate::{
    commands::add_spotify_listen::AddSpotifyListen,
    persistence::{file_writer::FileWriter, json_reader::JsonReader, writer::Writer},
    projections::{
        repository::Repository,
        stats::{day_stat::DayStat, Stats},
    },
    stores::store::Store,
};

pub const MIN_LISTEN_LENGTH: u64 = 1000 * 60; // 1000ms in s, 60s in minute

fn main() {
    let app = Command::new("krustens")
        .version("1.0.1")
        .author("derrickp")
        .about("Generate stats from spotify history")
        .arg(
            Arg::new("input")
                .long("input")
                .short('i')
                .required(false)
                .takes_value(true)
                .help("folder that contains the spotify streaming history")
                .default_value("./data/spotify_play_history"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .takes_value(true)
                .help("Folder to place the generated stats in")
                .default_value("./output"),
        )
        .arg(
            Arg::new("count")
                .long("count")
                .short('c')
                .takes_value(true)
                .validator(|count| match str::parse::<usize>(count) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                })
                .help("how many top artists/songs to include in the generated general statistics")
                .default_value("25"),
        )
        .arg(
            Arg::new("year")
                .long("year")
                .short('y')
                .required(false)
                .takes_value(true)
                .validator(|year| match str::parse::<i32>(year) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                })
                .help("year to generate stats for"),
        )
        .arg(
            Arg::new("split_monthly")
                .long("split_monthly")
                .short('s')
                .takes_value(true)
                .validator(|include| match str::parse::<bool>(include) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                })
                .help("split year stats down by months")
                .default_value("false"),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .short('m')
                .takes_value(true)
                .possible_values(&["process", "generate"]),
        );

    let matches = app.get_matches();
    let mode = matches.value_of("mode").unwrap_or("generate");
    let year = matches
        .value_of("year")
        .map(|it| str::parse::<i32>(it).unwrap());
    let split_monthly = match matches.value_of("split_monthly") {
        Some(it) => str::parse::<bool>(it).unwrap(),
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

    let existing_stream = match fs::read_to_string(app_files.streams_file()) {
        Ok(it) => it,
        _ => "".to_string(),
    };
    let stream_reader = JsonReader {
        contents: &existing_stream,
    };

    match mode {
        "process" => {
            process_listens(app_files, input_folder, Store::build(&stream_reader));
        }
        _ => {
            generate_stats(
                output_folder,
                stats_count,
                Store::build(&stream_reader),
                year,
                split_monthly,
            );
        }
    }
}

fn generate_stats(
    output_folder: &str,
    count: usize,
    store: Store,
    year: Option<i32>,
    split_monthly: bool,
) {
    match year {
        Some(it) => generate_stats_for_single_year(output_folder, count, store, it, split_monthly),
        _ => generate_all_stats(output_folder, count, store),
    }
}

fn generate_all_stats(output_folder: &str, count: usize, store: Store) {
    let folder = Folder {
        output_folder: output_folder.to_string(),
        year: None,
        month: None,
    };
    let event_stream = store.get_events("listens".to_string()).unwrap();

    let stats = Stats::generate(event_stream.events.iter().collect());
    write_stats(&folder, &stats, count);
}

fn generate_stats_for_single_year(
    output_folder: &str,
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

            let folder = Folder {
                output_folder: output_folder.to_string(),
                year: Some(year),
                month: Some(month),
            };

            let weekdays = vec![
                Weekday::Sun,
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
            ];

            let day_stats: Vec<DayStat> = weekdays
                .iter()
                .map(|day| {
                    Stats::generate_day_stat(
                        event_stream.events.iter().collect(),
                        year,
                        month,
                        *day,
                        5,
                    )
                })
                .collect();

            folder.create_if_necessary();
            FileWriter::yaml_writer(folder.file_name(&FileName::Daily))
                .write(&day_stats)
                .unwrap();

            write_stats(&folder, &stats, count);
        }
    }

    let stats = Stats::generate_for_year(event_stream.events.iter().collect(), year);

    let weekdays = vec![
        Weekday::Sun,
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
    ];

    let day_stats: Vec<DayStat> = weekdays
        .iter()
        .map(|day| {
            Stats::generate_day_stat_all_year(event_stream.events.iter().collect(), year, *day, 10)
        })
        .collect();

    let folder = Folder {
        output_folder: output_folder.to_string(),
        year: Some(year),
        month: None,
    };

    folder.create_if_necessary();
    FileWriter::yaml_writer(folder.file_name(&FileName::Daily))
        .write(&day_stats)
        .unwrap();

    write_stats(&folder, &stats, count);
}

fn write_stats(stats_folder: &Folder, stats: &Stats, count: usize) {
    stats_folder.create_if_necessary();

    FileWriter::yaml_writer(stats_folder.file_name(&FileName::General))
        .write(&stats.general_stats(count))
        .unwrap();
    FileWriter::from(stats_folder.file_name(&FileName::Complete))
        .write(stats)
        .unwrap();
    FileWriter::from(stats_folder.file_name(&FileName::Top50))
        .write(&stats.top(50))
        .unwrap();
    FileWriter::from(stats_folder.file_name(&FileName::Top100))
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
        fs::read_dir(&input_folder).unwrap_or_else(|_| panic!("Could not read {}", &input_folder));

    for entry in streaming_files {
        let path = entry.unwrap().path().clone();

        if !format!("{}", &path.display()).ends_with(".json") {
            continue;
        }

        let contents = fs::read_to_string(format!("{}", &path.display())).unwrap();
        let listens: Vec<TrackPlay> = match serde_json::from_str(&contents) {
            Ok(it) => it,
            _ => {
                println!("could not parse {}", path.display());
                continue;
            }
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

        println!("processed {}", path.display());
    }

    repository.flush(&app_files.snapshot_writer());
    store.flush(&app_files.streams_writer());
}
