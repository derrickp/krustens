use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageSet {
    pub title: String,
    pub messages: Vec<String>,
}
