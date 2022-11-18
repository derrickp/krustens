mod cli;
mod commands;
mod errors;
mod events;
mod persistence;
mod projections;
mod stores;
mod track_plays;
mod utils;

use std::{fs, io::Write, str::FromStr};

use clap::Parser;
use projections::{
    statistics::{ArtistsCounts, EventProcessor},
    statistics::{FileName, Folder},
};
use rand::Rng;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Pool, Sqlite,
};
use stores::SqliteStore;
use track_plays::ArtistName;

use crate::{
    commands::AddTrackPlay,
    persistence::{FileWriter, Writer},
    projections::listen_tracker_repo,
    stores::EventStore,
    track_plays::read_track_plays,
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
        cli::Commands::Interactive => {
            interactive(SqliteStore::build(pool.clone())).await;
            Ok(())
        }
    }
}

fn prompt(name: &str) -> String {
    let mut line = String::new();
    print!("{}", name);
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    return line.trim().to_string();
}

fn prompt_random_artist(processor: &EventProcessor) -> Vec<String> {
    let year = prompt("What year to look in? > ")
        .parse::<i32>()
        .expect("Error: Not a valid number");
    let num_artists = prompt("How many artists do you want names of (Default: 1) > ")
        .parse::<u32>()
        .ok()
        .filter(|num| num > &0)
        .unwrap_or(1);
    let min_listens = prompt("What minimum number of listens? > ")
        .parse::<u64>()
        .unwrap_or_default();
    let month = prompt("What month do you want to look in (1-12)? > ")
        .parse::<u32>()
        .ok()
        .filter(|m| (&1..=&12).contains(&m))
        .unwrap_or_default();

    processor
        .year_count(year)
        .map(|year_count| {
            let mut rng = rand::thread_rng();
            let mut artist_counters = if month == 0 {
                year_count.over_min_plays(min_listens)
            } else {
                year_count
                    .month_count(month)
                    .map(|month_count| month_count.over_min_plays(min_listens))
                    .unwrap_or_default()
            };

            if artist_counters.is_empty() {
                vec!["No artists found".to_string()]
            } else {
                let mut names: Vec<String> = Vec::new();

                for _ in 0..num_artists {
                    if artist_counters.is_empty() {
                        break;
                    }
                    let index = rng.gen_range(0..artist_counters.len());
                    let artist_counter = artist_counters.remove(index);
                    names.push(artist_counter.artist_name.to_string());
                }

                names
            }
        })
        .unwrap_or_else(|| vec!["No listens for that year".to_string()])
}

fn prompt_artist_songs(processor: &EventProcessor) -> Vec<String> {
    let artist_name = ArtistName(prompt("What artist do you want to look for? > "));

    processor
        .artist_song_counter(&artist_name)
        .map(|artist_counter| {
            artist_counter
                .play_details
                .all_song_plays()
                .iter()
                .map(|song_play| song_play.0.clone())
                .collect()
        })
        .unwrap_or_default()
}

async fn interactive(store: SqliteStore) {
    println!("Loading...");
    let mut processor = EventProcessor::default();
    let event_stream = store.get_events("listens".to_string()).await.unwrap();
    for event in event_stream.events.iter() {
        processor.process_event(event);
    }

    loop {
        let input = prompt("(q to exit, c for command list) > ");

        if input == "q" {
            break;
        } else if input == "random artist" {
            let artist_names = prompt_random_artist(&processor);
            for name in artist_names {
                println!(">> {}", name);
            }
        } else if input == "artist songs" {
            let song_names = prompt_artist_songs(&processor);
            for name in song_names {
                println!(">> {}", name);
            }
        } else if input == "c" {
            println!(">> random artist -> search for a random artist from your listening history.");
            println!(">> artist songs -> list out all of the songs for a single artist from your listening history.");
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
        _ => generate_all_stats(output_folder, count, store, split_monthly).await,
    }
}

async fn generate_all_stats(
    output_folder: &str,
    count: usize,
    store: SqliteStore,
    split_monthly: bool,
) {
    let folder = Folder {
        output_folder: output_folder.to_string(),
        year: None,
        month: None,
    };
    let event_stream = store.get_events("listens".to_string()).await.unwrap();
    let mut processor = EventProcessor::default();

    for event in event_stream.events.iter() {
        processor.process_event(event);
    }

    write_artists_counts(&folder, &processor.artists_counts, count).await;

    if split_monthly {
        for year_count in processor.year_counts() {
            let year_folder = Folder {
                output_folder: output_folder.to_string(),
                year: Some(year_count.year),
                month: None,
            };
            year_folder.create_if_necessary();
            write_artists_counts(&year_folder, &year_count.artists_counts, count).await;

            if split_monthly {
                for month_count in year_count.month_counts() {
                    let folder = Folder {
                        output_folder: output_folder.to_string(),
                        year: Some(year_count.year),
                        month: Some(month_count.month),
                    };

                    folder.create_if_necessary();
                    write_artists_counts(&folder, &month_count.artists_counts, count).await;
                }
            }
        }
    }
}

async fn generate_stats_for_single_year(
    output_folder: &str,
    count: usize,
    store: SqliteStore,
    year: i32,
    split_monthly: bool,
) {
    let event_stream = store.get_events("listens".to_string()).await.unwrap();

    let mut processor = EventProcessor::default();
    for event in event_stream.events.iter() {
        processor.process_event(event);
    }

    for year_count in processor
        .year_counts()
        .iter()
        .filter(|year_count| year_count.year == year)
    {
        let year_folder = Folder {
            output_folder: output_folder.to_string(),
            year: Some(year),
            month: None,
        };
        year_folder.create_if_necessary();
        write_artists_counts(&year_folder, &year_count.artists_counts, count).await;

        if split_monthly {
            for month_count in year_count.month_counts() {
                let folder = Folder {
                    output_folder: output_folder.to_string(),
                    year: Some(year_count.year),
                    month: Some(month_count.month),
                };

                folder.create_if_necessary();
                write_artists_counts(&folder, &month_count.artists_counts, count).await;
            }
        }
    }
}

async fn write_artists_counts(stats_folder: &Folder, stats: &ArtistsCounts, count: usize) {
    stats_folder.create_if_necessary();

    FileWriter::yaml_writer(stats_folder.file_name(&FileName::General))
        .write(&stats.general_stats(count))
        .await
        .unwrap();
    FileWriter::from(stats_folder.file_name(&FileName::Complete))
        .write(&stats.all())
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

        let track_plays = match read_track_plays(&path) {
            Ok(it) => it,
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        };

        for track_play in track_plays.iter() {
            let command = AddTrackPlay {
                track_play: track_play.clone(),
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
