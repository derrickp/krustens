mod cli;
mod errors;
mod events;
mod generation;
mod interactive;
mod persistence;
mod processing;
mod projections;
mod track_plays;
mod utils;

use std::sync::Arc;

use clap::Parser;
use generation::generate_stats;
use interactive::full_ui;
use persistence::sqlite::{listen_tracker_repo, DatabaseConfig, SqliteEventStore};

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
            full_ui(Arc::new(SqliteEventStore::from(pool.clone())))
                .await
                .unwrap();
            Ok(())
        }
    }
}
