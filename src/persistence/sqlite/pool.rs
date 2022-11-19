use std::str::FromStr;

use sqlx::{sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous}, Pool, Sqlite};

pub async fn build_pool_and_migrate(config: super::DatabaseConfig) -> Pool<Sqlite> {
    let connection_options = SqliteConnectOptions::from_str(&config.database_url)
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

    pool
}
