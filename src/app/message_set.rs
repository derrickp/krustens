use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct MessageSet {
    pub title: String,
    pub messages: Vec<String>,
}
