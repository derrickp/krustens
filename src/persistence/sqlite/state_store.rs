use sqlx::{Pool, Sqlite};

use crate::{
    app::State,
    errors::{ReadError, WriteError},
    persistence::StateStore,
};

pub struct SqliteStateStore {
    pool: Pool<Sqlite>,
}

impl From<Pool<Sqlite>> for SqliteStateStore {
    fn from(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl StateStore for SqliteStateStore {
    async fn push(&mut self, state: &State) -> Result<(), WriteError> {
        let query = "insert or replace into snapshots (name, version, data) values ($1, $2, $3)";

        let serialized = serde_json::to_string(&state).unwrap();

        sqlx::query(query)
            .bind("app_state")
            .bind(1)
            .bind(&serialized)
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }
    async fn get(&self) -> Result<State, ReadError> {
        let query = "select data, version from snapshots where name = 'app_state'";

        let row: Option<(String, u32)> = sqlx::query_as(query)
            .fetch_optional(&self.pool)
            .await
            .unwrap();

        match row {
            Some((data, _)) => match serde_json::from_str(&data) {
                Ok(it) => Ok(it),
                Err(e) => Err(ReadError::FailedToDeserializeJson {
                    message: e.to_string(),
                    file_name: "".to_string(),
                }),
            },
            None => Ok(State::default()),
        }
    }
}
