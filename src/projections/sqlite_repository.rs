use sqlx::{Pool, Sqlite};

use super::{build_id, listen_tracker::ListenTracker, projection_repository::ProjectionRepository};

use std::collections::HashSet;

use async_trait::async_trait;

use crate::{events::EventData, persistence::ReadError, stores::EventStore};

pub struct SqliteListenTrackerRepository {
    pool: Pool<Sqlite>,
    listen_tracker: ListenTracker,
    dirty: bool,
    buffer_count: usize,
    not_persisted_count: usize,
}

#[async_trait]
impl ProjectionRepository<ListenTracker> for SqliteListenTrackerRepository {
    async fn get(&mut self, store: &(impl EventStore + Send + std::marker::Sync)) -> ListenTracker {
        let current_version = self.listen_tracker.version;
        let store_version = store.stream_version("listens".to_string()).await;

        if current_version == store_version {
            return self.listen_tracker.clone();
        }

        let event_stream = store
            .get_events_after("listens".to_string(), self.listen_tracker.version)
            .await
            .unwrap();

        for event in event_stream.events.iter() {
            match &event.data {
                EventData::TrackPlayAdded(listen) => {
                    let id = build_id(&listen.artist_name, &listen.track_name, &listen.end_time);
                    self.listen_tracker.version += 1;
                    self.listen_tracker.listens.insert(id.clone())
                }
                EventData::TrackPlayIgnored(ignored) => {
                    let id = build_id(&ignored.artist_name, &ignored.track_name, &ignored.end_time);
                    self.listen_tracker.version += 1;
                    self.listen_tracker.listens.insert(id.clone())
                }
            };
            self.not_persisted_count += 1;
        }
        self.dirty = true;

        if self.not_persisted_count >= self.buffer_count {
            SqliteListenTrackerRepository::write(&self.pool, &self.listen_tracker)
                .await
                .unwrap();
            self.reset_persistence();
        }

        self.listen_tracker.clone()
    }

    async fn flush(&mut self) {
        if !self.dirty {
            return;
        }

        SqliteListenTrackerRepository::write(&self.pool, &self.listen_tracker)
            .await
            .unwrap();
        self.reset_persistence();
    }
}

impl SqliteListenTrackerRepository {
    async fn read(pool: &Pool<Sqlite>) -> Result<ListenTracker, ReadError> {
        let query = "select data, version from snapshots where name = 'listen_tracker'";

        let row: Option<(String, u32)> = sqlx::query_as(query).fetch_optional(pool).await.unwrap();

        match row {
            Some((data, version)) => {
                let listens: HashSet<String> = serde_json::from_str(&data).unwrap();

                Ok(ListenTracker { listens, version })
            }
            None => Ok(ListenTracker {
                listens: HashSet::new(),
                version: 0,
            }),
        }
    }

    async fn write(
        pool: &Pool<Sqlite>,
        value: &ListenTracker,
    ) -> Result<bool, crate::persistence::WriteError> {
        let query = "insert or replace into snapshots (name, version, data) values ($1, $2, $3)";

        let serialized = serde_json::to_string(&value.listens).unwrap();

        sqlx::query(query)
            .bind("listen_tracker")
            .bind(value.version.clone())
            .bind(&serialized)
            .execute(pool)
            .await
            .unwrap();

        Ok(true)
    }

    fn reset_persistence(&mut self) {
        self.dirty = false;
        self.not_persisted_count = 0;
    }
}

pub async fn listen_tracker_repo(
    buffer_count: usize,
    pool: &Pool<Sqlite>,
) -> impl ProjectionRepository<ListenTracker> {
    let listen_tracker = SqliteListenTrackerRepository::read(&pool).await.unwrap();

    SqliteListenTrackerRepository {
        pool: pool.clone(),
        listen_tracker,
        dirty: false,
        buffer_count,
        not_persisted_count: 0,
    }
}
