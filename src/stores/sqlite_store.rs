use async_trait::async_trait;
use sqlx::{sqlite::SqliteRow, Pool, Row, Sqlite};

use crate::events::{Event, EventData};

use super::{event_store::EventStore, AddEventError, EventStream, GetEventsError};

pub struct SqliteStore {
    pool: Pool<Sqlite>,
}

impl SqliteStore {
    pub fn build(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStore for SqliteStore {
    async fn stream_version(&self, stream: String) -> u32 {
        let query =
            sqlx::query("select MAX(position) from streams where stream = $1").bind(&stream);

        let row: Option<u32> = match query
            .map(|row: SqliteRow| row.get(0))
            .fetch_one(&self.pool)
            .await
        {
            Ok(it) => it,
            Err(e) => {
                println!("{:?}", e);
                None
            }
        };

        row.unwrap_or(0)
    }

    async fn add_event(
        &self,
        stream: String,
        event: &Event,
        expected_version: u32,
    ) -> Result<(), AddEventError> {
        let current_version = self.stream_version(stream.clone()).await;

        if expected_version <= current_version as u32 {
            return Err(AddEventError);
        }

        let data = serde_json::to_string(&event.data).unwrap();

        let query = "insert into streams (stream, position, data) values ($1, $2, $3)";
        sqlx::query(query)
            .bind(&stream)
            .bind(event.version)
            .bind(&data)
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }

    async fn get_events(&self, stream: String) -> Result<EventStream, GetEventsError> {
        let query = "select data, position from streams where stream = $1";

        let events: Vec<Event> = sqlx::query(query)
            .bind(&stream)
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
            .await
            .unwrap();

        let version = events.len() as u32;

        Ok(EventStream { events, version })
    }

    async fn get_events_after(
        &self,
        stream: String,
        version: u32,
    ) -> Result<EventStream, GetEventsError> {
        let query = "select data, position from streams where stream = $1 and position > $2";

        let events: Vec<Event> = sqlx::query(query)
            .bind(&stream)
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
            .await
            .unwrap();

        let current_version = events.len() as u32;
        Ok(EventStream {
            events,
            version: current_version,
        })
    }
}
