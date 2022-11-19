mod cli;
mod errors;
mod events;
mod generation;
mod persistence;
mod processing;
mod projections;
mod track_plays;
mod utils;

use std::{io::Write, sync::Arc};

use clap::Parser;
use generation::generate_stats;
use persistence::sqlite::{listen_tracker_repo, DatabaseConfig, SqliteEventStore};
use projections::statistics::EventProcessor;
use rand::Rng;

use crate::{persistence::EventStore, track_plays::ArtistName};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let database_file = "krustens.sqlite";
    let database_url = format!("sqlite://{}", database_file);
    let db_config = DatabaseConfig::from(database_url);
    let pool = persistence::sqlite::build_pool_and_migrate(db_config).await;
    let args = cli::Arguments::parse();

    match args.command {
        cli::Commands::Process(process_args) => {
            let store = Arc::new(SqliteEventStore::from(pool.clone()));
            let mut repository = listen_tracker_repo(1500, &pool, store.clone()).await;
            processing::process_listens(
                &process_args.input,
                Arc::new(SqliteEventStore::from(pool.clone())),
                &mut repository,
            )
            .await;
            Ok(())
        }
        cli::Commands::Generate(generate_args) => {
            generate_stats(
                &generate_args.output,
                generate_args.count,
                Arc::new(SqliteEventStore::from(pool.clone())),
                generate_args.year,
                generate_args.split_monthly,
            )
            .await;
            Ok(())
        }
        cli::Commands::Interactive => {
            interactive(SqliteEventStore::from(pool.clone())).await;
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

fn prompt_artists_on_day(processor: &EventProcessor) -> Vec<String> {
    let year = prompt("What year to look in? > ")
        .parse::<i32>()
        .expect("Error: Not a valid number");
    let month = prompt("What month do you want to look in (1-12)? > ")
        .parse::<u32>()
        .ok()
        .filter(|m| (1..=12).contains(m))
        .expect("Error: Not a valid month");
    let day = prompt("What day of the month? > ")
        .parse::<u32>()
        .ok()
        .filter(|d| (1..=31).contains(d))
        .expect("Error: Not a valid day");

    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .map(|date| {
            let names = processor
                .artists_on_day(date)
                .into_iter()
                .map(|artist_counter| artist_counter.total_plays_display())
                .collect::<Vec<String>>();

            if names.is_empty() {
                vec!["No artists listened to on that day".to_string()]
            } else {
                names
            }
        })
        .unwrap_or_else(|| vec!["Error: Invalid date".to_string()])
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

async fn interactive(store: SqliteEventStore) {
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
        } else if input == "random artists" {
            let artist_names = prompt_random_artist(&processor);
            for name in artist_names {
                println!(">> {}", name);
            }
        } else if input == "artist songs" {
            let song_names = prompt_artist_songs(&processor);
            for name in song_names {
                println!(">> {}", name);
            }
        } else if input == "artists on day" {
            let names = prompt_artists_on_day(&processor);
            for name in names {
                println!(">> {}", name);
            }
        } else if input == "c" {
            println!(">> random artists -> search for a random artist from your listening history");
            println!(">> artist songs -> list out all of the songs for a single artist from your listening history");
            println!(">> artists on day -> list all artists listened to on a specific day")
        }
    }
}
