mod cli;
mod commands;
mod errors;
mod events;
mod persistence;
mod projections;
mod stores;
mod track_plays;

use std::{fs, str::FromStr, sync::Arc};

use chrono::Weekday;
use clap::Parser;
use projections::{
    stats::{FileName, Folder},
    AlbumListenCounts,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Pool, Sqlite,
};
use stores::SqliteStore;
use tokio::sync::Mutex;
use tune_sage::api::{cache::FileSystemCache, recordings::RecordingApi, HttpRemote};

use crate::{
    commands::AddTrackPlay,
    persistence::{FileWriter, Writer},
    projections::{
        listen_tracker_repo,
        stats::{DayStat, Stats},
    },
    stores::EventStore,
    track_plays::Spotify,
};

pub const MIN_LISTEN_LENGTH: u64 = 1000 * 10;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let database_file = "krustens.sqlite";
    let database_url = format!("sqlite://{}", database_file);

    let connection_options = SqliteConnectOptions::from_str(&database_url)
        .unwrap()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .connect_with(connection_options)
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    sqlx::query("pragma temp_store = memory;")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("pragma mmap_size = 30000000000;")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("pragma page_size = 4096;")
        .execute(&pool)
        .await
        .unwrap();

    let args = cli::Arguments::parse();

    match args.command {
        cli::Commands::Process(process_args) => {
            process_listens(&process_args.input, SqliteStore::build(pool.clone()), &pool).await;
            Ok(())
        }
        cli::Commands::Generate(generate_args) => {
            generate_stats(
                &generate_args.output,
                generate_args.count,
                SqliteStore::build(pool.clone()),
                generate_args.year,
                generate_args.split_monthly,
            )
            .await;
            Ok(())
        }
    }
}

async fn generate_stats(
    output_folder: &str,
    count: usize,
    store: SqliteStore,
    year: Option<i32>,
    split_monthly: bool,
) {
    match year {
        Some(it) => {
            generate_stats_for_single_year(output_folder, count, store, it, split_monthly).await
        }
        _ => generate_all_stats(output_folder, count, store).await,
    }
}

async fn generate_all_stats(output_folder: &str, count: usize, store: SqliteStore) {
    let folder = Folder {
        output_folder: output_folder.to_string(),
        year: None,
        month: None,
    };
    let event_stream = store.get_events("listens".to_string()).await.unwrap();

    let cache = Arc::new(Mutex::new(FileSystemCache::for_folder("./output")));
    let http_remote = HttpRemote;
    let config = tune_sage::api::Config {
        base_url: "https://musicbrainz.org/ws/2".to_string(),
        user_agent: "Krustens <https://github.com/derrickp/krustens>".to_string(),
    };

    let api = RecordingApi {
        config,
        cache,
        remote: Arc::new(http_remote),
    };

    let mut count_projection = AlbumListenCounts {
        recording_api: Arc::new(Mutex::new(api)),
    };

    let monthly_stats = count_projection
        .monthly_listens(event_stream.events.iter().collect())
        .await;

    FileWriter::from("./output/monthly.json".to_string())
        .write(&monthly_stats)
        .await
        .unwrap();

    let stats = Stats::generate(event_stream.events.iter().collect());
    write_stats(&folder, &stats, count).await;
}

async fn generate_stats_for_single_year(
    output_folder: &str,
    count: usize,
    store: SqliteStore,
    year: i32,
    split_monthly: bool,
) {
    let event_stream = store.get_events("listens".to_string()).await.unwrap();

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
                .await
                .unwrap();

            write_stats(&folder, &stats, count).await;
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
        .await
        .unwrap();

    write_stats(&folder, &stats, count).await;
}

async fn write_stats(stats_folder: &Folder, stats: &Stats, count: usize) {
    stats_folder.create_if_necessary();

    FileWriter::yaml_writer(stats_folder.file_name(&FileName::General))
        .write(&stats.general_stats(count))
        .await
        .unwrap();
    FileWriter::from(stats_folder.file_name(&FileName::Complete))
        .write(stats)
        .await
        .unwrap();
    FileWriter::from(stats_folder.file_name(&FileName::Top50))
        .write(&stats.top(50))
        .await
        .unwrap();
    FileWriter::from(stats_folder.file_name(&FileName::Top100))
        .write(&stats.top(100))
        .await
        .unwrap();
}

async fn process_listens(input_folder: &str, store: SqliteStore, pool: &Pool<Sqlite>) {
    let mut repository = listen_tracker_repo(1500, pool).await;
    let streaming_files =
        fs::read_dir(input_folder).unwrap_or_else(|_| panic!("Could not read {}", &input_folder));

    for entry in streaming_files {
        let path = entry.unwrap().path().clone();

        if !format!("{}", &path.display()).ends_with(".json") {
            continue;
        }

        let contents = fs::read_to_string(format!("{}", &path.display())).unwrap();
        let listens: Vec<Spotify> = match serde_json::from_str(&contents) {
            Ok(it) => it,
            _ => {
                println!("could not parse {}", path.display());
                continue;
            }
        };
        for listen in listens.iter() {
            let command = AddTrackPlay {
                track_play: track_plays::TrackPlay::Spotify(listen.clone()),
                min_listen_length: MIN_LISTEN_LENGTH,
            };
            let tracker = repository.get(&store).await;
            let handle_result = command.handle(tracker);

            if let Some(event) = handle_result {
                if let Err(err) = store
                    .add_event("listens".to_string(), &event, event.version)
                    .await
                {
                    println!("{:?}", err);
                }
            }
        }

        println!("processed {}", path.display());
    }

    let _ = repository.get(&store).await;
    repository.flush().await;
}
