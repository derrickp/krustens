use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct AppMessageSet {
    pub title: String,
    pub messages: Vec<String>,
}
