use sqlx::migrate::MigrateDatabase;

use super::errors::PersistenceError;

pub async fn bootstrap() -> Result<String, PersistenceError> {
    let exists = match sqlx::Sqlite::database_exists("sqlite:krustens.db").await {
        Ok(it) => it,
        Err(e) => return Err(PersistenceError::General(e.to_string())),
    };

    if !exists {
        if let Err(e) = sqlx::Sqlite::create_database("sqlite:krustens.db").await {
            return Err(PersistenceError::General(e.to_string()));
        }
    }

    Ok("sqlite:krustens.db".to_string())
}
