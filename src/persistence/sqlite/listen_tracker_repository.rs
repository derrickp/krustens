use sqlx::{Pool, Sqlite};
use tokio::sync::Mutex;

use crate::{
    processing::events::{Event, EventData},
    projections::{build_id, ListenTracker, ListenTrackerRepository},
};

use std::{collections::HashSet, sync::Arc};

use crate::{errors::ReadError, persistence::EventStore};

pub struct SqliteListenTrackerRepository {
    pool: Pool<Sqlite>,
    listen_tracker: ListenTracker,
    dirty: bool,
    buffer_count: usize,
    not_persisted_count: usize,
}

#[async_trait::async_trait]
impl ListenTrackerRepository for SqliteListenTrackerRepository {
    fn get(&self) -> &ListenTracker {
        &self.listen_tracker
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

    async fn project_event(&mut self, event: &Event) {
        match &event.data {
            EventData::TrackPlayAdded(listen) => {
                let id = build_id(&listen.artist_name, &listen.track_name, &listen.end_time);
                self.listen_tracker.version += 1;
                self.listen_tracker.listens.insert(id)
            }
            EventData::TrackPlayIgnored(ignored) => {
                let id = build_id(&ignored.artist_name, &ignored.track_name, &ignored.end_time);
                self.listen_tracker.version += 1;
                self.listen_tracker.listens.insert(id)
            }
        };
        self.not_persisted_count += 1;

        self.dirty = true;

        if self.not_persisted_count >= self.buffer_count {
            SqliteListenTrackerRepository::write(&self.pool, &self.listen_tracker)
                .await
                .unwrap();
            self.reset_persistence();
        }
    }
}

impl SqliteListenTrackerRepository {
    async fn read(pool: &Pool<Sqlite>) -> Result<ListenTracker, ReadError> {
        let query = "select data, version from snapshots where name = 'listen_tracker'";

        let row: Option<(String, u32)> = sqlx::query_as(query).fetch_optional(pool).await.unwrap();

        match row {
            Some((data, version)) => match serde_json::from_str(&data) {
                Ok(it) => Ok(ListenTracker {
                    listens: it,
                    version,
                }),
                Err(e) => Err(ReadError::FailedToDeserializeJson {
                    message: e.to_string(),
                    file_name: "".to_string(),
                }),
            },
            None => Ok(ListenTracker {
                listens: HashSet::new(),
                version: 0,
            }),
        }
    }

    async fn write(
        pool: &Pool<Sqlite>,
        value: &ListenTracker,
    ) -> Result<bool, crate::errors::WriteError> {
        let query = "insert or replace into snapshots (name, version, data) values ($1, $2, $3)";

        let serialized = serde_json::to_string(&value.listens).unwrap();

        sqlx::query(query)
            .bind("listen_tracker")
            .bind(value.version)
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
    store: Arc<Mutex<dyn EventStore + Send + Sync>>,
) -> SqliteListenTrackerRepository {
    let listen_tracker = SqliteListenTrackerRepository::read(pool).await.unwrap();
    let current_version = listen_tracker.version;
    let mut repository = SqliteListenTrackerRepository {
        pool: pool.clone(),
        listen_tracker,
        dirty: false,
        buffer_count,
        not_persisted_count: 0,
    };

    let event_store = store.lock().await;
    let store_version = event_store.stream_version("listens").await;

    if current_version == store_version {
        return repository;
    }

    let event_stream = event_store
        .get_events_after("listens", current_version)
        .await
        .unwrap();

    for event in event_stream.events.iter() {
        repository.project_event(event).await;
    }
    repository.flush().await;
    repository
}
