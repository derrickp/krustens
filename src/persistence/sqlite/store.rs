use async_trait::async_trait;
use sqlx::{sqlite::SqliteRow, Pool, Row, Sqlite};

use crate::{
    errors::{AddEventError, GetEventsError},
    persistence::EventStore,
    processing::events::{Event, EventData, EventStream},
};

pub struct SqliteEventStore {
    pool: Pool<Sqlite>,
}

impl From<Pool<Sqlite>> for SqliteEventStore {
    fn from(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStore for SqliteEventStore {
    async fn stream_version(&self, stream: &str) -> u32 {
        let query = sqlx::query("select MAX(position) from streams where stream = $1").bind(stream);

        let row: Option<u32> = match query
            .map(|row: SqliteRow| row.get(0))
            .fetch_one(&self.pool)
            .await
        {
            Ok(it) => it,
            Err(e) => {
                println!("{e:?}");
                None
            }
        };

        row.unwrap_or(0)
    }

    async fn add_event(
        &mut self,
        stream: &str,
        event: Event,
        expected_version: u32,
    ) -> Result<Event, AddEventError> {
        let current_version = self.stream_version(stream).await;

        if expected_version <= current_version {
            return Err(AddEventError::VersionOutOfDate {
                expected_version,
                current_version,
            });
        }

        let data = serde_json::to_string(&event.data).unwrap();

        let query = "insert into streams (stream, position, data) values ($1, $2, $3)";
        sqlx::query(query)
            .bind(stream)
            .bind(event.version)
            .bind(&data)
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(event)
    }

    async fn get_events(&self, stream: &str) -> Result<EventStream, GetEventsError> {
        let query = "select data, position from streams where stream = $1";

        let result = sqlx::query(query)
            .bind(stream)
            .map(|row: SqliteRow| {
                let data: String = row.try_get("data").unwrap();
                let version = row.try_get("position").unwrap();
                let event_data: EventData = serde_json::from_str(&data).unwrap();
                Event {
                    version,
                    data: event_data,
                }
            })
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(events) => {
                let version = events.len() as u32;

                Ok(EventStream { events, version })
            }
            Err(e) => Err(GetEventsError::UnableToReadStream {
                message: e.to_string(),
                event_source: "sqlite".to_string(),
            }),
        }
    }

    async fn get_events_after(
        &self,
        stream: &str,
        version: u32,
    ) -> Result<EventStream, GetEventsError> {
        let query = "select data, position from streams where stream = $1 and position > $2";

        let result = sqlx::query(query)
            .bind(stream)
            .bind(version)
            .map(|row: SqliteRow| {
                let data: String = row.try_get("data").unwrap();
                let version = row.try_get("position").unwrap();
                let event_data: EventData = serde_json::from_str(&data).unwrap();
                Event {
                    version,
                    data: event_data,
                }
            })
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(events) => {
                let current_version = events.len() as u32;
                Ok(EventStream {
                    events,
                    version: current_version,
                })
            }
            Err(e) => Err(GetEventsError::UnableToReadStream {
                message: e.to_string(),
                event_source: "sqlite".to_string(),
            }),
        }
    }
}
