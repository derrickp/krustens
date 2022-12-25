use serde::{Deserialize, Serialize};

use super::HasId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageSet {
    id: String,
    title: String,
    messages: Vec<String>,
}

impl HasId for MessageSet {
    fn id(&self) -> &str {
        &self.id
    }
}

impl MessageSet {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn messages(&self) -> &Vec<String> {
        &self.messages
    }

    pub fn append_messages(&mut self, messages: &mut Vec<String>) {
        self.messages.append(messages);
    }

    pub fn push_message(&mut self, message: &str) {
        self.messages.push(message.to_string())
    }

    pub fn with_messages(title: &str, messages: Vec<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            messages,
        }
    }
}
