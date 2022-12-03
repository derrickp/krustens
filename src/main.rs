mod errors;
mod events;
mod interactive;
mod persistence;
mod processing;
mod projections;
mod track_plays;
mod utils;

use std::sync::Arc;

use interactive::full_ui;
use persistence::sqlite::{listen_tracker_repo, DatabaseConfig, SqliteEventStore};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let database_file = "krustens.sqlite";
    let database_url = format!("sqlite://{}", database_file);
    let db_config = DatabaseConfig::from(database_url);
    let pool = persistence::sqlite::build_pool_and_migrate(db_config).await;

    let store = Arc::new(SqliteEventStore::from(pool.clone()));
    let repository = Arc::new(Mutex::new(
        listen_tracker_repo(20_000, &pool, store.clone()).await,
    ));
    full_ui(store, repository).await.unwrap();
    Ok(())
}
