pub struct DatabaseConfig {
    pub database_url: String,
}

impl From<&str> for DatabaseConfig {
    fn from(value: &str) -> Self {
        Self {
            database_url: value.to_string(),
        }
    }
}

impl From<String> for DatabaseConfig {
    fn from(value: String) -> Self {
        Self {
            database_url: value,
        }
    }
}
